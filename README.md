# Pagina

> **Modern, declarative document authoring combining the mental model of HTML & CSS with first-class document typesetting.**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.85+-orange.svg)](https://www.rust-lang.org/)
[![React](https://img.shields.io/badge/React-19-blue.svg)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5-blue.svg)](https://www.typescriptlang.org/)

**Pagina** is an open-source, AI-agent-friendly Document Definition Language (DDL) and live typesetting engine. It replaces rigid LaTeX macros and bloated, binary DOCX files with a clean, semantic XML structure styled with standard CSS.

---

## Features

- **HTML/CSS Mental Model**: Write intuitive tags (`<section>`, `<grid>`, `<callout>`, `<h1>`-`<h6>`, `<table>`, `<p>`) and style them using real CSS.
- **First-Class Document Paging**: True paper coordinate systems (A4, Letter, margins, running headers/footers with dynamic page numbers and watermarks).
- **Multi-Column Flow with CSS Grid**: Compose nested columns cleanly using `<grid cols="1fr 1fr" gap="6mm">` without page layout crashes.
- **Modular Data & Styling**: Decouple document content (`.dxml`) from presentation (`.css`) using `<link rel="stylesheet" href="styles.css" />`.
- **Live Reactive Monaco Studio**: Built with Monaco Editor (the VS Code editing engine) featuring in-editor error squiggles, syntax formatting, grammar suggestions, and dual-tag renaming.
- **Ultra-Fast Rust Layout Engine**: Core AST and parser written in Rust with sub-millisecond compile times.
- **Interactive Documentation**: Built-in developer guide and tag specification portal.

---

## Quickstart

### Prerequisites
- [Node.js](https://nodejs.org) (v18+)
- [Rust & Cargo](https://rustup.rs) (for native engine compilation)

### Running the Live Studio
```bash
# Clone the repository
git clone https://github.com/ashwin-2504/pagina.git
cd pagina/editor

# Install dependencies
npm install

# Start the live development editor
npm run dev
```

Open [http://localhost:5173](http://localhost:5173) in your browser.

### Running the Rust Engine Backend
```bash
cd ddl-engine
cargo run --bin server
```

The native Rust API will be available at `http://localhost:3001`.

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
    <link rel="stylesheet" href="styles.css" />

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

```
pagina/
├── editor/              # Frontend Studio (React + TypeScript + Monaco Editor)
│   ├── src/
│   │   ├── App.tsx             # Studio interface & split workspace
│   │   ├── DocumentationSite.tsx # Connected developer docs portal
│   │   ├── DocumentRenderer.tsx # Virtual A4 canvas & Paged renderer
│   │   ├── formatter.ts        # DOM-tree syntax & CSS beautifier
│   │   ├── ddlLanguage.ts      # Custom Monarch tokenizer with embedded CSS
│   │   └── monacoFeatures.ts   # Autocomplete, dual-tag rename & hover docs
├── ddl-engine/          # High-performance Rust layout core
│   ├── src/
│   │   ├── ast.rs              # Document & CSS AST models
│   │   ├── parser.rs           # Resilient recursive-descent parser
│   │   ├── bin/server.rs       # Axum HTTP backend daemon
│   │   └── bin/ddl-cli.rs      # Native command-line binary
└── LICENSE              # MIT License
```

---

## License

Distributed under the **MIT License**. See `LICENSE` for details.
