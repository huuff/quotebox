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

impl Into<db::Quote> for QuotePost {
    fn into(self) -> db::Quote {
        db::Quote {
            _id: None,
            content: self.content,
            tags: self.tags.unwrap_or_else(|| vec![]),
            author_id: self.author_id,
        }
    }
}

// TODO: Some validations
async fn create_quote(
    State(database): State<Database>,
    Json(quote): Json<QuotePost>,
) -> Result<impl IntoResponse, AppError> {
    let collection = database.collection::<db::Quote>("quotes");

    let quote: db::Quote = quote.into();
    
    if let Bson::ObjectId(inserted_id) = collection.insert_one(quote, None).await?.inserted_id {
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

impl From<db::Quote> for QuoteGetResponse {
    fn from(value: db::Quote) -> Self {
        Self {
            id: value._id.unwrap().to_hex(),
            content: value.content,
            tags: value.tags,
            author_id: value.author_id,
        }
    }
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
        .map(|it| QuoteGetResponse::from(it))
        .collect_vec()
        ;


    Ok(Json(response))
}
