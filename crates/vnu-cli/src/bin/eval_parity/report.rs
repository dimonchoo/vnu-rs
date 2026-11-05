use super::runner::ThreadBatchResult;
use std::collections::BTreeMap;
use std::time::Duration;

pub fn print_parity_report(
    results: Vec<ThreadBatchResult>,
    initial_skipped: Vec<String>,
    elapsed: Duration,
    filter: Option<&str>,
) {
    let mut total = 0;
    let mut matches = 0;
    let mut skipped = initial_skipped;
    let mut cat_total: BTreeMap<String, usize> = BTreeMap::new();
    let mut cat_match: BTreeMap<String, usize> = BTreeMap::new();

    for batch in results {
        total += batch.total;
        matches += batch.matches;
        skipped.extend(batch.skipped);
        for (k, v) in batch.cat_total {
            *cat_total.entry(k).or_default() += v;
        }
        for (k, v) in batch.cat_match {
            *cat_match.entry(k).or_default() += v;
        }
        for f in batch.failures {
            print!("{}", f);
        }
    }

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
        println!(
            "Overall Match: {} / {} ({:.2}%)",
            matches,
            total,
            (matches as f64 / total as f64) * 100.0
        );
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
