# DDL (Document Definition Language) Grammar, AST & Design Specification

This document defines the architecture, syntax grammar, and AST design for the custom XML + CSS Document Definition Language, incorporating the developer's core principles:
1. **HTML & CSS Mental Model**: Keep it as intuitive as writing HTML tags (`h1`-`h6`, `p`, `b`, `i`, `table`, `grid`, etc.) and CSS styling.
2. **`<props>` Configuration Block**: A single, clean header block (analogous to `<head>` in HTML) for global/one-time configuration (paper geometry, default margins, base fonts, watermarks, and named page styles).
3. **Pluggable Named Page Layouts**: Handle changing headers/footers cleanly (e.g., Cover page without header/footer, Preface with Roman numerals `i, ii`, Main body with Arabic numbers `1, 2`).
4. **CSS Grid / Column Units for Flexible Nesting**: Any block can declare `display: grid;` or `<grid cols="1fr 1fr">` to create nested multi-column layouts without breaking document geometry.
5. **Phase Separation**: Defer hyper-complex TeX-level features (e.g., raw LaTeX math macros, exotic OpenType kerning) to later phases, prioritizing a solid, rock-solid core layout engine.

---

## 1. Document Structure & Example

```xml
<doc>
  <!-- 1. Global Setup & Metadata (The <props> block) -->
  <props>
    <meta>
      <title>Autonomous Systems Engineering Specification</title>
      <author>Engineering Team</author>
      <date>2026-10-21</date>
    </meta>

    <!-- Global Paper Defaults -->
    <paper size="A4" orientation="portrait" margin="20mm" />

    <!-- Default Document Typography & Global Styling -->
    <style>
      doc {
        font-family: "Inter", sans-serif;
        font-size: 11pt;
        line-height: 1.5;
        color: #1a1a1a;
      }

      h1 { font-size: 24pt; font-weight: bold; margin-bottom: 8mm; color: #1e3a8a; }
      h2 { font-size: 16pt; font-weight: bold; margin-top: 6mm; margin-bottom: 4mm; color: #1e293b; }
      p  { margin-bottom: 4mm; text-align: justify; }

      .callout {
        background: #f0fdf4;
        border-left: 4pt solid #16a34a;
        padding: 4mm 6mm;
        margin: 4mm 0;
        border-radius: 2pt;
      }

      .highlight { background-color: #fef08a; }
      .text-danger { color: #dc2626; }

      /* CSS Grid support for multi-column structures */
      .two-col {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 8mm;
      }

      .sidebar-layout {
        display: grid;
        grid-template-columns: 2fr 1fr;
        gap: 10mm;
      }
    </style>

    <!-- Named Page Templates: Switchable per section -->
    <page-styles>
      <!-- Cover Page: No running header or footer -->
      <page-style name="cover">
        <margins top="30mm" bottom="30mm" left="25mm" right="25mm" />
        <watermark text="CONFIDENTIAL" opacity="0.08" angle="-45" />
      </page-style>

      <!-- Frontmatter: Roman numeral numbering -->
      <page-style name="frontmatter">
        <running-header>
          <right><text>Specification Overview</text></right>
        </running-header>
        <running-footer>
          <center><page-number format="roman" /></center>
        </running-footer>
      </page-style>

      <!-- Standard Body: Arabic numbering with total page count -->
      <page-style name="body" default="true">
        <running-header>
          <left><doc-title /></left>
          <right><section-title /></right>
        </running-header>
        <running-footer>
          <left>© 2026 Acme Corp</left>
          <right><page-number format="decimal" /> of <total-pages /></right>
        </running-footer>
      </page-style>
    </page-styles>
  </props>

  <!-- 2. Document Content Body -->
  <body>
    <!-- Apply the 'cover' page style -->
    <section page-style="cover">
      <h1 style="text-align: center; margin-top: 60mm; font-size: 32pt;">Next-Gen Engine Architecture</h1>
      <p style="text-align: center; font-size: 14pt; color: #64748b;">A declarative typesetting system for users and AI agents</p>
    </section>

    <!-- Break to frontmatter with roman numerals -->
    <section page-style="frontmatter" page-break="true">
      <h2>Executive Summary</h2>
      <p>This document details our novel XML-based document definition format.</p>
    </section>

    <!-- Standard body pages -->
    <section page-style="body" page-break="true">
      <h2>1. Architectural Overview</h2>
      <p>
        By using standard tags like <b>bold</b>, <i>italic</i>, and <span class="highlight">highlight</span>,
        authors experience zero friction.
      </p>

      <!-- Multi-column grid flow (can be nested cleanly) -->
      <grid cols="1fr 1fr" gap="8mm">
        <div>
          <h3>Left Column</h3>
          <p>This column flows independently inside its cell.</p>
          
          <!-- Nested sub-grid -->
          <grid cols="1fr 1fr" gap="4mm" style="background: #f8fafc; padding: 2mm;">
            <div><b>Sub-col A</b></div>
            <div><b>Sub-col B</b></div>
          </grid>
        </div>
        <div>
          <h3>Right Column</h3>
          <div class="callout">
            A styled callout box created with regular CSS classes.
          </div>
        </div>
      </grid>

      <!-- Tables: Clean, HTML-like structure -->
      <table style="width: 100%; margin-top: 6mm;">
        <thead>
          <tr>
            <th>Feature</th>
            <th>DDL Implementation</th>
            <th>Advantage</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>Page Setup</td>
            <td><code>&lt;props&gt;</code> block</td>
            <td>Single point of truth</td>
          </tr>
          <tr>
            <td>Multi-column</td>
            <td><code>&lt;grid&gt;</code> or CSS Grid</td>
            <td>Arbitrary nesting without page crashes</td>
          </tr>
        </tbody>
      </table>
    </section>
  </body>
</doc>
```

---

## 2. EBNF Grammar

```ebnf
Document       ::= S* '<doc>' S* Props? S* Body S* '</doc>' S*

/* Props & Configuration Block */
Props          ::= '<props>' S* PropsMember* S* '</props>'
PropsMember    ::= Meta | Paper | StyleBlock | PageStyles
Meta           ::= '<meta>' S* ( '<' TagName '>' Text '</' TagName '>' S* )* '</meta>'
Paper          ::= '<paper' S+ AttributeList S* '/>'
StyleBlock     ::= '<style>' Stylesheet '</style>'
PageStyles     ::= '<page-styles>' S* PageStyleDef* S* '</page-styles>'
PageStyleDef   ::= '<page-style' S+ AttributeList S* '>' S* PageStyleMember* S* '</page-style>'
PageStyleMember::= Margins | RunningHeader | RunningFooter | Watermark

RunningHeader  ::= '<running-header' AttributeList? '>' S* HeaderFooterContent* S* '</running-header>'
RunningFooter  ::= '<running-footer' AttributeList? '>' S* HeaderFooterContent* S* '</running-footer>'
HeaderFooterContent ::= '<' ('left'|'center'|'right') '>' Node* '</' ('left'|'center'|'right') '>'
Watermark      ::= '<watermark' S+ AttributeList S* '/>'

/* Content Body */
Body           ::= '<body' AttributeList? '>' S* Node* S* '</body>'

Node           ::= Section | BlockElement | Text | Comment

Section        ::= '<section' AttributeList? '>' S* Node* S* '</section>'

BlockElement   ::= '<' BlockTag AttributeList? ( '/>' | '>' Node* '</' BlockTag '>' )
BlockTag       ::= 'h1' | 'h2' | 'h3' | 'h4' | 'h5' | 'h6' 
                 | 'p' | 'div' | 'grid' | 'table' | 'thead' | 'tbody' | 'tr' | 'th' | 'td'
                 | 'ul' | 'ol' | 'li' | 'figure' | 'callout' | 'hr'

InlineElement  ::= '<' InlineTag AttributeList? ( '/>' | '>' Node* '</' InlineTag '>' )
InlineTag      ::= 'b' | 'strong' | 'i' | 'em' | 'u' | 's' | 'sup' | 'sub' 
                 | 'span' | 'code' | 'page-number' | 'total-pages' | 'doc-title' | 'section-title'

AttributeList  ::= ( S+ AttrName S* '=' S* ( '"' AttrValue '"' | "'" AttrValue "'" ) )*

TagName        ::= [a-zA-Z_][a-zA-Z0-9_\-]*
AttrName       ::= [a-zA-Z_][a-zA-Z0-9_\-]*
AttrValue      ::= [^"']*
Text           ::= [^<>&]+

/* CSS Stylesheet Grammar */
Stylesheet     ::= ( Rule | S )*
Rule           ::= SelectorList S* '{' S* DeclarationList S* '}'
SelectorList   ::= Selector ( S* ',' S* Selector )*
Selector       ::= ( TagSelector | ClassSelector | IdSelector )+
TagSelector    ::= [a-zA-Z_][a-zA-Z0-9_\-]*
ClassSelector  ::= '.' [a-zA-Z0-9_\-]+
IdSelector     ::= '#' [a-zA-Z0-9_\-]+

DeclarationList::= Declaration ( S* ';' S* Declaration )* S* ';'?
Declaration    ::= Property S* ':' S* Value
Property       ::= [a-zA-Z\-]+
Value          ::= [^;{}]+
```

---

## 3. Rust AST Data Structures

```rust
use std::collections::HashMap;

/// Source code location span for diagnostics and bi-directional cursor jumping
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub col: usize,
}

/// The Root Document AST
#[derive(Debug, Clone)]
pub struct Document {
    pub props: DocumentProps,
    pub body: BodyNode,
    pub span: Span,
}

/// Props: Header metadata, default paper, styles, and page layouts
#[derive(Debug, Clone, Default)]
pub struct DocumentProps {
    pub metadata: HashMap<String, String>,
    pub default_paper: PaperConfig,
    pub stylesheet: Stylesheet,
    pub page_styles: HashMap<String, PageStyle>,
    pub default_page_style: String,
}

#[derive(Debug, Clone)]
pub struct PaperConfig {
    pub size: PaperSize,
    pub orientation: Orientation,
    pub margins: Margins,
}

#[derive(Debug, Clone)]
pub enum PaperSize {
    A4,
    A3,
    A5,
    Letter,
    Legal,
    Custom { width_mm: f32, height_mm: f32 },
}

#[derive(Debug, Clone)]
pub enum Orientation {
    Portrait,
    Landscape,
}

#[derive(Debug, Clone)]
pub struct Margins {
    pub top: Length,
    pub bottom: Length,
    pub left: Length,
    pub right: Length,
}

/// Dynamic Page Style (swappable per section or page break)
#[derive(Debug, Clone)]
pub struct PageStyle {
    pub name: String,
    pub margins: Option<Margins>,
    pub running_header: Option<HeaderFooterDef>,
    pub running_footer: Option<HeaderFooterDef>,
    pub watermark: Option<WatermarkDef>,
}

#[derive(Debug, Clone)]
pub struct HeaderFooterDef {
    pub left: Vec<Node>,
    pub center: Vec<Node>,
    pub right: Vec<Node>,
}

#[derive(Debug, Clone)]
pub struct WatermarkDef {
    pub text: String,
    pub opacity: f32,
    pub angle_deg: f32,
}

/// Content Nodes
#[derive(Debug, Clone)]
pub struct BodyNode {
    pub attributes: HashMap<String, String>,
    pub children: Vec<Node>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Node {
    Section(SectionNode),
    Element(ElementNode),
    Text(TextNode),
    Comment(String),
}

#[derive(Debug, Clone)]
pub struct SectionNode {
    pub page_style: Option<String>,
    pub force_page_break: bool,
    pub attributes: HashMap<String, String>,
    pub children: Vec<Node>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ElementNode {
    pub tag: String,
    pub attributes: HashMap<String, String>,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub inline_styles: HashMap<String, Value>,
    pub children: Vec<Node>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TextNode {
    pub content: String,
    pub span: Span,
}

/// CSS Sub-AST
#[derive(Debug, Clone, Default)]
pub struct Stylesheet {
    pub rules: Vec<StyleRule>,
}

#[derive(Debug, Clone)]
pub struct StyleRule {
    pub selectors: Vec<Selector>,
    pub declarations: HashMap<String, Value>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Selector {
    Tag(String),
    Class(String),
    Id(String),
    Compound(Vec<Selector>), // e.g. "h1.title"
}

#[derive(Debug, Clone)]
pub enum Value {
    Length(Length),
    Color(Color),
    Keyword(String),
    Number(f32),
    StringLiteral(String),
    GridTemplate(Vec<GridTrack>),
}

#[derive(Debug, Clone)]
pub struct Length {
    pub val: f32,
    pub unit: Unit,
}

#[derive(Debug, Clone)]
pub enum Unit {
    Mm,
    Cm,
    Inch,
    Pt,
    Px,
    Em,
    Percent,
}

#[derive(Debug, Clone)]
pub enum GridTrack {
    Fr(f32),
    Length(Length),
    Auto,
}

#[derive(Debug, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32,
}
```

---

## 4. Key Design Decisions

1. **`<props>` Tag as Single Source of Truth**:
   - Like `<head>` in HTML, `<props>` houses metadata, `<paper>`, `<style>`, and `<page-styles>`.
   - AI agents and users don't have to search through the document to find where margins or fonts were altered.

2. **Named Page Styles (`<page-styles>`)**:
   - Solves the classic Word frustration: having one cover page without headers, followed by preface pages with small roman numerals (`i, ii`), followed by the body with Arabic numbers (`1, 2`).
   - Sections simply declare `<section page-style="cover">` or `<section page-style="body" page-break="true">`.

3. **Multi-Column via CSS Grid & `<grid>` Tag**:
   - Instead of complex multi-column section breaks that break document continuity, authors can use CSS `display: grid; grid-template-columns: 1fr 1fr;` or `<grid cols="1fr 1fr" gap="8mm">`.
   - Supports nesting columns inside columns without layout corruption.

4. **Familiar Text Tags**:
   - Standard HTML tags (`h1`-`h6`, `p`, `b`, `i`, `u`, `s`, `sup`, `sub`, `code`, `table`, `ul`, `ol`, `li`, `span`, `div`) work out of the box with CSS styling.
