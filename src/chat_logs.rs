use async_recursion::async_recursion;
use meilisearch_sdk::documents::DocumentQuery;
use serde::{Serialize, Deserialize};
use serenity::{http::Http, model::prelude::{Message, ChannelId}};

use crate::utils;



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawMessage {
  pub id: u64,
  pub user: u64,
  pub user_name: String,
  pub timestamp: i64,
  pub user_image: String,
  pub message: String,
  pub reference_id: Option<u64>,
  pub read: bool,
}


#[derive(Debug)]
pub struct SavedMessagesFromChannel{pub channel_id:u64,pub quantity:usize}

#[derive(Debug)]
pub struct SavedMessage(pub String);


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatLogs(pub Vec<RawMessage>);
impl ChatLogs {
    pub async fn build(&self, target_message_id: Option<u64>) -> String{
      let bot_id = crate::get_bot_id().await;
      let mut messages: String = "".into();
      for raw_message in self.0.clone() {
        if raw_message.user == bot_id {
            messages += &format!("SUA MENSAGEM! | TEMPO: {} - Você respondeu = {}\n", 
                            serenity::model::Timestamp::from_unix_timestamp(raw_message.timestamp).unwrap().to_string(), 
                            raw_message.message
                        );
        }
        else{
            if target_message_id.is_some(){
                if raw_message.id == target_message_id.unwrap() {
                    messages += &format!("ID DA MENSAGEM QUE PRECISA SER RESPONDIDO: {} | TEMPO: {} - NOME DE USUÁRIO:'{}' disse = {}\n", 
                    raw_message.id,
                    serenity::model::Timestamp::from_unix_timestamp(raw_message.timestamp).unwrap().to_string(), 
                    raw_message.user_name, 
                    raw_message.message);
                }
                else{
                    messages += &format!("MENSAGEM PARA CONTEXTO | TEMPO: {} - NOME DE USUÁRIO:'{}' disse = {}\n", 
                    serenity::model::Timestamp::from_unix_timestamp(raw_message.timestamp).unwrap().to_string(), 
                    raw_message.user_name, 
                    raw_message.message);
                }
            }
            else{
                if raw_message.read {
                    messages += &format!("MENSAGEM PARA CONTEXTO | TEMPO: {} - NOME DE USUÁRIO:'{}' disse = {}\n", 
                    serenity::model::Timestamp::from_unix_timestamp(raw_message.timestamp).unwrap().to_string(), 
                    raw_message.user_name, 
                    raw_message.message);
                }
                else{
                    messages += &format!("ID DA MENSAGEM: {} | TEMPO: {} - NOME DE USUÁRIO:'{}' disse = {}\n", 
                    raw_message.id,
                    serenity::model::Timestamp::from_unix_timestamp(raw_message.timestamp).unwrap().to_string(), 
                    raw_message.user_name, 
                    raw_message.message);
                }
            }
        }
      }
      messages
    }
    pub fn filter_read(&mut self){
        self.0 = self.0.clone().into_iter().filter(|msg| !msg.read).collect();
    }
    pub fn get_ids(&self) -> Vec<u64>{
        self.0.clone().into_iter().map(|log|{log.id}).collect()
    }
}


pub async fn set_read(logs: ChatLogs) -> Result<(), utils::Error>{
    let logs: Vec<RawMessage> = logs.0.into_iter().map(|mut message| {message.read = true; message}).collect();
    match  utils::get_database_client()
    .index(utils::DBIndexes::RawMessage.as_str())
    .add_or_replace(&logs, None)
    .await{
      Ok(_)=>{
        log::info!("The following messages were set as read in database: {:?}", logs)
      }
      Err(e)=>{return Err(utils::Error::MeiliSearchError(e))}   
    };
    Ok(())
}

pub async fn delete_logs(logs: ChatLogs) -> Result<(), utils::Error>{
    match  utils::get_database_client()
    .index(utils::DBIndexes::RawMessage.as_str())
    .delete_documents(&logs.get_ids())
    .await{
      Ok(_)=>{
        log::info!("The following messages were deleted from database: {:?}", logs)
      }
      Err(e)=>{return Err(utils::Error::MeiliSearchError(e))}   
    };
    Ok(())
}


pub async fn get_specific_message(target_message:u64) -> Result<RawMessage, utils::Error>{
    let raw_message = DocumentQuery::new( &utils::get_database_client().index(utils::DBIndexes::RawMessage.as_str()))
        .execute::<RawMessage>(&target_message.to_string())
        .await;
        match raw_message{
            Ok(raw_message) => {return Ok(raw_message)}
            Err(e)=>{ return Err(utils::Error::NoReferencedMessageFound(e.to_string()))}
        }
}

pub async fn get_conversation_with_user(target_message:u64) -> Result<ChatLogs, utils::Error>{
    #[async_recursion]
    async fn get_referenced_message(target_message: u64, mut total_messages: &mut Vec<RawMessage>) -> (){
        println!("{:?}", total_messages);
        let new_target_result = get_specific_message(target_message).await;
        if new_target_result.is_err(){return} 
        let new_target = new_target_result.unwrap();
        total_messages.push(new_target.clone());
        if new_target.reference_id.is_some() {
            get_referenced_message(new_target.reference_id.unwrap(), &mut total_messages).await;
        }
    }

    let mut messages : Vec<RawMessage> = vec!();
    get_referenced_message(target_message, &mut messages).await;

    messages.sort_by(|a,b| {a.timestamp.cmp(&b.timestamp)});

    Ok(ChatLogs(messages))
}



pub async fn get_last_n_messages(n: usize) -> Result<ChatLogs, utils::Error>{
    let mut raw_messages_searched = match  utils::get_database_client()
    .index(utils::DBIndexes::RawMessage.as_str())
    .search()
    .with_filter("message IS NOT EMPTY")
    .with_sort(&["timestamp:desc"])
    .with_limit(n)
    .execute::<RawMessage>()
    .await{
      Ok(msg)=>{msg}
      Err(e)=>{return Err(utils::Error::MeiliSearchError(e))}   
    };
    raw_messages_searched.hits.sort_by(|a,b| {a.result.timestamp.cmp(&b.result.timestamp)});
    Ok(ChatLogs( raw_messages_searched.hits.into_iter().map(|msg| msg.result).collect()) )
  }


pub async fn save_last_n_messages(http: &Http, channel_id: u64, n: u64) -> Result<SavedMessagesFromChannel, utils::Error>{
    let messages: Vec<Message> = ChannelId(channel_id).messages(&http, |retriever| retriever.limit(n)).await.unwrap();
    let raw_messages : Vec<RawMessage> = messages.iter().map(|message| {
            let referenced_message: Option<u64> = match message.referenced_message.clone(){
                Some(m)=>{Some(m.id.0)}
                None=>{None}
            };
            RawMessage{user_name:message.author.clone().name,
                id: message.id.0,
                timestamp:message.timestamp.unix_timestamp(),
                user: message.author.id.0,
                user_image: message.author.clone().avatar_url().unwrap_or("NO AVATAR".into()),
                message: message.clone().content,
                reference_id: referenced_message,
                read: false
            }
        }
    ).collect();

    let db = utils::get_database_client();
    let db_messages = db.index(utils::DBIndexes::RawMessage.as_str());
    db_messages.add_documents(&raw_messages, None).await.unwrap();

    Ok(SavedMessagesFromChannel{ channel_id: channel_id, quantity: raw_messages.len() })
    
}

pub async fn save_message(message: Message, pre_defined_reference: Option<u64>) -> Result<SavedMessage, utils::Error>{
    let db = utils::get_database_client();

    let referenced_message: Option<u64> = match pre_defined_reference {
        Some(m)=>{Some(m)}
        None=>{
            match message.clone().referenced_message{
                Some(m)=>{Some(m.id.0)}
                None=>{None}
            }
        }
    };

    let raw_message = RawMessage{
        user_name:message.author.clone().name,
        id: message.id.0,
        timestamp:message.timestamp.unix_timestamp(),
        user: message.author.id.0,
        user_image: message.author.clone().avatar_url().unwrap_or("NO AVATAR".into()),
        message: message.clone().content,
        reference_id: referenced_message,
        read:false
    };

    db.index(utils::DBIndexes::RawMessage.as_str())
        .add_documents(&[raw_message], None)
        .await.unwrap();

    Ok(SavedMessage(message.clone().content))
}