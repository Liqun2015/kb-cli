use std::fs;
use std::path::PathBuf;
use anyhow::Result;
use serde_json;

use super::model_config::ModelManager;

/// Default model configuration (backward compatibility)
const DEFAULT_MODEL_URL: &str = "http://scc.ustc.edu.cn/portal/api/ask";
const DEFAULT_API_KEY_ENV: &str = "DEEPSEEK_API_USTC";
const DEFAULT_OUTPUT_FILE: &str = ".model_switch_output.json";
const DEFAULT_INPUT_FILE: &str = ".model_switch_input.json";

/// Model-Switch bridge module
/// Handles interaction with model-switch tool, checking status and reading/writing LLM responses

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

/// Check if model-switch output file exists and is readable
pub fn is_available() -> bool {
    PathBuf::from(DEFAULT_OUTPUT_FILE).exists()
}

/// Write input to model-switch (triggers LLM request)
pub fn write_input(question: &str) -> Result<()> {
    let manager = ModelManager::new()?;

    // Get current model info and add to context
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

/// Read output from model-switch (get LLM response)
pub fn read_output() -> Result<Option<String>> {
    let output_path = PathBuf::from(DEFAULT_OUTPUT_FILE);

    // Check if output file exists
    if !output_path.exists() {
        return Ok(None);
    }

    // Read output file
    match fs::read_to_string(&output_path) {
        Ok(content) => {
            match serde_json::from_str::<ModelSwitchOutput>(&content) {
                Ok(output) => {
                    // Check for errors
                    if let Some(error) = output.error {
                        return Ok(Some(format!("[Error] {}", error)));
                    }
                    // Return response
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

/// Get LLM response (complete workflow)
/// 1. Write input file
/// 2. Wait for model-switch processing (handled externally)
/// 3. Read output file
pub fn get_llm_response(question: &str, auto_write: bool) -> Result<String> {
    if auto_write {
        write_input(question)?;
    }

    match read_output()? {
        Some(response) => Ok(response),
        None => {
            // model-switch not running or no output
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

/// Get current model URL (integrated with ModelManager)
pub fn get_current_model_url() -> Result<String> {
    match ModelManager::new() {
        Ok(manager) => Ok(manager.get_model_url()),
        Err(_) => {
            // Fallback to hardcoded default
            Ok(DEFAULT_MODEL_URL.to_string())
        }
    }
}

/// Get current model API Key (integrated with ModelManager)
pub fn get_current_api_key() -> Result<String> {
    match ModelManager::new() {
        Ok(manager) => manager.get_api_key(),
        Err(_) => {
            // Fallback to environment variable
            std::env::var(DEFAULT_API_KEY_ENV)
                .map_err(|_| anyhow::anyhow!("API key not found in config or environment"))
        }
    }
}

/// Check API Key configuration (backward compatibility)
pub fn check_api_key() -> bool {
    // Check config file first
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

    // Fallback to environment variable
    std::env::var(DEFAULT_API_KEY_ENV).is_ok()
}

/// Get default model URL (backward compatibility)
pub fn get_default_model_url() -> &'static str {
    DEFAULT_MODEL_URL
}

/// Get API Key environment variable name (backward compatibility)
pub fn get_api_key_env() -> &'static str {
    DEFAULT_API_KEY_ENV
}

/// Get input file path
pub fn get_input_file() -> &'static str {
    DEFAULT_INPUT_FILE
}

/// Get output file path
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
