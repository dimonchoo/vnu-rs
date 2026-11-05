use regex::Regex;
use vnu_core::{MessageSubType, MessageType, VNuMessage};

#[derive(Clone, Copy, Default, Debug)]
pub struct MessageFilterConfig {
    pub errors_only: bool,
    pub skip_info_messages: bool,
    pub asciiquotes: bool,
}

impl MessageFilterConfig {
    pub fn is_noop(&self) -> bool {
        !self.errors_only && !self.skip_info_messages && !self.asciiquotes
    }
}

pub fn filter_messages(
    messages: Vec<VNuMessage>,
    cfg: &MessageFilterConfig,
    filter_re: Option<&Regex>,
) -> (Vec<VNuMessage>, bool, bool) {
    let mut filtered = Vec::new();
    let mut has_errors = false;
    let mut has_warnings = false;

    for mut m in messages {
        let is_err = matches!(
            m.msg_type,
            MessageType::Error | MessageType::NonDocumentError
        );
        let is_warn =
            m.msg_type == MessageType::Info && m.sub_type == Some(MessageSubType::Warning);
        let is_info = m.msg_type == MessageType::Info && !is_warn;

        if cfg.errors_only && !is_err {
            continue;
        }
        if cfg.skip_info_messages && is_info {
            continue;
        }
        if let Some(re) = filter_re {
            if re.is_match(&m.message) {
                continue;
            }
        }
        if cfg.asciiquotes {
            m.message = m
                .message
                .replace('“', "\"")
                .replace('”', "\"")
                .replace('‘', "'")
                .replace('’', "'");
        }

        if is_err {
            has_errors = true;
        } else if is_warn {
            has_warnings = true;
        }

        filtered.push(m);
    }

    (filtered, has_errors, has_warnings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vnu_core::{MessageSubType, MessageType, VNuMessage};

    #[test]
    fn errors_only_keeps_errors_and_drops_warnings_and_info() {
        let messages = vec![
            VNuMessage::error("bad", 1, 1),
            VNuMessage::warning("careful", 1, 2),
            VNuMessage::info("note", 1, 3),
            VNuMessage::non_document_error("cannot read"),
        ];
        let cfg = MessageFilterConfig {
            errors_only: true,
            ..MessageFilterConfig::default()
        };
        let (filtered, has_errors, has_warnings) = filter_messages(messages, &cfg, None);
        assert!(has_errors);
        assert!(!has_warnings);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].msg_type, MessageType::Error);
        assert_eq!(filtered[1].msg_type, MessageType::NonDocumentError);
    }

    #[test]
    fn skip_info_keeps_warnings() {
        let messages = vec![
            VNuMessage::warning("careful", 1, 2),
            VNuMessage::info("note", 1, 3),
        ];
        let cfg = MessageFilterConfig {
            skip_info_messages: true,
            ..MessageFilterConfig::default()
        };
        let (filtered, has_errors, has_warnings) = filter_messages(messages, &cfg, None);
        assert!(!has_errors);
        assert!(has_warnings);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].sub_type, Some(MessageSubType::Warning));
    }

    #[test]
    fn asciiquotes_and_filter_pattern_suppress_matches() {
        let messages = vec![
            VNuMessage::error("Expected “doctype”", 1, 1),
            VNuMessage::error("unrelated", 2, 1),
        ];
        let cfg = MessageFilterConfig {
            asciiquotes: true,
            ..MessageFilterConfig::default()
        };
        let pattern = Regex::new("unrelated").expect("regex");
        let (filtered, has_errors, _) = filter_messages(messages, &cfg, Some(&pattern));
        assert!(has_errors);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].message, "Expected \"doctype\"");
    }
}
