use super::styles::emit_inline_runs;
use crate::ast::{ElementNode, Node};

pub fn emit_table(el: &ElementNode, out: &mut String) {
    let is_borderless = el.classes.iter().any(|c| c == "borderless")
        || el
            .attributes
            .get("class")
            .map(|c| c.contains("borderless"))
            .unwrap_or(false)
        || el
            .inline_styles
            .get("border")
            .map(|b| b.contains("none"))
            .unwrap_or(false);

    let is_align_right = el.classes.iter().any(|c| c == "align-right")
        || el
            .attributes
            .get("class")
            .map(|c| c.contains("align-right"))
            .unwrap_or(false)
        || el
            .attributes
            .get("align")
            .map(|a| a == "right")
            .unwrap_or(false)
        || el
            .inline_styles
            .get("margin-left")
            .map(|m| m.contains("auto"))
            .unwrap_or(false);

    out.push_str("<w:tbl>\n");
    out.push_str("  <w:tblPr>\n");
    if is_align_right {
        out.push_str("    <w:tblInd w:w=\"7359\" w:type=\"dxa\"/>\n");
    }
    out.push_str("    <w:tblW w:w=\"5000\" w:type=\"pct\"/>\n");
    if is_borderless {
        out.push_str(
            r#"    <w:tblBorders>
      <w:top w:val="nil"/>
      <w:left w:val="nil"/>
      <w:bottom w:val="nil"/>
      <w:right w:val="nil"/>
      <w:insideH w:val="nil"/>
      <w:insideV w:val="nil"/>
    </w:tblBorders>
"#,
        );
    } else {
        out.push_str(
            r#"    <w:tblBorders>
      <w:top w:val="single" w:sz="4" w:space="0" w:color="CBD5E1"/>
      <w:left w:val="single" w:sz="4" w:space="0" w:color="CBD5E1"/>
      <w:bottom w:val="single" w:sz="4" w:space="0" w:color="CBD5E1"/>
      <w:right w:val="single" w:sz="4" w:space="0" w:color="CBD5E1"/>
      <w:insideH w:val="single" w:sz="4" w:space="0" w:color="E2E8F0"/>
      <w:insideV w:val="single" w:sz="4" w:space="0" w:color="E2E8F0"/>
    </w:tblBorders>
"#,
        );
    }
    out.push_str("  </w:tblPr>\n");

    let mut rows: Vec<&ElementNode> = Vec::new();
    collect_rows(el, &mut rows);

    for tr in rows {
        out.push_str("  <w:tr>\n");
        for child in &tr.children {
            if let Node::Element(cell) = child {
                let cell_tag = cell.tag.to_lowercase();
                if cell_tag == "td" || cell_tag == "th" {
                    let is_header = !is_borderless && cell_tag == "th";
                    out.push_str("    <w:tc>\n");
                    out.push_str("      <w:tcPr>\n");
                    out.push_str(
                        r#"        <w:tcMar>
          <w:top w:w="140" w:type="dxa"/>
          <w:left w:w="180" w:type="dxa"/>
          <w:bottom w:w="140" w:type="dxa"/>
          <w:right w:w="180" w:type="dxa"/>
        </w:tcMar>
"#,
                    );
                    if is_borderless {
                        out.push_str(
                            r#"        <w:tcBorders>
          <w:top w:val="nil"/>
          <w:left w:val="nil"/>
          <w:bottom w:val="nil"/>
          <w:right w:val="nil"/>
        </w:tcBorders>
"#,
                        );
                    }
                    if is_header {
                        out.push_str(
                            "        <w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"F8FAFC\"/>\n",
                        );
                    }
                    out.push_str("      </w:tcPr>\n");

                    let cell_align =
                        cell.attributes
                            .get("align")
                            .map(|s| s.as_str())
                            .or_else(|| {
                                cell.children.iter().find_map(|c| {
                                    if let Node::Element(p) = c {
                                        p.attributes.get("align").map(|s| s.as_str())
                                    } else {
                                        None
                                    }
                                })
                            });

                    let ppr = match cell_align {
                        Some("center") => "<w:pPr><w:jc w:val=\"center\"/></w:pPr>",
                        Some("right") => "<w:pPr><w:jc w:val=\"right\"/></w:pPr>",
                        _ => "",
                    };

                    out.push_str(&format!("      <w:p>{}", ppr));
                    emit_inline_runs(
                        cell, is_header, false, false, false, false, false, false, out,
                    );
                    out.push_str("</w:p>\n");
                    out.push_str("    </w:tc>\n");
                }
            }
        }
        out.push_str("  </w:tr>\n");
    }

    out.push_str("</w:tbl>\n");
}

pub fn collect_rows<'a>(el: &'a ElementNode, rows: &mut Vec<&'a ElementNode>) {
    for child in &el.children {
        if let Node::Element(child_el) = child {
            let tag = child_el.tag.to_lowercase();
            if tag == "tr" {
                rows.push(child_el);
            } else if tag == "thead" || tag == "tbody" || tag == "tfoot" {
                collect_rows(child_el, rows);
            }
        }
    }
}

pub fn emit_grid<F>(el: &ElementNode, out: &mut String, mut convert_child: F)
where
    F: FnMut(&Node, &mut String),
{
    let col_divs: Vec<&ElementNode> = el
        .children
        .iter()
        .filter_map(|c| {
            if let Node::Element(child_el) = c {
                Some(child_el)
            } else {
                None
            }
        })
        .collect();

    if col_divs.is_empty() {
        return;
    }

    let n = col_divs.len();
    let gap_pct = if n > 1 { 200 } else { 0 };
    let total_gap = (n - 1) * gap_pct;
    let col_pct = ((5000 - total_gap) / n).max(500);

    out.push_str("<w:tbl>\n");
    out.push_str(
        r#"  <w:tblPr>
    <w:tblW w:w="5000" w:type="pct"/>
    <w:tblBorders>
      <w:top w:val="nil"/>
      <w:left w:val="nil"/>
      <w:bottom w:val="nil"/>
      <w:right w:val="nil"/>
      <w:insideH w:val="nil"/>
      <w:insideV w:val="nil"/>
    </w:tblBorders>
  </w:tblPr>
  <w:tr>
"#,
    );

    for (i, col) in col_divs.iter().enumerate() {
        if i > 0 && gap_pct > 0 {
            out.push_str(&format!(
                r#"    <w:tc>
      <w:tcPr>
        <w:tcW w:w="{}" w:type="pct"/>
        <w:tcBorders>
          <w:top w:val="nil"/>
          <w:left w:val="nil"/>
          <w:bottom w:val="nil"/>
          <w:right w:val="nil"/>
        </w:tcBorders>
      </w:tcPr>
      <w:p/>
    </w:tc>
"#,
                gap_pct
            ));
        }

        let is_boxed = col.classes.iter().any(|c| c == "col-box");
        let box_styling = if is_boxed {
            r#"        <w:tcBorders>
          <w:top w:val="single" w:sz="6" w:space="0" w:color="E2E8F0"/>
          <w:left w:val="single" w:sz="6" w:space="0" w:color="E2E8F0"/>
          <w:bottom w:val="single" w:sz="6" w:space="0" w:color="E2E8F0"/>
          <w:right w:val="single" w:sz="6" w:space="0" w:color="E2E8F0"/>
        </w:tcBorders>
        <w:shd w:val="clear" w:color="auto" w:fill="F8FAFC"/>
        <w:tcMar>
          <w:top w:w="180" w:type="dxa"/>
          <w:left w:w="220" w:type="dxa"/>
          <w:bottom w:w="180" w:type="dxa"/>
          <w:right w:w="220" w:type="dxa"/>
        </w:tcMar>"#
        } else {
            r#"        <w:tcBorders>
          <w:top w:val="nil"/>
          <w:left w:val="nil"/>
          <w:bottom w:val="nil"/>
          <w:right w:val="nil"/>
        </w:tcBorders>
        <w:tcMar>
          <w:top w:w="120" w:type="dxa"/>
          <w:left w:w="120" w:type="dxa"/>
          <w:bottom w:w="120" w:type="dxa"/>
          <w:right w:w="120" w:type="dxa"/>
        </w:tcMar>"#
        };

        out.push_str(&format!(
            r#"    <w:tc>
      <w:tcPr>
        <w:tcW w:w="{}" w:type="pct"/>
{}
      </w:tcPr>
"#,
            col_pct, box_styling
        ));

        for child in &col.children {
            convert_child(child, out);
        }

        out.push_str("    </w:tc>\n");
    }

    out.push_str("  </w:tr>\n</w:tbl>\n");
}
