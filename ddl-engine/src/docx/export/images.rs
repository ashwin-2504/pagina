use crate::ast::{Document, ElementNode, Node};
use base64::Engine;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ImageEntry {
    pub r_id: String,
    pub filename: String,
    pub bytes: Vec<u8>,
    pub width_px: u32,
    pub height_px: u32,
}

pub fn collect_images_from_doc(doc: &Document) -> HashMap<String, ImageEntry> {
    let mut map = HashMap::new();
    let mut counter = 1;

    fn walk_nodes(nodes: &[Node], counter: &mut usize, map: &mut HashMap<String, ImageEntry>) {
        for n in nodes {
            match n {
                Node::Element(el) => {
                    if el.tag.to_lowercase() == "img" {
                        if let Some(src) = el.attributes.get("src") {
                            if !map.contains_key(src) {
                                let (bytes, ext) = if src.starts_with("data:image/") {
                                    if let Some(pos) = src.find(',') {
                                        let meta = &src[..pos];
                                        let ext = if meta.contains("jpeg") || meta.contains("jpg") {
                                            "jpeg"
                                        } else if meta.contains("gif") {
                                            "gif"
                                        } else if meta.contains("webp") {
                                            "webp"
                                        } else {
                                            "png"
                                        };
                                        if let Ok(b) = base64::engine::general_purpose::STANDARD
                                            .decode(src[pos + 1..].trim())
                                        {
                                            (b, ext)
                                        } else {
                                            (Vec::new(), ext)
                                        }
                                    } else {
                                        (Vec::new(), "png")
                                    }
                                } else if let Ok(b) = std::fs::read(src) {
                                    (b, "png")
                                } else {
                                    let mut found = None;
                                    for prefix in
                                        &["assets/", "media/", "../", "../media/", "word/media/"]
                                    {
                                        let candidate = format!("{}{}", prefix, src);
                                        if let Ok(b) = std::fs::read(&candidate) {
                                            found = Some((b, "png"));
                                            break;
                                        }
                                    }
                                    found.unwrap_or_else(|| (Vec::new(), "png"))
                                };

                                if !bytes.is_empty() {
                                    let (w, h) =
                                        if let Ok(dyn_img) = ::image::load_from_memory(&bytes) {
                                            (dyn_img.width(), dyn_img.height())
                                        } else {
                                            (800, 600)
                                        };

                                    let entry = ImageEntry {
                                        r_id: format!("rIdImg_{}", counter),
                                        filename: format!("image_{}.{}", counter, ext),
                                        bytes,
                                        width_px: w,
                                        height_px: h,
                                    };
                                    map.insert(src.clone(), entry);
                                    *counter += 1;
                                }
                            }
                        }
                    }
                    walk_nodes(&el.children, counter, map);
                }
                Node::Section(sec) => {
                    walk_nodes(&sec.children, counter, map);
                }
                _ => {}
            }
        }
    }

    walk_nodes(&doc.body.children, &mut counter, &mut map);
    map
}

pub fn emit_image(el: &ElementNode, images: &HashMap<String, ImageEntry>, out: &mut String) {
    let src = match el.attributes.get("src") {
        Some(s) if !s.is_empty() => s,
        _ => return,
    };

    let entry = match images.get(src) {
        Some(e) => e,
        None => return,
    };

    let align = el
        .attributes
        .get("align")
        .map(|s| s.as_str())
        .unwrap_or("center");
    let jc = match align {
        "left" => "left",
        "right" => "right",
        _ => "center",
    };

    let max_cx = 5400000u64;
    let cx = if entry.width_px > 0 {
        let desired = (entry.width_px as u64) * 9525;
        desired.min(max_cx)
    } else {
        max_cx
    };

    let cy = if entry.width_px > 0 && entry.height_px > 0 {
        cx * (entry.height_px as u64) / (entry.width_px as u64)
    } else {
        cx * 3 / 4
    };

    let id = entry
        .r_id
        .trim_start_matches("rIdImg_")
        .parse::<u32>()
        .unwrap_or(1);

    out.push_str(&format!(
        r#"<w:p>
  <w:pPr><w:jc w:val="{jc}"/></w:pPr>
  <w:r>
    <w:drawing>
      <wp:inline distT="0" distB="0" distL="0" distR="0">
        <wp:extent cx="{cx}" cy="{cy}"/>
        <wp:effectExtent l="0" t="0" r="0" b="0"/>
        <wp:docPr id="{id}" name="Picture {id}"/>
        <wp:cNvGraphicFramePr>
          <a:graphicFrameLocks xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" noChangeAspect="1"/>
        </wp:cNvGraphicFramePr>
        <a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
          <a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture">
            <pic:pic xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture">
              <pic:nvPicPr>
                <pic:cNvPr id="{id}" name="image_{id}"/>
                <pic:cNvPicPr/>
                <pic:nvPr/>
              </pic:nvPicPr>
              <pic:blipFill>
                <a:blip xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" r:embed="{r_id}"/>
                <a:stretch><a:fillRect/></a:stretch>
              </pic:blipFill>
              <pic:spPr>
                <a:xfrm><a:off x="0" y="0"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm>
                <a:prstGeom prst="rect"><a:avLst/></a:prstGeom>
              </pic:spPr>
            </pic:pic>
          </a:graphicData>
        </a:graphic>
      </wp:inline>
    </w:drawing>
  </w:r>
</w:p>
"#,
        jc = jc,
        cx = cx,
        cy = cy,
        id = id,
        r_id = entry.r_id
    ));
}
