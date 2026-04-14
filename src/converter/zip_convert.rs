use anyhow::Result;
use std::path::Path;
use zip::ZipArchive;

use super::convert_file;
use crate::walker::SUPPORTED_EXTENSIONS;

pub fn convert(path: &Path, ocr: bool) -> Result<String> {
    let file = std::fs::File::open(path)?;
    let mut archive = ZipArchive::new(file)?;

    let title = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    let mut output = format!("# {}\n\n", title);

    // Collect supported entry names (excluding nested ZIPs to prevent recursion)
    let mut entries: Vec<String> = archive
        .file_names()
        .filter(|n| !n.ends_with('/') && !n.starts_with('.'))
        .filter_map(|n| {
            let ext = Path::new(n)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            if ext != "zip" && SUPPORTED_EXTENSIONS.contains(&ext.as_str()) {
                Some(n.to_string())
            } else {
                None
            }
        })
        .collect();
    entries.sort();

    if entries.is_empty() {
        output.push_str("*No supported files found in archive.*\n");
        return Ok(output);
    }

    // Unique temp directory per process+archive to avoid collisions
    let work_dir = std::env::temp_dir().join(format!(
        "convert-to-md-{}-{}",
        std::process::id(),
        sanitize(&title)
    ));
    std::fs::create_dir_all(&work_dir)?;

    for entry_name in &entries {
        let result = extract_and_convert(&mut archive, entry_name, &work_dir, ocr);
        match result {
            Ok(content) => {
                output.push_str(&format!("## {}\n\n", entry_name));
                output.push_str(&content);
                output.push('\n');
            }
            Err(e) => {
                output.push_str(&format!(
                    "## {}\n\n*Conversion error: {}*\n\n",
                    entry_name, e
                ));
            }
        }
    }

    let _ = std::fs::remove_dir_all(&work_dir);
    Ok(output)
}

fn extract_and_convert(
    archive: &mut ZipArchive<std::fs::File>,
    name: &str,
    work_dir: &Path,
    ocr: bool,
) -> Result<String> {
    let file_name = Path::new(name)
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("invalid entry name: {}", name))?;
    let dest = work_dir.join(file_name);

    {
        let mut entry = archive.by_name(name)?;
        let mut out = std::fs::File::create(&dest)?;
        std::io::copy(&mut entry, &mut out)?;
    }

    convert_file(&dest, ocr)
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
