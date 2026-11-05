use vnu_core::VNuReport;
use vnu_parser::ValidatorEngine;

pub fn serialize_report_json(report: &VNuReport, pretty: bool) -> Result<String, String> {
    let json_res = if pretty {
        serde_json::to_string_pretty(report)
    } else {
        serde_json::to_string(report)
    };
    json_res.map_err(|e| format!("Serialization error: {}", e))
}

pub fn validate_content(
    engine: &mut ValidatorEngine,
    content: &str,
    pretty: bool,
) -> Result<(VNuReport, String), String> {
    let report = engine.validate(content);
    let json = serialize_report_json(&report, pretty)?;
    Ok((report, json))
}
