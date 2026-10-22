pub mod css;
pub mod document;
pub mod images;
pub mod list;
pub mod parts;
pub mod running;
pub mod styles;
pub mod table;
pub mod utils;

use crate::ast::Document;
use document::generate_document_xml;
use images::collect_images_from_doc;
use parts::{CONTENT_TYPES_XML, RELS_XML};
use running::{generate_footer_xml, generate_header_xml};
use styles::generate_styles_xml;

use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

pub fn export_docx(doc: &Document) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let cursor = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(cursor);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let images = collect_images_from_doc(doc);

    // 1. [Content_Types].xml
    zip.start_file("[Content_Types].xml", options)?;
    zip.write_all(CONTENT_TYPES_XML.as_bytes())?;

    // 2. _rels/.rels
    zip.start_file("_rels/.rels", options)?;
    zip.write_all(RELS_XML.as_bytes())?;

    // 3. word/_rels/document.xml.rels
    let mut word_rels = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
  <Relationship Id="rIdH1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/header" Target="header1.xml"/>
  <Relationship Id="rIdF1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer" Target="footer1.xml"/>
"#,
    );
    for entry in images.values() {
        word_rels.push_str(&format!(
            "  <Relationship Id=\"{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/image\" Target=\"media/{}\"/>\n",
            entry.r_id, entry.filename
        ));
    }
    word_rels.push_str("</Relationships>");

    zip.start_file("word/_rels/document.xml.rels", options)?;
    zip.write_all(word_rels.as_bytes())?;

    // 4. Write image binary files to word/media/
    for entry in images.values() {
        zip.start_file(format!("word/media/{}", entry.filename), options)?;
        zip.write_all(&entry.bytes)?;
    }

    // 5. word/styles.xml
    zip.start_file("word/styles.xml", options)?;
    let styles_xml = generate_styles_xml(doc);
    zip.write_all(styles_xml.as_bytes())?;

    // 6. word/header1.xml
    zip.start_file("word/header1.xml", options)?;
    let header_xml = generate_header_xml(doc);
    zip.write_all(header_xml.as_bytes())?;

    // 7. word/footer1.xml
    zip.start_file("word/footer1.xml", options)?;
    let footer_xml = generate_footer_xml(doc);
    zip.write_all(footer_xml.as_bytes())?;

    // 8. word/document.xml
    zip.start_file("word/document.xml", options)?;
    let doc_xml = generate_document_xml(doc, &images);
    zip.write_all(doc_xml.as_bytes())?;

    let finished_cursor = zip.finish()?;
    Ok(finished_cursor.into_inner())
}
