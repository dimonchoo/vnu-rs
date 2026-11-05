use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Error,
    Info,
    #[serde(rename = "non-document-error")]
    NonDocumentError,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageSubType {
    Warning,
    Fatal,
    Schema,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VNuMessage {
    #[serde(rename = "type")]
    pub msg_type: MessageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_type: Option<MessageSubType>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extract: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_column: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_column: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hilite_start: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hilite_length: Option<usize>,
}

impl VNuMessage {
    pub fn error(message: impl Into<String>, line: usize, col: usize) -> Self {
        Self {
            msg_type: MessageType::Error,
            sub_type: None,
            message: message.into(),
            extract: None,
            last_line: Some(line),
            first_line: Some(line),
            last_column: Some(col),
            first_column: Some(col),
            hilite_start: None,
            hilite_length: None,
        }
    }

    pub fn warning(message: impl Into<String>, line: usize, col: usize) -> Self {
        Self {
            msg_type: MessageType::Info,
            sub_type: Some(MessageSubType::Warning),
            message: message.into(),
            extract: None,
            last_line: Some(line),
            first_line: Some(line),
            last_column: Some(col),
            first_column: Some(col),
            hilite_start: None,
            hilite_length: None,
        }
    }

    pub fn info(message: impl Into<String>, line: usize, col: usize) -> Self {
        Self {
            msg_type: MessageType::Info,
            sub_type: None,
            message: message.into(),
            extract: None,
            last_line: Some(line),
            first_line: Some(line),
            last_column: Some(col),
            first_column: Some(col),
            hilite_start: None,
            hilite_length: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct VNuReport {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub messages: Vec<VNuMessage>,
}

impl VNuReport {
    pub fn new() -> Self {
        Self {
            version: Some(format!("vnu-rs {}", env!("CARGO_PKG_VERSION"))),
            messages: Vec::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.messages.iter().any(|m| {
            m.msg_type == MessageType::Error || m.msg_type == MessageType::NonDocumentError
        })
    }

    pub fn error_count(&self) -> usize {
        self.messages
            .iter()
            .filter(|m| {
                m.msg_type == MessageType::Error || m.msg_type == MessageType::NonDocumentError
            })
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.messages
            .iter()
            .filter(|m| {
                m.msg_type == MessageType::Info && m.sub_type == Some(MessageSubType::Warning)
            })
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_official_vnu_json() {
        let sample = r#"{
          "version": "26.11.4 (26b30ab)",
          "messages": [
            {
              "type": "error",
              "lastLine": 1,
              "lastColumn": 6,
              "firstColumn": 1,
              "message": "Start tag seen without seeing a doctype first. Expected “<!DOCTYPE html>”.",
              "extract": "<html><body>",
              "hiliteStart": 0,
              "hiliteLength": 6
            },
            {
              "type": "info",
              "lastLine": 1,
              "lastColumn": 6,
              "firstColumn": 1,
              "subType": "warning",
              "message": "Consider adding a “lang” attribute to the “html” start tag to declare the language of this document.",
              "extract": "<html><body>",
              "hiliteStart": 0,
              "hiliteLength": 6
            }
          ]
        }"#;

        let report: VNuReport =
            serde_json::from_str(sample).expect("failed to deserialize official vnu report");
        assert_eq!(report.messages.len(), 2);
        assert_eq!(report.messages[0].msg_type, MessageType::Error);
        assert_eq!(report.messages[0].last_line, Some(1));
        assert_eq!(report.messages[0].last_column, Some(6));
        assert_eq!(report.messages[0].first_column, Some(1));
        assert_eq!(report.messages[1].msg_type, MessageType::Info);
        assert_eq!(report.messages[1].sub_type, Some(MessageSubType::Warning));
        assert_eq!(report.error_count(), 1);
        assert_eq!(report.warning_count(), 1);
        assert!(!report.is_valid());
    }
}
