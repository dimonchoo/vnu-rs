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
