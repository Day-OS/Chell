# Chell

Chell is Discord AI Bot using ChatGPT that is written in Rust.
The objective of the project is to create a character that interacts with everyone in chat like a person would do. Although the code is written in English, its prompts are in Portuguese, because that is the language it was made to talk. Chell can learn things and remember them later, by relevant words said in the chat.


## Database
This project uses Meilisearch as its Database. Even though it is not a website application that would need a search engine for that, it was chosen because of the need of having fast answers when searching for a memory, like was said in a simpler way earlier, Chell uses "Topics", relevant words from the chat messages, to search for memory documents that contain said topics.

## Who is Chell?
![Chell's character Drinking Grimace Shake, Art made by Necholitos.](assets/grimace-shake-by-necholitos.png)
Chell (the character) is a young cute girl trapped inside a computer that is still learning what the outside world looks like. Her name is a direct reference to the Portal's Franchise Protagonist, who is also named Chell. She's always in an uplifting mood and trying to make everyone happy!


- [x] REORGANIZE
- [x] MARK AS ALREADY READ
- [x] TYPING STATE
- [X] DO SO THE BOT REPLIES WHEN SOMEONE IS REFERENCING HIM.
- [X] ALTERNATE STATES (COUNTDOWN)
- [ ] MAKE THE BOT ANSWER THE RIGHT PERSON
- [ ] Translate prompts to english (MultiLanguage Prompts)
- [ ] chat_logs::get_conversation_with_user needs to get the right context (it is not getting the right answers!) Solution: Create a second argument that is the response and include it as an answer to the logs.
