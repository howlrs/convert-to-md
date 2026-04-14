use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub fn convert(path: &Path, ocr: bool) -> Result<String> {
    let filename = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    let meta = std::fs::metadata(path)?;
    let size_kb = meta.len() as f64 / 1024.0;

    let mut output = format!("# {}\n\n", filename);
    output.push_str("## File Info\n\n");
    output.push_str(&format!("- **Filename**: {}\n", filename));
    output.push_str(&format!("- **Size**: {:.1} KB ({} bytes)\n", size_kb, meta.len()));

    // EXIF metadata
    match read_exif(path) {
        Ok(exif_md) if !exif_md.is_empty() => {
            output.push_str("\n## EXIF Metadata\n\n");
            output.push_str(&exif_md);
        }
        Ok(_) => {
            output.push_str("\n*No EXIF metadata found.*\n");
        }
        Err(_) => {
            output.push_str("\n*EXIF metadata not available for this file.*\n");
        }
    }

    if ocr {
        match run_tesseract(path) {
            Ok(text) if !text.trim().is_empty() => {
                output.push_str("\n## OCR Text\n\n");
                output.push_str(text.trim());
                output.push('\n');
            }
            Ok(_) => {
                output.push_str("\n*OCR produced no text — image may not contain readable text.*\n");
            }
            Err(e) => {
                output.push_str(&format!("\n*OCR error: {}*\n", e));
            }
        }
    }

    Ok(output)
}

fn run_tesseract(path: &Path) -> Result<String> {
    let result = Command::new("tesseract")
        .args([path.to_str().unwrap_or(""), "stdout", "-l", "eng"])
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                anyhow::anyhow!("tesseract not found — install tesseract-ocr")
            } else {
                anyhow::anyhow!("failed to run tesseract: {}", e)
            }
        })?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        anyhow::bail!("tesseract failed: {}", stderr.trim());
    }

    Ok(String::from_utf8_lossy(&result.stdout).into_owned())
}

fn read_exif(path: &Path) -> Result<String> {
    use exif::In;
    let file = std::fs::File::open(path)?;
    let mut buf = std::io::BufReader::new(file);
    let exif = exif::Reader::new().read_from_container(&mut buf)?;

    let mut lines = String::new();
    for field in exif.fields() {
        if field.ifd_num == In::PRIMARY {
            lines.push_str(&format!(
                "- **{}**: {}\n",
                field.tag,
                field.display_value().with_unit(&exif)
            ));
        }
    }
    Ok(lines)
}
