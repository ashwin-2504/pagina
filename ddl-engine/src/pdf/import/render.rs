use super::blocks::PdfBlock;

pub fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn render_pdf_block(block: PdfBlock) -> String {
    match block {
        PdfBlock::Heading(level, text) => {
            format!(
                "      <h{}>{}</h{}>",
                level,
                escape_xml(&text),
                level
            )
        }
        PdfBlock::Paragraph(text) => {
            format!("      <p>{}</p>", escape_xml(&text))
        }
        PdfBlock::Blockquote(text) => {
            format!(
                "      <blockquote>\n        <p>{}</p>\n      </blockquote>",
                escape_xml(&text)
            )
        }
        PdfBlock::AddressBlock(lines) => {
            let escaped_lines: Vec<String> = lines.iter().map(|l| escape_xml(l)).collect();
            format!(
                "      <p>{}</p>",
                escaped_lines.join("<br />\n         ")
            )
        }
        PdfBlock::BulletList(b_type, items) => {
            let type_attr = if b_type != "disc" {
                format!(" type=\"{}\"", b_type)
            } else {
                String::new()
            };
            let mut list_xml = format!("      <ul{}>\n", type_attr);
            for item in items {
                list_xml.push_str(&format!("        <li>{}</li>\n", escape_xml(&item)));
            }
            list_xml.push_str("      </ul>");
            list_xml
        }
        PdfBlock::NumberList(n_type, items) => {
            let type_attr = if n_type != "1" {
                format!(" type=\"{}\"", n_type)
            } else {
                String::new()
            };
            let mut list_xml = format!("      <ol{}>\n", type_attr);
            for item in items {
                list_xml.push_str(&format!("        <li>{}</li>\n", escape_xml(&item)));
            }
            list_xml.push_str("      </ol>");
            list_xml
        }
    }
}
