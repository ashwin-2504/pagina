# Pagina

> **Modern, declarative document authoring combining the mental model of HTML & CSS with first-class document typesetting.**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.90+-orange.svg)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-2-24C8DB.svg)](https://v2.tauri.app/)
[![React](https://img.shields.io/badge/React-19-blue.svg)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5-blue.svg)](https://www.typescriptlang.org/)

**Pagina** is an open-source, AI-agent-friendly Document Definition Language (DDL) and live typesetting engine available as both a native desktop application (powered by Tauri 2 and Rust) and a lightweight web studio. It replaces rigid LaTeX macros and bloated, binary DOCX files with a clean, semantic XML structure (`.dxml`) styled with standard CSS (`.dcss`).

---

## Features

- **Native Desktop Application**: Powered by Tauri 2, embedding the high-performance Rust layout engine directly in-process with zero network overhead.
- **HTML/CSS Mental Model**: Write intuitive tags (`<section>`, `<grid>`, `<callout>`, `<h1>`-`<h6>`, `<table>`, `<p>`) and style them using real CSS.
- **First-Class Document Paging**: True paper coordinate systems (A4, Letter, margins, running headers/footers with dynamic page numbers and watermarks).
- **Multi-Column Flow with CSS Grid**: Compose nested columns cleanly using `<grid cols="1fr 1fr" gap="6mm">` without page layout crashes.
- **Modular Data & Styling**: Decouple document content (`.dxml`) from presentation (`.dcss`) using `<link rel="stylesheet" href="styles.dcss" />`.
- **Native PDF & OpenXML (DOCX) Export**: High-performance binary document generation implemented directly in Rust with zero browser or Node dependencies.
- **Full Reverse-Engineering Engine**: Ingest existing DOCX and PDF documents and decompile them into clean `.dxml` and matching `.dcss` stylesheets.
- **Live Reactive Monaco Studio**: Built with Monaco Editor (the VS Code editing engine) featuring in-editor error squiggles, syntax formatting, grammar suggestions, and dual-tag renaming.
- **Ultra-Fast Rust Layout Engine**: Core AST, parser, and exporter written in Rust with sub-millisecond compile times.
- **Dual-Mode Operation**: Direct Tauri IPC when running natively on the desktop, with seamless fallback to an Axum HTTP daemon when running in web preview mode.

---

## Quickstart

### Prerequisites
- [Node.js](https://nodejs.org) (v18+)
- [Rust & Cargo](https://rustup.rs) (v1.90+)

### Running the Native Desktop Application (Tauri)
```bash
# Clone the repository
git clone https://github.com/ashwin-2504/pagina.git
cd pagina/editor

# Install dependencies
npm install

# Run native desktop application with live reload
npm run tauri:dev
```

To create an optimized standalone desktop executable installer for your operating system:
```bash
npm run tauri:build
```

### Running the Web Studio Preview (Optional)
```bash
# Terminal 1: Start the Rust backend engine daemon
cd pagina/ddl-engine
cargo run --bin server

# Terminal 2: Start the frontend Vite web server
cd pagina/editor
npm run dev
```

Open [http://localhost:5173](http://localhost:5173) in your browser.

### Using the Rust CLI
```bash
cd pagina/ddl-engine

# Parse a document to JSON AST
cargo run --bin ddl-cli -- parse document.dxml

# Export to PDF natively
cargo run --bin ddl-cli -- export --format pdf document.dxml -o output.pdf

# Export to OpenXML (.docx) natively
cargo run --bin ddl-cli -- export --format docx document.dxml -o output.docx

# Reverse-engineer DOCX or PDF into .dxml and .dcss
cargo run --bin ddl-cli -- import document.docx -o output_directory/
cargo run --bin ddl-cli -- import document.pdf -o output_directory/
```

---

## Minimal Example

```xml
<doc>
  <props>
    <meta>
      <title>Autonomous Systems Report</title>
      <author>Engineering Team</author>
    </meta>

    <!-- Global Paper Defaults -->
    <paper size="A4" orientation="portrait" margin="20mm" />

    <!-- External Stylesheet -->
    <link rel="stylesheet" href="styles.dcss" />

    <!-- Page Templates -->
    <page-styles>
      <page-style name="body" default="true">
        <running-header>
          <left><text>Technical Report</text></left>
          <right><page-number/> of <total-pages/></right>
        </running-header>
        <running-footer>
          <center><text>© 2026 Acme Corp</text></center>
        </running-footer>
      </page-style>
    </page-styles>
  </props>

  <body>
    <section page-style="body">
      <h1>Document Architecture</h1>
      <p>Clean typography styled with standard CSS.</p>

      <grid cols="1fr 1fr" gap="6mm">
        <div>
          <h3>Left Column</h3>
          <p>Flows cleanly inside its cell.</p>
        </div>
        <div>
          <h3>Right Column</h3>
          <p>No fragile column section breaks needed.</p>
        </div>
      </grid>
    </section>
  </body>
</doc>
```

---

## Project Architecture

The architecture maintains strict separation between the native typesetting engine, desktop application shell, and frontend studio:

```
pagina/
├── editor/
│   ├── src-tauri/               # Tauri 2 Desktop Application Host (Rust)
│   │   ├── Cargo.toml           # Links ddl-engine directly in-process
│   │   ├── tauri.conf.json      # Window, permission & bundle configuration
│   │   └── src/
│   │       ├── lib.rs           # Native Tauri commands (export_pdf, export_docx, import)
│   │       └── main.rs          # Application entrypoint
│   └── src/                     # Studio Frontend (React + TypeScript + Monaco Editor)
│       ├── App.tsx              # Studio interface, export/import actions, split workspace
│       ├── engineBridge.ts      # Dual-mode bridge (Tauri IPC vs Axum HTTP fallback)
│       ├── DocumentRenderer.tsx # Virtual A4 canvas & paged renderer
│       ├── DocumentationSite.tsx # Connected developer docs portal
│       ├── formatter.ts         # DOM-tree syntax & CSS beautifier
│       ├── ddlLanguage.ts       # Custom Monarch tokenizer with embedded CSS
│       └── monacoFeatures.ts    # Autocomplete, dual-tag rename & hover docs
├── ddl-engine/                  # High-performance Rust layout & conversion core
│   ├── src/
│   │   ├── ast.rs               # Document & CSS AST models
│   │   ├── parser.rs            # Resilient recursive-descent parser
│   │   ├── docx/
│   │   │   ├── export.rs        # OpenXML (.docx) zip/xml generator
│   │   │   └── import.rs        # OpenXML reverse decompiler into .dxml + .dcss
│   │   ├── pdf/
│   │   │   ├── export.rs        # Native PDF typesetter & vector generator
│   │   │   └── import.rs        # PDF text & structure extraction into .dxml + .dcss
│   │   ├── bin/server.rs        # Axum HTTP backend daemon (port 3001)
│   │   └── bin/ddl-cli.rs       # Native command-line binary
└── LICENSE                      # MIT License
```

---

## License

Distributed under the **MIT License**. See `LICENSE` for details.
