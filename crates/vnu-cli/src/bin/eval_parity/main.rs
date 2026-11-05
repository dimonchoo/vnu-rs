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

    let filter = std::env::args().nth(1);
    let start = Instant::now();

    let (all_cases, dir_skipped) = discovery::load_test_cases(&upstream_tests, filter.as_deref());
    let thread_results = runner::run_parity_workers(&all_cases, &upstream_tests);
    let elapsed = start.elapsed();

    report::print_parity_report(thread_results, dir_skipped, elapsed, filter.as_deref());
}
