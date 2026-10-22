use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct InlineToken {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub code: bool,
    pub sup: bool,
    pub sub: bool,
    pub highlight: bool,
    pub font_family: Option<String>,
    pub font_size_half_points: Option<u32>,
    pub color: Option<String>,
    pub highlight_color: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FlatListItem {
    pub ilvl: u32,
    pub is_bullet: bool,
    pub list_type: String,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct ItemNode {
    pub text: String,
    pub sublists: Vec<ListNode>,
}

#[derive(Debug, Clone)]
pub struct ListNode {
    pub is_bullet: bool,
    pub list_type: String,
    pub items: Vec<ItemNode>,
}

#[derive(Debug, Clone)]
pub enum ImportedBlock {
    Heading(u32, Option<String>, String),
    Paragraph(Option<String>, bool, String, String),
    Callout(String),
    Blockquote(String),
    List(Vec<FlatListItem>),
    Grid(Vec<Vec<String>>),
    Image(String),
    Table {
        rows: Vec<Vec<Vec<String>>>,
        is_borderless: bool,
        is_align_right: bool,
    },
}

#[derive(Debug, Clone)]
pub enum ListFormat {
    Bullet(String),
    Number(String),
}

#[derive(Debug, Default)]
pub struct NumberingInfo {
    pub num_to_abstract: HashMap<String, String>,
    pub abstract_lvl_format: HashMap<(String, u32), ListFormat>,
}

#[derive(Debug, Clone, Default)]
pub struct ImportedTextStyle {
    pub font_family: Option<String>,
    pub font_size_half_points: Option<u32>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct PartImg {
    pub r_id: String,
    pub cx: Option<u64>,
    pub cy: Option<u64>,
    pub pos_offset: Option<i64>,
    pub vml_style: String,
}

#[derive(Debug, Default, Clone)]
pub struct ParsedGenericPart {
    pub left_items: Vec<String>,
    pub center_items: Vec<String>,
    pub right_items: Vec<String>,
    pub max_image_height_mm: Option<f32>,
}

impl ParsedGenericPart {
    pub fn is_empty(&self) -> bool {
        self.left_items.is_empty() && self.center_items.is_empty() && self.right_items.is_empty()
    }

    pub fn to_dxml(&self, tag_name: &str) -> String {
        if self.is_empty() {
            return String::new();
        }

        let mut out = format!("        <{}>\n", tag_name);
        if !self.left_items.is_empty() {
            out.push_str("          <left>\n");
            for item in &self.left_items {
                out.push_str(&format!("            {}\n", item));
            }
            out.push_str("          </left>\n");
        }
        if !self.center_items.is_empty() {
            out.push_str("          <center>\n");
            for item in &self.center_items {
                out.push_str(&format!("            {}\n", item));
            }
            out.push_str("          </center>\n");
        }
        if !self.right_items.is_empty() {
            out.push_str("          <right>\n");
            for item in &self.right_items {
                out.push_str(&format!("            {}\n", item));
            }
            out.push_str("          </right>\n");
        }
        out.push_str(&format!("        </{}>", tag_name));
        out
    }
}

#[derive(Debug, Clone)]
pub struct ParsedDocumentXml {
    pub blocks: Vec<ImportedBlock>,
    pub paper_size: String,
    pub paper_orientation: String,
    pub paper_margin_mm: f32,
    pub header_distance_mm: Option<f32>,
    pub footer_distance_mm: Option<f32>,
    pub font_counts: HashMap<String, usize>,
    pub font_size_counts: HashMap<u32, usize>,
    pub font_color_counts: HashMap<String, usize>,
}

pub fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
