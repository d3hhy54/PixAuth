use axum::{Json, Router, routing::get};
use serde::{Deserialize, Serialize};

use anyhow::Result;

use crate::{crypto::CryptoEngine, database::DatabaseFactory};

mod crypto;
mod database;

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
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    #[allow(unused)]
    let crypto = CryptoEngine::init().unwrap();
    #[allow(unused)]
    let db = DatabaseFactory::init().await?;

    let app = Router::new().route("/", get(handler));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
