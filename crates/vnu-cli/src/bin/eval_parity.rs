use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::PathBuf;

use std::time::Instant;
use vnu_parser::ValidatorEngine;

fn normalize(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_control() { '?' } else { c })
        .collect()
}

fn resolve_upstream_path() -> Result<PathBuf, String> {
    if let Ok(val) = std::env::var("VNU_UPSTREAM_TESTS") {
        let p = PathBuf::from(val);
        if p.exists() {
            return Ok(p);
        }
    }
    for candidate in &[
        "../validator-upstream/tests",
        "../../validator-upstream/tests",
        "tests",
    ] {
        let p = PathBuf::from(candidate);
        if p.exists() {
            return Ok(p);
        }
    }
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let p = PathBuf::from(manifest_dir).join("../../validator-upstream/tests");
        if p.exists() {
            return Ok(p);
        }
    }
    Err("Upstream tests directory not found. Set VNU_UPSTREAM_TESTS or run next to validator-upstream.".to_string())
}

fn main() {
    let upstream_tests = match resolve_upstream_path() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let messages_file = upstream_tests.join("messages.json");
    let content = fs::read_to_string(&messages_file)
        .expect("Failed to read tests/messages.json");
    let expected_messages: HashMap<String, String> = serde_json::from_str(&content)
        .expect("Failed to parse messages.json");

    let filter = std::env::args().nth(1);

    let start = Instant::now();
    let mut engine = ValidatorEngine::default();

    let mut total = 0;
    let mut matches = 0;
    let mut skipped: Vec<String> = Vec::new();
    let mut cat_total: BTreeMap<String, usize> = BTreeMap::new();
    let mut cat_match: BTreeMap<String, usize> = BTreeMap::new();

    let mut keys: Vec<String> = expected_messages.keys().cloned().collect();
    keys.sort();

    for key in &keys {
        if let Some(ref f) = filter {
            if !key.contains(f) {
                continue;
            }
        }

        let file_path = upstream_tests.join(key);
        let src = match fs::read_to_string(&file_path) {
            Ok(s) => s,
            Err(_) => {
                match fs::read(&file_path) {
                    Ok(b) => String::from_utf8_lossy(&b).to_string(),
                    Err(e) => {
                        skipped.push(format!("{}: {}", key, e));
                        continue;
                    }
                }
            }
        };

        total += 1;
        let cat = key.split('/').take(3).collect::<Vec<_>>().join("/");
        *cat_total.entry(cat.clone()).or_default() += 1;

        let report = engine.validate(&src);
        let exp = &expected_messages[key];
        let exp_norm = normalize(exp);

        let found = report.messages.iter().any(|m| {
            let msg_norm = normalize(&m.message);
            msg_norm == exp_norm || msg_norm.contains(&exp_norm) || exp_norm.contains(&msg_norm)
        });

        if found {
            matches += 1;
            *cat_match.entry(cat).or_default() += 1;
        } else if filter.is_some() {
            println!("FAILED: {}\nEXPECTED: {}\nGOT:", key, exp);
            for m in &report.messages {
                println!("  - {}", m.message);
            }
        }
    }

    let elapsed = start.elapsed();
    println!("============================================================");
    println!("VNU-RS PARITY EVALUATION REPORT");
    println!("============================================================");
    println!("Evaluated in {:.2?}", elapsed);
    if !skipped.is_empty() {
        println!("Skipped files: {}", skipped.len());
        for s in skipped.iter().take(5) {
            println!("  [SKIP] {}", s);
        }
    }
    if total > 0 {
        println!("Overall Match: {} / {} ({:.2}%)", matches, total, (matches as f64 / total as f64) * 100.0);
    }
    println!("------------------------------------------------------------");
    for (cat, count) in &cat_total {
        let matched = cat_match.get(cat).copied().unwrap_or(0);
        let pct = (matched as f64 / *count as f64) * 100.0;
        if count >= &5 || filter.is_some() {
            println!("{:40} : {:4} / {:4} ({:6.2}%)", cat, matched, count, pct);
        }
    }
    println!("============================================================");
}
