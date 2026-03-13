use crate::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

const MODEL: &str = "llama-3.3-70b-versatile";
const MAX_RETRIES: u32 = 3;

async fn post_groq(
    client: &reqwest::Client,
    api_key: &str,
    messages: Vec<Message>,
) -> Result<String, AppError> {
    let url = "https://api.groq.com/openai/v1/chat/completions";
    let body = ChatRequest {
        model: MODEL.to_string(),
        messages,
    };

    let mut last_err = String::new();

    for attempt in 1..=MAX_RETRIES {
        let res = client
            .post(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body)
            .send()
            .await;

        match res {
            Err(e) => {
                last_err = format!("Request failed: {}", e);
                eprintln!("⚠️ Attempt {}/{} failed: {}", attempt, MAX_RETRIES, last_err);
                tokio::time::sleep(std::time::Duration::from_secs(attempt as u64)).await;
                continue;
            }
            Ok(response) => {
                let status = response.status();
                let text = response
                    .text()
                    .await
                    .map_err(|e| AppError::ApiError(e.to_string()))?;

                if !status.is_success() {
                    last_err = format!("Groq API Error ({}): {}", status, text);
                    eprintln!("⚠️ Attempt {}/{}: {}", attempt, MAX_RETRIES, last_err);
                    tokio::time::sleep(std::time::Duration::from_secs(attempt as u64)).await;
                    continue;
                }

                let data: ChatResponse =
                    serde_json::from_str(&text).map_err(|e| {
                        AppError::ApiError(format!(
                            "JSON Parse Error: {}. Response: {}",
                            e, text
                        ))
                    })?;

                return Ok(data.choices[0].message.content.clone());
            }
        }
    }

    Err(AppError::ApiError(format!(
        "All {} retries failed. Last error: {}",
        MAX_RETRIES, last_err
    )))
}

pub async fn call_ai(prompt: &str, api_key: &str) -> Result<String, AppError> {
    let client = reqwest::Client::new();
    let messages = vec![
        Message {
            role: "system".to_string(),
            content: "You are Jarvis, a Rust code assistant. \
                Stay focused on Rust and this project only. \
                When there are cargo errors, analyze them and suggest exact fixes. \
                Answer concisely in Thai."
                .to_string(),
        },
        Message {
            role: "user".to_string(),
            content: prompt.to_string(),
        },
    ];
    post_groq(&client, api_key, messages).await
}

pub async fn summarize_chats(history: &str, api_key: &str) -> Result<String, AppError> {
    let client = reqwest::Client::new();
    let messages = vec![
        Message {
            role: "system".to_string(),
            content: "Summarize the conversation below into a concise paragraph. \
                Focus on: user's name, preferences, and technical topics discussed. \
                Write in third person."
                .to_string(),
        },
        Message {
            role: "user".to_string(),
            content: history.to_string(),
        },
    ];
    post_groq(&client, api_key, messages).await
}
