use serde::{Serialize, Deserialize};
use serenity::model::Timestamp;
#[derive(Serialize, Deserialize, Debug)]
pub struct RawMessage {
  pub id: u64,
  pub user: u64,
  pub user_name: String,
  pub timestamp: i64,
  pub user_image: String,
  pub message: String,
}

pub enum DBIndexes{
  RawMessage
}

impl DBIndexes {
  pub fn as_str(&self) -> &'static str{
    match self {
      DBIndexes::RawMessage => "messages"
    }
  }
}

pub fn get_database_client()-> meilisearch_sdk::Client{
  meilisearch_sdk::Client::new( env!("MEILI_SERVER_HOST"), Some(env!("MEILI_MASTER_KEY")))
}

