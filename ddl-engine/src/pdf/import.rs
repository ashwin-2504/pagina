use crate::ImportResult;

#[derive(Debug)]
enum PdfBlock {
    Heading(u8, String),
    Paragraph(String),
    BulletList(Vec<String>),
    NumberList(Vec<String>),
    AddressBlock(Vec<String>),
}

fn is_page_number_or_header(line: &str) -> bool {
    let t = line.trim();
    if t.is_empty() {
        return true;
    }
    // Only numbers e.g. "1", "2"
    if t.chars().all(|c| c.is_ascii_digit()) && t.len() <= 4 {
        return true;
    }
    // "1 of 5", "Page 1 of 5"
    if (t.contains(" of ") || t.contains(" OF ")) && t.len() <= 15 {
        return true;
    }
    false
}

fn parse_list_item(line: &str) -> Option<(bool, String)> {
    let trimmed = line.trim();
    // Do not treat numbered headings ending in colon as list items:
    if trimmed.ends_with(':') {
        return None;
    }
    // Numbered list: "1. ", "2. ", "1) "
    if let Some(first) = trimmed.chars().next() {
        if first.is_ascii_digit() {
            let rest = trimmed.trim_start_matches(|c: char| c.is_ascii_digit());
            if let Some(content) = rest.strip_prefix(". ").or_else(|| rest.strip_prefix(") ")) {
                if !content.trim().is_empty() {
                    return Some((true, content.trim().to_string()));
                }
            }
        }
    }
    // Bullet: "• ", "- ", "* ", "\u{2022} "
    for prefix in &["• ", "- ", "* ", "\u{2022} ", "– "] {
        if let Some(content) = trimmed.strip_prefix(prefix) {
            if !content.trim().is_empty() {
                return Some((false, content.trim().to_string()));
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

    // Explicit markdown headings
    if let Some(h) = t.strip_prefix("### ") {
        return Some((3, h.trim().to_string()));
    }
    if let Some(h) = t.strip_prefix("## ") {
        return Some((2, h.trim().to_string()));
    }
    if let Some(h) = t.strip_prefix("# ") {
        return Some((1, h.trim().to_string()));
    }

    // Numbered section heading e.g. "1) Name of the Students:" or "2) Abstract Of Dissertation:"
    if t.ends_with(':') && t.len() < 55 {
        if t.chars().next().map_or(false, |c| c.is_ascii_digit()) && t.contains(')') {
            return Some((2, t.to_string()));
        }
        return Some((2, t.to_string()));
    }

    // All uppercase titles e.g. "U.G. PROJECT REGISTRATION FORM"
    let alphabetic_chars: Vec<char> = t.chars().filter(|c| c.is_alphabetic()).collect();
    if alphabetic_chars.len() >= 4 && alphabetic_chars.iter().all(|c| c.is_uppercase()) && t.len() < 60 {
        return Some((1, t.to_string()));
    }

    // Common standard section titles
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
    // Extract full text across all pages with full Adobe CMap & ToUnicode support
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
            let line = raw_lines[i].trim();
            i += 1;

            if line.is_empty() {
                // Empty line triggers paragraph flush
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

            if let Some((is_numbered, item_text)) = parse_list_item(line) {
                if !cur_para.is_empty() {
                    blocks.push(PdfBlock::Paragraph(cur_para.clone()));
                    cur_para.clear();
                }
                if !cur_address.is_empty() {
                    blocks.push(PdfBlock::AddressBlock(cur_address.clone()));
                    cur_address.clear();
                }

                if is_numbered {
                    if let Some(PdfBlock::NumberList(items)) = blocks.last_mut() {
                        items.push(item_text);
                    } else {
                        blocks.push(PdfBlock::NumberList(vec![item_text]));
                    }
                } else if let Some(PdfBlock::BulletList(items)) = blocks.last_mut() {
                    items.push(item_text);
                } else {
                    blocks.push(PdfBlock::BulletList(vec![item_text]));
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
                // If previous line was address line, this may conclude the address block
                cur_address.push(line.to_string());
                blocks.push(PdfBlock::AddressBlock(cur_address.clone()));
                cur_address.clear();
                continue;
            }

            // Accumulate into flowing paragraph
            if cur_para.is_empty() {
                cur_para.push_str(line);
            } else if cur_para.ends_with('-') && !cur_para.ends_with(" -") {
                cur_para.pop(); // remove trailing hyphen
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
                PdfBlock::AddressBlock(lines) => {
                    let escaped_lines: Vec<String> = lines.iter().map(|l| escape_xml(l)).collect();
                    page_nodes.push(format!("      <p>{}</p>", escaped_lines.join("<br />\n         ")));
                }
                PdfBlock::BulletList(items) => {
                    let mut list_xml = String::from("      <ul>\n");
                    for item in items {
                        list_xml.push_str(&format!("        <li>{}</li>\n", escape_xml(&item)));
                    }
                    list_xml.push_str("      </ul>");
                    page_nodes.push(list_xml);
                }
                PdfBlock::NumberList(items) => {
                    let mut list_xml = String::from("      <ol>\n");
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
  font-size: 13pt;
  font-weight: 600;
  color: #1e293b;
  margin-top: 4mm;
  margin-bottom: 2mm;
}

p {
  font-size: 10.5pt;
  line-height: 1.6;
  color: #334155;
  margin-bottom: 3mm;
}

ul, ol {
  margin: 3mm 0 4mm 6mm;
  padding: 0;
  color: #334155;
}

li {
  margin-bottom: 1.5mm;
  line-height: 1.5;
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
