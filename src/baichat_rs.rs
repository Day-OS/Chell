//FROM https://crates.io/crates/baichat-rs/0.1.0
//MODIFIED BECAUSE THE CODE WAS COMPLETE redacted

use ratmom::{prelude::*, Request, config::SslOption};

use serde::{Serialize, Deserialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    pub prompt: String,
    pub options: Options,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Options {
    #[serde(rename(serialize = "parentMessageId", deserialize = "parentMessageId"))]
    pub parent_message_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Delta {
    pub role: String,
    pub id: String,
    #[serde(rename(serialize = "parentMessageId", deserialize = "parentMessageId"))]
    pub parent_message_id: String,
    pub text: String,
    pub delta: String,
    pub detail: Detail,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Detail {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Choice {
    pub delta: DeltaChoice,
    pub index: i64,
    pub finish_reason: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeltaChoice {
    pub content: String,
}

pub struct ThebAI {
    pub parent_message_id: Option<String>,
}
impl ThebAI {
    pub fn new(parent_message_id: Option<&str>) -> ThebAI {
        if let Some(parent_message_id) = parent_message_id {
            return ThebAI {
                parent_message_id: Some(parent_message_id.to_string()),
            }
        } else {
            return ThebAI {
                parent_message_id: Some(String::from("8c00bd29-75b0-42c7-9d4f-05a94ac8b2de")),
            }
        }
    }

    pub async fn ask_single(&mut self, prompt: &str, parent_message_id: Option<String>) -> Result<Delta, Box<dyn std::error::Error>> {
        let parent_message_id: String = if let Some(parent_message_id) = parent_message_id {
            parent_message_id
        } else {
            self.parent_message_id.clone().unwrap()
        };
        let body: String = serde_json::to_string(&Input{ prompt:json!(prompt).to_string(), options: Options { parent_message_id: parent_message_id }}).unwrap();
        
        
        //println!("{}", body);
        let mut request = Request::builder()
            .method("POST")
            .uri("https://chatbot.theb.ai/api/chat-process")
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/112.0")
            .header("Accept-Language", "en-US,en;q=0.5")
            .header("Content-Type", "application/json")
            .header("Referer", "https://chatbot.theb.ai")
            .header("Origin", "https://chatbot.theb.ai")
            .ssl_options(SslOption::DANGER_ACCEPT_INVALID_CERTS | SslOption::DANGER_ACCEPT_INVALID_HOSTS | SslOption::DANGER_ACCEPT_REVOKED_CERTS)
            .body(body)?
            .send()?;
    
        let result = request.text()?;
    
        println!("BAICHAT RESULT: {}", result);
        //println!("{:?}", result.lines());
        let mut target_line: &str = "".into();
        for line in result.lines() {
            if line == "" {
                continue;
            }
            target_line = line;
        }
        match serde_json::from_str(target_line) {
            Ok (delta)=> {
                return Ok(delta)
            } 
            Err(err)=>{
                println!("BAICHAT ERRROR: {} \n {}", err, result);
                return Err(err.into())
            }
        }
        //println!("{:?}", deltas);
    
        ;
    }

    pub async fn ask(&mut self, prompt: &str, parent_message_id: Option<String>) -> Result<Vec<Delta>, Box<dyn std::error::Error>> {
        let parent_message_id: String = if let Some(parent_message_id) = parent_message_id {
            parent_message_id
        } else {
            self.parent_message_id.clone().unwrap()
        };
        let body: String = serde_json::to_string(&Input{ prompt:json!(prompt).to_string(), options: Options { parent_message_id: parent_message_id }}).unwrap();
        
        
        //println!("{}", body);
        let mut request = Request::builder()
            .method("POST")
            .uri("https://chatbot.theb.ai/api/chat-process")
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/112.0")
            .header("Accept-Language", "en-US,en;q=0.5")
            .header("Content-Type", "application/json")
            .header("Referer", "https://chatbot.theb.ai")
            .header("Origin", "https://chatbot.theb.ai")
            .ssl_options(SslOption::DANGER_ACCEPT_INVALID_CERTS | SslOption::DANGER_ACCEPT_INVALID_HOSTS | SslOption::DANGER_ACCEPT_REVOKED_CERTS)
            .body(body)?
            .send()?;
    
        let result = request.text()?;
    
        let mut deltas: Vec<Delta> = Vec::new();
        println!("BAICHAT RESULT: {}", result);
        //println!("{:?}", result.lines());
        for line in result.lines() {
            if line == "" {
                continue;
            }
            
            match serde_json::from_str(line) {
                Ok (delta)=> {
                    deltas.push(delta)
                } 
                Err(err)=>{println!("BAICHAT ERRROR: {} | {} ", err, line)}
            }
        }
        match deltas.last() {
            Some(delta)=>{self.parent_message_id = Some(delta.id.clone())}
            None => {return Err("Error::WrongFormat(result)".into())}
        }
    
        //println!("{:?}", deltas);
    
        return Ok(deltas);
    }
}
pub async fn deltas_to_string(delta : Vec<Delta>) -> String{
    delta[delta.len() - 1].text.clone()
}

