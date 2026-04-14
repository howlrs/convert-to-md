use anyhow::Result;
use std::path::Path;

pub fn convert(path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(path)?;
    Ok(content)
}
