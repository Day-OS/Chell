use std::env;

mod baichat_rs;
use std::result::Result;
use baichat_rs::Delta;
use meilisearch_sdk::search::{SearchResult, SearchResults};
use serde_json::json;
use serenity::http::Typing;
use serenity::model::mention;
use serenity::{async_trait, client};
use serenity::model::prelude::{Ready, ChannelId, GuildChannel};
use serenity::prelude::*;
use serenity::model::channel::Message;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{StandardFramework, CommandResult, Args, Command};


use crate::memory_core::RawMessage;
mod memory_core;
mod ai;
mod results;

#[group]
#[commands(populate, interpret, memorize)]
struct General;

struct Handler;


#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
    async fn message(&self, _ctx: Context, _new_message: Message) {
       //just check if the message.content.contains() "<@{}>", bot_id or <@!{}>, bot_id
    }
    
}

#[tokio::main]
async fn main() {


    let db = memory_core::get_database_client().index(memory_core::DBIndexes::RawMessage.as_str());
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

    let result = match memory_core::save_last_n_messages(&ctx.http, chat_id, limit).await.unwrap(){
        results::DatabaseResult::SavedMessagesFromChannel(result)=>{result},
        _=>{return Ok(())}
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
/*This gets the last messages stored in the database and puts it on a prompt to the AI API 
so it can then create a result of what it "learned". Storing it at a "memories" database
THIS MAY CHANGE LATER */
async fn interpret(ctx: &Context, arg_msg: &Message) -> CommandResult {
    let logs = memory_core::get_last_n_messages(28).await;

    match arg_msg.reply(&ctx.http, format!(" \n\n {}", ai::say(logs).await?)).await {
        Ok(_)=>{}
        Err(e)=> println!("{}", e),
    }
    Ok(())
}

#[command]
async fn memorize(ctx: &Context, arg_msg: &Message) -> CommandResult {
    let logs = memory_core::get_last_n_messages(28).await;

    let response = match ai::interpret_and_memorize(logs, serenity::model::Timestamp::now().unix_timestamp() as u64).await {
        Ok(response)=>{response}
        Err(e)=> panic!("{:?}", e),
    };
    match arg_msg.reply(&ctx.http, format!(" \n\n {}", response)).await {
        Ok(_)=>{}
        Err(e)=> println!("{}", e),
    }
    Ok(())
}