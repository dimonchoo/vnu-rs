pub mod dispatch;
pub mod html_stack;
pub mod scan_policy;

use self::dispatch::{StreamingSink, StreamingValidator};
use crate::context::LineIndex;
use crate::rule::Rule;
use crate::rules::default_rules;
use html5ever::tokenizer::{BufferQueue, Tokenizer, TokenizerOpts, TokenizerResult};
use std::cell::RefCell;
use vnu_core::VNuReport;

pub struct ValidatorEngine {
    rules: Vec<Box<dyn Rule>>,
}

impl Default for ValidatorEngine {
    fn default() -> Self {
        Self {
            rules: default_rules(),
        }
    }
}

impl ValidatorEngine {
    pub fn new(rules: Vec<Box<dyn Rule>>) -> Self {
        Self { rules }
    }

    pub fn validate(&mut self, html: &str) -> VNuReport {
        let mut report = VNuReport::new();
        let line_index = LineIndex::new(html);

        for r in &mut self.rules {
            r.reset();
        }

        // Take rules temporarily for streaming validation
        let active_rules = std::mem::take(&mut self.rules);
        let sink = StreamingSink {
            inner: RefCell::new(StreamingValidator::new(html, active_rules, &line_index)),
        };

        let tok = Tokenizer::new(sink, TokenizerOpts::default());
        let input = BufferQueue::default();
        input.push_back(html.into());

        let is_script_step = matches!(tok.feed(&input), TokenizerResult::Script(_));
        tok.end();

        // Reclaim rules and messages
        let inner = tok.sink.inner.into_inner();
        let (_ctx, mut rules, messages, mut scan_policy) = inner.finish();
        scan_policy
            .tracker
            .check_raw_script_step(is_script_step, html);

        for r in &mut rules {
            r.reset();
        }
        self.rules = rules;
        report.messages.extend(messages);

        scan_policy.maybe_scan(html, &mut report.messages);

        // Populate line extracts where possible (truncated to 120 chars for performance)
        for msg in &mut report.messages {
            if let Some(l) = msg.last_line {
                let snippet = line_index.line_snippet(html, l);
                if !snippet.is_empty() {
                    let truncated = if snippet.len() > 120 {
                        let mut end = 120;
                        while end > 0 && !snippet.is_char_boundary(end) {
                            end -= 1;
                        }
                        &snippet[..end]
                    } else {
                        snippet
                    };
                    msg.extract = Some(truncated.to_string());
                }
            }
        }

        report
    }
}
