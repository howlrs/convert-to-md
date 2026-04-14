use anyhow::Result;
use std::path::Path;
use std::process::Command;

/// Convert PDF to Markdown via `pdftotext` (poppler-utils).
/// In the Docker image, poppler-utils is installed in the runtime stage.
pub fn convert(path: &Path) -> Result<String> {
    let title = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    // Try pdftotext (poppler)
    let result = Command::new("pdftotext")
        .args(["-layout", "-enc", "UTF-8", path.to_str().unwrap_or(""), "-"])
        .output();

    match result {
        Ok(output) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout).into_owned();
            if text.trim().is_empty() {
                anyhow::bail!("pdftotext returned empty output — the PDF may be image-based (OCR required)");
            }
            Ok(format!("# {}\n\n{}", title, text))
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("pdftotext failed: {}", stderr.trim())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            anyhow::bail!(
                "pdftotext not found. \
                Install poppler-utils: `apt-get install poppler-utils` \
                or use the Docker image which includes it."
            )
        }
        Err(e) => anyhow::bail!("failed to run pdftotext: {}", e),
    }
}
