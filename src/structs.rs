use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct Response {
    pub candidates: [Candidates; 1],

    pub usageMetadata: UsageMetadata,
    pub error: Error,
}

impl Default for Response {
    fn default() -> Self {
        Response {
            candidates: [Candidates::default()],
            usageMetadata: UsageMetadata::default(),
            error: Error::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct Candidates {
    pub content: Content,
    pub finishReason: String,
    pub index: i32,
    pub safetyRatings: [SafetyRatings; 4],
}

impl Default for Candidates {
    fn default() -> Self {
        Candidates {
            content: Content::default(),
            finishReason: String::from("Unknown"),
            index: -1,
            safetyRatings: [
                SafetyRatings {
                    category: String::from(""),
                    probability: String::from(""),
                },
                SafetyRatings {
                    category: String::from(""),
                    probability: String::from(""),
                },
                SafetyRatings {
                    category: String::from(""),
                    probability: String::from(""),
                },
                SafetyRatings {
                    category: String::from(""),
                    probability: String::from(""),
                },
            ],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct Content {
    pub role: String,
    pub parts: [Parts; 1],
}

impl Default for Content {
    fn default() -> Self {
        Content {
            role: String::from(""),
            parts: [Parts::default()],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct Contents {
    pub role: String,
    pub parts: Parts,
}

impl Default for Contents {
    fn default() -> Self {
        Contents {
            role: String::from(""),
            parts: Parts::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct Conversation {
    pub contents: Vec<Contents>,
    pub safety_settings: [SafetySettings; 4],
}

impl Conversation {
    pub fn add_message(&mut self, msg: Contents) {
        let _ = self.contents.push(msg);
    }

    pub fn revert(&mut self) {
        tracing::info!("Deleting last user message");
        let _ = self.contents.pop();
    }

    pub fn get_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }

    pub fn reset_conversation(&mut self) {
        let _ = self.contents.clear();
    }

    pub fn delete_old(&mut self) {
        while self.contents.len() >= 20 {
            self.contents.remove(0);
        }
    }
}

impl Default for Conversation {
    fn default() -> Self {
        Conversation {
            contents: Vec::new(),
            safety_settings: [
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
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct Parts {
    pub text: String,
}

impl Default for Parts {
    fn default() -> Self {
        Parts {
            text: String::from(""),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct FileData {
    pub mimeType: String,
    pub fileUri: String,
}

impl Default for FileData {
    fn default() -> Self {
        FileData {
            mimeType: String::from(""),
            fileUri: String::from(""),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct PromptFeedback {
    pub blockReason: String,
    pub safetyRatings: [SafetyRatings; 4],
}

impl Default for PromptFeedback {
    fn default() -> Self {
        PromptFeedback {
            blockReason: String::from(""),
            safetyRatings: [
                SafetyRatings {
                    category: String::from(""),
                    probability: String::from(""),
                },
                SafetyRatings {
                    category: String::from(""),
                    probability: String::from(""),
                },
                SafetyRatings {
                    category: String::from(""),
                    probability: String::from(""),
                },
                SafetyRatings {
                    category: String::from(""),
                    probability: String::from(""),
                },
            ],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct SafetyRatings {
    pub category: String,
    pub probability: String,
}

impl Default for SafetyRatings {
    fn default() -> Self {
        SafetyRatings {
            category: String::from(""),
            probability: String::from(""),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct UsageMetadata {
    pub promptTokenCount: i32,
    pub candidatesTokenCount: i32,
    pub totalTokenCount: i32,
}

impl Default for UsageMetadata {
    fn default() -> Self {
        UsageMetadata {
            promptTokenCount: -1,
            candidatesTokenCount: -1,
            totalTokenCount: -1,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct Error {
    pub code: i32,
    pub message: String,
    pub status: String,
}

impl Default for Error {
    fn default() -> Self {
        Error {
            code: -1,
            message: String::from("Unknown error"),
            status: String::from(""),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct SafetySettings {
    pub category: String,
    pub threshold: String,
}

impl Default for SafetySettings {
    fn default() -> Self {
        SafetySettings {
            category: String::from(""),
            threshold: String::from(""),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct InlineData {
    pub data: String,
    pub mimeType: String,
}

impl Default for InlineData {
    fn default() -> Self {
        InlineData {
            data: String::from(""),
            mimeType: String::from(""),
        }
    }
}
