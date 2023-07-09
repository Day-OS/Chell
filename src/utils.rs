#[derive(Debug)]
pub enum Error{
    MeiliSearchError(meilisearch_sdk::errors::Error),
    ChannelIsNSFW,
    PrivateChannelUserIsNotOwner,
    Generic,
    NoTopicsFound,
    NoMemoriesToBeSaved,
    NoMemoriesFound,
    NoMessagesFound,
    NoReferencedMessageFound(String)
}



pub enum DBIndexes{
  RawMessage,
  InputMemory
}

impl DBIndexes {
  pub fn as_str(&self) -> &'static str{
    match self {
      DBIndexes::RawMessage => "messages",
      DBIndexes::InputMemory => "memories"
    }
  }
}

pub fn get_database_client()-> meilisearch_sdk::Client{
  meilisearch_sdk::Client::new( env!("MEILI_SERVER_HOST"), Some(env!("MEILI_MASTER_KEY")))
}