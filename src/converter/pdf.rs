use anyhow::Result;
use std::path::Path;
use std::process::Command;

/// Convert PDF to Markdown.
///
/// Strategy:
///   1. Pure-Rust extraction via `pdf-extract` (no system dependencies).
///   2. If that yields empty text, fall back to `pdftotext` (poppler-utils)
///      when available — useful for some complex or encrypted PDFs.
///   3. If `ocr=true`, fall back to `pdftoppm` + `tesseract` page-by-page OCR.
pub fn convert(path: &Path, ocr: bool) -> Result<String> {
    let title = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    // --- 1. Native Rust extraction ---
    match pdf_extract::extract_text(path) {
        Ok(text) if !text.trim().is_empty() => {
            return Ok(format!("# {}\n\n{}", title, text.trim()));
        }
        Ok(_) => {}
        Err(_) => {}
    }

    // --- 2. Optional fallback: pdftotext ---
    let pdftotext_result = Command::new("pdftotext")
        .args(["-layout", "-enc", "UTF-8", path.to_str().unwrap_or(""), "-"])
        .output();

    match pdftotext_result {
        Ok(output) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout).into_owned();
            if !text.trim().is_empty() {
                return Ok(format!("# {}\n\n{}", title, text.trim()));
            }
        }
        Ok(_) => {}
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            let _ = e; // pdftotext found but failed — continue to OCR
        }
        Err(_) => {} // pdftotext not installed — continue
    }

    // --- 3. OCR fallback (requires --ocr flag) ---
    if ocr {
        return ocr_pdf(path, &title);
    }

    anyhow::bail!(
        "PDF appears to be image-based (scanned). \
        Use --ocr to extract text via tesseract OCR, or install \
        poppler-utils for additional fallback support."
    )
}

fn ocr_pdf(path: &Path, title: &str) -> Result<String> {
    let tmp = std::env::temp_dir().join(format!("convert-to-md-ocr-{}", std::process::id()));
    std::fs::create_dir_all(&tmp)?;

    let prefix = tmp.join("page");
    let status = Command::new("pdftoppm")
        .args([
            "-png",
            "-r",
            "150",
            path.to_str().unwrap_or(""),
            prefix.to_str().unwrap_or(""),
        ])
        .status()
        .map_err(|e| {
            let _ = std::fs::remove_dir_all(&tmp);
            if e.kind() == std::io::ErrorKind::NotFound {
                anyhow::anyhow!("pdftoppm not found — install poppler-utils for PDF OCR")
            } else {
                anyhow::anyhow!("failed to run pdftoppm: {}", e)
            }
        })?;

    if !status.success() {
        let _ = std::fs::remove_dir_all(&tmp);
        anyhow::bail!("pdftoppm failed converting PDF to images");
    }

    let mut pages: Vec<std::path::PathBuf> = std::fs::read_dir(&tmp)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|e| e == "png").unwrap_or(false))
        .collect();
    pages.sort();

    let mut text = String::new();
    for page_path in &pages {
        match Command::new("tesseract")
            .args([page_path.to_str().unwrap_or(""), "stdout", "-l", "eng"])
            .output()
        {
            Ok(o) if o.status.success() => {
                text.push_str(&String::from_utf8_lossy(&o.stdout));
            }
            Ok(o) => {
                eprintln!(
                    "  [WARN] tesseract: {}",
                    String::from_utf8_lossy(&o.stderr).trim()
                );
            }
            Err(e) => {
                let _ = std::fs::remove_dir_all(&tmp);
                if e.kind() == std::io::ErrorKind::NotFound {
                    anyhow::bail!("tesseract not found — install tesseract-ocr");
                }
                anyhow::bail!("failed to run tesseract: {}", e);
            }
        }
    }

    let _ = std::fs::remove_dir_all(&tmp);

    if text.trim().is_empty() {
        anyhow::bail!("OCR produced no text from PDF");
    }

    Ok(format!("# {}\n\n{}", title, text.trim()))
}
