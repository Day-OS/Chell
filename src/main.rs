use std::env;

mod baichat_rs;
use crate::chat_logs::ChatLogs;
use serenity::model::Timestamp;
use serenity::{async_trait};
use serenity::model::prelude::{Ready, interaction};
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
#[commands(memories)]
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
            BotState::Standby =>{2000}
            BotState::Active => {100}
            _=>{panic!()}
        }
    }
}


lazy_static::lazy_static!{
    static ref QUESTIONS_WAITING_ANSWERS: Mutex<Vec<WaitingAnswerArgs>> = Mutex::new(vec!());
    static ref MESSAGES_CAP : Mutex<u64> = Mutex::new(BotState::Active.get_cap());
    static ref MESSAGES_COUNTER : Mutex<u64> = Mutex::new(0);
    static ref BOT_STATE: Mutex<BotState> = Mutex::new(BotState::Active);
    static ref BOT_ID: Mutex<u64> = Mutex::new(0);
    static ref LAST_ACTIVITY_TIME: Mutex<i64> = Mutex::new(0);
    static ref AICHAT: Mutex<baichat_rs::ThebAI> = Mutex::new(ai::get_ai());
}
pub async fn get_bot_id() -> u64{
    BOT_ID.lock().await.clone()
}

struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        *LAST_ACTIVITY_TIME.lock().await = Timestamp::now().unix_timestamp();
        *BOT_ID.lock().await = ctx.http.application_id().unwrap();
        let whitelisted_channel: u64 = env::var("WHITELISTED_CHANNEL").unwrap().parse::<u64>().unwrap();

        match chat_logs::save_last_n_messages(&ctx.http, whitelisted_channel, 25).await {
            Ok(saved_message)=>{
                println!("AUTOMATIC SAVING SUCCEED: {} {}", saved_message.channel_id ,saved_message.quantity);
                log::info!("AUTOMATIC SAVING SUCCEED: {} {}", saved_message.channel_id ,saved_message.quantity);
            }
            Err(err)=>{
                println!("MESSAGE NOT SAVED | {:?}", err);
                log::error!("MESSAGE NOT SAVED | {:?}", err);
            }
        };
        println!("{} is connected!", ready.user.name);
    }

    async fn message(&self, ctx: Context, message: Message) {

        //---------------------------------- CHECKS ----------------------------------

        //TEMPORARIO PARA TESTES VVVVVVVVVV
        //let owner: u64 = env!("DISCORD_BOT_OWNER").parse::<u64>().unwrap();
        //if message.is_private() && (message.author.bot || message.author.id.0 == owner){}else {return};
        
        let whitelisted_channel: u64 = env::var("WHITELISTED_CHANNEL").unwrap().parse::<u64>().unwrap();
        if message.channel_id.0 != whitelisted_channel {return};     

        //---------------------------------- MANAGING STATES ----------------------------------

        let mut state = BOT_STATE.lock().await;
        let mut message_counter = MESSAGES_COUNTER.lock().await;
        let mut message_cap = MESSAGES_CAP.lock().await;

        let bot_id: u64 = ctx.http.application_id().unwrap();
        
        match message.clone().referenced_message {
            Some(reply_message)=>{if reply_message.author.id == bot_id {
                *state = BotState::WaitingAnswer(WaitingAnswerArgs { user_id: message.author.id.0.to_string(), message: message.id.0})
            }}
            None=>{}
        };

        let referenced_message_id: Option<u64> = match message.clone().referenced_message{
            Some(reference)=>{Some(reference.id.0)}
            None=>{None}
        };

        let mut target_message_id: Option<u64> = None;
        match chat_logs::save_message(message.clone(), referenced_message_id).await {
            Ok(saved_message)=>{
                println!("MESSAGE SAVED: {}", saved_message.0);
                log::info!("MESSAGE SAVED: {}", saved_message.0);
            }
            Err(err)=>{
                println!("MESSAGE NOT SAVED | {:?}", err);
                log::error!("MESSAGE NOT SAVED | {:?}", err);
            }
        }
        
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        let (logs, maximum_cap): (ChatLogs, u64) = match state.clone() {
            BotState::Standby| BotState::Active=>{
                if (LAST_ACTIVITY_TIME.lock().await.clone() - Timestamp::now().unix_timestamp()) > 900 {
                    *state = BotState::Standby;
                    return
                }
                if (message.content.contains(&format!("<@{}>", bot_id)) ||
                   message.content.contains(&format!("<@!{}>", bot_id)) ||
                   message.content.to_lowercase().contains("chell") ||
                   *message_counter > *message_cap ) && message.author.id.0 != bot_id
                {
                    (chat_logs::get_last_n_messages(10).await.unwrap(), state.clone().get_cap())
                }
                else{
                    *message_counter += 1; 
                    return
                }
            }
            BotState::WaitingAnswer(answer_args)=>{
                if (LAST_ACTIVITY_TIME.lock().await.clone() - Timestamp::now().unix_timestamp()) > 60 {
                    *state = BotState::Active;
                    return
                }
                if message.author.id.0 != answer_args.user_id.parse::<u64>().unwrap(){return}
                target_message_id = Some(message.id.0);
                (chat_logs::get_conversation_with_user(message.id.0).await.unwrap(), 2)
            }
        };

        //---------------------------------- GENERATING RESPONSE ----------------------------------
        
        *LAST_ACTIVITY_TIME.lock().await = Timestamp::now().unix_timestamp();
        let typing = message.channel_id.start_typing(&ctx.http).unwrap();

        let topics: topics::Topics = topics::Topics::from_logs(&logs).await.unwrap();
    
        let memories = memory_core::load_memory(topics).await;
        let memories: Option<String> = if memories.is_ok(){Some(memories.unwrap().to_string().await)} else{None};
        let prompt  = logs.build(target_message_id).await;

        println!("PROMPT: {}", prompt);
        log::info!("PROMPT: {}", prompt);
        let mut response =  match ai::reply(prompt, memories.clone()).await{
            Ok(res)=>{res},
            Err(_)=>{return}
        };
        
        //---------------------------------- SENDING RESPONSE ----------------------------------

        let reply_target: Option<Message> = match response.reply.clone() {
            Some(reply_id) => {
                let channel_id = message.channel_id.0;
                let message_id = reply_id;
                Some(ctx.http.get_message(channel_id, message_id.parse::<u64>().unwrap()).await.unwrap())
            }
            None => None,
        };
    
        if response.question { response.message += &format!( " | ⤵️ ({})", message.author.name)}
        _ = match reply_target {
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

        chat_logs::set_read(logs).await;
        //STATE SET HERE IS USED IN THE NEXT TIME THE EVENT IS FIRED
        if response.question { *state = BotState::WaitingAnswer(WaitingAnswerArgs { user_id: message.author.id.0.to_string(), message: message.id.0}) }
        else{ *state = BotState::Active }
        _ = memory_core::save_memory(&response).await;

        
        _ = typing.stop();
        *message_counter = 0;
        *message_cap = rand::thread_rng().gen_range(1..maximum_cap);
         
        
    }
    
    
}

#[tokio::main]
async fn main() {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Info, ConfigBuilder::new().add_filter_allow("chell".into()).build(), File::create("Chell.log").unwrap()),
        ]
    ).unwrap();

    let db = utils::get_database_client().index(utils::DBIndexes::RawMessage.as_str());
    db.set_sortable_attributes(["timestamp"]).await.unwrap();
    db.set_filterable_attributes(["message", "reference_id"]).await.unwrap();
    
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

/*
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
         result.quantity,result.channel_id)).await{
        Ok(_)=>{println!("AAAAA")},
        Err(e)=>{println!("{}", e)}
    };

    Ok(())
}
 */

#[command]
async fn memories(ctx: &Context, arg_msg: &Message) -> CommandResult {
    let mems = memory_core::load_last_n_memories(25, None).await.unwrap();
    _ = arg_msg.channel_id.send_message(&ctx.http, |msg| 
        msg
        .embed(|embed| 
            embed.description("hmmm"))
        ).await;
    //reply_message(ctx, arg_msg.channel_id, chat_logs::get_last_n_messages(28).await.unwrap()).await;
    Ok(())
}