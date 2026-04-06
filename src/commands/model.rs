use clap::Args;
use anyhow::Result;

use crate::commands::model_config::ModelManager;

/// Bash 模式的模型管理命令

/// 列出所有配置的模型
pub fn execute_list_models() -> Result<()> {
    match ModelManager::new() {
        Ok(manager) => {
            let models = manager.list_models();

            println!("Configured Models");
            println!("=================\n");

            for model in &models {
                let is_default = if let Some(current_model) = manager.get_current_model() {
                    current_model.id == model.id
                } else {
                    false
                };

                if is_default {
                    print!("[DEFAULT] ");
                } else {
                    print!("          ");
                }

                println!("{}", model.id);
                println!("  Name: {}", model.name);
                println!("  URL: {}", model.url);

                match &model.api_key_source {
                    crate::commands::model_config::ApiKeySource::Env => {
                        if let Some(env) = &model.api_key_env {
                            println!("  API Key: {}", env);
                        }
                    }
                    crate::commands::model_config::ApiKeySource::Direct => {
                        if let Some(_) = &model.api_key_value {
                            println!("  API Key: *** (stored in config)");
                        }
                    }
                }

                println!("  Status: {}", if model.enabled { "Enabled" } else { "Disabled" });
                println!();
            }

            println!("Total: {} models configured", models.len());

            if let Some(current) = manager.get_current_model() {
                println!("Default: {}", current.id);
            }
        }
        Err(e) => {
            eprintln!("Error loading model configuration: {}", e);
        }
    }

    Ok(())
}

/// 显示当前模型详情
pub fn execute_show_model() -> Result<()> {
    match ModelManager::new() {
        Ok(manager) => {
            println!("Current Model Configuration");
            println!("===========================\n");

            if let Some(model) = manager.get_current_model() {
                println!("Model ID: {}", model.id);
                println!("Name: {}", model.name);
                println!("URL: {}", model.url);

                match &model.api_key_source {
                    crate::commands::model_config::ApiKeySource::Env => {
                        if let Some(env) = &model.api_key_env {
                            println!("API Key Source: Environment");
                            println!("API Key: {}", env);
                        }
                    }
                    crate::commands::model_config::ApiKeySource::Direct => {
                        println!("API Key Source: Direct (stored in config)");
                    }
                }

                if let Some(model_name) = &model.model_name {
                    println!("Model: {}", model_name);
                }

                println!("Status: Active");
            } else {
                println!("No active model configured.");
                println!("Run 'list-models' to see available models.");
                println!("Run 'switch-model <id>' to set an active model.");
            }
        }
        Err(e) => {
            eprintln!("Error loading model configuration: {}", e);
        }
    }

    Ok(())
}

/// 添加新模型（非交互式）
#[derive(Args, Clone, Default)]
pub struct ModelArgs {
    #[arg(short = 'i', long = "id")]
    id: Option<String>,

    #[arg(short = 'n', long = "name")]
    name: Option<String>,

    #[arg(short = 'u', long = "url")]
    url: Option<String>,

    #[arg(long = "api-key-env")]
    api_key_env: Option<String>,

    #[arg(long = "api-key-value")]
    api_key_value: Option<String>,

    #[arg(short = 'm', long = "model")]
    model_name: Option<String>,

    #[arg(short = 'd', long = "description")]
    description: Option<String>,
}

pub fn execute_add_model(args: ModelArgs) -> Result<()> {
    let id = args.id.unwrap_or_else(|| {
        // 从 name 生成 id（如果未提供）
        args.name.as_ref()
            .map(|n| n.to_lowercase().replace(' ', "-"))
            .unwrap_or_else(|| "custom-model".to_string())
    });

    let name = args.name.unwrap_or_else(|| "Custom Model".to_string());
    let url = args.url.unwrap_or_else(|| "http://localhost:8080/v1/chat/completions".to_string());

    let mut manager = match ModelManager::new() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error loading model configuration: {}", e);
            return Ok(());
        }
    };

    // 判断 API Key 来源
    let api_key_source = if args.api_key_env.is_some() {
        crate::commands::model_config::ApiKeySource::Env
    } else if args.api_key_value.is_some() {
        crate::commands::model_config::ApiKeySource::Direct
    } else {
        crate::commands::model_config::ApiKeySource::Env  // 默认使用环境变量
    };

    let entry = crate::commands::model_config::ModelEntry {
        id: id.clone(),
        name,
        url,
        api_key_source,
        api_key_env: args.api_key_env,
        api_key_value: args.api_key_value,
        model_name: args.model_name,
        description: args.description,
        created_at: chrono::Utc::now(),
        enabled: true,
    };

    match manager.add_model(entry.clone()) {
        Ok(_) => {
            if let Err(e) = manager.save() {
                eprintln!("Error saving configuration: {}", e);
            } else {
                println!("Model added successfully!");
                println!();
                println!("Model ID: {}", id);
                println!("Name: {}", entry.name);
                println!("URL: {}", entry.url);
                println!();
                println!("To set as active: ./cli.exe switch-model {}", id);
            }
        }
        Err(e) => {
            eprintln!("Error adding model: {}", e);
        }
    }

    Ok(())
}

/// 切换模型
pub fn execute_switch_model(id: &str) -> Result<()> {
    let mut manager = match ModelManager::new() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error loading model configuration: {}", e);
            return Ok(());
        }
    };

    match manager.switch_model(id) {
        Ok(_) => {
            if let Err(e) = manager.save() {
                eprintln!("Error saving configuration: {}", e);
            } else {
                if let Some(model) = manager.get_current_model() {
                    println!("Switched active model to: {} ({})", model.id, model.name);
                    println!();
                    println!("Verify with: ./cli.exe show-model");
                }
            }
        }
        Err(e) => {
            eprintln!("Error switching model: {}", e);
            println!();
            println!("Available models:");
            for model in manager.list_models() {
                println!("  - {} ({})", model.id, model.name);
            }
        }
    }

    Ok(())
}

/// 删除模型
pub fn execute_delete_model(id: &str) -> Result<()> {
    let mut manager = match ModelManager::new() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error loading model configuration: {}", e);
            return Ok(());
        }
    };

    // 获取当前模型信息（用于确认）
    #[allow(unused_variables)]
    let old_default = manager.get_current_model().map(|m| m.id.clone());

    match manager.delete_model(id) {
        Ok(_) => {
            if let Err(e) = manager.save() {
                eprintln!("Error saving configuration: {}", e);
            } else {
                println!("Model '{}' deleted.", id);

                // 检查默认模型是否改变
                if let Some(new_default) = manager.get_current_model() {
                    println!("Default model switched to: {} ({})", new_default.id, new_default.name);
                }
            }
        }
        Err(e) => {
            eprintln!("Error deleting model: {}", e);
            println!();
            println!("Available models:");
            for model in manager.list_models() {
                println!("  - {} ({})", model.id, model.name);
            }
        }
    }

    Ok(())
}

/// 验证模型（可选实现）
pub fn execute_validate_model(id: Option<&str>) -> Result<()> {
    let manager = match ModelManager::new() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error loading model configuration: {}", e);
            return Ok(());
        }
    };

    let models = manager.list_models();
    let model = if let Some(id) = id {
        // 验证指定模型
        models.iter().find(|m| m.id == id).map(|r| *r)
    } else {
        // 验证当前模型
        manager.get_current_model()
    };

    match model {
        Some(model) => {
            println!("Validating model: {} ({})", model.id, model.name);
            println!();
            println!("Configuration:");
            println!("  URL: {}", model.url);

            match &model.api_key_source {
                crate::commands::model_config::ApiKeySource::Env => {
                    if let Some(env) = &model.api_key_env {
                        println!("  API Key Source: Environment ({})", env);
                        match std::env::var(env) {
                            Ok(_) => println!("  API Key: Set"),
                            Err(_) => println!("  API Key: Not set"),
                        }
                    }
                }
                crate::commands::model_config::ApiKeySource::Direct => {
                    if model.api_key_value.is_some() {
                        println!("  API Key Source: Direct (stored in config)");
                        println!("  API Key: Set");
                    } else {
                        println!("  API Key: Not set");
                    }
                }
            }

            println!();
            println!("Note: Network validation is not implemented.");
            println!("To test connectivity, use 'repl' mode and try 'ask' command.");
        }
        None => {
            println!("No model to validate.");
            println!();
            println!("Available models:");
            for model in manager.list_models() {
                println!("  - {} ({})", model.id, model.name);
            }
        }
    }

    Ok(())
}
