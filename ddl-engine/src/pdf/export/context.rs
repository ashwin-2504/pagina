use super::fonts::FontSet;
use crate::ast::Document;
use printpdf::{Mm, PdfDocumentReference, PdfLayerReference, PdfPageIndex};

pub struct PdfRenderContext<'a> {
    pub pdf_doc: &'a PdfDocumentReference,
    pub current_page: &'a mut PdfPageIndex,
    pub current_layer: &'a mut PdfLayerReference,
    pub page_count: &'a mut usize,
    pub cursor_y: &'a mut f32,
    pub left_margin: f32,
    pub content_width: f32,
    pub height_mm: f32,
    pub top_margin: f32,
    pub bottom_margin: f32,
    pub width_mm: f32,
    pub fonts: &'a FontSet<'a>,
    pub doc: &'a Document,
}

impl<'a> PdfRenderContext<'a> {
    pub fn check_page_overflow(&mut self, required_height: f32) {
        if *self.cursor_y - required_height < self.bottom_margin {
            *self.page_count += 1;
            let (new_page, new_layer) =
                self.pdf_doc.add_page(Mm(self.width_mm), Mm(self.height_mm), "Content");
            *self.current_page = new_page;
            *self.current_layer = self.pdf_doc.get_page(new_page).get_layer(new_layer);
            super::running::draw_running_header_footer(
                self.current_layer,
                *self.page_count,
                self.left_margin,
                self.content_width,
                self.width_mm,
                self.height_mm,
                self.fonts.helvetica_reg,
                self.doc,
            );
            *self.cursor_y = self.height_mm - self.top_margin;
        }
    }
}
