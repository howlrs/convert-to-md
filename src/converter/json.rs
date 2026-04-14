use anyhow::Result;
use std::path::Path;

pub fn convert(path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(path)?;
    let title = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    // Pretty-print JSON
    let value: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("JSON parse error: {}", e))?;
    let pretty = serde_json::to_string_pretty(&value)?;

    Ok(format!("# {}\n\n```json\n{}\n```\n", title, pretty))
}
