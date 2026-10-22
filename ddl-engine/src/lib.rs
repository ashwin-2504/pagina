pub mod ast;
pub mod docx;
pub mod parser;
pub mod pdf;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub dxml: String,
    pub dcss: String,
    #[serde(default)]
    pub assets: HashMap<String, String>,
}

#[wasm_bindgen]
pub fn parse_ddl_to_json(input: &str) -> Result<String, JsValue> {
    let mut parser = parser::Parser::new(input);
    match parser.parse() {
        Ok(doc) => serde_json::to_string(&doc).map_err(|e| JsValue::from_str(&e.to_string())),
        Err(diags) => {
            let err_msg = diags
                .iter()
                .map(|d| d.message.clone())
                .collect::<Vec<_>>()
                .join(", ");
            Err(JsValue::from_str(&err_msg))
        }
    }
}
