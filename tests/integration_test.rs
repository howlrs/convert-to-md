use std::io::Write;
use std::path::Path;

use convert_to_md_rs::converter::convert_file;

// ── helpers ──────────────────────────────────────────────────────────────────

/// Build a minimal DOCX (ZIP with word/document.xml).
fn write_minimal_docx(path: &Path, paragraph: &str, table_rows: Option<&[&[&str]]>) {
    use zip::write::FileOptions;
    use zip::ZipWriter;

    let mut body = String::new();
    body.push_str(&format!(
        "<w:p><w:r><w:t>{}</w:t></w:r></w:p>",
        xml_escape(paragraph)
    ));

    if let Some(rows) = table_rows {
        body.push_str("<w:tbl>");
        for row in rows {
            body.push_str("<w:tr>");
            for cell in *row {
                body.push_str(&format!(
                    "<w:tc><w:p><w:r><w:t>{}</w:t></w:r></w:p></w:tc>",
                    xml_escape(cell)
                ));
            }
            body.push_str("</w:tr>");
        }
        body.push_str("</w:tbl>");
    }

    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{}</w:body></w:document>"#,
        body
    );

    let file = std::fs::File::create(path).unwrap();
    let mut zip = ZipWriter::new(file);
    let opts = FileOptions::default();
    zip.start_file("word/document.xml", opts).unwrap();
    zip.write_all(xml.as_bytes()).unwrap();
    zip.finish().unwrap();
}

/// Build a minimal PPTX (ZIP with ppt/slides/slide1.xml).
fn write_minimal_pptx(path: &Path, title: &str, body_text: &str, table_rows: Option<&[&[&str]]>) {
    use zip::write::FileOptions;
    use zip::ZipWriter;

    let title_shape = format!(
        r#"<p:sp><p:nvSpPr><p:nvPr><p:ph type="title"/></p:nvPr></p:nvSpPr><p:spPr/><p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:t>{}</a:t></a:p></p:txBody></p:sp>"#,
        xml_escape(title)
    );
    let body_shape = format!(
        r#"<p:sp><p:spPr/><p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:t>{}</a:t></a:p></p:txBody></p:sp>"#,
        xml_escape(body_text)
    );

    let mut tbl_xml = String::new();
    if let Some(rows) = table_rows {
        tbl_xml.push_str(r#"<p:graphicFrame><a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/table"><a:tbl>"#);
        for row in rows {
            tbl_xml.push_str("<a:tr>");
            for cell in *row {
                tbl_xml.push_str(&format!(
                    "<a:tc><a:txBody><a:bodyPr/><a:lstStyle/><a:p><a:t>{}</a:t></a:p></a:txBody></a:tc>",
                    xml_escape(cell)
                ));
            }
            tbl_xml.push_str("</a:tr>");
        }
        tbl_xml.push_str("</a:tbl></a:graphicData></a:graphic></p:graphicFrame>");
    }

    let slide_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><p:cSld><p:spTree>{}{}{}</p:spTree></p:cSld></p:sld>"#,
        title_shape, body_shape, tbl_xml
    );

    let file = std::fs::File::create(path).unwrap();
    let mut zip = ZipWriter::new(file);
    let opts = FileOptions::default();
    zip.start_file("ppt/slides/slide1.xml", opts).unwrap();
    zip.write_all(slide_xml.as_bytes()).unwrap();
    zip.finish().unwrap();
}

/// Build a minimal XLSX with numeric data (no shared strings needed).
fn write_minimal_xlsx(path: &Path) {
    use zip::write::FileOptions;
    use zip::ZipWriter;

    let file = std::fs::File::create(path).unwrap();
    let mut z = ZipWriter::new(file);
    let opts = FileOptions::default();

    z.start_file("[Content_Types].xml", opts).unwrap();
    z.write_all(br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/><Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/></Types>"#).unwrap();

    z.start_file("_rels/.rels", opts).unwrap();
    z.write_all(br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>"#).unwrap();

    z.start_file("xl/workbook.xml", opts).unwrap();
    z.write_all(br#"<?xml version="1.0"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="Sheet1" sheetId="1" r:id="rId1"/></sheets></workbook>"#).unwrap();

    z.start_file("xl/_rels/workbook.xml.rels", opts).unwrap();
    z.write_all(br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/></Relationships>"#).unwrap();

    z.start_file("xl/worksheets/sheet1.xml", opts).unwrap();
    z.write_all(br#"<?xml version="1.0"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData><row r="1"><c r="A1"><v>10</v></c><c r="B1"><v>20</v></c><c r="C1"><v>30</v></c></row><row r="2"><c r="A2"><v>1</v></c><c r="B2"><v>2</v></c><c r="C2"><v>3</v></c></row></sheetData></worksheet>"#).unwrap();

    z.finish().unwrap();
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[test]
fn test_csv_conversion() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample.csv");
    let md = convert_file(&fixture, false).expect("CSV conversion failed");
    assert!(md.contains('|'), "expected Markdown table in CSV output");
    assert!(md.contains("Alice"), "expected row data in CSV output");
    assert!(md.contains("Score"), "expected header in CSV output");
}

#[test]
fn test_html_conversion() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample.html");
    let md = convert_file(&fixture, false).expect("HTML conversion failed");
    assert!(md.contains("Hello World"), "expected heading in HTML output");
}

#[test]
fn test_docx_plain_paragraph() {
    let dir = tempdir();
    let path = dir.join("test.docx");
    write_minimal_docx(&path, "Hello from DOCX", None);

    let md = convert_file(&path, false).expect("DOCX conversion failed");
    assert!(md.contains("Hello from DOCX"), "expected text in DOCX output");
}

#[test]
fn test_docx_table() {
    let dir = tempdir();
    let path = dir.join("table.docx");
    write_minimal_docx(
        &path,
        "Intro",
        Some(&[&["Name", "Age"], &["Alice", "30"], &["Bob", "25"]]),
    );

    let md = convert_file(&path, false).expect("DOCX table conversion failed");
    assert!(md.contains('|'), "expected Markdown table in DOCX output");
    assert!(md.contains("Name"), "expected table header");
    assert!(md.contains("---"), "expected table separator");
    assert!(md.contains("Alice"), "expected table data row");
}

#[test]
fn test_pptx_conversion() {
    let dir = tempdir();
    let path = dir.join("test.pptx");
    write_minimal_pptx(&path, "My Slide Title", "Body content here", None);

    let md = convert_file(&path, false).expect("PPTX conversion failed");
    assert!(md.contains("Slide 1"), "expected slide header in PPTX output");
    assert!(
        md.contains("My Slide Title"),
        "expected title in PPTX output"
    );
}

#[test]
fn test_pptx_table() {
    let dir = tempdir();
    let path = dir.join("table.pptx");
    write_minimal_pptx(
        &path,
        "Table Slide",
        "",
        Some(&[&["Col1", "Col2"], &["A", "B"]]),
    );

    let md = convert_file(&path, false).expect("PPTX table conversion failed");
    assert!(md.contains('|'), "expected Markdown table in PPTX output");
    assert!(md.contains("Col1"), "expected table header");
    assert!(md.contains("---"), "expected table separator");
}

#[test]
fn test_xlsx_conversion() {
    let dir = tempdir();
    let path = dir.join("test.xlsx");
    write_minimal_xlsx(&path);

    let md = convert_file(&path, false).expect("XLSX conversion failed");
    assert!(md.contains("Sheet1"), "expected sheet name in XLSX output");
    assert!(md.contains('|'), "expected Markdown table in XLSX output");
}

// ── ZIP integration ───────────────────────────────────────────────────────────

#[test]
fn test_zip_conversion() {
    use zip::write::FileOptions;
    use zip::ZipWriter;

    let dir = tempdir();
    let zip_path = dir.join("archive.zip");

    // Build a ZIP containing a CSV
    let csv_content = b"x,y\n1,2\n3,4\n";
    {
        let file = std::fs::File::create(&zip_path).unwrap();
        let mut z = ZipWriter::new(file);
        z.start_file("data.csv", FileOptions::default()).unwrap();
        z.write_all(csv_content).unwrap();
        z.finish().unwrap();
    }

    let md = convert_file(&zip_path, false).expect("ZIP conversion failed");
    assert!(md.contains("data.csv"), "expected entry name in ZIP output");
    assert!(md.contains('|'), "expected Markdown table from inner CSV");
}

// ── helper ────────────────────────────────────────────────────────────────────

fn tempdir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "convert-to-md-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
