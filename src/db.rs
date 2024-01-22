use mongodb::bson;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Quote {
    pub _id: Option<bson::oid::ObjectId>,
    pub content: String,
    pub author_id: Option<String>,
    pub tags: Vec<String>,
}

