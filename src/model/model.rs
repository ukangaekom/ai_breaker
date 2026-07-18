use genai::chat::{ChatMessage, ChatRequest};
use genai::Client;
use std::sync::{Arc, OnceLock};

// static PROCESS_SYSTEM_CONFIGURATION: OnceCell<String> = OnceCell::const_new();
static CLIENT: OnceLock<Arc<Client>> = OnceLock::new();

#[inline(always)]
pub fn get_client() -> Arc<Client> {
    CLIENT.get_or_init(|| Arc::new(Client::default())).clone()
}

async fn model_chat(
    model_name: &str,
    system_instruction: &str,
    prompt: &str,
) -> Option<std::string::String> {
    let client = get_client();
    let chat_req: ChatRequest = ChatRequest::new(vec![
        ChatMessage::system(system_instruction),
        ChatMessage::user(prompt),
    ]);

    let model: &str = model_name;

    let chat_res = client.exec_chat(model, chat_req, None).await;

    println!("{:?}", &chat_res);

    chat_res.expect("REASON").into_first_text()
}
