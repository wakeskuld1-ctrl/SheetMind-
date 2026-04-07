use crate::ops::foundation::ontology_store::OntologyStore;

// 2026-04-07 CST: 这里定义导航请求，原因是 capability router 是 foundation 主链起点，
// 需要把“原始问题文本”收敛成独立输入对象，避免后续模块直接依赖裸字符串。
// 目的：先固定路由输入边界，为后续扩展请求元信息保留稳定落点。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavigationRequest {
    pub question: String,
}

impl NavigationRequest {
    // 2026-04-07 CST: 这里提供请求构造函数，原因是当前 Task 5 的 TDD 测试需要直接声明问题文本，
    // 没必要为最小路由闭环过早引入更复杂的 builder 或上下文对象。
    // 目的：先让 capability router 拥有稳定且简单的输入创建方式。
    pub fn new(question: impl Into<String>) -> Self {
        Self {
            question: question.into(),
        }
    }
}

// 2026-04-07 CST: 这里定义路由结果，原因是 router 的职责是把问题映射成种子概念集，
// 不应该提前混入 graph hit、evidence 或其他下游模块数据。
// 目的：先用最小结构把“命中的概念 id 列表”固定下来，保持主链边界清晰。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityRoute {
    pub matched_concept_ids: Vec<String>,
}

// 2026-04-07 CST: 这里定义路由错误，原因是当问题里完全没有已知概念线索时，
// 主链应该明确停下来，而不是默默返回空结果让后续 roaming/retrieval 继续误跑。
// 目的：把 router 的失败边界显式化，方便后续 orchestrator 或上层 UI 做清晰反馈。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityRouterError {
    NoConceptMatched { question: String },
}

// 2026-04-07 CST: 这里把 capability router 设计成只依赖 ontology store 的轻量路由层，
// 原因是 ontology store 已经收口了概念名与别名查询，router 不应该反向窥探 schema 内部结构。
// 目的：继续坚持 ontology-first 的架构顺序，让“问题 -> 概念”这一步独立于 retrieval。
#[derive(Debug, Clone)]
pub struct CapabilityRouter {
    ontology_store: OntologyStore,
}

impl CapabilityRouter {
    // 2026-04-07 CST: 这里新增 router 构造函数，原因是 Task 5 需要一个显式的路由对象来持有
    // ontology store，后续可以在不改调用面的前提下继续演进匹配策略。
    // 目的：先把 capability router 的依赖边界固定为 store，而不是散落的辅助函数。
    pub fn new(ontology_store: OntologyStore) -> Self {
        Self { ontology_store }
    }

    // 2026-04-07 CST: 这里实现最小路由入口，原因是方案 B 已确定采用“短语优先、token 回退”
    // 的匹配顺序，以避免多词 alias 被拆散后丢失语义。
    // 目的：先把问题文本稳定映射到种子概念列表，为后续 roaming 提供干净输入。
    pub fn route(
        &self,
        request: &NavigationRequest,
    ) -> Result<CapabilityRoute, CapabilityRouterError> {
        let tokens = Self::tokenize_question(&request.question);
        let matched_concept_ids = self.match_concept_ids(&tokens);

        if matched_concept_ids.is_empty() {
            return Err(CapabilityRouterError::NoConceptMatched {
                question: request.question.clone(),
            });
        }

        Ok(CapabilityRoute { matched_concept_ids })
    }

    // 2026-04-07 CST: 这里把匹配过程拆成独立函数，原因是路由策略本身是 Task 5 的核心变化点，
    // 单独收口后更容易继续补 phrase/token 边界测试，而不影响外部接口。
    // 目的：用清晰步骤实现“长短语优先、单词兜底、结果去重”的最小契约。
    fn match_concept_ids(&self, tokens: &[String]) -> Vec<String> {
        let mut matched_concept_ids = Vec::new();
        let mut covered_tokens = vec![false; tokens.len()];

        for span_length in (1..=tokens.len()).rev() {
            for start in 0..=tokens.len().saturating_sub(span_length) {
                let end = start + span_length;
                if covered_tokens[start..end].iter().any(|covered| *covered) {
                    continue;
                }

                let candidate = tokens[start..end].join(" ");
                if let Some(concept_id) = self.ontology_store.find_concept_id(&candidate) {
                    if !matched_concept_ids.iter().any(|matched| matched == concept_id) {
                        matched_concept_ids.push(concept_id.to_string());
                    }

                    for covered in &mut covered_tokens[start..end] {
                        *covered = true;
                    }
                }
            }
        }

        matched_concept_ids
    }

    // 2026-04-07 CST: 这里做最小问题 token 化，原因是当前 capability router 只需要支持
    // 英文单词与多词短语匹配，不应该在 Task 5 过早引入 NLP 或更重的文本标准化链路。
    // 目的：先以低成本把标点噪声剥掉并统一小写，给 phrase-first 匹配提供稳定输入。
    fn tokenize_question(question: &str) -> Vec<String> {
        question
            .chars()
            .map(|character| {
                if character.is_alphanumeric() {
                    character.to_ascii_lowercase()
                } else {
                    ' '
                }
            })
            .collect::<String>()
            .split_whitespace()
            .map(ToString::to_string)
            .collect()
    }
}
