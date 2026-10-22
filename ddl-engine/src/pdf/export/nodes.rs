use super::blocks::render_element_block;
use super::context::PdfRenderContext;
use super::fonts::{detect_font_family, FontFamily};
use super::grid::render_grid;
use super::image::render_image;
use super::list::render_list_element;
use super::table::render_table;
use super::text::render_styled_paragraph;
use super::types::{get_align, StyledRun};
use crate::ast::Node;
use printpdf::{Color, Line, Mm, Point, Rgb};

pub fn render_doc_nodes(
    nodes: &[Node],
    ctx: &mut PdfRenderContext,
    indent_level: usize,
) {
    for node in nodes {
        match node {
            Node::Text(t) => {
                let trimmed = t.content.trim();
                if !trimmed.is_empty() {
                    let run = StyledRun {
                        text: trimmed.to_string(),
                        family: FontFamily::SansSerif,
                        bold: false,
                        italic: false,
                    };
                    render_styled_paragraph(&[run], 10.5, None, None, false, ctx);
                }
            }
            Node::Comment(_) => {}
            Node::Section(sec) => {
                render_doc_nodes(&sec.children, ctx, indent_level);
            }
            Node::Element(el) => {
                let tag = el.tag.to_lowercase();
                let align = get_align(el);
                let el_font_family = detect_font_family(el, ctx.doc);

                let handled = render_element_block(
                    el,
                    &tag,
                    align,
                    el_font_family,
                    ctx,
                );
                if handled {
                    continue;
                }

                match tag.as_str() {
                    "grid" => {
                        render_grid(el, ctx);
                    }
                    "ul" | "ol" => {
                        render_list_element(el, &tag, el_font_family, indent_level, ctx);
                    }
                    "img" => {
                        render_image(el, ctx);
                    }
                    "table" => {
                        render_table(el, ctx);
                    }
                    "hr" => {
                        *ctx.cursor_y -= 3.0;
                        ctx.current_layer.set_outline_color(Color::Rgb(Rgb::new(
                            203.0 / 255.0,
                            213.0 / 255.0,
                            225.0 / 255.0,
                            None,
                        )));
                        ctx.current_layer.set_outline_thickness(0.6);
                        let points = vec![
                            (Point::new(Mm(ctx.left_margin), Mm(*ctx.cursor_y)), false),
                            (
                                Point::new(Mm(ctx.left_margin + ctx.content_width), Mm(*ctx.cursor_y)),
                                false,
                            ),
                        ];
                        let line = Line {
                            points,
                            is_closed: false,
                        };
                        ctx.current_layer.add_line(line);
                        *ctx.cursor_y -= 4.0;
                    }
                    _ => {
                        render_doc_nodes(&el.children, ctx, indent_level);
                    }
                }
            }
        }
    }
}
