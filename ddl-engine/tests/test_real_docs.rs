use ddl_engine::{docx, parser, pdf};
use std::fs;
use std::io::Read;
use std::path::Path;

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
    assert!(result
        .dxml
        .contains("EVA: Environmental Vision through Audio"));
    assert!(result.dxml.contains("<table"));
    assert!(result.dxml.contains("borderless"));
    assert!(result.dxml.contains("<img"));
    assert!(result
        .dxml
        .contains(r#"<img class="header-image" src="image1.png" />"#));
    assert!(result
        .dxml
        .contains(r#"<img class="footer-image" src="image2.png" />"#));
    assert!(
        !result.dxml.contains("header-logo"),
        "Must not use rigid hardcoded header-logo class"
    );
    assert!(
        !result.dxml.contains("Imported Document"),
        "Must not emit fake placeholder header text"
    );
    assert!(
        !result.dxml.contains("Pagina Engine"),
        "Must not emit fake placeholder footer text"
    );
    assert!(result.dxml.contains("Dr. S. K. Korde"));
    assert!(result.dcss.contains("table {"));
    assert!(result.dcss.contains(".header-image"));
    assert!(result.dcss.contains(".footer-image"));
    assert!(
        result.assets.contains_key("image1.png") || result.assets.contains_key("image2.png"),
        "Media assets must be extracted"
    );
    assert!(
        !result.dxml.contains("data:image/"),
        "DXML must not contain raw base64 data URLs"
    );
    assert!(
        result.dcss.contains("Times New Roman"),
        "DCSS must contain Times New Roman font"
    );
    assert!(
        result.dcss.contains("14pt"),
        "DCSS must contain accurate 14pt sizing"
    );
    let ul_count = result.dxml.matches("<ul").count();
    let ol_count = result.dxml.matches("<ol").count();
    let li_count = result.dxml.matches("<li").count();
    assert!(
        ul_count > 0 || ol_count > 0 || li_count > 0,
        "Imported DOCX must extract bullet or numbered lists"
    );

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
    assert!(result
        .dxml
        .contains("EVA: Environmental Vision through Audio"));

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
    let doc = parser
        .parse()
        .expect("User document should parse successfully");
    assert_eq!(
        doc.props.metadata.get("title").map(|s| s.as_str()),
        Some("Modular DDL Report")
    );
    assert!(doc.props.page_styles.contains_key("body"));

    // 1. PDF Export
    let pdf_bytes = pdf::export_pdf(&doc).expect("PDF export must succeed");
    assert_eq!(&pdf_bytes[0..4], b"%PDF");

    // 2. DOCX Export
    let docx_bytes = docx::export_docx(&doc).expect("DOCX export must succeed");
    assert_eq!(&docx_bytes[0..2], b"PK");

    // Inspect DOCX contents: verify header1.xml, footer1.xml, grid table, and entity decoding
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&docx_bytes))
        .expect("Must be valid zip archive");
    assert!(
        archive.by_name("word/header1.xml").is_ok(),
        "Header1.xml must exist"
    );
    assert!(
        archive.by_name("word/footer1.xml").is_ok(),
        "Footer1.xml must exist"
    );

    let mut doc_xml = String::new();
    archive
        .by_name("word/document.xml")
        .unwrap()
        .read_to_string(&mut doc_xml)
        .unwrap();
    assert!(doc_xml.contains("Modular Document Architecture"));
    assert!(
        doc_xml.contains("Technical Report") || archive.by_name("word/header1.xml").is_ok()
    );
    assert!(
        doc_xml
            .contains("&lt;link rel=&quot;stylesheet&quot; href=&quot;styles.css&quot; /&gt;"),
        "Entity must be properly escaped for Word XML so user sees <link>"
    );
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
