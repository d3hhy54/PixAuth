use axum::{Json, Router, routing::get};
use serde::{Deserialize, Serialize};

use crate::crypto::CryptoEngine;

mod crypto;

#[derive(Deserialize, Serialize)]
struct ClassicAnswer {
    answer: String,
}

async fn handler() -> Json<ClassicAnswer> {
    Json(ClassicAnswer {
        answer: "Hello world!".to_string(),
    })
}

#[tokio::main]
async fn main() {
    CryptoEngine::init().unwrap();
    let app = Router::new().route("/", get(handler));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
