/// 用户意图类型
#[derive(Debug, Clone, PartialEq)]
pub enum Intent {
    // 直接操作命令（无需 LLM）
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

    // 需要 LLM 的命令
    AskQuestion,
    SummarizePapers,
    SummarizeNotes,
    ExplainConcept,
    GenerateOutline,

    // === 模型管理命令（新增）===
    ListModel,
    ShowModel,
    AddModel,
    SwitchModel,
    DeleteModel,
    ValidateModel,
}

/// 关键词模式定义
pub struct KeywordPattern;

impl KeywordPattern {
    /// 返回所有关键词模式（正则表达式, 对应意图）
    pub fn all() -> Vec<(&'static str, Intent)> {
        vec![
            // 退出命令
            (r"^(exit|quit|q|bye)$", Intent::Exit),
            (r"^(quit|exit|bye)\s+", Intent::Exit),

            // 帮助命令
            (r"^(help|h|\?)$", Intent::Help),
            (r"^(help|h|\?)\s+", Intent::Help),

            // 清屏命令
            (r"^(clear|cls)$", Intent::Clear),
            (r"^(clear|cls)\s+", Intent::Clear),

            // 列出论文
            (r"^list\s+(papers|paper|pdf|pdfs)$", Intent::ListPapers),
            (r"^papers$", Intent::ListPapers),
            (r"^show\s+(papers|paper|pdf|pdfs)$", Intent::ListPapers),

            // 列出笔记
            (r"^list\s+(notes|note|docs|documents)$", Intent::ListNotes),
            (r"^notes$", Intent::ListNotes),
            (r"^show\s+(notes|note|docs|documents)$", Intent::ListNotes),

            // 搜索论文
            (r"^search\s+", Intent::SearchPapers),
            (r"^find\s+", Intent::SearchPapers),
            (r"^grep\s+", Intent::SearchPapers),
            (r"^lookup\s+", Intent::SearchPapers),
            (r"^search\s+papers?\s+", Intent::SearchPapers),
            (r"^find\s+papers?\s+", Intent::SearchPapers),

            // 搜索笔记
            (r"^search\s+notes?\s+", Intent::SearchNotes),
            (r"^find\s+notes?\s+", Intent::SearchNotes),

            // 设置知识库路径
            (r"^set\s+kb\s+", Intent::SetKnowledgeBase),
            (r"^change\s+kb\s+", Intent::SetKnowledgeBase),
            (r"^set\s+knowledge(-|\s)?base\s+", Intent::SetKnowledgeBase),
            (r"^cd\s+", Intent::SetKnowledgeBase),

            // 初始化
            (r"^init$", Intent::Initialize),
            (r"^init\s+", Intent::Initialize),
            (r"^initialize$", Intent::Initialize),

            // 提取元数据
            (r"^extract-metadata$", Intent::ExtractMetadata),
            (r"^extract\s+metadata$", Intent::ExtractMetadata),
            (r"^extract\s+meta$", Intent::ExtractMetadata),
            (r"^extract$", Intent::ExtractMetadata),

            // 构建 Wiki
            (r"^build-wiki$", Intent::BuildWiki),
            (r"^build\s+wiki$", Intent::BuildWiki),
            (r"^generate\s+wiki$", Intent::BuildWiki),
            (r"^update\s+wiki$", Intent::BuildWiki),
            (r"^rebuild\s+wiki$", Intent::BuildWiki),

            // 提问（需要 LLM）
            (r"^ask\s+", Intent::AskQuestion),
            (r"^question\s+", Intent::AskQuestion),
            (r"^tell\s+me\s+about\s+", Intent::AskQuestion),
            (r"^explain\s+", Intent::AskQuestion),
            (r"^what\s+is\s+", Intent::AskQuestion),
            (r"^how\s+(do|does|to|can|would|should)\s+", Intent::AskQuestion),
            (r"^why\s+", Intent::AskQuestion),
            (r"^when\s+", Intent::AskQuestion),
            (r"^which\s+", Intent::AskQuestion),
            (r"^define\s+", Intent::AskQuestion),
            (r"^describe\s+", Intent::AskQuestion),
            (r"^compare\s+", Intent::AskQuestion),

            // 总结论文（需要 LLM）
            (r"^summarize\s+papers?$", Intent::SummarizePapers),
            (r"^summarise\s+papers?$", Intent::SummarizePapers),
            (r"^summary\s+papers?$", Intent::SummarizePapers),
            (r"^sum\s+papers?$", Intent::SummarizePapers),
            (r"^papers?\s+summary$", Intent::SummarizePapers),
            (r"^papers?\s+summar(y|ise)$", Intent::SummarizePapers),

            // 总结笔记（需要 LLM）
            (r"^summarize\s+notes?$", Intent::SummarizeNotes),
            (r"^summarise\s+notes?$", Intent::SummarizeNotes),
            (r"^summary\s+notes?$", Intent::SummarizeNotes),
            (r"^sum\s+notes?$", Intent::SummarizeNotes),
            (r"^notes?\s+summary$", Intent::SummarizeNotes),
            (r"^notes?\s+summar(y|ise)$", Intent::SummarizeNotes),

            // 解释概念（需要 LLM）
            (r"^explain\s+concept\s+", Intent::ExplainConcept),
            (r"^what\s+(is|are)\s+", Intent::ExplainConcept),
            (r"^define\s+", Intent::ExplainConcept),

            // 生成大纲（需要 LLM）
            (r"^generate\s+outline", Intent::GenerateOutline),
            (r"^create\s+outline", Intent::GenerateOutline),
            (r"^outline\s+", Intent::GenerateOutline),

            // === 模型管理命令（新增）===
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

    /// 获取意图的友好描述
    pub fn description(intent: &Intent) -> &'static str {
        match intent {
            Intent::Help => "显示帮助信息",
            Intent::Exit => "退出 REPL",
            Intent::Clear => "清屏",
            Intent::ListPapers => "列出所有论文",
            Intent::ListNotes => "列出所有笔记",
            Intent::SearchPapers => "搜索论文",
            Intent::SearchNotes => "搜索笔记",
            Intent::SetKnowledgeBase => "设置知识库路径",
            Intent::Initialize => "初始化知识库",
            Intent::ExtractMetadata => "提取论文元数据",
            Intent::BuildWiki => "构建 Wiki 页面",
            Intent::AskQuestion => "提问（需要 LLM）",
            Intent::SummarizePapers => "总结论文（需要 LLM）",
            Intent::SummarizeNotes => "总结笔记（需要 LLM）",
            Intent::ExplainConcept => "解释概念（需要 LLM）",
            Intent::GenerateOutline => "生成大纲（需要 LLM）",
            // === 模型管理（新增）===
            Intent::ListModel => "列出所有配置的模型",
            Intent::ShowModel => "显示当前模型详情",
            Intent::AddModel => "添加新模型",
            Intent::SwitchModel => "切换到指定模型",
            Intent::DeleteModel => "删除指定模型",
            Intent::ValidateModel => "验证模型配置",
        }
    }

    /// 判断意图是否需要 LLM
    pub fn is_llm_required(intent: &Intent) -> bool {
        matches!(
            intent,
            Intent::AskQuestion
                | Intent::SummarizePapers
                | Intent::SummarizeNotes
                | Intent::ExplainConcept
                | Intent::GenerateOutline
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_description() {
        assert_eq!(KeywordPattern::description(&Intent::Help), "显示帮助信息");
        assert_eq!(KeywordPattern::description(&Intent::Exit), "退出 REPL");
    }

    #[test]
    fn test_is_llm_required() {
        assert!(KeywordPattern::is_llm_required(&Intent::AskQuestion));
        assert!(KeywordPattern::is_llm_required(&Intent::SummarizePapers));
        assert!(!KeywordPattern::is_llm_required(&Intent::ListPapers));
        assert!(!KeywordPattern::is_llm_required(&Intent::Help));
    }
}
