use std::env;

mod baichat_rs;
use crate::chat_logs::ChatLogs;
use serenity::{async_trait};
use serenity::model::prelude::{Ready};
use serenity::prelude::*;
use serenity::model::channel::Message;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{StandardFramework, CommandResult, Args};
use simplelog::*;
use std::fs::File;
use rand::Rng;
use lazy_static;
mod memory_core;
mod ai;
mod utils;
mod topics;
mod chat_logs;


#[group]
#[commands(populate, talk)]
struct General;


#[derive(Clone, Debug)]
struct WaitingAnswerArgs {
    user_id: String,
    message: u64,
}

#[derive(Clone, Debug)]
enum BotState{
    Standby,
    Active,
    WaitingAnswer(WaitingAnswerArgs) //STRING = USER_ID that the question was asked
}

impl BotState {
    fn get_cap(&self)->u64{
        match self {
            BotState::Standby =>{200}
            BotState::Active => {15}
            _=>{panic!()}
        }
    }
}


lazy_static::lazy_static!{
    static ref MESSAGES_CAP : Mutex<u64> = Mutex::new(15);
    static ref MESSAGES_COUNTER : Mutex<u64> = Mutex::new(0);
    static ref BOT_STATE: Mutex<BotState> = Mutex::new(BotState::Active); //Mutex::new(BotState::WaitingAnswer(WaitingAnswerArgs { user_id: "236575283984072704".into(), message: 1125966367254913126 })); 
    static ref BOT_ID: Mutex<u64> = Mutex::new(0);
}
pub async fn get_bot_id() -> u64{
    BOT_ID.lock().await.clone()
}

struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        *BOT_ID.lock().await = ctx.http.application_id().unwrap();
        println!("{} is connected!", ready.user.name);
    }

    async fn message(&self, ctx: Context, message: Message) {
        let owner: u64 = env!("DISCORD_BOT_OWNER").parse::<u64>().unwrap();

        //---------------------------------- CHECKS ----------------------------------
        //TEMPORARIO PARA TESTES VVVVVVVVVV
        
        //if message.is_private() && (message.author.bot || message.author.id.0 == owner){}else {return};
        let whitelisted_channel: u64 = env::var("WHITELISTED_CHANNEL").unwrap().parse::<u64>().unwrap();
        if message.channel_id.0 != whitelisted_channel {return};     

        //---------------------------------- MANAGING STATES ----------------------------------


        let mut state = BOT_STATE.lock().await;
        println!("state: {:?}", state);
        let mut message_counter = MESSAGES_COUNTER.lock().await;
        let mut message_cap = MESSAGES_CAP.lock().await;

        let bot_id: u64 = ctx.http.application_id().unwrap();
        
        match message.clone().referenced_message {
            Some(reply_message)=>{if reply_message.author.id == bot_id {*state = BotState::WaitingAnswer(WaitingAnswerArgs { user_id: message.author.id.0.to_string(), message: message.id.0})}  }
            None=>{}
        };

        let referenced_message_id: Option<u64> = match message.clone().referenced_message{
            Some(reference)=>{Some(reference.id.0)}
            None=>{None}
        };

        // VVVVV - U64 = MAXIMUM CAP
        let (logs, maximum_cap): (ChatLogs, u64) = match state.clone() {
            BotState::Standby| BotState::Active=>{
                let _ = chat_logs::save_message(message.clone(), None).await;
                
                if (message.content.contains(&format!("<@{}>", bot_id)) ||
                   message.content.contains(&format!("<@!{}>", bot_id)) ||
                   message.content.to_lowercase().contains("chell") ||
                   *message_counter > *message_cap ) && message.author.id.0 != bot_id
                { 
                    let logs = chat_logs::get_last_n_messages(*message_cap as usize).await.unwrap();
                    (logs, state.clone().get_cap())
                }
                else{
                    *message_counter += 1; 
                    return
                }
            }
            BotState::WaitingAnswer(answer_args)=>{
                let _ = chat_logs::save_message(message.clone(), referenced_message_id).await;
                if message.author.id.0 != answer_args.user_id.parse::<u64>().unwrap(){return}
                *state = BotState::Active;
                (chat_logs::get_conversation_with_user(message.id.0).await.unwrap(), 2)
            }
        };

        println!("{:?}", logs.to_string().await);
        //---------------------------------- GENERATING RESPONSE ----------------------------------
        let typing = message.channel_id.start_typing(&ctx.http).unwrap();

        let topics: topics::Topics = topics::Topics::from_logs(&logs).await.unwrap();
    
        let memories: Option<String> = memory_core::load_memory_from_db(topics).await;
    
        let mut response: Option<ai::ResponseMessage> = None;
        for _ in 0..5 {
            response = match ai::reply(logs.to_string().await, memories.clone()).await {
                Ok(response)=>{
                    println!("{:?}", response);
                    Some(response)
                }
                Err(e)=>{
                    match e {
                        ai::Error::CouldntGenerateResponseFromAI | ai::Error::CouldntConvertToJSON =>{
                            log::error!("MESSAGE NOT GENERATED, RETRYING! err:{:?}", e);
                            println!("MESSAGE NOT GENERATED, RETRYING! err:{:?}", e);
                            continue
                            
                        }
                    }
                }
            };
            break;
        }
        let mut response: ai::ResponseMessage = response.unwrap();
        
        //---------------------------------- SENDING RESPONSE ----------------------------------

        let reply_target: Option<Message> = match response.reply.clone() {
            Some(reply_id) => {
                let channel_id = message.channel_id.0;
                let message_id = reply_id;
                Some(ctx.http.get_message(channel_id, message_id.parse::<u64>().unwrap()).await.unwrap())
            }
            None => None,
        };
    
        if response.question { response.message += " | ⤵️" }
        let reply_message : Message = match reply_target {
            Some(target)=>{
                log::info!("Answered {}'s message. | Response: {}", target.author.name, response.message.clone());
                target.reply(&ctx.http, response.message.clone()).await.unwrap()
            }
            None=>{
                log::info!("Sent a message to everyone in chat. | Response: {}", response.message.clone());
                message.channel_id.send_message(&ctx.http, |m|{
                    m.content(response.message.clone())
                }).await.unwrap()
                
            }
        };

        if response.question { *state = BotState::WaitingAnswer(WaitingAnswerArgs { user_id: message.author.id.0.to_string(), message: message.id.0}) }
        else{ *state = BotState::Active }
        _ = memory_core::save_memory_to_db(&response).await;

        
        _ = typing.stop();
        //_ = chat_logs::delete_logs(logs).await;
        println!("AAAAAAAAAAAAAA");
        *message_counter = 0;
        *message_cap = rand::thread_rng().gen_range(1..maximum_cap);
        
        
    }
    
    
}

#[tokio::main]
async fn main() {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Info, Config::default(), File::create("Chell.log").unwrap()),
        ]
    ).unwrap();

    let db = utils::get_database_client().index(utils::DBIndexes::RawMessage.as_str());
    db.set_sortable_attributes(["timestamp", "username", "message"]).await.unwrap();
    db.set_filterable_attributes(["message"]).await.unwrap();
    
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) 
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let mut client = serenity::Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }


}


#[command]
async fn populate(ctx: &Context, arg_msg: &Message, mut args: Args) -> CommandResult {

    let owner: u64 = env!("DISCORD_BOT_OWNER").parse::<u64>()?;
    if arg_msg.is_private() && !arg_msg.author.bot && arg_msg.author.id.0 == owner{}else {return Ok(())}

    let chat_id = args.single_quoted::<u64>().unwrap_or(0);
    let limit = args.single_quoted::<u64>().unwrap_or(1);

    let result = match chat_logs::save_last_n_messages(&ctx.http, chat_id, limit).await{
        Ok(result)=>{result},
        Err(_)=>{return Ok(())}
    };
    //let channel: GuildChannel = .guild().unwrap();

    match arg_msg.reply(&ctx.http, 
        format!("💾 Queried {} messages to DB from channel {}.\n ~Im getting smarter :3",
         result.quantity,result.channel_name)).await{
        Ok(_)=>{println!("AAAAA")},
        Err(e)=>{println!("{}", e)}
    };

    Ok(())
}

#[command]
async fn talk(ctx: &Context, arg_msg: &Message) -> CommandResult {
    //reply_message(ctx, arg_msg.channel_id, chat_logs::get_last_n_messages(28).await.unwrap()).await;
    Ok(())
}