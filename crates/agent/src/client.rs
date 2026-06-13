use crate::{AiProvider, AgentConfig};
use anyhow::Result;
use reqwest::Client;
use serde_json::{json, Value};
use std::env;

const SYSTEM_PROMPT: &str = "You are a deterministic Jito tip calculator. Output ONLY valid JSON. \
    No commentary, no markdown fences, no explanation outside the JSON object. \
    You must pick a tip amount in lamports within the bounds provided.";

pub async fn call_agent(config: &AgentConfig, prompt: &str) -> Result<String> {
    let raw = match config.provider {
        AiProvider::Anthropic => call_anthropic(&config.model, prompt).await,
        AiProvider::OpenAI => call_openai(&config.model, prompt).await,
        AiProvider::Gemini => call_gemini(&config.model, prompt).await,
        AiProvider::Grok => call_grok(&config.model, prompt).await,
    }?;

    Ok(strip_markdown_fences(&raw))
}

/// Strip markdown code fences (```json ... ```) that LLMs commonly wrap around JSON.
fn strip_markdown_fences(s: &str) -> String {
    let trimmed = s.trim();
    let without_start = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .unwrap_or(trimmed);
    let without_end = without_start
        .trim()
        .strip_suffix("```")
        .unwrap_or(without_start);
    without_end.trim().to_string()
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
            "max_tokens": 200,
            "temperature": 0.0,
            "system": SYSTEM_PROMPT,
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
            "temperature": 0.0,
            "max_tokens": 200,
            "messages": [
                {"role": "system", "content": SYSTEM_PROMPT},
                {"role": "user", "content": prompt}
            ]
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
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    let response = client
        .post(url)
        .json(&json!({
            "generationConfig": {
                "temperature": 0.0,
                "maxOutputTokens": 1000
            },
            "systemInstruction": {
                "parts": [{"text": SYSTEM_PROMPT}]
            },
            "contents": [{
                "parts": [{"text": prompt}]
            }]
        }))
        .send()
        .await?;

    let data: Value = response.json().await?;
    let text = data["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No text in Gemini response. Raw response: {}", data))?
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
            "temperature": 0.0,
            "max_tokens": 200,
            "messages": [
                {"role": "system", "content": SYSTEM_PROMPT},
                {"role": "user", "content": prompt}
            ]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_markdown_fences_json() {
        let input = "```json\n{\"recommended_lamports\": 1000}\n```";
        assert_eq!(strip_markdown_fences(input), "{\"recommended_lamports\": 1000}");
    }

    #[test]
    fn test_strip_markdown_fences_plain() {
        let input = "```\n{\"foo\": 1}\n```";
        assert_eq!(strip_markdown_fences(input), "{\"foo\": 1}");
    }

    #[test]
    fn test_strip_markdown_fences_none() {
        let input = "{\"foo\": 1}";
        assert_eq!(strip_markdown_fences(input), "{\"foo\": 1}");
    }
}
