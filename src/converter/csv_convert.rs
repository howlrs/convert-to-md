use anyhow::Result;
use std::path::Path;

pub fn convert(path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(path)?;
    let title = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(content.as_bytes());

    let headers: Vec<String> = rdr
        .headers()?
        .iter()
        .map(|h| h.replace('|', "\\|"))
        .collect();

    if headers.is_empty() {
        return Ok(format!("# {}\n\n*(empty)*\n", title));
    }

    let mut output = format!("# {}\n\n", title);

    // Header row
    output.push_str("| ");
    output.push_str(&headers.join(" | "));
    output.push_str(" |\n");

    // Separator
    output.push_str("| ");
    output.push_str(
        &headers
            .iter()
            .map(|_| "---")
            .collect::<Vec<_>>()
            .join(" | "),
    );
    output.push_str(" |\n");

    // Data rows
    for result in rdr.records() {
        let record = result?;
        if record.iter().all(|f| f.is_empty()) {
            continue;
        }
        let mut cells: Vec<String> = record
            .iter()
            .map(|f| f.replace('|', "\\|").replace('\n', " "))
            .collect();
        cells.resize(headers.len(), String::new());

        output.push_str("| ");
        output.push_str(&cells.join(" | "));
        output.push_str(" |\n");
    }
    output.push('\n');

    Ok(output)
}
