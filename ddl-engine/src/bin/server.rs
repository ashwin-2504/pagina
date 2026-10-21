use axum::{
    body::Bytes,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use ddl_engine::docx::{export_docx, import_docx};
use ddl_engine::parser::Parser;
use ddl_engine::pdf::{export_pdf, import_pdf};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

#[derive(Deserialize)]
struct ParseRequest {
    ddl: String,
}

#[derive(Serialize)]
struct ParseResponse {
    success: bool,
    document: Option<serde_json::Value>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct ExportRequest {
    ddl: String,
}

#[derive(Serialize)]
struct ImportResponse {
    success: bool,
    dxml: Option<String>,
    dcss: Option<String>,
    error: Option<String>,
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .expose_headers([header::CONTENT_DISPOSITION]);

    let app = Router::new()
        .route("/health", get(|| async { "DDL Engine Backend Operational" }))
        .route("/api/parse", post(parse_handler))
        .route("/api/export/pdf", post(export_pdf_handler))
        .route("/api/export/docx", post(export_docx_handler))
        .route("/api/import/docx", post(import_docx_handler))
        .route("/api/import/pdf", post(import_pdf_handler))
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    println!(">>> DDL Rust Backend Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn parse_handler(Json(payload): Json<ParseRequest>) -> Json<ParseResponse> {
    let mut parser = Parser::new(&payload.ddl);
    match parser.parse() {
        Ok(doc) => {
            let doc_value = serde_json::to_value(&doc).unwrap();
            Json(ParseResponse {
                success: true,
                document: Some(doc_value),
                error: None,
            })
        }
        Err(diags) => {
            let err_msg = diags
                .iter()
                .map(|d| d.message.clone())
                .collect::<Vec<_>>()
                .join(", ");
            Json(ParseResponse {
                success: false,
                document: None,
                error: Some(if err_msg.is_empty() {
                    "Syntax parse error".to_string()
                } else {
                    err_msg
                }),
            })
        }
    }
}

async fn export_pdf_handler(Json(payload): Json<ExportRequest>) -> Result<Response, (StatusCode, String)> {
    let mut parser = Parser::new(&payload.ddl);
    let doc = parser
        .parse()
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Parse error: {:?}", e)))?;

    let pdf_bytes = export_pdf(&doc)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("PDF export error: {}", e)))?;

    Ok((
        [
            (header::CONTENT_TYPE, "application/pdf"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"document.pdf\"",
            ),
        ],
        pdf_bytes,
    )
        .into_response())
}

async fn export_docx_handler(Json(payload): Json<ExportRequest>) -> Result<Response, (StatusCode, String)> {
    let mut parser = Parser::new(&payload.ddl);
    let doc = parser
        .parse()
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Parse error: {:?}", e)))?;

    let docx_bytes = export_docx(&doc)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DOCX export error: {}", e)))?;

    Ok((
        [
            (
                header::CONTENT_TYPE,
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            ),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"document.docx\"",
            ),
        ],
        docx_bytes,
    )
        .into_response())
}

async fn import_docx_handler(body: Bytes) -> Result<Json<ImportResponse>, (StatusCode, String)> {
    if body.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Empty file uploaded".to_string()));
    }

    match import_docx(&body) {
        Ok(res) => Ok(Json(ImportResponse {
            success: true,
            dxml: Some(res.dxml),
            dcss: Some(res.dcss),
            error: None,
        })),
        Err(e) => Ok(Json(ImportResponse {
            success: false,
            dxml: None,
            dcss: None,
            error: Some(format!("Failed to parse DOCX: {}", e)),
        })),
    }
}

async fn import_pdf_handler(body: Bytes) -> Result<Json<ImportResponse>, (StatusCode, String)> {
    if body.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Empty file uploaded".to_string()));
    }

    match import_pdf(&body) {
        Ok(res) => Ok(Json(ImportResponse {
            success: true,
            dxml: Some(res.dxml),
            dcss: Some(res.dcss),
            error: None,
        })),
        Err(e) => Ok(Json(ImportResponse {
            success: false,
            dxml: None,
            dcss: None,
            error: Some(format!("Failed to parse PDF: {}", e)),
        })),
    }
}
