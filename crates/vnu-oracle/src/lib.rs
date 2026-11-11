use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use vnu_core::VNuReport;
use vnu_parser::ValidatorEngine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffVerdict {
    pub html_snippet: String,
    pub oracle_valid: bool,
    pub local_valid: bool,
    pub oracle_error_count: usize,
    pub local_error_count: usize,
    pub matched_rules: Vec<String>,
    pub missing_in_local: Vec<String>,
    pub extra_in_local: Vec<String>,
}

pub struct OracleHarness {
    endpoint: String,
    client: reqwest::blocking::Client,
}

/// Java v.Nu endpoint. :8888 now runs vnu-rs itself, so the oracle defaults to :8899
/// (`make oracle-up`) and can be overridden with VNU_ORACLE_URL.
pub fn oracle_endpoint() -> String {
    std::env::var("VNU_ORACLE_URL").unwrap_or_else(|_| "http://127.0.0.1:8899/?out=json".into())
}

impl Default for OracleHarness {
    fn default() -> Self {
        Self::new(oracle_endpoint())
    }
}

impl OracleHarness {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            client: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .expect("failed to create reqwest client"),
        }
    }

    pub fn query_oracle(&self, html: &str) -> Result<VNuReport, Box<dyn std::error::Error>> {
        let content_type = if html.trim_start().starts_with("<svg") {
            "image/svg+xml"
        } else {
            "text/html; charset=utf-8"
        };
        let resp = self
            .client
            .post(&self.endpoint)
            .header("Content-Type", content_type)
            .header("User-Agent", "vnu-rs-oracle/0.1.0")
            .body(html.to_string())
            .send()?;
        let status = resp.status();
        let body = resp.text()?;
        if !status.is_success() {
            return Err(format!("oracle returned HTTP {}: {}", status, body).into());
        }
        let report: VNuReport = serde_json::from_str(&body)
            .map_err(|e| format!("failed to parse json '{}': {}", body, e))?;
        Ok(report)
    }

    pub fn compare(&self, html: &str) -> Result<DiffVerdict, Box<dyn std::error::Error>> {
        let oracle_report = self.query_oracle(html)?;
        let mut engine = ValidatorEngine::default();
        let local_report = engine.validate(html);

        // Exact message comparison over errors and warnings (infos are advisory).
        let reportable = |r: &VNuReport| -> HashSet<String> {
            r.messages
                .iter()
                .filter(|m| {
                    m.msg_type != vnu_core::MessageType::Info
                        || m.sub_type == Some(vnu_core::MessageSubType::Warning)
                })
                .map(|m| m.message.clone())
                .collect()
        };
        let oracle_set = reportable(&oracle_report);
        let local_set = reportable(&local_report);
        let mut matched: Vec<String> = oracle_set.intersection(&local_set).cloned().collect();
        let mut missing: Vec<String> = oracle_set.difference(&local_set).cloned().collect();
        let mut extra: Vec<String> = local_set.difference(&oracle_set).cloned().collect();
        matched.sort();
        missing.sort();
        extra.sort();

        Ok(DiffVerdict {
            html_snippet: html.to_string(),
            oracle_valid: oracle_report.is_valid(),
            local_valid: local_report.is_valid(),
            oracle_error_count: oracle_report.error_count(),
            local_error_count: local_report.error_count(),
            matched_rules: matched,
            missing_in_local: missing,
            extra_in_local: extra,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires running Java v.Nu daemon on :8888"]
    fn test_differential_oracle_comparison() {
        let harness = OracleHarness::default();
        let test_html = "<!DOCTYPE html><html lang=\"en\"><head><title>Test</title></head><body><p>Hello</p></body></html>";
        match harness.compare(test_html) {
            Ok(diff) => {
                println!(
                    "Valid snippet -> Oracle valid={}, Local valid={}",
                    diff.oracle_valid, diff.local_valid
                );
                assert_eq!(diff.oracle_valid, diff.local_valid);
            }
            Err(e) => {
                panic!("Oracle check failed: {:?}", e);
            }
        }
    }

    #[test]
    #[ignore = "requires running Java v.Nu daemon on :8888"]
    fn test_invalid_snippet_differential() {
        let harness = OracleHarness::default();
        let test_html = "<html><body><img src='test.png'><a href='#'><a>nested</a></a></div>";
        match harness.compare(test_html) {
            Ok(diff) => {
                println!(
                    "Invalid snippet -> Oracle errors: {}, Local errors: {}",
                    diff.oracle_error_count, diff.local_error_count
                );
                println!("Matched rules: {:?}", diff.matched_rules);
                println!("Missing in local: {:?}", diff.missing_in_local);
                println!("Extra in local: {:?}", diff.extra_in_local);
                assert!(!diff.oracle_valid);
                assert!(!diff.local_valid);
                assert!(
                    !diff.matched_rules.is_empty(),
                    "Must match at least one official rule!"
                );
            }
            Err(e) => {
                panic!("Oracle check failed: {:?}", e);
            }
        }
    }

    #[test]
    #[ignore = "requires running Java v.Nu daemon on :8888"]
    fn test_obsolete_and_forms_differential() {
        let harness = OracleHarness::default();
        let test_html = "<!DOCTYPE html><html lang='en'><head><title>T</title></head><body><form><form></form></form><center>old</center><font color='red'>text</font><div align='center'>d</div></body></html>";
        match harness.compare(test_html) {
            Ok(diff) => {
                println!("Obsolete & forms diff -> Matched: {:?}", diff.matched_rules);
                println!("Missing in local: {:?}", diff.missing_in_local);
                assert!(!diff.oracle_valid);
                assert!(!diff.local_valid);
                assert!(
                    diff.matched_rules
                        .iter()
                        .any(|m| m.contains("Nested forms are not allowed"))
                );
                assert!(diff.matched_rules.iter().any(|m| m.contains("center")));
                assert!(diff.matched_rules.iter().any(|m| m.contains("font")));
            }
            Err(e) => {
                panic!("Oracle check failed: {:?}", e);
            }
        }
    }

    #[test]
    #[ignore = "requires running Java v.Nu daemon on :8888"]
    fn test_aria_differential() {
        let harness = OracleHarness::default();
        let test_html = "<!DOCTYPE html><html lang='en'><head><title>T</title></head><body><div role='invalid_role'>div</div><div aria-fake='test'>div</div></body></html>";
        match harness.compare(test_html) {
            Ok(diff) => {
                println!("ARIA diff -> Matched: {:?}", diff.matched_rules);
                println!("Missing in local: {:?}", diff.missing_in_local);
                assert!(!diff.oracle_valid);
                assert!(!diff.local_valid);
                assert!(
                    diff.matched_rules
                        .iter()
                        .any(|m| m.contains("Bad value “invalid_role” for attribute “role”"))
                );
                assert!(
                    diff.matched_rules
                        .iter()
                        .any(|m| m.contains("Attribute “aria-fake” not allowed"))
                );
            }
            Err(e) => {
                panic!("Oracle check failed: {:?}", e);
            }
        }
    }

    #[test]
    #[ignore = "requires running Java v.Nu daemon on :8888"]
    fn test_metadata_differential() {
        let harness = OracleHarness::default();
        let test_html = "<!DOCTYPE html><html lang='en'><head><title>T</title><base href='/'><base href='/2'><meta charset='latin1'><link rel='stylesheet'></head><body><svg><path d='M0 0'></svg></body></html>";
        match harness.compare(test_html) {
            Ok(diff) => {
                println!("Metadata diff -> Matched: {:?}", diff.matched_rules);
                println!("Missing in local: {:?}", diff.missing_in_local);
                assert!(!diff.oracle_valid);
                assert!(!diff.local_valid);
                assert!(diff.matched_rules.iter().any(|m| m.contains("base")));
                assert!(diff.matched_rules.iter().any(|m| m.contains("charset")));
                assert!(diff.matched_rules.iter().any(|m| m.contains("link")));
                assert!(
                    diff.matched_rules
                        .iter()
                        .any(|m| m.contains("End tag “svg” did not match"))
                );
            }
            Err(e) => {
                panic!("Oracle check failed: {:?}", e);
            }
        }
    }

    #[test]
    #[ignore = "requires running Java v.Nu daemon on :8888"]
    fn test_svg_differential() {
        let harness = OracleHarness::default();
        let valid_svg = "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><circle cx='50' cy='50' r='40'/></svg>";
        match harness.compare(valid_svg) {
            Ok(diff) => {
                println!(
                    "SVG valid diff -> Oracle valid={}, Local valid={}",
                    diff.oracle_valid, diff.local_valid
                );
                assert_eq!(diff.oracle_valid, diff.local_valid);
                assert!(
                    diff.matched_rules
                        .iter()
                        .any(|m| m.contains("preset for SVG"))
                );
            }
            Err(e) => {
                panic!("Oracle check failed: {:?}", e);
            }
        }

        let missing_ns_svg = "<svg viewBox='0 0 100 100'><circle cx='50' cy='50' r='40'/></svg>";
        match harness.compare(missing_ns_svg) {
            Ok(diff) => {
                println!(
                    "SVG missing ns diff -> Oracle valid={}, Local valid={}",
                    diff.oracle_valid, diff.local_valid
                );
                assert_eq!(diff.oracle_valid, diff.local_valid);
                assert!(
                    diff.matched_rules
                        .iter()
                        .any(|m| m.contains("XML document with no namespace"))
                );
            }
            Err(e) => {
                panic!("Oracle check failed: {:?}", e);
            }
        }
    }
}
