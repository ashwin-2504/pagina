pub mod blocks;
pub mod render;
pub mod templates;

use crate::ImportResult;
use blocks::{is_heading_candidate, is_page_number_or_header, parse_list_item, ListPrefix, PdfBlock};
use render::render_pdf_block;
use std::collections::HashMap;
use templates::{default_pdf_dcss, wrap_sections_in_dxml};

pub fn import_pdf(bytes: &[u8]) -> Result<ImportResult, Box<dyn std::error::Error + Send + Sync>> {
    let full_text = match pdf_extract::extract_text_from_mem(bytes) {
        Ok(t) => t,
        Err(_) => {
            let doc = lopdf::Document::load_mem(bytes)?;
            let pages = doc.get_pages();
            let mut combined = String::new();
            for (&page_num, _) in &pages {
                if let Ok(t) = doc.extract_text(&[page_num]) {
                    combined.push_str(&t);
                    combined.push('\x0C');
                }
            }
            combined
        }
    };

    let raw_pages: Vec<&str> = if full_text.contains('\x0C') {
        full_text.split('\x0C').collect()
    } else {
        vec![&full_text]
    };

    let mut section_xmls: Vec<String> = Vec::new();

    for page_str in raw_pages {
        let raw_lines: Vec<&str> = page_str.lines().collect();
        if raw_lines.is_empty() {
            continue;
        }

        let mut blocks: Vec<PdfBlock> = Vec::new();
        let mut cur_para = String::new();
        let mut cur_address: Vec<String> = Vec::new();

        let mut i = 0;
        let n = raw_lines.len();

        while i < n {
            let raw_line = raw_lines[i];
            let line = raw_line.trim();
            i += 1;

            if line.is_empty() {
                if !cur_para.is_empty() {
                    blocks.push(PdfBlock::Paragraph(cur_para.clone()));
                    cur_para.clear();
                }
                if !cur_address.is_empty() {
                    blocks.push(PdfBlock::AddressBlock(cur_address.clone()));
                    cur_address.clear();
                }
                continue;
            }

            if is_page_number_or_header(line) {
                continue;
            }

            if let Some((level, heading_text)) = is_heading_candidate(line) {
                if !cur_para.is_empty() {
                    blocks.push(PdfBlock::Paragraph(cur_para.clone()));
                    cur_para.clear();
                }
                if !cur_address.is_empty() {
                    blocks.push(PdfBlock::AddressBlock(cur_address.clone()));
                    cur_address.clear();
                }
                blocks.push(PdfBlock::Heading(level, heading_text));
                continue;
            }

            if let Some(list_prefix) = parse_list_item(line) {
                if !cur_para.is_empty() {
                    blocks.push(PdfBlock::Paragraph(cur_para.clone()));
                    cur_para.clear();
                }
                if !cur_address.is_empty() {
                    blocks.push(PdfBlock::AddressBlock(cur_address.clone()));
                    cur_address.clear();
                }

                match list_prefix {
                    ListPrefix::Number(num_type, item_text) => {
                        if let Some(PdfBlock::NumberList(cur_type, items)) = blocks.last_mut() {
                            if cur_type == num_type {
                                items.push(item_text);
                            } else {
                                blocks.push(PdfBlock::NumberList(
                                    num_type.to_string(),
                                    vec![item_text],
                                ));
                            }
                        } else {
                            blocks
                                .push(PdfBlock::NumberList(num_type.to_string(), vec![item_text]));
                        }
                    }
                    ListPrefix::Bullet(bullet_type, item_text) => {
                        if let Some(PdfBlock::BulletList(cur_type, items)) = blocks.last_mut() {
                            if cur_type == bullet_type {
                                items.push(item_text);
                            } else {
                                blocks.push(PdfBlock::BulletList(
                                    bullet_type.to_string(),
                                    vec![item_text],
                                ));
                            }
                        } else {
                            blocks.push(PdfBlock::BulletList(
                                bullet_type.to_string(),
                                vec![item_text],
                            ));
                        }
                    }
                }
                continue;
            }

            // Address line (ends with comma or is To / From)
            let is_addr = (line.ends_with(',') && line.len() < 50)
                || (cur_address.is_empty()
                    && (line == "To" || line == "To," || line == "From" || line == "From,"));

            if is_addr {
                if !cur_para.is_empty() {
                    blocks.push(PdfBlock::Paragraph(cur_para.clone()));
                    cur_para.clear();
                }
                cur_address.push(line.to_string());
                continue;
            }

            if !cur_address.is_empty() {
                cur_address.push(line.to_string());
                blocks.push(PdfBlock::AddressBlock(cur_address.clone()));
                cur_address.clear();
                continue;
            }

            // Indented blockquote heuristic (line starts with 4+ spaces or tab)
            let is_indented = raw_line.starts_with("    ") || raw_line.starts_with('\t');
            if is_indented && cur_para.is_empty() && line.len() < 120 {
                blocks.push(PdfBlock::Blockquote(line.to_string()));
                continue;
            }

            // Accumulate into flowing paragraph
            if cur_para.is_empty() {
                cur_para.push_str(line);
            } else if cur_para.ends_with('-') && !cur_para.ends_with(" -") {
                cur_para.pop();
                cur_para.push_str(line);
            } else {
                cur_para.push(' ');
                cur_para.push_str(line);
            }
        }

        if !cur_para.is_empty() {
            blocks.push(PdfBlock::Paragraph(cur_para.clone()));
        }
        if !cur_address.is_empty() {
            blocks.push(PdfBlock::AddressBlock(cur_address.clone()));
        }

        // Render page blocks into section DXML
        let page_nodes: Vec<String> = blocks.into_iter().map(render_pdf_block).collect();

        if !page_nodes.is_empty() {
            section_xmls.push(format!(
                "    <section page-style=\"body\">\n{}\n    </section>",
                page_nodes.join("\n")
            ));
        }
    }

    if section_xmls.is_empty() {
        section_xmls.push(
            "    <section page-style=\"body\">\n      <p>Empty document</p>\n    </section>"
                .to_string(),
        );
    }

    let dxml = wrap_sections_in_dxml(&section_xmls);
    let dcss = default_pdf_dcss().to_string();

    Ok(ImportResult {
        dxml,
        dcss,
        assets: HashMap::new(),
    })
}
