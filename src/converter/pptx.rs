use anyhow::Result;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

pub fn convert(path: &Path) -> Result<String> {
    let file = std::fs::File::open(path)?;
    let mut archive = ZipArchive::new(file)?;

    // Collect slide file names (ppt/slides/slideN.xml) sorted by N
    let mut slides: Vec<(u32, String)> = archive
        .file_names()
        .filter(|n| n.starts_with("ppt/slides/slide") && n.ends_with(".xml"))
        .filter_map(|n| {
            let num_str = n
                .trim_start_matches("ppt/slides/slide")
                .trim_end_matches(".xml");
            num_str.parse::<u32>().ok().map(|num| (num, n.to_string()))
        })
        .collect();
    slides.sort_by_key(|(n, _)| *n);

    if slides.is_empty() {
        anyhow::bail!("No slides found in PPTX");
    }

    let title = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    let mut output = format!("# {}\n\n", title);

    for (i, (_, slide_name)) in slides.iter().enumerate() {
        let xml_content = {
            let mut entry = archive
                .by_name(slide_name)
                .map_err(|_| anyhow::anyhow!("cannot open {}", slide_name))?;
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf)?;
            String::from_utf8_lossy(&buf).into_owned()
        };

        let slide_text = extract_slide_text(&xml_content)?;
        if !slide_text.trim().is_empty() {
            output.push_str(&format!("## Slide {}\n\n", i + 1));
            output.push_str(&slide_text);
            output.push('\n');
        }
    }

    Ok(output)
}

struct SlideState {
    in_a_p: bool,
    in_a_t: bool,
    current_para: String,
    is_title_shape: bool,
}

impl SlideState {
    fn new() -> Self {
        Self {
            in_a_p: false,
            in_a_t: false,
            current_para: String::new(),
            is_title_shape: false,
        }
    }
}

fn handle_open(e: &quick_xml::events::BytesStart<'_>, state: &mut SlideState) {
    match e.name().as_ref() {
        b"p:sp" => state.is_title_shape = false,
        b"p:ph" => {
            for attr in e.attributes().flatten() {
                if attr.key.as_ref() == b"type" {
                    let val = std::str::from_utf8(&attr.value).unwrap_or("");
                    if val == "title" || val == "ctrTitle" {
                        state.is_title_shape = true;
                    }
                }
            }
        }
        b"a:p" => {
            state.in_a_p = true;
            state.current_para.clear();
        }
        b"a:t" => state.in_a_t = true,
        _ => {}
    }
}

fn extract_slide_text(xml: &str) -> Result<String> {
    let mut reader = Reader::from_str(xml);
    let mut output = String::new();
    let mut state = SlideState::new();

    loop {
        match reader.read_event()? {
            Event::Start(e) => handle_open(&e, &mut state),
            Event::Empty(e) => handle_open(&e, &mut state),
            Event::End(e) => match e.name().as_ref() {
                b"a:p" => {
                    let text = state.current_para.trim().to_string();
                    if !text.is_empty() {
                        if state.is_title_shape {
                            output.push_str(&format!("### {}\n\n", text));
                        } else {
                            output.push_str(&format!("{}\n\n", text));
                        }
                    }
                    state.in_a_p = false;
                    state.in_a_t = false;
                    state.current_para.clear();
                }
                b"a:t" => state.in_a_t = false,
                _ => {}
            },
            Event::Text(e) => {
                if state.in_a_t && state.in_a_p {
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
