use anyhow::Result;
use calamine::{open_workbook_auto, Data, Reader};
use std::path::Path;

pub fn convert(path: &Path) -> Result<String> {
    let mut workbook = open_workbook_auto(path)?;

    let title = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    let mut output = format!("# {}\n\n", title);

    let sheet_names: Vec<String> = workbook.sheet_names().to_owned();

    for sheet_name in &sheet_names {
        match workbook.worksheet_range(&sheet_name) {
            Ok(range) => {
                output.push_str(&format!("## {}\n\n", sheet_name));

                let rows: Vec<Vec<String>> = range
                    .rows()
                    .map(|row| row.iter().map(cell_to_string).collect())
                    .collect();

                if rows.is_empty() {
                    output.push_str("*(empty sheet)*\n\n");
                    continue;
                }

                // Header row
                let header = &rows[0];
                if !header.is_empty() {
                    output.push_str("| ");
                    output.push_str(&header.join(" | "));
                    output.push_str(" |\n");

                    // Separator
                    output.push_str("| ");
                    output.push_str(
                        &header
                            .iter()
                            .map(|_| "---")
                            .collect::<Vec<_>>()
                            .join(" | "),
                    );
                    output.push_str(" |\n");
                }

                // Data rows
                let col_count = header.len().max(1);
                for row in rows.iter().skip(1) {
                    if row.iter().all(|c| c.is_empty()) {
                        continue;
                    }
                    let mut padded = row.clone();
                    padded.resize(col_count, String::new());

                    output.push_str("| ");
                    output.push_str(&padded.join(" | "));
                    output.push_str(" |\n");
                }
                output.push('\n');
            }
            Err(e) => {
                output.push_str(&format!("## {}\n\n*Error reading sheet: {}*\n\n", sheet_name, e));
            }
        }
    }

    Ok(output)
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::String(s) => s.replace('|', "\\|").replace('\n', " "),
        Data::Float(f) => {
            if f.fract() == 0.0 && f.abs() < 1e15 {
                format!("{}", *f as i64)
            } else {
                format!("{}", f)
            }
        }
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::Empty => String::new(),
        Data::Error(e) => format!("{:?}", e),
        Data::DateTime(dt) => format!("{:?}", dt),
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
    }
}
