//FROM https://crates.io/crates/baichat-rs/0.1.0
//MODIFIED BECAUSE THE CODE WAS COMPLETE SHIT

use ratmom::{prelude::*, Request};

use serde::{Serialize, Deserialize};

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

    pub async fn ask(&mut self, prompt: &str, parent_message_id: Option<String>) -> Result<Vec<Delta>, Box<dyn std::error::Error>> {

        let mut body = String::new();
        body.push_str(r#"{
            "prompt": ""#);
        body.push_str(&prompt);
        if let Some(parent_message_id) = parent_message_id {
            body.push_str(r#"",
            "options": {
                "parentMessageId": ""#);
            body.push_str(parent_message_id.as_str());
            body.push_str(r#""
                }
            }"#);
        } else {
            body.push_str(r#"",
            "options": {
                "parentMessageId": ""#);
            body.push_str(self.parent_message_id.as_ref().unwrap().as_str());
            body.push_str(r#""
                }
            }"#);
        }
        let mut request = Request::builder()
            .method("POST")
            .uri("https://chatbot.theb.ai/api/chat-process")
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/112.0")
            .header("Accept-Language", "en-US,en;q=0.5")
            .header("Content-Type", "application/json")
            //.header("Host", "chatbot.theb.ai")
            .header("Referer", "https://chatbot.theb.ai")
            .header("Origin", "https://chatbot.theb.ai")
            .body(body)?
            .send()?;
    
        let result = request.text()?;
    
        let mut deltas: Vec<Delta> = Vec::new();
        for line in result.lines() {
            if line == "" {
                continue;
            }
            let delta:  Delta = serde_json::from_str(line).unwrap();
            deltas.push(delta);
        }

        self.parent_message_id = Some(deltas.last().unwrap().id.clone());
    
        println!("{:?}", deltas);
    
        return Ok(deltas);
    }
}


