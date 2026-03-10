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

pub async fn call_ai(prompt: &str, api_key: &str) -> Result<String, AppError> {
    let client = reqwest::Client::new();
    let url = "https://api.groq.com/openai/v1/chat/completions";

    let body = ChatRequest {
        model: "llama-3.3-70b-versatile".to_string(), // ตรวจสอบชื่อรุ่นอีกที
        messages: vec![
            Message {
                role: "system".to_string(),
                content: "You are Jarvis, a Rust expert assistant with full project context.".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            },
        ],
    };

    let res = client
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::ApiError(format!("Request failed: {}", e)))?;

    // --- ส่วนที่แก้ไขเพิ่ม ---
    let status = res.status();
    let text = res
        .text()
        .await
        .map_err(|e| AppError::ApiError(e.to_string()))?;

    if !status.is_success() {
        // ถ้า Error จะเห็นข้อความจริงๆ จาก Groq ที่นี่
        return Err(AppError::ApiError(format!(
            "Groq API Error ({}): {}",
            status, text
        )));
    }

    let response_data: ChatResponse = serde_json::from_str(&text).map_err(|e| {
        AppError::ApiError(format!("JSON Parse Error: {}. Response was: {}", e, text))
    })?;
    // ------------------------

    Ok(response_data.choices[0].message.content.clone())
}
// เพิ่มใน src/brain/api.rs
pub async fn summarize_chats(history: &str, api_key: &str) -> Result<String, AppError> {
    let prompt = format!(
        "Summarize the following technical discussion into a concise paragraph: \n{}",
        history
    );
    call_ai(&prompt, api_key).await
}
