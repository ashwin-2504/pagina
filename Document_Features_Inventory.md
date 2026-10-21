# Comprehensive Document Editing Feature Inventory & Mapping

This document catalogues all essential features and capabilities present in professional word processors (Microsoft Word, LibreOffice Writer, OnlyOffice, WPS Office, and Google Docs). It serves as our baseline matrix to design the syntax, CSS styling capabilities, and AST models for our custom Document Definition Language (DDL).

---

## 1. Page & Layout Architecture

| Category | Feature Description | Traditional Word Processor Implementation |
| :--- | :--- | :--- |
| **Paper Standards** | Standard paper sizes & custom geometries | A4, A3, A5, US Letter, Legal, Tabloid, custom `width` × `height`. |
| **Orientation** | Page direction | Portrait vs. Landscape (configurable per section). |
| **Margins** | Content boundary clearance | Top, Bottom, Left, Right margins; Mirror margins (Inside/Outside for book binding / facing pages); Gutter margin. |
| **Sections & Breaks** | Layout partition boundaries | Page break, Column break, Section break (Next Page, Continuous, Even/Odd page for book chapters). |
| **Multi-Column Flow** | Column layouts | 1, 2, 3, or arbitrary *N* columns with configurable gap, column divider lines, and unbalanced vs. balanced column filling. |
| **Running Headers & Footers** | Persistent page top/bottom content | First page different, Odd/Even different headers; distance from edge; dynamic page numbers (`Page X of Y`), dynamic section title extraction. |
| **Page Background & Watermark** | Page-level decorative layers | Solid color, image background, full-page watermark text (diagonal/horizontal) with opacity. |

---

## 2. Typography & Character Formatting (Inline Level)

| Feature | Details & Edge Cases |
| :--- | :--- |
| **Typeface & Size** | Font family, fallback chains, fractional point sizes (e.g. `10.5pt`). |
| **Basic Styles** | Bold, Italic, Oblique, Underline (single, double, dotted, wavy, color-specific), Strikethrough, Double Strikethrough. |
| **Scripting** | Superscript (e.g. $x^2$, footnote references) and Subscript (e.g. $H_2O$). |
| **Text Color & Highlight** | Font color (RGB/Hex), text highlight/marker color (background of glyph span). |
| **Spacing & Kerning** | Tracking/letter spacing, OpenType ligature control (common, discretionary, tabular numbers), font kerning. |
| **Inline Code & Math** | Monospace inline spans, inline LaTeX/MathML equations. |
| **Text Effects** | Capitalization transformations (Uppercase, Lowercase, Title Case, Small Caps). |

---

## 3. Paragraph & Block Formatting

| Feature | Details & Edge Cases |
| :--- | :--- |
| **Alignment & Justification**| Left, Right, Center, Justified (with micro-spacing or hyphenation control). |
| **Line Spacing** | Single, 1.15, 1.5, Double, Exactly $N$ pt, At least $N$ pt. |
| **Paragraph Spacing** | Space before (margin-top) and Space after (margin-bottom), paragraph collapse handling. |
| **Indentation** | Left indent, Right indent, First-line indent (traditional novels/essays), Hanging indent (citations, bibliographies). |
| **Pagination Rules** | **Keep with next** (prevents orphaned headings), **Keep lines together** (prevents paragraph splitting across pages), **Widow/Orphan control** (minimum 2 lines at page top/bottom). |
| **Drop Caps** | Large capital letter dropped into the first 2–4 lines of a paragraph. |
| **Borders & Shading** | Paragraph-level border (top, bottom, left, right, custom stroke & color) and background tint. |

---

## 4. Lists & Outlines

| Feature | Details & Edge Cases |
| :--- | :--- |
| **Bullet Lists** | Disc, circle, square, hyphen, custom unicode bullet, custom image bullet. |
| **Numbered Lists** | Decimal (`1, 2, 3`), Roman (`I, II, III` / `i, ii, iii`), Alphabetical (`A, B, C` / `a, b, c`), with custom prefix/suffix (`1.`, `1)`, `(1)`). |
| **Multilevel Lists / Outline** | Legal numbering (`1.1`, `1.1.1`), hierarchical indentation, auto-restart or continue numbering across sections. |
| **Task / Checklist** | Interactive or static checkbox (`[ ]`, `[x]`). |

---

## 5. Tables & Tabular Layouts

| Feature | Details & Edge Cases |
| :--- | :--- |
| **Grid Specification** | Fixed column widths, proportional (`1fr 2fr`), or auto-fit to content/window. |
| **Cell Merging** | `rowspan` (vertical spanning) and `colspan` (horizontal spanning). |
| **Header Handling** | Repeat header row on every new page when a table breaks across pages. |
| **Borders & Padding** | Cell padding (top/bottom/left/right), inner grid lines, outer frame, customizable border style/thickness/color. |
| **Cell Alignment** | Horizontal (left, center, right) and Vertical alignment (top, middle, bottom). |
| **Page Break in Table** | Allow row to break across pages vs. `cantSplit` (keep row intact). |

---

## 6. Media, Graphics & Floats

| Feature | Details & Edge Cases |
| :--- | :--- |
| **Images & Vectors** | Raster formats (PNG, JPEG, WebP) and Vector formats (SVG). |
| **Dimensions & Sizing** | Explicit width/height, max-width, aspect ratio preservation, object-fit (cover, contain). |
| **Wrap Text Modes** | **Inline with text**, **Square/Tight** (text flows around bounding box), **Top & Bottom** (clears text), **Behind text / In front of text** (absolute overlay). |
| **Captions** | Automated numbering (`Figure 1: Title`), cross-referencing capabilities. |
| **Callout Boxes / Admonitions** | Distinct boxed containers with icon, colored border, and background for tips, notes, warnings. |

---

## 7. Academic & Structural References

| Feature | Details & Edge Cases |
| :--- | :--- |
| **Footnotes** | Anchored in paragraph text, rendered at the bottom of the exact page containing the anchor; auto-numbering ($1, 2, 3$). |
| **Endnotes** | Rendered at the end of the section or end of the document ($i, ii, iii$). |
| **Table of Contents (TOC)** | Auto-generated from heading hierarchy (`h1`-`h6`), with leader dots (`......`) and page numbers. |
| **Citations & Bibliography**| In-text citations `[@smith2024]`, bibliography section with hanging indents. |
| **Cross-References** | Dynamic links to sections, figures, tables, or page numbers ("see Figure 2 on page 14"). |

---

## 8. Proofing, Metadata & Document Control

| Feature | Details & Edge Cases |
| :--- | :--- |
| **Document Metadata** | Title, Subject, Author, Keywords, Created/Modified timestamps. |
| **Comments / Annotations**| Anchor to a span of text with author name, timestamp, and comment content. |
| **Change Tracking** | Insertion marks (underline + color) and deletion marks (strikethrough + color) with revision author metadata. |

---

## 9. Next Step: Mapping to Our DDL Architecture

To make our language both **beginner-friendly** and **AI-agent friendly**, we will classify each feature into:
1. **XML Structural Elements** (e.g. `<doc>`, `<section>`, `<table>`, `<footnote>`, `<callout>`).
2. **CSS Styling Properties** (e.g. `page-break-inside: avoid;`, `line-height: 1.5;`, `font-family: ...;`, `margin-outside: 30mm;`).
3. **Rust Layout Engine Responsibilities** (e.g. Knuth-Plass line breaking, Footnote split budget on page bottom, Table header repetition).
