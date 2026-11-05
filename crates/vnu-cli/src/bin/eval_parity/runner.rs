use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use vnu_parser::ValidatorEngine;

pub struct ThreadBatchResult {
    pub total: usize,
    pub matches: usize,
    pub skipped: Vec<String>,
    pub cat_total: BTreeMap<String, usize>,
    pub cat_match: BTreeMap<String, usize>,
    pub failures: Vec<String>,
}

fn normalize(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_control() { '?' } else { c })
        .collect()
}

pub fn run_parity_workers(
    all_cases: &[(String, Option<String>)],
    upstream_tests: &Path,
) -> Vec<ThreadBatchResult> {
    let worker_count = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(all_cases.len().max(1));
    let next_idx = AtomicUsize::new(0);

    thread::scope(|s| {
        let mut handles = Vec::with_capacity(worker_count);
        let next_idx_ref = &next_idx;

        for _ in 0..worker_count {
            handles.push(s.spawn(move || {
                let mut engine = ValidatorEngine::default();
                let mut local_total = 0usize;
                let mut local_matches = 0usize;
                let mut local_skipped: Vec<String> = Vec::new();
                let mut local_cat_total: BTreeMap<String, usize> = BTreeMap::new();
                let mut local_cat_match: BTreeMap<String, usize> = BTreeMap::new();
                let mut local_failures: Vec<String> = Vec::new();

                loop {
                    let idx = next_idx_ref.fetch_add(1, Ordering::Relaxed);
                    if idx >= all_cases.len() {
                        break;
                    }
                    let (key, exp_opt) = &all_cases[idx];
                    let file_path = upstream_tests.join(key);
                    let src = match fs::read_to_string(&file_path) {
                        Ok(s) => s,
                        Err(_) => match fs::read(&file_path) {
                            Ok(b) => String::from_utf8_lossy(&b).to_string(),
                            Err(e) => {
                                local_skipped.push(format!("{}: {}", key, e));
                                continue;
                            }
                        },
                    };

                    local_total += 1;
                    let cat = key.split('/').take(3).collect::<Vec<_>>().join("/");
                    *local_cat_total.entry(cat.clone()).or_default() += 1;

                    let report = engine.validate(&src);
                    let found = match exp_opt {
                        Some(exp) => {
                            let exp_norm = normalize(exp);
                            report.messages.iter().any(|m| {
                                let msg_norm = normalize(&m.message);
                                msg_norm == exp_norm
                                    || msg_norm.contains(&exp_norm)
                                    || exp_norm.contains(&msg_norm)
                            })
                        }
                        None => report.is_valid(),
                    };

                    if found {
                        local_matches += 1;
                        *local_cat_match.entry(cat).or_default() += 1;
                    } else {
                        let exp_desc = exp_opt.as_deref().unwrap_or("<VALID: 0 errors>");
                        let mut fail_msg =
                            format!("FAILED: {}\nEXPECTED: {}\nGOT:\n", key, exp_desc);
                        for m in &report.messages {
                            fail_msg.push_str(&format!("  - [{:?}] {}\n", m.msg_type, m.message));
                        }
                        local_failures.push(fail_msg);
                    }
                }

                ThreadBatchResult {
                    total: local_total,
                    matches: local_matches,
                    skipped: local_skipped,
                    cat_total: local_cat_total,
                    cat_match: local_cat_match,
                    failures: local_failures,
                }
            }));
        }

        handles
            .into_iter()
            .map(|h| h.join().expect("worker thread panicked"))
            .collect::<Vec<_>>()
    })
}
