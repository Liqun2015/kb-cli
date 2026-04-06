use std::io::Write;
use std::path::PathBuf;
use anyhow::Result;

use crate::commands::init::get_kb_path;
use crate::commands::model_switch;
use crate::commands::model_config::ModelManager;
use crate::intent::{IntentParser, Intent};

/// Main REPL interactive loop
pub fn execute() -> Result<()> {
    let kb_path = get_kb_path(None);
    let parser = IntentParser::new();

    println!("CLI Knowledge Base Assistant");
    println!("Knowledge base: {}", kb_path.display());
    println!("Type 'help' for available commands or 'exit' to quit");
    println!();

    let stdin = std::io::stdin();
    std::io::stdout().flush();

    loop {
        // Display prompt
        print!("kb> ");
        std::io::stdout().flush()?;

        // Read user input
        let mut input = String::new();
        stdin.read_line(&mut input)?;

        let input = input.trim();

        // Empty input, continue
        if input.is_empty() {
            continue;
        }

        // Parse intent
        if let Some(intent) = parser.parse(input) {
            execute_intent(&kb_path, &parser, &intent, input)?;
        } else {
            println!("Unknown command: '{}'", input);
            println!("Type 'help' for available commands");
        }
    }
}

/// Execute the identified intent
fn execute_intent(kb_path: &PathBuf, parser: &IntentParser, intent: &Intent, input: &str) -> Result<()> {
    match intent {
        // Exit
        Intent::Exit => {
            println!("Goodbye!");
            std::process::exit(0);
        }

        // Help
        Intent::Help => {
            print_help();
        }

        // Clear screen
        Intent::Clear => {
            print_clear();
        }

        // List papers
        Intent::ListPapers => {
            list_papers(kb_path);
        }

        // List notes
        Intent::ListNotes => {
            list_notes(kb_path);
        }

        // Search papers
        Intent::SearchPapers => {
            if let Some(query) = parser.extract_query(input, intent) {
                search_papers(kb_path, &query);
            }
        }

        // Search notes
        Intent::SearchNotes => {
            if let Some(query) = parser.extract_query(input, intent) {
                search_notes(kb_path, &query);
            } else {
                search_papers(kb_path, &input); // Default to searching papers
            }
        }

        // Set knowledge base
        Intent::SetKnowledgeBase => {
            println!("Feature not implemented yet: 'set kb'");
            println!("Current KB: {}", kb_path.display());
        }

        // Initialize
        Intent::Initialize => {
            println!("Run 'cli init' in bash mode to initialize knowledge base.");
        }

        // Extract metadata
        Intent::ExtractMetadata => {
            println!("Run 'cli extract-metadata' in bash mode to extract metadata.");
        }

        // Build Wiki
        Intent::BuildWiki => {
            println!("Run 'cli build-wiki' in bash mode to build wiki pages.");
        }

        // Commands requiring LLM
        Intent::AskQuestion => {
            if let Some(question) = parser.extract_query(input, intent) {
                ask_question(&question);
            } else {
                println!("Usage: ask <question>");
            }
        }

        Intent::SummarizePapers => {
            summarize_papers();
        }

        Intent::SummarizeNotes => {
            summarize_notes();
        }

        Intent::ExplainConcept => {
            if let Some(concept) = parser.extract_query(input, intent) {
                explain_concept(&concept);
            } else {
                println!("Usage: explain <concept>");
            }
        }

        Intent::GenerateOutline => {
            println!("Outline generation feature coming soon.");
        }

        // === Model Management Commands ===
        Intent::ListModel => {
            list_models();
        }

        Intent::ShowModel => {
            show_model();
        }

        Intent::AddModel => {
            add_model_interactive();
        }

        Intent::SwitchModel => {
            if let Some(id) = parser.extract_model_id(input, intent) {
                switch_model(&id);
            } else {
                println!("Usage: switch model <id>");
            }
        }

        Intent::DeleteModel => {
            if let Some(id) = parser.extract_model_id(input, intent) {
                delete_model(&id);
            } else {
                println!("Usage: delete model <id>");
            }
        }

        Intent::ValidateModel => {
            if let Some(id) = parser.extract_model_id(input, intent) {
                validate_model(&id);
            } else {
                validate_current_model();
            }
        }
    }

    Ok(())
}

fn print_help() {
    println!();
    println!("Available commands:");
    println!();
    println!("  Direct commands (no LLM):");
    println!("    init                    Initialize knowledge base");
    println!("    extract-metadata       Extract PDF metadata");
    println!("    build-wiki             Build wiki pages");
    println!("    list papers            List all papers");
    println!("    list notes             List all notes");
    println!("    search <query>         Search for papers/notes");
    println!("    clear                  Clear screen");
    println!("    set kb <path>          Change knowledge base path");
    println!("    help                   Show this help");
    println!("    exit                   Exit REPL");
    println!();
    println!("  Commands requiring LLM:");
    println!("    ask <question>         Ask a question");
    println!("    explain <concept>      Explain a concept");
    println!("    summarize papers       Summarize papers");
    println!("    summarize notes        Summarize notes");
    println!("    what is <term>         Define a term");
    println!("    how to <action>        Get help with something");
    println!();
    println!("Usage examples:");
    println!("  kb> search droplet");
    println!("  kb> ask what is T-junction");
    println!("  kb> explain flow focusing");
    println!("  kb> summarize papers");
    println!("  kb> list papers");
    println!("  kb> help");
}

fn print_clear() {
    // Clear screen (simple implementation)
    print!("\x1B[2J\x1B[2J\x1B[2J"); // VT100 clear screen sequence
    std::io::stdout().flush().ok();
}

fn list_papers(kb_path: &PathBuf) {
    let papers_dir = kb_path.join("wiki/papers");

    if !papers_dir.exists() {
        println!("No papers directory found.");
        return;
    }

    match std::fs::read_dir(&papers_dir) {
        Ok(entries) => {
            let mut count = 0;
            for entry in entries {
                if let Ok(entry) = entry {
                    if entry.path().is_dir() {
                        continue;
                    }
                    if let Some(name) = entry.file_name().to_str() {
                        if name.ends_with(".md") {
                            count += 1;
                            if count <= 10 {
                                println!("  - {}", name.trim_end_matches(".md"));
                            }
                        }
                    }
                }
            }
            println!("\nTotal papers: {}", count);
        }
        Err(e) => {
            eprintln!("Error listing papers: {}", e);
        }
    }
}

fn list_notes(kb_path: &PathBuf) {
    let notes_dir = kb_path.join("wiki/notes");

    if !notes_dir.exists() {
        println!("No notes directory found.");
        return;
    }

    match std::fs::read_dir(&notes_dir) {
        Ok(entries) => {
            let mut count = 0;
            for entry in entries {
                if let Ok(entry) = entry {
                    if entry.path().is_dir() {
                        continue;
                    }
                    if let Some(name) = entry.file_name().to_str() {
                        if name.ends_with(".md") {
                            count += 1;
                            if count <= 10 {
                                println!("  - {}", name.trim_end_matches(".md"));
                            }
                        }
                    }
                }
            }
            println!("\nTotal notes: {}", count);
        }
        Err(e) => {
            eprintln!("Error listing notes: {}", e);
        }
    }
}

fn search_papers(kb_path: &PathBuf, query: &str) {
    let papers_dir = kb_path.join("wiki/papers");

    if !papers_dir.exists() {
        println!("No papers directory found.");
        return;
    }

    match std::fs::read_dir(&papers_dir) {
        Ok(entries) => {
            let query_lower = query.to_lowercase();
            let mut found = false;

            println!("\nSearching for: '{}'\n", query);

            for entry in entries {
                if let Ok(entry) = entry {
                    if entry.path().is_dir() {
                        continue;
                    }
                    if let Some(name) = entry.file_name().to_str() {
                        if name.to_lowercase().contains(&query_lower) {
                            println!("Found: {}", name.trim_end_matches(".md"));
                            found = true;
                        }
                    }
                }
            }

            if !found {
                println!("No matches found for '{}'", query);
            }
        }
        Err(e) => {
            eprintln!("Error searching papers: {}", e);
        }
    }
}

fn search_notes(kb_path: &PathBuf, query: &str) {
    let notes_dir = kb_path.join("wiki/notes");

    if !notes_dir.exists() {
        println!("No notes directory found.");
        return;
    }

    match std::fs::read_dir(&notes_dir) {
        Ok(entries) => {
            let query_lower = query.to_lowercase();
            let mut found = false;

            println!("\nSearching notes for: '{}'\n", query);

            for entry in entries {
                if let Ok(entry) = entry {
                    if entry.path().is_dir() {
                        continue;
                    }
                    if let Some(name) = entry.file_name().to_str() {
                        if name.to_lowercase().contains(&query_lower) {
                            println!("Found: {}", name.trim_end_matches(".md"));
                            found = true;
                        }
                    }
                }
            }

            if !found {
                println!("No matches found for '{}'", query);
            }
        }
        Err(e) => {
            eprintln!("Error searching notes: {}", e);
        }
    }
}

fn ask_question(question: &str) {
    println!("\nQuestion: '{}'", question);

    // Check if model-switch is available
    if !model_switch::is_available() {
        println!("\nModel-switch output file not found.");
        println!("\nTo use LLM features:");
        println!("1. Start the model-switch tool");
        println!("2. Ensure it writes to: {}", model_switch::get_output_file());
        println!("\nDefault configuration:");
        println!("  Model URL: {}", model_switch::get_default_model_url());
        println!("  API Key: ${}", model_switch::get_api_key_env());
        println!("\nOr configure model-switch to write to: {}", model_switch::get_output_file());
        return;
    }

    // Try to read LLM response
    match model_switch::get_llm_response(question, true) {
        Ok(response) => {
            println!("\nResponse:");
            println!("{}", response);
        }
        Err(e) => {
            eprintln!("\nError getting LLM response: {}", e);
        }
    }
}

fn summarize_papers() {
    println!("\nSummarizing recent papers...");

    let prompt = "Please summarize the recent papers in the droplet microfluidics knowledge base. \
                   Focus on key findings, methodologies, and applications.";

    match model_switch::get_llm_response(prompt, true) {
        Ok(response) => {
            println!("\nSummary:");
            println!("{}", response);
        }
        Err(e) => {
            eprintln!("\nError generating summary: {}", e);
        }
    }
}

fn summarize_notes() {
    println!("\nSummarizing notes...");

    let prompt = "Please summarize the key notes and reference materials in the knowledge base. \
                   Highlight the main topics and concepts covered.";

    match model_switch::get_llm_response(prompt, true) {
        Ok(response) => {
            println!("\nSummary:");
            println!("{}", response);
        }
        Err(e) => {
            eprintln!("\nError generating summary: {}", e);
        }
    }
}

fn explain_concept(concept: &str) {
    println!("\nExplaining concept: '{}'", concept);

    let prompt = format!("Please explain the concept of '{}' in the context of droplet microfluidics.", concept);

    match model_switch::get_llm_response(&prompt, true) {
        Ok(response) => {
            println!("\nExplanation:");
            println!("{}", response);
        }
        Err(e) => {
            eprintln!("\nError explaining concept: {}", e);
        }
    }
}

// === Model Management Functions ===

fn list_models() {
    match ModelManager::new() {
        Ok(manager) => {
            let models = manager.list_models();

            println!("\nConfigured Models");
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
                            match std::env::var(env) {
                                Ok(_) => println!("  Status: Set"),
                                Err(_) => println!("  Status: Not set"),
                            }
                        }
                    }
                    crate::commands::model_config::ApiKeySource::Direct => {
                        if model.api_key_value.is_some() {
                            println!("  API Key: *** (stored in config)");
                            println!("  Status: Set");
                        } else {
                            println!("  API Key: Not set");
                        }
                    }
                }

                println!("  Enabled: {}", if model.enabled { "Yes" } else { "No" });
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
}

fn show_model() {
    match ModelManager::new() {
        Ok(manager) => {
            println!("\nCurrent Model Configuration");
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
                            match std::env::var(env) {
                                Ok(_) => println!("Status: Set"),
                                Err(_) => println!("Status: Not set"),
                            }
                        }
                    }
                    crate::commands::model_config::ApiKeySource::Direct => {
                        println!("API Key Source: Direct (stored in config)");
                        println!("API Key: *** (hidden)");
                    }
                }

                if let Some(model_name) = &model.model_name {
                    println!("Model: {}", model_name);
                }

                println!("Status: Active");
                println!();
                println!("To switch models, use: switch model <id>");
                println!("To list models, use: list models");
            } else {
                println!("No active model configured.");
                println!();
                println!("To list available models: list models");
                println!("To set an active model: switch model <id>");
            }
        }
        Err(e) => {
            eprintln!("Error loading model configuration: {}", e);
        }
    }
}

fn add_model_interactive() {
    println!("\nModel Management - Add New Model");
    println!("{}\n", "=".repeat(40));

    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    stdout.flush().ok();

    print!("Enter model ID: ");
    stdout.flush().ok();
    let mut id = String::new();
    stdin.read_line(&mut id).ok();
    let id = id.trim();

    if id.is_empty() {
        println!("Cancelled.");
        return;
    }

    print!("Enter model name: ");
    stdout.flush().ok();
    let mut name = String::new();
    stdin.read_line(&mut name).ok();
    let name = name.trim();

    if name.is_empty() {
        println!("Cancelled.");
        return;
    }

    print!("Enter model URL: ");
    stdout.flush().ok();
    let mut url = String::new();
    stdin.read_line(&mut url).ok();
    let url = url.trim();

    if url.is_empty() {
        println!("Cancelled.");
        return;
    }

    println!("\nAPI Key Source:");
    println!("  1. Environment variable");
    println!("  2. Direct (store in config)");
    print!("\nSelect (1 or 2): ");
    stdout.flush().ok();
    let mut source = String::new();
    stdin.read_line(&mut source).ok();
    let source = source.trim();

    let (api_key_source, api_key_env, api_key_value) = match source {
        "1" | "env" | "environment" => {
            print!("Enter API key environment variable name: ");
            stdout.flush().ok();
            let mut env_var = String::new();
            stdin.read_line(&mut env_var).ok();
            let env_var = env_var.trim();

            (
                crate::commands::model_config::ApiKeySource::Env,
                if env_var.is_empty() { None } else { Some(env_var.to_string()) },
                None,
            )
        }
        "2" | "direct" => {
            print!("Enter API key (will be stored in plain text): ");
            stdout.flush().ok();
            let mut key = String::new();
            stdin.read_line(&mut key).ok();
            let key = key.trim();

            (
                crate::commands::model_config::ApiKeySource::Direct,
                None,
                if key.is_empty() { None } else { Some(key.to_string()) },
            )
        }
        _ => {
            println!("Invalid choice. Cancelled.");
            return;
        }
    };

    print!("\nEnter model name (optional, press Enter to skip): ");
    stdout.flush().ok();
    let mut model_name = String::new();
    stdin.read_line(&mut model_name).ok();
    let model_name = model_name.trim();

    print!("Enter description (optional, press Enter to skip): ");
    stdout.flush().ok();
    let mut description = String::new();
    stdin.read_line(&mut description).ok();
    let description = description.trim();

    let entry = crate::commands::model_config::ModelEntry {
        id: id.to_string(),
        name: name.to_string(),
        url: url.to_string(),
        api_key_source,
        api_key_env,
        api_key_value,
        model_name: if model_name.is_empty() { None } else { Some(model_name.to_string()) },
        description: if description.is_empty() { None } else { Some(description.to_string()) },
        created_at: chrono::Utc::now(),
        enabled: true,
    };

    let mut manager = match ModelManager::new() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error loading model configuration: {}", e);
            return;
        }
    };

    println!("\n[Review Configuration]");
    println!("-----------------------");
    println!("ID: {}", entry.id);
    println!("Name: {}", entry.name);
    println!("URL: {}", entry.url);

    match &entry.api_key_source {
        crate::commands::model_config::ApiKeySource::Env => {
            if let Some(env) = &entry.api_key_env {
                println!("API Key Source: Environment ({})", env);
            }
        }
        crate::commands::model_config::ApiKeySource::Direct => {
            println!("API Key Source: Direct (stored in config)");
        }
    }

    if let Some(model_name) = &entry.model_name {
        println!("Model Name: {}", model_name);
    }

    if let Some(desc) = &entry.description {
        println!("Description: {}", desc);
    }

    println!();
    print!("Confirm add? (yes/no): ");
    stdout.flush().ok();
    let mut confirm = String::new();
    stdin.read_line(&mut confirm).ok();
    let confirm = confirm.trim().to_lowercase();

    if confirm != "yes" && confirm != "y" {
        println!("Cancelled.");
        return;
    }

    let entry_id = entry.id.clone();
    match manager.add_model(entry) {
        Ok(_) => {
            if let Err(e) = manager.save() {
                eprintln!("Error saving configuration: {}", e);
            } else {
                println!("\nModel added successfully!");
                println!();
                print!("Set as default? (yes/no): ");
                stdout.flush().ok();
                let mut set_default = String::new();
                stdin.read_line(&mut set_default).ok();
                let set_default = set_default.trim().to_lowercase();

                if set_default == "yes" || set_default == "y" {
                    match manager.switch_model(&entry_id) {
                        Ok(_) => {
                            if let Err(e) = manager.save() {
                                eprintln!("Error saving configuration: {}", e);
                            } else {
                                println!("Default model set to: {}", entry_id);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error setting default model: {}", e);
                        }
                    }
                } else {
                    println!("Model added but not set as default.");
                    println!("Use 'switch model {}' to set it as default.", entry_id);
                }
            }
        }
        Err(e) => {
            eprintln!("Error adding model: {}", e);
        }
    }
}

fn switch_model(id: &str) {
    let mut manager = match ModelManager::new() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error loading model configuration: {}", e);
            return;
        }
    };

    match manager.switch_model(id) {
        Ok(_) => {
            if let Err(e) = manager.save() {
                eprintln!("Error saving configuration: {}", e);
            } else {
                if let Some(model) = manager.get_current_model() {
                    println!("\nSwitched active model to: {} ({})", model.id, model.name);
                    println!();
                    println!("Verify with: show model");
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
}

fn delete_model(id: &str) {
    let mut manager = match ModelManager::new() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error loading model configuration: {}", e);
            return;
        }
    };

    let old_default = manager.get_current_model().map(|m| m.clone());

    println!("\nWARNING: You are about to delete model '{}'", id);

    if let Some(ref old) = old_default {
        if old.id == id {
            println!("This is currently set as the default model.");
        }
    }

    println!("This action cannot be undone.");
    println!();
    print!("Continue? (yes/no): ");
    std::io::stdout().flush().ok();
    let mut confirm2 = String::new();
    let stdin = std::io::stdin();
    stdin.read_line(&mut confirm2).ok();
    let confirm = confirm2.trim().to_lowercase();

    if confirm != "yes" && confirm != "y" {
        println!("Cancelled.");
        return;
    }

    match manager.delete_model(id) {
        Ok(_) => {
            if let Err(e) = manager.save() {
                eprintln!("Error saving configuration: {}", e);
            } else {
                println!("\nModel '{}' deleted successfully.", id);

                if let Some(new_default) = manager.get_current_model() {
                    if old_default.as_ref().map(|o| o.id != new_default.id).unwrap_or(true) {
                        println!("Default model switched to: {} ({})", new_default.id, new_default.name);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error deleting model: {}", e);
        }
    }
}

fn validate_current_model() {
    validate_model_internal(None);
}

fn validate_model(id: &str) {
    validate_model_internal(Some(id));
}

fn validate_model_internal(id: Option<&str>) {
    match ModelManager::new() {
        Ok(manager) => {
            let models = manager.list_models();
            let model = if let Some(id) = id {
                models.iter().find(|m| m.id == id).map(|r| *r)
            } else {
                manager.get_current_model()
            };

            match model {
                Some(model) => {
                    println!("\nValidating model: {} ({})", model.id, model.name);
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

                    if let Some(model_name) = &model.model_name {
                        println!("  Model: {}", model_name);
                    }

                    println!();
                    println!("Status: Configuration looks valid");
                    println!("Note: Network connectivity not tested in this validation.");
                    println!("To test LLM functionality, use 'ask' command in REPL mode.");
                }
                None => {
                    println!("\nNo model to validate.");
                    println!();
                    println!("Available models:");
                    for model in manager.list_models() {
                        println!("  - {} ({})", model.id, model.name);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error loading model configuration: {}", e);
        }
    }
}
