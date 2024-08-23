mod structs;

use crate::structs::*;
use serenity::all::ActivityData;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use tracing::{error, info};

struct Handler {
    gemini_api_key: String,
    conversation: Mutex<Conversation>,
    url: String,
    client: reqwest::Client,
}

impl Handler {
    pub async fn reset_conversation(&self) {
        info!("Reseting history...");
        let mut local_conversation = self.conversation.lock().await;
        local_conversation.reset_conversation();
    }

    pub async fn send_msg_to_gemini(
        &self,
        message: String,
        image_base64: String,
        content_type: String,
    ) -> (String, i32) {
        info!("Forwarding message to gemini...");
        let json_to_send: String;

        // lock the conversation struct that holds history and add the new message to it
        let mut local_conversation = self.conversation.lock().await;

        // instance struct that will store the user's message
        let user_content = Contents {
            role: "user".to_string(),
            parts: Parts {
                text: message.to_string(),
            },
        };

        info!("Adding user's message to history...");
        local_conversation.add_message(user_content);

        info!("Content type is: {}", content_type);
        if content_type == "text" {
            // convert the entire conversation to json string
            json_to_send = match local_conversation.get_json() {
                Ok(text) => text,
                Err(error) => {
                    error!("Error converting to json: {}", error);
                    // messages_to_send.push(String::from("Error creating json of user's message"));
                    local_conversation.revert();
                    return ("Error creating json of user's message".to_string(), -1);
                }
            };
        } else {
            json_to_send = format!(
                r#"
                {{
                    "contents": {{
                        "role": "user",
                        "parts": [
                            {{
                                "inlineData": {{
                                    "data": "{}",
                                    "mimeType": "{}"
                                }}
                            }},
                            {{
                                "text": "{}"
                            }}
                        ]
                    }},
                    "safety_settings": [
                        {{
                            "category": "HARM_CATEGORY_SEXUALLY_EXPLICIT",
                            "threshold": "BLOCK_NONE"
                        }},
                        {{
                            "category": "HARM_CATEGORY_HATE_SPEECH",
                            "threshold": "BLOCK_NONE"
                        }},
                        {{
                            "category": "HARM_CATEGORY_HARASSMENT",
                            "threshold": "BLOCK_NONE"
                        }},
                        {{
                            "category": "HARM_CATEGORY_DANGEROUS_CONTENT",
                            "threshold": "BLOCK_NONE"
                    }}
                    ]
                }}
                "#,
                image_base64, content_type, message
            );
        }
        // println!("{}", &json_to_send);
        // println!("{}", local_conversation.contents.len());
        info!("size in kb: {}", json_to_send.len() as f32 / 1024.0);

        // send the POST request and get the response
        info!("Sending POST request...");
        let post_request = self
            .client
            .post(&self.url)
            .body(json_to_send)
            .header("Content-Type", "application/json")
            .send()
            .await;

        // check if it was successful
        info!("Getting response to POST request...");
        let response: reqwest::Response = match post_request {
            Ok(res) => res,
            Err(error) => {
                let err_msg = "Error sending POST request to gemini";
                error!("{}: {}", err_msg, error);
                local_conversation.revert();
                return (err_msg.to_string(), -1);
            }
        };

        info!("Getting string from POST request response...");
        let response_json = match response.text().await {
            Ok(text) => text,
            Err(error) => {
                let err_msg = "Error getting text from gemini's POST request's response";
                error!("{}: {}", err_msg, error);
                local_conversation.revert();
                return (err_msg.to_string(), -1);
            }
        };

        // println!("{}", response_json);

        // deserializes the json received as reply
        info!("Deserializing string from POST request response...");
        let response_json: Response = match serde_json::from_str(&&response_json) {
            Ok(res) => res,
            Err(error) => {
                let err_msg = "Error deserializing json received from gemini";
                error!("{}: {}", err_msg, error);
                local_conversation.revert();
                return (err_msg.to_string(), -1);
            }
        };

        if !&response_json.candidates.is_empty()
            && &response_json.candidates[0].finishReason == "STOP"
        {
            info!("Successful response from gemini");
            let response_text: &str = &response_json.candidates[0].content.parts[0].text.clone();

            let gemini_response = Contents {
                role: "model".to_string(),
                parts: Parts {
                    text: response_text.to_string(),
                },
            };

            info!("Adding bot's reply to history...");
            local_conversation.add_message(gemini_response);

            local_conversation.delete_old();

            return (
                response_text.to_string(),
                response_json.usageMetadata.totalTokenCount,
            );
        }
        // if safety trigger
        else if !&response_json.candidates.is_empty()
            && &response_json.candidates[0].finishReason == "SAFETY"
        {
            local_conversation.revert();
            return ("https://i.imgur.com/DJqE6wq.jpeg".to_string(), -1);
        }
        // other unknown response
        else {
            error!("Unknown error: {}", response_json.error.message);
            let error_message = response_json
                .error
                .message
                .replace(&self.gemini_api_key, "API KEY");
            local_conversation.revert();
            return (error_message, -1);
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let bot_id = ctx.cache.current_user().id;
        if msg.author.id == bot_id {
            return;
        }
        let discord_bot_id = format!("<@{}>", bot_id);

        if msg.author.id == 202850246261211136 {
            if msg.content == "!resetgemini" {
                info!("Reseting conversation...");
                self.reset_conversation().await;
                ctx.set_presence(
                    Option::from(ActivityData::custom("Tokens: 0")),
                    Default::default(),
                );
                if let Err(why) = msg.channel_id.broadcast_typing(&ctx.http).await {
                    error!("Error sending typing: {why:?}");
                }
                if let Err(why) = msg
                    .channel_id
                    .say(&ctx.http, "Conversation has been reset!")
                    .await
                {
                    error!("Error sending message: {why:?}");
                }
                return;
            }
        }
        // checks if mentioned
        let mut mentioned: bool = false;
        for mention in &msg.mentions {
            if mention.id == bot_id {
                mentioned = true;
            }
        }

        // check if starts with question mark
        let mut question_mark: bool = false;
        if msg.content.starts_with("? ") {
            question_mark = true;
        }

        if mentioned || question_mark {
            // removes mention from message
            let mut no_mention_msg = msg.content.replace(&discord_bot_id, "");
            if no_mention_msg.chars().next() == Some(' ') {
                no_mention_msg = no_mention_msg[1..].to_string();
            } else if question_mark {
                no_mention_msg = no_mention_msg[2..].to_string();
            }

            // sends typing indicator thing to discord
            if let Err(why) = msg.channel_id.broadcast_typing(&ctx.http).await {
                error!("Error sending typing: {why:?}");
            }

            // sets these values to default, will be used later if there is image attachment
            let mut base64: String = "".to_string();
            let mut content_type: String = "text".to_string();

            // checks if there is attachment and grabs the first one
            if let Some(attachment) = msg.attachments.get(0) {
                info!("Attachment found: {:?}", attachment);
                // gets the attachment content type
                content_type = match &attachment.content_type {
                    Some(value) => value.to_string(),
                    None => {
                        // returns if for some reason there is no content type
                        let err_msg = "Could not find content type of attachment";
                        if let Err(err) = msg.reply(&ctx.http, err_msg).await {
                            error!("Error sending message: {err:?}");
                        };
                        return;
                    }
                };
                // check if attachment is in supported format
                let wtf: &str = &content_type;
                if ["image/jpg", "image/jpeg", "image/png"].contains(&wtf) {
                    // download the attachment
                    info!("Received an image as attachment, downloading...");
                    let content = match attachment.download().await {
                        Ok(content) => content,
                        Err(err) => {
                            // if for some reason download fails
                            error!("{:?}", err);
                            if let Err(err) =
                                msg.reply(&ctx.http, "Error downloading attachment").await
                            {
                                error!("Error sending message: {err:?}");
                            };
                            return;
                        }
                    };
                    // converts to base64
                    info!("Converting to base64...");
                    base64 = base64::encode(content);
                    info!("Size is: {}", base64.len());
                } else {
                    if let Err(err) = msg
                        .reply(&ctx.http, "Unsupported attachment type".to_string())
                        .await
                    {
                        error!("{err:?}");
                    }
                    return;
                }
            }
            let response = self
                .send_msg_to_gemini(no_mention_msg, base64, content_type)
                .await;
            let chunks = split_string(&response.0);
            for (_, part) in chunks.iter().enumerate() {
                if let Err(why) = msg.reply(&ctx.http, part).await {
                    error!("Error sending message: {why:?}");
                }
            }
            // if answer was really successful
            if response.1 != -1 {
                let status = format!("Tokens: {}", response.1);
                ctx.set_presence(
                    Option::from(ActivityData::custom(status)),
                    Default::default(),
                );
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_presence(
            Option::from(ActivityData::custom("Tokens: 0")),
            Default::default(),
        );
        let msg = format!("{} is connected!", ready.user.name);
        info!(msg);
        println!("{}", msg);
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
    // let mut vec: Vec<i32> = vec![];

    // for i in 0..10 {
    //     if vec.len() >= 5 {
    //         vec.remove(0);
    //     }

    //     vec.push(i);

    //     for element in vec.iter() {
    //         print!("{}, ", element);
    //     }
    // }

    let file_appender = tracing_appender::rolling::daily("log", "app.log");

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG) // Set max level to DEBUG
        .with_writer(file_appender)
        .with_ansi(false)
        .init();

    // loads dotenv values
    dotenv::dotenv().ok();
    info!("Starting...");
    let gemini_api_key = std::env::var("GEMINI_API_KEY").expect("Gemini API key missing from env");
    let discord_token = std::env::var("DISCORD_TOKEN").expect("Discord API key missing from env");

    // set gateway intents which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // this will store the conversation history with gemini
    let conversation = Conversation::default();

    let handler = Handler {
        gemini_api_key: gemini_api_key.clone(),
        conversation: Mutex::new(conversation),
        url: format!("https://generativelanguage.googleapis.com/v1/models/gemini-1.5-flash-001:generateContent?key={}", gemini_api_key),
        client: reqwest::Client::new()
    };

    // creates discord bot client
    let mut client = Client::builder(&discord_token, intents)
        .event_handler(handler)
        .await
        .expect("Error creating client");

    client.start().await.expect("Error starting client");
}
