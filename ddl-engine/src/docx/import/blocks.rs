use super::list::{build_list_tree, render_list_node_dxml};
use super::types::ImportedBlock;

pub fn render_blocks_to_dxml(blocks: Vec<ImportedBlock>) -> Vec<String> {
    let mut body_nodes: Vec<String> = Vec::new();

    for block in blocks {
        match block {
            ImportedBlock::Heading(level, align, text) => {
                let align_attr = align
                    .map(|a| format!(" align=\"{}\"", a))
                    .unwrap_or_default();
                body_nodes.push(format!(
                    "      <h{}{}>{}</h{}>",
                    level, align_attr, text, level
                ));
            }
            ImportedBlock::Paragraph(align, is_lead, text, direct_style) => {
                let mut attrs = String::new();
                if is_lead {
                    attrs.push_str(" class=\"lead\"");
                }
                if let Some(a) = align {
                    attrs.push_str(&format!(" align=\"{}\"", a));
                }
                if !direct_style.is_empty() {
                    attrs.push_str(&format!(" style=\"{}\"", direct_style));
                }
                body_nodes.push(format!("      <p{}>{}</p>", attrs, text));
            }
            ImportedBlock::Callout(text) => {
                body_nodes.push(format!(
                    "      <div class=\"callout\">\n        {}\n      </div>",
                    text
                ));
            }
            ImportedBlock::Blockquote(text) => {
                body_nodes.push(format!(
                    "      <blockquote>\n        <p>{}</p>\n      </blockquote>",
                    text
                ));
            }
            ImportedBlock::List(items) => {
                let list_trees = build_list_tree(&items);
                for tree in list_trees {
                    let mut dxml_out = String::new();
                    render_list_node_dxml(&tree, "      ", &mut dxml_out);
                    body_nodes.push(dxml_out);
                }
            }
            ImportedBlock::Grid(cols) => {
                let cols_spec = vec!["1fr"; cols.len()].join(" ");
                let mut grid_xml = format!("      <grid cols=\"{}\" gap=\"6mm\">\n", cols_spec);
                for col_ps in cols {
                    grid_xml.push_str("        <div class=\"col-box\">\n");
                    for p in col_ps {
                        grid_xml.push_str(&format!("          {}\n", p));
                    }
                    grid_xml.push_str("        </div>\n");
                }
                grid_xml.push_str("      </grid>");
                body_nodes.push(grid_xml);
            }
            ImportedBlock::Image(url) => {
                body_nodes.push(format!(
                    "      <div align=\"center\" style=\"margin: 4mm 0;\"><img src=\"{}\" style=\"max-width: 100%; height: auto;\" /></div>",
                    url
                ));
            }
            ImportedBlock::Table {
                rows,
                is_borderless,
                is_align_right,
            } => {
                let mut classes = Vec::new();
                if is_borderless {
                    classes.push("borderless");
                }
                if is_align_right {
                    classes.push("align-right");
                }
                let class_attr = if !classes.is_empty() {
                    format!(" class=\"{}\"", classes.join(" "))
                } else {
                    String::new()
                };
                let style_attr = if is_align_right {
                    " style=\"margin-left: auto; width: 45%;\""
                } else {
                    ""
                };
                let mut tbl_xml = format!("      <table{}{}>\n", class_attr, style_attr);
                for (row_idx, row) in rows.iter().enumerate() {
                    tbl_xml.push_str("        <tr>\n");
                    for cell_ps in row {
                        let tag = if row_idx == 0 && !is_borderless {
                            "th"
                        } else {
                            "td"
                        };
                        let td_style = if is_borderless {
                            " style=\"border: none;\""
                        } else {
                            ""
                        };
                        let content = if cell_ps.is_empty() {
                            String::new()
                        } else {
                            cell_ps.join("\n          ")
                        };
                        tbl_xml.push_str(&format!(
                            "          <{}{}>\n            {}\n          </{}>\n",
                            tag,
                            td_style,
                            content.trim(),
                            tag
                        ));
                    }
                    tbl_xml.push_str("        </tr>\n");
                }
                tbl_xml.push_str("      </table>");
                body_nodes.push(tbl_xml);
            }
        }
    }

    body_nodes
}
