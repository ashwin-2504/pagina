use axum::{
    routing::{get, post},
    Json, Router,
};
use ddl_engine::parser::Parser;
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

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(|| async { "DDL Engine Backend Operational" }))
        .route("/api/parse", post(parse_handler))
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
