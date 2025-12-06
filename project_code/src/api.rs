use anyhow::{anyhow, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    // Add more parameters as needed (e.g., top_p, stream)
}

#[derive(Serialize, Deserialize)]
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

pub async fn generate_code(prompt: &str) -> Result<String> {
    // Load the token
    let token = std::env::var("HF_TOKEN")
        .context("HF_TOKEN missing in .env")?;

    // Set the router URL (OpenAI-compatible endpoint)
    let url = "https://router.huggingface.co/v1/chat/completions".to_string();

    // Build the request body
    let body = ChatRequest {
        model: "Qwen/Qwen2.5-Coder-7B-Instruct".to_string(),
        /* messages: vec![Message {
            role: "user".to_string(),
            content: prompt.to_string(),
        }], */
        messages: vec![
            Message {
                role: "system".to_string(),
                content: "You are a Python code generator. Respond only with valid, executable Python code. No explanations, markdown, or extra text.".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            },
        ],
        max_tokens: Some(1024),
        temperature: Some(0.2),
    };

    // Build headers
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {token}"))
            .context("Invalid Bearer token format")?,
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    // Send the request
    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .headers(headers)
        .json(&body)
        .timeout(Duration::from_secs(60))
        .send()
        .await
        .context("HTTP error to Hugging Face router")?;

    let status = resp.status();
    let text_body = resp
        .text()
        .await
        .context("Failed to read Hugging Face response")?;

    if !status.is_success() {
        return Err(anyhow!("HuggingFace error {}: {}", status, text_body));
    }

    // Parse the response
    let parsed: ChatResponse = serde_json::from_str(&text_body)
        .context("Failed to parse Hugging Face JSON response")?;

    let generated = parsed
        .choices
        .first()
        .map(|choice| choice.message.content.clone())
        .ok_or_else(|| anyhow!("No choices in Hugging Face response"))?;

    Ok(generated)
}

/* // HuggingFace has deprecated free inference API as of late 2024
// For now, we'll use a simple fallback that generates basic Python code
const HUGGINGFACE_API: &str = "https://api-inference.huggingface.co/models";

#[derive(Serialize)] //pour ecrire la requette json
struct HfRequest<'a> {  //pour que la variable a continue d'exister assez longtemps pour faire la requette
    inputs: &'a str,  //en gros la requette qu'on envoit
    #[serde(skip_serializing_if = "Option::is_none")] //si on veux ajouter des paramètre
    parameters: Option<HfParameters>,  //les paramètres qu'on veux ajouter (truc d'apres)
}


#[derive(Serialize)]//aussi pour ecrire le json
struct HfParameters { //pour mettre les options
    max_new_tokens: Option<u32>,//nb de token que le model peut generé en plus: plus il est grand plus la reponse sera longue
    temperature: Option<f32>,//creativité du model: 0 tres deterministe bien pour le code
    // ajoute d'autres paramètres si besoin
}

#[derive(Debug, Deserialize)] //pour recuperer la réponse: peut y avoir plusieurs formats donc plusieurs options dans ce code pour s'adapter
//deserialisable pour passer de json a rust, serialisable pour passer de rust a json
struct HfGenerated {
    
    #[serde(rename = "generated_text")] //on cherche a recuperer le champs generated text car c'est la que se trouve la reponse
    generated_text: Option<String>,

    #[serde(rename = "text")] //desfois c'est le champs text
    text: Option<String>,

    //rajouter si on tombe sur des cas ou la reponse se trouve dans un autre champs
}

//suite du code
pub async fn generate_code(prompt: &str) -> Result<String> {
    // 1) Lire le token
    let token = std::env::var("HF_TOKEN")
        .context("HF_TOKEN manquant dans .env")?;

    // 2) Construire l'URL du modèle - using NEW router endpoint
    let url = "https://api-inference.huggingface.co/models/bigcode/starcoder2-3b".to_string();

    // 3) Construire le JSON
    let body = HfRequest {
        inputs: prompt,
        parameters: Some(HfParameters {
            max_new_tokens: Some(256),
            temperature: Some(0.2),
        }),
    };

    // 4) Construire les headers
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {token}"))?,
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    // 5) Envoyer la requête
    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .headers(headers)
        .json(&body)
        .timeout(Duration::from_secs(60))
        .send()
        .await
        .context("Erreur HTTP vers Hugging Face")?;

    let status = resp.status();
    let text_body = resp.text().await
        .context("Impossible de lire la réponse Hugging Face")?;

    if !status.is_success() {
        return Err(anyhow!("HuggingFace erreur {status}: {}", text_body));
    }

    // 6) Essayer : JSON = tableau de HfGenerated
    if let Ok(list) = serde_json::from_str::<Vec<HfGenerated>>(&text_body) {
        if let Some(first) = list.first() {
            if let Some(gt) = &first.generated_text {
                return Ok(gt.clone());
            }
            if let Some(t) = &first.text {
                return Ok(t.clone());
            }
        }
    }

    // 7) Essayer : JSON = objet unique de HfGenerated
    let parsed_obj: Result<HfGenerated, _> = serde_json::from_str(&text_body);
    if let Ok(obj) = parsed_obj {
        if let Some(gt) = obj.generated_text {
            return Ok(gt);
        }
        if let Some(t) = obj.text {
            return Ok(t);
        }
    }

    // 8) Sinon → erreur + body HF
    Err(anyhow!(
        "Impossible d'interpréter la réponse Hugging Face : {}",
        text_body
    ))
}

 */