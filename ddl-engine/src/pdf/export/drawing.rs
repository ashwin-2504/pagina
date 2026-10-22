use crate::ast::{ElementNode, Node};
use base64::Engine;
use printpdf::path::{PaintMode, WindingOrder};
use printpdf::{Image, ImageTransform, Mm, PdfLayerReference, Point, Polygon, Rect};

pub fn find_img_in_nodes<'a>(nodes: &'a [Node]) -> Option<&'a ElementNode> {
    for n in nodes {
        match n {
            Node::Element(el) => {
                if el.tag.to_lowercase() == "img" {
                    return Some(el);
                }
                if let Some(sub) = find_img_in_nodes(&el.children) {
                    return Some(sub);
                }
            }
            _ => {}
        }
    }
    None
}

pub fn load_image_bytes(src: &str) -> Option<Vec<u8>> {
    if src.starts_with("data:image/") {
        if let Some(pos) = src.find(',') {
            let b64 = &src[pos + 1..];
            return base64::engine::general_purpose::STANDARD
                .decode(b64.trim())
                .ok();
        }
    }
    if let Ok(bytes) = std::fs::read(src) {
        return Some(bytes);
    }
    for prefix in &["assets/", "media/", "../", "../media/", "word/media/"] {
        let candidate = format!("{}{}", prefix, src);
        if let Ok(bytes) = std::fs::read(&candidate) {
            return Some(bytes);
        }
    }
    if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(src.trim()) {
        if !bytes.is_empty() {
            return Some(bytes);
        }
    }
    None
}

pub fn draw_image_on_layer(
    layer: &PdfLayerReference,
    dynamic_img: &::image::DynamicImage,
    x_mm: f32,
    y_bottom_mm: f32,
    target_w_mm: f32,
    target_h_mm: f32,
) {
    let px_w = dynamic_img.width() as f32;
    let px_h = dynamic_img.height() as f32;
    if px_w <= 0.0 || px_h <= 0.0 {
        return;
    }

    let printpdf_img = Image::from_dynamic_image(dynamic_img);

    let native_w_mm = px_w * 25.4 / 300.0;
    let native_h_mm = px_h * 25.4 / 300.0;
    let scale_x = target_w_mm / native_w_mm;
    let scale_y = target_h_mm / native_h_mm;

    let transform = ImageTransform {
        translate_x: Some(Mm(x_mm)),
        translate_y: Some(Mm(y_bottom_mm)),
        rotate: None,
        scale_x: Some(scale_x),
        scale_y: Some(scale_y),
        dpi: Some(300.0),
    };

    printpdf_img.add_to_layer(layer.clone(), transform);
}

pub fn draw_rounded_rect(
    layer: &PdfLayerReference,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    mode: PaintMode,
) {
    let r = radius.min(w / 2.0).min(h / 2.0);
    if r <= 0.05 {
        let mut rect = Rect::new(Mm(x), Mm(y), Mm(x + w), Mm(y + h));
        rect.mode = mode;
        layer.add_rect(rect);
        return;
    }

    let k = 0.55228475 * r;
    let ring: Vec<(Point, bool)> = vec![
        (Point::new(Mm(x + r), Mm(y)), false),
        (Point::new(Mm(x + w - r), Mm(y)), false),
        (Point::new(Mm(x + w - r + k), Mm(y)), true),
        (Point::new(Mm(x + w), Mm(y + r - k)), true),
        (Point::new(Mm(x + w), Mm(y + r)), false),
        (Point::new(Mm(x + w), Mm(y + h - r)), false),
        (Point::new(Mm(x + w), Mm(y + h - r + k)), true),
        (Point::new(Mm(x + w - r + k), Mm(y + h)), true),
        (Point::new(Mm(x + w - r), Mm(y + h)), false),
        (Point::new(Mm(x + r), Mm(y + h)), false),
        (Point::new(Mm(x + r - k), Mm(y + h)), true),
        (Point::new(Mm(x), Mm(y + h - r + k)), true),
        (Point::new(Mm(x), Mm(y + h - r)), false),
        (Point::new(Mm(x), Mm(y + r)), false),
        (Point::new(Mm(x), Mm(y + r - k)), true),
        (Point::new(Mm(x + r - k), Mm(y)), true),
        (Point::new(Mm(x + r), Mm(y)), false),
    ];

    let poly = Polygon {
        rings: vec![ring],
        mode,
        winding_order: WindingOrder::NonZero,
    };
    layer.add_polygon(poly);
}
