mod db;

use axum::{Json, response::IntoResponse, Router, routing::post, extract::State, http::{header, StatusCode}};
use mongodb::{Client, Database};
use serde::Deserialize;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // TODO: from env vars or something
    let mongo_url = "mongodb://root:password@localhost:27017/";
    let client = Client::with_uri_str(mongo_url).await?;
    let database = client.database("quotebox");
    
    let app = Router::new()
        .route(
            "/",
            post(create_quote),
        ).with_state(database);

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
) -> impl IntoResponse {
    let collection = database.collection::<db::Quote>("quotes");
    
    let document = db::Quote {
        content,
        author_id,
        tags: tags.unwrap_or_else(|| vec![]),
    };
    // TODO: No unwrap
    let res = collection.insert_one(document, None).await.unwrap();


    // TODO: Can I avoid the to_string?
    (
        StatusCode::CREATED,
        [
            (header::LOCATION, res.inserted_id.to_string())
        ]
    )
}
