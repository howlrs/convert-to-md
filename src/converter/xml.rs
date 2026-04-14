use anyhow::Result;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::path::Path;

pub fn convert(path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(path)?;
    let title = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    let mut output = format!("# {}\n\n", title);
    output.push_str(&extract_xml_text(&content)?);
    Ok(output)
}

fn extract_xml_text(xml: &str) -> Result<String> {
    let mut reader = Reader::from_str(xml);
    let mut output = String::new();
    let mut depth = 0usize;

    loop {
        match reader.read_event()? {
            Event::Start(_) => depth += 1,
            Event::End(_) => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    output.push('\n');
                }
            }
            Event::Text(e) => {
                if let Ok(text) = e.unescape() {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        let indent = "  ".repeat(depth.saturating_sub(1));
                        output.push_str(&format!("{}{}\n", indent, trimmed));
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(output)
}
