use serde::{Serialize, Deserialize};
use crate::{utils::{self, DBIndexes}, topics::Topics, ai};


#[derive(Serialize, Deserialize, Debug, Clone,)]
pub struct Memory{
  pub id: String,
  pub timestamp: u64,
  pub content: String,
}
#[derive(Debug)]
pub struct SavedMemories(pub String);

pub async fn load_memory_from_db(topics: Topics) -> Option<String>{
  let mut memories :String = "".into();
  let hits =  match utils::get_database_client()
    .index(DBIndexes::InputMemory.as_str())
    .search().with_query(&topics.to_query())
    .with_limit(1)
    .execute::<Memory>().await{
        Ok(pages)=>{pages},
        Err(_)=>{return None}
  }.hits;
  for r in hits {
    memories += &format!("{}\n", r.result.content);
  }
  if !memories.is_empty() {
    return Some(memories)
  }
  None
}


pub async fn save_memory_to_db(input_memory: &ai::ResponseMessage) -> Result<SavedMemories, utils::Error>{
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
  println!("{:?}", memory);
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