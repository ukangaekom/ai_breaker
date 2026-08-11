use genai::chat::{ChatMessage as GenAiMessage, ChatRequest};
use genai::Client;
use std::sync::{Arc, Mutex};
use crate::database::ChatMessage;

// Dynamic client container allowing client re-instantiation when API keys change
static CLIENT: Mutex<Option<Arc<Client>>> = Mutex::new(None);

pub fn get_client() -> Arc<Client> {
    let mut lock = CLIENT.lock().unwrap();
    if let Some(ref client) = *lock {
        client.clone()
    } else {
        let client = Arc::new(Client::default());
        *lock = Some(client.clone());
        client
    }
}

pub fn reset_client() {
    let mut lock = CLIENT.lock().unwrap();
    *lock = Some(Arc::new(Client::default()));
}

pub fn resolve_api_key_env_var(model_name: &str) -> &'static str {
    let m = model_name.to_lowercase();
    if m.contains("gemini") || m.contains("google") {
        "GEMINI_API_KEY"
    } else if m.contains("gpt") || m.contains("openai") {
        "OPENAI_API_KEY"
    } else if m.contains("claude") || m.contains("anthropic") {
        "ANTHROPIC_API_KEY"
    } else if m.contains("groq") {
        "GROQ_API_KEY"
    } else if m.contains("deepseek") {
        "DEEPSEEK_API_KEY"
    } else if m.contains("cohere") {
        "COHERE_API_KEY"
    } else if m.contains("together") {
        "TOGETHER_API_KEY"
    } else if m.contains("fireworks") {
        "FIREWORKS_API_KEY"
    } else if m.contains("mistral") {
        "MISTRAL_API_KEY"
    } else if m.contains("xai") || m.contains("grok") {
        "XAI_API_KEY"
    } else if m.contains("ollama") {
        "OLLAMA_API_KEY"
    } else {
        "API_KEY"
    }
}

pub async fn exec_model_chat(
    model_name: &str,
    system_instruction: Option<&str>,
    history: &[ChatMessage],
    prompt: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = get_client();
    let mut messages = Vec::new();

    if let Some(sys) = system_instruction {
        if !sys.trim().is_empty() {
            messages.push(GenAiMessage::system(sys));
        }
    }

    for msg in history {
        match msg.role.to_lowercase().as_str() {
            "system" => messages.push(GenAiMessage::system(&msg.content)),
            "assistant" | "model" => messages.push(GenAiMessage::assistant(&msg.content)),
            _ => messages.push(GenAiMessage::user(&msg.content)),
        }
    }

    messages.push(GenAiMessage::user(prompt));

    let chat_req = ChatRequest::new(messages);
    let chat_res = client.exec_chat(model_name, chat_req, None).await?;

    let text = chat_res.into_first_text().unwrap_or_else(|| "No text response".into());
    Ok(text)
}
