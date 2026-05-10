use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Result};

/// Deterministic interactive shell.
///
/// The shell is intentionally not an LLM chat interface. It handles only
/// shell/session commands directly and delegates all knowledge-base commands
/// back to the same `kb` binary so batch mode and shell mode share the same
/// command semantics.
pub fn execute(custom_kb: Option<&Path>) -> Result<()> {
    let mut state = ShellState::new(custom_kb);

    println!("kb shell v0.6.0.1");
    println!("Deterministic interactive command shell. Type `help` or `exit`.");
    println!("Current KB: {}", state.kb_path.display());

    let stdin = io::stdin();
    loop {
        print!("kb> ");
        io::stdout().flush()?;

        let mut input = String::new();
        let bytes_read = stdin.read_line(&mut input)?;
        if bytes_read == 0 {
            println!();
            break;
        }

        let line = input.trim();
        if line.is_empty() {
            continue;
        }

        if let Err(err) = state.handle_line(line) {
            eprintln!("error: {err}");
        }

        if state.should_exit {
            break;
        }
    }

    Ok(())
}

struct ShellState {
    kb_path: PathBuf,
    should_exit: bool,
}

impl ShellState {
    fn new(custom_kb: Option<&Path>) -> Self {
        Self {
            kb_path: crate::commands::init::get_kb_path(custom_kb),
            should_exit: false,
        }
    }

    fn handle_line(&mut self, line: &str) -> Result<()> {
        let tokens = split_shell_words(line)?;
        if tokens.is_empty() {
            return Ok(());
        }

        match tokens[0].as_str() {
            "exit" | "quit" | "q" => {
                self.should_exit = true;
                Ok(())
            }
            "help" | "h" | "?" => {
                print_help();
                Ok(())
            }
            "pwd" => {
                println!("{}", self.kb_path.display());
                Ok(())
            }
            "use" | "cd" => self.set_kb_path(&tokens),
            "clear" | "cls" => {
                print_clear();
                Ok(())
            }
            "shell" | "repl" => {
                println!("Already inside kb shell. Use `exit` to leave.");
                Ok(())
            }
            _ if is_known_batch_command(&tokens[0]) => self.run_batch_command(&tokens),
            _ => {
                println!("Unknown command. No action taken. Type `help` for available commands.");
                Ok(())
            }
        }
    }

    fn set_kb_path(&mut self, tokens: &[String]) -> Result<()> {
        if tokens.len() < 2 {
            return Err(anyhow!("usage: use <knowledge-base-path>"));
        }
        let next_path = PathBuf::from(tokens[1..].join(" "));
        self.kb_path = next_path;
        println!("Current KB: {}", self.kb_path.display());
        Ok(())
    }

    fn run_batch_command(&self, tokens: &[String]) -> Result<()> {
        let exe = std::env::current_exe()
            .map_err(|err| anyhow!("could not locate current executable: {err}"))?;
        let status = Command::new(exe)
            .arg("--kb-path")
            .arg(&self.kb_path)
            .args(tokens)
            .status()
            .map_err(|err| anyhow!("failed to run kb command: {err}"))?;

        if !status.success() {
            if let Some(code) = status.code() {
                eprintln!("command exited with status code {code}");
            } else {
                eprintln!("command terminated by signal");
            }
        }
        Ok(())
    }
}

fn is_known_batch_command(command: &str) -> bool {
    matches!(
        command,
        "init"
            | "ingest"
            | "bootstrap"
            | "build-wiki"
            | "prepare"
            | "query"
            | "links"
            | "keywords"
            | "refs"
            | "refs-index"
            | "refs-graph"
            | "tasks"
            | "memory"
            | "grep"
            | "health"
            | "topic"
            | "view"
            | "sync-wiki"
            | "lint-static"
            | "status"
            | "extract-metadata"
            | "extract-text"
            | "list-models"
            | "show-model"
            | "add-model"
            | "switch-model"
            | "delete-model"
            | "validate-model"
    )
}

fn print_help() {
    println!();
    println!("kb shell commands:");
    println!("  use <path>        Set the knowledge base path for later commands");
    println!("  pwd               Show the current knowledge base path");
    println!("  clear             Clear the terminal screen");
    println!("  help              Show this help");
    println!("  exit              Leave shell mode");
    println!();
    println!("Batch command mapping:");
    println!("  kb --kb-path PATH query thermal cloak");
    println!("  kb> use PATH");
    println!("  kb> query thermal cloak");
    println!();
    println!("Common deterministic commands:");
    println!("  status --unprocessed");
    println!("  query <terms...>");
    println!("  grep <pattern> --path processing/text");
    println!("  extract-text --dry-run");
    println!("  refs --path processing/text");
    println!("  refs-index --dry-run");
    println!("  refs-graph --json --dry-run");
    println!("  keywords <terms...> --dry-run");
    println!("  links --unresolved");
    println!("  health --dry-run");
    println!("  view --dry-run");
    println!("  topic init <topic>");
    println!("  topic list");
    println!("  topic status <topic>");
    println!("  topic rank <topic> --dry-run");
    println!("  topic review <topic> --dry-run");
    println!("  tasks --dry-run");
    println!("  memory --task-id ID --summary TEXT");
    println!();
    println!("Boundary:");
    println!("  kb> is a deterministic command shell, not an LLM chat interface.");
    println!("  Free-form natural-language or LLM-style requests are not shell commands.");
    println!("  Unknown input is a safe no-op: the shell does not guess intent.");
    println!("  LLM work should be represented through explicit LLM/tasks/ handoff files.");
}

fn print_clear() {
    print!("\x1B[2J\x1B[H");
    let _ = io::stdout().flush();
}

fn split_shell_words(input: &str) -> Result<Vec<String>> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut quote: Option<char> = None;

    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                if let Some(active_quote) = quote {
                    if let Some(&next) = chars.peek() {
                        if next == active_quote || next == '\\' {
                            current.push(next);
                            chars.next();
                        } else {
                            current.push(ch);
                        }
                    } else {
                        current.push(ch);
                    }
                } else {
                    current.push(ch);
                }
            }
            '\'' | '"' => {
                if quote == Some(ch) {
                    quote = None;
                } else if quote.is_none() {
                    quote = Some(ch);
                } else {
                    current.push(ch);
                }
            }
            ch if ch.is_whitespace() && quote.is_none() => {
                if !current.is_empty() {
                    words.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }

    if let Some(q) = quote {
        return Err(anyhow!("unterminated quote: {q}"));
    }

    if !current.is_empty() {
        words.push(current);
    }

    Ok(words)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_shell_words_handles_quotes() {
        let words = split_shell_words("query \"thermal cloak\" --limit 5").unwrap();
        assert_eq!(words, vec!["query", "thermal cloak", "--limit", "5"]);
    }

    #[test]
    fn split_shell_words_handles_windows_paths() {
        let words = split_shell_words(r#"use "D:\github\llm-wiki\quantum""#).unwrap();
        assert_eq!(words, vec!["use", r#"D:\github\llm-wiki\quantum"#]);
    }

    #[test]
    fn known_batch_command_whitelist_allows_structured_commands() {
        assert!(is_known_batch_command("query"));
        assert!(is_known_batch_command("refs-index"));
        assert!(is_known_batch_command("health"));
    }

    #[test]
    fn known_batch_command_whitelist_rejects_free_form_text() {
        assert!(!is_known_batch_command("help-me-summarize"));
        assert!(!is_known_batch_command("帮我总结"));
    }
}
