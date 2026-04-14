use anyhow::Result;
use std::path::Path;

pub fn convert(path: &Path) -> Result<String> {
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

    Ok(output)
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
