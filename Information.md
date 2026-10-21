# Project Specification: Document Definition Language (DDL) & Live Engine

## 1. Executive Summary & Vision

Modern document authoring is fractured:
- **LaTeX**: Extremely powerful typography and page management, but cursed with a 1980s macro language, cryptic error messages, and zero reactive feedback. Non-beginner friendly.
- **Markdown**: Easy to read and write, but completely inadequate for real-world publication layouts (no multi-column splits, page numbering, running headers/footers, or fine margin control).
- **DOCX / OpenXML**: The commercial standard, but binary/bloated XML ZIP archives make it nearly impossible for humans or **AI agents** to edit directly, reliably, and in real time without corruption.

### The Objective
Create an **agent-friendly, human-readable, custom XML-based Document Definition Language** paired with a CSS-like styling paradigm and an ultra-fast **Rust compilation and layout engine**.
The editor provides a live, split-screen interactive environment using **CodeMirror 6** on one side and a native, accurately paginated document renderer on the other.

---

## 2. Core Architectural Pillars

```
+-----------------------------------------------------------+
|                      User / AI Agent                      |
|           (Edits custom XML + stylesheet rules)           |
+-----------------------------+-----------------------------+
                              |
                              v
                [ CodeMirror 6 Editor Pane ]
                              |
                              | (Live AST / Delta Stream)
                              v
+-----------------------------------------------------------+
|                    Rust Core Engine                       |
|  +-----------------------------------------------------+  |
|  | 1. XML Lexer & Parser (Strict & Resilient Parsing)  |  |
|  +-----------------------------------------------------+  |
|  | 2. Style Resolver (CSS-like cascading & classes)    |  |
|  +-----------------------------------------------------+  |
|  | 3. Layout & Paging Engine (Box Model, Breaks, Flow) |  |
|  +-----------------------------------------------------+  |
|  | 4. Export Pipelines (PDF, HTML, DOCX, Canvas IR)    |  |
|  +-----------------------------------------------------+  |
+-----------------------------+-----------------------------+
                              |
                              v
           [ Interactive Live Document Canvas Pane ]
             - True Page Boundaries (A4, Letter, etc.)
             - Running Headers / Footers
             - Bi-directional Selection / Jump-to-Tag
```

### Key Differences from Conventional Tools
1. **Not a Web-to-PDF Hack**: We do not rely on browser print polyfills (`Paged.js` / HTML print emulation). The Rust engine treats pages as first-class geometric coordinate systems.
2. **AI-Agent Native**: AI agents excel at hierarchical, structured syntax like XML and declarative selector rules like CSS. Unlike opaque DOCX XML structures (e.g. `w:r`, `w:rPr`, `w:t`), our syntax is clean, clean, semantic, and human/machine-readable.
3. **Sub-millisecond Feedback**: Built with Rust (Wasm-ready for client or native desktop binary via Tauri/native GUI), enabling real-time re-layout on every keystroke.

---

## 3. Syntax Specification (Ideation)

### A. The Document Structure (`.ddl` / `.dxml`)
Clean, semantic tags that define document hierarchy:

```xml
<doc size="A4" orientation="portrait" margin="25mm">
  <defs>
    <!-- Stylesheet: CSS-like syntax adapted for document primitives -->
    <style>
      doc {
        font-family: "Inter", sans-serif;
        font-size: 11pt;
        line-height: 1.5;
        color: #111827;
      }

      header.primary {
        font-size: 24pt;
        font-weight: 700;
        margin-bottom: 8mm;
        color: #1e3a8a;
      }

      .callout {
        background-color: #f0fdf4;
        border-left: 3pt solid #16a34a;
        padding: 4mm 6mm;
        margin: 4mm 0;
        border-radius: 2pt;
      }

      .meta-grid {
        display: columns;
        columns: 2;
        gap: 10mm;
      }

      #abstract {
        font-style: italic;
        margin-bottom: 6mm;
      }
    </style>
  </defs>

  <!-- Page Template Rules -->
  <page-setup>
    <running-header>
      <left>Confidential Internal Report</left>
      <right><page-number/> / <total-pages/></right>
    </running-header>
    <running-footer>
      <center>© 2026 Acme Corp. All rights reserved.</center>
    </running-footer>
  </page-setup>

  <!-- Content Body -->
  <body class="report-body">
    <h1 class="primary">Autonomous Systems Architecture</h1>
    
    <div id="abstract">
      A comprehensive review of real-time document engines designed for agentic editing workflows.
    </div>

    <div class="callout">
      <b>Status:</b> Ready for prototype review. All specifications match engine target v0.1.
    </div>

    <section title="System Overview">
      <p>
        Documents need not be trapped inside proprietary formats like binary DOCX or ancient macro languages like LaTeX.
      </p>

      <div class="meta-grid">
        <div>
          <h3>Editor Interface</h3>
          <p>Powered by CodeMirror 6 with custom grammar highlighting, linting, and AST projection.</p>
        </div>
        <div>
          <h3>Rust Layout Core</h3>
          <p>Calculates glyph metrics, line breaking (Knuth-Plass), and paginates box structures natively.</p>
        </div>
      </div>
    </section>
  </body>
</doc>
```

---

## 4. The Rust Engine Modules

1. **Parser Module (`ddl_parser`)**:
   - Parses the XML-like structure into a Concrete Syntax Tree (CST) and Document Object Tree (DOM).
   - Tolerant parsing: Does not crash on incomplete tags while the user or AI agent is typing.

2. **Style Engine (`ddl_style`)**:
   - Parses CSS declarations and handles cascading, inheritance, class matching (`.name`), ID matching (`#id`), and tag matching.
   - Computes resolved styles (`ComputedStyle`) for every node.

3. **Layout & Typesetting Engine (`ddl_layout`)**:
   - **Text Shaping**: Font loading, OpenType feature processing, HarfBuzz / cosmic-text or rustybuzz integration.
   - **Line Breaking**: Paragraph layout with hyphenation support.
   - **Page Breaking**: Automatically detects overflow, calculates split points, handles `page-break-before`, `page-break-inside: avoid`, and orphans/widows.
   - Generates an Intermediate Representation (IR) consisting of Pages $\rightarrow$ Bounding Boxes $\rightarrow$ Glyphs/Shapes.

4. **Render & Export Targets (`ddl_export`)**:
   - **Interactive Live View**: Emits low-overhead canvas/SVG commands or WebGPU drawing instructions straight to the preview panel.
   - **PDF**: Direct PDF generation (via `printpdf` or `pdf-writer`) with embedded fonts, vector graphics, and metadata.
   - **DOCX / HTML**: Translates the clean semantic AST into OpenXML or HTML semantics.

---

## 5. Why AI Agents Benefit Immensely

| Format | AI Readability | AI Edit Precision | Real-Time Sync | Page Fidelity |
| :--- | :--- | :--- | :--- | :--- |
| **DOCX** | Terrible (complex XML schema in zip) | Low (prone to corruption) | Impossible live | Medium |
| **LaTeX** | Moderate (confusing macros) | Low-Medium (silent compile fails) | Slow (seconds) | High |
| **Markdown**| High (simple plain text) | High | Fast | Terrible (no true pages) |
| **Our DDL** | **High (clean, strict XML + CSS)** | **High (predictable tag/class edits)** | **Instant (Rust engine)** | **High (first-class pages)** |

---

## 6. Next Steps & Roadmap

- [ ] **Phase 1: Grammar & AST**: Finalize tag vocabulary and CSS property subset in Rust.
- [ ] **Phase 2: Minimal Prototype**:
  - Connect CodeMirror 6 in the editor UI.
  - Compile the Rust parser & layout engine (via Wasm or native IPC).
  - Render simple multi-page layout with headers/footers in real time.
- [ ] **Phase 3: Export Pipelines**: PDF & HTML generators.
