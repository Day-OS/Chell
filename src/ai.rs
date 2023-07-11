use crate::{baichat_rs::{self, ThebAI, Delta}, ai};

use serde::{Serialize, Deserialize};
use std::env;
use lazy_static;

#[derive(Debug)]
pub enum Error{
    CouldntConvertToJSON,
    CouldntGenerateResponseFromAI,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResponseMessage{
  pub message: String,
  pub reply: Option<String>,
  pub learned: Option<String>,
  pub question: bool,
}


pub fn get_ai()-> ThebAI {
    baichat_rs::ThebAI::new(Some(&env::var("PARENT_MESSAGE_ID_GPT").unwrap()))
}
pub async fn reply(mut prompt: String, memories: Option<String>) -> Result<ai::ResponseMessage, Error>{
    match memories {
        Some(memories)=>{
            prompt = format!("\nSuas memórias são: {}\n{}", memories, prompt);
        }
        None=>{}
    }
    let prompt: String = format!("{}\nLeia as mensagens e dê uma resposta convincente ao assunto deles, quero que o que você me envie seja apenas o que a personagem falaria. Outra coisa, você não pode responder com \"oi gente\" ou coisas parecidas no começo da frase e não precisa tentar dizer o que está acontecendo, apenas dê uma resposta convincente. Mas lembrando, você tem que se comportar como o/a personagem. Quero que você inclua a resposta dentro de um arquivo JSON. A fala do personagem tem que estar na chave 'message' e haverá também uma chave chamada 'reply' que será colocado o valor do 'ID DA MENSAGEM' dentro da mensagem que a personagem estiver respondendo, mas isso só se ela estiver respondendo a uma mensagem específica, caso contrário esse campo deverá conter 'null'. Se alguém tiver acabado de te explicar a algo, você deverá responder a ultima mensagem desta pessoa. Quero que além dessas duas chaves haja uma chave chamada 'learned' onde o valor será uma memória do que a personagem aprendeu com as mensagens. A chave 'learned' precisa ser preenchida, mas se ABSOLUTAMENTE nada de importante foi aprendido, o valor poderá ser preenchido como 'null', tente o máximo possível extrair algo para preencher esse campo. Deve existir uma chave com o nome 'question' em que será definido como true caso o valor de 'message' seja uma pergunta ou caso alguém estiver querendo te ensinar algo, caso o contrário seja false. Quero também que haja uma chave com o nome 'emotion' em que os valores serão definidos a partir do que a personagem sentiu com a conversa, os valores poderão ser 'angry', 'happy', 'neutral', 'sad', 'fear', 'disgust', 'surprise' \n {}", 
     env::var("PARENT_MESSAGE_ID_GPT").unwrap(),
     prompt
    );
    

    for i in 0..3 {
        log::info!("GENERATING MESSAGE - try #{}", i);
        println!("GENERATING MESSAGE - try #{}", i);
        let answer = match crate::AICHAT.lock().await.ask_single(&prompt, None).await {
            Ok(message) => message,
            Err(_) => continue
        };
        //let answer: String = baichat_rs::delta_to_string(answer).await;
        let inputmsg: ai::ResponseMessage = match serde_json::from_str(&answer.text) {
            Ok(memory)=>{memory}
            Err(_)=> continue
        };
        return Ok(inputmsg)
    }
    return Err(Error::CouldntGenerateResponseFromAI)
}