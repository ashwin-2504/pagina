import React, { useState } from 'react';
import { BookOpen, ArrowLeft, ExternalLink, ChevronRight, Copy, Check } from 'lucide-react';

interface DocsProps {
  onBackToStudio: () => void;
}

export const DocumentationSite: React.FC<DocsProps> = ({ onBackToStudio }) => {
  const [activeSection, setActiveSection] = useState<string>('intro');
  const [copiedCode, setCopiedCode] = useState<string | null>(null);

  const copyToClipboard = (text: string, id: string) => {
    navigator.clipboard.writeText(text);
    setCopiedCode(id);
    setTimeout(() => setCopiedCode(null), 2000);
  };

  const navItems = [
    {
      category: 'Getting Started',
      items: [
        { id: 'intro', label: 'Introduction & Vision' },
        { id: 'quickstart', label: 'Quickstart Guide' },
        { id: 'document-structure', label: 'Document Hierarchy' },
      ],
    },
    {
      category: 'Grammar & Tags',
      items: [
        { id: 'props-block', label: '<props> Configuration' },
        { id: 'paper-geometry', label: '<paper> & Page Sizing' },
        { id: 'page-styles', label: '<page-styles> & Running Headers' },
        { id: 'sections-blocks', label: '<section> & Core Elements' },
        { id: 'css-grid', label: '<grid> Multi-Column Flow' },
        { id: 'callouts-tables', label: '<callout> & <table>' },
      ],
    },
    {
      category: 'Modular Files & CSS',
      items: [
        { id: 'external-css', label: 'External CSS / DCSS & <link>' },
        { id: 'css-properties', label: 'Supported CSS Properties' },
      ],
    },
    {
      category: 'Rust Engine & CLI',
      items: [
        { id: 'engine-ast', label: 'AST & Parsing Core' },
        { id: 'backend-api', label: 'Rust HTTP Backend & CLI' },
      ],
    },
    {
      category: 'Export & Conversion',
      items: [
        { id: 'export-formats', label: 'PDF & OpenXML (DOCX) Export' },
        { id: 'reverse-engineering', label: 'Reverse Conversion (.dxml & .dcss)' },
      ],
    },
  ];

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh', width: '100vw', background: '#090d16', color: '#f8fafc', fontFamily: 'Inter, system-ui, sans-serif' }}>
      {/* Top Navbar */}
      <header style={{
        height: '54px',
        borderBottom: '1px solid #1e293b',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: '0 24px',
        background: '#0f172a',
        flexShrink: 0,
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
          <button
            onClick={onBackToStudio}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              background: '#1e293b',
              color: '#38bdf8',
              border: '1px solid #334155',
              padding: '6px 12px',
              borderRadius: '6px',
              fontSize: '12px',
              fontWeight: 600,
              cursor: 'pointer',
            }}
          >
            <ArrowLeft size={14} /> Back to Studio
          </button>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <BookOpen size={20} color="#38bdf8" />
            <span style={{ fontWeight: 700, fontSize: '16px', color: '#f8fafc' }}>Pagina Documentation</span>
            <span style={{ fontSize: '11px', background: '#1e293b', color: '#94a3b8', padding: '2px 8px', borderRadius: '12px' }}>v0.1 Spec</span>
          </div>
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: '16px', fontSize: '13px' }}>
          <a
            href="https://github.com/ashwin-2504/pagina"
            target="_blank"
            rel="noreferrer"
            style={{ display: 'flex', alignItems: 'center', gap: '6px', color: '#94a3b8', textDecoration: 'none' }}
          >
            GitHub <ExternalLink size={13} />
          </a>
        </div>
      </header>

      {/* Main Documentation Split */}
      <div style={{ display: 'flex', flex: 1, minHeight: 0 }}>
        {/* Sidebar Nav */}
        <aside style={{
          width: '280px',
          borderRight: '1px solid #1e293b',
          background: '#0b1120',
          overflowY: 'auto',
          padding: '24px 16px',
          flexShrink: 0,
        }}>
          {navItems.map((group, gIdx) => (
            <div key={gIdx} style={{ marginBottom: '24px' }}>
              <div style={{
                fontSize: '11px',
                fontWeight: 700,
                color: '#64748b',
                textTransform: 'uppercase',
                letterSpacing: '0.05em',
                marginBottom: '8px',
                paddingLeft: '10px',
              }}>
                {group.category}
              </div>
              <ul style={{ listStyle: 'none', padding: 0, margin: 0 }}>
                {group.items.map(item => {
                  const isActive = activeSection === item.id;
                  return (
                    <li key={item.id}>
                      <button
                        onClick={() => setActiveSection(item.id)}
                        style={{
                          width: '100%',
                          textAlign: 'left',
                          background: isActive ? '#1e293b' : 'transparent',
                          color: isActive ? '#38bdf8' : '#94a3b8',
                          border: 'none',
                          padding: '8px 12px',
                          borderRadius: '6px',
                          fontSize: '13px',
                          fontWeight: isActive ? 600 : 500,
                          cursor: 'pointer',
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'space-between',
                          marginBottom: '2px',
                          transition: 'all 0.15s ease',
                        }}
                      >
                        <span>{item.label}</span>
                        {isActive && <ChevronRight size={14} color="#38bdf8" />}
                      </button>
                    </li>
                  );
                })}
              </ul>
            </div>
          ))}
        </aside>

        {/* Content Pane */}
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
                onCopy={copyToClipboard}
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
                onCopy={copyToClipboard}
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
                onCopy={copyToClipboard}
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
                onCopy={copyToClipboard}
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
                onCopy={copyToClipboard}
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
                onCopy={copyToClipboard}
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
                onCopy={copyToClipboard}
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
      </div>
    </div>
  );
};

const CodeSnippet: React.FC<{ id: string; code: string; copiedId: string | null; onCopy: (text: string, id: string) => void }> = ({ id, code, copiedId, onCopy }) => {
  const isCopied = copiedId === id;
  return (
    <div style={{ position: 'relative', marginTop: '12px', marginBottom: '24px' }}>
      <button
        onClick={() => onCopy(code, id)}
        style={{
          position: 'absolute',
          top: '12px',
          right: '12px',
          background: '#1e293b',
          color: isCopied ? '#4ade80' : '#94a3b8',
          border: '1px solid #334155',
          borderRadius: '4px',
          padding: '4px 8px',
          fontSize: '11px',
          cursor: 'pointer',
          display: 'flex',
          alignItems: 'center',
          gap: '4px',
        }}
      >
        {isCopied ? <Check size={12} /> : <Copy size={12} />}
        {isCopied ? 'Copied' : 'Copy'}
      </button>
      <pre style={{
        background: '#020617',
        border: '1px solid #1e293b',
        borderRadius: '8px',
        padding: '16px 20px',
        overflowX: 'auto',
        fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace',
        fontSize: '13px',
        color: '#e2e8f0',
        lineHeight: 1.6,
      }}>
        <code>{code}</code>
      </pre>
    </div>
  );
};
