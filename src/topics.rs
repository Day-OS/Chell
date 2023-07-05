use serde::{Serialize, Deserialize};
use serde_json::json;
use crate::utils;
use crate::chat_logs::ChatLogs;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Topics(Vec<String>);
impl Topics {
    pub async fn from_logs(logs: &ChatLogs) -> Result<Topics, utils::Error>{
        let mut message_buffer : String = "".into();
        for log in logs.clone().0 {
          message_buffer += &log.message;
        }
        let stopwords: Vec<String> = stop_words::get(stop_words::LANGUAGE::Portuguese);
        let mut topics: Vec<String> = message_buffer.split_whitespace()
                                      .filter(|word| !stopwords.contains(&json!(word.to_string().to_lowercase())
                                      .to_string())).map(|word| word.to_string()).collect();
        topics.reverse();
        println!("{:?}",topics);
        Ok(Topics(topics))
    }
    pub fn to_query(&self) -> String{
        let mut topics_query_text: String = "".into();
        for topic in self.0.clone() {
            topics_query_text += &format!("{} or ", topic);
        }
        topics_query_text
    }

}

