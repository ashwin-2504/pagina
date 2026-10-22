pub mod blocks;
pub mod context;
pub mod drawing;
pub mod fonts;
pub mod grid;
pub mod image;
pub mod layout;
pub mod list;
pub mod nodes;
pub mod running;
pub mod table;
pub mod text;
pub mod types;

pub use fonts::{FontFamily, FontSet};

use crate::ast::Document;
use context::PdfRenderContext;
use fonts::FontSet as LocalFontSet;
use nodes::render_doc_nodes;
use running::{draw_running_header_footer, get_header_and_footer_heights};
use types::length_to_mm;

use printpdf::{BuiltinFont, Mm, PdfDocument};
use std::io::Cursor;

pub fn export_pdf(doc: &Document) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let title = doc
        .props
        .metadata
        .get("title")
        .map(|s| s.as_str())
        .unwrap_or("Pagina Document");

    let is_landscape = doc.props.paper.orientation.to_lowercase() == "landscape";
    let is_letter = doc.props.paper.size.to_lowercase() == "letter";
    let (mut width_mm, mut height_mm) = if is_letter {
        (215.9, 279.4)
    } else {
        (210.0, 297.0)
    };
    if is_landscape {
        std::mem::swap(&mut width_mm, &mut height_mm);
    }

    let (pdf_doc, first_page, first_layer) =
        PdfDocument::new(title, Mm(width_mm), Mm(height_mm), "Content");

    let font_regular = pdf_doc.add_builtin_font(BuiltinFont::Helvetica)?;
    let font_bold = pdf_doc.add_builtin_font(BuiltinFont::HelveticaBold)?;
    let font_italic = pdf_doc.add_builtin_font(BuiltinFont::HelveticaOblique)?;
    let font_code = pdf_doc.add_builtin_font(BuiltinFont::Courier)?;
    let font_times_reg = pdf_doc.add_builtin_font(BuiltinFont::TimesRoman)?;
    let font_times_bold = pdf_doc.add_builtin_font(BuiltinFont::TimesBold)?;
    let font_times_italic = pdf_doc.add_builtin_font(BuiltinFont::TimesItalic)?;

    let fonts = LocalFontSet {
        helvetica_reg: &font_regular,
        helvetica_bold: &font_bold,
        helvetica_italic: &font_italic,
        times_reg: &font_times_reg,
        times_bold: &font_times_bold,
        times_italic: &font_times_italic,
        courier: &font_code,
    };

    let mut current_page = first_page;
    let mut current_layer = pdf_doc.get_page(current_page).get_layer(first_layer);
    let mut page_count = 1;

    let margin_mm = length_to_mm(&doc.props.paper.margin);
    let left_margin = margin_mm.max(12.0);
    let right_margin = margin_mm.max(12.0);
    let content_width = width_mm - left_margin - right_margin;
    let (header_clearance, footer_clearance) =
        get_header_and_footer_heights(doc, width_mm, content_width);
    let default_running_distance = (margin_mm / 2.0).min(12.7).max(5.0);
    let header_distance = doc
        .props
        .paper
        .header_distance
        .as_ref()
        .map(length_to_mm)
        .unwrap_or(default_running_distance);
    let footer_distance = doc
        .props
        .paper
        .footer_distance
        .as_ref()
        .map(length_to_mm)
        .unwrap_or(default_running_distance);
    let top_margin = margin_mm.max(header_distance + header_clearance).max(18.0);
    let bottom_margin = margin_mm.max(footer_distance + footer_clearance).max(18.0);

    draw_running_header_footer(
        &current_layer,
        1,
        left_margin,
        content_width,
        width_mm,
        height_mm,
        fonts.helvetica_reg,
        doc,
    );

    let mut cursor_y = height_mm - top_margin;

    let mut ctx = PdfRenderContext {
        pdf_doc: &pdf_doc,
        current_page: &mut current_page,
        current_layer: &mut current_layer,
        page_count: &mut page_count,
        cursor_y: &mut cursor_y,
        left_margin,
        content_width,
        height_mm,
        top_margin,
        bottom_margin,
        width_mm,
        fonts: &fonts,
        doc,
    };

    render_doc_nodes(&doc.body.children, &mut ctx, 0);

    let mut buf = std::io::BufWriter::new(Cursor::new(Vec::new()));
    pdf_doc.save(&mut buf)?;
    Ok(buf.into_inner()?.into_inner())
}
