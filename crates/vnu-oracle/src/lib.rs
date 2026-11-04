use std::collections::HashSet;
use serde::{Deserialize, Serialize};
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

impl Default for OracleHarness {
    fn default() -> Self {
        Self::new("http://127.0.0.1:8888/?out=json")
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
        let resp = self
            .client
            .post(&self.endpoint)
            .header("Content-Type", "text/html; charset=utf-8")
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

        let local_set: HashSet<&str> = local_report
            .messages
            .iter()
            .map(|m| m.message.as_str())
            .collect();

        let mut matched = Vec::new();
        let mut missing = Vec::new();

        for o_msg in &oracle_report.messages {
            let o_str = o_msg.message.as_str();
            if local_set.contains(o_str) {
                matched.push(o_msg.message.clone());
            } else if local_report.messages.iter().any(|l| {
                l.message.contains(o_str) || o_str.contains(&l.message)
            }) {
                matched.push(o_msg.message.clone());
            } else {
                missing.push(o_msg.message.clone());
            }
        }

        let oracle_set: HashSet<&str> = oracle_report
            .messages
            .iter()
            .map(|m| m.message.as_str())
            .collect();

        let mut extra = Vec::new();
        for l_msg in &local_report.messages {
            let l_str = l_msg.message.as_str();
            if !oracle_set.contains(l_str)
                && !oracle_report.messages.iter().any(|o| {
                    o.message.contains(l_str) || l_str.contains(&o.message)
                })
            {
                extra.push(l_msg.message.clone());
            }
        }

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
    fn test_differential_oracle_comparison() {
        let harness = OracleHarness::default();
        let test_html = "<!DOCTYPE html><html lang=\"en\"><head><title>Test</title></head><body><p>Hello</p></body></html>";
        match harness.compare(test_html) {
            Ok(diff) => {
                println!("Valid snippet -> Oracle valid={}, Local valid={}", diff.oracle_valid, diff.local_valid);
                assert_eq!(diff.oracle_valid, diff.local_valid);
            }
            Err(e) => {
                panic!("Oracle check failed: {:?}", e);
            }
        }
    }

    #[test]
    fn test_invalid_snippet_differential() {
        let harness = OracleHarness::default();
        let test_html = "<html><body><img src='test.png'><a href='#'><a>nested</a></a></div>";
        match harness.compare(test_html) {
            Ok(diff) => {
                println!("Invalid snippet -> Oracle errors: {}, Local errors: {}", diff.oracle_error_count, diff.local_error_count);
                println!("Matched rules: {:?}", diff.matched_rules);
                println!("Missing in local: {:?}", diff.missing_in_local);
                println!("Extra in local: {:?}", diff.extra_in_local);
                assert!(!diff.oracle_valid);
                assert!(!diff.local_valid);
                assert!(!diff.matched_rules.is_empty(), "Must match at least one official rule!");
            }
            Err(e) => {
                panic!("Oracle check failed: {:?}", e);
            }
        }
    }
}
