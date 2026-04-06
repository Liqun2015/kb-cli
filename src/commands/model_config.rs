use std::fs;
use std::path::PathBuf;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 配置文件路径
const CONFIG_FILE: &str = ".model_config.json";
const CONFIG_VERSION: &str = "1.0";

/// 模型配置错误类型
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

/// API Key 来源
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiKeySource {
    Env,
    Direct,
}

/// 模型条目
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

/// 模型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub version: String,
    pub default_model: String,
    pub models: Vec<ModelEntry>,
}

/// 模型管理器
pub struct ModelManager {
    config_path: PathBuf,
    pub config: ModelConfig,
}

impl ModelManager {
    /// 创建或加载模型配置
    pub fn new() -> Result<Self> {
        let config_path = PathBuf::from(CONFIG_FILE);

        if config_path.exists() {
            Self::from_path(config_path)
        } else {
            // 创建默认配置
            Self::create_default()
        }
    }

    /// 从指定路径加载配置
    pub fn from_path(path: PathBuf) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .map_err(|e| ModelConfigError::InvalidJson(e.to_string()))?;

        let config: ModelConfig = serde_json::from_str(&content)
            .map_err(|e| ModelConfigError::InvalidJson(e.to_string()))?;

        // 验证配置结构
        if config.version != CONFIG_VERSION {
            return Err(anyhow!("Unsupported config version: {}", config.version));
        }

        if config.models.is_empty() {
            return Err(anyhow!("No models configured"));
        }

        // 验证默认模型存在
        let default_exists = config.models.iter().any(|m| m.id == config.default_model);
        if !default_exists {
            return Err(anyhow!("Default model '{}' not found in models list", config.default_model));
        }

        Ok(ModelManager { config_path: path, config })
    }

    /// 保存配置到磁盘
    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.config)?;
        fs::write(&self.config_path, json)?;
        Ok(())
    }

    /// 添加新模型
    pub fn add_model(&mut self, entry: ModelEntry) -> Result<()> {
        // 检查 ID 是否重复
        if self.config.models.iter().any(|m| m.id == entry.id) {
            return Err(ModelConfigError::DuplicateModelId(entry.id).into());
        }

        // 验证 URL
        if !entry.url.starts_with("http://") && !entry.url.starts_with("https://") {
            return Err(ModelConfigError::InvalidUrl(entry.url).into());
        }

        self.config.models.push(entry);
        Ok(())
    }

    /// 删除模型
    pub fn delete_model(&mut self, id: &str) -> Result<()> {
        let initial_len = self.config.models.len();

        self.config.models.retain(|m| m.id != id);

        if self.config.models.len() == initial_len {
            return Err(ModelConfigError::ModelNotFound(id.to_string()).into());
        }

        // 如果删除的是默认模型，切换到第一个可用模型
        if self.config.default_model == id {
            if let Some(first_model) = self.config.models.first() {
                self.config.default_model = first_model.id.clone();
            }
        }

        Ok(())
    }

    /// 获取所有模型
    pub fn list_models(&self) -> Vec<&ModelEntry> {
        self.config.models.iter().collect()
    }

    /// 获取当前（默认）模型
    pub fn get_current_model(&self) -> Option<&ModelEntry> {
        self.config.models.iter().find(|m| m.id == self.config.default_model)
    }

    /// 切换到指定模型
    pub fn switch_model(&mut self, id: &str) -> Result<()> {
        // 检查模型是否存在
        if !self.config.models.iter().any(|m| m.id == id) {
            return Err(ModelConfigError::ModelNotFound(id.to_string()).into());
        }

        self.config.default_model = id.to_string();
        Ok(())
    }

    /// 获取当前模型的 URL
    pub fn get_model_url(&self) -> String {
        if let Some(model) = self.get_current_model() {
            model.url.clone()
        } else {
            // 回退到默认 USTC URL
            "http://scc.ustc.edu.cn/portal/api/ask".to_string()
        }
    }

    /// 获取当前模型的 API Key
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

    /// 创建默认配置（包含 4 个预配置模型）
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
                description: Some("默认 DeepSeek 模型（USTC 代理）".to_string()),
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
                description: Some("DeepSeek 官方 API".to_string()),
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
                description: Some("月之暗面 Kimi 模型".to_string()),
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
                description: Some("智谱 GLM-5 模型".to_string()),
                created_at: now,
                enabled: true,
            },
        ];

        let config = ModelConfig {
            version: CONFIG_VERSION.to_string(),
            default_model: "deepseek-ustc".to_string(),
            models,
        };

        // 保存默认配置
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
