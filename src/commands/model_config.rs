use std::fs;
use std::path::PathBuf;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// filepath to model settings
const CONFIG_FILE: &str = ".model_config.json";
const CONFIG_VERSION: &str = "1.0";

/// errors for model setup
#[derive(Debug)]
pub enum ModelConfigError {
    FileNotFound,
    InvalidJson(String),
    InvalidStructure(String),
    ModelNotFound(String),
    DuplicateModelId(String),
    ApiKeyMissing,
    InvalidUrl(String),
}

impl std::fmt::Display for ModelConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ModelConfigError::FileNotFound => write!(f, "Model configuration file not found"),
            ModelConfigError::InvalidJson(msg) => write!(f, "Invalid JSON: {}", msg),
            ModelConfigError::InvalidStructure(msg) => write!(f, "Invalid configuration structure: {}", msg),
            ModelConfigError::ModelNotFound(id) => write!(f, "Model '{}' not found", id),
            ModelConfigError::DuplicateModelId(id) => write!(f, "Model ID '{}' already exists", id),
            ModelConfigError::ApiKeyMissing => write!(f, "API key not configured"),
            ModelConfigError::InvalidUrl(url) => write!(f, "Invalid URL: {}", url),
        }
    }
}

impl std::error::Error for ModelConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self)
    }
}

/// source of API Key
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiKeySource {
    Env,
    Direct,
}

/// model items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub id: String,
    pub name: String,
    pub url: String,
    pub api_key_source: ApiKeySource,
    pub api_key_env: Option<String>,
    pub api_key_value: Option<String>,
    pub model_name: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub enabled: bool,
}

/// model settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub version: String,
    pub default_model: String,
    pub models: Vec<ModelEntry>,
}

/// model management
pub struct ModelManager {
    config_path: PathBuf,
    pub config: ModelConfig,
}

impl ModelManager {
    /// creation or load model settings
    pub fn new() -> Result<Self> {
        let config_path = PathBuf::from(CONFIG_FILE);

        if config_path.exists() {
            Self::from_path(config_path)
        } else {
            // creating default settings
            Self::create_default()
        }
    }

    /// load settings from designated path
    pub fn from_path(path: PathBuf) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .map_err(|e| ModelConfigError::InvalidJson(e.to_string()))?;

        let config: ModelConfig = serde_json::from_str(&content)
            .map_err(|e| ModelConfigError::InvalidJson(e.to_string()))?;

        // verfication of setting configure
        if config.version != CONFIG_VERSION {
            return Err(anyhow!("Unsupported config version: {}", config.version));
        }

        if config.models.is_empty() {
            return Err(anyhow!("No models configured"));
        }

        // verifying the presence of default model
        let default_exists = config.models.iter().any(|m| m.id == config.default_model);
        if !default_exists {
            return Err(anyhow!("Default model '{}' not found in models list", config.default_model));
        }

        Ok(ModelManager { config_path: path, config })
    }

    /// save the settings
    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.config)?;
        fs::write(&self.config_path, json)?;
        Ok(())
    }

    /// add a model
    pub fn add_model(&mut self, entry: ModelEntry) -> Result<()> {
        // check whether the ID has been used?
        if self.config.models.iter().any(|m| m.id == entry.id) {
            return Err(ModelConfigError::DuplicateModelId(entry.id).into());
        }

        // verfiying the URL
        if !entry.url.starts_with("http://") && !entry.url.starts_with("https://") {
            return Err(ModelConfigError::InvalidUrl(entry.url).into());
        }

        self.config.models.push(entry);
        Ok(())
    }

    /// delete a model
    pub fn delete_model(&mut self, id: &str) -> Result<()> {
        let initial_len = self.config.models.len();

        self.config.models.retain(|m| m.id != id);

        if self.config.models.len() == initial_len {
            return Err(ModelConfigError::ModelNotFound(id.to_string()).into());
        }

        // If deleting a default model, it turns to the first available model
        if self.config.default_model == id {
            if let Some(first_model) = self.config.models.first() {
                self.config.default_model = first_model.id.clone();
            }
        }

        Ok(())
    }

    /// fetch all of models
    pub fn list_models(&self) -> Vec<&ModelEntry> {
        self.config.models.iter().collect()
    }

    /// fetch the current(default) model
    pub fn get_current_model(&self) -> Option<&ModelEntry> {
        self.config.models.iter().find(|m| m.id == self.config.default_model)
    }

    /// switch to a designated model
    pub fn switch_model(&mut self, id: &str) -> Result<()> {
        // check if it exists
        if !self.config.models.iter().any(|m| m.id == id) {
            return Err(ModelConfigError::ModelNotFound(id.to_string()).into());
        }

        self.config.default_model = id.to_string();
        Ok(())
    }

    /// fetch the URL of current model 
    pub fn get_model_url(&self) -> String {
        if let Some(model) = self.get_current_model() {
            model.url.clone()
        } else {
            // return to the default USTC URL
            "http://scc.ustc.edu.cn/portal/api/ask".to_string()
        }
    }

    /// fetch the current model's API Key
    pub fn get_api_key(&self) -> Result<String> {
        if let Some(model) = self.get_current_model() {
            match model.api_key_source {
                ApiKeySource::Env => {
                    if let Some(env_var) = &model.api_key_env {
                        std::env::var(env_var).map_err(|_| ModelConfigError::ApiKeyMissing.into())
                    } else {
                        Err(ModelConfigError::ApiKeyMissing.into())
                    }
                }
                ApiKeySource::Direct => {
                    if let Some(key) = &model.api_key_value {
                        Ok(key.clone())
                    } else {
                        Err(ModelConfigError::ApiKeyMissing.into())
                    }
                }
            }
        } else {
            Err(ModelConfigError::ApiKeyMissing.into())
        }
    }

    /// creating the default settings ( for the four preset models) 
    fn create_default() -> Result<Self> {
        let config_path = PathBuf::from(CONFIG_FILE);

        let now = Utc::now();

        let models = vec![
            ModelEntry {
                id: "deepseek-ustc".to_string(),
                name: "DeepSeek USTC".to_string(),
                url: "http://scc.ustc.edu.cn/portal/api/ask".to_string(),
                api_key_source: ApiKeySource::Env,
                api_key_env: Some("DEEPSEEK_API_USTC".to_string()),
                api_key_value: None,
                model_name: Some("deepseek-chat".to_string()),
                description: Some("Default DeepSeek ( at USTC )".to_string()),
                created_at: now,
                enabled: true,
            },
            ModelEntry {
                id: "deepseek-official".to_string(),
                name: "DeepSeek Official".to_string(),
                url: "https://api.deepseek.com".to_string(),
                api_key_source: ApiKeySource::Env,
                api_key_env: Some("DEEPSEEK_API_KEY".to_string()),
                api_key_value: None,
                model_name: Some("deepseek-chat".to_string()),
                description: Some("DeepSeek Official API".to_string()),
                created_at: now,
                enabled: true,
            },
            ModelEntry {
                id: "kimi-k2.5".to_string(),
                name: "Kimi K2.5".to_string(),
                url: "https://api.moonshot.cn/v1".to_string(),
                api_key_source: ApiKeySource::Env,
                api_key_env: Some("KIMI-k2.5".to_string()),
                api_key_value: None,
                model_name: Some("moonshot-v1-8k".to_string()),
                description: Some("Moonshot Kimi model".to_string()),
                created_at: now,
                enabled: true,
            },
            ModelEntry {
                id: "glm5".to_string(),
                name: "GLM-5".to_string(),
                url: "https://open.bigmodel.cn/api/paas/v4/chat/completions".to_string(),
                api_key_source: ApiKeySource::Env,
                api_key_env: Some("GLM5".to_string()),
                api_key_value: None,
                model_name: Some("glm-4".to_string()),
                description: Some("Zhipu GLM-5 model".to_string()),
                created_at: now,
                enabled: true,
            },
        ];

        let config = ModelConfig {
            version: CONFIG_VERSION.to_string(),
            default_model: "deepseek-ustc".to_string(),
            models,
        };

        // save default settings
        let manager = ModelManager { config_path, config };
        manager.save()?;

        Ok(manager)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_default_config() {
        let manager = ModelManager::create_default().unwrap();
        assert_eq!(manager.config.models.len(), 4);
        assert_eq!(manager.config.default_model, "deepseek-ustc");
        assert_eq!(manager.config.version, "1.0");
    }

    #[test]
    fn test_get_current_model() {
        let manager = ModelManager::create_default().unwrap();
        let current = manager.get_current_model();
        assert!(current.is_some());
        assert_eq!(current.unwrap().id, "deepseek-ustc");
    }

    #[test]
    fn test_list_models() {
        let manager = ModelManager::create_default().unwrap();
        let models = manager.list_models();
        assert_eq!(models.len(), 4);
    }

    #[test]
    fn test_switch_model() {
        let mut manager = ModelManager::create_default().unwrap();
        assert!(manager.switch_model("kimi-k2.5").is_ok());
        assert_eq!(manager.config.default_model, "kimi-k2.5");
    }

    #[test]
    fn test_delete_model() {
        let mut manager = ModelManager::create_default().unwrap();
        assert_eq!(manager.config.models.len(), 4);
        assert!(manager.delete_model("deepseek-ustc").is_ok());
        assert_eq!(manager.config.models.len(), 3);
    }

    #[test]
    fn test_duplicate_id() {
        let mut manager = ModelManager::create_default().unwrap();
        let entry = ModelEntry {
            id: "deepseek-ustc".to_string(),
            name: "Test".to_string(),
            url: "https://api.test.com".to_string(),
            api_key_source: ApiKeySource::Env,
            api_key_env: Some("TEST_KEY".to_string()),
            api_key_value: None,
            model_name: None,
            description: None,
            created_at: chrono::Utc::now(),
            enabled: true,
        };
        assert!(manager.add_model(entry).is_err());
    }
}
