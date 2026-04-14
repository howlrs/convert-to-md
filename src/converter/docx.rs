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
}

impl State {
    fn new() -> Self {
        Self {
            in_para: false,
            in_w_t: false,
            in_del: false,
            heading_level: None,
            current_para: String::new(),
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
                b"w:p" => {
                    let text = state.current_para.trim().to_string();
                    if !text.is_empty() {
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
