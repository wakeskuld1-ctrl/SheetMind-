use crate::ops::foundation::metadata_constraint::{MetadataConstraint, MetadataScope};
use crate::ops::foundation::ontology_store::OntologyStore;

// 2026-04-07 CST: 这里定义导航请求，原因是 capability router 是 foundation 主链起点，
// 需要把“原始问题文本”收敛成独立输入对象，避免后续模块直接依赖裸字符串。
// 目的：先固定路由输入边界，为后续扩展请求元信息保留稳定落点。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavigationRequest {
    pub question: String,
    // 2026-04-09 CST: 这里增加 required_concept_tags，原因是 foundation 路由现在要支持最小 scope 约束，
    // 让同一短语在不同概念候选之间能按调用方给出的标签范围收敛。
    // 目的：先用轻量标签约束验证“同词不同域”的主链行为，不提前引入更重的过滤 DSL。
    pub required_concept_tags: Vec<String>,
    // 2026-04-09 CST: 这里新增 metadata_constraints，原因是 foundation 第一轮通用元数据能力要通过 request 正式进入主链，
    // 不能让 metadata 过滤停留在临时参数或 retrieval 私有接口里。
    // 目的：把 MetadataConstraint 提升为 NavigationRequest 的标准输入之一。
    pub metadata_scope: MetadataScope,
}

impl NavigationRequest {
    // 2026-04-07 CST: 这里提供请求构造函数，原因是当前 Task 5 的 TDD 测试需要直接声明问题文本，
    // 没必要为最小路由闭环过早引入更复杂的 builder 或上下文对象。
    // 目的：先让 capability router 拥有稳定且简单的输入创建方式。
    pub fn new(question: impl Into<String>) -> Self {
        Self {
            question: question.into(),
            required_concept_tags: Vec::new(),
            metadata_scope: MetadataScope::new(),
        }
    }

    // 2026-04-09 CST: 这里增加请求级概念标签约束，原因是方案 A 这轮要先把“路由可约束”能力落到
    // foundation 主线，而不是把同词消歧推迟到业务层特判。
    // 目的：给 route 一个最小、稳定、可测试的 scope 输入。
    pub fn with_required_concept_tags(mut self, tags: Vec<&str>) -> Self {
        self.required_concept_tags = tags.into_iter().map(str::to_string).collect();
        self
    }

    // 2026-04-09 CST: 这里补 request 级 metadata 约束构造器，原因是通用元数据能力应由调用方显式声明，
    // 再由 pipeline 统一传递到 retrieval，而不是让下游偷偷附带过滤条件。
    // 目的：给 foundation 主链提供标准化、可测试的 metadata 输入入口。
    pub fn with_metadata_constraints(mut self, constraints: Vec<MetadataConstraint>) -> Self {
        self.metadata_scope = MetadataScope::from_constraints(constraints);
        self
    }
}

// 2026-04-07 CST: 这里定义路由结果，原因是 router 的职责是把问题映射成种子概念集，
// 不应该提前混入 graph hit、evidence 或其他下游模块数据。
// 目的：先用最小结构把“命中的概念 id 列表”固定下来，保持主链边界清晰。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityRoute {
    pub matched_concept_ids: Vec<String>,
    // 2026-04-09 CST: 这里追加 matched_terms，原因是当前 foundation 路由需要给下游保留
    // “是问题里的哪段短语命中了概念”，不能只丢 concept id 后再让 trace/调试层回头猜。
    // 目的：先把最小可解释路由证据固化下来，为后续导航 trace 和观测能力提供稳定输入。
    pub matched_terms: Vec<String>,
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
        let matched_candidates =
            self.match_candidates(&tokens, request.required_concept_tags.as_slice());
        let matched_concept_ids = matched_candidates
            .iter()
            .map(|candidate| candidate.concept_id.clone())
            .collect::<Vec<_>>();
        let matched_terms = matched_candidates
            .iter()
            .map(|candidate| candidate.matched_term.clone())
            .collect::<Vec<_>>();

        if matched_concept_ids.is_empty() {
            return Err(CapabilityRouterError::NoConceptMatched {
                question: request.question.clone(),
            });
        }

        Ok(CapabilityRoute {
            matched_concept_ids,
            matched_terms,
        })
    }

    // 2026-04-09 CST: 这里把“命中概念 + 命中短语”收口成局部结构，原因是方案 A 第一刀要在不引入
    // 业务字段的前提下提升 foundation 路由的可解释性。
    // 目的：保持 phrase-first 的匹配顺序不变，同时把 matched term 一起保留下来。
    fn match_candidates(
        &self,
        tokens: &[String],
        required_tags: &[String],
    ) -> Vec<MatchedConcept> {
        let mut matched_concept_ids = Vec::new();
        let mut matched_terms = Vec::new();
        let mut covered_tokens = vec![false; tokens.len()];

        for span_length in (1..=tokens.len()).rev() {
            for start in 0..=tokens.len().saturating_sub(span_length) {
                let end = start + span_length;
                if covered_tokens[start..end].iter().any(|covered| *covered) {
                    continue;
                }

                let candidate = tokens[start..end].join(" ");
                let constrained_candidates =
                    self.constrained_concept_ids(&candidate, required_tags);
                if let Some(concept_id) = constrained_candidates.first() {
                    if !matched_concept_ids.iter().any(|matched| matched == concept_id) {
                        matched_concept_ids.push((*concept_id).to_string());
                        matched_terms.push(candidate.clone());
                    }

                    for covered in &mut covered_tokens[start..end] {
                        *covered = true;
                    }
                }
            }
        }

        matched_concept_ids
            .into_iter()
            .zip(matched_terms)
            .map(|(concept_id, matched_term)| MatchedConcept {
                concept_id,
                matched_term,
            })
            .collect()
    }

    // 2026-04-09 CST: 这里集中做候选过滤，原因是 schema 现在允许同一 lookup key 返回多个 concept，
    // router 需要在 phrase-first 流程里就地应用最小标签约束。
    // 目的：把多候选 lookup 与 request scope 收口到一个局部步骤，避免后续匹配主循环继续膨胀。
    fn constrained_concept_ids<'a>(
        &'a self,
        raw_candidate: &str,
        required_tags: &[String],
    ) -> Vec<&'a str> {
        let concept_ids = self.ontology_store.find_concept_ids(raw_candidate);
        if required_tags.is_empty() {
            return concept_ids;
        }

        concept_ids
            .into_iter()
            .filter(|concept_id| {
                self.ontology_store
                    .concept(concept_id)
                    .map(|concept| {
                        required_tags
                            .iter()
                            .any(|required_tag| concept.tags.iter().any(|tag| tag == required_tag))
                    })
                    .unwrap_or(false)
            })
            .collect()
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

// 2026-04-09 CST: 这里增加局部命中结构，原因是当前 route 组装需要同时携带 concept id 和 matched term，
// 直接并排维护两个向量会让后续扩展 trace 字段时更容易错位。
// 目的：先用最小内部结构把“命中了谁”和“为什么命中”绑定在一起，保持实现清晰。
#[derive(Debug, Clone, PartialEq, Eq)]
struct MatchedConcept {
    concept_id: String,
    matched_term: String,
}
