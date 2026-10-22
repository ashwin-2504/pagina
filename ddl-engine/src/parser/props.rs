use super::css::{parse_css, parse_length};
use super::Parser;
use crate::ast::*;
use std::collections::HashMap;

impl<'a> Parser<'a> {
    pub(crate) fn parse_props_block(&mut self) -> DocumentProps {
        let mut props = DocumentProps::default();
        let (tag, _, is_self_closing) = match self.parse_start_tag() {
            Ok(t) => t,
            Err(_) => return props,
        };

        if tag != "props" || is_self_closing {
            return props;
        }

        while self.pos < self.chars.len() {
            self.skip_whitespace();
            if self.starts_with("</props>") {
                self.advance_str("</props>");
                break;
            }

            if self.starts_with("<paper") {
                props.paper = self.parse_paper();
            } else if self.starts_with("<style") {
                props.stylesheet = self.parse_style_tag();
            } else if self.starts_with("<link") {
                if let Ok((_tag, attrs, _)) = self.parse_start_tag() {
                    if let Some(href) = attrs.get("href") {
                        for prefix in &["", "assets/", "../"] {
                            let path = format!("{}{}", prefix, href);
                            if let Ok(css_content) = std::fs::read_to_string(&path) {
                                let wrapped_css = format!("<style>{}</style>", css_content);
                                let mut css_parser = Parser::new(&wrapped_css);
                                let ss = css_parser.parse_style_tag();
                                props.stylesheet.rules.extend(ss.rules);
                                break;
                            }
                        }
                    }
                }
            } else if self.starts_with("<page-styles") {
                props.page_styles = self.parse_page_styles();
            } else if self.starts_with("<meta") {
                props.metadata = self.parse_meta();
            } else {
                let _ = self.parse_node();
            }
        }

        props
    }

    pub(crate) fn parse_paper(&mut self) -> PaperConfig {
        let mut config = PaperConfig::default();
        if let Ok((_tag, attrs, _)) = self.parse_start_tag() {
            if let Some(size) = attrs.get("size") {
                config.size = size.clone();
            }
            if let Some(orientation) = attrs.get("orientation") {
                config.orientation = orientation.clone();
            }
            if let Some(margin) = attrs.get("margin") {
                config.margin = parse_length(margin);
            }
            if let Some(distance) = attrs.get("header-distance") {
                config.header_distance = Some(parse_length(distance));
            }
            if let Some(distance) = attrs.get("footer-distance") {
                config.footer_distance = Some(parse_length(distance));
            }
        }
        config
    }

    pub(crate) fn parse_meta(&mut self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        if let Ok((tag, _, is_self)) = self.parse_start_tag() {
            if tag != "meta" || is_self {
                return map;
            }
        }

        while self.pos < self.chars.len() {
            self.skip_whitespace();
            if self.starts_with("</meta>") {
                self.advance_str("</meta>");
                break;
            }
            if let Some(el) = self.parse_element() {
                let text = el
                    .children
                    .iter()
                    .filter_map(|c| {
                        if let Node::Text(t) = c {
                            Some(t.content.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("");
                map.insert(el.tag, text.trim().to_string());
            } else {
                self.advance();
            }
        }
        map
    }

    pub(crate) fn parse_page_styles(&mut self) -> HashMap<String, PageStyle> {
        let mut styles = HashMap::new();
        if let Ok((tag, _, is_self)) = self.parse_start_tag() {
            if tag != "page-styles" || is_self {
                return styles;
            }
        }

        while self.pos < self.chars.len() {
            self.skip_whitespace();
            if self.starts_with("</page-styles>") {
                self.advance_str("</page-styles>");
                break;
            }

            if self.starts_with("<page-style") {
                if let Some(ps) = self.parse_single_page_style() {
                    styles.insert(ps.name.clone(), ps);
                }
            } else {
                self.advance();
            }
        }
        styles
    }

    pub(crate) fn parse_single_page_style(&mut self) -> Option<PageStyle> {
        let (tag, attrs, is_self_closing) = self.parse_start_tag().ok()?;
        if tag != "page-style" {
            return None;
        }

        let name = attrs
            .get("name")
            .cloned()
            .unwrap_or_else(|| "default".to_string());
        let is_default = attrs.get("default").map(|v| v == "true").unwrap_or(false);

        let mut page_style = PageStyle {
            name,
            is_default,
            margins: None,
            running_header: None,
            running_footer: None,
            watermark: None,
        };

        if is_self_closing {
            return Some(page_style);
        }

        while self.pos < self.chars.len() {
            self.skip_whitespace();
            if self.starts_with("</page-style>") {
                self.advance_str("</page-style>");
                break;
            }

            if self.starts_with("<running-header") {
                page_style.running_header = Some(self.parse_header_footer("running-header"));
            } else if self.starts_with("<running-footer") {
                page_style.running_footer = Some(self.parse_header_footer("running-footer"));
            } else if self.starts_with("<watermark") {
                if let Ok((_, wattrs, _)) = self.parse_start_tag() {
                    page_style.watermark = Some(WatermarkDef {
                        text: wattrs.get("text").cloned().unwrap_or_default(),
                        opacity: wattrs
                            .get("opacity")
                            .and_then(|v| v.parse().ok())
                            .unwrap_or(0.1),
                        angle_deg: wattrs
                            .get("angle")
                            .and_then(|v| v.parse().ok())
                            .unwrap_or(-45.0),
                    });
                }
            } else {
                self.advance();
            }
        }

        Some(page_style)
    }

    pub(crate) fn parse_header_footer(&mut self, closing_tag: &str) -> HeaderFooterDef {
        let close_str = format!("</{}>", closing_tag);
        let mut def = HeaderFooterDef {
            left: Vec::new(),
            center: Vec::new(),
            right: Vec::new(),
        };

        let _ = self.parse_start_tag();

        while self.pos < self.chars.len() {
            self.skip_whitespace();
            if self.starts_with(&close_str) {
                self.advance_str(&close_str);
                break;
            }

            if self.starts_with("<left>") {
                self.advance_str("<left>");
                def.left = self.collect_until("</left>");
            } else if self.starts_with("<center>") {
                self.advance_str("<center>");
                def.center = self.collect_until("</center>");
            } else if self.starts_with("<right>") {
                self.advance_str("<right>");
                def.right = self.collect_until("</right>");
            } else if let Some(node) = self.parse_node() {
                def.center.push(node);
            } else {
                self.advance();
            }
        }

        def
    }

    pub(crate) fn collect_until(&mut self, close_tag: &str) -> Vec<Node> {
        let mut nodes = Vec::new();
        while self.pos < self.chars.len() {
            self.skip_whitespace();
            if self.starts_with(close_tag) {
                self.advance_str(close_tag);
                break;
            }
            if let Some(node) = self.parse_node() {
                nodes.push(node);
            } else {
                break;
            }
        }
        nodes
    }

    pub(crate) fn parse_style_tag(&mut self) -> Stylesheet {
        let _ = self.parse_start_tag();
        let start_pos = self.pos;
        let mut css_text = String::new();

        while self.pos < self.chars.len() {
            if self.starts_with("</style>") {
                break;
            }
            if let Some(ch) = self.advance() {
                css_text.push(ch);
            }
        }

        if self.starts_with("</style>") {
            self.advance_str("</style>");
        }

        parse_css(&css_text, start_pos)
    }
}
