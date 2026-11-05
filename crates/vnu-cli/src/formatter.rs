use crate::format::OutputFormat;
use crate::service::serialize_report_json;
use vnu_core::{MessageSubType, MessageType, VNuMessage, VNuReport};

pub fn format_report(
    file_path: &str,
    messages: Vec<VNuMessage>,
    format: OutputFormat,
) -> Result<String, String> {
    match format {
        OutputFormat::Json => {
            let mut report = VNuReport::new();
            report.messages = messages;
            serialize_report_json(&report, false)
        }
        OutputFormat::Gnu => {
            let mut out = String::new();
            for (i, m) in messages.iter().enumerate() {
                if i > 0 {
                    out.push('\n');
                }
                let loc = match (m.last_line, m.first_column, m.last_column) {
                    (Some(l), Some(fc), Some(lc)) => format!("{}.{}-{}.{}: ", l, fc, l, lc),
                    (Some(l), None, Some(lc)) => format!("{}.{}: ", l, lc),
                    (Some(l), _, _) => format!("{}: ", l),
                    _ => String::new(),
                };
                let kind = match m.msg_type {
                    MessageType::Info if m.sub_type == Some(MessageSubType::Warning) => {
                        "info warning: "
                    }
                    MessageType::Info => "info: ",
                    MessageType::Error => "error: ",
                    MessageType::NonDocumentError => "fatal error: ",
                };
                let prefix = if file_path == "-" {
                    String::new()
                } else {
                    format!("\"{}\":", file_path)
                };
                out.push_str(&format!("{}{}{}{}", prefix, loc, kind, m.message));
            }
            Ok(out)
        }
        OutputFormat::Text => {
            let mut out = String::new();
            if messages.is_empty() {
                out.push_str(&format!("{}: The document is valid.", file_path));
            } else {
                for (i, m) in messages.iter().enumerate() {
                    if i > 0 {
                        out.push('\n');
                    }
                    let loc = match (m.last_line, m.last_column) {
                        (Some(l), Some(c)) => format!("{}:{}: ", l, c),
                        (Some(l), None) => format!("{}: ", l),
                        _ => String::new(),
                    };
                    let kind = match m.msg_type {
                        MessageType::Info if m.sub_type == Some(MessageSubType::Warning) => {
                            "warning"
                        }
                        MessageType::Info => "info",
                        MessageType::Error => "error",
                        MessageType::NonDocumentError => "error",
                    };
                    out.push_str(&format!("{}: {}{}: {}", file_path, loc, kind, m.message));
                }
            }
            Ok(out)
        }
        OutputFormat::Xml => {
            let mut out = String::from(
                "<?xml version='1.0' encoding='utf-8'?>\n<messages xmlns=\"http://n.validator.nu/messages/\">\n",
            );
            for m in &messages {
                let tag = match m.msg_type {
                    MessageType::Error => "error",
                    MessageType::NonDocumentError => "non-document-error",
                    MessageType::Info if m.sub_type == Some(MessageSubType::Warning) => {
                        "info type=\"warning\""
                    }
                    MessageType::Info => "info",
                };
                let line_attr = m
                    .last_line
                    .map(|l| format!(" last-line=\"{}\"", l))
                    .unwrap_or_default();
                let col_attr = m
                    .last_column
                    .map(|c| format!(" last-column=\"{}\"", c))
                    .unwrap_or_default();
                out.push_str(&format!("  <{} {}{}>\n", tag, line_attr, col_attr));
                out.push_str(&format!(
                    "    <message>{}</message>\n",
                    quick_xml_escape(&m.message)
                ));
                if let Some(ref ext) = m.extract {
                    out.push_str(&format!(
                        "    <extract>{}</extract>\n",
                        quick_xml_escape(ext)
                    ));
                }
                let end_tag = if tag.contains(' ') { "info" } else { tag };
                out.push_str(&format!("  </{}>\n", end_tag));
            }
            out.push_str("</messages>");
            Ok(out)
        }
    }
}

fn quick_xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
