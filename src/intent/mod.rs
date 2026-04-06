pub mod keywords;

pub use keywords::Intent;

use regex::Regex;
use crate::intent::keywords::KeywordPattern;

/// 意图解析器
/// 根据用户输入识别其意图（直接命令 vs LLM 需求）
pub struct IntentParser {
    patterns: Vec<(Regex, Intent)>,
}

impl IntentParser {
    /// 创建新的意图解析器
    pub fn new() -> Self {
        let mut patterns = Vec::new();

        // 构建正则表达式模式
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

    /// 解析用户输入，返回识别到的意图
    pub fn parse(&self, input: &str) -> Option<Intent> {
        let input_lower = input.to_lowercase();

        for (regex, intent) in &self.patterns {
            if regex.is_match(&input_lower) {
                return Some(intent.clone());
            }
        }

        None
    }

    /// 判断是否需要 LLM
    pub fn requires_llm(&self, intent: &Intent) -> bool {
        matches!(
            intent,
            Intent::AskQuestion | Intent::SummarizePapers | Intent::SummarizeNotes
        )
    }

    /// 提取搜索查询词
    pub fn extract_query(&self, input: &str, intent: &Intent) -> Option<String> {
        match intent {
            Intent::SearchPapers | Intent::SearchNotes => {
                // 处理各种搜索模式
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
            Intent::AskQuestion | Intent::ExplainConcept => {
                // 处理各种提问模式
                let patterns = [
                    "ask ", "question ", "tell me about ", "explain ", "what is ", "how do ", "how does ",
                    "how to ", "how can ", "how would ", "how should ", "why ", "when ", "which ", "define ", "describe "
                ];
                for pattern in &patterns {
                    if let Some(question) = input.strip_prefix(pattern) {
                        return Some(question.trim().to_string());
                    }
                    if let Some(question) = input.strip_prefix(&pattern.to_lowercase()) {
                        return Some(question.trim().to_string());
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// 提取模型 ID（用于 switch/delete/validate 命令）
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
    fn test_parse_ask() {
        let parser = IntentParser::new();
        assert!(matches!(
            parser.parse("ask what is t-junction"),
            Some(Intent::AskQuestion)
        ));
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
        assert!(parser.requires_llm(&Intent::AskQuestion));
        assert!(!parser.requires_llm(&Intent::ListPapers));
    }
}
