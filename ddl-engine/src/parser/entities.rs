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
