mod db;
mod error;

use axum::{Json, response::IntoResponse, Router, routing::post, extract::State, http::{header, StatusCode}};
use error::AppError;
use mongodb::{Client, Database, bson::Bson};
use serde::Deserialize;
use std::str;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // TODO: from env vars or something
    let mongo_url = "mongodb://root:password@localhost:27017/";
    let client = Client::with_uri_str(mongo_url).await?;
    let database = client.database("quotebox");
    
    let app = Router::new()
        .route("/quotes", post(create_quote))
        .with_state(database)
        ;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Deserialize)]
struct QuotePost {
    pub content: String,
    pub tags: Option<Vec<String>>,
    pub author_id: Option<String>,
}

// TODO: Some validations
async fn create_quote(
    State(database): State<Database>,
    Json(QuotePost { content, tags, author_id }): Json<QuotePost>,
) -> Result<impl IntoResponse, AppError> {
    let collection = database.collection::<db::Quote>("quotes");
    
    let document = db::Quote {
        content,
        author_id,
        tags: tags.unwrap_or_else(|| vec![]),
    };

    if let Bson::ObjectId(inserted_id) = collection.insert_one(document, None).await?.inserted_id {
        Ok((
            StatusCode::CREATED,
            [
                (header::LOCATION, inserted_id.to_hex())
            ]
        ).into_response())
    } else {
        Ok((StatusCode::INTERNAL_SERVER_ERROR).into_response())
    }


}
