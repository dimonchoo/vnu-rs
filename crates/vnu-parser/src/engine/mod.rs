pub mod attribute_normalization;
pub mod dispatch;
pub mod html_stack;
pub mod scan_policy;
pub mod sink_adapter;
pub mod stack_policy;

use self::dispatch::StreamingValidator;
use self::sink_adapter::StreamingSink;
use crate::context::LineIndex;
use crate::rule::Rule;
use crate::rules::default_rules;
use html5ever::tokenizer::{BufferQueue, Tokenizer, TokenizerOpts, TokenizerResult};
use std::cell::RefCell;
use vnu_core::VNuReport;

pub struct ValidatorEngine {
    rules: Vec<Box<dyn Rule>>,
    char_rule_indices: Vec<usize>,
    stray_rule_indices: Vec<usize>,
    line_index: LineIndex,
}

impl Default for ValidatorEngine {
    fn default() -> Self {
        Self::new(default_rules())
    }
}

impl ValidatorEngine {
    pub fn new(rules: Vec<Box<dyn Rule>>) -> Self {
        let char_rule_indices = rules
            .iter()
            .enumerate()
            .filter(|(_, r)| r.handles_character_tokens())
            .map(|(i, _)| i)
            .collect();
        let stray_rule_indices = rules
            .iter()
            .enumerate()
            .filter(|(_, r)| r.handles_stray_end_tags())
            .map(|(i, _)| i)
            .collect();
        Self {
            rules,
            char_rule_indices,
            stray_rule_indices,
            line_index: LineIndex::default(),
        }
    }

    pub fn validate(&mut self, html: &str) -> VNuReport {
        let mut report = VNuReport::new();
        self.line_index.reset(html);

        for r in &mut self.rules {
            r.reset();
        }

        // Take rules and cached indices temporarily for streaming validation
        let active_rules = std::mem::take(&mut self.rules);
        let char_indices = std::mem::take(&mut self.char_rule_indices);
        let stray_indices = std::mem::take(&mut self.stray_rule_indices);
        let sink = StreamingSink {
            inner: RefCell::new(StreamingValidator::with_indices(
                html,
                active_rules,
                &self.line_index,
                char_indices,
                stray_indices,
            )),
        };

        let tok = Tokenizer::new(sink, TokenizerOpts::default());
        let input = BufferQueue::default();
        input.push_back(html.into());

        let is_script_step = matches!(tok.feed(&input), TokenizerResult::Script(_));
        tok.end();

        // Reclaim rules, cached indices, and messages
        let mut inner = tok.sink.inner.into_inner();
        let char_indices = std::mem::take(&mut inner.char_rule_indices);
        let stray_indices = std::mem::take(&mut inner.stray_rule_indices);
        let (_ctx, rules, messages, mut scan_policy) = inner.finish();
        scan_policy
            .tracker
            .check_raw_script_step(is_script_step, html);

        self.rules = rules;
        self.char_rule_indices = char_indices;
        self.stray_rule_indices = stray_indices;
        report.messages.extend(messages);

        scan_policy.maybe_scan(html, &mut report.messages);

        // Populate line extracts where possible (truncated to 120 chars for performance)
        for msg in &mut report.messages {
            if let Some(l) = msg.last_line {
                let snippet = self.line_index.line_snippet(html, l);
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
