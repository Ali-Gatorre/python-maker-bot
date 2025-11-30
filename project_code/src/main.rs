use anyhow::{Context, Result};
use dotenvy::dotenv;
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;
use std::env;

#[derive(Serialize)]
struct HFRequest {
    inputs: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env (if present) for local development
    dotenv().ok();

    let hf_token = env::var("HF_TOKEN").context("HF_TOKEN environment variable not set")?;

    let body = HFRequest {
        inputs: "Write a Python script that prints 'hello'".to_string(),
    };

    let client = Client::new();
    let res = client
        .post("https://router.huggingface.co/hf-inference/models/bigcode/starcoder2-3b")
        .bearer_auth(hf_token)
        .json(&body)
        .send()
        .await
        .context("HTTP request failed")?;

    // Parse response as JSON and pretty-print it. Many HF endpoints return JSON.
    let json: Value = res.json().await.context("Failed to parse response JSON")?;
    println!("{}", serde_json::to_string_pretty(&json)?);

    Ok(())
}
