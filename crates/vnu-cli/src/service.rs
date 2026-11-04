use vnu_core::VNuReport;
use vnu_parser::ValidatorEngine;

pub fn validate_content(
    engine: &mut ValidatorEngine,
    content: &str,
    pretty: bool,
) -> Result<(VNuReport, String), String> {
    let report = engine.validate(content);
    let json_res = if pretty {
        serde_json::to_string_pretty(&report)
    } else {
        serde_json::to_string(&report)
    };
    match json_res {
        Ok(json) => Ok((report, json)),
        Err(e) => Err(format!("Serialization error: {}", e)),
    }
}
