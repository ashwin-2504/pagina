use std::collections::HashMap;
use quick_xml::events::Event;
use quick_xml::Reader;

use super::document_attrs::*;
use super::document_p::resolve_paragraph_block;
use super::document_run::{handle_run_element, DocumentRunState};
use super::document_table::TableParserState;
use super::styles::{format_coalesced_inlines, paragraph_style};
use super::types::{ImportedBlock, InlineToken, NumberingInfo, ParsedDocumentXml};

pub fn parse_document_xml(
    doc_xml: &str,
    doc_img_rels: &HashMap<String, String>,
    numbering_info: &NumberingInfo,
) -> Result<ParsedDocumentXml, Box<dyn std::error::Error + Send + Sync>> {
    let mut reader = Reader::from_str(doc_xml);
    reader.config_mut().trim_text(false);

    let mut blocks: Vec<ImportedBlock> = Vec::new();
    let mut current_p_style: Option<String> = None;
    let mut current_p_align: Option<String> = None;
    let mut current_p_is_callout = false;
    let mut current_p_num_id: Option<String> = None;
    let mut current_p_ilvl: u32 = 0;
    let mut current_p_tokens: Vec<InlineToken> = Vec::new();

    let mut font_counts: HashMap<String, usize> = HashMap::new();
    let mut font_size_counts: HashMap<u32, usize> = HashMap::new();
    let mut font_color_counts: HashMap<String, usize> = HashMap::new();

    let mut run = DocumentRunState::default();
    let mut tbl = TableParserState::default();
    let mut in_p_pr = false;

    let mut current_p_before_twips: Option<u32> = None;
    let mut current_p_after_twips: Option<u32> = None;
    let mut current_p_line_twips: Option<u32> = None;
    let mut current_p_left_twips: Option<u32> = None;
    let mut current_p_right_twips: Option<u32> = None;
    let mut current_p_first_line_twips: Option<i32> = None;

    let mut paper_size = "A4".to_string();
    let mut paper_orientation = "portrait".to_string();
    let mut paper_margin_mm = 20.0f32;
    let mut header_distance_mm: Option<f32> = None;
    let mut footer_distance_mm: Option<f32> = None;

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) | Event::Empty(ref e) => {
                let name = e.name();
                match name.as_ref() {
                    b"w:tbl" => tbl.start_table(),
                    b"w:tr" => tbl.start_row(),
                    b"w:tc" => tbl.start_cell(),
                    b"w:p" => {
                        current_p_style = None;
                        current_p_align = None;
                        current_p_is_callout = false;
                        current_p_num_id = None;
                        current_p_ilvl = 0;
                        current_p_tokens.clear();
                        current_p_before_twips = None;
                        current_p_after_twips = None;
                        current_p_line_twips = None;
                        current_p_left_twips = None;
                        current_p_right_twips = None;
                        current_p_first_line_twips = None;
                    }
                    b"w:tblInd" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"w:w" {
                                if let Ok(val) = std::str::from_utf8(&attr.value) {
                                    tbl.indent = val.parse::<u32>().unwrap_or(0);
                                }
                            }
                        }
                    }
                    b"w:top" | b"w:left" | b"w:bottom" | b"w:right" | b"w:insideH"
                    | b"w:insideV" => {
                        tbl.check_border(attr_val(e).as_deref());
                        if in_p_pr && name.as_ref() == b"w:left" {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"w:color" {
                                    if let Ok(val) = std::str::from_utf8(&attr.value) {
                                        let col = val.to_lowercase();
                                        if col.starts_with("16a") || col.starts_with("22c") {
                                            current_p_is_callout = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    b"a:blip" | b"v:imagedata" => {
                        for attr in e.attributes().flatten() {
                            let k = attr.key.as_ref();
                            if k == b"r:embed" || k == b"r:id" {
                                if let Ok(rid) = std::str::from_utf8(&attr.value) {
                                    if let Some(url) = doc_img_rels.get(rid) {
                                        blocks.push(ImportedBlock::Image(url.clone()));
                                    }
                                }
                            }
                        }
                    }
                    b"w:pStyle" => current_p_style = attr_val(e),
                    b"w:jc" => current_p_align = attr_val(e),
                    b"w:numId" => current_p_num_id = attr_val(e),
                    b"w:ilvl" => {
                        current_p_ilvl =
                            attr_val(e).and_then(|v| v.parse::<u32>().ok()).unwrap_or(0)
                    }
                    b"w:pPr" => in_p_pr = true,
                    b"w:r" => run.reset_run(),
                    b"w:t" => run.in_t = true,
                    b"w:spacing" if in_p_pr => parse_spacing(
                        e,
                        &mut current_p_before_twips,
                        &mut current_p_after_twips,
                        &mut current_p_line_twips,
                    ),
                    b"w:ind" if in_p_pr => parse_indent(
                        e,
                        &mut current_p_left_twips,
                        &mut current_p_right_twips,
                        &mut current_p_first_line_twips,
                    ),
                    b"w:br" => current_p_tokens.push(InlineToken {
                        text: "<br />".to_string(),
                        bold: false,
                        italic: false,
                        underline: false,
                        code: false,
                        sup: false,
                        sub: false,
                        highlight: false,
                        font_family: None,
                        font_size_half_points: None,
                        color: None,
                        highlight_color: None,
                    }),
                    b"w:pgSz" => parse_pg_sz(e, &mut paper_orientation, &mut paper_size),
                    b"w:pgMar" => parse_pg_mar(
                        e,
                        &mut paper_margin_mm,
                        &mut header_distance_mm,
                        &mut footer_distance_mm,
                    ),
                    _ => {
                        if in_p_pr && name.as_ref() == b"w:shd" {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"w:fill" {
                                    if let Ok(val) = std::str::from_utf8(&attr.value) {
                                        let fill = val.to_lowercase();
                                        if fill.starts_with("f0fd") || fill.starts_with("dcfc") {
                                            current_p_is_callout = true;
                                        }
                                    }
                                }
                            }
                        }
                        handle_run_element(
                            name.as_ref(),
                            e,
                            &mut run,
                            &mut font_counts,
                            &mut font_size_counts,
                            &mut font_color_counts,
                        );
                    }
                }
            }
            Event::Text(ref e) => {
                if run.in_t {
                    let text = e.unescape()?.to_string();
                    if !text.is_empty() {
                        current_p_tokens.push(run.make_token(text));
                    }
                }
            }
            Event::End(ref e) => match e.name().as_ref() {
                b"w:t" => run.in_t = false,
                b"w:r" => run.in_r = false,
                b"w:pPr" => in_p_pr = false,
                b"w:p" => {
                    let formatted_line = format_coalesced_inlines(&current_p_tokens);
                    let trimmed = formatted_line.trim();

                    if !trimmed.is_empty() {
                        let direct_style = paragraph_style(
                            current_p_before_twips,
                            current_p_after_twips,
                            current_p_line_twips,
                            current_p_left_twips,
                            current_p_right_twips,
                            current_p_first_line_twips,
                        );

                        if tbl.in_tbl {
                            let align_attr = current_p_align
                                .as_ref()
                                .map(|a| format!(" align=\"{}\"", a))
                                .unwrap_or_default();
                            let style_attr = if direct_style.is_empty() {
                                String::new()
                            } else {
                                format!(" style=\"{}\"", direct_style)
                            };
                            tbl.push_cell_paragraph(format!(
                                "<p{}{}>{}</p>",
                                align_attr, style_attr, formatted_line
                            ));
                        } else {
                            let block = resolve_paragraph_block(
                                current_p_style.as_deref(),
                                current_p_align.clone(),
                                direct_style,
                                current_p_num_id.as_deref(),
                                current_p_ilvl,
                                current_p_is_callout,
                                &current_p_tokens,
                                numbering_info,
                                &formatted_line,
                                trimmed,
                            );

                            if let ImportedBlock::List(mut new_items) = block {
                                if let Some(ImportedBlock::List(items)) = blocks.last_mut() {
                                    items.append(&mut new_items);
                                } else {
                                    blocks.push(ImportedBlock::List(new_items));
                                }
                            } else {
                                blocks.push(block);
                            }
                        }
                    }
                }
                b"w:tc" => tbl.end_cell(),
                b"w:tr" => tbl.end_row(),
                b"w:tbl" => {
                    if let Some(tbl_block) = tbl.end_table() {
                        blocks.push(tbl_block);
                    }
                }
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(ParsedDocumentXml {
        blocks,
        paper_size,
        paper_orientation,
        paper_margin_mm,
        header_distance_mm,
        footer_distance_mm,
        font_counts,
        font_size_counts,
        font_color_counts,
    })
}
