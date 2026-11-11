use super::*;

fn run(html: &str) -> Vec<String> {
    let mut tb = TreeBuilder::default();
    // Minimal driver: tags only, positions ignored.
    let mut i = 0;
    let b = html.as_bytes();
    while i < b.len() {
        if b[i] == b'<' {
            let end = html[i..].find('>').map(|e| i + e).unwrap_or(b.len());
            let inner = &html[i + 1..end];
            if let Some(rest) = inner.strip_prefix('!') {
                if rest.to_ascii_lowercase().starts_with("doctype") {
                    tb.doctype(Some("html"), None, None, false, 1, i + 1);
                }
            } else if let Some(name) = inner.strip_prefix('/') {
                tb.end_tag(&name.trim().to_ascii_lowercase(), 1, i + 1);
            } else {
                let self_closing = inner.ends_with('/');
                let inner = inner.trim_end_matches('/');
                let mut parts = inner.split_whitespace();
                let name = parts.next().unwrap_or("").to_ascii_lowercase();
                let attrs: Vec<(String, String)> = parts
                    .map(|p| {
                        let mut kv = p.splitn(2, '=');
                        (
                            kv.next().unwrap_or("").to_string(),
                            kv.next().unwrap_or("").trim_matches('"').to_string(),
                        )
                    })
                    .collect();
                tb.start_tag(&name, attrs.into(), self_closing, 1, i + 1);
            }
            i = end + 1;
        } else {
            let end = html[i..].find('<').map(|e| i + e).unwrap_or(b.len());
            tb.characters(&html[i..end], 1, i + 1);
            i = end;
        }
    }
    tb.eof(1, html.len());
    let mut msgs = Vec::new();
    tb.take_messages(&mut msgs);
    msgs.into_iter().map(|m| m.message).collect()
}

const HEAD: &str = "<!DOCTYPE html><html lang=en><head><title>t</title></head><body>";

#[test]
fn valid_page_has_no_errors() {
    let msgs = run(&format!("{HEAD}<p>x</p><ul><li>a<li>b</ul></body></html>"));
    assert!(msgs.is_empty(), "{msgs:?}");
}

#[test]
fn start_tag_in_table_is_fatal() {
    let msgs = run(&format!("{HEAD}<table><p>x</p></table><img></body></html>"));
    assert_eq!(
        msgs,
        vec![
            "Start tag “p” seen in “table”.",
            "Cannot recover after last error. Any further errors will be ignored."
        ]
    );
}

#[test]
fn implied_p_end_lists_unclosed_elements() {
    let msgs = run(&format!("{HEAD}<p><b>x<p>y</b></body></html>"));
    assert_eq!(
        msgs,
        vec![
            "End tag “p” implied, but there were open elements.",
            "Unclosed element “b”."
        ]
    );
}

#[test]
fn optional_end_tags_are_silent() {
    let msgs = run(&format!("{HEAD}<div><p>x</div></body></html>"));
    assert!(msgs.is_empty(), "{msgs:?}");
}

#[test]
fn end_tag_with_open_children() {
    let msgs = run(&format!("{HEAD}<div><span>x</div></body></html>"));
    assert_eq!(
        msgs,
        vec![
            "End tag “div” seen, but there were open elements.",
            "Unclosed element “span”."
        ]
    );
}

#[test]
fn stray_end_tags() {
    let msgs = run("<html><body></div></body></html>");
    assert_eq!(
        msgs,
        vec![
            "Start tag seen without seeing a doctype first. Expected “<!DOCTYPE html>”.",
            "Stray end tag “div”."
        ]
    );
}

#[test]
fn start_tag_after_html_end_is_fatal() {
    let msgs = run(&format!("{HEAD}<p>x</body></html><p>after"));
    assert_eq!(
        msgs,
        vec![
            "Stray start tag “p”.",
            "Cannot recover after last error. Any further errors will be ignored."
        ]
    );
}

#[test]
fn duplicate_ids_point_at_the_first() {
    let msgs = run(&format!("{HEAD}<p id=x></p><b id=x></b></body></html>"));
    assert_eq!(
        msgs,
        vec![
            "Duplicate ID “x”.",
            "The first occurrence of ID “x” was here."
        ]
    );
}
