pub fn decode_xml_entities(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '&' {
            let mut entity = String::new();
            while let Some(&next_ch) = chars.peek() {
                if next_ch == ';' {
                    chars.next();
                    break;
                }
                if next_ch.is_whitespace() || entity.len() > 10 {
                    break;
                }
                entity.push(chars.next().unwrap());
            }
            match entity.as_str() {
                "lt" => result.push('<'),
                "gt" => result.push('>'),
                "amp" => result.push('&'),
                "quot" => result.push('"'),
                "apos" => result.push('\''),
                "nbsp" => result.push(' '),
                _ if entity.starts_with('#') => {
                    let code_res = if entity.starts_with("#x") || entity.starts_with("#X") {
                        u32::from_str_radix(&entity[2..], 16)
                    } else {
                        entity[1..].parse::<u32>()
                    };
                    if let Ok(code) = code_res {
                        if let Some(c) = std::char::from_u32(code) {
                            result.push(c);
                        } else {
                            result.push('&');
                            result.push_str(&entity);
                            result.push(';');
                        }
                    } else {
                        result.push('&');
                        result.push_str(&entity);
                        result.push(';');
                    }
                }
                _ => {
                    result.push('&');
                    result.push_str(&entity);
                    result.push(';');
                }
            }
        } else {
            result.push(ch);
        }
    }
    result
}

use crate::ast::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub message: String,
    pub span: Span,
    pub severity: DiagnosticSeverity,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

pub struct Parser<'a> {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    pub diagnostics: Vec<Diagnostic>,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            diagnostics: Vec::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    fn current(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.current() {
            self.pos += 1;
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn current_span(&self) -> Span {
        Span {
            start: self.pos,
            end: self.pos,
            line: self.line,
            col: self.col,
        }
    }

    pub fn parse(&mut self) -> Result<Document, Vec<Diagnostic>> {
        self.skip_whitespace();
        let doc_span = self.current_span();

        let mut props = DocumentProps::default();
        let mut body = BodyNode {
            attributes: HashMap::new(),
            children: Vec::new(),
            span: doc_span.clone(),
        };

        // Expect <doc>
        if self.starts_with("<doc") {
            self.parse_doc_wrapper(&mut props, &mut body)?;
        } else {
            // Fault-tolerant: if <doc> is omitted, parse props and nodes directly
            self.parse_loose_nodes(&mut props, &mut body);
        }

        Ok(Document {
            props,
            body,
            span: Span {
                start: doc_span.start,
                end: self.pos,
                line: doc_span.line,
                col: doc_span.col,
            },
        })
    }

    fn starts_with(&self, s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        for (i, &ch) in chars.iter().enumerate() {
            if self.peek(i) != Some(ch) {
                return false;
            }
        }
        true
    }

    fn parse_doc_wrapper(
        &mut self,
        props: &mut DocumentProps,
        body: &mut BodyNode,
    ) -> Result<(), Vec<Diagnostic>> {
        let (tag, _attrs, _) = self
            .parse_start_tag()
            .map_err(|_| self.diagnostics.clone())?;
        if tag != "doc" {
            return Ok(());
        }

        while self.pos < self.chars.len() {
            self.skip_whitespace();
            if self.starts_with("</doc>") {
                self.advance_str("</doc>");
                break;
            }
            if self.starts_with("<props") {
                *props = self.parse_props_block();
            } else if self.starts_with("<body") {
                *body = self.parse_body_block();
            } else {
                // Any node outside explicit <body> gets grouped into body
                if let Some(node) = self.parse_node() {
                    body.children.push(node);
                } else {
                    break;
                }
            }
        }
        Ok(())
    }

    fn parse_loose_nodes(&mut self, props: &mut DocumentProps, body: &mut BodyNode) {
        while self.pos < self.chars.len() {
            self.skip_whitespace();
            if self.starts_with("<props") {
                *props = self.parse_props_block();
            } else if self.starts_with("<body") {
                *body = self.parse_body_block();
            } else {
                if let Some(node) = self.parse_node() {
                    body.children.push(node);
                } else {
                    break;
                }
            }
        }
    }

    fn advance_str(&mut self, s: &str) {
        for _ in 0..s.chars().count() {
            self.advance();
        }
    }

    fn parse_props_block(&mut self) -> DocumentProps {
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
                // skip unknown tag
                let _ = self.parse_node();
            }
        }

        props
    }

    fn parse_paper(&mut self) -> PaperConfig {
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
        }
        config
    }

    fn parse_meta(&mut self) -> HashMap<String, String> {
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

    fn parse_page_styles(&mut self) -> HashMap<String, PageStyle> {
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

    fn parse_single_page_style(&mut self) -> Option<PageStyle> {
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

    fn parse_header_footer(&mut self, closing_tag: &str) -> HeaderFooterDef {
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

    fn collect_until(&mut self, close_tag: &str) -> Vec<Node> {
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

    fn parse_style_tag(&mut self) -> Stylesheet {
        let _ = self.parse_start_tag(); // consume <style>
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

    fn parse_body_block(&mut self) -> BodyNode {
        let start_span = self.current_span();
        let (_tag, attrs, is_self_closing) = match self.parse_start_tag() {
            Ok(res) => res,
            Err(_) => {
                return BodyNode {
                    attributes: HashMap::new(),
                    children: Vec::new(),
                    span: start_span,
                }
            }
        };

        let mut children = Vec::new();
        if !is_self_closing {
            while self.pos < self.chars.len() {
                self.skip_whitespace();
                if self.starts_with("</body>") {
                    self.advance_str("</body>");
                    break;
                }
                if let Some(node) = self.parse_node() {
                    children.push(node);
                } else {
                    break;
                }
            }
        }

        BodyNode {
            attributes: attrs,
            children,
            span: Span {
                start: start_span.start,
                end: self.pos,
                line: start_span.line,
                col: start_span.col,
            },
        }
    }

    pub fn parse_node(&mut self) -> Option<Node> {
        self.skip_whitespace();
        if self.pos >= self.chars.len() {
            return None;
        }

        if self.starts_with("<!--") {
            return Some(Node::Comment(self.parse_comment()));
        }

        if self.starts_with("<section") {
            return self.parse_section().map(Node::Section);
        }

        if self.current() == Some('<') && self.peek(1) != Some('/') {
            return self.parse_element().map(Node::Element);
        }

        // Text Node
        let start_span = self.current_span();
        let mut text = String::new();
        while let Some(ch) = self.current() {
            if ch == '<' {
                break;
            }
            text.push(ch);
            self.advance();
        }

        if text.is_empty() {
            None
        } else {
            Some(Node::Text(TextNode {
                content: text,
                span: Span {
                    start: start_span.start,
                    end: self.pos,
                    line: start_span.line,
                    col: start_span.col,
                },
            }))
        }
    }

    fn parse_comment(&mut self) -> CommentNode {
        let start_span = self.current_span();
        self.advance_str("<!--");
        let mut content = String::new();
        while self.pos < self.chars.len() {
            if self.starts_with("-->") {
                self.advance_str("-->");
                break;
            }
            if let Some(ch) = self.advance() {
                content.push(ch);
            }
        }
        CommentNode {
            content,
            span: Span {
                start: start_span.start,
                end: self.pos,
                line: start_span.line,
                col: start_span.col,
            },
        }
    }

    fn read_text_up_to_tag(&mut self) -> String {
        let mut raw = String::new();
        while let Some(ch) = self.current() {
            if ch == '<' {
                break;
            }
            raw.push(ch);
            self.advance();
        }
        decode_xml_entities(&raw)
    }

    fn parse_section(&mut self) -> Option<SectionNode> {
        let start_span = self.current_span();
        let (tag, attrs, is_self_closing) = self.parse_start_tag().ok()?;
        if tag != "section" {
            return None;
        }

        let page_style = attrs.get("page-style").cloned();
        let page_break = attrs
            .get("page-break")
            .map(|v| v == "true")
            .unwrap_or(false);

        let mut children = Vec::new();
        if !is_self_closing {
            while self.pos < self.chars.len() {
                if self.starts_with("</section>") {
                    self.advance_str("</section>");
                    break;
                }
                if self.starts_with("<!--") {
                    children.push(Node::Comment(self.parse_comment()));
                    continue;
                }
                if self.current() == Some('<') && self.peek(1) != Some('/') {
                    if let Some(el) = self.parse_element() {
                        children.push(Node::Element(el));
                    } else {
                        self.advance();
                    }
                    continue;
                }
                let start_span = self.current_span();
                let text = self.read_text_up_to_tag();
                if !text.trim().is_empty() {
                    children.push(Node::Text(TextNode {
                        content: text,
                        span: Span {
                            start: start_span.start,
                            end: self.pos,
                            line: start_span.line,
                            col: start_span.col,
                        },
                    }));
                }
            }
        }

        Some(SectionNode {
            page_style,
            page_break,
            attributes: attrs,
            children,
            span: Span {
                start: start_span.start,
                end: self.pos,
                line: start_span.line,
                col: start_span.col,
            },
        })
    }

    fn parse_element(&mut self) -> Option<ElementNode> {
        let start_span = self.current_span();
        let (tag, attrs, is_self_closing) = match self.parse_start_tag() {
            Ok(res) => res,
            Err(_) => return None,
        };

        let id = attrs.get("id").cloned();
        let classes = attrs
            .get("class")
            .map(|s| s.split_whitespace().map(|c| c.to_string()).collect())
            .unwrap_or_default();

        let inline_styles = attrs
            .get("style")
            .map(|s| parse_inline_styles(s))
            .unwrap_or_default();

        let mut children = Vec::new();
        let close_tag = format!("</{}>", tag);

        if !is_self_closing {
            let is_block_container = matches!(
                tag.as_str(),
                "table" | "tr" | "grid" | "thead" | "tbody" | "page-styles"
            );
            while self.pos < self.chars.len() {
                if is_block_container {
                    self.skip_whitespace();
                }
                if self.starts_with(&close_tag) {
                    self.advance_str(&close_tag);
                    break;
                }
                if self.starts_with("</") {
                    break;
                }
                if self.starts_with("<!--") {
                    children.push(Node::Comment(self.parse_comment()));
                    continue;
                }
                if self.current() == Some('<') {
                    if let Some(child) = self.parse_element() {
                        children.push(Node::Element(child));
                    } else {
                        self.advance();
                    }
                    continue;
                }

                // Text Node inside element (preserving spaces and decoding XML entities)
                let start_span = self.current_span();
                let text = self.read_text_up_to_tag();
                if !text.is_empty() {
                    if is_block_container && text.trim().is_empty() {
                        continue;
                    }
                    children.push(Node::Text(TextNode {
                        content: text,
                        span: Span {
                            start: start_span.start,
                            end: self.pos,
                            line: start_span.line,
                            col: start_span.col,
                        },
                    }));
                }
            }
        }

        Some(ElementNode {
            tag,
            attributes: attrs,
            id,
            classes,
            inline_styles,
            children,
            span: Span {
                start: start_span.start,
                end: self.pos,
                line: start_span.line,
                col: start_span.col,
            },
        })
    }

    fn parse_start_tag(&mut self) -> Result<(String, HashMap<String, String>, bool), ()> {
        if self.current() != Some('<') {
            return Err(());
        }
        self.advance(); // consume '<'

        self.skip_whitespace();
        let mut tag = String::new();
        while let Some(ch) = self.current() {
            if ch.is_alphanumeric() || ch == '-' || ch == '_' {
                tag.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let mut attrs = HashMap::new();
        loop {
            self.skip_whitespace();
            if self.current() == Some('>') {
                self.advance();
                return Ok((tag, attrs, false));
            }
            if self.starts_with("/>") {
                self.advance_str("/>");
                return Ok((tag, attrs, true));
            }
            if self.pos >= self.chars.len() {
                return Ok((tag, attrs, false));
            }

            // Parse Attribute Name
            let mut attr_name = String::new();
            while let Some(ch) = self.current() {
                if ch.is_alphanumeric() || ch == '-' || ch == '_' {
                    attr_name.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }

            if attr_name.is_empty() {
                self.advance();
                continue;
            }

            self.skip_whitespace();
            let mut attr_val = String::new();
            if self.current() == Some('=') {
                self.advance();
                self.skip_whitespace();
                if let Some(quote) = self.current() {
                    if quote == '"' || quote == '\'' {
                        self.advance();
                        while let Some(val_ch) = self.current() {
                            if val_ch == quote {
                                self.advance();
                                break;
                            }
                            attr_val.push(val_ch);
                            self.advance();
                        }
                    }
                }
            }
            attrs.insert(attr_name, attr_val);
        }
    }
}

// ---------------- CSS Parsing Helper ----------------

fn parse_css(css: &str, base_offset: usize) -> Stylesheet {
    let mut rules = Vec::new();
    let parts = css.split('}');

    for part in parts {
        let clean = part.trim();
        if clean.is_empty() {
            continue;
        }

        if let Some((sel_str, decls_str)) = clean.split_once('{') {
            let selectors = sel_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            let mut declarations = HashMap::new();
            for decl in decls_str.split(';') {
                if let Some((prop, val)) = decl.split_once(':') {
                    let p = prop.trim().to_string();
                    let v = val.trim().to_string();
                    if !p.is_empty() && !v.is_empty() {
                        declarations.insert(p, v);
                    }
                }
            }

            rules.push(StyleRule {
                selectors,
                declarations,
                span: Span {
                    start: base_offset,
                    end: base_offset + clean.len(),
                    line: 1,
                    col: 1,
                },
            });
        }
    }

    Stylesheet { rules }
}

fn parse_inline_styles(style_str: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for decl in style_str.split(';') {
        if let Some((prop, val)) = decl.split_once(':') {
            let p = prop.trim().to_string();
            let v = val.trim().to_string();
            if !p.is_empty() && !v.is_empty() {
                map.insert(p, v);
            }
        }
    }
    map
}

fn parse_length(s: &str) -> Length {
    let s = s.trim();
    if s.ends_with("mm") {
        let val = s.trim_end_matches("mm").trim().parse().unwrap_or(20.0);
        Length {
            val,
            unit: Unit::Mm,
        }
    } else if s.ends_with("pt") {
        let val = s.trim_end_matches("pt").trim().parse().unwrap_or(12.0);
        Length {
            val,
            unit: Unit::Pt,
        }
    } else if s.ends_with("in") {
        let val = s.trim_end_matches("in").trim().parse().unwrap_or(1.0);
        Length {
            val,
            unit: Unit::Inch,
        }
    } else {
        let val = s.parse().unwrap_or(20.0);
        Length {
            val,
            unit: Unit::Mm,
        }
    }
}
