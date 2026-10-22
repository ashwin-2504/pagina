use super::css::parse_inline_styles;
use super::entities::decode_xml_entities;
use super::Parser;
use crate::ast::*;
use std::collections::HashMap;

impl<'a> Parser<'a> {
    pub(crate) fn parse_body_block(&mut self) -> BodyNode {
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

    pub(crate) fn parse_comment(&mut self) -> CommentNode {
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

    pub(crate) fn read_text_up_to_tag(&mut self) -> String {
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

    pub(crate) fn parse_section(&mut self) -> Option<SectionNode> {
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

    pub(crate) fn parse_element(&mut self) -> Option<ElementNode> {
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
}
