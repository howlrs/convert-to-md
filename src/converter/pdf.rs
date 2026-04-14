use anyhow::Result;
use std::path::Path;
use std::process::Command;

/// Convert PDF to Markdown.
///
/// Strategy:
///   1. Pure-Rust extraction via `pdf-extract` (no system dependencies).
///   2. If that yields empty text, fall back to `pdftotext` (poppler-utils)
///      when available — useful for some complex or encrypted PDFs.
///   3. If both produce no text the PDF is likely image-based (OCR required).
pub fn convert(path: &Path) -> Result<String> {
    let title = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    // --- 1. Native Rust extraction (works on Windows/Linux without extra installs) ---
    match pdf_extract::extract_text(path) {
        Ok(text) if !text.trim().is_empty() => {
            return Ok(format!("# {}\n\n{}", title, text.trim()));
        }
        Ok(_) => {
            // Empty text — PDF may be image-based; fall through to pdftotext
        }
        Err(_) => {
            // Parse error — fall through to pdftotext
        }
    }

    // --- 2. Optional fallback: pdftotext (poppler-utils) ---
    let result = Command::new("pdftotext")
        .args(["-layout", "-enc", "UTF-8", path.to_str().unwrap_or(""), "-"])
        .output();

    match result {
        Ok(output) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout).into_owned();
            if text.trim().is_empty() {
                anyhow::bail!(
                    "PDF text extraction returned empty output — \
                    the PDF may be image-based (scanned). OCR is required."
                );
            }
            Ok(format!("# {}\n\n{}", title, text.trim()))
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("pdftotext failed: {}", stderr.trim())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // pdftotext unavailable and native extraction yielded nothing
            anyhow::bail!(
                "PDF appears to be image-based (scanned). \
                OCR is required to extract text. \
                Tip: install poppler-utils (`apt-get install poppler-utils` / \
                `choco install poppler`) for additional fallback support."
            )
        }
        Err(e) => anyhow::bail!("failed to run pdftotext: {}", e),
    }
}
