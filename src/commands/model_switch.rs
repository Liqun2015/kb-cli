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

/// Model-Switch bridge module.
///
/// The bridge is file based: `kb repl` writes a request file and an external
/// model-switch process may write a response file. Each request includes a
/// request_id so the CLI does not accidentally display a stale response.

#[derive(serde::Deserialize, Debug)]
struct ModelSwitchOutput {
    #[serde(alias = "requestId")]
    request_id: Option<String>,
    response: Option<String>,
    error: Option<String>,
    model: Option<String>,
}

#[derive(serde::Serialize)]
struct ModelSwitchInput {
    request_id: String,
    prompt: String,
    context: Option<String>,
    timestamp: String,
}

/// Check if model-switch output file exists and is readable.
#[allow(dead_code)]
pub fn is_available() -> bool {
    PathBuf::from(DEFAULT_OUTPUT_FILE).exists()
}

/// Write input to model-switch and return the request id.
pub fn write_input(question: &str) -> Result<String> {
    let manager = ModelManager::new()?;

    // Get current model info and add to context.
    let context_opt = if let Some(model) = manager.get_current_model() {
        Some(serde_json::json!({
            "model_id": model.id,
            "model_name": model.name,
            "model_url": model.url
        }).to_string())
    } else {
        None
    };

    let request_id = format!(
        "kb-{}-{}",
        chrono::Utc::now().timestamp_millis(),
        std::process::id()
    );

    let input = ModelSwitchInput {
        request_id: request_id.clone(),
        prompt: question.to_string(),
        context: context_opt,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    // Remove stale output before writing a new request. This avoids reading a
    // previous answer when the external model-switch has not processed the new
    // input yet.
    let output_path = PathBuf::from(DEFAULT_OUTPUT_FILE);
    if output_path.exists() {
        let _ = fs::remove_file(&output_path);
    }

    let json = serde_json::to_string_pretty(&input)?;
    fs::write(DEFAULT_INPUT_FILE, json)?;
    Ok(request_id)
}

/// Read output from model-switch. When `expected_request_id` is set, output from
/// any other request is treated as stale and ignored.
pub fn read_output(expected_request_id: Option<&str>) -> Result<Option<String>> {
    let output_path = PathBuf::from(DEFAULT_OUTPUT_FILE);

    if !output_path.exists() {
        return Ok(None);
    }

    match fs::read_to_string(&output_path) {
        Ok(content) => match serde_json::from_str::<ModelSwitchOutput>(&content) {
            Ok(output) => {
                if let Some(expected) = expected_request_id {
                    match output.request_id.as_deref() {
                        Some(actual) if actual == expected => {}
                        Some(actual) => {
                            return Ok(Some(format!(
                                "Model-switch output belongs to another request ({actual}); expected {expected}."
                            )));
                        }
                        None => {
                            return Ok(Some(format!(
                                "Model-switch output has no request_id; expected {expected}. Ignoring it to avoid stale responses."
                            )));
                        }
                    }
                }

                if let Some(error) = output.error {
                    return Ok(Some(format!("[Error] {}", error)));
                }

                let response = output.response.map(|text| {
                    if let Some(model) = output.model {
                        format!("{}\n\n[model: {}]", text, model)
                    } else {
                        text
                    }
                });

                Ok(response)
            }
            Err(e) => {
                eprintln!("Error parsing model-switch output: {}", e);
                Ok(None)
            }
        },
        Err(e) => {
            eprintln!("Error reading model-switch output file: {}", e);
            Ok(None)
        }
    }
}

/// Get LLM response (complete workflow).
/// 1. Write input file.
/// 2. External model-switch handles processing.
/// 3. Read a matching output file if it already exists.
pub fn get_llm_response(question: &str, auto_write: bool) -> Result<String> {
    let request_id = if auto_write {
        Some(write_input(question)?)
    } else {
        None
    };

    match read_output(request_id.as_deref())? {
        Some(response) => Ok(response),
        None => {
            let manager = ModelManager::new();

            let mut message = if let Some(id) = request_id {
                format!(
                    "Request written to {input}.\nRequest ID: {id}\n\nNo matching response is available yet. Start or check the external model-switch tool and make sure it writes to {output} with the same request_id.",
                    input = DEFAULT_INPUT_FILE,
                    output = DEFAULT_OUTPUT_FILE,
                    id = id,
                )
            } else {
                "Model-switch is not running or no response is available.".to_string()
            };

            if let Ok(manager) = &manager {
                if let Some(model) = manager.get_current_model() {
                    message.push_str("\n\n");
                    message.push_str(&format!("Current model: {} ({})", model.id, model.name));
                    message.push_str(&format!("\n  URL: {}", model.url));

                    match &model.api_key_source {
                        super::model_config::ApiKeySource::Env => {
                            if let Some(env) = &model.api_key_env {
                                message.push_str(&format!("\n  API Key: ${}", env));
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

/// Get current model URL (integrated with ModelManager).
#[allow(dead_code)]
pub fn get_current_model_url() -> Result<String> {
    match ModelManager::new() {
        Ok(manager) => Ok(manager.get_model_url()),
        Err(_) => Ok(DEFAULT_MODEL_URL.to_string()),
    }
}

/// Get current model API Key (integrated with ModelManager).
#[allow(dead_code)]
pub fn get_current_api_key() -> Result<String> {
    match ModelManager::new() {
        Ok(manager) => manager.get_api_key(),
        Err(_) => std::env::var(DEFAULT_API_KEY_ENV)
            .map_err(|_| anyhow::anyhow!("API key not found in config or environment")),
    }
}

/// Check API Key configuration (backward compatibility).
#[allow(dead_code)]
pub fn check_api_key() -> bool {
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

    std::env::var(DEFAULT_API_KEY_ENV).is_ok()
}

/// Get default model URL (backward compatibility).
#[allow(dead_code)]
pub fn get_default_model_url() -> &'static str {
    DEFAULT_MODEL_URL
}

/// Get API Key environment variable name (backward compatibility).
#[allow(dead_code)]
pub fn get_api_key_env() -> &'static str {
    DEFAULT_API_KEY_ENV
}

/// Get input file path.
#[allow(dead_code)]
pub fn get_input_file() -> &'static str {
    DEFAULT_INPUT_FILE
}

/// Get output file path.
#[allow(dead_code)]
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
