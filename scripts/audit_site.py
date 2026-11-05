import concurrent.futures
import json
import os
import re
import sys
import time
import urllib.error
import urllib.request
import xml.etree.ElementTree as ET

JAVA_VNU_URL = os.environ.get("JAVA_VNU_URL", "http://127.0.0.1:8888/?out=json")
RUST_VNU_URL = os.environ.get("RUST_VNU_URL", "http://127.0.0.1:8889/?out=json")


def get_urls_from_sitemaps(sitemaps):
    urls = []
    ns = {"s": "http://www.sitemaps.org/schemas/sitemap/0.9"}
    for sm in sitemaps:
        try:
            req = urllib.request.Request(
                sm,
                headers={"User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)"},
            )
            with urllib.request.urlopen(req, timeout=10) as resp:
                xml_content = resp.read()
            root = ET.fromstring(xml_content)
            for elem in root.findall(".//s:loc", ns):
                if elem.text:
                    urls.append(elem.text.strip())
        except Exception as e:
            print(f"Error fetching sitemap {sm}: {e}", file=sys.stderr)

    # Deduplicate while preserving order
    seen = set()
    dedup = []
    for u in urls:
        if u not in seen:
            seen.add(u)
            dedup.append(u)
    return dedup


def validate(endpoint, html_bytes):
    t0 = time.perf_counter()
    req = urllib.request.Request(
        endpoint,
        data=html_bytes,
        headers={"Content-Type": "text/html; charset=utf-8"},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=30) as resp:
        body = resp.read()
    t1 = time.perf_counter()
    data = json.loads(body.decode("utf-8", errors="replace"))
    return data.get("messages", []), (t1 - t0) * 1000.0


def normalize_msg(m):
    msg_type = m.get("type", "error")
    sub_type = m.get("subType", "")
    full_type = f"{msg_type}/{sub_type}" if sub_type else msg_type
    raw_text = m.get("message", "").strip()
    return full_type, raw_text


def process_url(url):
    req = urllib.request.Request(
        url,
        headers={
            "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        },
    )
    try:
        with urllib.request.urlopen(req, timeout=15) as resp:
            content_type = resp.headers.get("Content-Type", "")
            if "text/html" not in content_type:
                return None
            html_bytes = resp.read()
    except Exception as e:
        return {"url": url, "error": str(e)}

    # Send to both validators
    try:
        java_msgs, java_time = validate(JAVA_VNU_URL, html_bytes)
    except Exception as e:
        java_msgs, java_time = [{"type": "internal-error", "message": str(e)}], 0.0

    try:
        rust_msgs, rust_time = validate(RUST_VNU_URL, html_bytes)
    except Exception as e:
        rust_msgs, rust_time = [{"type": "internal-error", "message": str(e)}], 0.0

    j_signatures = [normalize_msg(m) for m in java_msgs]
    r_signatures = [normalize_msg(m) for m in rust_msgs]
    r_rem = list(r_signatures)

    both = []
    java_only = []
    for sig in j_signatures:
        if sig in r_rem:
            r_rem.remove(sig)
            both.append(sig)
        else:
            java_only.append(sig)
    rust_only = r_rem

    return {
        "url": url,
        "size_kb": len(html_bytes) / 1024.0,
        "java_time": java_time,
        "rust_time": rust_time,
        "both": both,
        "java_only": java_only,
        "rust_only": rust_only,
    }


def main():
    sitemaps = sys.argv[1:]
    if not sitemaps and os.environ.get("SITEMAP_URL"):
        sitemaps = [os.environ["SITEMAP_URL"]]

    if not sitemaps:
        print("Usage: python3 scripts/audit_site.py <sitemap_url_1> [sitemap_url_2 ...]")
        print("   or: SITEMAP_URL=https://example.com/sitemap.xml python3 scripts/audit_site.py")
        sys.exit(0)

    print("Fetching sitemaps...")
    urls = get_urls_from_sitemaps(sitemaps)
    print(f"Discovered {len(urls)} URLs")

    if not urls:
        print("No URLs found in provided sitemaps.")
        return

    start_time = time.perf_counter()
    max_workers = 10
    completed = 0
    total = len(urls)

    tot_pages = 0
    tot_size_kb = 0.0
    tot_java_time = 0.0
    tot_rust_time = 0.0

    java_only_counts = {}
    rust_only_counts = {}
    both_counts = {}

    print(f"Starting crawl & dual-validation with {max_workers} threads...")
    with concurrent.futures.ThreadPoolExecutor(max_workers=max_workers) as executor:
        future_to_url = {executor.submit(process_url, u): u for u in urls}
        for future in concurrent.futures.as_completed(future_to_url):
            completed += 1
            res = future.result()
            if res and "size_kb" in res:
                tot_pages += 1
                tot_size_kb += res["size_kb"]
                tot_java_time += res["java_time"]
                tot_rust_time += res["rust_time"]
                u = res["url"]
                for sig in res["both"]:
                    both_counts.setdefault(sig, []).append(u)
                for sig in res["java_only"]:
                    java_only_counts.setdefault(sig, []).append(u)
                for sig in res["rust_only"]:
                    rust_only_counts.setdefault(sig, []).append(u)

            if completed % 25 == 0 or completed == total:
                elapsed = time.perf_counter() - start_time
                print(f"[{completed}/{total}] pages processed ({elapsed:.1f}s)...")

    total_time = time.perf_counter() - start_time
    print(f"\nCompleted crawling & validating {tot_pages} HTML pages in {total_time:.2f}s!")

    print("\n--- PERFORMANCE ---")
    print(f"Total HTML: {tot_size_kb / 1024.0:.2f} MB across {tot_pages} pages")
    if tot_pages > 0:
        print(f"Java validator total validation time: {tot_java_time / 1000.0:.2f}s (avg {tot_java_time / tot_pages:.2f} ms/page)")
        print(f"Rust vnu-rs total validation time:    {tot_rust_time / 1000.0:.2f}s (avg {tot_rust_time / tot_pages:.2f} ms/page)")
    if tot_rust_time > 0:
        print(f"Speedup: {tot_java_time / tot_rust_time:.2f}x faster!")

    print("\n" + "=" * 80)
    print("MESSAGES DETECTED BY BOTH VALIDATORS (PERFECT PARITY):")
    print("=" * 80)
    for (t, msg), page_list in sorted(both_counts.items(), key=lambda x: -len(x[1])):
        print(f"[{t}] (on {len(page_list)} pages, {len(page_list)} occurrences): {msg[:100]}...")

    print("\n" + "=" * 80)
    print("MESSAGES FOUND ONLY BY JAVA VNU (MISSED BY RUST VNU-RS):")
    print("=" * 80)
    for (t, msg), page_list in sorted(java_only_counts.items(), key=lambda x: -len(x[1])):
        print(f"- [{t}] (found on {len(set(page_list))} pages, {len(page_list)} total times):")
        print(f"  Message: {msg}")
        print(f"  Sample page: {page_list[0]}")

    print("\n" + "=" * 80)
    print("MESSAGES FOUND ONLY BY RUST VNU-RS (FALSE POSITIVES IN RUST):")
    print("=" * 80)
    for (t, msg), page_list in sorted(rust_only_counts.items(), key=lambda x: -len(x[1])):
        print(f"- [{t}] (found on {len(set(page_list))} pages, {len(page_list)} total times):")
        print(f"  Message: {msg}")
        print(f"  Sample page: {page_list[0]}")

    summary_path = os.environ.get(
        "AUDIT_OUTPUT_PATH",
        os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "artifacts", "site_diff.json")),
    )
    os.makedirs(os.path.dirname(os.path.abspath(summary_path)), exist_ok=True)
    summary_data = {
        "total_pages": tot_pages,
        "total_html_mb": tot_size_kb / 1024.0,
        "performance": {
            "java_total_sec": tot_java_time / 1000.0,
            "rust_total_sec": tot_rust_time / 1000.0,
            "speedup": tot_java_time / tot_rust_time if tot_rust_time else 0,
        },
        "both": [{"type": t, "msg": m, "count": len(u_list), "sample_url": u_list[0]} for (t, m), u_list in both_counts.items()],
        "java_only": [{"type": t, "msg": m, "count": len(u_list), "pages": len(set(u_list)), "sample_url": u_list[0]} for (t, m), u_list in java_only_counts.items()],
        "rust_only": [{"type": t, "msg": m, "count": len(u_list), "pages": len(set(u_list)), "sample_url": u_list[0]} for (t, m), u_list in rust_only_counts.items()],
    }
    with open(summary_path, "w") as f:
        json.dump(summary_data, f, indent=2)
    print(f"\nScan summary written to: {summary_path}")


if __name__ == "__main__":
    main()
