use crate::format::OutputFormat;
use regex::Regex;
use std::fs;

#[derive(Clone, Default, Debug)]
pub struct CliOptions {
    pub format: OutputFormat,
    pub errors_only: bool,
    pub werror: bool,
    pub exit_zero_always: bool,
    pub stdout: bool,
    pub asciiquotes: bool,
    pub no_langdetect: bool,
    pub skip_info_messages: bool,
    pub skip_non_html: bool,
    pub skip_non_svg: bool,
    pub skip_non_css: bool,
    pub verbose: bool,
    pub server_port: Option<u16>,
    pub filter_patterns: Vec<String>,
    pub filter_files: Vec<String>,
    pub files: Vec<String>,
}

impl CliOptions {
    pub fn load_filter_files(&mut self) -> Result<(), String> {
        for path in &self.filter_files {
            let content = fs::read_to_string(path)
                .map_err(|e| format!("Cannot read filterfile '{}': {}", path, e))?;
            for line in content.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    self.filter_patterns.push(trimmed.to_string());
                }
            }
        }
        Ok(())
    }

    pub fn build_filter_regex(&self) -> Result<Option<Regex>, String> {
        if self.filter_patterns.is_empty() {
            return Ok(None);
        }
        let joined = self
            .filter_patterns
            .iter()
            .map(|p| format!("(?:{})", p))
            .collect::<Vec<_>>()
            .join("|");
        Regex::new(&joined)
            .map(Some)
            .map_err(|e| format!("Invalid filter pattern '{}': {}", joined, e))
    }
}

pub fn print_help() {
    println!(
        r#"NAME
    vnu – The Nu Html Checker (vnu-rs)

SYNOPSIS
    vnu [OPTIONS]... FILES...
    echo '<!doctype html><title>...' | vnu -
    vnu --server [PORT]

OPTIONS
    --errors-only
        Specifies that only error-level messages and non-document-error messages
        are reported (warnings and info messages are not reported).

    --skip-info-messages
        Specifies that only error-level messages, warnings, and non-document-error
        messages are reported — but not info messages.

    --Werror
        Makes vnu exit non-zero if any warnings are encountered (even if no errors).

    --exit-zero-always
        Makes vnu exit zero even if errors are reported for any documents.

    --stdout
        Makes vnu report errors and warnings to stdout rather than stderr.

    --asciiquotes
        Specifies whether ASCII quotation marks are substituted for Unicode
        smart quotation marks in messages.

    --no-langdetect
        Disables language detection and html[lang] checking.

    --format gnu|xml|json|text
        Specifies output format for reporting results (default: "gnu").

    --filterpattern REGEXP
        Regular expression to filter out / suppress matching messages.

    --filterfile FILENAME
        Specifies a file containing regular expressions (one per line) to filter out.

    --skip-non-html
        Skip documents that don’t have *.html, *.htm, *.xhtml, or *.xht extensions.

    --skip-non-svg
        Check documents as SVG but skip documents that don’t have *.svg extensions.

    --skip-non-css
        Check documents as CSS but skip documents that don’t have *.css extensions.

    --server [PORT]
        Run as an HTTP server matching the official v.Nu API (default port: 8888).

    --verbose
        Verbose output; prints names of files being checked.

    -h, --help
        Print this help message.

    -v, --version
        Print version number.
"#
    );
}

pub fn parse_args(args: &[String]) -> Result<CliOptions, String> {
    let mut opts = CliOptions::default();
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--errors-only" => opts.errors_only = true,
            "--skip-info-messages" => opts.skip_info_messages = true,
            "--Werror" => opts.werror = true,
            "--exit-zero-always" => opts.exit_zero_always = true,
            "--stdout" => opts.stdout = true,
            "--asciiquotes" => opts.asciiquotes = true,
            "--no-langdetect" => opts.no_langdetect = true,
            "--skip-non-html" => opts.skip_non_html = true,
            "--skip-non-svg" => opts.skip_non_svg = true,
            "--skip-non-css" => opts.skip_non_css = true,
            "--verbose" => opts.verbose = true,
            "--format" => {
                if let Some(fmt) = iter.next() {
                    match fmt.to_lowercase().as_str() {
                        "json" => opts.format = OutputFormat::Json,
                        "xml" => opts.format = OutputFormat::Xml,
                        "text" => opts.format = OutputFormat::Text,
                        "gnu" => opts.format = OutputFormat::Gnu,
                        other => return Err(format!("Unknown output format: {}", other)),
                    }
                } else {
                    return Err("--format requires an argument (gnu|xml|json|text)".to_string());
                }
            }
            "--filterpattern" => {
                if let Some(pattern) = iter.next() {
                    opts.filter_patterns.push(pattern.clone());
                } else {
                    return Err("--filterpattern requires a regular expression".to_string());
                }
            }
            "--filterfile" => {
                if let Some(path) = iter.next() {
                    opts.filter_files.push(path.clone());
                } else {
                    return Err("--filterfile requires a file path".to_string());
                }
            }
            "--server" => {
                let port = iter
                    .clone()
                    .next()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or(8888);
                opts.server_port = Some(port);
            }
            file if !file.starts_with("--") => {
                opts.files.push(file.to_string());
            }
            unknown => {
                return Err(format!("Unknown option: {}", unknown));
            }
        }
    }

    Ok(opts)
}
