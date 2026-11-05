mod server;
mod service;

use service::validate_content;
use std::env;
use std::fs;
use std::io::{self, Read};
use std::process;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use vnu_core::{MessageSubType, MessageType};
use vnu_parser::ValidatorEngine;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum OutputFormat {
    #[default]
    Text,
    Gnu,
    Json,
}

fn print_help() {
    println!(
        r#"vnu-rs: Native ultra-fast W3C HTML5 validator

USAGE:
    vnu [OPTIONS] <FILE>...
    vnu - (reads from stdin)
    vnu --server [PORT] (starts HTTP daemon, default: 8889)

OPTIONS:
    --format <text|gnu|json> Output format (default: text)
    --server [PORT]          Run as an HTTP server matching v.Nu API
    -h, --help               Print this help message
    -v, --version            Print version
"#
    );
}

fn process_one_file(
    engine: &mut ValidatorEngine,
    file_path: &str,
    format: OutputFormat,
) -> (bool, Option<String>, Option<String>) {
    let content = if file_path == "-" {
        let mut buf = String::new();
        if let Err(e) = io::stdin().read_to_string(&mut buf) {
            return (true, None, Some(format!("Error reading stdin: {}", e)));
        }
        buf
    } else {
        match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                return (
                    true,
                    None,
                    Some(format!("Error reading {}: {}", file_path, e)),
                );
            }
        }
    };

    match format {
        OutputFormat::Json => match validate_content(engine, &content, true) {
            Ok((report, json)) => (!report.is_valid(), Some(json), None),
            Err(e) => (
                true,
                None,
                Some(format!("Validation failure for {}: {}", file_path, e)),
            ),
        },
        OutputFormat::Gnu => {
            let report = engine.validate(&content);
            let has_err = !report.is_valid();
            let mut out = String::new();
            for (i, m) in report.messages.into_iter().enumerate() {
                if i > 0 {
                    out.push('\n');
                }
                let loc = match (m.last_line, m.last_column) {
                    (Some(l), Some(c)) => format!("{}.{}: ", l, c),
                    (Some(l), None) => format!("{}: ", l),
                    _ => String::new(),
                };
                let kind = match m.msg_type {
                    MessageType::Info if m.sub_type == Some(MessageSubType::Warning) => "warning",
                    MessageType::Info => "info",
                    MessageType::Error => "error",
                    MessageType::NonDocumentError => "fatal error",
                };
                out.push_str(&format!(
                    "\"{}\": {}{}: {}",
                    file_path, loc, kind, m.message
                ));
            }
            (has_err, Some(out), None)
        }
        OutputFormat::Text => {
            let report = engine.validate(&content);
            let has_err = !report.is_valid();
            let mut out = String::new();
            if report.messages.is_empty() {
                out.push_str(&format!("{}: The document is valid.", file_path));
            } else {
                for (i, m) in report.messages.into_iter().enumerate() {
                    if i > 0 {
                        out.push('\n');
                    }
                    let loc = match (m.last_line, m.last_column) {
                        (Some(l), Some(c)) => format!("{}:{}: ", l, c),
                        (Some(l), None) => format!("{}: ", l),
                        _ => String::new(),
                    };
                    let kind = match m.msg_type {
                        MessageType::Info if m.sub_type == Some(MessageSubType::Warning) => {
                            "warning"
                        }
                        MessageType::Info => "info",
                        MessageType::Error => "error",
                        MessageType::NonDocumentError => "error",
                    };
                    out.push_str(&format!("{}: {}{}: {}", file_path, loc, kind, m.message));
                }
            }
            (has_err, Some(out), None)
        }
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

    if let Some(pos) = args.iter().position(|a| a == "--server") {
        let port = args
            .get(pos + 1)
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8889);
        server::run_server(port);
        return;
    }

    let mut format = OutputFormat::Text;
    let mut files = Vec::new();

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        if arg == "--format" {
            if let Some(fmt) = iter.next() {
                match fmt.to_lowercase().as_str() {
                    "json" => format = OutputFormat::Json,
                    "gnu" => format = OutputFormat::Gnu,
                    _ => format = OutputFormat::Text,
                }
            }
        } else if !arg.starts_with("--") {
            files.push(arg);
        }
    }

    let mut has_errors = false;

    if files.len() <= 1 || files.iter().any(|f| f == "-") {
        let mut engine = ValidatorEngine::default();
        for file_path in &files {
            let (err, stdout_msg, stderr_msg) = process_one_file(&mut engine, file_path, format);
            if err {
                has_errors = true;
            }
            if let Some(msg) = stderr_msg {
                eprintln!("{}", msg);
            }
            if let Some(msg) = stdout_msg {
                println!("{}", msg);
            }
        }
    } else {
        let worker_count = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .min(files.len());
        let next_idx = AtomicUsize::new(0);
        let files_ref = &files;
        let next_idx_ref = &next_idx;

        let (thread_results, worker_failed) = thread::scope(|s| {
            let mut handles = Vec::with_capacity(worker_count);
            for _ in 0..worker_count {
                handles.push(s.spawn(move || {
                    let mut engine = ValidatorEngine::default();
                    let mut local_out = Vec::new();
                    loop {
                        let idx = next_idx_ref.fetch_add(1, Ordering::Relaxed);
                        if idx >= files_ref.len() {
                            break;
                        }
                        let res = process_one_file(&mut engine, &files_ref[idx], format);
                        local_out.push((idx, res));
                    }
                    local_out
                }));
            }
            let mut combined = Vec::with_capacity(files.len());
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
        for (_, (err, stdout_msg, stderr_msg)) in ordered {
            if err {
                has_errors = true;
            }
            if let Some(msg) = stderr_msg {
                eprintln!("{}", msg);
            }
            if let Some(msg) = stdout_msg {
                println!("{}", msg);
            }
        }
    }

    if has_errors {
        process::exit(1);
    }
}
