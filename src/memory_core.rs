use serde::{Serialize, Deserialize};
use crate::{utils::{self, DBIndexes}, topics::Topics, ai};
use crate::Timestamp;

#[derive(Serialize, Deserialize, Debug, Clone,)]
pub struct Memory{
  pub id: String,
  pub timestamp: u64,
  pub content: String,
}
#[derive(Debug)]
pub struct SavedMemories(pub String);
#[derive(Debug)]
pub struct LoadedMemories(pub Vec<Memory>);
impl LoadedMemories {
  pub async fn to_string(&self) -> String{
    let mut memories :String = "".into();
    for r in self.0.clone() {
      memories += &format!("{}\n", r.content);
    }
    memories
  }
}

pub async fn load_last_n_memories(n: usize, max_timestamp: Option<u64>) -> Result<LoadedMemories, utils::Error>{
  let hits =  match utils::get_database_client()
    .index(DBIndexes::InputMemory.as_str())
    .search()
    .with_filter(&format!("timestamp < {}", max_timestamp.unwrap_or(Timestamp::now().unix_timestamp() as u64)))
    .with_limit(n)
    .execute::<Memory>().await{
        Ok(pages)=>{pages},
        Err(_)=>{return Err(utils::Error::NoMemoriesFound)}
  }.hits;

  Ok(LoadedMemories(hits.into_iter().map(|result| result.result).collect()))
}

pub async fn load_memory(topics: Topics) -> Result<LoadedMemories, utils::Error>{
  let hits =  match utils::get_database_client()
    .index(DBIndexes::InputMemory.as_str())
    .search().with_query(&topics.to_query())
    .with_limit(3)
    .execute::<Memory>().await{
        Ok(pages)=>{pages},
        Err(_)=>{return Err(utils::Error::NoMemoriesFound)}
  }.hits;
  Ok(LoadedMemories(hits.into_iter().map(|result| result.result).collect()))
}


pub async fn save_memory(input_memory: &ai::ResponseMessage) -> Result<SavedMemories, utils::Error>{
  let learned: String = match &input_memory.learned {
      Some(learned)=>{if learned.to_lowercase() == "null"{return Err(utils::Error::NoMemoriesToBeSaved)} learned.to_string()}
      None=>{return Err(utils::Error::NoMemoriesToBeSaved)}
  };
  
  let id: String = learned.split(" ").flat_map(|s| s.chars().nth(0)).collect(); 
  let memory = Memory{
                          id: id,
                          timestamp:serenity::model::Timestamp::now().unix_timestamp() as u64, 
                          content:learned
                        };
  let content = memory.clone().content;
  match utils::get_database_client()
        .index(DBIndexes::InputMemory.as_str())
        .add_documents(&[memory], None)
        .await{
    Ok(o)=>{println!("{:?}", o)}
    Err(o)=>{println!("{}", o)}  
  };
  Ok(SavedMemories(content))
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