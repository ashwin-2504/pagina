use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;

pub fn parse_relationships(xml_str: &str) -> (HashMap<String, String>, HashMap<String, String>) {
    let mut id_map = HashMap::new();
    let mut type_map = HashMap::new();
    let mut reader = Reader::from_str(xml_str);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                if e.name().as_ref() == b"Relationship" {
                    let mut id = String::new();
                    let mut target = String::new();
                    let mut rel_type = String::new();
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"Id" {
                            if let Ok(v) = std::str::from_utf8(&attr.value) {
                                id = v.to_string();
                            }
                        } else if attr.key.as_ref() == b"Target" {
                            if let Ok(v) = std::str::from_utf8(&attr.value) {
                                target = v.to_string();
                            }
                        } else if attr.key.as_ref() == b"Type" {
                            if let Ok(v) = std::str::from_utf8(&attr.value) {
                                rel_type = v.to_string();
                            }
                        }
                    }
                    if !id.is_empty() && !target.is_empty() {
                        id_map.insert(id, target.clone());
                    }
                    if !rel_type.is_empty() && !target.is_empty() {
                        let short_type = rel_type
                            .split('/')
                            .last()
                            .unwrap_or(&rel_type)
                            .to_lowercase();
                        type_map.insert(short_type, target);
                    }
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    (id_map, type_map)
}

pub fn extract_pt_from_style(style: &str, key: &str) -> Option<f32> {
    if let Some(pos) = style.find(key) {
        let after = &style[pos + key.len()..];
        let num_str: String = after
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        return num_str.parse::<f32>().ok();
    }
    None
}
