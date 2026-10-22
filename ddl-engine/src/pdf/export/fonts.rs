use crate::ast::{Document, ElementNode};
use printpdf::IndirectFontRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontFamily {
    SansSerif,
    Serif,
    Monospace,
}

#[derive(Clone, Copy)]
pub struct FontSet<'a> {
    pub helvetica_reg: &'a IndirectFontRef,
    pub helvetica_bold: &'a IndirectFontRef,
    pub helvetica_italic: &'a IndirectFontRef,
    pub times_reg: &'a IndirectFontRef,
    pub times_bold: &'a IndirectFontRef,
    pub times_italic: &'a IndirectFontRef,
    pub courier: &'a IndirectFontRef,
}

impl<'a> FontSet<'a> {
    pub fn select(&self, family: FontFamily, bold: bool, italic: bool) -> &'a IndirectFontRef {
        match family {
            FontFamily::Monospace => self.courier,
            FontFamily::Serif => {
                if bold {
                    self.times_bold
                } else if italic {
                    self.times_italic
                } else {
                    self.times_reg
                }
            }
            FontFamily::SansSerif => {
                if bold {
                    self.helvetica_bold
                } else if italic {
                    self.helvetica_italic
                } else {
                    self.helvetica_reg
                }
            }
        }
    }
}

pub fn detect_font_family(el: &ElementNode, doc: &Document) -> FontFamily {
    if let Some(ff) = el.inline_styles.get("font-family") {
        let ffl = ff.to_lowercase();
        if ffl.contains("serif") || ffl.contains("times") || ffl.contains("georgia") {
            return FontFamily::Serif;
        }
        if ffl.contains("mono") || ffl.contains("consolas") || ffl.contains("courier") {
            return FontFamily::Monospace;
        }
    }
    for rule in &doc.props.stylesheet.rules {
        if rule.selectors.iter().any(|s| s == "body" || s == &el.tag) {
            if let Some(ff) = rule.declarations.get("font-family") {
                let ffl = ff.to_lowercase();
                if ffl.contains("serif") || ffl.contains("times") || ffl.contains("georgia") {
                    return FontFamily::Serif;
                }
                if ffl.contains("mono") || ffl.contains("consolas") || ffl.contains("courier") {
                    return FontFamily::Monospace;
                }
            }
        }
    }
    FontFamily::SansSerif
}

pub fn char_width(ch: char, family: FontFamily, bold: bool) -> f32 {
    match family {
        FontFamily::Monospace => 600.0 / 1000.0,
        FontFamily::Serif => {
            let w = if bold {
                match ch {
                    ' ' => 250.0,
                    'i' | 'l' | '!' | '.' | ',' | ':' | ';' | '\'' | '`' => 278.0,
                    'j' | 'r' | 't' | 'f' | '-' | '(' | ')' | '[' | ']' => 333.0,
                    's' => 444.0,
                    'a' | 'c' | 'e' | 'o' | 'u' | 'v' | 'x' | 'z' => 500.0,
                    'b' | 'd' | 'g' | 'h' | 'k' | 'n' | 'p' | 'q' | 'y' | '0'..='9' => 556.0,
                    'w' => 722.0,
                    'm' => 833.0,
                    'I' => 333.0,
                    'J' => 444.0,
                    'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'K' | 'N' | 'O' | 'P' | 'Q'
                    | 'R' | 'T' | 'U' | 'V' | 'X' | 'Y' | 'Z' => 667.0,
                    'M' | 'W' => 889.0,
                    _ => 500.0,
                }
            } else {
                match ch {
                    ' ' => 250.0,
                    'i' | 'l' | '!' | '.' | ',' | ':' | ';' | '\'' | '`' => 250.0,
                    'j' | 'r' | 't' | 'f' | '-' | '(' | ')' | '[' | ']' => 333.0,
                    's' => 389.0,
                    'a' | 'c' | 'e' | 'o' | 'u' | 'v' | 'x' | 'z' => 444.0,
                    'b' | 'd' | 'g' | 'h' | 'k' | 'n' | 'p' | 'q' | 'y' | '0'..='9' => 500.0,
                    'w' => 722.0,
                    'm' => 778.0,
                    'I' => 333.0,
                    'J' => 389.0,
                    'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'K' | 'N' | 'O' | 'P' | 'Q'
                    | 'R' | 'T' | 'U' | 'V' | 'X' | 'Y' | 'Z' => 667.0,
                    'M' | 'W' => 889.0,
                    _ => 500.0,
                }
            };
            w / 1000.0
        }
        FontFamily::SansSerif => {
            let w = if bold {
                match ch {
                    ' ' => 278.0,
                    '!' | ',' | '.' | ':' | ';' | 'i' | 'l' | '\'' | '`' => 300.0,
                    'j' | 'r' | 't' | 'f' | '-' | '(' | ')' | '[' | ']' => 389.0,
                    's' => 556.0,
                    'a'
                    | 'b'
                    | 'c'
                    | 'd'
                    | 'e'
                    | 'g'
                    | 'h'
                    | 'k'
                    | 'n'
                    | 'o'
                    | 'p'
                    | 'q'
                    | 'u'
                    | 'v'
                    | 'x'
                    | 'y'
                    | 'z'
                    | '0'..='9' => 611.0,
                    'w' => 778.0,
                    'm' => 889.0,
                    'I' => 333.0,
                    'J' => 556.0,
                    'L' => 611.0,
                    'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'K' | 'N' | 'O' | 'P' | 'Q'
                    | 'R' | 'T' | 'U' | 'V' | 'X' | 'Y' | 'Z' => 722.0,
                    'M' | 'W' => 944.0,
                    _ => 611.0,
                }
            } else {
                match ch {
                    ' ' => 278.0,
                    '!' | ',' | '.' | ':' | ';' | 'i' | 'l' | '\'' | '`' => 278.0,
                    'j' | 'r' | 't' | 'f' | '-' | '(' | ')' | '[' | ']' => 333.0,
                    's' => 500.0,
                    'a'
                    | 'b'
                    | 'c'
                    | 'd'
                    | 'e'
                    | 'g'
                    | 'h'
                    | 'k'
                    | 'n'
                    | 'o'
                    | 'p'
                    | 'q'
                    | 'u'
                    | 'v'
                    | 'x'
                    | 'y'
                    | 'z'
                    | '0'..='9' => 556.0,
                    'w' => 722.0,
                    'm' => 833.0,
                    'I' => 278.0,
                    'J' => 500.0,
                    'L' => 556.0,
                    'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'K' | 'N' | 'O' | 'P' | 'Q'
                    | 'R' | 'T' | 'U' | 'V' | 'X' | 'Y' | 'Z' => 667.0,
                    'M' | 'W' => 833.0,
                    _ => 556.0,
                }
            };
            w / 1000.0
        }
    }
}

pub fn measure_text(text: &str, font_size: f32, family: FontFamily, bold: bool) -> f32 {
    let pt_to_mm = 0.352778;
    if family == FontFamily::Monospace {
        (text.chars().count() as f32) * 0.60 * font_size * pt_to_mm
    } else {
        let em_sum: f32 = text.chars().map(|c| char_width(c, family, bold)).sum();
        em_sum * font_size * pt_to_mm
    }
}
