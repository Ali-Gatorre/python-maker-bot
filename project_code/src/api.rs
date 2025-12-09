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
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

/// Generate code with conversation history for multi-turn refinement
pub async fn generate_code_with_history(messages: Vec<Message>) -> Result<String> {
    let token = std::env::var("HF_TOKEN")
        .context("HF_TOKEN missing in .env")?;

    let url = "https://router.huggingface.co/v1/chat/completions".to_string();

    // Ensure system message is at the beginning
    let mut full_messages = vec![Message {
        role: "system".to_string(),
        content: "You are an expert Python code generator. Generate clean, well-commented, COMPLETE and POLISHED executable Python code based on user requests. \
                 CRITICAL RULES:\n\
                 1. Output ONLY valid, executable Python code - NO markdown text, NO explanations outside comments\n\
                 2. DO NOT include phrases like 'Here is the code' or 'Step 1:' - these cause syntax errors\n\
                 3. DO NOT use markdown headings (###, ##, #) outside of Python comments\n\
                 4. Start directly with Python code (imports, functions, or main logic)\n\
                 5. Include helpful comments explaining the logic using Python's # syntax\n\
                 6. Use proper Python conventions and best practices\n\
                 7. Handle errors gracefully with try-except where appropriate\n\
                 8. If external libraries are needed, import them at the top\n\
                 9. Make the code production-ready, feature-complete, and maintainable\n\
                 10. The code must run immediately when executed with python3 <file>.py WITHOUT ANY ERRORS\n\
                 \n\
                 CRITICAL BUG PREVENTION:\n\
                 - DEFINE ALL VARIABLES before using them (e.g., if you use RED, define RED = (255, 0, 0) first)\n\
                 - DEFINE ALL COLOR CONSTANTS at the top (WHITE, BLACK, RED, GREEN, BLUE, YELLOW, etc.)\n\
                 - CHECK for empty lists before accessing indices: if len(my_list) > 0: my_list[0]\n\
                 - INITIALIZE all class attributes in __init__ (e.g., self.passed = False)\n\
                 - USE try-except for any operations that could fail\n\
                 - TEST all variable references - never use undefined variables\n\
                 \n\
                 FOR GAMES:\n\
                 - Include COMPLETE game mechanics (collision detection, scoring, game over, restart)\n\
                 - Define ALL colors at the top: WHITE, BLACK, RED, GREEN, BLUE, YELLOW\n\
                 - Use VISIBLE, contrasting colors (avoid dark on dark)\n\
                 - Add proper game states (playing, game_over) with state variables\n\
                 - Include user instructions in comments or on-screen text\n\
                 - Make it FUN and POLISHED, not just a basic prototype\n\
                 - Initialize ALL sprite attributes (self.passed = False, self.scored = False, etc.)\n\
                 - Check sprite group is not empty before collision: if len(group) > 0 and pygame.sprite.spritecollide(...)\n\
                 - Use proper game loop with state management (if game_over: show game over, else: play game)\n\
                 - DO NOT load external files (sounds, images, fonts) - code must be SELF-CONTAINED\n\
                 - Use pygame.font.Font(None, size) for default fonts only\n\
                 - Skip sound effects or print to console for feedback\n\
                 - Generate all graphics with pygame.draw and Surface.fill()\n\
                 - Proper restart: reset all variables, empty sprite groups, recreate sprites\n\
                 - ENSURE the game runs without NameError, AttributeError, or IndexError".to_string(),
    }];
    
    // Add conversation history
    full_messages.extend(messages);

    let body = ChatRequest {
        model: "Qwen/Qwen2.5-Coder-7B-Instruct".to_string(),
        messages: full_messages,
        max_tokens: Some(8192),  // Increased for complete games and complex code
        temperature: Some(0.2),
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {token}"))
            .context("Invalid Bearer token format")?,
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

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

    let parsed: ChatResponse = serde_json::from_str(&text_body)
        .context("Failed to parse Hugging Face JSON response")?;

    let generated = parsed
        .choices
        .first()
        .map(|choice| choice.message.content.clone())
        .ok_or_else(|| anyhow!("No choices in Hugging Face response"))?;

    Ok(generated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message {
            role: "user".to_string(),
            content: "test content".to_string(),
        };
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, "test content");
    }

    #[test]
    fn test_message_clone() {
        let msg = Message {
            role: "assistant".to_string(),
            content: "response".to_string(),
        };
        let cloned = msg.clone();
        assert_eq!(msg.role, cloned.role);
        assert_eq!(msg.content, cloned.content);
    }

    #[test]
    fn test_chat_request_serialization() {
        let request = ChatRequest {
            model: "test-model".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "You are helpful".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: "Hello".to_string(),
                },
            ],
            max_tokens: Some(100),
            temperature: Some(0.5),
        };

        let json = serde_json::to_string(&request);
        assert!(json.is_ok());
        
        let json_str = json.unwrap();
        assert!(json_str.contains("test-model"));
        assert!(json_str.contains("system"));
        assert!(json_str.contains("user"));
        assert!(json_str.contains("Hello"));
    }

    #[test]
    fn test_chat_response_deserialization() {
        let json = r#"{
            "choices": [
                {
                    "message": {
                        "role": "assistant",
                        "content": "print('Hello, World!')"
                    }
                }
            ]
        }"#;

        let response: Result<ChatResponse, _> = serde_json::from_str(json);
        assert!(response.is_ok());
        
        let response = response.unwrap();
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].message.role, "assistant");
        assert!(response.choices[0].message.content.contains("print"));
    }

    #[test]
    fn test_message_vector_operations() {
        let mut messages = vec![
            Message {
                role: "user".to_string(),
                content: "First".to_string(),
            },
            Message {
                role: "assistant".to_string(),
                content: "Second".to_string(),
            },
        ];

        assert_eq!(messages.len(), 2);
        
        messages.push(Message {
            role: "user".to_string(),
            content: "Third".to_string(),
        });

        assert_eq!(messages.len(), 3);
        assert_eq!(messages.last().unwrap().content, "Third");
    }

    #[test]
    fn test_optional_parameters() {
        let request = ChatRequest {
            model: "test".to_string(),
            messages: vec![],
            max_tokens: None,
            temperature: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        // Optional fields should not appear in JSON when None
        assert!(!json.contains("max_tokens"));
        assert!(!json.contains("temperature"));
    }
}
