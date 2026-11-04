use std::env;
use std::fs;
use std::io::{self, Read};
use std::process;
use vnu_parser::ValidatorEngine;

fn print_help() {
    println!(
        r#"vnu-rs: Native ultra-fast W3C HTML5 validator

USAGE:
    vnu [OPTIONS] <FILE>...
    vnu - (reads from stdin)

OPTIONS:
    --format <text|json>    Output format (default: json)
    -h, --help              Print this help message
    -v, --version           Print version
"#
    );
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

    let mut is_json = true;
    let mut files = Vec::new();

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        if arg == "--format" {
            if let Some(fmt) = iter.next() {
                if fmt == "text" {
                    is_json = false;
                }
            }
        } else if !arg.starts_with("--") {
            files.push(arg);
        }
    }

    let mut engine = ValidatorEngine::default();
    let mut has_errors = false;

    for file_path in files {
        let content = if file_path == "-" {
            let mut buf = String::new();
            if let Err(e) = io::stdin().read_to_string(&mut buf) {
                eprintln!("Error reading stdin: {}", e);
                has_errors = true;
                continue;
            }
            buf
        } else {
            match fs::read_to_string(&file_path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error reading {}: {}", file_path, e);
                    has_errors = true;
                    continue;
                }
            }
        };

        let report = engine.validate(&content);
        if !report.is_valid() {
            has_errors = true;
        }

        if is_json {
            match serde_json::to_string_pretty(&report) {
                Ok(json) => println!("{}", json),
                Err(e) => {
                    eprintln!("Error serializing JSON report: {}", e);
                    has_errors = true;
                }
            }
        } else {
            if report.messages.is_empty() {
                println!("{}: The document is valid.", file_path);
            } else {
                for m in report.messages {
                    let loc = match (m.last_line, m.last_column) {
                        (Some(l), Some(c)) => format!("{}:{}: ", l, c),
                        (Some(l), None) => format!("{}: ", l),
                        _ => String::new(),
                    };
                    let kind = format!("{:?}", m.msg_type).to_lowercase();
                    println!("{}: {}{}: {}", file_path, loc, kind, m.message);
                }
            }
        }
    }

    if has_errors {
        process::exit(1);
    }
}
