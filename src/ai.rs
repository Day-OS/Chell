use crate::{baichat_rs::{self, ThebAI, Delta}, memory_core, results};
use std::env;



pub fn get_ai()-> ThebAI {
    baichat_rs::ThebAI::new(Some(&env::var("PARENT_MESSAGE_ID_GPT").unwrap()))
}

pub async fn interpret_and_memorize(logs: String, timestamp: u64) -> Result<String, results::Error>{
    let prompt: String = format!("{}\nLeia as mensagens e pense sobre o que você aprendeu com o assunto deles. Se você não aprendeu nada relevante ou não compreendeu totalmente o assunto, envie um arquivo json com apenas uma chave chamada 'relevancy' com o valor false. Caso contrário, envie um arquivo JSON em que tenha a chave 'relevancy' com o valor true e outra chave, esta por sua vez chamada 'content', com o que a personagem aprendeu com o conteúdo nas mensagens dentro. Mas lembrando, o conteúdo na chave 'content' precisa ser fiel ao personagem.\n {}",
     env::var("PARENT_MESSAGE_ID_GPT").unwrap(),
     logs
    );
    
    let mut ai: baichat_rs::ThebAI = get_ai();
    let answer: Vec<Delta> = ai.ask(&prompt, Some(env::var("PARENT_MESSAGE_ID_GPT").unwrap())).await.expect("answer");
    let answer = answer[answer.len() - 1].text.clone();
    match memory_core::save_memory_to_db(&answer, timestamp).await {
        Ok(result) =>{Ok(answer)},
        Err(e) => Err(e),
    }
}

pub async fn say(logs: String) -> Result<String, String>{
    //let prompt: String = format!("{}\nLeia as mensagens e dê uma resposta convincente ao assunto deles, quero que o que você me envie seja apenas o que a personagem falaria. Outra coisa, você não pode responder com \"oi gente\" ou coisas parecidas no começo da frase e não precisa tentar dizer o que está acontecendo, apenas dê uma resposta convincente. Mas lembrando, você tem que se comportar como o/a personagem.\n {}", 

    let prompt: String = format!("{}\nLeia as mensagens e dê uma resposta convincente ao assunto deles, quero que o que você me envie seja apenas o que a personagem falaria. Outra coisa, você não pode responder com \"oi gente\" ou coisas parecidas no começo da frase e não precisa tentar dizer o que está acontecendo, apenas dê uma resposta convincente. Mas lembrando, você tem que se comportar como o/a personagem. Quero que você inclua a resposta dentro de um arquivo JSON. A fala do personagem tem que estar na chave 'message' e haverá também uma chave chamada 'reply' que será colocado o valor do 'ID DA MENSAGEM' dentro da mensagem que a personagem estiver respondendo, mas isso só se ela estiver respondendo a uma mensagem específica, caso contrário esse campo deverá conter 'null' \n {}", 
     env::var("PARENT_MESSAGE_ID_GPT").unwrap(),
     logs
    );
    
    let mut ai: baichat_rs::ThebAI = get_ai();
    let answer: Vec<Delta> = ai.ask(&prompt, Some(env::var("PARENT_MESSAGE_ID_GPT").unwrap())).await.expect("answer");
    let answer = answer[answer.len() - 1].text.clone();
    Ok(answer)
}

pub fn remember(){
    let mut prompt: String = format!("{}\nLeia as mensagens e coloque palavras chaves que sejam relevantes do assunto dentro de um array em uma linguagem de programação, apenas uma palavra importante por vez, quero que o que você me envie apenas um arquivo json com uma chave chamada 'query' e um array contendo as palavras chaves como valor dessa chave. (NÃO COLOQUE A RESPOSTA ENTRE ASPAS). Mas lembrando, você tem que se comportar como o/a personagem.\\n", env::var("PARENT_MESSAGE_ID_GPT").unwrap());
}