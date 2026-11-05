mod filter;
mod format;
mod formatter;
mod options;
mod server;
mod service;

use filter::{MessageFilterConfig, filter_messages};
use format::OutputFormat;
use formatter::format_report;
use options::{CliOptions, parse_args, print_help};
use regex::Regex;
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::process;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use vnu_parser::ValidatorEngine;

fn should_skip_file(file_path: &str, opts: &CliOptions) -> bool {
    if file_path == "-" {
        return false;
    }
    let lower = file_path.to_ascii_lowercase();
    if opts.skip_non_html {
        return !(lower.ends_with(".html")
            || lower.ends_with(".htm")
            || lower.ends_with(".xhtml")
            || lower.ends_with(".xht"));
    }
    if opts.skip_non_svg {
        return !lower.ends_with(".svg");
    }
    if opts.skip_non_css {
        return !lower.ends_with(".css");
    }
    false
}

fn process_one_file(
    engine: &mut ValidatorEngine,
    file_path: &str,
    opts: &CliOptions,
    filter_cfg: &MessageFilterConfig,
    filter_re: Option<&Regex>,
) -> (bool, bool, Option<String>, Option<String>) {
    if should_skip_file(file_path, opts) {
        return (false, false, None, None);
    }

    if opts.verbose {
        eprintln!("Checking {}", file_path);
    }

    let content = if file_path == "-" {
        let mut buf = String::new();
        if let Err(e) = io::stdin().read_to_string(&mut buf) {
            return (
                true,
                false,
                None,
                Some(format!("Error reading stdin: {}", e)),
            );
        }
        buf
    } else {
        match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                return (
                    true,
                    false,
                    None,
                    Some(format!("Error reading {}: {}", file_path, e)),
                );
            }
        }
    };

    let report = engine.validate(&content);
    let (filtered, has_errors, has_warnings) = if filter_cfg.is_noop() && filter_re.is_none() {
        let has_err = !report.is_valid();
        let has_warn = report.warning_count() > 0;
        (report.messages, has_err, has_warn)
    } else {
        filter_messages(report.messages, filter_cfg, filter_re)
    };

    if filtered.is_empty() && opts.format != OutputFormat::Text {
        return (has_errors, has_warnings, None, None);
    }

    match format_report(file_path, filtered, opts.format) {
        Ok(formatted) => (has_errors, has_warnings, Some(formatted), None),
        Err(err) => (true, has_warnings, None, Some(err)),
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        print_help();
        return;
    }

    if args.contains(&"-v".to_string()) || args.contains(&"--version".to_string()) {
        println!("vnu-rs {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let mut opts = match parse_args(&args) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!("Run 'vnu --help' for usage details.");
            process::exit(1);
        }
    };

    if let Some(port) = opts.server_port {
        server::run_server(port);
        return;
    }

    if let Err(e) = opts.load_filter_files() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }

    if opts.files.is_empty() {
        eprintln!("Error: No input files specified. Use '-' to read from standard input.");
        eprintln!("Run 'vnu --help' for usage details.");
        process::exit(1);
    }

    let filter_re = match opts.build_filter_regex() {
        Ok(re) => re,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };
    let filter_re_ref = filter_re.as_ref();

    let filter_cfg = MessageFilterConfig {
        errors_only: opts.errors_only,
        skip_info_messages: opts.skip_info_messages,
        asciiquotes: opts.asciiquotes,
    };
    let filter_cfg_ref = &filter_cfg;

    let mut has_errors = false;
    let mut has_warnings = false;

    let write_output = |stdout_msg: Option<String>, stderr_msg: Option<String>| {
        if let Some(msg) = stderr_msg {
            eprintln!("{}", msg);
        }
        if let Some(msg) = stdout_msg {
            if opts.stdout {
                println!("{}", msg);
            } else {
                eprintln!("{}", msg);
            }
        }
    };

    let enable_langdetect = !opts.no_langdetect;

    if opts.files.len() <= 1 || opts.files.iter().any(|f| f == "-") {
        let mut engine = ValidatorEngine::with_options(enable_langdetect);
        for file_path in &opts.files {
            let (err, warn, stdout_msg, stderr_msg) =
                process_one_file(&mut engine, file_path, &opts, filter_cfg_ref, filter_re_ref);
            if err {
                has_errors = true;
            }
            if warn {
                has_warnings = true;
            }
            write_output(stdout_msg, stderr_msg);
        }
    } else {
        let worker_count = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .min(opts.files.len());
        let next_idx = AtomicUsize::new(0);
        let files_ref = &opts.files;
        let next_idx_ref = &next_idx;
        let opts_ref = &opts;

        let (thread_results, worker_failed) = thread::scope(|s| {
            let mut handles = Vec::with_capacity(worker_count);
            for _ in 0..worker_count {
                handles.push(s.spawn(move || {
                    let mut engine = ValidatorEngine::with_options(enable_langdetect);
                    let mut local_out = Vec::new();
                    loop {
                        let idx = next_idx_ref.fetch_add(1, Ordering::Relaxed);
                        if idx >= files_ref.len() {
                            break;
                        }
                        let res = process_one_file(
                            &mut engine,
                            &files_ref[idx],
                            opts_ref,
                            filter_cfg_ref,
                            filter_re_ref,
                        );
                        local_out.push((idx, res));
                    }
                    local_out
                }));
            }
            let mut combined = Vec::with_capacity(opts.files.len());
            let mut failed = false;
            for h in handles {
                match h.join() {
                    Ok(batch) => combined.extend(batch),
                    Err(_) => {
                        eprintln!("Error: CLI worker thread failed unexpectedly");
                        failed = true;
                    }
                }
            }
            (combined, failed)
        });

        if worker_failed {
            has_errors = true;
        }

        let mut ordered = thread_results;
        ordered.sort_unstable_by_key(|(idx, _)| *idx);
        for (_, (err, warn, stdout_msg, stderr_msg)) in ordered {
            if err {
                has_errors = true;
            }
            if warn {
                has_warnings = true;
            }
            write_output(stdout_msg, stderr_msg);
        }
    }

    if let Err(e) = io::stdout().flush() {
        eprintln!("Warning: Failed to flush stdout: {}", e);
    }
    if let Err(e) = io::stderr().flush() {
        eprintln!("Warning: Failed to flush stderr: {}", e);
    }

    if opts.exit_zero_always {
        process::exit(0);
    }

    if has_errors || (opts.werror && has_warnings) {
        process::exit(1);
    }
}
