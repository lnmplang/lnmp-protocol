//! OpenAI API Client for Real LLM Integration

use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChatMessage,
}

pub struct OpenAIClient {
    api_key: String,
    model: String,
}

impl OpenAIClient {
    pub fn new() -> Result<Self, String> {
        dotenv::dotenv().ok(); // Load .env file

        let api_key = env::var("OPENAI_API_KEY").map_err(|_| {
            "OPENAI_API_KEY not found in environment. Please set it in .env file.".to_string()
        })?;

        Ok(Self {
            api_key,
            model: "gpt-4o-mini".to_string(), // Using gpt-4o-mini for cost efficiency
        })
    }

    pub fn chat(&self, system_prompt: &str, user_message: &str) -> Result<String, String> {
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_message.to_string(),
            },
        ];

        let request = ChatRequest {
            model: self.model.clone(),
            messages,
            temperature: 0.7,
            max_tokens: 500,
        };

        let client = reqwest::blocking::Client::new();
        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .map_err(|e| format!("Failed to send request: {}", e))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("OpenAI API error: {}", error_text));
        }

        let chat_response: ChatResponse = response
            .json()
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        chat_response
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| "No response from OpenAI".to_string())
    }
}

impl Default for OpenAIClient {
    fn default() -> Self {
        Self::new().expect("Failed to initialize OpenAI client")
    }
}
