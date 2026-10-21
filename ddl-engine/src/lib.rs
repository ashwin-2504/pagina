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
    use std::path::Path;

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
        assert!(result.dxml.contains("<table>"));
        assert!(result.dxml.contains("Dr. S. K. Korde"));
        assert!(result.dcss.contains("table {"));

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
}
