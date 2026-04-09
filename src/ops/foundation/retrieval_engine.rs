use std::collections::BTreeSet;

use crate::ops::foundation::knowledge_graph_store::KnowledgeGraphStore;
use crate::ops::foundation::knowledge_record::EvidenceRef;
use crate::ops::foundation::metadata_registry::{
    MetadataFieldTarget, MetadataRegistry, MetadataRegistryError,
};
use crate::ops::foundation::roaming_engine::CandidateScope;

// 2026-04-07 CST: 这里定义 retrieval hit，原因是 Task 7 需要把“命中了哪个节点、分数是多少、证据来自哪里”
// 作为 retrieval 层自己的稳定输出，而不是让上层再回头重拼节点信息。
// 目的：先固定最小命中载体，给后续 evidence assembly 一个可直接消费的结果对象。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievalHit {
    pub node_id: String,
    pub score: usize,
    pub evidence_refs: Vec<EvidenceRef>,
}

// 2026-04-07 CST: 这里定义 retrieval 错误边界，原因是当前架构要求 retrieval 只负责候选域内证据命中，
// 所以“没有命中证据”应该在本层显式表达，而不是返回空数组把语义丢给下游继续猜。
// 目的：把 retrieval 阶段的最小失败语义固定下来，便于主链后续稳定处理空命中场景。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetrievalEngineError {
    NoEvidenceFound { question: String },
    InvalidMetadataConstraint(MetadataRegistryError),
}

// 2026-04-07 CST: 这里保留 retrieval engine 为无状态执行器，原因是当前 Task 7 只需要在给定问题、
// 候选域和 graph store 上做一次受控检索，不需要提前引入缓存、索引器或外部依赖。
// 目的：坚持 foundation 主线“先最小可用，再逐层增强”的节奏，不让 retrieval 过早膨胀。
#[derive(Debug, Clone, Default)]
pub struct RetrievalEngine;

impl RetrievalEngine {
    // 2026-04-07 CST: 这里提供最小构造函数，原因是测试和后续 orchestrator 都需要一个稳定入口，
    // 但当前阶段没有任何可配置项，不应该为了形式而引入空配置对象。
    // 目的：把 Task 7 的调用方式固定成简单、可复用的无状态执行器接口。
    pub fn new() -> Self {
        Self
    }

    // 2026-04-07 CST: 这里实现候选域内检索入口，原因是 retrieval 在 foundation 架构里只能消费
    // roaming 给出的 CandidateScope，不能直接从全图开始搜索，否则会破坏“route -> roam -> retrieve”的顺序。
    // 目的：先用最小关键词交集评分把候选概念映射到候选节点，再按分数输出稳定的 hit 列表。
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

            // 2026-04-09 CST: 这里先在评分前应用 MetadataConstraint，原因是 metadata 是候选证据收敛条件，
            // 不应等命中完成后再做结果裁剪，否则会混淆“没有证据”与“证据被约束排除”的阶段语义。
            // 目的：确保 RetrievalEngine 真正工作在 metadata 收敛后的候选节点集合上。
            if !scope.metadata_scope.matches(node) {
                continue;
            }

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

    // 2026-04-09 CST: 这里补带 metadata registry 的 retrieval 入口，原因是字段目录阶段需要让 retrieval 显式知道
    // 哪些约束字段应该作用于 node，而不是把 concept-only 字段也拿来过滤节点。
    // 目的：让 node-level metadata 检索正式切到字段注册表驱动的标准语义。
    pub fn retrieve_with_metadata_registry(
        &self,
        question: &str,
        scope: &CandidateScope,
        graph_store: &KnowledgeGraphStore,
        metadata_registry: &MetadataRegistry,
    ) -> Result<Vec<RetrievalHit>, RetrievalEngineError> {
        let question_tokens = tokenize(question);
        // 2026-04-09 CST: 这里先对 node-target 约束做 registry 校验，原因是 registry 模式的目标不是“静默跳过非法字段”，
        // 而是把字段治理问题显式暴露出来。
        // 目的：让 retrieval 在开始打分前就区分“约束非法”和“合法但没有命中证据”。
        let applicable_constraints = scope
            .metadata_scope
            .constraints_for_registered_target(metadata_registry, MetadataFieldTarget::Node)
            .map_err(RetrievalEngineError::InvalidMetadataConstraint)?;
        let concept_id_refs: Vec<&str> = scope.concept_ids.iter().map(String::as_str).collect();
        let scoped_node_ids = graph_store.node_ids_for_concepts(&concept_id_refs);
        let mut hits = Vec::new();

        for node_id in scoped_node_ids {
            let Some(node) = graph_store.node(node_id) else {
                continue;
            };

            if !applicable_constraints
                .iter()
                .all(|constraint| constraint.matches_metadata(&node.metadata))
            {
                continue;
            }

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

// 2026-04-07 CST: 这里提取最小分词函数，原因是当前评分只依赖大小写无关的关键词交集，
// 不需要提前引入更重的文本分析策略，但仍然要保证 title/body 与问题文本走同一套归一化规则。
// 目的：让排序测试和无命中测试都依赖同一份可预测的文本切分逻辑，减少后续行为抖动。
fn tokenize(text: &str) -> BTreeSet<String> {
    text.split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(|token| token.to_ascii_lowercase())
        .collect()
}

// 2026-04-07 CST: 这里提取关键词交集评分，原因是 Task 7 当前只需要最小可用的 scoped retrieval，
// 所以评分模型保持简单透明，直接统计问题 token 和节点 title/body token 的交集数量即可。
// 目的：先把“能稳定排序、能明确无命中”的基础行为跑通，为后续更复杂评分留出替换位置。
fn overlap_score(question_tokens: &BTreeSet<String>, title: &str, body: &str) -> usize {
    let mut content_tokens = tokenize(title);
    content_tokens.extend(tokenize(body));

    question_tokens
        .iter()
        .filter(|token| content_tokens.contains(*token))
        .count()
}
