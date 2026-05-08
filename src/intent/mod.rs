pub mod keywords;

pub use keywords::Intent;

use regex::Regex;
use crate::intent::keywords::KeywordPattern;

/// Intent parser
/// Identifies user intent from input (direct commands vs LLM requirements)
pub struct IntentParser {
    patterns: Vec<(Regex, Intent)>,
}

impl IntentParser {
    /// Create new intent parser
    pub fn new() -> Self {
        let mut patterns = Vec::new();

        // Build regex patterns
        for (pattern_str, intent) in KeywordPattern::all() {
            match Regex::new(pattern_str) {
                Ok(regex) => patterns.push((regex, intent)),
                Err(e) => {
                    eprintln!("Warning: Failed to compile regex '{}': {}", pattern_str, e);
                }
            }
        }

        IntentParser { patterns }
    }

    /// Parse user input, return identified intent
    pub fn parse(&self, input: &str) -> Option<Intent> {
        let input_lower = input.to_lowercase();

        for (regex, intent) in &self.patterns {
            if regex.is_match(&input_lower) {
                return Some(intent.clone());
            }
        }

        None
    }

    /// Check if LLM is required.
    ///
    /// The deterministic `kb>` shell does not expose any LLM-triggering intent.
    #[allow(dead_code)]
    pub fn requires_llm(&self, _intent: &Intent) -> bool {
        false
    }

    /// Extract search query
    pub fn extract_query(&self, input: &str, intent: &Intent) -> Option<String> {
        match intent {
            Intent::SearchPapers | Intent::SearchNotes => {
                // Handle various search patterns
                let patterns = ["search ", "find ", "grep ", "lookup ", "search papers ", "search notes "];
                for pattern in &patterns {
                    if let Some(query) = input.strip_prefix(pattern) {
                        return Some(query.trim().to_string());
                    }
                    if let Some(query) = input.strip_prefix(&pattern.to_lowercase()) {
                        return Some(query.trim().to_string());
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Extract model ID (for switch/delete/validate commands)
    pub fn extract_model_id(&self, input: &str, intent: &Intent) -> Option<String> {
        match intent {
            Intent::SwitchModel | Intent::DeleteModel | Intent::ValidateModel => {
                let patterns = ["switch model ", "switch to ", "use model ", "delete model ", "remove model ", "del model ", "validate model ", "test model "];
                for pattern in &patterns {
                    if let Some(id) = input.strip_prefix(pattern) {
                        return Some(id.trim().to_string());
                    }
                    if let Some(id) = input.strip_prefix(&pattern.to_lowercase()) {
                        return Some(id.trim().to_string());
                    }
                }
                None
            }
            _ => None,
        }
    }
}

impl Default for IntentParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_search() {
        let parser = IntentParser::new();
        assert!(matches!(
            parser.parse("search droplet"),
            Some(Intent::SearchPapers)
        ));
    }

    #[test]
    fn test_free_form_llm_like_text_is_not_parsed() {
        let parser = IntentParser::new();
        assert_eq!(parser.parse("ask what is t-junction"), None);
        assert_eq!(parser.parse("summarize papers"), None);
        assert_eq!(parser.parse("explain transformation thermotics"), None);
    }

    #[test]
    fn test_parse_list() {
        let parser = IntentParser::new();
        assert!(matches!(
            parser.parse("list papers"),
            Some(Intent::ListPapers)
        ));
    }

    #[test]
    fn test_requires_llm() {
        let parser = IntentParser::new();
        assert!(!parser.requires_llm(&Intent::ListPapers));
        assert!(!parser.requires_llm(&Intent::SearchPapers));
    }
}
