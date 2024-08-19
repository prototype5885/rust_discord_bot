mod structs;

use base64;
use dotenv::dotenv;

use crate::structs::*;
use serenity::all::{ActivityData, Attachment};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

struct Handler {
    api_key: String,
    conversation: Mutex<Conversation>,
}

impl Handler {
    pub async fn reset_conversation(&self) {
        let mut local_conversation = self.conversation.lock().await;
        local_conversation.reset_conversation();
    }

    pub async fn send_msg_to_gemini(&self, message: &str) -> String {
        // instance struct that will store the user's message
        let user_content = Contents {
            role: "user".to_string(),
            parts: Parts {
                text: String::from(message),
            },
        };

        // lock the conversation struct that holds history and add the new message to it
        let mut local_conversation = self.conversation.lock().await;
        local_conversation.add_message(user_content);

        // convert the entire conversation to json string
        let conversation_json = match local_conversation.get_json() {
            Ok(text) => text,
            Err(error) => {
                println!("{}", error);
                // messages_to_send.push(String::from("Error creating json of user's message"));
                return "Error creating json of user's message".to_string();
            }
        };

        // create a new reqwest client
        let client = reqwest::Client::new();

        // the URL of the API endpoint
        let url = format!(
            "https://generativelanguage.googleapis.com/v1/models/gemini-pro:generateContent?key={}",
            &self.api_key
        );

        println!("{}", conversation_json);
        // println!("{}", conversation_json.len());

        // send the POST request and get the response
        let response_result = client
            .post(&url)
            .body(conversation_json)
            .header("Content-Type", "application/json")
            .send()
            .await;

        // check if it was successful
        let response: reqwest::Response = match response_result {
            Ok(res) => res,
            Err(error) => {
                println!("{}", error);
                return "Error sending POST request to gemini".to_string();
            }
        };

        let response_string = match response.text().await {
            Ok(text) => text,
            Err(error) => {
                println!("{}", error);
                return "Error getting text from gemini's POST request's response".to_string();
            }
        };

        // deserializes the json received as reply
        let response_json: Response = match serde_json::from_str(&&response_string) {
            Ok(res) => res,
            Err(error) => {
                println!("{}", error);
                return "Error deserializing json received from gemini".to_string();
            }
        };

        // if there is error response from gemini, this is not connection error
        if response_json.error.code == 400 {
            println!("{}", response_json.error.message);
            return "[400 Bad Request] Gemini API free tier is not available in your country <@202850246261211136>".to_string();
        }
        // if successful response
        else if !&response_json.candidates.is_empty()
            && &response_json.candidates[0].finishReason == "STOP"
        {
            let response_text: &str = &response_json.candidates[0].content.parts[0].text.clone();
            // println!("{}", response_text);

            let gemini_response = Contents {
                role: "model".to_string(),
                parts: Parts {
                    text: response_text.to_string(),
                },
            };

            local_conversation.add_message(gemini_response);
            return response_text.to_string();
        }
        // if safety trigger
        else if !&response_json.candidates.is_empty()
            && &response_json.candidates[0].finishReason == "SAFETY"
        {
            return "https://i.imgur.com/DJqE6wq.jpeg".to_string();
        }
        // other unknown response
        else {
            println!("{}", response_json.error.message);
            return "error".to_string();
        }
    }

    pub async fn send_img_to_gemini(&self, attachment: &&Attachment) -> String {
        println!("Downloading picture from discord...");
        let content = match attachment.download().await {
            Ok(content) => content,
            Err(why) => {
                println!("Error downloading attachment: {:?}", why);
                return "Error downloading attachment".to_string();
            }
        };
        println!("Converting to base64...");
        let base64 = base64::encode(content);
        println!("Size is: {}", base64.len());

        base64
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let bot_id = ctx.cache.current_user().id;
        let discord_bot_id = format!("<@{}>", bot_id);
        if msg.author.id == bot_id {
            return;
        }

        if msg.author.id == 202850246261211136 {
            if msg.content == "!resetgeminirust" {
                self.reset_conversation().await;
                println!("Reseting history...");
                if let Err(why) = msg.channel_id.broadcast_typing(&ctx.http).await {
                    println!("Error sending typing: {why:?}");
                }
                if let Err(why) = msg
                    .channel_id
                    .say(&ctx.http, "Conversation has been reset!")
                    .await
                {
                    println!("Error sending message: {why:?}");
                }
            }
        }

        let supported_img_types = ["image/jpg", "image/jpeg", "image/png"];

        for mention in &msg.mentions {
            if mention.id == bot_id {
                let no_mention_msg = &msg.content.replace(&discord_bot_id, "");
                for attachment in &msg.attachments {
                    // if asking about picture
                    if attachment.size > 0 {
                        let content_type = match &attachment.content_type {
                            Some(value) => value,
                            None => "",
                        };
                        if supported_img_types.contains(&content_type) {
                            println!("Received an image as attachment...");
                            if let Err(why) = msg.channel_id.broadcast_typing(&ctx.http).await {
                                println!("Error sending typing: {why:?}");
                                return;
                            }
                            let response: String = self.send_img_to_gemini(&attachment).await;
                        }
                    }
                    return;
                }
                println!("Forwarding message to gemini...");

                if let Err(why) = msg.channel_id.broadcast_typing(&ctx.http).await {
                    println!("Error sending typing: {why:?}");
                }
                let message_to_send = self.send_msg_to_gemini(no_mention_msg).await;
                let chunks = split_string(&message_to_send);
                for (_, part) in chunks.iter().enumerate() {
                    if let Err(why) = msg.reply(&ctx.http, part).await {
                        println!("Error sending message: {why:?}");
                    }
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_presence(
            Option::from(ActivityData::custom("Rust")),
            Default::default(),
        );
        println!("{} is connected!", ready.user.name);
    }
}

fn split_string(s: &String) -> Vec<String> {
    let max_len = 2000;
    // Ensure the string is non-empty and the max length is positive
    if s.is_empty() || max_len == 0 {
        return vec![];
    }

    // Create an empty vector to hold the string parts
    let mut parts = Vec::new();

    // Iterate over the string, taking slices of max_len size
    let mut start = 0;
    while start < s.len() {
        // Calculate the end of the slice, ensuring it doesn't exceed the string length
        let end = usize::min(start + max_len, s.len());

        // Push the slice as a new string into the vector
        parts.push(s[start..end].to_string());

        // Move the start index forward
        start += max_len;
    }
    parts
}

#[tokio::main]
async fn main() {
    // loads dotenv values
    dotenv().ok();
    let gemini_api_key = std::env::var("GEMINI_API_KEY").expect("Gemini API key missing from env");
    let discord_token = std::env::var("DISCORD_TOKEN").expect("Discord API key missing from env");

    // set gateway intents which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // this will store the conversation history with gemini
    let conversation = Conversation {
        contents: Vec::new(),
        safety_settings: vec![
            SafetySettings {
                category: String::from("HARM_CATEGORY_SEXUALLY_EXPLICIT"),
                threshold: String::from("BLOCK_NONE"),
            },
            SafetySettings {
                category: String::from("HARM_CATEGORY_HATE_SPEECH"),
                threshold: String::from("BLOCK_NONE"),
            },
            SafetySettings {
                category: String::from("HARM_CATEGORY_HARASSMENT"),
                threshold: String::from("BLOCK_NONE"),
            },
            SafetySettings {
                category: String::from("HARM_CATEGORY_DANGEROUS_CONTENT"),
                threshold: String::from("BLOCK_NONE"),
            },
        ],
    };

    let handler = Handler {
        api_key: gemini_api_key,
        conversation: Mutex::new(conversation),
    };

    // creates discord bot client
    let mut client = Client::builder(&discord_token, intents)
        .event_handler(handler)
        .await
        .expect("Error creating client");

    client.start().await.expect("Error starting client");

    // start listening
    // if let Err(why) = client.start().await {
    //     println!("Client error: {why:?}");
    // }
}
