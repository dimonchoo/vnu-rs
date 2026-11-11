mod discovery;
mod report;
mod runner;

use std::time::Instant;

fn main() {
    let upstream_tests = match discovery::resolve_upstream_path() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    // --lenient keeps the old substring match; default is upstream TestRunner semantics.
    let args: Vec<String> = std::env::args().skip(1).collect();
    let strict = !args.iter().any(|a| a == "--lenient");
    let filter = args.into_iter().find(|a| !a.starts_with("--"));
    let start = Instant::now();

    let (all_cases, dir_skipped) = discovery::load_test_cases(&upstream_tests, filter.as_deref());
    let thread_results = runner::run_parity_workers(&all_cases, &upstream_tests, strict);
    let elapsed = start.elapsed();

    println!(
        "Mode: {}",
        if strict {
            "strict (upstream TestRunner)"
        } else {
            "lenient"
        }
    );
    report::print_parity_report(thread_results, dir_skipped, elapsed, filter.as_deref());
}
