use crate::ops::foundation::ontology_store::OntologyStore;

// 2026-04-08 CST: 这里定义导航请求，原因是 router 作为主链起点不应继续直接消费裸字符串，
// 需要先把问题文本收口到独立输入对象，为后续扩展请求元信息留位置。
// 目的：固定 capability router 的输入边界。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavigationRequest {
    pub question: String,
}

impl NavigationRequest {
    // 2026-04-08 CST: 这里提供最小构造函数，原因是当前阶段只需要稳定承载问题文本，
    // 不需要为了形式引入更重的 builder 或上下文对象。
    // 目的：让测试和后续 pipeline 调用方式保持简单直接。
    pub fn new(question: impl Into<String>) -> Self {
        Self {
            question: question.into(),
        }
    }
}

// 2026-04-08 CST: 这里定义路由结果，原因是 router 的职责只是把问题映射到种子 concept，
// 不能在这一层混入 node 命中、证据或业务结果。
// 目的：把输出收敛为 concept id 列表，保持层次清晰。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityRoute {
    pub matched_concept_ids: Vec<String>,
}

// 2026-04-08 CST: 这里定义 router 失败边界，原因是主链如果连概念都没命中，
// 就应该在最前面明确失败，而不是给后面几层传空输入。
// 目的：稳定暴露 router 的最小失败语义。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityRouterError {
    NoConceptMatched { question: String },
}

// 2026-04-08 CST: 这里把 router 设计成依赖 ontology store 的轻量路由器，原因是 concept 的查找规则
// 应由 store/schema 统一托管，而不是散落在一堆文本辅助函数里。
// 目的：把问题文本到 seed concept 的映射稳定收口。
#[derive(Debug, Clone)]
pub struct CapabilityRouter {
    ontology_store: OntologyStore,
}

impl CapabilityRouter {
    // 2026-04-08 CST: 这里提供构造函数，原因是 router 需要显式持有 ontology store，
    // 后续就算查找策略变化，也不必改上层调用面。
    // 目的：固定 router 的依赖边界。
    pub fn new(ontology_store: OntologyStore) -> Self {
        Self { ontology_store }
    }

    // 2026-04-08 CST: 这里实现最小路由入口，原因是当前阶段已经明确采用
    // “短语优先、token 回退”的策略来避免多词 alias 被拆散。
    // 目的：把问题文本稳定映射到种子 concept id 列表。
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

        Ok(CapabilityRoute {
            matched_concept_ids,
        })
    }

    // 2026-04-08 CST: 这里把匹配过程拆开，原因是 phrase-first 是当前 router 的核心行为，
    // 单独收口更容易继续补边界测试，不把复杂度塞进 route 主函数。
    // 目的：先用清晰步骤实现长短语优先、单词兜底和结果去重。
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
                    if !matched_concept_ids
                        .iter()
                        .any(|matched| matched == concept_id)
                    {
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

    // 2026-04-08 CST: 这里做最小 token 化，原因是当前阶段只需要支持英文字母数字和多词短语，
    // 不应在还没有明确需求时过早引入 NLP 或更重文本标准化。
    // 目的：给短语优先匹配提供统一、可预测的输入。
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
