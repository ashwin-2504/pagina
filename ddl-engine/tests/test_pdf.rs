use ddl_engine::{docx, parser, pdf};

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
    assert!(import_res
        .dxml
        .contains("Pagina is a native document typesetting engine."));
    assert!(import_res.dcss.contains("h1 {"));
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

    let mut archive =
        zip::ZipArchive::new(std::io::Cursor::new(docx_bytes)).expect("Must be valid zip");
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
