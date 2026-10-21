use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub col: usize,
}

impl Default for Span {
    fn default() -> Self {
        Self {
            start: 0,
            end: 0,
            line: 1,
            col: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub props: DocumentProps,
    pub body: BodyNode,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentProps {
    pub metadata: HashMap<String, String>,
    pub paper: PaperConfig,
    pub stylesheet: Stylesheet,
    pub page_styles: HashMap<String, PageStyle>,
    pub default_page_style: String,
}

impl Default for DocumentProps {
    fn default() -> Self {
        Self {
            metadata: HashMap::new(),
            paper: PaperConfig::default(),
            stylesheet: Stylesheet::default(),
            page_styles: HashMap::new(),
            default_page_style: "body".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperConfig {
    pub size: String,
    pub orientation: String,
    pub margin: Length,
}

impl Default for PaperConfig {
    fn default() -> Self {
        Self {
            size: "A4".to_string(),
            orientation: "portrait".to_string(),
            margin: Length {
                val: 20.0,
                unit: Unit::Mm,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageStyle {
    pub name: String,
    pub is_default: bool,
    pub margins: Option<Margins>,
    pub running_header: Option<HeaderFooterDef>,
    pub running_footer: Option<HeaderFooterDef>,
    pub watermark: Option<WatermarkDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Margins {
    pub top: Length,
    pub bottom: Length,
    pub left: Length,
    pub right: Length,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderFooterDef {
    pub left: Vec<Node>,
    pub center: Vec<Node>,
    pub right: Vec<Node>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatermarkDef {
    pub text: String,
    pub opacity: f32,
    pub angle_deg: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyNode {
    pub attributes: HashMap<String, String>,
    pub children: Vec<Node>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Node {
    Section(SectionNode),
    Element(ElementNode),
    Text(TextNode),
    Comment(CommentNode),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionNode {
    pub page_style: Option<String>,
    pub page_break: bool,
    pub attributes: HashMap<String, String>,
    pub children: Vec<Node>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementNode {
    pub tag: String,
    pub attributes: HashMap<String, String>,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub inline_styles: HashMap<String, String>,
    pub children: Vec<Node>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextNode {
    pub content: String,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentNode {
    pub content: String,
    pub span: Span,
}

// ---------------- Stylesheet AST ----------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Stylesheet {
    pub rules: Vec<StyleRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleRule {
    pub selectors: Vec<String>,
    pub declarations: HashMap<String, String>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Length {
    pub val: f32,
    pub unit: Unit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Unit {
    Mm,
    Cm,
    Inch,
    Pt,
    Px,
    Em,
    Percent,
}
