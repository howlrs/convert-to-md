use clap::Parser;
use std::path::PathBuf;

/// Convert documents, PDFs, Office files, images, and more to Markdown.
///
/// Supported formats: pdf, docx, pptx, xlsx, xls, html, csv, json, xml, txt,
///                    jpg, png, gif, bmp, tiff, webp
#[derive(Parser, Debug)]
#[command(name = "convert-to-md-rs")]
#[command(version)]
#[command(about = "Convert various file formats to Markdown")]
pub struct Cli {
    /// Input file or directory
    #[arg(short, long, default_value = "resources")]
    pub input: PathBuf,

    /// Output directory for generated Markdown files
    #[arg(short, long, default_value = "data/output/markdown")]
    pub output: PathBuf,

    /// Overwrite existing output files
    #[arg(long)]
    pub overwrite: bool,

    /// List files to be converted without converting
    #[arg(long)]
    pub list: bool,

    /// Number of parallel jobs (0 = use all CPU cores)
    #[arg(short, long, default_value = "0")]
    pub jobs: usize,
}
