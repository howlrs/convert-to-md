mod cli;
mod converter;
mod walker;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use rayon::prelude::*;

use cli::Cli;
use converter::convert_file;
use walker::collect_files;

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", "[ERROR]".red().bold(), e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    if cli.jobs > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(cli.jobs)
            .build_global()
            .ok();
    }

    if !cli.input.exists() {
        anyhow::bail!(
            "Input path does not exist: {}\n  Place files in resources/ and run again.",
            cli.input.display()
        );
    }

    let files = collect_files(&cli.input);

    if files.is_empty() {
        println!(
            "{} No supported files found in: {}",
            "[INFO]".cyan(),
            cli.input.display()
        );
        println!(
            "  Supported extensions: {}",
            walker::SUPPORTED_EXTENSIONS
                .iter()
                .map(|e| format!(".{}", e))
                .collect::<Vec<_>>()
                .join(", ")
        );
        return Ok(());
    }

    println!(
        "{} {} file(s)  input: {} → output: {}",
        "[INFO]".cyan().bold(),
        files.len(),
        cli.input.display(),
        cli.output.display()
    );
    println!();

    if cli.list {
        for f in &files {
            println!("  {}", f.display());
        }
        return Ok(());
    }

    std::fs::create_dir_all(&cli.output)?;

    let ok = Arc::new(AtomicUsize::new(0));
    let skip = Arc::new(AtomicUsize::new(0));
    let err = Arc::new(AtomicUsize::new(0));

    files.par_iter().for_each(|src| {
        let out_dir = compute_output_dir(src, &cli.input, &cli.output);
        match process_file(src, &out_dir, cli.overwrite) {
            Status::Ok => {
                ok.fetch_add(1, Ordering::Relaxed);
            }
            Status::Skip => {
                skip.fetch_add(1, Ordering::Relaxed);
            }
            Status::Err => {
                err.fetch_add(1, Ordering::Relaxed);
            }
        }
    });

    println!();
    println!(
        "Done: {} ok / {} skip / {} error  (total {})",
        ok.load(Ordering::Relaxed).to_string().green(),
        skip.load(Ordering::Relaxed).to_string().yellow(),
        err.load(Ordering::Relaxed).to_string().red(),
        files.len()
    );
    if ok.load(Ordering::Relaxed) > 0 {
        let out_abs = cli
            .output
            .canonicalize()
            .unwrap_or_else(|_| cli.output.clone());
        println!("Output: {}", out_abs.display());
    }

    Ok(())
}

enum Status {
    Ok,
    Skip,
    Err,
}

fn compute_output_dir(src: &Path, input_base: &Path, output_base: &Path) -> PathBuf {
    if input_base.is_dir() {
        if let Ok(rel) = src.strip_prefix(input_base) {
            if let Some(parent) = rel.parent() {
                return output_base.join(parent);
            }
        }
    }
    output_base.to_owned()
}

fn process_file(src: &Path, out_dir: &Path, overwrite: bool) -> Status {
    let stem = src.file_stem().unwrap_or_default().to_string_lossy();
    let dest = out_dir.join(format!("{}.md", stem));

    if dest.exists() && !overwrite {
        println!(
            "  {} {} (use --overwrite to replace)",
            "[SKIP]".yellow(),
            src.file_name().unwrap_or_default().to_string_lossy()
        );
        return Status::Skip;
    }

    match convert_file(src) {
        Ok(content) if content.trim().is_empty() => {
            println!(
                "  {} {} → conversion result is empty",
                "[WARN]".yellow(),
                src.file_name().unwrap_or_default().to_string_lossy()
            );
            Status::Err
        }
        Ok(content) => {
            if let Err(e) = write_output(&dest, &content) {
                println!(
                    "  {} {} → write error: {}",
                    "[ERR]".red(),
                    src.file_name().unwrap_or_default().to_string_lossy(),
                    e
                );
                return Status::Err;
            }
            println!(
                "  {}  {} → {}",
                "[OK]".green(),
                src.file_name().unwrap_or_default().to_string_lossy(),
                dest.display()
            );
            Status::Ok
        }
        Err(e) => {
            println!(
                "  {} {} → {}",
                "[ERR]".red(),
                src.file_name().unwrap_or_default().to_string_lossy(),
                e
            );
            Status::Err
        }
    }
}

fn write_output(dest: &Path, content: &str) -> Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(dest, content)?;
    Ok(())
}
