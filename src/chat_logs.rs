use async_recursion::async_recursion;
use futures::FutureExt;
use serde::{Serialize, Deserialize};
use serenity::{http::Http, model::prelude::{Message, Channel}};

use crate::utils;



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawMessage {
  pub id: u64,
  pub user: u64,
  pub user_name: String,
  pub timestamp: i64,
  pub user_image: String,
  pub message: String,
  pub reference_id: Option<u64>
}


#[derive(Debug)]
pub struct SavedMessagesFromChannel{pub channel_name:String,pub quantity:usize}

#[derive(Debug)]
pub struct SavedMessage(pub String);


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatLogs(pub Vec<RawMessage>);
impl ChatLogs {
    pub async fn to_string(&self) -> String{
      let bot_id = crate::get_bot_id().await;
      println!("BOOOOOOOOOOOOOOOT ID!!!!!!!!!!!!!!!1 {}", bot_id);
      let mut messages: String = "".into();
      for raw_message in self.0.clone() {
        if raw_message.user == bot_id {
            messages += &format!("SUA MENSAGEM! | TEMPO: {} - Você respondeu = {}\n", 
                            serenity::model::Timestamp::from_unix_timestamp(raw_message.timestamp).unwrap().to_string(), 
                            raw_message.message
                        );
        }
        else{
            messages += &format!("ID DA MENSAGEM: {} | TEMPO: {} - NOME DE USUÁRIO:'{}' disse = {}\n", 
                            raw_message.id,
                            serenity::model::Timestamp::from_unix_timestamp(raw_message.timestamp).unwrap().to_string(), 
                            raw_message.user_name, 
                            raw_message.message
                        );
        }
      }
      messages
    }
    pub fn get_ids(&self) -> Vec<u64>{
        self.0.clone().into_iter().map(|log|{log.id}).collect()
    }
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

pub async fn get_conversation_with_user(target_message:u64) -> Result<ChatLogs, utils::Error>{
    async fn search_message_from_id(target_message: u64) -> Result<RawMessage, utils::Error>{
        let raw_messages_searched = utils::get_database_client()
        .index(utils::DBIndexes::RawMessage.as_str())
        .search()
        .with_query(&target_message.to_string())
        .with_sort(&["timestamp:desc", "username:desc", "message:desc"])
        .with_limit(1)
        .execute::<RawMessage>()
        .await.unwrap();
        Ok(raw_messages_searched.hits.first().unwrap().result.clone())
    }
    #[async_recursion]
    async fn get_referenced_message(target_message: u64, mut total_messages: &mut Vec<RawMessage>) -> (){
        //println!("{:?}", total_messages);
        let new_target: RawMessage = search_message_from_id(target_message).await.unwrap();
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

pub async fn get_specific_message(target_message:u64) -> Result<RawMessage, utils::Error>{
    let raw_messages_searched = utils::get_database_client()
        .index(utils::DBIndexes::RawMessage.as_str())
        .search()
        .with_query(&target_message.to_string())
        .with_sort(&["timestamp:desc", "username:desc", "message:desc"])
        .with_limit(1)
        .execute::<RawMessage>()
        .await.unwrap();
    Ok(raw_messages_searched.hits.first().unwrap().result.clone())
}


pub async fn get_last_n_messages(n: usize) -> Result<ChatLogs, utils::Error>{
    let mut raw_messages_searched = match  utils::get_database_client()
    .index(utils::DBIndexes::RawMessage.as_str())
    .search()
    .with_filter("message IS NOT EMPTY")
    .with_sort(&["timestamp:desc", "username:desc", "message:desc"])
    .with_limit(n)
    .execute::<RawMessage>()
    .await{
      Ok(msg)=>{msg}
      Err(e)=>{return Err(utils::Error::MeiliSearchError(e))}   
    };
  
    raw_messages_searched.hits.sort_by(|a,b| {a.result.timestamp.cmp(&b.result.timestamp)});
    let mut logs: ChatLogs = ChatLogs(vec!());
    for raw_message in raw_messages_searched.hits {logs.0.push(raw_message.result);}
    Ok(logs)
  }


pub async fn save_last_n_messages(http: &Http, chat_id: u64, n: u64) -> Result<SavedMessagesFromChannel, utils::Error>{
    let channel: Channel = http.get_channel(chat_id).await.unwrap();
    fn add_raw_message(raw_messages: &mut Vec<RawMessage>, message: &Message){
        let referenced_message: Option<u64> = match message.referenced_message.clone(){
            Some(m)=>{Some(m.id.0)}
            None=>{None}
        };


        let raw_message = RawMessage{
            user_name:message.author.clone().name,
            id: message.id.0,
            timestamp:message.timestamp.unix_timestamp(),
            user: message.author.id.0,
            user_image: message.author.clone().avatar_url().unwrap_or("NO AVATAR".into()),
            message: message.clone().content,
            reference_id: referenced_message
        };
        raw_messages.push(raw_message);
    }
    let channel_name;
    let discord_messages: Vec<Message> =  match channel {
        Channel::Guild(channel)=>{
            if channel.nsfw {return Err(utils::Error::ChannelIsNSFW)};
            channel_name = channel.name().into();
            channel.messages(&http, |retriever| retriever.limit(n)).await.unwrap()
        }
        Channel::Private(channel)=>{
            //PREVENTS SKETCHY DM WEIRDOS FROM... DATING THE AI
            let owner: u64 = env!("DISCORD_BOT_OWNER").parse::<u64>().unwrap();
            if channel.recipient.id != owner{return Err(utils::Error::PrivateChannelUserIsNotOwner)}

            channel_name = channel.clone().recipient.name + "'s DM";
            channel.messages(&http, |retriever| retriever.limit(n)).await.unwrap()
        }
        _=>{return Err(utils::Error::Generic)}
    };


    let db = utils::get_database_client();
    let db_messages = db.index(utils::DBIndexes::RawMessage.as_str());

    let mut raw_messages : Vec<RawMessage> = vec!();
    for message in discord_messages{
        let idstr = message.id.0.to_string();
        let id = idstr.as_str();
        match db_messages.search().with_query(id).execute::<RawMessage>().await{
            Ok(pages)=>{if pages.hits.len() == 0 {add_raw_message(&mut raw_messages, &message)}},
            Err(_)=>{add_raw_message(&mut raw_messages, &message)}
        }
    }

    db.index(utils::DBIndexes::RawMessage.as_str())
        .add_documents(&raw_messages, None)
        .await.unwrap();

    Ok(SavedMessagesFromChannel{ channel_name: channel_name, quantity: raw_messages.len() })
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
        reference_id: referenced_message
    };

    db.index(utils::DBIndexes::RawMessage.as_str())
        .add_documents(&[raw_message], None)
        .await.unwrap();

    Ok(SavedMessage(message.clone().content))
}