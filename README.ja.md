# convert-to-md-rs

[![CI](https://github.com/howlrs/convert-to-md/actions/workflows/ci.yml/badge.svg)](https://github.com/howlrs/convert-to-md/actions)
[![Release](https://img.shields.io/github/v/release/howlrs/convert-to-md)](https://github.com/howlrs/convert-to-md/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

> **[English README](README.md)**

Rust 製の高速並列ファイル→Markdown 変換ツール。  
Docker 経由で実行するため、ホストに Python や追加ランタイム不要。

## 対応フォーマット

| カテゴリ | 拡張子 |
|---------|--------|
| ドキュメント | `.pdf`, `.docx`, `.doc`, `.pptx`, `.ppt` |
| 表計算 | `.xlsx`, `.xls` |
| Web / テキスト | `.html`, `.htm`, `.csv`, `.json`, `.xml`, `.txt` |
| 画像 | `.jpg`, `.jpeg`, `.png`, `.gif`, `.bmp`, `.tiff`, `.webp` |

## ダウンロード

ビルド済みバイナリは [Releases ページ](https://github.com/howlrs/convert-to-md/releases/latest) からダウンロードできます。

| プラットフォーム | ファイル |
|----------------|----------|
| Linux x86_64 (静的リンク) | `convert-to-md-rs-*-linux-x86_64.tar.gz` |
| Windows x86_64 | `convert-to-md-rs-*-windows-x86_64.zip` |

```bash
# Linux: 展開して実行
tar xzf convert-to-md-rs-*-linux-x86_64.tar.gz
./convert-to-md-rs --input resources/ --output data/output/markdown/

# Windows (PowerShell): 展開して実行
Expand-Archive convert-to-md-rs-*-windows-x86_64.zip .
.\convert-to-md-rs.exe --input resources --output data\output\markdown
```

## クイックスタート

### Docker（推奨 — Windows / Linux 共通）

```bash
# 1. resources/ にファイルを置く
cp my-report.pdf resources/

# 2. 実行（初回は自動でイメージをビルド）
./scripts/convert.sh

# 変換結果 → data/output/markdown/
```

### ローカルビルド（Rust 1.75+ 必要）

```bash
cargo build --release
./target/release/convert-to-md-rs --input resources/ --output data/output/markdown/
```

## オプション

```
convert-to-md-rs [OPTIONS]

OPTIONS:
  -i, --input <PATH>   入力ファイルまたはディレクトリ  [default: resources]
  -o, --output <PATH>  出力ディレクトリ               [default: data/output/markdown]
      --overwrite      既存ファイルを上書き
      --list           変換せずにファイル一覧を表示
  -j, --jobs <N>       並列ジョブ数 (0 = 全コア使用)  [default: 0]
  -h, --help           ヘルプを表示
  -V, --version        バージョンを表示
```

## Docker ラッパー（`scripts/convert.sh`）

| コマンド | 説明 |
|---------|------|
| `./scripts/convert.sh` | `resources/` 内を一括変換 |
| `./scripts/convert.sh --list` | 変換対象ファイルの確認 |
| `./scripts/convert.sh --overwrite` | 既存ファイルを上書きして変換 |
| `./scripts/convert.sh --build` | Docker イメージを強制再ビルド |

## ディレクトリ構成

```
resources/                   ← ここにファイルを置く
  ├── report.pdf
  ├── slides.pptx
  └── data/
      └── budget.xlsx

data/output/markdown/        ← 変換された Markdown が出力される
  ├── report.md
  ├── slides.md
  └── data/
      └── budget.md
```

## 内部構成

Rust バイナリが拡張子からフォーマットを検出し、各コンバーターに振り分けます。

| フォーマット | コンバーター |
|------------|------------|
| PDF | `pdftotext` サブプロセス（Docker イメージに poppler 同梱） |
| DOCX | ZIP + XML パース（`quick-xml`） |
| PPTX | ZIP + XML パース（`quick-xml`） |
| XLSX/XLS | `calamine` クレート |
| HTML | `htmd` クレート |
| CSV | `csv` クレート → Markdown テーブル |
| JSON | `serde_json` → コードブロック |
| XML | `quick-xml` テキスト抽出 |
| 画像 | `kamadak-exif` による EXIF メタデータ出力 |

## 活用シーン

- **朝のブリーフィング**: 会議資料（pptx/pdf）を resources/ に置いて変換 → LLM で要約
- **スキルシートのバージョン管理**: docx → Markdown に変換して Git 差分管理
- **RAG 入力素材の準備**: ドキュメントを一括 Markdown 化してベクター DB に登録
- **Excel → Markdown テーブル**: 月次レポートの自動処理パイプライン

## 開発

```bash
cargo test
cargo clippy
cargo build --release
```

## ライセンス

MIT — [LICENSE](LICENSE) を参照
