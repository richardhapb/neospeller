use reqwest::blocking::Client;

use serde::Deserialize;
use serde_json::json;
use std::env;

/// OpenAI response format

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
    index: i32,
    finish_reason: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Message {
    role: String,
    content: String,
}

pub fn check_grammar(json_data: &str, language: &str) -> Result<String, Box<dyn std::error::Error>> {
    let openai_token = env::var("OPENAI_API_KEY")?;

    let initial_prompt = format!(
        r#"I will send you a JSON containing comments from a {} source file. Your task is to check the grammar and ensure that the comments are straightforward, clear, and concise. Respond in the same JSON format, including the line number and the corrected text.

- Do not remove or add punctuation at the end of lines unless necessary for clarity
- Do not remove formatters such as '-' or '*'; preserve the original formatting and change only the text when necessary.
- Do not change the line numbers for each comment, mantain the original line numbers.
- Keep quotes if they exist, along with language syntax such as commas, brackets, etc.
- Do not replace variable names like line_number to line number
- Do not mix single-line comments with multi-line comments; keep them separate."#,
        language
    );

    let url = env::var("OPENAI_API_BASE_URL").unwrap_or_else(|_| "https://api.openai.com".to_string());
    let client = Client::new();

    let res = client
        .post(format!("{}/v1/chat/completions", url))
        .header("Authorization", format!("Bearer {}", openai_token))
        .header("Content-Type", "application/json")
        .json(&json!({
            "model": "gpt-4o-mini-2024-07-18",
            "messages": [
                {
                    "role": "system",
                    "content": initial_prompt
                },
                {
                    "role": "user",
                    "content": json_data
                }
            ],
            "max_completion_tokens": 2000,
            "temperature": 0.5,
            "response_format": {"type": "json_object"}
        }))
        .send()?;

    let response_text = res.text()?;
    let response: OpenAIResponse = serde_json::from_str(&response_text)?;

    // Return the content string from the first choice
    if let Some(choice) = response.choices.first() {
        Ok(choice.message.content.clone().replace("\n", ""))
    } else {
        Err("No choices found in the response".into())
    }
}
