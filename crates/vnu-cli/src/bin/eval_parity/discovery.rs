use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub fn resolve_upstream_path() -> Result<PathBuf, String> {
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

pub fn collect_valid_files(
    dir: &Path,
    base: &Path,
    known_messages: &HashSet<String>,
    out: &mut Vec<String>,
    skipped: &mut Vec<String>,
) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(err) => {
            skipped.push(format!(
                "directory read error on {}: {}",
                dir.display(),
                err
            ));
            return;
        }
    };
    for res in entries {
        let entry = match res {
            Ok(e) => e,
            Err(e) => {
                skipped.push(format!("entry read error in {}: {}", dir.display(), e));
                continue;
            }
        };
        let path = entry.path();
        if path.is_dir() {
            collect_valid_files(&path, base, known_messages, out, skipped);
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if !matches!(ext, "html" | "xhtml" | "htm" | "xht") {
                continue;
            }
            if path.file_name().and_then(|n| n.to_str()) == Some("template.html") {
                continue;
            }
            let Ok(rel) = path.strip_prefix(base) else {
                continue;
            };
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            if known_messages.contains(&rel_str)
                || rel_str.contains("novalid")
                || rel_str.contains("haswarn")
                || rel_str.contains("hasinfo")
                || rel_str.split('/').any(|p| p == "invalid")
            {
                continue;
            }
            out.push(rel_str);
        }
    }
}

pub fn load_test_cases(
    upstream_tests: &Path,
    filter: Option<&str>,
) -> (Vec<(String, Option<String>)>, Vec<String>) {
    let messages_file = upstream_tests.join("messages.json");
    let content = fs::read_to_string(&messages_file).expect("Failed to read tests/messages.json");
    let expected_messages: HashMap<String, String> =
        serde_json::from_str(&content).expect("Failed to parse messages.json");

    let known_set: HashSet<String> = expected_messages.keys().cloned().collect();
    let mut valid_keys = Vec::new();
    let mut dir_skipped = Vec::new();
    collect_valid_files(
        upstream_tests,
        upstream_tests,
        &known_set,
        &mut valid_keys,
        &mut dir_skipped,
    );

    let mut all_cases: Vec<(String, Option<String>)> = expected_messages
        .into_iter()
        .map(|(k, v)| (k, Some(v)))
        .chain(valid_keys.into_iter().map(|k| (k, None)))
        .filter(|(k, _)| filter.as_ref().map(|f| k.contains(*f)).unwrap_or(true))
        .collect();
    all_cases.sort_by(|a, b| a.0.cmp(&b.0));

    (all_cases, dir_skipped)
}
