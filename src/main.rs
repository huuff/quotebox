mod db;
mod error;

use itertools::Itertools as _;
use axum::{Json, response::IntoResponse, Router, routing::{post, get}, extract::State, http::{header, StatusCode}};
use error::AppError;
use futures::TryStreamExt;
use mongodb::{Client, Database, bson::{Bson, doc}};
use serde::{Deserialize, Serialize};
use std::str;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // TODO: from env vars or something
    let mongo_url = "mongodb://root:password@localhost:27017/";
    let client = Client::with_uri_str(mongo_url).await?;
    let database = client.database("quotebox");
    
    let app = Router::new()
        .route("/quotes", post(create_quote))
        .route("/quotes", get(get_all_quotes))
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
        _id: None,
        content,
        author_id,
        tags: tags.unwrap_or_else(|| vec![]),
    };

    if let Bson::ObjectId(inserted_id) = collection.insert_one(document, None).await?.inserted_id {
        Ok((StatusCode::CREATED, [(header::LOCATION, inserted_id.to_hex())]).into_response())
    } else {
        Ok((StatusCode::INTERNAL_SERVER_ERROR).into_response())
    }
}

#[derive(Serialize)]
struct QuoteGetResponse {
    pub id: String,
    pub content: String,
    pub tags: Vec<String>,
    pub author_id: Option<String>,
}

async fn get_all_quotes(
    State(database): State<Database>,
) -> Result<impl IntoResponse, AppError> {
    let collection = database.collection::<db::Quote>("quotes");

    // TODO: Can I just map as I retrieve instead of collecting before mapping?
    let documents: Vec<db::Quote> = collection.find(doc!{}, None).await?
        .try_collect()
        .await?
        ;

    let response = documents.into_iter()
        .map(|db::Quote { _id, content, tags, author_id, .. }|
        QuoteGetResponse {
            id: _id.unwrap().to_hex(),
            content, tags, author_id 
        })
        .collect_vec()
        ;


    Ok(Json(response))
}
