use std::env;

mod baichat_rs;
use std::result::Result;
use baichat_rs::Delta;
use meilisearch_sdk::search::{SearchResult, SearchResults};
use serenity::http::Typing;
use serenity::{async_trait, client};
use serenity::model::prelude::{Ready, ChannelId, GuildChannel};
use serenity::prelude::*;
use serenity::model::channel::Message;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{StandardFramework, CommandResult, Args, Command};


use crate::database::RawMessage;
mod database;

#[group]
#[commands(populate, interpret)]
struct General;

struct Handler;


#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
    async fn message(&self, _ctx: Context, _new_message: Message) {
       
    }
}

#[tokio::main]
async fn main() {


    database::get_database_client()
    .index(database::DBIndexes::RawMessage.as_str())
    .set_sortable_attributes(["timestamp", "username", "message"])
    .await
    .unwrap();
    
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

    let owner: u64 = env!("DISCORD_BOT_OWNER").parse::<u64>()?;
    if arg_msg.is_private() && !arg_msg.author.bot && arg_msg.author.id.0 == owner{}else {return Ok(())}

    let chat_id = args.single_quoted::<u64>().unwrap_or(0);
    let limit = args.single_quoted::<u64>().unwrap_or(1);
    let channel: GuildChannel = ctx.http.get_channel(chat_id).await?.guild().unwrap();
    let discord_messages: Vec<Message> = channel.messages(&ctx.http, |retriever| retriever.limit(limit)).await?;

    let db = database::get_database_client();
    let db_massages = db.index(database::DBIndexes::RawMessage.as_str());
    
    let mut raw_messages : Vec<RawMessage> = vec!();
    for message in discord_messages{
        let idstr = message.id.0.to_string();
        let id = idstr.as_str();
        match db_massages.search().with_query(id).execute::<RawMessage>().await{
            Ok(pages)=>{if pages.hits.len() == 0 {add_raw_message(&mut raw_messages, &message)}},
            Err(_)=>{add_raw_message(&mut raw_messages, &message)}
        }
    }

    db.index(database::DBIndexes::RawMessage.as_str())
        .add_documents(&raw_messages, None)
        .await?;

    match arg_msg.reply(&ctx.http, format!("💾 Queried {} messages to DB from channel {}.\n ~Im getting smarter :3", raw_messages.len(), channel.name)).await{
        Ok(_)=>{println!("AAAAA")},
        Err(e)=>{println!("{}", e)}
    };

    Ok(())
}

#[command]
/*This gets the last messages stored in the database and puts it on a prompt to the AI API 
so it can then create a result of what it "learned". Storing it at a "memories" database
THIS MAY CHANGE LATER */
async fn interpret(ctx: &Context, arg_msg: &Message, mut args: Args) -> CommandResult {
    let mut msgs: SearchResults<RawMessage> = database::get_database_client()
        .index(database::DBIndexes::RawMessage.as_str())
        .search()
        .with_sort(&["timestamp:desc", "username:desc", "message:desc"])
        .with_limit(10)
        .execute::<RawMessage>()
        .await
        .unwrap();
    
    msgs.hits.sort_by(|a,b| {
        a.result.timestamp.cmp(&b.result.timestamp)
    });


    let mut prompt: String = "Leia as mensagens e finja que você está participando da conversa, me dê uma resposta que poderia responder os participantes.".into();

    for raw_message in msgs.hits {
        let raw_message: RawMessage = raw_message.result;
        prompt += &format!("Usuário {}, no tempo {}, disse: \"{}\"\n", 
                            raw_message.user_name, 
                            serenity::model::Timestamp::from_unix_timestamp(raw_message.timestamp).unwrap().to_string(), 
                            raw_message.message);
    }

    let mut AI = baichat_rs::ThebAI::new(None);
    let answer: Vec<Delta> = AI.ask(&prompt, None).await.expect("answer");
    let answer = answer[answer.len() - 1].text.clone();


    println!("{:?}", prompt);
    let res = arg_msg.reply(&ctx.http, format!("{} \n\n {}", prompt, answer)).await;
    match res {
        Ok(i)=>{
            println!("{:?}", i)
        }
        Err(e)=>{
            println!("{}", e)
        }
    }
    Ok(())
}