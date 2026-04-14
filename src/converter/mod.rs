mod csv_convert;
mod docx;
mod html;
mod image;
mod json;
mod pdf;
mod plain_text;
mod pptx;
mod xlsx;
mod xml;

use anyhow::Result;
use std::path::Path;

pub fn convert_file(path: &Path) -> Result<String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "docx" | "doc" => docx::convert(path),
        "pptx" | "ppt" => pptx::convert(path),
        "xlsx" | "xls" => xlsx::convert(path),
        "csv" => csv_convert::convert(path),
        "html" | "htm" => html::convert(path),
        "txt" => plain_text::convert(path),
        "json" => json::convert(path),
        "xml" => xml::convert(path),
        "pdf" => pdf::convert(path),
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "tif" | "webp" => image::convert(path),
        _ => anyhow::bail!("Unsupported format: .{}", ext),
    }
}
