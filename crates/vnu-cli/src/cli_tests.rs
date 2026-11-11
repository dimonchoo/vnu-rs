use super::{process_one_file, select_check, status_code};
use crate::filter::MessageFilterConfig;
use crate::formatter::format_messages;
use crate::options::{CliOptions, parse_args};
use std::fs;
use std::path::Path;
use vnu_parser::{CheckMode, ValidatorEngine};

#[test]
fn server_port_is_consumed_and_not_treated_as_a_file() {
    let args = vec!["--server".to_string(), "9000".to_string()];
    let opts = parse_args(&args).expect("parse");
    assert_eq!(opts.server_port, Some(9000));
    assert!(opts.files.is_empty());
}

#[test]
fn server_flag_does_not_consume_following_option() {
    let args = vec!["--server".to_string(), "--errors-only".to_string()];
    let opts = parse_args(&args).expect("parse");
    assert_eq!(opts.server_port, Some(8888));
    assert!(opts.errors_only);
    assert!(opts.files.is_empty());
}

#[test]
fn werror_does_not_fail_when_errors_only_hides_warnings() {
    let mut opts = CliOptions::default();
    opts.werror = true;
    opts.errors_only = true;
    assert_eq!(status_code(false, true, &opts), 0);

    opts.errors_only = false;
    assert_eq!(status_code(false, true, &opts), 1);
    assert_eq!(status_code(true, false, &opts), 1);

    opts.exit_zero_always = true;
    assert_eq!(status_code(true, true, &opts), 0);
}

#[test]
fn missing_file_is_reported_as_a_non_document_error() {
    let path = "/tmp/vnu-rs-missing-no-such-file.html";
    let opts = CliOptions {
        files: vec![path.to_string()],
        ..CliOptions::default()
    };
    let mut engine = ValidatorEngine::with_options(true);
    let outcome = process_one_file(
        &mut engine,
        path,
        &opts,
        &MessageFilterConfig::default(),
        None,
        Path::new("/"),
    );
    assert!(outcome.has_errors);
    assert_eq!(outcome.messages.len(), 1);
    assert!(outcome.messages[0].message.contains("Error reading"));
    assert_eq!(
        outcome.messages[0].msg_type,
        vnu_core::MessageType::NonDocumentError
    );
    let gnu = format_messages(&outcome.messages, crate::format::OutputFormat::Gnu).expect("gnu");
    assert!(gnu.contains("non-document-error:"));
    assert!(gnu.contains("file:/tmp/vnu-rs-missing-no-such-file.html"));
}

#[test]
fn skip_non_css_checks_css_and_skips_html() {
    let mut opts = CliOptions::default();
    opts.skip_non_css = true;
    opts.skip_non_html = true;
    assert_eq!(select_check("app.css", &opts), Some(CheckMode::Css));
    assert_eq!(select_check("index.html", &opts), None);
}

#[test]
fn also_check_css_leaves_html_on_the_html_checker() {
    let mut opts = CliOptions::default();
    opts.also_check_css = true;
    assert_eq!(select_check("app.css", &opts), Some(CheckMode::Css));
    assert_eq!(select_check("index.html", &opts), Some(CheckMode::Html));
    assert_eq!(select_check("notes.txt", &opts), Some(CheckMode::Html));
}

#[test]
fn force_css_wins_over_svg_and_checks_every_file() {
    let mut opts = CliOptions::default();
    opts.force_css = true;
    opts.force_svg = true;
    assert_eq!(select_check("index.html", &opts), Some(CheckMode::Css));
    assert_eq!(select_check("icon.svg", &opts), Some(CheckMode::Css));
}

#[test]
fn svg_filters_select_the_svg_checker() {
    let mut skip = CliOptions::default();
    skip.skip_non_svg = true;
    assert_eq!(select_check("icon.svg", &skip), Some(CheckMode::Svg));
    assert_eq!(select_check("index.html", &skip), None);

    let mut also = CliOptions::default();
    also.also_check_svg = true;
    assert_eq!(select_check("icon.svg", &also), Some(CheckMode::Svg));
    assert_eq!(select_check("index.html", &also), Some(CheckMode::Html));
}

#[test]
fn css_file_is_not_reported_as_html() {
    let root = std::env::temp_dir().join(format!(
        "vnu-rs-css-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    fs::create_dir_all(&root).expect("mkdir");
    let css = root.join("bad.css");
    fs::write(&css, "p { colr: red; }\n").expect("write");
    let mut opts = CliOptions::default();
    opts.skip_non_css = true;
    let mut engine = ValidatorEngine::with_options(true);
    let outcome = process_one_file(
        &mut engine,
        &css.to_string_lossy(),
        &opts,
        &MessageFilterConfig::default(),
        None,
        Path::new("/"),
    );
    assert!(outcome.has_errors);
    assert!(
        outcome
            .messages
            .iter()
            .any(|msg| msg.message.contains("Property “colr”"))
    );
    assert!(
        outcome
            .messages
            .iter()
            .all(|msg| !msg.message.contains("doctype"))
    );
    let _ = fs::remove_dir_all(&root);
}
