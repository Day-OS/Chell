use futures::future::ok;
use meilisearch_sdk::search::SearchResults;
use serde::{Serialize, Deserialize};
use serenity::{model::{Timestamp, prelude::{Channel, Message}}, http::{Http, self}, Error};
use crate::results::{self, SavedMemories};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InputMemory{
  pub relevancy: bool,
  pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone,)]
pub struct Memory{
  pub id: String,
  pub timestamp: u64,
  pub content: String,
}

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
/**
pub async fn load_memory_from_db(memory_json: String,) -> Result<results::DatabaseResult, results::Error>{

  let mut raw_messages : Vec<Memory> = vec!();
  for message in discord_messages{
      let idstr = message.id.0.to_string();
      let id = idstr.as_str();
      match db_messages.search().with_query(id).execute::<RawMessage>().await{
          Ok(pages)=>{if pages.hits.len() == 0 {add_raw_message(&mut raw_messages, &message)}},
          Err(_)=>{add_raw_message(&mut raw_messages, &message)}
      }
  }

}
 */

pub async fn save_memory_to_db(memory_json: &String, timestamp: u64) -> Result<results::DatabaseResult, results::Error>{
  let input_memory : InputMemory = match serde_json::from_str(&memory_json) {
      Ok(memory)=>{memory}
      Err(_)=>{return Err(results::Error::CouldntConvertToJSON)}
  };
  if !input_memory.relevancy {return Err(results::Error::NothingUsefulToBeSaved)}
  let id: String = input_memory.content.split(" ").flat_map(|s| s.chars().nth(0)).collect(); 
  let memory = Memory{
                          id: id,
                          timestamp:timestamp, 
                          content:input_memory.content
                        };
  let content = memory.clone().content;
  println!("{:?}", memory);
  match get_database_client()
        .index(DBIndexes::InputMemory.as_str())
        .add_documents(&[memory], None)
        .await{
    Ok(o)=>{println!("{:?}", o)}
    Err(o)=>{println!("{}", o)}  
  };
  Ok(results::DatabaseResult::SavedMemories(SavedMemories{memory:content}))
}

pub async fn get_last_n_messages(n: usize) -> String{
  let mut raw_messages_searched = get_database_client()
  .index(DBIndexes::RawMessage.as_str())
  .search()
  .with_filter("message IS NOT EMPTY")
  .with_sort(&["timestamp:desc", "username:desc", "message:desc"])
  .with_limit(n)
  .execute::<RawMessage>()
  .await
  .unwrap();

  raw_messages_searched.hits.sort_by(|a,b| {
    a.result.timestamp.cmp(&b.result.timestamp)
  });
  
  let mut messages: String = "".into();
  for raw_message in raw_messages_searched.hits {
    let raw_message: RawMessage = raw_message.result;
    messages += &format!("ID DA MENSAGEM: {} | TEMPO: {} - {} disse =  {}\n", 
                        raw_message.id,
                        serenity::model::Timestamp::from_unix_timestamp(raw_message.timestamp).unwrap().to_string(), 
                        raw_message.user_name, 
                        raw_message.message
                    );
  }
  messages
}

//
pub async fn save_last_n_messages(http: &Http, chat_id: u64, n: u64) -> Result<results::DatabaseResult, results::Error>{
  let channel: Channel = http.get_channel(chat_id).await.unwrap();
  fn add_raw_message(raw_messages: &mut Vec<RawMessage>, message: &Message){
    let raw_message = RawMessage{
        user_name:message.author.clone().name,
        id: message.id.0,
        timestamp:message.timestamp.unix_timestamp(),
        user: message.author.id.0,
        user_image: message.author.clone().avatar_url().unwrap_or("NO AVATAR".into()),
        message: message.clone().content,
    };
    raw_messages.push(raw_message);
}
  let channel_name;
  let discord_messages: Vec<Message> =  match channel {
      Channel::Guild(channel)=>{
        if channel.nsfw {return Err(results::Error::ChannelIsNSFW)};
        channel_name = channel.name().into();
        channel.messages(&http, |retriever| retriever.limit(n)).await.unwrap()
      }
      Channel::Private(channel)=>{
        //PREVENTS SKETCHY DM WEIRDOS FROM... DATING THE AI
        let owner: u64 = env!("DISCORD_BOT_OWNER").parse::<u64>().unwrap();
        if channel.recipient.id != owner{return Err(results::Error::PrivateChannelUserIsNotOwner)}

        channel_name = channel.clone().recipient.name + "'s DM";
        channel.messages(&http, |retriever| retriever.limit(n)).await.unwrap()
      }
      _=>{return Err(results::Error::Generic)}
  };
  

  let db = get_database_client();
  let db_messages = db.index(DBIndexes::RawMessage.as_str());
  
  let mut raw_messages : Vec<RawMessage> = vec!();
  for message in discord_messages{
      let idstr = message.id.0.to_string();
      let id = idstr.as_str();
      match db_messages.search().with_query(id).execute::<RawMessage>().await{
          Ok(pages)=>{if pages.hits.len() == 0 {add_raw_message(&mut raw_messages, &message)}},
          Err(_)=>{add_raw_message(&mut raw_messages, &message)}
      }
  }

  db.index(DBIndexes::RawMessage.as_str())
      .add_documents(&raw_messages, None)
      .await.unwrap();

  Ok(results::DatabaseResult::SavedMessagesFromChannel(results::SavedMessagesFromChannel{ channel_name: channel_name, quantity: raw_messages.len() }))
}

/*
░░░░░░░░░░░░░░░▓▒░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ \ Haha portal reference how original
░░░░░░░░░░░░░░██▓▒▓▒▒▒▒▒██████▓▓▒▒░░░░░░░░░░░░░░░░ \ Haha portal reference how original
░░░░░░░░░░░░░██▒░░░░░░░░░░░░░░▒▒▓▓████▓▒░░░░░░░░░░ \ Haha portal reference how original
░░░░░░░░░░░░██▓░░░░▒▒▒░░░░░░░▒░░░░░░░░▒▓▓▓▒░░░░░░░ \ Haha portal reference how original
░░░░░░░░░░░▒██▓░░░░░░▓▓▒▒▒▓▓▓▓▓▓▒░░░░░░░░░░▓██▒░░░ \ Haha portal reference how original
░░░░░░░░░░░░░░░▒░░░░░▒▓██▓▓▓▒▒▓▓▓▓██▓▒░░░▒██▓░░░░░ \ Haha portal reference how original
░░░░░░▒▓▓░░░░░▓▓░░░░▓█▒░░░░░░░▒▓▒░▒▒██▒▒██▓░░░░░░░ \ Haha portal reference how original
░░░░░▓██▓▓░░░░░░░░░▓▒░░░░░░▒▒▒▓▓▓▓▓▒▓▓██▓░░░░░░░░░ \ Haha portal reference how original
░░░░▓███▓▓░░░░░░░░▒░░░░░░▓▓▒▒▒▒▒▒▒▓▓█▓▓██▒░░░░░░░░ \ Haha portal reference how original
░░░░████▓▓░░░░░░░▒▒░░░░░██████████████▓▓██░░░░░░░░ \ Haha portal reference how original
░░░▓██████▒░░░░░░█▒░░░░▒██▓████████████▓██▓░░░░░░░ \ Haha portal reference how original
░░░███████▒░░░░░░▓▓▒░░░▓█▓█████████████▓███░░░░░░░ \ Haha portal reference how original
░░░██████▓░░░░░░░▓█▒░░▒▓▓██████████████▓██▓░░░░░░░ \ Haha portal reference how original
░░░██████▒░░░░░░░▒█▓▒▒▓██▓████████████▓▓█▓▓░░░░░░░ \ Haha portal reference how original
░░░██████▒▒░░░░░░░▓█▓▒▓▓█▓███████████▓▓▓█▓▓░░░░░░░ \ Haha portal reference how original
░░░█████▓▒▒▒░░▒▒▒▒▒▓█▓▓▒▒▒▓████████▓▓▓██▓▓▒░░░░░░░ \ Haha portal reference how original
░░░░███▓▒▒▒▒▒▒▒▒▒▒▒▓▓██▓▓▓▓▓▓▓▓███████▓▓▓▓░░░░░░░░ \ Haha portal reference how original
░░░░░▓▓▓▒▓▓█▓▓▓▓▒▓▒▓██████▓▓▓██████▓████▓░░░░░░░░░ \ Haha portal reference how original
░░░░░░▒▓▓▒▓█████▓█▓█████████████████████▓░░░░░░░░░ \ Haha portal reference how original
░░░░░░░░▓▓██████▓█████████████████████▓███▓▒░░░░░░ \ Haha portal reference how original
░░░░░░░░░▒▓█████▓▓▓▓▓▓▓█████████████▓▒░▒▒▓███▓░░░░ \ Haha portal reference how original
░░░░░░░░░░░░▒▒▓▓███████████████████████▓▒▒▒▒▒▓▒░░░ \ Haha portal reference how original
░░░░░░░░░░░░░░░░░░░▒▒░░▒▒▒▒░░░░░░░░░░░░░░░░░░░░░░░ \ Haha portal reference how original
*/