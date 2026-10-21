use crate::ImportResult;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::io::{Cursor, Read};

pub fn import_docx(bytes: &[u8]) -> Result<ImportResult, Box<dyn std::error::Error + Send + Sync>> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))?;
    
    let mut doc_xml = String::new();
    {
        let mut file = archive.by_name("word/document.xml")?;
        file.read_to_string(&mut doc_xml)?;
    }

    let mut reader = Reader::from_str(&doc_xml);
    reader.config_mut().trim_text(false);

    let mut body_nodes: Vec<String> = Vec::new();
    let mut current_p_style: Option<String> = None;
    let mut current_p_text: Vec<String> = Vec::new();
    let mut in_r = false;
    let mut in_t = false;
    let mut is_bold = false;
    let mut is_italic = false;
    let mut is_underline = false;

    // Table state
    let mut in_tbl = false;
    let mut current_table_rows: Vec<Vec<String>> = Vec::new();
    let mut current_row: Vec<String> = Vec::new();
    let mut current_cell_paragraphs: Vec<String> = Vec::new();

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) => {
                let name = e.name();
                match name.as_ref() {
                    b"w:tbl" => {
                        in_tbl = true;
                        current_table_rows.clear();
                    }
                    b"w:tr" => {
                        current_row.clear();
                    }
                    b"w:tc" => {
                        current_cell_paragraphs.clear();
                    }
                    b"w:p" => {
                        current_p_style = None;
                        current_p_text.clear();
                    }
                    b"w:r" => {
                        in_r = true;
                        is_bold = false;
                        is_italic = false;
                        is_underline = false;
                    }
                    b"w:t" => {
                        in_t = true;
                    }
                    _ => {}
                }
            }
            Event::Empty(e) => {
                let name = e.name();
                match name.as_ref() {
                    b"w:pStyle" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"w:val" {
                                if let Ok(val) = std::str::from_utf8(&attr.value) {
                                    current_p_style = Some(val.to_string());
                                }
                            }
                        }
                    }
                    b"w:b" => {
                        if in_r {
                            is_bold = true;
                        }
                    }
                    b"w:i" => {
                        if in_r {
                            is_italic = true;
                        }
                    }
                    b"w:u" => {
                        if in_r {
                            is_underline = true;
                        }
                    }
                    _ => {}
                }
            }
            Event::Text(e) => {
                if in_t {
                    let text = e.unescape()?.to_string();
                    if !text.is_empty() {
                        let mut formatted = escape_xml(&text);
                        if is_bold && is_italic {
                            formatted = format!("<b><i>{}</i></b>", formatted);
                        } else if is_bold {
                            formatted = format!("<b>{}</b>", formatted);
                        } else if is_italic {
                            formatted = format!("<i>{}</i>", formatted);
                        }
                        if is_underline {
                            formatted = format!("<u>{}</u>", formatted);
                        }
                        current_p_text.push(formatted);
                    }
                }
            }
            Event::End(e) => {
                let name = e.name();
                match name.as_ref() {
                    b"w:t" => {
                        in_t = false;
                    }
                    b"w:r" => {
                        in_r = false;
                    }
                    b"w:p" => {
                        let line = current_p_text.join("");
                        if !line.trim().is_empty() {
                            let style_lower = current_p_style.as_deref().unwrap_or("").to_lowercase();
                            let node_xml = if style_lower.contains("heading1") || style_lower == "1" {
                                format!("      <h1>{}</h1>", line)
                            } else if style_lower.contains("heading2") || style_lower == "2" {
                                format!("      <h2>{}</h2>", line)
                            } else if style_lower.contains("heading3") || style_lower == "3" {
                                format!("      <h3>{}</h3>", line)
                            } else if style_lower.contains("listbullet") {
                                format!("      <ul><li>{}</li></ul>", line)
                            } else if style_lower.contains("listnumber") {
                                format!("      <ol><li>{}</li></ol>", line)
                            } else {
                                format!("      <p>{}</p>", line)
                            };

                            if in_tbl {
                                current_cell_paragraphs.push(line);
                            } else {
                                body_nodes.push(node_xml);
                            }
                        }
                    }
                    b"w:tc" => {
                        let cell_content = if current_cell_paragraphs.is_empty() {
                            String::new()
                        } else {
                            current_cell_paragraphs.join("<br/>")
                        };
                        current_row.push(cell_content);
                    }
                    b"w:tr" => {
                        current_table_rows.push(current_row.clone());
                    }
                    b"w:tbl" => {
                        in_tbl = false;
                        let mut tbl_xml = String::from("      <table>\n");
                        for (row_idx, row) in current_table_rows.iter().enumerate() {
                            tbl_xml.push_str("        <tr>\n");
                            for cell in row {
                                let tag = if row_idx == 0 { "th" } else { "td" };
                                tbl_xml.push_str(&format!("          <{}>{}</{}>\n", tag, cell.trim(), tag));
                            }
                            tbl_xml.push_str("        </tr>\n");
                        }
                        tbl_xml.push_str("      </table>");
                        body_nodes.push(tbl_xml);
                    }
                    _ => {}
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    let dxml = format!(
        r#"<doc>
  <props>
    <meta>
      <title>Imported OpenXML Document</title>
      <author>Pagina Reverse-Engineering Engine</author>
    </meta>
    <paper size="A4" orientation="portrait" margin="20mm" />
    <link rel="stylesheet" href="styles.dcss" />
    <page-styles>
      <page-style name="body" default="true">
        <running-header>
          <left><text>Imported Document</text></left>
          <right><page-number/> of <total-pages/></right>
        </running-header>
        <running-footer>
          <center><text>Pagina Engine</text></center>
        </running-footer>
      </page-style>
    </page-styles>
  </props>

  <body>
    <section page-style="body">
{}
    </section>
  </body>
</doc>"#,
        if body_nodes.is_empty() {
            "      <p>Empty document</p>".to_string()
        } else {
            body_nodes.join("\n")
        }
    );

    let dcss = r#"/* Generated by Pagina OpenXML Reverse Engine */
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
  margin-bottom: 2mm;
}

p {
  font-size: 10.5pt;
  line-height: 1.6;
  color: #334155;
  margin-bottom: 3mm;
}

table {
  width: 100%;
  border-collapse: collapse;
  margin: 4mm 0;
}

th, td {
  border: 1px solid #cbd5e1;
  padding: 2mm 3mm;
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
