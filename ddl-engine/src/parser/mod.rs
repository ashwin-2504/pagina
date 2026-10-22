pub mod body;
pub mod css;
pub mod entities;
pub mod props;
pub mod tags;

pub use entities::decode_xml_entities;

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
    pub(crate) chars: Vec<char>,
    pub(crate) pos: usize,
    pub(crate) line: usize,
    pub(crate) col: usize,
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

    pub(crate) fn current(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    pub(crate) fn peek(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    pub(crate) fn advance(&mut self) -> Option<char> {
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

    pub(crate) fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    pub(crate) fn current_span(&self) -> Span {
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

        if self.starts_with("<doc") {
            self.parse_doc_wrapper(&mut props, &mut body)?;
        } else {
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

    pub(crate) fn starts_with(&self, s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        for (i, &ch) in chars.iter().enumerate() {
            if self.peek(i) != Some(ch) {
                return false;
            }
        }
        true
    }

    pub(crate) fn parse_doc_wrapper(
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
            } else if let Some(node) = self.parse_node() {
                body.children.push(node);
            } else {
                break;
            }
        }
        Ok(())
    }

    pub(crate) fn parse_loose_nodes(&mut self, props: &mut DocumentProps, body: &mut BodyNode) {
        while self.pos < self.chars.len() {
            self.skip_whitespace();
            if self.starts_with("<props") {
                *props = self.parse_props_block();
            } else if self.starts_with("<body") {
                *body = self.parse_body_block();
            } else if let Some(node) = self.parse_node() {
                body.children.push(node);
            } else {
                break;
            }
        }
    }

    pub(crate) fn advance_str(&mut self, s: &str) {
        for _ in 0..s.chars().count() {
            self.advance();
        }
    }
}
