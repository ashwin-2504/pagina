pub mod blocks;
pub mod dcss;
pub mod document_attrs;
pub mod document_p;
pub mod document_run;
pub mod document_table;
pub mod document_xml;
pub mod generic_part;
pub mod list;
pub mod numbering;
pub mod relationships;
pub mod styles;
pub mod types;

use std::collections::HashMap;
use std::io::{Cursor, Read};
use base64::Engine;
use zip::ZipArchive;

use crate::ImportResult;
use blocks::render_blocks_to_dxml;
use dcss::generate_imported_dcss;
use document_xml::parse_document_xml;
use generic_part::parse_generic_part;
use numbering::parse_numbering_xml;
use relationships::parse_relationships;
use styles::parse_document_styles;
use types::{NumberingInfo, ParsedGenericPart};

fn extract_part(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    doc_type_rels: &HashMap<String, String>,
    media_path_to_filename: &HashMap<String, String>,
    rel_type: &str,
    default_name: &str,
    is_footer: bool,
) -> ParsedGenericPart {
    let target = doc_type_rels
        .get(rel_type)
        .cloned()
        .unwrap_or_else(|| default_name.to_string());
    let part_name = if target.starts_with("word/") {
        target.clone()
    } else {
        format!("word/{}", target)
    };
    let rels_name = format!(
        "word/_rels/{}.rels",
        target.split('/').last().unwrap_or(&target)
    );

    let mut part_xml = String::new();
    if let Ok(mut f) = archive.by_name(&part_name) {
        let _ = f.read_to_string(&mut part_xml);
    }

    let mut rels_xml = String::new();
    if let Ok(mut f) = archive.by_name(&rels_name) {
        let _ = f.read_to_string(&mut rels_xml);
    }
    let (raw_rels, _) = parse_relationships(&rels_xml);
    let part_rels: HashMap<String, String> = raw_rels
        .into_iter()
        .filter_map(|(id, tgt)| {
            let direct = media_path_to_filename.get(&tgt);
            let fallback = media_path_to_filename.get(tgt.split('/').last().unwrap_or(&tgt));
            direct.or(fallback).map(|fname| (id, fname.clone()))
        })
        .collect();

    if !part_xml.is_empty() {
        parse_generic_part(&part_xml, &part_rels, is_footer)
    } else {
        ParsedGenericPart::default()
    }
}

pub fn import_docx(bytes: &[u8]) -> Result<ImportResult, Box<dyn std::error::Error + Send + Sync>> {
    let mut archive = ZipArchive::new(Cursor::new(bytes))?;

    let mut assets: HashMap<String, String> = HashMap::new();
    let mut media_path_to_filename: HashMap<String, String> = HashMap::new();
    for i in 0..archive.len() {
        if let Ok(mut file) = archive.by_index(i) {
            let name = file.name().to_string();
            let is_media = name.contains("media/")
                || name.ends_with(".png")
                || name.ends_with(".jpg")
                || name.ends_with(".jpeg")
                || name.ends_with(".gif")
                || name.ends_with(".bmp")
                || name.ends_with(".webp");
            if is_media {
                let mut img_bytes = Vec::new();
                if file.read_to_end(&mut img_bytes).is_ok() && !img_bytes.is_empty() {
                    let ext = name.split('.').last().unwrap_or("png").to_lowercase();
                    let mime = match ext.as_str() {
                        "png" => "image/png",
                        "jpg" | "jpeg" => "image/jpeg",
                        "gif" => "image/gif",
                        "bmp" => "image/bmp",
                        "webp" => "image/webp",
                        _ => "image/png",
                    };
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&img_bytes);
                    let data_url = format!("data:{};base64,{}", mime, b64);
                    let filename = name.split('/').last().unwrap_or(&name).to_string();
                    assets.insert(filename.clone(), data_url);
                    media_path_to_filename.insert(name.clone(), filename.clone());
                    media_path_to_filename.insert(filename.clone(), filename.clone());
                    media_path_to_filename.insert(format!("media/{}", filename), filename.clone());
                    media_path_to_filename.insert(format!("../media/{}", filename), filename.clone());
                    media_path_to_filename.insert(format!("word/media/{}", filename), filename.clone());
                }
            }
        }
    }

    let (doc_id_rels, doc_type_rels) = {
        let mut xml = String::new();
        if let Ok(mut f) = archive.by_name("word/_rels/document.xml.rels") {
            let _ = f.read_to_string(&mut xml);
        }
        parse_relationships(&xml)
    };

    let doc_img_rels: HashMap<String, String> = doc_id_rels
        .iter()
        .filter_map(|(id, target)| {
            let direct = media_path_to_filename.get(target);
            let fallback = media_path_to_filename.get(target.split('/').last().unwrap_or(target));
            direct.or(fallback).map(|fname| (id.clone(), fname.clone()))
        })
        .collect();

    let parsed_header = extract_part(
        &mut archive,
        &doc_type_rels,
        &media_path_to_filename,
        "header",
        "header1.xml",
        false,
    );

    let parsed_footer = extract_part(
        &mut archive,
        &doc_type_rels,
        &media_path_to_filename,
        "footer",
        "footer1.xml",
        true,
    );

    let numbering_info = {
        let mut num_xml = String::new();
        if let Ok(mut file) = archive.by_name("word/numbering.xml") {
            let _ = file.read_to_string(&mut num_xml);
        }
        if !num_xml.is_empty() {
            parse_numbering_xml(&num_xml)
        } else {
            NumberingInfo::default()
        }
    };

    let mut doc_xml = String::new();
    {
        let mut file = archive.by_name("word/document.xml")?;
        file.read_to_string(&mut doc_xml)?;
    }

    let document_styles = {
        let mut styles_xml = String::new();
        if let Ok(mut file) = archive.by_name("word/styles.xml") {
            let _ = file.read_to_string(&mut styles_xml);
        }
        parse_document_styles(&styles_xml)
    };

    let parsed_doc = parse_document_xml(&doc_xml, &doc_img_rels, &numbering_info)?;
    let body_nodes = render_blocks_to_dxml(parsed_doc.blocks);

    let header_xml = parsed_header.to_dxml("running-header");
    let footer_xml = parsed_footer.to_dxml("running-footer");

    let mut page_style_inner = String::new();
    if !header_xml.is_empty() {
        page_style_inner.push_str(&header_xml);
        page_style_inner.push('\n');
    }
    if !footer_xml.is_empty() {
        page_style_inner.push_str(&footer_xml);
        page_style_inner.push('\n');
    }

    let page_styles_block = if !page_style_inner.is_empty() {
        format!(
            "    <page-styles>\n      <page-style name=\"body\" default=\"true\">\n{}\n      </page-style>\n    </page-styles>",
            page_style_inner.trim_end()
        )
    } else {
        "    <page-styles>\n      <page-style name=\"body\" default=\"true\" />\n    </page-styles>".to_string()
    };

    let header_distance_attr = parsed_doc
        .header_distance_mm
        .map(|d| format!(" header-distance=\"{:.2}mm\"", d))
        .unwrap_or_default();
    let footer_distance_attr = parsed_doc
        .footer_distance_mm
        .map(|d| format!(" footer-distance=\"{:.2}mm\"", d))
        .unwrap_or_default();

    let dxml = format!(
        r#"<doc>
  <props>
    <meta>
      <title>Imported OpenXML Document</title>
      <author>Pagina Reverse-Engineering Engine</author>
    </meta>
    <paper size="{}" orientation="{}" margin="{}mm"{}{} />
    <link rel="stylesheet" href="styles.dcss" />
{}
  </props>

  <body>
    <section page-style="body">
{}
    </section>
  </body>
</doc>"#,
        parsed_doc.paper_size,
        parsed_doc.paper_orientation,
        parsed_doc.paper_margin_mm,
        header_distance_attr,
        footer_distance_attr,
        page_styles_block,
        if body_nodes.is_empty() {
            "      <p>Empty document</p>".to_string()
        } else {
            body_nodes.join("\n")
        }
    );

    let dcss = generate_imported_dcss(
        &document_styles,
        &parsed_doc.font_counts,
        &parsed_doc.font_size_counts,
        &parsed_doc.font_color_counts,
        &parsed_header,
        &parsed_footer,
    );

    Ok(ImportResult { dxml, dcss, assets })
}
