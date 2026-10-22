import React from 'react';
import { NAV_ITEMS, type NavGroup, CodeSnippet } from './docsNav';

export { NAV_ITEMS, type NavGroup };

export const DocsContent: React.FC<{
  activeSection: string;
  copiedCode: string | null;
  onCopy: (text: string, id: string) => void;
}> = ({ activeSection, copiedCode, onCopy }) => {
  return (
    <main style={{
      flex: 1,
      overflowY: 'auto',
      padding: '40px 60px',
      background: '#090d16',
      maxWidth: '900px',
      lineHeight: 1.7,
    }}>
      {activeSection === 'intro' && (
        <section>
          <h1 style={{ fontSize: '32px', fontWeight: 800, color: '#f8fafc', marginBottom: '16px' }}>
            Pagina: Document Definition Language (DDL)
          </h1>
          <p style={{ fontSize: '16px', color: '#94a3b8', marginBottom: '24px' }}>
            Pagina is an open-source markup specification and typesetting engine designed as a modern alternative to LaTeX, DOCX, and HTML-to-PDF print engines.
          </p>

          <h2 style={{ fontSize: '20px', fontWeight: 700, color: '#38bdf8', marginTop: '32px', marginBottom: '12px' }}>
            Core Philosophy
          </h2>
          <ul style={{ color: '#cbd5e1', paddingLeft: '20px', marginBottom: '24px' }}>
            <li><b>Strict Document Hierarchy:</b> A clean document tree consisting of properties, metadata, and sections.</li>
            <li><b>Decoupled Content & Style:</b> Content is defined in <code>.dxml</code> files, while presentation rules reside in <code>.dcss</code> or <code>.css</code> stylesheets.</li>
            <li><b>First-Class Virtual Paper:</b> Physical units (mm, cm, pt, in), orientation, margins, running headers, and footers.</li>
            <li><b>Native Rust Core:</b> Lightning-fast AST parsing, validation, PDF generation, and OpenXML export.</li>
          </ul>
        </section>
      )}

      {activeSection === 'quickstart' && (
        <section>
          <h1 style={{ fontSize: '28px', fontWeight: 800, color: '#f8fafc', marginBottom: '16px' }}>Quickstart</h1>
          <p style={{ color: '#94a3b8', marginBottom: '20px' }}>
            Create a document with decoupled content and styling:
          </p>

          <CodeSnippet
            id="quickstart-code"
            copiedId={copiedCode}
            onCopy={onCopy}
            code={`<doc>
  <props>
    <meta><title>Executive Summary</title></meta>
    <paper size="A4" orientation="portrait" margin="20mm" />
    <link rel="stylesheet" href="styles.dcss" />
  </props>
  <body>
    <section>
      <h1>Modular Document Architecture</h1>
      <p>Content is compiled into physical pages natively.</p>
    </section>
  </body>
</doc>`}
          />
        </section>
      )}

      {activeSection === 'export-formats' && (
        <section>
          <h1 style={{ fontSize: '28px', fontWeight: 800, color: '#f8fafc', marginBottom: '16px' }}>PDF & OpenXML (DOCX) Export</h1>
          <p style={{ color: '#94a3b8', marginBottom: '20px' }}>
            Pagina compiles documents natively in Rust directly into PDF and Microsoft Word OpenXML (.docx) formats with zero browser or Node.js dependencies.
          </p>

          <h2 style={{ fontSize: '18px', fontWeight: 700, color: '#38bdf8', marginTop: '24px', marginBottom: '10px' }}>
            CLI Export Command
          </h2>
          <CodeSnippet
            id="cli-export-code"
            copiedId={copiedCode}
            onCopy={onCopy}
            code={`# Export to PDF natively
ddl-cli export --format pdf document.dxml -o output.pdf

# Export to OpenXML (.docx) natively
ddl-cli export --format docx document.dxml -o output.docx`}
          />

          <h2 style={{ fontSize: '18px', fontWeight: 700, color: '#38bdf8', marginTop: '24px', marginBottom: '10px' }}>
            HTTP Engine API
          </h2>
          <CodeSnippet
            id="http-export-code"
            copiedId={copiedCode}
            onCopy={onCopy}
            code={`POST http://127.0.0.1:3001/api/export/pdf
Content-Type: application/json
{ "ddl": "<doc>...</doc>" }

POST http://127.0.0.1:3001/api/export/docx
Content-Type: application/json
{ "ddl": "<doc>...</doc>" }`}
          />
        </section>
      )}

      {activeSection === 'reverse-engineering' && (
        <section>
          <h1 style={{ fontSize: '28px', fontWeight: 800, color: '#f8fafc', marginBottom: '16px' }}>Reverse Conversion to DXML & DCSS</h1>
          <p style={{ color: '#94a3b8', marginBottom: '20px' }}>
            The Pagina engine can ingest existing DOCX and PDF files, analyze their document structure, fonts, and headings, and decompile them into clean <code>.dxml</code> markup and a matching <code>.dcss</code> stylesheet.
          </p>

          <h2 style={{ fontSize: '18px', fontWeight: 700, color: '#38bdf8', marginTop: '24px', marginBottom: '10px' }}>
            CLI Reverse-Engineering Command
          </h2>
          <CodeSnippet
            id="cli-import-code"
            copiedId={copiedCode}
            onCopy={onCopy}
            code={`# Reverse-engineer DOCX into .dxml and .dcss
ddl-cli import proposal.docx -o output_dir/

# Reverse-engineer PDF into .dxml and .dcss
ddl-cli import report.pdf -o output_dir/`}
          />

          <h2 style={{ fontSize: '18px', fontWeight: 700, color: '#38bdf8', marginTop: '24px', marginBottom: '10px' }}>
            HTTP Reverse-Engineering API
          </h2>
          <CodeSnippet
            id="http-import-code"
            copiedId={copiedCode}
            onCopy={onCopy}
            code={`POST http://127.0.0.1:3001/api/import/docx
Content-Type: application/octet-stream
[Raw DOCX binary payload]

POST http://127.0.0.1:3001/api/import/pdf
Content-Type: application/octet-stream
[Raw PDF binary payload]

Response JSON:
{
  "success": true,
  "dxml": "<doc>...</doc>",
  "dcss": "h1 { ... }"
}`}
          />
        </section>
      )}

      {activeSection === 'engine-ast' && (
        <section>
          <h1 style={{ fontSize: '28px', fontWeight: 800, color: '#f8fafc', marginBottom: '16px' }}>Rust Engine & AST Core</h1>
          <p style={{ color: '#94a3b8', marginBottom: '20px' }}>
            The layout engine is engineered in Rust (<code>ddl-engine</code>) using memory-safe, high-performance AST structures.
          </p>

          <CodeSnippet
            id="rust-ast-code"
            copiedId={copiedCode}
            onCopy={onCopy}
            code={`pub struct Document {
    pub props: DocumentProps,
    pub body: BodyNode,
    pub span: Span,
}

pub struct DocumentProps {
    pub metadata: HashMap<String, String>,
    pub paper: PaperConfig,
    pub stylesheet: Stylesheet,
    pub page_styles: HashMap<String, PageStyle>,
}

pub enum Node {
    Section(SectionNode),
    Element(ElementNode),
    Text(TextNode),
    Comment(CommentNode),
}`}
          />
        </section>
      )}

      {activeSection === 'backend-api' && (
        <section>
          <h1 style={{ fontSize: '28px', fontWeight: 800, color: '#f8fafc', marginBottom: '16px' }}>Rust HTTP Backend Daemon</h1>
          <p style={{ color: '#94a3b8', marginBottom: '20px' }}>
            The Axum-based server runs on <code>http://127.0.0.1:3001</code> providing parsing, export, and reverse-conversion endpoints.
          </p>

          <CodeSnippet
            id="server-routes-code"
            copiedId={copiedCode}
            onCopy={onCopy}
            code={`GET  /health            # Service healthcheck
POST /api/parse         # Parses DDL XML into AST JSON
POST /api/export/pdf    # Native PDF binary generation
POST /api/export/docx   # Native OpenXML (.docx) generation
POST /api/import/docx   # Reverse conversion: DOCX -> .dxml + .dcss
POST /api/import/pdf    # Reverse conversion: PDF -> .dxml + .dcss`}
          />
        </section>
      )}
    </main>
  );
};
