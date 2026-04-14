use anyhow::Result;
use std::path::Path;

pub fn convert(path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(path)?;
    let md = htmd::convert(&content).unwrap_or_else(|_| content.clone());
    Ok(md)
}
