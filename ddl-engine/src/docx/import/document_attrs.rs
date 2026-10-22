use quick_xml::events::BytesStart;

pub fn attr_val(e: &BytesStart) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == b"w:val" {
            if let Ok(v) = std::str::from_utf8(&attr.value) {
                return Some(v.to_string());
            }
        }
    }
    None
}

pub fn parse_pg_sz(
    e: &BytesStart,
    paper_orientation: &mut String,
    paper_size: &mut String,
) {
    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == b"w:orient" {
            if let Ok(val) = std::str::from_utf8(&attr.value) {
                *paper_orientation = val.to_string();
            }
        } else if attr.key.as_ref() == b"w:w" {
            if let Ok(w_str) = std::str::from_utf8(&attr.value) {
                if let Ok(w_val) = w_str.parse::<u32>() {
                    if (12200..=12300).contains(&w_val) {
                        *paper_size = "Letter".to_string();
                    }
                }
            }
        }
    }
}

pub fn parse_pg_mar(
    e: &BytesStart,
    paper_margin_mm: &mut f32,
    header_distance_mm: &mut Option<f32>,
    footer_distance_mm: &mut Option<f32>,
) {
    for attr in e.attributes().flatten() {
        if let Ok(m_str) = std::str::from_utf8(&attr.value) {
            if let Ok(m_val) = m_str.parse::<f32>() {
                let mm = (m_val * 25.4 / 1440.0 * 10.0).round() / 10.0;
                match attr.key.as_ref() {
                    b"w:left" => {
                        if mm > 0.0 && mm <= 60.0 {
                            *paper_margin_mm = mm;
                        }
                    }
                    b"w:top" => {
                        if *paper_margin_mm == 20.0 && mm > 0.0 && mm <= 35.0 {
                            *paper_margin_mm = mm;
                        }
                    }
                    b"w:header" if mm >= 0.0 => *header_distance_mm = Some(mm),
                    b"w:footer" if mm >= 0.0 => *footer_distance_mm = Some(mm),
                    _ => {}
                }
            }
        }
    }
}

pub fn parse_spacing(
    e: &BytesStart,
    before: &mut Option<u32>,
    after: &mut Option<u32>,
    line: &mut Option<u32>,
) {
    for attr in e.attributes().flatten() {
        let value = std::str::from_utf8(&attr.value)
            .ok()
            .and_then(|v| v.parse::<u32>().ok());
        match attr.key.as_ref() {
            b"w:before" => *before = value,
            b"w:after" => *after = value,
            b"w:line" => *line = value,
            _ => {}
        }
    }
}

pub fn parse_indent(
    e: &BytesStart,
    left: &mut Option<u32>,
    right: &mut Option<u32>,
    first_line: &mut Option<i32>,
) {
    for attr in e.attributes().flatten() {
        let value = std::str::from_utf8(&attr.value)
            .ok()
            .and_then(|v| v.parse::<i32>().ok());
        match attr.key.as_ref() {
            b"w:left" | b"w:start" => *left = value.and_then(|v| u32::try_from(v).ok()),
            b"w:right" | b"w:end" => *right = value.and_then(|v| u32::try_from(v).ok()),
            b"w:firstLine" => *first_line = value,
            b"w:hanging" => *first_line = value.map(|v| -v),
            _ => {}
        }
    }
}
