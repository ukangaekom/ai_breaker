use genai::chat::{ChatMessage as GenAiMessage, ChatRequest};
use genai::Client;
use std::sync::{Arc, OnceLock};
use crate::database::ChatMessage;

static CLIENT: OnceLock<Arc<Client>> = OnceLock::new();

#[inline(always)]
pub fn get_client() -> Arc<Client> {
    CLIENT.get_or_init(|| Arc::new(Client::default())).clone()
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
