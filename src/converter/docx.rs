use anyhow::Result;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

pub fn convert(path: &Path) -> Result<String> {
    let file = std::fs::File::open(path)?;
    let mut archive = ZipArchive::new(file)?;

    let xml_content = read_zip_entry(&mut archive, "word/document.xml")?;

    let title = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    let mut output = format!("# {}\n\n", title);
    output.push_str(&parse_document_xml(&xml_content)?);
    Ok(output)
}

fn read_zip_entry(archive: &mut ZipArchive<std::fs::File>, name: &str) -> Result<String> {
    let mut entry = archive
        .by_name(name)
        .map_err(|_| anyhow::anyhow!("'{}' not found in archive", name))?;
    let mut buf = Vec::new();
    entry.read_to_end(&mut buf)?;
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

struct State {
    in_para: bool,
    in_w_t: bool,
    in_del: bool,
    heading_level: Option<usize>,
    current_para: String,
    // table state
    in_table: bool,
    in_cell: bool,
    current_cell: String,
    current_row: Vec<String>,
    table_row_idx: usize,
}

impl State {
    fn new() -> Self {
        Self {
            in_para: false,
            in_w_t: false,
            in_del: false,
            heading_level: None,
            current_para: String::new(),
            in_table: false,
            in_cell: false,
            current_cell: String::new(),
            current_row: Vec::new(),
            table_row_idx: 0,
        }
    }
}

fn style_to_level(val: &str) -> Option<usize> {
    let lower = val.to_lowercase();
    if lower.starts_with("heading") {
        lower
            .trim_start_matches("heading")
            .trim()
            .parse::<usize>()
            .ok()
            .filter(|&n| (1..=6).contains(&n))
    } else if lower == "title" {
        Some(1)
    } else if lower == "subtitle" {
        Some(2)
    } else {
        None
    }
}

fn handle_open(e: &quick_xml::events::BytesStart<'_>, state: &mut State) {
    match e.name().as_ref() {
        b"w:tbl" => {
            state.in_table = true;
            state.table_row_idx = 0;
        }
        b"w:tr" => {
            state.current_row.clear();
        }
        b"w:tc" => {
            state.in_cell = true;
            state.current_cell.clear();
        }
        b"w:p" => {
            state.in_para = true;
            state.current_para.clear();
            state.heading_level = None;
        }
        b"w:t" => state.in_w_t = true,
        b"w:del" => state.in_del = true,
        b"w:pStyle" => {
            for attr in e.attributes().flatten() {
                if attr.key.as_ref() == b"w:val" {
                    let val = std::str::from_utf8(&attr.value).unwrap_or("");
                    if let Some(level) = style_to_level(val) {
                        state.heading_level = Some(level);
                    }
                }
            }
        }
        _ => {}
    }
}

fn parse_document_xml(xml: &str) -> Result<String> {
    let mut reader = Reader::from_str(xml);
    let mut output = String::new();
    let mut state = State::new();

    loop {
        match reader.read_event()? {
            Event::Start(e) => handle_open(&e, &mut state),
            Event::Empty(e) => handle_open(&e, &mut state),
            Event::End(e) => match e.name().as_ref() {
                b"w:tbl" => {
                    state.in_table = false;
                    state.in_cell = false;
                    state.table_row_idx = 0;
                    output.push('\n');
                }
                b"w:tr" => {
                    if !state.current_row.is_empty() {
                        output.push_str(&format!("|{}|\n", state.current_row.join("|")));
                        if state.table_row_idx == 0 {
                            let sep = state
                                .current_row
                                .iter()
                                .map(|_| "---")
                                .collect::<Vec<_>>()
                                .join("|");
                            output.push_str(&format!("|{}|\n", sep));
                        }
                        state.table_row_idx += 1;
                    }
                    state.current_row.clear();
                }
                b"w:tc" => {
                    let cell = state.current_cell.replace('|', "\\|");
                    state.current_row.push(cell.trim().to_string());
                    state.current_cell.clear();
                    state.in_cell = false;
                }
                b"w:p" => {
                    let text = state.current_para.trim().to_string();
                    if state.in_cell {
                        if !text.is_empty() {
                            if !state.current_cell.is_empty() {
                                state.current_cell.push(' ');
                            }
                            state.current_cell.push_str(&text);
                        }
                    } else if !text.is_empty() {
                        match state.heading_level {
                            Some(level) => {
                                let prefix = "#".repeat(level.min(6));
                                output.push_str(&format!("{} {}\n\n", prefix, text));
                            }
                            None => output.push_str(&format!("{}\n\n", text)),
                        }
                    }
                    state.in_para = false;
                    state.in_w_t = false;
                    state.current_para.clear();
                    state.heading_level = None;
                }
                b"w:t" => state.in_w_t = false,
                b"w:del" => state.in_del = false,
                _ => {}
            },
            Event::Text(e) => {
                if state.in_w_t && state.in_para && !state.in_del {
                    if let Ok(text) = e.unescape() {
                        state.current_para.push_str(&text);
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(output)
}
