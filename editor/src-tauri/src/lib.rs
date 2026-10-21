use base64::Engine;
use ddl_engine::{docx, parser, pdf, ImportResult};

#[tauri::command]
fn ping_engine() -> &'static str {
    "native-tauri"
}

#[tauri::command]
fn parse_ddl(ddl: String) -> Result<String, String> {
    let mut p = parser::Parser::new(&ddl);
    match p.parse() {
        Ok(doc) => serde_json::to_string(&doc).map_err(|e| e.to_string()),
        Err(diags) => {
            let err_msg = diags
                .iter()
                .map(|d| d.message.clone())
                .collect::<Vec<_>>()
                .join(", ");
            Err(err_msg)
        }
    }
}

#[tauri::command]
fn export_pdf(ddl: String) -> Result<String, String> {
    let mut p = parser::Parser::new(&ddl);
    let doc = p.parse().map_err(|diags| {
        diags
            .iter()
            .map(|d| d.message.clone())
            .collect::<Vec<_>>()
            .join(", ")
    })?;

    let pdf_bytes = pdf::export_pdf(&doc).map_err(|e| e.to_string())?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&pdf_bytes))
}

#[tauri::command]
fn export_docx(ddl: String) -> Result<String, String> {
    let mut p = parser::Parser::new(&ddl);
    let doc = p.parse().map_err(|diags| {
        diags
            .iter()
            .map(|d| d.message.clone())
            .collect::<Vec<_>>()
            .join(", ")
    })?;

    let docx_bytes = docx::export_docx(&doc).map_err(|e| e.to_string())?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&docx_bytes))
}

#[tauri::command]
fn import_docx(bytes_base64: String) -> Result<ImportResult, String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&bytes_base64)
        .map_err(|e| format!("Base64 decode error: {e}"))?;

    docx::import_docx(&bytes).map_err(|e| e.to_string())
}

#[tauri::command]
fn import_pdf(bytes_base64: String) -> Result<ImportResult, String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&bytes_base64)
        .map_err(|e| format!("Base64 decode error: {e}"))?;

    pdf::import_pdf(&bytes).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ping_engine,
            parse_ddl,
            export_pdf,
            export_docx,
            import_docx,
            import_pdf
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
