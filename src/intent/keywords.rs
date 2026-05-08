/// User intent types
#[derive(Debug, Clone, PartialEq)]
pub enum Intent {
    // Direct commands (no LLM required)
    Help,
    Exit,
    Clear,
    ListPapers,
    ListNotes,
    SearchPapers,
    SearchNotes,
    SetKnowledgeBase,
    Initialize,
    ExtractMetadata,
    BuildWiki,


    // === Model Management Commands ===
    ListModel,
    ShowModel,
    AddModel,
    SwitchModel,
    DeleteModel,
    ValidateModel,
}

/// Keyword pattern definitions
pub struct KeywordPattern;

impl KeywordPattern {
    /// Returns all keyword patterns (regex, corresponding intent)
    pub fn all() -> Vec<(&'static str, Intent)> {
        vec![
            // Exit commands
            (r"^(exit|quit|q|bye)$", Intent::Exit),
            (r"^(quit|exit|bye)\s+", Intent::Exit),

            // Help commands
            (r"^(help|h|\?)$", Intent::Help),
            (r"^(help|h|\?)\s+", Intent::Help),

            // Clear screen commands
            (r"^(clear|cls)$", Intent::Clear),
            (r"^(clear|cls)\s+", Intent::Clear),

            // List papers
            (r"^list\s+(papers|paper|pdf|pdfs)$", Intent::ListPapers),
            (r"^papers$", Intent::ListPapers),
            (r"^show\s+(papers|paper|pdf|pdfs)$", Intent::ListPapers),

            // List notes
            (r"^list\s+(notes|note|docs|documents)$", Intent::ListNotes),
            (r"^notes$", Intent::ListNotes),
            (r"^show\s+(notes|note|docs|documents)$", Intent::ListNotes),

            // Search papers
            (r"^search\s+", Intent::SearchPapers),
            (r"^find\s+", Intent::SearchPapers),
            (r"^grep\s+", Intent::SearchPapers),
            (r"^lookup\s+", Intent::SearchPapers),
            (r"^search\s+papers?\s+", Intent::SearchPapers),
            (r"^find\s+papers?\s+", Intent::SearchPapers),

            // Search notes
            (r"^search\s+notes?\s+", Intent::SearchNotes),
            (r"^find\s+notes?\s+", Intent::SearchNotes),

            // Set knowledge base path
            (r"^set\s+kb\s+", Intent::SetKnowledgeBase),
            (r"^change\s+kb\s+", Intent::SetKnowledgeBase),
            (r"^set\s+knowledge(-|\s)?base\s+", Intent::SetKnowledgeBase),
            (r"^cd\s+", Intent::SetKnowledgeBase),

            // Initialize
            (r"^init$", Intent::Initialize),
            (r"^init\s+", Intent::Initialize),
            (r"^initialize$", Intent::Initialize),

            // Extract metadata
            (r"^extract-metadata$", Intent::ExtractMetadata),
            (r"^extract\s+metadata$", Intent::ExtractMetadata),
            (r"^extract\s+meta$", Intent::ExtractMetadata),
            (r"^extract$", Intent::ExtractMetadata),

            // Build Wiki
            (r"^build-wiki$", Intent::BuildWiki),
            (r"^build\s+wiki$", Intent::BuildWiki),
            (r"^generate\s+wiki$", Intent::BuildWiki),
            (r"^update\s+wiki$", Intent::BuildWiki),
            (r"^rebuild\s+wiki$", Intent::BuildWiki),

            // === Model Management Commands ===
            (r"^list\s+models?", Intent::ListModel),
            (r"^models?$", Intent::ListModel),
            (r"^show\s+models?", Intent::ListModel),
            (r"^add\s+model", Intent::AddModel),
            (r"^new\s+model", Intent::AddModel),
            (r"^create\s+model", Intent::AddModel),
            (r"^delete\s+model", Intent::DeleteModel),
            (r"^remove\s+model", Intent::DeleteModel),
            (r"^del\s+model", Intent::DeleteModel),
            (r"^switch\s+model", Intent::SwitchModel),
            (r"^use\s+model", Intent::SwitchModel),
            (r"^set\s+model", Intent::SwitchModel),
            (r"^show\s+model", Intent::ShowModel),
            (r"^current\s+model", Intent::ShowModel),
            (r"^validate\s+model", Intent::ValidateModel),
            (r"^test\s+model", Intent::ValidateModel),
        ]
    }

    /// Get friendly description for intent
    #[allow(dead_code)]
    pub fn description(intent: &Intent) -> &'static str {
        match intent {
            Intent::Help => "Show help information",
            Intent::Exit => "Exit REPL",
            Intent::Clear => "Clear screen",
            Intent::ListPapers => "List all papers",
            Intent::ListNotes => "List all notes",
            Intent::SearchPapers => "Search papers",
            Intent::SearchNotes => "Search notes",
            Intent::SetKnowledgeBase => "Set knowledge base path",
            Intent::Initialize => "Initialize knowledge base",
            Intent::ExtractMetadata => "Extract paper metadata",
            Intent::BuildWiki => "Build Wiki pages",
            // === Model Management ===
            Intent::ListModel => "List all configured models",
            Intent::ShowModel => "Show current model details",
            Intent::AddModel => "Add new model",
            Intent::SwitchModel => "Switch to specified model",
            Intent::DeleteModel => "Delete specified model",
            Intent::ValidateModel => "Validate model configuration",
        }
    }

    /// Check if intent requires LLM.
    ///
    /// In the deterministic `kb>` shell, no parsed intent requires LLM execution.
    #[allow(dead_code)]
    pub fn is_llm_required(_intent: &Intent) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_description() {
        assert_eq!(KeywordPattern::description(&Intent::Help), "Show help information");
        assert_eq!(KeywordPattern::description(&Intent::Exit), "Exit REPL");
    }

    #[test]
    fn test_is_llm_required() {
        assert!(!KeywordPattern::is_llm_required(&Intent::ListPapers));
        assert!(!KeywordPattern::is_llm_required(&Intent::SearchPapers));
        assert!(!KeywordPattern::is_llm_required(&Intent::Help));
    }
}
