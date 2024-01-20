use axum::{Json, response::IntoResponse, Router, routing::post, extract::State};
use mongodb::{Client, Database, bson::Document};
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
    pub tags: Vec<String>,
    pub author_id: Option<String>,
}

async fn create_quote(
    State(database): State<Database>,
    Json(payload): Json<QuotePost>,
) -> impl IntoResponse {
    let collection = database.collection::<Document>("quotes");

    ""
}
