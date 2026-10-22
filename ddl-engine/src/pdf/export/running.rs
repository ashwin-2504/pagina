use crate::ast::{Document, PageStyle};
use super::drawing::{draw_image_on_layer, find_img_in_nodes, load_image_bytes};
use super::types::{extract_nodes_text, length_to_mm};
use printpdf::{Color, IndirectFontRef, Line, Mm, PdfLayerReference, Point, Rgb};

pub fn get_active_page_style(doc: &Document) -> Option<&PageStyle> {
    if let Some(ps) = doc.props.page_styles.get("body") {
        return Some(ps);
    }
    doc.props
        .page_styles
        .values()
        .find(|ps| ps.is_default)
        .or_else(|| doc.props.page_styles.values().next())
}

pub fn get_header_and_footer_heights(doc: &Document, _width_mm: f32, content_width: f32) -> (f32, f32) {
    let ps = get_active_page_style(doc);
    let mut header_h = 0.0f32;
    let mut footer_h = 0.0f32;

    if let Some(p) = ps {
        if let Some(h) = p.running_header.as_ref() {
            let img_opt = find_img_in_nodes(&h.center)
                .or_else(|| find_img_in_nodes(&h.left))
                .or_else(|| find_img_in_nodes(&h.right));
            if let Some(img_el) = img_opt {
                if let Some(src) = img_el.attributes.get("src") {
                    if let Some(bytes) = load_image_bytes(src) {
                        if let Ok(dyn_img) = ::image::load_from_memory(&bytes) {
                            let px_w = dyn_img.width() as f32;
                            let px_h = dyn_img.height() as f32;
                            let aspect = if px_w > 0.0 { px_h / px_w } else { 0.2 };
                            let max_target_h = 35.0_f32;
                            let target_w = content_width.min(max_target_h / aspect);
                            let target_h = target_w * aspect;
                            header_h = target_h + 8.0;
                        }
                    }
                }
            }
            if header_h == 0.0
                && (!h.left.is_empty() || !h.center.is_empty() || !h.right.is_empty())
            {
                header_h = 16.0;
            }
        }

        if let Some(f) = p.running_footer.as_ref() {
            let img_opt = find_img_in_nodes(&f.center)
                .or_else(|| find_img_in_nodes(&f.left))
                .or_else(|| find_img_in_nodes(&f.right));
            if let Some(img_el) = img_opt {
                if let Some(src) = img_el.attributes.get("src") {
                    if let Some(bytes) = load_image_bytes(src) {
                        if let Ok(dyn_img) = ::image::load_from_memory(&bytes) {
                            let px_w = dyn_img.width() as f32;
                            let px_h = dyn_img.height() as f32;
                            let aspect = if px_w > 0.0 { px_h / px_w } else { 0.2 };
                            let max_target_h = 35.0_f32;
                            let target_w = content_width.min(max_target_h / aspect);
                            let target_h = target_w * aspect;
                            footer_h = target_h + 8.0;
                        }
                    }
                }
            }
            if footer_h == 0.0
                && (!f.left.is_empty() || !f.center.is_empty() || !f.right.is_empty())
            {
                footer_h = 14.0;
            }
        }
    }

    (header_h, footer_h)
}

pub fn draw_running_header_footer(
    layer: &PdfLayerReference,
    page_num: usize,
    left_margin: f32,
    content_width: f32,
    _width_mm: f32,
    height_mm: f32,
    font_reg: &IndirectFontRef,
    doc: &Document,
) {
    let ps = get_active_page_style(doc);
    let margin_mm = length_to_mm(&doc.props.paper.margin);
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

    let header_img_opt = ps.and_then(|p| p.running_header.as_ref()).and_then(|h| {
        find_img_in_nodes(&h.center)
            .or_else(|| find_img_in_nodes(&h.left))
            .or_else(|| find_img_in_nodes(&h.right))
    });

    let mut drew_header_image = false;
    if let Some(img_el) = header_img_opt {
        if let Some(src) = img_el.attributes.get("src") {
            if let Some(bytes) = load_image_bytes(src) {
                if let Ok(dyn_img) = ::image::load_from_memory(&bytes) {
                    let px_w = dyn_img.width() as f32;
                    let px_h = dyn_img.height() as f32;
                    let aspect = if px_w > 0.0 { px_h / px_w } else { 0.2 };
                    let max_target_h = 35.0_f32;
                    let target_w = content_width.min(max_target_h / aspect);
                    let target_h = target_w * aspect;
                    let img_x = left_margin + (content_width - target_w) / 2.0;
                    let img_y = (height_mm - header_distance - target_h).max(0.0);
                    draw_image_on_layer(layer, &dyn_img, img_x, img_y, target_w, target_h);
                    drew_header_image = true;
                }
            }
        }
    }

    if !drew_header_image {
        if let Some(h) = ps.and_then(|p| p.running_header.as_ref()) {
            let left_text = extract_nodes_text(&h.left);
            let center_text = extract_nodes_text(&h.center);
            let right_text = extract_nodes_text(&h.right);
            let display_right = if !right_text.is_empty() {
                right_text.replace("<page-number/>", &format!("{}", page_num))
            } else if !left_text.is_empty() || !center_text.is_empty() {
                format!("{} of 1", page_num)
            } else {
                String::new()
            };

            if !left_text.is_empty() || !center_text.is_empty() || !display_right.is_empty() {
                layer.set_fill_color(Color::Rgb(Rgb::new(
                    100.0 / 255.0,
                    116.0 / 255.0,
                    139.0 / 255.0,
                    None,
                )));

                let header_y = height_mm - header_distance;
                if !left_text.is_empty() {
                    layer.use_text(&left_text, 8.5, Mm(left_margin), Mm(header_y), font_reg);
                }
                if !center_text.is_empty() {
                    let center_w = (center_text.len() as f32) * 8.5 * 0.19;
                    let center_x =
                        (left_margin + (content_width - center_w) / 2.0).max(left_margin);
                    layer.use_text(&center_text, 8.5, Mm(center_x), Mm(header_y), font_reg);
                }
                if !display_right.is_empty() {
                    let right_w = (display_right.len() as f32) * 8.5 * 0.19;
                    let right_x = (left_margin + content_width - right_w).max(left_margin);
                    layer.use_text(&display_right, 8.5, Mm(right_x), Mm(header_y), font_reg);
                }

                let rule_y = (header_y - 2.5).max(0.0);
                layer.set_outline_color(Color::Rgb(Rgb::new(
                    203.0 / 255.0,
                    213.0 / 255.0,
                    225.0 / 255.0,
                    None,
                )));
                layer.set_outline_thickness(0.5);
                let points = vec![
                    (Point::new(Mm(left_margin), Mm(rule_y)), false),
                    (
                        Point::new(Mm(left_margin + content_width), Mm(rule_y)),
                        false,
                    ),
                ];
                layer.add_line(Line {
                    points,
                    is_closed: false,
                });
            }
        }
    }

    let footer_img_opt = ps.and_then(|p| p.running_footer.as_ref()).and_then(|f| {
        find_img_in_nodes(&f.center)
            .or_else(|| find_img_in_nodes(&f.left))
            .or_else(|| find_img_in_nodes(&f.right))
    });

    let mut drew_footer_image = false;
    if let Some(img_el) = footer_img_opt {
        if let Some(src) = img_el.attributes.get("src") {
            if let Some(bytes) = load_image_bytes(src) {
                if let Ok(dyn_img) = ::image::load_from_memory(&bytes) {
                    let px_w = dyn_img.width() as f32;
                    let px_h = dyn_img.height() as f32;
                    let aspect = if px_w > 0.0 { px_h / px_w } else { 0.2 };
                    let max_target_h = 35.0_f32;
                    let target_w = content_width.min(max_target_h / aspect);
                    let target_h = target_w * aspect;
                    let img_x = left_margin + (content_width - target_w) / 2.0;
                    let img_y = footer_distance;
                    draw_image_on_layer(layer, &dyn_img, img_x, img_y, target_w, target_h);
                    drew_footer_image = true;
                }
            }
        }
    }

    if !drew_footer_image {
        if let Some(f) = ps.and_then(|p| p.running_footer.as_ref()) {
            let left_text = extract_nodes_text(&f.left);
            let center_text = extract_nodes_text(&f.center);
            let right_text = extract_nodes_text(&f.right);

            if !left_text.is_empty() || !center_text.is_empty() || !right_text.is_empty() {
                layer.set_fill_color(Color::Rgb(Rgb::new(
                    100.0 / 255.0,
                    116.0 / 255.0,
                    139.0 / 255.0,
                    None,
                )));

                let footer_y = footer_distance;
                if !left_text.is_empty() {
                    layer.use_text(&left_text, 8.5, Mm(left_margin), Mm(footer_y), font_reg);
                }
                if !center_text.is_empty() {
                    let footer_w = (center_text.len() as f32) * 8.5 * 0.19;
                    let footer_x =
                        (left_margin + (content_width - footer_w) / 2.0).max(left_margin);
                    layer.use_text(&center_text, 8.5, Mm(footer_x), Mm(footer_y), font_reg);
                }
                if !right_text.is_empty() {
                    let right_w = (right_text.len() as f32) * 8.5 * 0.19;
                    let right_x = (left_margin + content_width - right_w).max(left_margin);
                    layer.use_text(&right_text, 8.5, Mm(right_x), Mm(footer_y), font_reg);
                }
            }
        }
    }

    layer.set_fill_color(Color::Rgb(Rgb::new(
        15.0 / 255.0,
        23.0 / 255.0,
        42.0 / 255.0,
        None,
    )));
}
