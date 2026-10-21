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
        { id: 'external-css', label: 'External CSS & <link>' },
        { id: 'css-properties', label: 'Supported CSS Properties' },
      ],
    },
    {
      category: 'Rust Engine Architecture',
      items: [
        { id: 'engine-ast', label: 'AST & Parsing Core' },
        { id: 'paged-layout', label: 'First-Class Paging Engine' },
        { id: 'backend-api', label: 'Rust HTTP Backend & CLI' },
      ],
    },
  ];

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh', width: '100vw', background: '#090d16', color: '#f8fafc', fontFamily: 'Inter, system-ui, sans-serif' }}>
      {/* Top Navbar */}
      <header style={{
        height: '54px',
        borderBottom: '1px solid #1e293b',
        background: '#0b1120',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: '0 24px',
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
            <span style={{ fontWeight: 700, fontSize: '16px', color: '#f8fafc' }}>DDL Documentation</span>
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
                          fontWeight: isActive ? 600 : 400,
                          cursor: 'pointer',
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'space-between',
                          marginBottom: '2px',
                        }}
                      >
                        <span>{item.label}</span>
                        {isActive && <ChevronRight size={14} />}
                      </button>
                    </li>
                  );
                })}
              </ul>
            </div>
          ))}
        </aside>

        {/* Documentation Content View */}
        <main style={{
          flex: 1,
          overflowY: 'auto',
          padding: '40px 64px',
          maxWidth: '960px',
          lineHeight: 1.7,
        }}>
          {activeSection === 'intro' && (
            <section>
              <h1 style={{ fontSize: '32px', fontWeight: 800, color: '#f8fafc', marginBottom: '16px' }}>
                Introduction to Document Definition Language (DDL)
              </h1>
              <p style={{ fontSize: '16px', color: '#94a3b8', marginBottom: '24px' }}>
                DDL is an agent-friendly, human-readable document authoring specification designed to combine the intuitive mental model of HTML & CSS with first-class document typesetting and page awareness.
              </p>

              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '16px', marginBottom: '32px' }}>
                <div style={{ padding: '20px', background: '#0f172a', border: '1px solid #1e293b', borderRadius: '8px' }}>
                  <h3 style={{ fontSize: '16px', fontWeight: 700, color: '#ef4444', marginBottom: '8px' }}>The Legacy Dilemma</h3>
                  <p style={{ fontSize: '13px', color: '#94a3b8', margin: 0 }}>
                    <strong>LaTeX</strong> has complex macros and silent compile errors. <strong>Markdown</strong> lacks page headers and multi-columns. <strong>DOCX/OpenXML</strong> is bloated binary zip archives impossible for humans or AI to reliably manipulate live.
                  </p>
                </div>
                <div style={{ padding: '20px', background: '#0f172a', border: '1px solid #1e293b', borderRadius: '8px' }}>
                  <h3 style={{ fontSize: '16px', fontWeight: 700, color: '#38bdf8', marginBottom: '8px' }}>The DDL Solution</h3>
                  <p style={{ fontSize: '13px', color: '#94a3b8', margin: 0 }}>
                    Clean XML syntax (`&lt;section&gt;`, `&lt;grid&gt;`, `&lt;paper&gt;`) coupled with true CSS styling, modular files (`.dxml` + `.css`), and a sub-millisecond Rust layout engine.
                  </p>
                </div>
              </div>
            </section>
          )}

          {activeSection === 'quickstart' && (
            <section>
              <h1 style={{ fontSize: '28px', fontWeight: 800, color: '#f8fafc', marginBottom: '16px' }}>Quickstart Guide</h1>
              <p style={{ color: '#94a3b8', marginBottom: '20px' }}>
                Create your first document definition in seconds. A minimal DDL file consists of a root <code>&lt;doc&gt;</code> containing <code>&lt;props&gt;</code> and <code>&lt;body&gt;</code>.
              </p>

              <CodeSnippet
                id="quickstart-code"
                copiedId={copiedCode}
                onCopy={copyToClipboard}
                code={`<doc>
  <props>
    <paper size="A4" orientation="portrait" margin="20mm" />
    <style>
      h1 { font-size: 24pt; color: #1e3a8a; }
      p { line-height: 1.5; color: #334155; }
    </style>
  </props>

  <body>
    <section>
      <h1>Hello World</h1>
      <p>This is my first document typeset natively with DDL.</p>
    </section>
  </body>
</doc>`}
              />
            </section>
          )}

          {activeSection === 'props-block' && (
            <section>
              <h1 style={{ fontSize: '28px', fontWeight: 800, color: '#f8fafc', marginBottom: '16px' }}>The &lt;props&gt; Configuration Block</h1>
              <p style={{ color: '#94a3b8', marginBottom: '20px' }}>
                Similar to <code>&lt;head&gt;</code> in HTML, the <code>&lt;props&gt;</code> block is the single point of truth for global document settings: paper geometries, linked stylesheets, and named page templates.
              </p>

              <CodeSnippet
                id="props-code"
                copiedId={copiedCode}
                onCopy={copyToClipboard}
                code={`<props>
  <!-- 1. Metadata -->
  <meta>
    <title>Quarterly Engineering Report</title>
    <author>Architecture Guild</author>
  </meta>

  <!-- 2. Paper Geometry Defaults -->
  <paper size="A4" orientation="portrait" margin="20mm" />

  <!-- 3. Linked or Inline Stylesheets -->
  <link rel="stylesheet" href="styles.css" />

  <!-- 4. Named Page Styles -->
  <page-styles>
    <page-style name="body" default="true">
      <running-header>
        <left><text>Confidential Report</text></left>
        <right><page-number/> of <total-pages/></right>
      </running-header>
      <running-footer>
        <center><text>© 2026 Acme Corp</text></center>
      </running-footer>
    </page-style>
  </page-styles>
</props>`}
              />
            </section>
          )}

          {activeSection === 'css-grid' && (
            <section>
              <h1 style={{ fontSize: '28px', fontWeight: 800, color: '#f8fafc', marginBottom: '16px' }}>Multi-Column Flow with &lt;grid&gt;</h1>
              <p style={{ color: '#94a3b8', marginBottom: '20px' }}>
                Traditional word processors require brittle section breaks to introduce columns. DDL allows any block to declare columns using the <code>&lt;grid&gt;</code> tag or CSS Grid, supporting infinite nesting without page overflow bugs.
              </p>

              <CodeSnippet
                id="grid-code"
                copiedId={copiedCode}
                onCopy={copyToClipboard}
                code={`<!-- Two Equal Columns -->
<grid cols="1fr 1fr" gap="6mm">
  <div>
    <h3>Column Left</h3>
    <p>Flows cleanly inside its cell.</p>
  </div>
  <div>
    <h3>Column Right</h3>
    <!-- Nested Inner Grid -->
    <grid cols="1fr 2fr" gap="3mm">
      <div>Sub A</div>
      <div>Sub B</div>
    </grid>
  </div>
</grid>`}
              />
            </section>
          )}

          {activeSection === 'external-css' && (
            <section>
              <h1 style={{ fontSize: '28px', fontWeight: 800, color: '#f8fafc', marginBottom: '16px' }}>Decoupled Styling with External CSS</h1>
              <p style={{ color: '#94a3b8', marginBottom: '20px' }}>
                DDL allows separating document data from presentation by linking external <code>.css</code> files.
              </p>

              <div style={{ padding: '16px', background: '#0f172a', borderLeft: '4pt solid #38bdf8', marginBottom: '20px', borderRadius: '4px' }}>
                <b>In document.dxml:</b>
                <pre style={{ margin: '8px 0 0 0', color: '#38bdf8', fontFamily: 'monospace' }}>
                  &lt;link rel="stylesheet" href="styles.css" /&gt;
                </pre>
              </div>

              <CodeSnippet
                id="ext-css-code"
                copiedId={copiedCode}
                onCopy={copyToClipboard}
                code={`/* In styles.css */
h1 {
  font-size: 26pt;
  font-weight: bold;
  color: #1e3a8a;
  margin-bottom: 6mm;
}

.callout {
  background-color: #f0fdf4;
  border-left: 4pt solid #16a34a;
  padding: 4mm 6mm;
  border-radius: 4px;
}`}
              />
            </section>
          )}

          {activeSection === 'engine-ast' && (
            <section>
              <h1 style={{ fontSize: '28px', fontWeight: 800, color: '#f8fafc', marginBottom: '16px' }}>Rust Engine & AST Core</h1>
              <p style={{ color: '#94a3b8', marginBottom: '20px' }}>
                The layout engine is engineered in Rust (`ddl-engine`) using memory-safe, high-performance AST structures.
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
