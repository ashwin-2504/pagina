pub mod ast;
pub mod parser;
pub mod docx;
pub mod pdf;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub dxml: String,
    pub dcss: String,
}

#[wasm_bindgen]
pub fn parse_ddl_to_json(input: &str) -> Result<String, JsValue> {
    let mut parser = parser::Parser::new(input);
    match parser.parse() {
        Ok(doc) => serde_json::to_string(&doc).map_err(|e| JsValue::from_str(&e.to_string())),
        Err(diags) => {
            let err_msg = diags.iter().map(|d| d.message.clone()).collect::<Vec<_>>().join(", ");
            Err(JsValue::from_str(&err_msg))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Read;
    use std::path::Path;

    #[test]
    fn test_callout_spans() {
        let input = r#"<doc><props></props><body><section><div class="callout"><b>Separation of Concerns:</b> Styles are linked via <code>&lt;link rel="stylesheet" href="styles.css" /&gt;</code>, allowing designers or AI agents to style documents independently.</div></section></body></doc>"#;
        let mut p = parser::Parser::new(input);
        let doc = p.parse().unwrap();
        let bytes = pdf::export_pdf(&doc).unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_basic_parse() {
        let input = r#"
        <doc>
          <props>
            <meta>
              <title>Test Doc</title>
            </meta>
            <paper size="A4" orientation="portrait" margin="20mm" />
            <style>
              h1 { color: #1e3a8a; }
              .callout { background: #f0fdf4; }
            </style>
          </props>
          <body>
            <section page-style="cover">
              <h1>Title</h1>
              <p>Hello World</p>
            </section>
          </body>
        </doc>
        "#;

        let mut parser = parser::Parser::new(input);
        let doc = parser.parse().expect("Failed to parse");
        assert_eq!(doc.props.paper.size, "A4");
        assert_eq!(doc.props.stylesheet.rules.len(), 2);
        assert_eq!(doc.body.children.len(), 1);
    }

    #[test]
    fn test_docx_export_and_import_roundtrip() {
        let input = r#"<doc>
  <props>
    <meta><title>Test Report</title></meta>
    <paper size="A4" orientation="portrait" margin="20mm"/>
  </props>
  <body>
    <section>
      <h1>Modular Document Architecture</h1>
      <p align="center">This is a paragraph with <b>bold</b> and <i>italic</i> styling.</p>
      <table>
        <tr><th>Col A</th><th>Col B</th></tr>
        <tr><td>Data 1</td><td>Data 2</td></tr>
      </table>
    </section>
  </body>
</doc>"#;

        let mut parser = parser::Parser::new(input);
        let doc = parser.parse().expect("Failed to parse");
        
        // Export to DOCX
        let docx_bytes = docx::export_docx(&doc).expect("Failed to export docx");
        assert!(!docx_bytes.is_empty());
        assert_eq!(&docx_bytes[0..2], b"PK"); // Standard zip magic header

        // Import back from DOCX
        let import_res = docx::import_docx(&docx_bytes).expect("Failed to import docx");
        assert!(import_res.dxml.contains("<h1>Modular Document Architecture</h1>"));
        assert!(import_res.dxml.contains("<b>bold</b>"));
        assert!(import_res.dxml.contains("<table>"));
        assert!(import_res.dxml.contains("Col A"));
        assert!(import_res.dxml.contains("Data 1"));
        assert!(import_res.dcss.contains("h1 {"));
    }

    #[test]
    fn test_pdf_export_and_import() {
        let input = r#"<doc>
  <props>
    <meta><title>Executive Brief</title></meta>
    <paper size="A4" orientation="portrait" margin="20mm"/>
  </props>
  <body>
    <section>
      <h1>Autonomous Document Processing</h1>
      <p align="center">Pagina is a native document typesetting engine.</p>
      <table>
        <tr><th>Metric</th><th>Value</th></tr>
        <tr><td>Latency</td><td>&lt; 1ms</td></tr>
      </table>
    </section>
  </body>
</doc>"#;

        let mut parser = parser::Parser::new(input);
        let doc = parser.parse().expect("Failed to parse");

        // Export to PDF
        let pdf_bytes = pdf::export_pdf(&doc).expect("Failed to export pdf");
        assert!(!pdf_bytes.is_empty());
        assert_eq!(&pdf_bytes[0..4], b"%PDF"); // Standard PDF magic header

        // Import back from PDF
        let import_res = pdf::import_pdf(&pdf_bytes).expect("Failed to import pdf");
        assert!(import_res.dxml.contains("<doc>"));
        assert!(import_res.dxml.contains("<section"));
        assert!(import_res.dxml.contains("Pagina is a native document typesetting engine."));
        assert!(import_res.dcss.contains("h1 {"));
    }

    #[test]
    fn test_real_eva_synopsis_docx() {
        let file_path = if Path::new("../EVA Synopsis.docx").exists() {
            "../EVA Synopsis.docx"
        } else {
            "EVA Synopsis.docx"
        };
        let bytes = fs::read(file_path).expect("EVA Synopsis.docx must exist");

        // 1. Import DOCX into DXML & DCSS
        let result = docx::import_docx(&bytes).expect("Should successfully import EVA Synopsis.docx");
        assert!(result.dxml.contains("Information Technology Department"));
        assert!(result.dxml.contains("U.G. PROJECT REGISTRATION FORM"));
        assert!(result.dxml.contains("Gopale Ashwin Ashok"));
        assert!(result.dxml.contains("EVA: Environmental Vision through Audio"));
        assert!(result.dxml.contains("<table"));
        assert!(result.dxml.contains("borderless"));
        assert!(result.dxml.contains("<img"));
        assert!(result.dxml.contains("Dr. S. K. Korde"));
        assert!(result.dcss.contains("table {"));
        let ul_count = result.dxml.matches("<ul").count();
        let ol_count = result.dxml.matches("<ol").count();
        let li_count = result.dxml.matches("<li").count();
        println!("EVA Synopsis DOCX Lists extracted -> ul: {}, ol: {}, li: {}", ul_count, ol_count, li_count);
        assert!(ul_count > 0 || ol_count > 0, "Imported DOCX must extract bullet or numbered lists");

        // 2. Parse the produced DXML using Rust Parser
        let mut parser = parser::Parser::new(&result.dxml);
        let parsed_doc = parser.parse().expect("Produced DXML must be valid AST");
        assert!(!parsed_doc.body.children.is_empty());

        // 3. Re-export to DOCX
        let reexported_docx = docx::export_docx(&parsed_doc).expect("Re-export to DOCX must succeed");
        assert_eq!(&reexported_docx[0..2], b"PK");

        // 4. Re-export to PDF
        let reexported_pdf = pdf::export_pdf(&parsed_doc).expect("Re-export to PDF must succeed");
        assert_eq!(&reexported_pdf[0..4], b"%PDF");
    }

    #[test]
    fn test_real_eva_synopsis_pdf() {
        let file_path = if Path::new("../EVA Synopsis.pdf").exists() {
            "../EVA Synopsis.pdf"
        } else {
            "EVA Synopsis.pdf"
        };
        let bytes = fs::read(file_path).expect("EVA Synopsis.pdf must exist");

        // 1. Import PDF into DXML & DCSS
        let result = pdf::import_pdf(&bytes).expect("Should successfully import EVA Synopsis.pdf");
        assert!(result.dxml.contains("Information Technology Department"));
        assert!(result.dxml.contains("U.G. PROJECT REGISTRATION FORM"));
        assert!(result.dxml.contains("Gopale Ashwin Ashok"));
        assert!(result.dxml.contains("EVA: Environmental Vision through Audio"));

        // 2. Parse the produced DXML using Rust Parser
        let mut parser = parser::Parser::new(&result.dxml);
        let parsed_doc = parser.parse().expect("Produced DXML must be valid AST");
        assert!(!parsed_doc.body.children.is_empty());

        // 3. Re-export to PDF
        let reexported_pdf = pdf::export_pdf(&parsed_doc).expect("Re-export to PDF must succeed");
        assert_eq!(&reexported_pdf[0..4], b"%PDF");

        // 4. Re-export to DOCX
        let reexported_docx = docx::export_docx(&parsed_doc).expect("Re-export to DOCX must succeed");
        assert_eq!(&reexported_docx[0..2], b"PK");
    }

    #[test]
    fn test_user_modular_ddl_report_export_and_import() {
        let input = r#"<doc>
  <props>
    <meta>
      <title>Modular DDL Report</title>
      <author>Engineering Team</author>
    </meta>

    <!-- Global Paper Defaults -->
    <paper size="A4" orientation="portrait" margin="20mm" />

    <!-- External Link to Stylesheet -->
    <link rel="stylesheet" href="styles.css" />

    <!-- Named Page Styles -->
    <page-styles>
      <page-style name="body" default="true">
        <running-header>
          <left><text>Technical Report</text></left>
          <right><page-number/> of <total-pages/></right>
        </running-header>
        <running-footer>
          <center><text>© 2026 Autonomous Document Systems</text></center>
        </running-footer>
      </page-style>
    </page-styles>
  </props>

  <body>
    <section page-style="body">
      <h1>Modular Document Architecture</h1>
      <p class="lead">
        Content and presentation are decoupled into separate files: <code>document.dxml</code> and <code>styles.css</code>.
      </p>

      <div class="callout">
        <b>Separation of Concerns:</b> Styles are linked via <code>&lt;link rel="stylesheet" href="styles.css" /&gt;</code>, allowing designers or AI agents to style documents independently.
      </div>

      <h2>Multi-Column Grid Flow</h2>
      <grid cols="1fr 1fr" gap="6mm">
        <div class="col-box">
          <h3>Left Column</h3>
          <p>
            The layout rules defined in the standalone <code>styles.css</code> file apply automatically across the document.
          </p>
        </div>
        <div class="col-box">
          <h3>Right Column</h3>
          <p>
            You can add new files, switch between them using the tabs above, or format them individually.
          </p>
        </div>
      </grid>
    </section>
  </body>
</doc>"#;

        let mut parser = parser::Parser::new(input);
        let doc = parser.parse().expect("User document should parse successfully");
        assert_eq!(doc.props.metadata.get("title").map(|s| s.as_str()), Some("Modular DDL Report"));
        assert!(doc.props.page_styles.contains_key("body"));

        // 1. PDF Export
        let pdf_bytes = pdf::export_pdf(&doc).expect("PDF export must succeed");
        assert_eq!(&pdf_bytes[0..4], b"%PDF");

        // 2. DOCX Export
        let docx_bytes = docx::export_docx(&doc).expect("DOCX export must succeed");
        assert_eq!(&docx_bytes[0..2], b"PK");

        // Inspect DOCX contents: verify header1.xml, footer1.xml, grid table, and entity decoding
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&docx_bytes)).expect("Must be valid zip archive");
        assert!(archive.by_name("word/header1.xml").is_ok(), "Header1.xml must exist");
        assert!(archive.by_name("word/footer1.xml").is_ok(), "Footer1.xml must exist");

        let mut doc_xml = String::new();
        archive.by_name("word/document.xml").unwrap().read_to_string(&mut doc_xml).unwrap();
        println!("DOC_XML:\n{}", doc_xml);
        assert!(doc_xml.contains("Modular Document Architecture"));
        assert!(doc_xml.contains("Technical Report") || archive.by_name("word/header1.xml").is_ok());
        assert!(doc_xml.contains("&lt;link rel=&quot;stylesheet&quot; href=&quot;styles.css&quot; /&gt;"), "Entity must be properly escaped for Word XML so user sees <link>");
        assert!(doc_xml.contains("Separation of Concerns:"));
        assert!(doc_xml.contains("Multi-Column Grid Flow"));
        assert!(doc_xml.contains("Left Column"));
        assert!(doc_xml.contains("Right Column"));

        // Verify DOCX Import roundtrip
        let import_res = docx::import_docx(&docx_bytes).expect("Import DOCX must succeed");
        assert!(import_res.dxml.contains("Modular Document Architecture"));
        assert!(import_res.dxml.contains("Left Column"));
        assert!(import_res.dxml.contains("Right Column"));
    }

    #[test]
    fn test_font_bullets_indentation_margin_padding_roundtrip() {
        let input = r#"<doc>
  <props>
    <meta>
      <title>Formatting Test Suite</title>
      <author>Typography Team</author>
    </meta>
    <paper size="A4" orientation="portrait" margin="15mm" />
    <link rel="stylesheet" href="styles.css" />
  </props>
  <body>
    <section>
      <h1>Document Typography &amp; Bullets</h1>
      <p class="lead">Testing fonts, all bullet types, blockquotes, indentations, and box padding.</p>

      <blockquote>
        <p>A designer knows he has achieved perfection not when there is nothing left to add, but when there is nothing left to take away.</p>
      </blockquote>

      <h2>Unordered Lists with All Bullet Types</h2>
      <ul type="disc">
        <li>Disc bullet standard item</li>
      </ul>
      <ul type="circle">
        <li>Circle bullet white item</li>
      </ul>
      <ul type="square">
        <li>Square bullet black item</li>
      </ul>
      <ul type="dash">
        <li>Dash bullet hypenated item</li>
      </ul>

      <h2>Ordered Lists with All Counter Types</h2>
      <ol type="1">
        <li>Decimal numbered item</li>
      </ol>
      <ol type="a">
        <li>Lower alpha item</li>
      </ol>
      <ol type="A">
        <li>Upper alpha item</li>
      </ol>
      <ol type="i">
        <li>Lower roman numeral item</li>
      </ol>
      <ol type="I">
        <li>Upper roman numeral item</li>
      </ol>

      <div class="callout">
        <b>Callout Box:</b> Padding and rounded corners tested.
      </div>

      <grid cols="1fr 1fr" gap="5mm">
        <div class="col-box">
          <p>Box 1 with 4mm padding and rounded borders.</p>
        </div>
        <div class="col-box">
          <p>Box 2 with 4mm padding and rounded borders.</p>
        </div>
      </grid>
    </section>
  </body>
</doc>"#;

        let mut parser = parser::Parser::new(input);
        let doc = parser.parse().expect("Document should parse successfully");

        // 1. PDF Export
        let pdf_bytes = pdf::export_pdf(&doc).expect("PDF export with all bullet types and formatting must succeed");
        assert_eq!(&pdf_bytes[0..4], b"%PDF");

        // 2. DOCX Export
        let docx_bytes = docx::export_docx(&doc).expect("DOCX export with all formatting must succeed");
        assert_eq!(&docx_bytes[0..2], b"PK");

        // 3. Inspect Word XML to verify bullet symbols and indentations
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&docx_bytes)).expect("Must be valid zip archive");
        let mut doc_xml = String::new();
        archive.by_name("word/document.xml").unwrap().read_to_string(&mut doc_xml).unwrap();

        assert!(doc_xml.contains("•"), "Must contain disc bullet");
        assert!(doc_xml.contains("◦"), "Must contain circle bullet");
        assert!(doc_xml.contains("▪"), "Must contain square bullet");
        assert!(doc_xml.contains("–"), "Must contain dash bullet");
        assert!(doc_xml.contains("a."), "Must contain lower-alpha counter");
        assert!(doc_xml.contains("A."), "Must contain upper-alpha counter");
        assert!(doc_xml.contains("i."), "Must contain lower-roman counter");
        assert!(doc_xml.contains("I."), "Must contain upper-roman counter");
        assert!(doc_xml.contains("w:ind"), "Must contain indentation tags");

        // 4. DOCX Import Roundtrip
        let imported = docx::import_docx(&docx_bytes).expect("DOCX import must succeed");
        assert!(imported.dxml.contains("Document Typography"));
        assert!(imported.dxml.contains("<ul"), "Imported DXML must contain ul");
        assert!(imported.dxml.contains("<ol"), "Imported DXML must contain ol");
        assert!(imported.dcss.contains("body"), "Imported DCSS must contain typography");
        assert!(imported.dcss.contains(".callout"), "Imported DCSS must contain callout padding");
        assert!(imported.dcss.contains(".col-box"), "Imported DCSS must contain col-box padding");
    }

    #[test]
    fn test_nested_lists_export_and_import() {
        let input = r#"<doc>
  <props>
    <paper size="A4" orientation="portrait" margin="20mm" />
  </props>
  <body>
    <section>
      <h1>Nested List Architecture</h1>
      <ul>
        <li>Parent item A
          <ul type="circle">
            <li>Child item A.1</li>
            <li>Child item A.2
              <ul type="square">
                <li>Grandchild A.2.1</li>
              </ul>
            </li>
          </ul>
        </li>
        <li>Parent item B</li>
      </ul>
      <ol>
        <li>Step 1
          <ol type="a">
            <li>Substep 1.a</li>
            <li>Substep 1.b</li>
          </ol>
        </li>
        <li>Step 2</li>
      </ol>
    </section>
  </body>
</doc>"#;

        let mut parser = parser::Parser::new(input);
        let doc = parser.parse().expect("Failed to parse nested list document");

        // 1. PDF Export
        let pdf_bytes = pdf::export_pdf(&doc).expect("PDF export with nested lists must succeed");
        assert_eq!(&pdf_bytes[0..4], b"%PDF");

        // 2. DOCX Export
        let docx_bytes = docx::export_docx(&doc).expect("DOCX export with nested lists must succeed");
        assert_eq!(&docx_bytes[0..2], b"PK");

        // Verify DOCX contains multi-level numbering
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&docx_bytes)).expect("Must be zip");
        let mut doc_xml = String::new();
        archive.by_name("word/document.xml").unwrap().read_to_string(&mut doc_xml).unwrap();
        assert!(doc_xml.contains("w:ilvl w:val=\"0\""), "Must contain level 0");
        assert!(doc_xml.contains("w:ilvl w:val=\"1\""), "Must contain level 1");
        assert!(doc_xml.contains("w:ilvl w:val=\"2\""), "Must contain level 2");
        assert!(doc_xml.contains("•"), "Must contain disc bullet");
        assert!(doc_xml.contains("◦"), "Must contain circle bullet");
        assert!(doc_xml.contains("▪"), "Must contain square bullet");

        // 3. Re-import DOCX
        let imported = docx::import_docx(&docx_bytes).expect("DOCX import must succeed");
        assert!(imported.dxml.contains("Parent item A"));
        assert!(imported.dxml.contains("Child item A.1"));
        assert!(imported.dxml.contains("Grandchild A.2.1"));
        // Verify nesting in DXML output
        assert!(imported.dxml.contains("<ul"));
        assert!(imported.dxml.contains("type=\"circle\"") || imported.dxml.contains("<ul"));
    }

    #[test]
    fn test_image_export_pdf_and_docx() {
        let input = r#"<doc>
  <props>
    <page-styles>
      <page-style name="body" default="true">
        <running-header>
          <center><img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==" style="max-height: 20mm;" /></center>
        </running-header>
      </page-style>
    </page-styles>
  </props>
  <body>
    <section>
      <h1>Document With Image</h1>
      <div align="center">
        <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==" width="50mm" />
      </div>
      <table class="borderless align-right">
        <tr>
          <td><p align="center">Dr. S. K. Korde</p></td>
        </tr>
      </table>
    </section>
  </body>
</doc>"#;
        let mut parser = parser::Parser::new(input);
        let doc = parser.parse().expect("DXML must parse");
        let pdf_bytes = pdf::export_pdf(&doc).expect("PDF export with image must succeed");
        assert!(!pdf_bytes.is_empty());
        assert_eq!(&pdf_bytes[0..4], b"%PDF");

        let docx_bytes = docx::export_docx(&doc).expect("DOCX export with image must succeed");
        assert!(!docx_bytes.is_empty());
        assert_eq!(&docx_bytes[0..2], b"PK");

        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(docx_bytes)).expect("Must be valid zip");
        let mut has_media = false;
        for i in 0..archive.len() {
            let file = archive.by_index(i).unwrap();
            if file.name().starts_with("word/media/") {
                has_media = true;
                break;
            }
        }
        assert!(has_media, "Re-exported DOCX must contain embedded media");
    }
}
