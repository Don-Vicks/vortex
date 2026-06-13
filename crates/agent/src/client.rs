use crate::{AiProvider, AgentConfig};
use anyhow::Result;
use reqwest::Client;
use serde_json::{json, Value};
use std::env;

pub async fn call_agent(config: &AgentConfig, prompt: &str) -> Result<String> {
    match config.provider {
        AiProvider::Anthropic => call_anthropic(&config.model, prompt).await,
        AiProvider::OpenAI => call_openai(&config.model, prompt).await,
        AiProvider::Gemini => call_gemini(&config.model, prompt).await,
        AiProvider::Grok => call_grok(&config.model, prompt).await,
    }
}

async fn call_anthropic(model: &str, prompt: &str) -> Result<String> {
    let api_key = env::var("CLAUDE_API_KEY").expect("CLAUDE_API_KEY must be set");
    let client = Client::new();

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&json!({
            "model": model,
            "max_tokens": 1000,
            "messages": [{"role": "user", "content": prompt}]
        }))
        .send()
        .await?;

    let data: Value = response.json().await?;
    let text = data["content"][0]["text"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No text in Anthropic response"))?
        .to_string();

    Ok(text)
}

async fn call_openai(model: &str, prompt: &str) -> Result<String> {
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let client = Client::new();

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
            "model": model,
            "messages": [{"role": "user", "content": prompt}]
        }))
        .send()
        .await?;

    let data: Value = response.json().await?;
    let text = data["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No text in OpenAI response"))?
        .to_string();

    Ok(text)
}

async fn call_gemini(model: &str, prompt: &str) -> Result<String> {
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
    let client = Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1/models/{}:generateContent?key={}",
        model, api_key
    );

    let response = client
        .post(url)
        .json(&json!({
            "contents": [{
                "parts": [{"text": prompt}]
            }]
        }))
        .send()
        .await?;

    let data: Value = response.json().await?;
    let text = data["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No text in Gemini response"))?
        .to_string();

    Ok(text)
}

async fn call_grok(model: &str, prompt: &str) -> Result<String> {
    let api_key = env::var("GROK_API_KEY").expect("GROK_API_KEY must be set");
    let client = Client::new();

    let response = client
        .post("https://api.x.ai/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
            "model": model,
            "messages": [{"role": "user", "content": prompt}]
        }))
        .send()
        .await?;

    let data: Value = response.json().await?;
    let text = data["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No text in Grok response"))?
        .to_string();

    Ok(text)
}
