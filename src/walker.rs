use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    // Documents
    "pdf", "docx", "doc", "pptx", "ppt", "xlsx", "xls",
    // Archives
    "zip",
    // Text-based
    "html", "htm", "csv", "json", "xml", "txt",
    // Images
    "jpg", "jpeg", "png", "gif", "bmp", "tiff", "tif", "webp",
];

pub fn collect_files(input: &Path) -> Vec<PathBuf> {
    if input.is_file() {
        let ext = ext_lower(input);
        if SUPPORTED_EXTENSIONS.contains(&ext.as_str()) {
            return vec![input.to_owned()];
        } else {
            eprintln!(
                "  [WARN] Unsupported extension: .{} ({})",
                ext,
                input.display()
            );
            return vec![];
        }
    }

    let mut files: Vec<PathBuf> = WalkDir::new(input)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            if !e.file_type().is_file() {
                return false;
            }
            let name = e.file_name().to_string_lossy();
            if name.starts_with('.') {
                return false;
            }
            let ext = ext_lower(e.path());
            SUPPORTED_EXTENSIONS.contains(&ext.as_str())
        })
        .map(|e| e.into_path())
        .collect();

    files.sort();
    files
}

fn ext_lower(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
}
