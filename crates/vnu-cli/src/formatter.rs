use crate::format::OutputFormat;
use crate::service::serialize_report_json;
use vnu_core::{MessageSubType, MessageType, VNuMessage, VNuReport};

pub fn format_messages(messages: &[VNuMessage], format: OutputFormat) -> Result<String, String> {
    match format {
        OutputFormat::Json => {
            let mut report = VNuReport::new();
            report.messages = messages.to_vec();
            serialize_report_json(&report, false)
        }
        OutputFormat::Gnu => Ok(messages
            .iter()
            .map(format_gnu_message)
            .collect::<Vec<_>>()
            .join("\n")),
        OutputFormat::Text => {
            let mut out = String::new();
            for message in messages {
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str(&format_text_message(message));
            }
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(text_summary(messages));
            Ok(out)
        }
        OutputFormat::Xml => Ok(format_xml(messages)),
    }
}

fn format_gnu_message(message: &VNuMessage) -> String {
    let system_id = gnu_system_id(message.url.as_deref().unwrap_or(""));
    let location = gnu_location(message);
    format!(
        "{system_id}:{location}{}: {}",
        gnu_flat_type(message),
        message.message
    )
}

fn gnu_system_id(url: &str) -> String {
    let mut quoted = String::from("\"");
    for ch in url.chars() {
        if ch == '"' {
            quoted.push_str("%22");
        } else {
            quoted.push(ch);
        }
    }
    quoted.push('"');
    quoted
}

fn gnu_location(message: &VNuMessage) -> String {
    let Some(last_line) = message.last_line else {
        return " ".to_string();
    };
    let mut location = String::new();
    if let Some(first_line) = message.first_line {
        location.push_str(&first_line.to_string());
        if message.first_column.is_some() && message.last_column.is_some() {
            location.push('.');
            location.push_str(&message.first_column.unwrap().to_string());
        }
        location.push('-');
    }
    location.push_str(&last_line.to_string());
    if message.first_column.is_some() && message.last_column.is_some() {
        location.push('.');
        location.push_str(&message.last_column.unwrap().to_string());
    }
    location.push(':');
    location.push(' ');
    location
}

fn gnu_flat_type(message: &VNuMessage) -> &'static str {
    match (message.msg_type, message.sub_type) {
        (MessageType::Error, None) => "error",
        (MessageType::Error, Some(MessageSubType::Fatal)) => "error fatal",
        (MessageType::Error, Some(MessageSubType::Warning)) => "error warning",
        (MessageType::Error, Some(MessageSubType::Schema)) => "error schema",
        (MessageType::Error, Some(MessageSubType::Internal)) => "error internal",
        (MessageType::Info, None) => "info",
        (MessageType::Info, Some(MessageSubType::Warning)) => "info warning",
        (MessageType::Info, Some(MessageSubType::Fatal)) => "info fatal",
        (MessageType::Info, Some(MessageSubType::Schema)) => "info schema",
        (MessageType::Info, Some(MessageSubType::Internal)) => "info internal",
        (MessageType::NonDocumentError, None) => "non-document-error",
        (MessageType::NonDocumentError, Some(MessageSubType::Warning)) => {
            "non-document-error warning"
        }
        (MessageType::NonDocumentError, Some(MessageSubType::Fatal)) => "non-document-error fatal",
        (MessageType::NonDocumentError, Some(MessageSubType::Schema)) => {
            "non-document-error schema"
        }
        (MessageType::NonDocumentError, Some(MessageSubType::Internal)) => {
            "non-document-error internal"
        }
    }
}

fn format_text_message(message: &VNuMessage) -> String {
    let mut out = format!("{}: {}", text_presentation(message), message.message);
    let location = text_location(message);
    if !location.is_empty() {
        out.push('\n');
        out.push_str(&location);
    }
    out
}

fn text_presentation(message: &VNuMessage) -> &'static str {
    match (message.msg_type, message.sub_type) {
        (MessageType::Error, Some(MessageSubType::Fatal)) => "Fatal Error",
        (MessageType::Error, _) => "Error",
        (MessageType::Info, Some(MessageSubType::Warning)) => "Warning",
        (MessageType::Info, _) => "Info",
        (MessageType::NonDocumentError, Some(MessageSubType::Schema)) => "Schema Error",
        (MessageType::NonDocumentError, Some(MessageSubType::Internal)) => "Internal Error",
        (MessageType::NonDocumentError, _) => "Non-Document Error",
    }
}

fn text_location(message: &VNuMessage) -> String {
    let Some(last_line) = message.last_line else {
        return message.url.clone().unwrap_or_default();
    };
    let location = if message.last_column.is_none() {
        format!("On line {last_line}")
    } else if message.first_line.is_none()
        || (message.first_line == Some(last_line) && message.first_column == message.last_column)
    {
        format!(
            "At line {last_line}, column {}",
            message.last_column.unwrap()
        )
    } else {
        let first_line = message.first_line.unwrap_or(last_line);
        let first_column = message
            .first_column
            .unwrap_or(message.last_column.unwrap_or(1));
        let last_column = message.last_column.unwrap();
        format!(
            "From line {first_line}, column {first_column}; to line {last_line}, column {last_column}"
        )
    };
    match &message.url {
        Some(url) => format!("{location} in resource {url}"),
        None => location,
    }
}

fn text_summary(messages: &[VNuMessage]) -> &'static str {
    if messages
        .iter()
        .any(|message| message.msg_type == MessageType::NonDocumentError)
    {
        "The result cannot be determined due to a non-document-error."
    } else if messages
        .iter()
        .any(|message| message.msg_type == MessageType::Error)
    {
        "Document checking completed."
    } else {
        "Document checking completed. No errors found."
    }
}

fn format_xml(messages: &[VNuMessage]) -> String {
    let mut out = String::from(
        "<?xml version='1.0' encoding='utf-8'?>\n<messages xmlns=\"http://n.validator.nu/messages/\">",
    );
    for message in messages {
        out.push('\n');
        out.push('<');
        out.push_str(xml_element(message));
        out.push_str(&xml_attrs(message));
        out.push_str(">\n<message>");
        out.push_str(&quick_xml_escape(&message.message));
        out.push_str("</message>\n");
        if let Some(extract) = &message.extract {
            out.push_str("<extract>");
            out.push_str(&quick_xml_escape(extract));
            out.push_str("</extract>\n");
        }
        out.push_str("</");
        out.push_str(xml_element(message));
        out.push('>');
    }
    out.push_str("\n</messages>");
    out
}

fn xml_element(message: &VNuMessage) -> &'static str {
    match message.msg_type {
        MessageType::Error => "error",
        MessageType::NonDocumentError => "non-document-error",
        MessageType::Info => "info",
    }
}

fn xml_attrs(message: &VNuMessage) -> String {
    let mut attrs = String::new();
    if let Some(url) = &message.url {
        attrs.push_str(" url=\"");
        attrs.push_str(&quick_xml_escape(url));
        attrs.push('"');
    }
    if let Some(last_line) = message.last_line {
        attrs.push_str(&format!(" last-line=\"{last_line}\""));
        if let Some(first_line) = message.first_line {
            if first_line != last_line {
                attrs.push_str(&format!(" first-line=\"{first_line}\""));
            }
        }
        if let Some(last_column) = message.last_column {
            attrs.push_str(&format!(" last-column=\"{last_column}\""));
            if let Some(first_column) = message.first_column {
                if first_column != last_column {
                    attrs.push_str(&format!(" first-column=\"{first_column}\""));
                }
            }
        }
    }
    if let Some(sub_type) = message.sub_type {
        let name = match sub_type {
            MessageSubType::Warning => "warning",
            MessageSubType::Fatal => "fatal",
            MessageSubType::Schema => "schema",
            MessageSubType::Internal => "internal",
        };
        attrs.push_str(&format!(" type=\"{name}\""));
    }
    attrs
}

fn quick_xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ranged_error() -> VNuMessage {
        let mut message = VNuMessage::error("Bad value “x” & <y>", 3, 4);
        message.first_line = Some(1);
        message.first_column = Some(2);
        message.with_url("file:/tmp/a.html")
    }

    #[test]
    fn gnu_spans_first_and_last_and_escapes_quotes() {
        let mut message = ranged_error();
        message.url = Some("file:/tmp/a\"b.html".to_string());
        let formatted = format_messages(&[message], OutputFormat::Gnu).expect("gnu");
        assert_eq!(
            formatted,
            "\"file:/tmp/a%22b.html\":1.2-3.4: error: Bad value “x” & <y>"
        );
    }

    #[test]
    fn gnu_omits_columns_when_last_column_is_missing() {
        let mut message = VNuMessage::error("span", 3, 1);
        message.first_line = Some(1);
        message.first_column = Some(2);
        message.last_column = None;
        message.url = Some("file:/tmp/a.html".to_string());
        let formatted = format_messages(&[message], OutputFormat::Gnu).expect("gnu");
        assert_eq!(formatted, "\"file:/tmp/a.html\":1-3: error: span");
    }

    #[test]
    fn gnu_read_error_has_no_location() {
        let message = VNuMessage::non_document_error("Error reading missing.html: not found")
            .with_url("file:/tmp/missing.html");
        let formatted = format_messages(&[message], OutputFormat::Gnu).expect("gnu");
        assert_eq!(
            formatted,
            "\"file:/tmp/missing.html\": non-document-error: Error reading missing.html: not found"
        );
    }

    #[test]
    fn text_uses_upstream_wording_and_one_summary() {
        let error = ranged_error();
        let warning = VNuMessage::warning("consider lang", 1, 1).with_url("file:/tmp/a.html");
        let formatted = format_messages(&[error, warning], OutputFormat::Text).expect("text");
        assert_eq!(
            formatted,
            "Error: Bad value “x” & <y>\n\
From line 1, column 2; to line 3, column 4 in resource file:/tmp/a.html\n\
Warning: consider lang\n\
At line 1, column 1 in resource file:/tmp/a.html\n\
Document checking completed."
        );
    }

    #[test]
    fn text_summary_is_indeterminate_for_non_document_errors() {
        let message =
            VNuMessage::non_document_error("cannot read").with_url("file:/tmp/missing.html");
        let formatted = format_messages(&[message], OutputFormat::Text).expect("text");
        assert!(formatted.contains("Non-Document Error: cannot read\nfile:/tmp/missing.html\n"));
        assert!(
            formatted.ends_with("The result cannot be determined due to a non-document-error.")
        );
    }

    #[test]
    fn xml_emits_one_document_with_range_attributes() {
        let mut message = ranged_error();
        message.extract = Some("a < b".to_string());
        let formatted =
            format_messages(std::slice::from_ref(&message), OutputFormat::Xml).expect("xml");
        assert_eq!(
            formatted,
            "<?xml version='1.0' encoding='utf-8'?>\n\
<messages xmlns=\"http://n.validator.nu/messages/\">\n\
<error url=\"file:/tmp/a.html\" last-line=\"3\" first-line=\"1\" last-column=\"4\" first-column=\"2\">\n\
<message>Bad value “x” &amp; &lt;y&gt;</message>\n\
<extract>a &lt; b</extract>\n\
</error>\n\
</messages>"
        );
    }

    #[test]
    fn json_is_one_report_for_multiple_files() {
        let first = VNuMessage::error("one", 1, 1).with_url("file:/tmp/a.html");
        let second = VNuMessage::error("two", 2, 2).with_url("file:/tmp/b.html");
        let formatted = format_messages(&[first, second], OutputFormat::Json).expect("json");
        let value: serde_json::Value = serde_json::from_str(&formatted).expect("one json value");
        assert_eq!(value["messages"].as_array().expect("messages").len(), 2);
        assert_eq!(value["messages"][0]["url"], "file:/tmp/a.html");
        assert_eq!(value["messages"][1]["url"], "file:/tmp/b.html");
    }
}
