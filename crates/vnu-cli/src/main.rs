mod filter;
mod format;
mod formatter;
mod inputs;
mod options;
mod server;
mod service;

use filter::{MessageFilterConfig, filter_messages};
use formatter::format_messages;
use inputs::{document_url, expand_inputs};
use options::{CliOptions, parse_args, print_help};
use regex::Regex;
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use vnu_core::VNuMessage;
use vnu_parser::{CheckMode, ValidatorEngine};

struct FileOutcome {
    has_errors: bool,
    has_warnings: bool,
    messages: Vec<VNuMessage>,
}

fn ends_with_ignore_ascii_case(path: &str, suffix: &str) -> bool {
    let start = path.len().saturating_sub(suffix.len());
    path.is_char_boundary(start) && path[start..].eq_ignore_ascii_case(suffix)
}

fn select_check(file_path: &str, opts: &CliOptions) -> Option<CheckMode> {
    let is_css = file_path != "-" && ends_with_ignore_ascii_case(file_path, ".css");
    let is_svg = file_path != "-" && ends_with_ignore_ascii_case(file_path, ".svg");
    let is_html = file_path == "-"
        || ends_with_ignore_ascii_case(file_path, ".html")
        || ends_with_ignore_ascii_case(file_path, ".htm")
        || ends_with_ignore_ascii_case(file_path, ".xhtml")
        || ends_with_ignore_ascii_case(file_path, ".xht");

    // Same priority as upstream vnu: CSS options win over SVG options.
    if opts.force_css {
        return Some(CheckMode::Css);
    }
    if opts.skip_non_css {
        return is_css.then_some(CheckMode::Css);
    }
    if opts.also_check_css && is_css {
        return Some(CheckMode::Css);
    }
    if opts.force_svg {
        return Some(CheckMode::Svg);
    }
    if opts.skip_non_svg {
        return is_svg.then_some(CheckMode::Svg);
    }
    if opts.also_check_svg && is_svg {
        return Some(CheckMode::Svg);
    }
    if opts.skip_non_html && !is_html {
        return None;
    }
    Some(CheckMode::Html)
}

fn tag_with_url(messages: &mut [VNuMessage], url: &str) {
    for message in messages {
        if message.url.is_none() {
            message.url = Some(url.to_string());
        }
    }
}

fn process_one_file(
    engine: &mut ValidatorEngine,
    file_path: &str,
    opts: &CliOptions,
    filter_cfg: &MessageFilterConfig,
    filter_re: Option<&Regex>,
    cwd: &Path,
) -> FileOutcome {
    let Some(mode) = select_check(file_path, opts) else {
        return FileOutcome {
            has_errors: false,
            has_warnings: false,
            messages: Vec::new(),
        };
    };

    if opts.verbose {
        eprintln!("Checking {file_path}");
    }

    let url = document_url(file_path, cwd);
    let content = if file_path == "-" {
        let mut buf = String::new();
        if let Err(err) = io::stdin().read_to_string(&mut buf) {
            return read_error(file_path, &url, err, filter_cfg, filter_re);
        }
        buf
    } else {
        match fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(err) => return read_error(file_path, &url, err, filter_cfg, filter_re),
        }
    };

    let report = engine.validate_as(&content, mode);
    let (mut filtered, has_errors, has_warnings) = if filter_cfg.is_noop() && filter_re.is_none() {
        let has_errors = !report.is_valid();
        let has_warnings = report.warning_count() > 0;
        (report.messages, has_errors, has_warnings)
    } else {
        filter_messages(report.messages, filter_cfg, filter_re)
    };
    tag_with_url(&mut filtered, &url);
    FileOutcome {
        has_errors,
        has_warnings,
        messages: filtered,
    }
}

fn read_error(
    file_path: &str,
    url: &str,
    err: impl std::fmt::Display,
    filter_cfg: &MessageFilterConfig,
    filter_re: Option<&Regex>,
) -> FileOutcome {
    let messages = vec![
        VNuMessage::non_document_error(format!("Error reading {file_path}: {err}")).with_url(url),
    ];
    let (filtered, has_errors, has_warnings) = filter_messages(messages, filter_cfg, filter_re);
    FileOutcome {
        has_errors,
        has_warnings,
        messages: filtered,
    }
}

fn status_code(has_errors: bool, has_warnings: bool, opts: &CliOptions) -> i32 {
    if opts.exit_zero_always {
        0
    } else if has_errors || (opts.werror && has_warnings && !opts.errors_only) {
        1
    } else {
        0
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
        Ok(opts) => opts,
        Err(err) => {
            eprintln!("Error: {err}");
            eprintln!("Run 'vnu --help' for usage details.");
            process::exit(1);
        }
    };

    if let Some(port) = opts.server_port {
        server::run_server(port);
        return;
    }

    if let Err(err) = opts.load_filter_files() {
        eprintln!("Error: {err}");
        process::exit(1);
    }

    if opts.files.is_empty() {
        eprintln!("Error: No input files specified. Use '-' to read from standard input.");
        eprintln!("Run 'vnu --help' for usage details.");
        process::exit(1);
    }

    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let files = match expand_inputs(&opts.files) {
        Ok(files) => files,
        Err(err) => emit_and_exit(
            &opts,
            vec![VNuMessage::non_document_error(err)],
            true,
            false,
        ),
    };
    opts.files = files;

    let filter_re = match opts.build_filter_regex() {
        Ok(re) => re,
        Err(err) => {
            eprintln!("Error: {err}");
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
    let mut messages = Vec::new();

    let enable_langdetect = !opts.no_langdetect;

    if opts.files.len() <= 1 || opts.files.iter().any(|file| file == "-") {
        let mut engine = ValidatorEngine::with_options(enable_langdetect);
        for file_path in &opts.files {
            let outcome = process_one_file(
                &mut engine,
                file_path,
                &opts,
                filter_cfg_ref,
                filter_re_ref,
                &cwd,
            );
            has_errors |= outcome.has_errors;
            has_warnings |= outcome.has_warnings;
            messages.extend(outcome.messages);
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
        let cwd_ref = cwd.as_path();

        let (thread_results, worker_failed) = thread::scope(|scope| {
            let mut handles = Vec::with_capacity(worker_count);
            for _ in 0..worker_count {
                handles.push(scope.spawn(move || {
                    let mut engine = ValidatorEngine::with_options(enable_langdetect);
                    let mut local_out = Vec::new();
                    loop {
                        let idx = next_idx_ref.fetch_add(1, Ordering::Relaxed);
                        if idx >= files_ref.len() {
                            break;
                        }
                        let outcome = process_one_file(
                            &mut engine,
                            &files_ref[idx],
                            opts_ref,
                            filter_cfg_ref,
                            filter_re_ref,
                            cwd_ref,
                        );
                        local_out.push((idx, outcome));
                    }
                    local_out
                }));
            }
            let mut combined = Vec::with_capacity(opts.files.len());
            let mut failed = false;
            for handle in handles {
                match handle.join() {
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
        for (_, outcome) in ordered {
            has_errors |= outcome.has_errors;
            has_warnings |= outcome.has_warnings;
            messages.extend(outcome.messages);
        }
    }

    emit_and_exit(&opts, messages, has_errors, has_warnings);
}

fn emit_and_exit(
    opts: &CliOptions,
    messages: Vec<VNuMessage>,
    mut has_errors: bool,
    has_warnings: bool,
) -> ! {
    match format_messages(&messages, opts.format) {
        Ok(formatted) if !formatted.is_empty() => {
            if opts.stdout {
                println!("{formatted}");
            } else {
                eprintln!("{formatted}");
            }
        }
        Ok(_) => {}
        Err(err) => {
            eprintln!("{err}");
            has_errors = true;
        }
    }

    if let Err(err) = io::stdout().flush() {
        eprintln!("Warning: Failed to flush stdout: {err}");
    }
    if let Err(err) = io::stderr().flush() {
        eprintln!("Warning: Failed to flush stderr: {err}");
    }

    process::exit(status_code(has_errors, has_warnings, opts));
}

#[cfg(test)]
mod cli_tests;
