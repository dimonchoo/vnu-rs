//! Differential check of vnu-rs against Java v.Nu on a directory of HTML files.
//! Usage: oracle_diff [DIR]   (default: crates/vnu-oracle/corpus; Java at VNU_ORACLE_URL or :8899)

use std::path::PathBuf;
use vnu_oracle::{OracleHarness, oracle_endpoint};

fn main() {
    let dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/corpus")));
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "html"))
        .collect();
    files.sort();

    let harness = OracleHarness::default();
    let (mut same, mut with_missing, mut with_extra) = (0, 0, 0);
    for path in &files {
        let html = std::fs::read_to_string(path).expect("read corpus file");
        let name = path.file_stem().unwrap().to_string_lossy();
        let v = match harness.compare(&html) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("oracle unavailable at {}: {e}", oracle_endpoint());
                std::process::exit(2);
            }
        };
        if v.missing_in_local.is_empty() && v.extra_in_local.is_empty() {
            same += 1;
            println!("OK   {name}");
            continue;
        }
        println!("DIFF {name}");
        for m in &v.missing_in_local {
            println!("   missing: {m}");
        }
        for m in &v.extra_in_local {
            println!("   extra:   {m}");
        }
        with_missing += usize::from(!v.missing_in_local.is_empty());
        with_extra += usize::from(!v.extra_in_local.is_empty());
    }
    println!(
        "\n{same}/{} identical; {with_missing} with missed messages, {with_extra} with extra messages",
        files.len()
    );
}
