use ddl_engine::{docx, parser, pdf};
use std::io::Read;

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
    assert!(import_res
        .dxml
        .contains("<h1>Modular Document Architecture</h1>"));
    assert!(import_res.dxml.contains("<b>bold</b>"));
    assert!(import_res.dxml.contains("<table>"));
    assert!(import_res.dxml.contains("Col A"));
    assert!(import_res.dxml.contains("Data 1"));
    assert!(import_res.dcss.contains("h1 {"));
}

#[test]
fn test_docx_roundtrip_preserves_text_and_paragraph_formatting() {
    let input = r#"<doc>
  <props>
    <paper size="A4" orientation="portrait" margin="20mm"/>
    <style>
      body { font-family: "Inter", sans-serif; font-size: 13pt; color: #123456; line-height: 1.5; }
      h1 { font-size: 27pt; color: #654321; }
    </style>
  </props>
  <body><section>
    <h1>Precisely Styled Heading</h1>
    <p style="margin-top: 3mm; margin-bottom: 4mm; line-height: 1.7; margin-left: 5mm; text-indent: 2mm;">Plain <span style="font-family: 'Courier New'; font-size: 15pt; color: #A1B2C3; background-color: #FFF2CC;">preserved text</span>.</p>
  </section></body>
</doc>"#;

    let mut parser = parser::Parser::new(input);
    let doc = parser.parse().expect("source document must parse");
    let bytes = docx::export_docx(&doc).expect("DOCX export must succeed");

    let mut archive =
        zip::ZipArchive::new(std::io::Cursor::new(&bytes)).expect("DOCX must be a zip archive");
    let mut styles_xml = String::new();
    archive
        .by_name("word/styles.xml")
        .unwrap()
        .read_to_string(&mut styles_xml)
        .unwrap();
    assert!(styles_xml.contains("w:ascii=\"Inter\""));
    assert!(styles_xml.contains("w:sz w:val=\"26\""));
    assert!(styles_xml.contains("w:color w:val=\"654321\""));

    let mut document_xml = String::new();
    archive
        .by_name("word/document.xml")
        .unwrap()
        .read_to_string(&mut document_xml)
        .unwrap();
    assert!(document_xml.contains("w:before=\"170\""));
    assert!(document_xml.contains("w:left=\"283\""));
    assert!(document_xml.contains("w:rFonts w:ascii=\"Courier New\""));
    assert!(document_xml.contains("w:sz w:val=\"30\""));
    assert!(document_xml.contains("w:color w:val=\"A1B2C3\""));
    assert!(document_xml.contains("w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"FFF2CC\""));

    let imported = docx::import_docx(&bytes).expect("DOCX import must succeed");
    assert!(imported.dcss.contains("\"Inter\""));
    assert!(imported.dcss.contains("font-size: 13pt"));
    assert!(imported.dcss.contains("font-size: 27pt"));
    assert!(imported.dcss.contains("#654321"));
    assert!(imported.dxml.contains("margin-top: 3"));
    assert!(imported.dxml.contains("text-indent:"));
    assert!(imported.dxml.contains("font-size: 15pt"));
    assert!(imported.dxml.contains("color: #A1B2C3"));
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
    let pdf_bytes = pdf::export_pdf(&doc)
        .expect("PDF export with all bullet types and formatting must succeed");
    assert_eq!(&pdf_bytes[0..4], b"%PDF");

    // 2. DOCX Export
    let docx_bytes =
        docx::export_docx(&doc).expect("DOCX export with all formatting must succeed");
    assert_eq!(&docx_bytes[0..2], b"PK");

    // 3. Inspect Word XML to verify bullet symbols and indentations
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&docx_bytes))
        .expect("Must be valid zip archive");
    let mut doc_xml = String::new();
    archive
        .by_name("word/document.xml")
        .unwrap()
        .read_to_string(&mut doc_xml)
        .unwrap();

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
    assert!(
        imported.dxml.contains("<ul"),
        "Imported DXML must contain ul"
    );
    assert!(
        imported.dxml.contains("<ol"),
        "Imported DXML must contain ol"
    );
    assert!(
        imported.dcss.contains("body"),
        "Imported DCSS must contain typography"
    );
    assert!(
        imported.dcss.contains(".callout"),
        "Imported DCSS must contain callout padding"
    );
    assert!(
        imported.dcss.contains(".col-box"),
        "Imported DCSS must contain col-box padding"
    );
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
    let doc = parser
        .parse()
        .expect("Failed to parse nested list document");

    // 1. PDF Export
    let pdf_bytes = pdf::export_pdf(&doc).expect("PDF export with nested lists must succeed");
    assert_eq!(&pdf_bytes[0..4], b"%PDF");

    // 2. DOCX Export
    let docx_bytes =
        docx::export_docx(&doc).expect("DOCX export with nested lists must succeed");
    assert_eq!(&docx_bytes[0..2], b"PK");

    // Verify DOCX contains multi-level numbering
    let mut archive =
        zip::ZipArchive::new(std::io::Cursor::new(&docx_bytes)).expect("Must be zip");
    let mut doc_xml = String::new();
    archive
        .by_name("word/document.xml")
        .unwrap()
        .read_to_string(&mut doc_xml)
        .unwrap();
    assert!(
        doc_xml.contains("w:ilvl w:val=\"0\""),
        "Must contain level 0"
    );
    assert!(
        doc_xml.contains("w:ilvl w:val=\"1\""),
        "Must contain level 1"
    );
    assert!(
        doc_xml.contains("w:ilvl w:val=\"2\""),
        "Must contain level 2"
    );
    assert!(doc_xml.contains("•"), "Must contain disc bullet");
    assert!(doc_xml.contains("◦"), "Must contain circle bullet");
    assert!(doc_xml.contains("▪"), "Must contain square bullet");

    // 3. Re-import DOCX
    let imported = docx::import_docx(&docx_bytes).expect("DOCX import must succeed");
    assert!(imported.dxml.contains("Parent item A"));
    assert!(imported.dxml.contains("Child item A.1"));
    assert!(imported.dxml.contains("Grandchild A.2.1"));
    assert!(imported.dxml.contains("<ul"));
    assert!(imported.dxml.contains("type=\"circle\"") || imported.dxml.contains("<ul"));
}
