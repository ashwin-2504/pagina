use super::types::{ListFormat, NumberingInfo};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;

pub fn parse_numbering_xml(xml: &str) -> NumberingInfo {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut num_to_abstract = HashMap::new();
    let mut abstract_lvl_format = HashMap::new();

    let mut current_abstract_num_id: Option<String> = None;
    let mut current_ilvl: Option<u32> = None;
    let mut current_num_id: Option<String> = None;
    let mut current_num_fmt: Option<String> = None;
    let mut current_lvl_text: Option<String> = None;

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let name = e.name();
                match name.as_ref() {
                    b"w:abstractNum" => {
                        current_abstract_num_id = None;
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"w:abstractNumId" {
                                if let Ok(val) = std::str::from_utf8(&attr.value) {
                                    current_abstract_num_id = Some(val.to_string());
                                }
                            }
                        }
                    }
                    b"w:lvl" => {
                        current_ilvl = None;
                        current_num_fmt = None;
                        current_lvl_text = None;
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"w:ilvl" {
                                if let Ok(val) = std::str::from_utf8(&attr.value) {
                                    current_ilvl = val.parse::<u32>().ok();
                                }
                            }
                        }
                    }
                    b"w:numFmt" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"w:val" {
                                if let Ok(val) = std::str::from_utf8(&attr.value) {
                                    current_num_fmt = Some(val.to_string());
                                }
                            }
                        }
                    }
                    b"w:lvlText" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"w:val" {
                                if let Ok(val) = std::str::from_utf8(&attr.value) {
                                    current_lvl_text = Some(val.to_string());
                                }
                            }
                        }
                    }
                    b"w:num" => {
                        current_num_id = None;
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"w:numId" {
                                if let Ok(val) = std::str::from_utf8(&attr.value) {
                                    current_num_id = Some(val.to_string());
                                }
                            }
                        }
                    }
                    b"w:abstractNumId" => {
                        if let Some(ref nid) = current_num_id {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"w:val" {
                                    if let Ok(val) = std::str::from_utf8(&attr.value) {
                                        num_to_abstract.insert(nid.clone(), val.to_string());
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                match name.as_ref() {
                    b"w:lvl" => {
                        if let (Some(aid), Some(ilvl)) = (&current_abstract_num_id, current_ilvl) {
                            let fmt = match current_num_fmt.as_deref().unwrap_or("bullet") {
                                "bullet" => {
                                    let b_type = match current_lvl_text.as_deref() {
                                        Some(txt) if txt.contains('o') || txt.contains('◦') => {
                                            "circle"
                                        }
                                        Some(txt)
                                            if txt.contains('▪')
                                                || txt.contains('■')
                                                || txt.contains('§') =>
                                        {
                                            "square"
                                        }
                                        Some(txt) if txt.contains('–') || txt.contains('-') => {
                                            "dash"
                                        }
                                        _ => "disc",
                                    };
                                    ListFormat::Bullet(b_type.to_string())
                                }
                                "decimal" => ListFormat::Number("1".to_string()),
                                "lowerLetter" => ListFormat::Number("a".to_string()),
                                "upperLetter" => ListFormat::Number("A".to_string()),
                                "lowerRoman" => ListFormat::Number("i".to_string()),
                                "upperRoman" => ListFormat::Number("I".to_string()),
                                other => {
                                    if other.to_lowercase().contains("bullet") {
                                        ListFormat::Bullet("disc".to_string())
                                    } else {
                                        ListFormat::Number("1".to_string())
                                    }
                                }
                            };
                            abstract_lvl_format.insert((aid.clone(), ilvl), fmt);
                        }
                        current_ilvl = None;
                        current_num_fmt = None;
                        current_lvl_text = None;
                    }
                    b"w:abstractNum" => {
                        current_abstract_num_id = None;
                    }
                    b"w:num" => {
                        current_num_id = None;
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    NumberingInfo {
        num_to_abstract,
        abstract_lvl_format,
    }
}
