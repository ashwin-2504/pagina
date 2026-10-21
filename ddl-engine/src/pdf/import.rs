use crate::ImportResult;

#[derive(Debug)]
enum ListPrefix {
    Bullet(&'static str, String),
    Number(&'static str, String),
}

#[derive(Debug)]
enum PdfBlock {
    Heading(u8, String),
    Paragraph(String),
    Blockquote(String),
    BulletList(String, Vec<String>),
    NumberList(String, Vec<String>),
    AddressBlock(Vec<String>),
}

fn is_page_number_or_header(line: &str) -> bool {
    let t = line.trim();
    if t.is_empty() {
        return true;
    }
    if t.chars().all(|c| c.is_ascii_digit()) && t.len() <= 4 {
        return true;
    }
    if (t.contains(" of ") || t.contains(" OF ")) && t.len() <= 15 {
        return true;
    }
    false
}

fn parse_list_item(line: &str) -> Option<ListPrefix> {
    let trimmed = line.trim();
    if trimmed.ends_with(':') {
        return None;
    }

    // Numbered list: "1. ", "2. ", "1) ", "(1) "
    if let Some(first) = trimmed.chars().next() {
        if first.is_ascii_digit() {
            let rest = trimmed.trim_start_matches(|c: char| c.is_ascii_digit());
            if let Some(content) = rest.strip_prefix(". ").or_else(|| rest.strip_prefix(") ")) {
                if !content.trim().is_empty() {
                    return Some(ListPrefix::Number("1", content.trim().to_string()));
                }
            }
        }
    }

    // Lowercase alphabetic: "a. ", "b. ", "a) "
    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() >= 3
        && chars[0].is_ascii_lowercase()
        && (chars[1] == '.' || chars[1] == ')')
        && chars[2].is_whitespace()
    {
        return Some(ListPrefix::Number("a", trimmed[3..].trim().to_string()));
    }

    // Uppercase alphabetic: "A. ", "B. ", "A) "
    if chars.len() >= 3
        && chars[0].is_ascii_uppercase()
        && (chars[1] == '.' || chars[1] == ')')
        && chars[2].is_whitespace()
    {
        return Some(ListPrefix::Number("A", trimmed[3..].trim().to_string()));
    }

    // Roman numerals (lower): "i. ", "ii. ", "iii. "
    let lower_roman_prefixes = [
        "i. ", "ii. ", "iii. ", "iv. ", "v. ", "vi. ", "vii. ", "viii. ", "ix. ", "x. ",
    ];
    for p in &lower_roman_prefixes {
        if trimmed.starts_with(p) {
            return Some(ListPrefix::Number("i", trimmed[p.len()..].trim().to_string()));
        }
    }

    // Roman numerals (upper): "I. ", "II. ", "III. "
    let upper_roman_prefixes = [
        "I. ", "II. ", "III. ", "IV. ", "V. ", "VI. ", "VII. ", "VIII. ", "IX. ", "X. ",
    ];
    for p in &upper_roman_prefixes {
        if trimmed.starts_with(p) {
            return Some(ListPrefix::Number("I", trimmed[p.len()..].trim().to_string()));
        }
    }

    // Bullet: square
    for prefix in &["▪ ", "▪", "\u{25AA} ", "■ ", "\u{25A0} "] {
        if let Some(content) = trimmed.strip_prefix(prefix) {
            if !content.trim().is_empty() {
                return Some(ListPrefix::Bullet("square", content.trim().to_string()));
            }
        }
    }

    // Bullet: circle
    for prefix in &["◦ ", "◦", "\u{25E6} "] {
        if let Some(content) = trimmed.strip_prefix(prefix) {
            if !content.trim().is_empty() {
                return Some(ListPrefix::Bullet("circle", content.trim().to_string()));
            }
        }
    }

    // Bullet: dash
    for prefix in &["– ", "— ", "- "] {
        if let Some(content) = trimmed.strip_prefix(prefix) {
            if !content.trim().is_empty() {
                return Some(ListPrefix::Bullet("dash", content.trim().to_string()));
            }
        }
    }

    // Bullet: disc (default)
    for prefix in &["• ", "•", "* ", "\u{2022} "] {
        if let Some(content) = trimmed.strip_prefix(prefix) {
            if !content.trim().is_empty() {
                return Some(ListPrefix::Bullet("disc", content.trim().to_string()));
            }
        }
    }

    None
}

fn is_heading_candidate(line: &str) -> Option<(u8, String)> {
    let t = line.trim();
    if t.is_empty() {
        return None;
    }

    if let Some(h) = t.strip_prefix("### ") {
        return Some((3, h.trim().to_string()));
    }
    if let Some(h) = t.strip_prefix("## ") {
        return Some((2, h.trim().to_string()));
    }
    if let Some(h) = t.strip_prefix("# ") {
        return Some((1, h.trim().to_string()));
    }

    if t.ends_with(':') && t.len() < 55 {
        if t.chars().next().map_or(false, |c| c.is_ascii_digit()) && t.contains(')') {
            return Some((2, t.to_string()));
        }
        return Some((2, t.to_string()));
    }

    let alphabetic_chars: Vec<char> = t.chars().filter(|c| c.is_alphabetic()).collect();
    if alphabetic_chars.len() >= 4 && alphabetic_chars.iter().all(|c| c.is_uppercase()) && t.len() < 60 {
        return Some((1, t.to_string()));
    }

    let lower = t.to_lowercase();
    let common_titles = [
        "abstract", "introduction", "existing system", "proposed system",
        "system architecture", "hardware requirements", "software requirements",
        "methodology", "literature survey", "conclusion", "references", "acknowledgement"
    ];
    for title in &common_titles {
        if lower == *title || lower == format!("{}:", title) {
            return Some((2, t.to_string()));
        }
    }

    None
}

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
                                blocks.push(PdfBlock::NumberList(num_type.to_string(), vec![item_text]));
                            }
                        } else {
                            blocks.push(PdfBlock::NumberList(num_type.to_string(), vec![item_text]));
                        }
                    }
                    ListPrefix::Bullet(bullet_type, item_text) => {
                        if let Some(PdfBlock::BulletList(cur_type, items)) = blocks.last_mut() {
                            if cur_type == bullet_type {
                                items.push(item_text);
                            } else {
                                blocks.push(PdfBlock::BulletList(bullet_type.to_string(), vec![item_text]));
                            }
                        } else {
                            blocks.push(PdfBlock::BulletList(bullet_type.to_string(), vec![item_text]));
                        }
                    }
                }
                continue;
            }

            // Address line (ends with comma or is To / From)
            let is_addr = (line.ends_with(',') && line.len() < 50)
                || (cur_address.is_empty() && (line == "To" || line == "To," || line == "From" || line == "From,"));

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
        let mut page_nodes: Vec<String> = Vec::new();

        for block in blocks {
            match block {
                PdfBlock::Heading(level, text) => {
                    page_nodes.push(format!("      <h{}>{}</h{}>", level, escape_xml(&text), level));
                }
                PdfBlock::Paragraph(text) => {
                    page_nodes.push(format!("      <p>{}</p>", escape_xml(&text)));
                }
                PdfBlock::Blockquote(text) => {
                    page_nodes.push(format!("      <blockquote>\n        <p>{}</p>\n      </blockquote>", escape_xml(&text)));
                }
                PdfBlock::AddressBlock(lines) => {
                    let escaped_lines: Vec<String> = lines.iter().map(|l| escape_xml(l)).collect();
                    page_nodes.push(format!("      <p>{}</p>", escaped_lines.join("<br />\n         ")));
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
                    page_nodes.push(list_xml);
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
                    page_nodes.push(list_xml);
                }
            }
        }

        if !page_nodes.is_empty() {
            section_xmls.push(format!(
                "    <section page-style=\"body\">\n{}\n    </section>",
                page_nodes.join("\n")
            ));
        }
    }

    if section_xmls.is_empty() {
        section_xmls.push("    <section page-style=\"body\">\n      <p>Empty document</p>\n    </section>".to_string());
    }

    let dxml = format!(
        r#"<doc>
  <props>
    <meta>
      <title>Imported PDF Document</title>
      <author>Pagina Reverse-Engineering Engine</author>
    </meta>
    <paper size="A4" orientation="portrait" margin="20mm" />
    <link rel="stylesheet" href="styles.dcss" />
    <page-styles>
      <page-style name="body" default="true">
        <running-header>
          <left><text>Imported PDF</text></left>
          <right><page-number/> of <total-pages/></right>
        </running-header>
        <running-footer>
          <center><text>Pagina Engine</text></center>
        </running-footer>
      </page-style>
    </page-styles>
  </props>

  <body>
{}
  </body>
</doc>"#,
        section_xmls.join("\n\n")
    );

    let dcss = r#"/* Generated by Pagina PDF Reverse-Engineering Engine */
body {
  font-family: Inter, Segoe UI, system-ui, -apple-system, sans-serif;
  color: #334155;
}

h1 {
  font-size: 24pt;
  font-weight: bold;
  color: #1e3a8a;
  margin-bottom: 5mm;
}

h2 {
  font-size: 16pt;
  font-weight: bold;
  color: #0f172a;
  margin-top: 6mm;
  margin-bottom: 3mm;
}

h3 {
  font-size: 12pt;
  font-weight: 600;
  color: #1e293b;
  margin-top: 4mm;
  margin-bottom: 2mm;
}

p {
  font-size: 10.5pt;
  line-height: 1.6;
  color: #334155;
  margin-bottom: 3.5mm;
}

blockquote {
  margin-left: 6mm;
  padding-left: 4mm;
  border-left: 3px solid #cbd5e1;
  color: #475569;
  margin-bottom: 4mm;
}

ul, ol {
  padding-left: 24px;
  margin-top: 2mm;
  margin-bottom: 3.5mm;
  color: #334155;
}

li {
  margin-bottom: 1.5mm;
  line-height: 1.6;
}

.callout {
  background-color: #f0fdf4;
  border-left: 4px solid #16a34a;
  padding: 4mm 6mm;
  margin: 4mm 0;
  border-radius: 4px;
}

.col-box {
  background-color: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  padding: 4mm;
}

code {
  font-family: Consolas, monospace;
  background-color: #f1f5f9;
  padding: 1px 4px;
  border-radius: 3px;
  font-size: 9.5pt;
}

table {
  width: 100%;
  border-collapse: collapse;
  margin: 4mm 0;
}

th, td {
  border: 1px solid #cbd5e1;
  padding: 2.5mm 3.5mm;
  text-align: left;
}

th {
  background-color: #f8fafc;
  font-weight: bold;
}
"#.to_string();

    Ok(ImportResult { dxml, dcss })
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
