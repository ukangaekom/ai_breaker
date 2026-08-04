use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use ulid::Ulid;

const MODELS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("models");
const CHAT_LOGS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("chat_logs");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelInfo {
    pub name: String,
    pub provider: String,
    pub description: Option<String>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatLog {
    pub id: String,
    pub model_name: String,
    pub system_prompt: Option<String>,
    pub prompt: String,
    pub response: String,
    pub messages: Vec<ChatMessage>,
    pub timestamp: u64,
    pub latency_ms: Option<u64>,
    pub tokens_used: Option<u32>,
    pub metadata: Option<serde_json::Value>,
}

impl ChatLog {
    pub fn new(
        model_name: impl Into<String>,
        system_prompt: Option<String>,
        prompt: impl Into<String>,
        response: impl Into<String>,
        messages: Vec<ChatMessage>,
        latency_ms: Option<u64>,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            id: Ulid::new().to_string(),
            model_name: model_name.into(),
            system_prompt,
            prompt: prompt.into(),
            response: response.into(),
            messages,
            timestamp: now,
            latency_ms,
            tokens_used: None,
            metadata: None,
        }
    }
}

#[derive(Clone)]
pub struct DatabaseManager {
    db: Arc<Database>,
    path: PathBuf,
}

#[allow(dead_code)]
impl DatabaseManager {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let db_path = path.as_ref().to_path_buf();
        let db = Database::create(&db_path)?;

        let write_txn = db.begin_write()?;
        {
            let _ = write_txn.open_table(MODELS_TABLE)?;
            let _ = write_txn.open_table(CHAT_LOGS_TABLE)?;
        }
        write_txn.commit()?;

        Ok(Self {
            db: Arc::new(db),
            path: db_path,
        })
    }

    pub fn open_default() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Self::open("ai_breaker.redb")
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    // --- Model Operations ---
    pub fn save_model(&self, model: &ModelInfo) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let json = serde_json::to_string(model)?;
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(MODELS_TABLE)?;
            table.insert(model.name.as_str(), json.as_str())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn get_model(&self, name: &str) -> Result<Option<ModelInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(MODELS_TABLE)?;
        if let Some(guard) = table.get(name)? {
            let val = guard.value();
            let info: ModelInfo = serde_json::from_str(val)?;
            Ok(Some(info))
        } else {
            Ok(None)
        }
    }

    pub fn list_models(&self) -> Result<Vec<ModelInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(MODELS_TABLE)?;
        let mut list = Vec::new();
        for item in table.iter()? {
            let item = item?;
            let info: ModelInfo = serde_json::from_str(item.1.value())?;
            list.push(info);
        }
        Ok(list)
    }

    pub fn delete_model(&self, name: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let write_txn = self.db.begin_write()?;
        let removed = {
            let mut table = write_txn.open_table(MODELS_TABLE)?;
            let res = table.remove(name)?;
            res.is_some()
        };
        write_txn.commit()?;
        Ok(removed)
    }

    // --- Chat Log Operations ---
    pub fn save_chat_log(&self, log: &ChatLog) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let json = serde_json::to_string(log)?;
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(CHAT_LOGS_TABLE)?;
            table.insert(log.id.as_str(), json.as_str())?;
        }
        write_txn.commit()?;

        // Also register model automatically if not already registered
        if self.get_model(&log.model_name)?.is_none() {
            let provider = log
                .model_name
                .split('/')
                .next()
                .unwrap_or(&log.model_name)
                .to_string();
            let info = ModelInfo {
                name: log.model_name.clone(),
                provider,
                description: Some("Auto-registered from chat log session".into()),
                created_at: log.timestamp,
            };
            let _ = self.save_model(&info);
        }

        Ok(())
    }

    pub fn get_chat_log(&self, id: &str) -> Result<Option<ChatLog>, Box<dyn std::error::Error + Send + Sync>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(CHAT_LOGS_TABLE)?;
        if let Some(guard) = table.get(id)? {
            let log: ChatLog = serde_json::from_str(guard.value())?;
            Ok(Some(log))
        } else {
            Ok(None)
        }
    }

    pub fn list_chat_logs(&self) -> Result<Vec<ChatLog>, Box<dyn std::error::Error + Send + Sync>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(CHAT_LOGS_TABLE)?;
        let mut list = Vec::new();
        for item in table.iter()? {
            let item = item?;
            let log: ChatLog = serde_json::from_str(item.1.value())?;
            list.push(log);
        }
        Ok(list)
    }

    pub fn get_chat_logs_by_model(&self, model_name: &str) -> Result<Vec<ChatLog>, Box<dyn std::error::Error + Send + Sync>> {
        let logs = self.list_chat_logs()?;
        Ok(logs.into_iter().filter(|l| l.model_name == model_name).collect())
    }

    pub fn delete_chat_log(&self, id: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let write_txn = self.db.begin_write()?;
        let removed = {
            let mut table = write_txn.open_table(CHAT_LOGS_TABLE)?;
            let res = table.remove(id)?;
            res.is_some()
        };
        write_txn.commit()?;
        Ok(removed)
    }

    pub fn clear_chat_logs(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let logs = self.list_chat_logs()?;
        let count = logs.len();
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(CHAT_LOGS_TABLE)?;
            for log in &logs {
                table.remove(log.id.as_str())?;
            }
        }
        write_txn.commit()?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_model_and_chat_operations() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("test_ai_breaker_{}.redb", Ulid::new()));

        let db_manager = DatabaseManager::open(&db_path).unwrap();

        // 1. Model Operations
        let model = ModelInfo {
            name: "gemini-1.5-flash".into(),
            provider: "gemini".into(),
            description: Some("Google Gemini Flash".into()),
            created_at: 1000,
        };

        db_manager.save_model(&model).unwrap();

        let fetched_model = db_manager.get_model("gemini-1.5-flash").unwrap();
        assert_eq!(fetched_model, Some(model.clone()));

        let models = db_manager.list_models().unwrap();
        assert_eq!(models.len(), 1);

        // 2. Chat Log Operations
        let chat_log = ChatLog::new(
            "gemini-1.5-flash",
            Some("You are a helpful assistant.".into()),
            "Hello, test prompt",
            "Hello, test response",
            vec![
                ChatMessage {
                    role: "user".into(),
                    content: "Hello, test prompt".into(),
                    timestamp: 1001,
                },
                ChatMessage {
                    role: "assistant".into(),
                    content: "Hello, test response".into(),
                    timestamp: 1002,
                },
            ],
            Some(150),
        );

        db_manager.save_chat_log(&chat_log).unwrap();

        let fetched_log = db_manager.get_chat_log(&chat_log.id).unwrap();
        assert!(fetched_log.is_some());
        let fetched_log = fetched_log.unwrap();
        assert_eq!(fetched_log.model_name, "gemini-1.5-flash");
        assert_eq!(fetched_log.prompt, "Hello, test prompt");
        assert_eq!(fetched_log.response, "Hello, test response");

        let logs_for_model = db_manager.get_chat_logs_by_model("gemini-1.5-flash").unwrap();
        assert_eq!(logs_for_model.len(), 1);

        // Cleanup test DB file
        let _ = std::fs::remove_file(&db_path);
    }
}