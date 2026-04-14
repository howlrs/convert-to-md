# convert-to-md-rs

[![CI](https://github.com/howlrs/convert-to-md/actions/workflows/ci.yml/badge.svg)](https://github.com/howlrs/convert-to-md/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

> **[日本語版 README](README.ja.md)**

A fast, parallel file-to-Markdown converter written in Rust.  
Runs via Docker — no Python or runtime dependencies on the host.

## Supported Formats

| Category | Extensions |
|----------|-----------|
| Documents | `.pdf`, `.docx`, `.doc`, `.pptx`, `.ppt` |
| Spreadsheets | `.xlsx`, `.xls` |
| Web / Text | `.html`, `.htm`, `.csv`, `.json`, `.xml`, `.txt` |
| Images | `.jpg`, `.jpeg`, `.png`, `.gif`, `.bmp`, `.tiff`, `.webp` |

## Quick Start

### Docker (recommended — Windows & Linux)

```bash
# 1. Place files in resources/
#    (Linux/WSL)
cp my-report.pdf resources/

# 2. Run — auto-builds image on first run
./scripts/convert.sh

# Output → data/output/markdown/
```

### Build & run locally (requires Rust 1.75+)

```bash
cargo build --release
./target/release/convert-to-md-rs --input resources/ --output data/output/markdown/
```

## Options

```
convert-to-md-rs [OPTIONS]

OPTIONS:
  -i, --input <PATH>   Input file or directory [default: resources]
  -o, --output <PATH>  Output directory         [default: data/output/markdown]
      --overwrite      Overwrite existing files
      --list           List files without converting
  -j, --jobs <N>       Parallel jobs (0 = all cores) [default: 0]
  -h, --help           Print help
  -V, --version        Print version
```

## Docker wrapper (`scripts/convert.sh`)

| Command | Description |
|---------|-------------|
| `./scripts/convert.sh` | Convert all files in `resources/` |
| `./scripts/convert.sh --list` | Preview files to be converted |
| `./scripts/convert.sh --overwrite` | Overwrite existing output |
| `./scripts/convert.sh --build` | Force rebuild Docker image |

## Architecture

```
resources/                   ← drop files here
  ├── report.pdf
  ├── slides.pptx
  └── data/
      └── budget.xlsx

data/output/markdown/        ← converted Markdown output
  ├── report.md
  ├── slides.md
  └── data/
      └── budget.md
```

The Rust binary detects file format by extension and dispatches to a native converter:

| Format | Converter |
|--------|-----------|
| PDF | `pdftotext` subprocess (poppler, in Docker image) |
| DOCX | ZIP + XML parsing (`quick-xml`) |
| PPTX | ZIP + XML parsing (`quick-xml`) |
| XLSX/XLS | `calamine` crate |
| HTML | `htmd` crate |
| CSV | `csv` crate → Markdown table |
| JSON | `serde_json` → fenced code block |
| XML | `quick-xml` text extraction |
| Images | EXIF metadata via `kamadak-exif` |

## Use Cases

- **Morning briefing**: Convert meeting slides (pptx/pdf) placed in `resources/` → pipe to LLM summariser
- **Skill-sheet version control**: Convert docx → Markdown for Git diff tracking
- **RAG input preparation**: Batch-convert documents to Markdown for vector DB ingestion
- **Excel → Markdown tables**: Automate monthly report processing

## Development

```bash
cargo test
cargo clippy
cargo build --release
```

## License

MIT — see [LICENSE](LICENSE)
