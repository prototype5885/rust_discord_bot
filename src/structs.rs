use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct Response {
    pub candidates: Vec<Candidates>,
    // promptFeedback: PromptFeedback,
    pub usageMetadata: UsageMetadata,
    pub error: Error,
}

impl Default for Response {
    fn default() -> Self {
        Response {
            candidates: Vec::new(),
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
    pub safetyRatings: Vec<SafetyRatings>,
}

impl Default for Candidates {
    fn default() -> Self {
        Candidates {
            content: Content::default(),
            finishReason: "Unknown".to_string(),
            index: -1,
            safetyRatings: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct Content {
    pub parts: Vec<Parts>,
    pub role: String,
}

impl Default for Content {
    fn default() -> Self {
        Content {
            parts: Vec::new(),
            role: "".to_string(),
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
            role: "".to_string(),
            parts: Parts::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct Conversation {
    pub contents: Vec<Contents>,
    pub safety_settings: Vec<SafetySettings>,
}

impl Conversation {
    pub fn add_message(&mut self, msg: Contents) {
        let _ = &self.contents.push(msg);
    }

    pub fn revert(&mut self) {
        tracing::info!("Deleting last user message");
        let _ = &self.contents.pop();
    }

    pub fn get_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }

    pub fn reset_conversation(&mut self) {
        let _ = &self.contents.clear();
    }
}

impl Default for Conversation {
    fn default() -> Self {
        Conversation {
            contents: Vec::new(),
            safety_settings: Vec::new(),
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
            text: "".to_string(),
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
            mimeType: "".to_string(),
            fileUri: "".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct PromptFeedback {
    pub blockReason: String,
    pub safetyRatings: Vec<SafetyRatings>,
}

impl Default for PromptFeedback {
    fn default() -> Self {
        PromptFeedback {
            blockReason: "".to_string(),
            safetyRatings: Vec::new(),
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
            category: "".to_string(),
            probability: "".to_string(),
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
            message: "Unknown error".to_string(),
            status: "".to_string(),
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
