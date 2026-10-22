use super::fonts::{measure_text, FontFamily};
use super::types::{LineSpan, ParagraphWord, StyledRun};
use crate::ast::Node;

pub fn collect_styled_runs(
    node: &Node,
    family: FontFamily,
    bold: bool,
    italic: bool,
    runs: &mut Vec<StyledRun>,
) {
    match node {
        Node::Text(t) => {
            runs.push(StyledRun {
                text: t.content.clone(),
                family,
                bold,
                italic,
            });
        }
        Node::Element(el) => {
            let tag = el.tag.to_lowercase();
            if tag == "ul" || tag == "ol" {
                return;
            }
            let mut b = bold;
            let mut i = italic;
            let mut f = family;
            if tag == "b" || tag == "strong" {
                b = true;
            } else if tag == "i" || tag == "em" {
                i = true;
            } else if tag == "code" {
                f = FontFamily::Monospace;
            } else if tag == "br" {
                runs.push(StyledRun {
                    text: "\n".to_string(),
                    family: f,
                    bold: b,
                    italic: i,
                });
                return;
            }
            for child in &el.children {
                collect_styled_runs(child, f, b, i, runs);
            }
        }
        _ => {}
    }
}

pub fn layout_paragraph_lines(
    runs: &[StyledRun],
    font_size: f32,
    actual_width: f32,
) -> Vec<(Vec<LineSpan>, f32)> {
    let mut cleaned_runs: Vec<StyledRun> = Vec::new();
    let num_runs = runs.len();

    for (idx, run) in runs.iter().enumerate() {
        let mut text = run.text.clone();
        if idx == 0 {
            text = text.trim_start().to_string();
        }
        if idx == num_runs - 1 {
            text = text.trim_end().to_string();
        }
        if !text.is_empty() {
            cleaned_runs.push(StyledRun {
                text,
                family: run.family,
                bold: run.bold,
                italic: run.italic,
            });
        }
    }

    if cleaned_runs.is_empty() {
        return Vec::new();
    }

    let mut words: Vec<ParagraphWord> = Vec::new();
    let space_w = measure_text(" ", font_size, FontFamily::SansSerif, false);

    for run in &cleaned_runs {
        let family = run.family;
        let bold = run.bold;
        let italic = run.italic;
        let mut cur_word = String::new();
        let chars: Vec<char> = run.text.chars().collect();
        let n = chars.len();
        let mut i = 0;

        while i < n {
            let ch = chars[i];
            if ch == '\n' {
                if !cur_word.is_empty() {
                    let w = measure_text(&cur_word, font_size, family, bold);
                    words.push(ParagraphWord {
                        text: cur_word,
                        family,
                        bold,
                        italic,
                        has_trailing_space: false,
                        width_mm: w,
                    });
                    cur_word = String::new();
                }
                words.push(ParagraphWord {
                    text: "\n".to_string(),
                    family,
                    bold,
                    italic,
                    has_trailing_space: false,
                    width_mm: 0.0,
                });
                i += 1;
            } else if ch.is_whitespace() {
                if !cur_word.is_empty() {
                    let w = measure_text(&cur_word, font_size, family, bold);
                    words.push(ParagraphWord {
                        text: cur_word,
                        family,
                        bold,
                        italic,
                        has_trailing_space: true,
                        width_mm: w,
                    });
                    cur_word = String::new();
                } else if let Some(last) = words.last_mut() {
                    last.has_trailing_space = true;
                }
                while i < n && chars[i].is_whitespace() && chars[i] != '\n' {
                    i += 1;
                }
            } else if (ch == ',' || ch == '.' || ch == ':' || ch == ';' || ch == '!' || ch == '?')
                && cur_word.is_empty()
            {
                if let Some(last) = words.last_mut() {
                    if !last.has_trailing_space {
                        last.text.push(ch);
                        last.width_mm = measure_text(&last.text, font_size, last.family, last.bold);
                        i += 1;
                        continue;
                    }
                }
                cur_word.push(ch);
                i += 1;
            } else {
                cur_word.push(ch);
                i += 1;
            }
        }

        if !cur_word.is_empty() {
            let w = measure_text(&cur_word, font_size, family, bold);
            words.push(ParagraphWord {
                text: cur_word,
                family,
                bold,
                italic,
                has_trailing_space: false,
                width_mm: w,
            });
        }
    }

    if words.is_empty() {
        return Vec::new();
    }

    let mut lines: Vec<Vec<ParagraphWord>> = Vec::new();
    let mut cur_line: Vec<ParagraphWord> = Vec::new();
    let mut cur_line_width = 0.0f32;

    for word in words {
        if word.text == "\n" {
            lines.push(cur_line);
            cur_line = Vec::new();
            cur_line_width = 0.0;
            continue;
        }

        let needed = word.width_mm
            + if word.has_trailing_space {
                space_w
            } else {
                0.0
            };
        if !cur_line.is_empty() && cur_line_width + word.width_mm > actual_width {
            lines.push(cur_line);
            cur_line = Vec::new();
            cur_line_width = 0.0;
        }

        cur_line_width += needed;
        cur_line.push(word);
    }

    if !cur_line.is_empty() {
        lines.push(cur_line);
    }

    let mut rendered_lines: Vec<(Vec<LineSpan>, f32)> = Vec::new();

    for line_words in lines {
        if line_words.is_empty() {
            continue;
        }

        let mut spans: Vec<LineSpan> = Vec::new();
        let mut cur_span_text = String::new();
        let mut cur_family = FontFamily::SansSerif;
        let mut cur_bold = false;
        let mut cur_italic = false;
        let num_words = line_words.len();

        for (idx, w) in line_words.iter().enumerate() {
            let is_last_word = idx == num_words - 1;
            let style_matches = !cur_span_text.is_empty()
                && cur_family == w.family
                && cur_bold == w.bold
                && cur_italic == w.italic;

            if style_matches {
                cur_span_text.push_str(&w.text);
                if w.has_trailing_space && !is_last_word {
                    cur_span_text.push(' ');
                }
            } else {
                if !cur_span_text.is_empty() {
                    let span_w = measure_text(&cur_span_text, font_size, cur_family, cur_bold);
                    spans.push(LineSpan {
                        text: cur_span_text,
                        family: cur_family,
                        bold: cur_bold,
                        italic: cur_italic,
                        font_size,
                        width_mm: span_w,
                    });
                }
                cur_span_text = w.text.clone();
                if w.has_trailing_space && !is_last_word {
                    cur_span_text.push(' ');
                }
                cur_family = w.family;
                cur_bold = w.bold;
                cur_italic = w.italic;
            }
        }

        if !cur_span_text.is_empty() {
            let span_w = measure_text(&cur_span_text, font_size, cur_family, cur_bold);
            spans.push(LineSpan {
                text: cur_span_text,
                family: cur_family,
                bold: cur_bold,
                italic: cur_italic,
                font_size,
                width_mm: span_w,
            });
        }

        let total_line_w: f32 = spans.iter().map(|s| s.width_mm).sum();
        rendered_lines.push((spans, total_line_w));
    }

    rendered_lines
}
