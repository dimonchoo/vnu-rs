use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use unicode_normalization::{IsNormalized, UnicodeNormalization, is_nfc_quick};
use vnu_core::VNuMessage;

const MAX_AGGREGATED_BYTES: usize = 64 * 1024;
const MAX_AGGREGATED_RUNS: usize = 256;

#[derive(Default)]
pub struct LangDetectRule {
    html_lang: Option<String>,
    html_dir: Option<String>,
    total_non_ascii_bytes: usize,
    aggregated_runs: Vec<(String, usize, usize)>,
}

impl Rule for LangDetectRule {
    fn name(&self) -> &'static str {
        "langdetect"
    }

    fn reset(&mut self) {
        self.html_lang = None;
        self.html_dir = None;
        self.total_non_ascii_bytes = 0;
        self.aggregated_runs.clear();
    }

    fn handles_character_tokens(&self) -> bool {
        true
    }

    fn interested_in_characters(&self, ctx: &ValidationContext) -> bool {
        !ctx.is_descendant_of("pre")
            && !ctx.is_descendant_of("script")
            && !ctx.is_descendant_of("style")
    }

    fn on_characters(
        &mut self,
        _ctx: &ValidationContext,
        text: &str,
        line: usize,
        col: usize,
    ) -> Vec<VNuMessage> {
        // Fast-path: Skip pure ASCII
        if !text.as_bytes().iter().any(|&b| b >= 0x80) {
            return vec![];
        }

        // Defer expensive Unicode inspection & NFC checking to on_finish
        if self.total_non_ascii_bytes < MAX_AGGREGATED_BYTES
            && self.aggregated_runs.len() < MAX_AGGREGATED_RUNS
        {
            self.total_non_ascii_bytes += text.len();
            self.aggregated_runs.push((text.to_string(), line, col));
        }

        vec![]
    }

    fn interested_in_start_tag(&self, tag: &ElementContext) -> bool {
        tag.name.eq_ignore_ascii_case("html")
    }

    fn on_start_tag(&mut self, _ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        if tag.name.eq_ignore_ascii_case("html") {
            self.html_lang = tag.get_attr("lang").map(|s| s.to_string());
            self.html_dir = tag.get_attr("dir").map(|s| s.to_string());
        }
        vec![]
    }

    fn on_finish(&mut self, _ctx: &ValidationContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let mut hebrew_count = 0;
        let mut japanese_count = 0;
        let mut chinese_count = 0;

        for (text, line, col) in &self.aggregated_runs {
            for c in text.chars() {
                match c {
                    '\u{0590}'..='\u{05FF}' => hebrew_count += 1,
                    '\u{3040}'..='\u{309F}' => japanese_count += 1,
                    '\u{4E00}'..='\u{9FFF}' => chinese_count += 1,
                    _ => {}
                }
            }

            // Generic Unicode Normalization Form C (NFC) check (capped to 5 warnings)
            if msgs.len() < 5 && is_nfc_quick(text.chars()) != IsNormalized::Yes {
                let nfc: String = text.nfc().collect();
                if *text != nfc {
                    let nfc_display = if nfc.ends_with(" ?") {
                        format!("{} ", nfc.trim_end_matches('?').trim_end())
                    } else {
                        nfc
                    };
                    msgs.push(VNuMessage::warning(
                        format!(
                            "Text run is not in Unicode Normalization Form C. Should instead be “{}”. (Copy and paste that into your source document to replace the un-normalized text.)",
                            nfc_display
                        ),
                        *line,
                        *col,
                    ));
                }
            }
        }

        // Lang and Dir detection
        if hebrew_count > 50 && hebrew_count > chinese_count {
            if self.html_dir.as_deref() == Some("ltr") && self.html_lang.as_deref() == Some("he") {
                msgs.push(VNuMessage::warning(
                    "This document appears to be written in Hebrew but the “html” start tag has “dir=\"ltr\"”. Consider using “dir=\"rtl\"” instead.",
                    1,
                    1,
                ));
            } else if let Some(ref lang) = self.html_lang {
                if lang != "he" {
                    msgs.push(VNuMessage::warning(
                        format!(
                            "This document appears to be written in Hebrew but the “html” start tag has “lang=\"{}\"”. Consider using “lang=\"he\"” (or variant) instead.",
                            lang
                        ),
                        1,
                        1,
                    ));
                }
            }
        } else if japanese_count > 50 {
            if self.html_lang.as_deref() == Some("en") {
                msgs.push(VNuMessage::warning(
                    "This document appears to be written in Japanese but the “html” start tag has “lang=\"en\"”. Consider using “lang=\"ja\"” (or variant) instead.",
                    1,
                    1,
                ));
            }
        } else if chinese_count > 50 {
            if let Some(ref lang) = self.html_lang {
                if lang == "en" || lang == "he" {
                    msgs.push(VNuMessage::warning(
                        format!(
                            "This document appears to be written in Traditional Chinese but the “html” start tag has “lang=\"{}\"”. Consider using “lang=\"zh-hant\"” (or variant) instead.",
                            lang
                        ),
                        1,
                        1,
                    ));
                }
            }
        }

        msgs
    }
}
