use std::fs;
use std::path::PathBuf;
use anyhow::Result;
use serde_json;

use super::model_config::ModelManager;

/// 默认模型配置（向后兼容）
const DEFAULT_MODEL_URL: &str = "http://scc.ustc.edu.cn/portal/api/ask";
const DEFAULT_API_KEY_ENV: &str = "DEEPSEEK_API_USTC";
const DEFAULT_OUTPUT_FILE: &str = ".model_switch_output.json";
const DEFAULT_INPUT_FILE: &str = ".model_switch_input.json";

/// Model-Switch 桥接模块
/// 负责与 model-switch 工具交互，检查状态并读取/写入 LLM 响应

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ModelSwitchOutput {
    response: Option<String>,
    error: Option<String>,
    model: Option<String>,
}

#[derive(serde::Serialize)]
struct ModelSwitchInput {
    prompt: String,
    context: Option<String>,
    timestamp: String,
}

/// 检查 model-switch 输出文件是否存在且可读
pub fn is_available() -> bool {
    PathBuf::from(DEFAULT_OUTPUT_FILE).exists()
}

/// 向 model-switch 写入输入（触发 LLM 请求）
pub fn write_input(question: &str) -> Result<()> {
    let manager = ModelManager::new()?;

    // 获取当前模型信息并添加到上下文
    let context_opt = if let Some(model) = manager.get_current_model() {
        Some(serde_json::json!({
            "model_id": model.id,
            "model_name": model.name,
            "model_url": model.url
        }).to_string())
    } else {
        None
    };

    let input = ModelSwitchInput {
        prompt: question.to_string(),
        context: context_opt,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let json = serde_json::to_string_pretty(&input)?;
    fs::write(DEFAULT_INPUT_FILE, json)?;
    Ok(())
}

/// 从 model-switch 读取输出（获取 LLM 响应）
pub fn read_output() -> Result<Option<String>> {
    let output_path = PathBuf::from(DEFAULT_OUTPUT_FILE);

    // 检查输出文件是否存在
    if !output_path.exists() {
        return Ok(None);
    }

    // 读取输出文件
    match fs::read_to_string(&output_path) {
        Ok(content) => {
            match serde_json::from_str::<ModelSwitchOutput>(&content) {
                Ok(output) => {
                    // 检查是否有错误
                    if let Some(error) = output.error {
                        return Ok(Some(format!("[Error] {}", error)));
                    }
                    // 返回响应
                    Ok(output.response)
                }
                Err(e) => {
                    eprintln!("Error parsing model-switch output: {}", e);
                    Ok(None)
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading model-switch output file: {}", e);
            Ok(None)
        }
    }
}

/// 获取 LLM 响应（完整流程）
/// 1. 写入输入文件
/// 2. 等待 model-switch 处理（由外部处理）
/// 3. 读取输出文件
pub fn get_llm_response(question: &str, auto_write: bool) -> Result<String> {
    if auto_write {
        write_input(question)?;
    }

    match read_output()? {
        Some(response) => Ok(response),
        None => {
            // model-switch 未运行或无输出
            let manager = ModelManager::new();

            let mut message = "Model-switch is not running or no response available.\n\
               To use LLM features:\n\
               1. Start the model-switch tool\n\
               2. Ensure it's watching input/output files".to_string();

            if let Ok(manager) = &manager {
                if let Some(model) = manager.get_current_model() {
                    message.push_str("\n\n");
                    message.push_str(&format!("Current model: {} ({})", model.id, model.name));
                    message.push_str(&format!("  URL: {}", model.url));

                    match &model.api_key_source {
                        super::model_config::ApiKeySource::Env => {
                            if let Some(env) = &model.api_key_env {
                                message.push_str(&format!("  API Key: ${}", env));
                                match std::env::var(env) {
                                    Ok(_) => message.push_str("\n  API Key is set"),
                                    Err(_) => message.push_str("\n  API Key is NOT set"),
                                }
                            }
                        }
                        super::model_config::ApiKeySource::Direct => {
                            message.push_str("\n  API Key: stored in config");
                        }
                    }
                }
            }

            Ok(message)
        }
    }
}

/// 获取当前模型 URL（集成 ModelManager）
pub fn get_current_model_url() -> Result<String> {
    match ModelManager::new() {
        Ok(manager) => Ok(manager.get_model_url()),
        Err(_) => {
            // 回退到硬编码默认值
            Ok(DEFAULT_MODEL_URL.to_string())
        }
    }
}

/// 获取当前模型 API Key（集成 ModelManager）
pub fn get_current_api_key() -> Result<String> {
    match ModelManager::new() {
        Ok(manager) => manager.get_api_key(),
        Err(_) => {
            // 回退到环境变量
            std::env::var(DEFAULT_API_KEY_ENV)
                .map_err(|_| anyhow::anyhow!("API key not found in config or environment"))
        }
    }
}

/// 检查 API Key 配置（保留向后兼容）
pub fn check_api_key() -> bool {
    // 优先检查配置文件
    if let Ok(manager) = ModelManager::new() {
        if let Some(model) = manager.get_current_model() {
            match &model.api_key_source {
                super::model_config::ApiKeySource::Env => {
                    if let Some(env) = &model.api_key_env {
                        return std::env::var(env).is_ok();
                    }
                }
                super::model_config::ApiKeySource::Direct => {
                    return model.api_key_value.is_some();
                }
            }
        }
    }

    // 回退到环境变量
    std::env::var(DEFAULT_API_KEY_ENV).is_ok()
}

/// 获取默认模型 URL（保留向后兼容）
pub fn get_default_model_url() -> &'static str {
    DEFAULT_MODEL_URL
}

/// 获取 API Key 环境变量名（保留向后兼容）
pub fn get_api_key_env() -> &'static str {
    DEFAULT_API_KEY_ENV
}

/// 获取输入文件路径
pub fn get_input_file() -> &'static str {
    DEFAULT_INPUT_FILE
}

/// 获取输出文件路径
pub fn get_output_file() -> &'static str {
    DEFAULT_OUTPUT_FILE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        assert_eq!(get_default_model_url(), "http://scc.ustc.edu.cn/portal/api/ask");
        assert_eq!(get_api_key_env(), "DEEPSEEK_API_USTC");
        assert_eq!(get_input_file(), ".model_switch_input.json");
        assert_eq!(get_output_file(), ".model_switch_output.json");
    }
}
