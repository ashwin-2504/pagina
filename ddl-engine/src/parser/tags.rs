use super::Parser;
use std::collections::HashMap;

impl<'a> Parser<'a> {
    pub(crate) fn parse_start_tag(&mut self) -> Result<(String, HashMap<String, String>, bool), ()> {
        if self.current() != Some('<') {
            return Err(());
        }
        self.advance();

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
