use std::collections::BTreeSet;

use crate::ops::foundation::knowledge_graph_store::KnowledgeGraphStore;
use crate::ops::foundation::knowledge_record::EvidenceRef;
use crate::ops::foundation::roaming_engine::CandidateScope;

// 2026-04-08 CST: 这里定义 retrieval hit，原因是 Task 7 需要把“命中节点、分数、证据引用”
// 收口成 retrieval 层自己的稳定输出，而不是让上层回头再拼节点信息。
// 目的：为 evidence assembly 提供可直接消费的最小命中对象。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievalHit {
    pub node_id: String,
    pub score: usize,
    pub evidence_refs: Vec<EvidenceRef>,
}

// 2026-04-08 CST: 这里定义 retrieval 失败边界，原因是候选域内完全没有命中时，
// 需要在 retrieval 层明确表达，而不是返回空数组让下游继续猜测。
// 目的：固定 scoped retrieval 的最小失败语义。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetrievalEngineError {
    NoEvidenceFound { question: String },
}

// 2026-04-08 CST: 这里将 retrieval engine 保持为无状态执行器，原因是当前阶段只需要在给定问题、
// CandidateScope 与 graph store 上做一次受控检索，不应过早引入缓存或外部索引。
// 目的：维持 foundation 内核实现最小化。
#[derive(Debug, Clone, Default)]
pub struct RetrievalEngine;

impl RetrievalEngine {
    // 2026-04-08 CST: 这里提供最小构造函数，原因是测试与后续 pipeline 需要稳定入口，
    // 但当前没有需要显式注入的配置。
    // 目的：保持调用面简洁。
    pub fn new() -> Self {
        Self
    }

    // 2026-04-08 CST: 这里实现候选域内检索，原因是 retrieval 只能消费 roaming 给出的 scope，
    // 不能绕过主链去全图扫描，否则 foundation 分层会被破坏。
    // 目的：先用最小关键词交集评分把 concept scope 映射成稳定排序的命中列表。
    pub fn retrieve(
        &self,
        question: &str,
        scope: &CandidateScope,
        graph_store: &KnowledgeGraphStore,
    ) -> Result<Vec<RetrievalHit>, RetrievalEngineError> {
        let question_tokens = tokenize(question);
        let concept_id_refs: Vec<&str> = scope.concept_ids.iter().map(String::as_str).collect();
        let scoped_node_ids = graph_store.node_ids_for_concepts(&concept_id_refs);
        let mut hits = Vec::new();

        for node_id in scoped_node_ids {
            let Some(node) = graph_store.node(node_id) else {
                continue;
            };

            let score = overlap_score(&question_tokens, &node.title, &node.body);
            if score == 0 {
                continue;
            }

            hits.push(RetrievalHit {
                node_id: node.id.clone(),
                score,
                evidence_refs: node.evidence_refs.clone(),
            });
        }

        hits.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.node_id.cmp(&right.node_id))
        });

        if hits.is_empty() {
            return Err(RetrievalEngineError::NoEvidenceFound {
                question: question.to_string(),
            });
        }

        Ok(hits)
    }
}

// 2026-04-08 CST: 这里抽取最小分词逻辑，原因是当前评分只依赖大小写无关的 token 交集，
// 不需要提前引入更重的文本处理策略。
// 目的：让问题文本与节点文本走同一套归一化规则，保证评分可预测。
fn tokenize(text: &str) -> BTreeSet<String> {
    text.split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(|token| token.to_ascii_lowercase())
        .collect()
}

// 2026-04-08 CST: 这里采用最小交集评分，原因是 Task 7 当前只需要稳定排序与无命中边界，
// 不需要更复杂的权重模型。
// 目的：先以最低复杂度跑通 scoped retrieval 的核心行为。
fn overlap_score(question_tokens: &BTreeSet<String>, title: &str, body: &str) -> usize {
    let mut content_tokens = tokenize(title);
    content_tokens.extend(tokenize(body));

    question_tokens
        .iter()
        .filter(|token| content_tokens.contains(*token))
        .count()
}
