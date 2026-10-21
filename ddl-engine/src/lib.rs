pub mod ast;
pub mod parser;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn parse_ddl_to_json(input: &str) -> Result<String, JsValue> {
    let mut parser = parser::Parser::new(input);
    match parser.parse() {
        Ok(doc) => serde_json::to_string(&doc).map_err(|e| JsValue::from_str(&e.to_string())),
        Err(diags) => {
            let err_msg = diags.iter().map(|d| d.message.clone()).collect::<Vec<_>>().join(", ");
            Err(JsValue::from_str(&err_msg))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parse() {
        let input = r#"
        <doc>
          <props>
            <meta>
              <title>Test Doc</title>
            </meta>
            <paper size="A4" orientation="portrait" margin="20mm" />
            <style>
              h1 { color: #1e3a8a; }
              .callout { background: #f0fdf4; }
            </style>
          </props>
          <body>
            <section page-style="cover">
              <h1>Title</h1>
              <p>Hello World</p>
            </section>
          </body>
        </doc>
        "#;

        let mut parser = parser::Parser::new(input);
        let doc = parser.parse().expect("Failed to parse");
        assert_eq!(doc.props.paper.size, "A4");
        assert_eq!(doc.props.stylesheet.rules.len(), 2);
        assert_eq!(doc.body.children.len(), 1);
    }
}
