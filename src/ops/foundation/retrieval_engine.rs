use std::collections::{BTreeSet, HashSet};

use crate::ops::foundation::knowledge_graph_store::KnowledgeGraphStore;
use crate::ops::foundation::knowledge_record::{EvidenceRef, KnowledgeNode};
use crate::ops::foundation::roaming_engine::CandidateScope;

// 2026-04-07 CST: 这里定义 retrieval hit，原因是 Task 7 需要把“命中了哪个节点、分数是多少、证据来自哪里”
// 2026-04-07 CST: 作为 retrieval 层自己的稳定输出，而不是让上层再回头重拼节点信息。
// 2026-04-07 CST: 目的：先固定最小命中载体，给后续 evidence assembly 一个可直接消费的结果对象。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievalHit {
    pub node_id: String,
    pub score: usize,
    pub evidence_refs: Vec<EvidenceRef>,
}

// 2026-04-09 CST: 这里新增 retrieval 执行结果对象，原因是方案 A 需要在不破坏原有 `retrieve()` 合同的前提下，
// 2026-04-09 CST: 为 foundation 内部补出“命中列表 + 解释列表”的并行输出；目的：把 diagnostics 留在 retrieval 层内部，而不是提前扩张到 pipeline。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievalExecution {
    pub hits: Vec<RetrievalHit>,
    pub diagnostics: Vec<RetrievalDiagnostic>,
}

// 2026-04-09 CST: 这里新增 retrieval 诊断结构，原因是当前排序链已经存在，但后续 AI 还看不到“为什么命中”和“为什么排在这里”；
// 2026-04-09 CST: 目的：把 title/body/phrase/seed/source/evidence/locator 信号落成稳定合同，便于后续排查与交接。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievalDiagnostic {
    pub node_id: String,
    pub matched_title_tokens: Vec<String>,
    pub matched_body_tokens: Vec<String>,
    pub title_overlap: usize,
    pub body_overlap: usize,
    pub phrase_bonus: usize,
    pub seed_bonus: usize,
    pub text_score: usize,
    pub final_score: usize,
    pub source_priority: usize,
    pub evidence_ref_count: usize,
    pub best_locator: Option<String>,
    pub locator_priority: (usize, usize),
    pub duplicate_evidence_ref_count: usize,
    pub weak_locator_count: usize,
    pub weak_source_ref_count: usize,
    pub hygiene_flags: Vec<RetrievalHygieneFlag>,
}

// 2026-04-09 CST: 这里新增 retrieval hygiene 标记枚举，原因是 evidence diagnostics 已经不只解释排序，
// 2026-04-09 CST: 还需要解释“这些证据本身有没有质量风险”；目的：把重复证据、弱 locator、弱 source_ref 固化成可测试、可扩展的风险标签。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetrievalHygieneFlag {
    DuplicateEvidenceRefs,
    WeakLocator,
    WeakSourceRef,
}

// 2026-04-07 CST: 这里定义 retrieval 错误边界，原因是当前架构要求 retrieval 只负责候选域内证据命中，
// 2026-04-07 CST: 所以“没有命中证据”应该在本层显式表达，而不是返回空数组把语义丢给下游继续猜。
// 2026-04-07 CST: 目的：把 retrieval 阶段的最小失败语义固定下来，便于主链后续稳定处理空命中场景。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetrievalEngineError {
    NoEvidenceFound { question: String },
}

// 2026-04-07 CST: 这里保留 retrieval engine 为无状态执行器，原因是 Task 7 只需要在给定问题、
// 2026-04-07 CST: 候选域和 graph store 上做一次受控检索，不需要提前引入缓存、索引器或外部依赖。
// 2026-04-07 CST: 目的：坚持 foundation 主线“先最小可用，再逐层增强”的节奏，不让 retrieval 过早膨胀。
#[derive(Debug, Clone, Default)]
pub struct RetrievalEngine;

// 2026-04-09 CST: 这里新增 retrieval 内部候选对象，原因是 diagnostics 与排序规则现在共享同一批信号，
// 2026-04-09 CST: 如果继续分开各算一遍，后续很容易出现“排序逻辑改了、解释没改”的串台；目的：让命中结果和解释结果共用同一条执行链。
#[derive(Debug, Clone, PartialEq, Eq)]
struct ScoredRetrievalCandidate {
    hit: RetrievalHit,
    diagnostic: RetrievalDiagnostic,
}

impl RetrievalEngine {
    // 2026-04-07 CST: 这里提供最小构造函数，原因是测试和后续 orchestrator 都需要一个稳定入口，
    // 2026-04-07 CST: 但当前阶段没有任何可配置项，不应该为了形式而引入空配置对象。
    // 2026-04-07 CST: 目的：把 Task 7 的调用方式固定成简单、可复用的无状态执行器接口。
    pub fn new() -> Self {
        Self
    }

    // 2026-04-07 CST: 这里实现候选域内检索入口，原因是 retrieval 在 foundation 架构里只能消费
    // 2026-04-07 CST: roaming 给出的 CandidateScope，不能直接从全图开始搜索，否则会破坏“route -> roam -> retrieve”的顺序。
    // 2026-04-07 CST: 目的：先用最小排序模型把候选概念映射到候选节点，再按稳定分数输出 hit 列表。
    // 2026-04-08 CST: 本次追加第一层排序增强，原因是单纯 token 交集已经不足以稳定表达标题信号、
    // 2026-04-08 CST: 短语命中和 seed concept 优先级；目的：在不拆 RetrievalConfig 的前提下把检索从“可跑”推进到“更稳排序”。
    // 2026-04-09 CST: 这里继续追加第二层来源 tie-break，原因是用户已确认 source_ref 只能做次级排序，不能反压文本相关性。
    // 2026-04-09 CST: 目的：在不改 RetrievalHit 合同、不引入配置层的前提下，让同分命中更优先返回 primary/derived 来源。
    pub fn retrieve(
        &self,
        question: &str,
        scope: &CandidateScope,
        graph_store: &KnowledgeGraphStore,
    ) -> Result<Vec<RetrievalHit>, RetrievalEngineError> {
        self.retrieve_with_diagnostics(question, scope, graph_store)
            .map(|execution| execution.hits)
    }

    // 2026-04-09 CST: 这里新增 foundation 内部 diagnostics 入口，原因是方案 A 已确认本轮只做 retrieval 内部可解释化，
    // 2026-04-09 CST: 不接 CLI / Tool / GUI；目的：让底座调用方可以直接拿到排序解释，同时保持旧 `retrieve()` 合同不变。
    pub fn retrieve_with_diagnostics(
        &self,
        question: &str,
        scope: &CandidateScope,
        graph_store: &KnowledgeGraphStore,
    ) -> Result<RetrievalExecution, RetrievalEngineError> {
        let question_tokens = tokenize(question);
        let normalized_question = normalized_text(question);
        let seed_concept_ids = seed_concept_ids(scope);
        let concept_id_refs: Vec<&str> = scope.concept_ids.iter().map(String::as_str).collect();
        let scoped_node_ids = graph_store.node_ids_for_concepts(&concept_id_refs);
        let mut candidates = Vec::new();

        for node_id in scoped_node_ids {
            let Some(node) = graph_store.node(node_id) else {
                continue;
            };

            let Some(candidate) = build_scored_candidate(
                &question_tokens,
                &normalized_question,
                &seed_concept_ids,
                node,
            ) else {
                continue;
            };

            candidates.push(candidate);
        }

        candidates.sort_by(|left, right| {
            right
                .diagnostic
                .final_score
                .cmp(&left.diagnostic.final_score)
                .then_with(|| {
                    left.diagnostic
                        .source_priority
                        .cmp(&right.diagnostic.source_priority)
                })
                .then_with(|| {
                    right
                        .diagnostic
                        .evidence_ref_count
                        .cmp(&left.diagnostic.evidence_ref_count)
                })
                .then_with(|| {
                    left.diagnostic
                        .locator_priority
                        .cmp(&right.diagnostic.locator_priority)
                })
                .then_with(|| left.hit.node_id.cmp(&right.hit.node_id))
        });

        if candidates.is_empty() {
            return Err(RetrievalEngineError::NoEvidenceFound {
                question: question.to_string(),
            });
        }

        let hits = candidates
            .iter()
            .map(|candidate| candidate.hit.clone())
            .collect::<Vec<_>>();
        let diagnostics = candidates
            .into_iter()
            .map(|candidate| candidate.diagnostic)
            .collect::<Vec<_>>();

        Ok(RetrievalExecution { hits, diagnostics })
    }
}

// 2026-04-08 CST: 这里补最小评分聚合函数，原因是 Task 11 要在不改外部合同的前提下增强排序，
// 2026-04-08 CST: 因此需要把标题命中、正文命中、短语 bonus 与 seed bonus 收口到同一个位置。
// 2026-04-08 CST: 目的：让 retrieval 排序增强保持局部、透明、可测试，而不是把评分逻辑散落在主流程里。
fn build_scored_candidate(
    question_tokens: &BTreeSet<String>,
    normalized_question: &str,
    seed_concept_ids: &BTreeSet<String>,
    node: &KnowledgeNode,
) -> Option<ScoredRetrievalCandidate> {
    let title_tokens = tokenize(&node.title);
    let body_tokens = tokenize(&node.body);
    let matched_title_tokens = matched_tokens(question_tokens, &title_tokens);
    let matched_body_tokens = matched_tokens(question_tokens, &body_tokens);
    let title_overlap = matched_title_tokens.len();
    let body_overlap = matched_body_tokens.len();
    let phrase_bonus = phrase_bonus(normalized_question, &node.title, &node.body);
    let text_score = title_overlap * 3 + body_overlap + phrase_bonus;

    // 2026-04-08 CST: 这里先守住“无文本命中就不算 retrieval hit”的旧合同，原因是 seed bonus 只是排序增强，
    // 2026-04-08 CST: 不能把原本应该报 NoEvidenceFound 的节点硬抬成命中，否则 retrieval 语义会被结构 bonus 污染。
    // 2026-04-08 CST: 目的：确保 Task 11 只增强排序，不破坏 Task 7 已固定的命中边界。
    if text_score == 0 {
        return None;
    }

    let seed_bonus = if node
        .concept_ids
        .iter()
        .any(|concept_id| seed_concept_ids.contains(concept_id))
    {
        2
    } else {
        0
    };

    let final_score = text_score + seed_bonus;
    let source_priority = evidence_source_priority(&node.evidence_refs);
    let evidence_ref_count = evidence_ref_count_priority(&node.evidence_refs);
    let (best_locator, locator_priority) = best_locator_diagnostic(&node.evidence_refs);
    let duplicate_evidence_ref_count = duplicate_evidence_ref_count(&node.evidence_refs);
    let weak_locator_count = weak_locator_count(&node.evidence_refs);
    let weak_source_ref_count = weak_source_ref_count(&node.evidence_refs);
    let hygiene_flags = hygiene_flags(
        duplicate_evidence_ref_count,
        weak_locator_count,
        weak_source_ref_count,
    );

    Some(ScoredRetrievalCandidate {
        hit: RetrievalHit {
            node_id: node.id.clone(),
            score: final_score,
            evidence_refs: node.evidence_refs.clone(),
        },
        diagnostic: RetrievalDiagnostic {
            node_id: node.id.clone(),
            matched_title_tokens,
            matched_body_tokens,
            title_overlap,
            body_overlap,
            phrase_bonus,
            seed_bonus,
            text_score,
            final_score,
            source_priority,
            evidence_ref_count,
            best_locator,
            locator_priority,
            duplicate_evidence_ref_count,
            weak_locator_count,
            weak_source_ref_count,
            hygiene_flags,
        },
    })
}

// 2026-04-09 CST: 这里补节点证据来源优先级聚合，原因是 retrieval 第二层增强只允许在同分阶段比较 source_ref。
// 2026-04-09 CST: 节点可能挂多个 EvidenceRef，因此需要一个稳定聚合口径，而不是把来源判断散落到 sort_by 闭包里。
// 2026-04-09 CST: 目的：把“来源优先级只做 tie-break”的规则收口到单点，后续维护和测试都更直接。
fn evidence_source_priority(evidence_refs: &[EvidenceRef]) -> usize {
    evidence_refs
        .iter()
        .map(|evidence_ref| source_ref_priority(&evidence_ref.source_ref))
        .min()
        .unwrap_or(0)
}

// 2026-04-09 CST: 这里补证据数量优先级聚合，原因是方案 C 的第一层证据侧 tie-break 就是 evidence_refs 数量。
// 2026-04-09 CST: 这个信号只能排在文本分数和来源优先级之后，不能提前参与主分数或命中判定。
// 2026-04-09 CST: 目的：让“证据更丰富的节点更优先”成为稳定、可解释、可回归验证的排序规则。
fn evidence_ref_count_priority(evidence_refs: &[EvidenceRef]) -> usize {
    evidence_refs.len()
}

// 2026-04-09 CST: 这里补重复证据计数 helper，原因是 evidence count diagnostics 不能把“真实更多证据”和“同一证据重复出现”
// 2026-04-09 CST: 混成一个信号；目的：让后续 AI 能明确识别节点是不是靠重复引用造成证据膨胀感。
fn duplicate_evidence_ref_count(evidence_refs: &[EvidenceRef]) -> usize {
    let mut seen_refs = HashSet::new();
    let mut duplicate_count = 0usize;

    for evidence_ref in evidence_refs {
        let evidence_key = (
            evidence_ref.source_ref.clone(),
            evidence_ref.locator.clone(),
        );
        if !seen_refs.insert(evidence_key) {
            duplicate_count += 1;
        }
    }

    duplicate_count
}

// 2026-04-09 CST: 这里补弱 locator 计数 helper，原因是 diagnostics 现在已经会解释 locator 精度，
// 2026-04-09 CST: 但还需要告诉后续 AI “这个定位本身是否太弱”；目的：先用最小启发式把空值、无法解析和值域过宽的 locator 纳入风险视图。
fn weak_locator_count(evidence_refs: &[EvidenceRef]) -> usize {
    evidence_refs
        .iter()
        .filter(|evidence_ref| is_weak_locator(&evidence_ref.locator))
        .count()
}

// 2026-04-09 CST: 这里补弱 source_ref 计数 helper，原因是 source priority 只能表达排序层级，
// 2026-04-09 CST: 不能表达“来源命名是否过泛”；目的：把空来源和低区分度占位来源纳入最小 hygiene 诊断。
fn weak_source_ref_count(evidence_refs: &[EvidenceRef]) -> usize {
    evidence_refs
        .iter()
        .filter(|evidence_ref| is_weak_source_ref(&evidence_ref.source_ref))
        .count()
}

// 2026-04-09 CST: 这里统一聚合 hygiene 标记，原因是调用方后续更容易先看 flags 再看 counts，
// 2026-04-09 CST: 而不是自己重新拼规则；目的：让 retrieval diagnostics 对 AI 和人工阅读都更直接。
fn hygiene_flags(
    duplicate_evidence_ref_count: usize,
    weak_locator_count: usize,
    weak_source_ref_count: usize,
) -> Vec<RetrievalHygieneFlag> {
    let mut flags = Vec::new();

    if duplicate_evidence_ref_count > 0 {
        flags.push(RetrievalHygieneFlag::DuplicateEvidenceRefs);
    }
    if weak_locator_count > 0 {
        flags.push(RetrievalHygieneFlag::WeakLocator);
    }
    if weak_source_ref_count > 0 {
        flags.push(RetrievalHygieneFlag::WeakSourceRef);
    }

    flags
}

// 2026-04-09 CST: 这里补 diagnostics 版最佳 locator 聚合，原因是排序时我们只需要精度 tuple，但解释时还需要明确
// 2026-04-09 CST: “哪一个 locator 被当成最具体证据”；目的：让 diagnostics 同时具备机器可排序值和人工可阅读值。
fn best_locator_diagnostic(evidence_refs: &[EvidenceRef]) -> (Option<String>, (usize, usize)) {
    evidence_refs
        .iter()
        .map(|evidence_ref| {
            (
                evidence_ref.locator.clone(),
                locator_precision_priority(&evidence_ref.locator),
            )
        })
        .min_by(|left, right| left.1.cmp(&right.1).then_with(|| left.0.cmp(&right.0)))
        .map(|(locator, priority)| (Some(locator), priority))
        .unwrap_or((None, (usize::MAX, usize::MAX)))
}

// 2026-04-09 CST: 这里补弱 locator 判定 helper，原因是 hygiene diagnostics 需要区分“可排序 locator”与“可用 locator”；
// 2026-04-09 CST: 目的：先用最小启发式把空值、不可解析和过宽范围识别出来，而不引入配置系统。
fn is_weak_locator(locator: &str) -> bool {
    let trimmed_locator = locator.trim();
    if trimmed_locator.is_empty() {
        return true;
    }

    let locator_priority = locator_precision_priority(trimmed_locator);
    match locator_priority {
        (2, _) => true,
        (1, area) if area > 100 => true,
        _ => false,
    }
}

// 2026-04-09 CST: 这里补节点级 locator 精度聚合，原因是方案 C 在证据数量相同后还要继续比较定位具体程度。
// 2026-04-09 CST: 节点可能带多个 EvidenceRef，因此这里取“最具体的 locator”作为节点代表值，而不是把所有定位压平成平均值。
// 2026-04-09 CST: 目的：让 retrieval 在不改外部合同的前提下优先返回更容易被人工复核的精确证据。
// 2026-04-09 CST: 这里补 locator 精度规则，原因是方案 C 需要在证据数量 tie-break 之后继续表达“更具体定位更优先”。
// 2026-04-09 CST: 本轮只做最小启发式：单点优于范围，范围越小越优，无法识别的 locator 统一落到最低层。
// 2026-04-09 CST: 目的：在保持 foundation 轻量的同时，把 Excel/WPS 这类常见 locator 的可解释精度信号收进排序链。
fn locator_precision_priority(locator: &str) -> (usize, usize) {
    if let Some((column_index, row_index)) = parse_locator_cell(locator) {
        let _ = (column_index, row_index);
        return (0, 1);
    }

    if let Some((start, end)) = parse_locator_range(locator) {
        let column_span = start.0.abs_diff(end.0) + 1;
        let row_span = start.1.abs_diff(end.1) + 1;
        return (1, column_span.saturating_mul(row_span));
    }

    (2, usize::MAX)
}

// 2026-04-09 CST: 这里补 locator 范围解析辅助函数，原因是我们只需要支持最小的 `A1:B3` 形式来承接当前测试与底座信号。
// 2026-04-09 CST: 不引入正则和重型解析器，是为了保持 retrieval 增强仍然局部、简单、可维护。
// 2026-04-09 CST: 目的：把范围型 locator 统一收敛到单点解析能力之上，避免重复解析逻辑。
fn parse_locator_range(locator: &str) -> Option<((usize, usize), (usize, usize))> {
    // 2026-04-09 CST: 这里补 locator 范围解析的最小增强，原因是本轮 hygiene 边界已经明确要接受 `Sheet!A1:B3`
    // 2026-04-09 CST: 和带 `$` 的绝对引用范围；目的：只扩 foundation retrieval 需要的 A1 风格解析，不把完整 Excel 公式语义引进来。
    // 2026-04-09 CST: 追加修改，原因是 `C:\Reports\[Book.xlsx]Sheet!A1:B3` 这类 Windows 绝对路径前缀会让 drive letter 的 `:`
    // 2026-04-09 CST: 抢先命中当前 split_once，导致本来可接受的小范围 locator 被误诊为 weak；目的：先统一剥掉 workbook/sheet 前缀，再切 A1 范围。
    let normalized_locator = normalize_locator_range_text(locator);
    let (start, end) = normalized_locator.split_once(':')?;
    Some((parse_locator_cell(start)?, parse_locator_cell(end)?))
}

// 2026-04-09 CST: 这里补单点 locator 解析辅助函数，原因是 locator 精度规则需要先区分“单点”与“范围”。
// 2026-04-09 CST: 解析口径只覆盖“字母列 + 数字行”的最小格式，超出这个格式的 locator 直接回退到未知层。
// 2026-04-09 CST: 目的：在不扩大合同面的前提下，为 foundation retrieval 提供足够稳定的精度比较基线。
fn parse_locator_cell(locator: &str) -> Option<(usize, usize)> {
    // 2026-04-09 CST: 这里补单点 locator 预处理，原因是真实 evidence locator 常带 sheet 前缀和绝对引用标记，
    // 2026-04-09 CST: 如果直接按裸 `A1` 解析会把正常定位误诊为 weak locator；目的：先把最常见桌面表格定位格式纳入 foundation 可识别集合。
    let trimmed_locator = normalize_locator_cell_text(locator);
    if trimmed_locator.is_empty() {
        return None;
    }

    let mut column_end = 0usize;
    for character in trimmed_locator.chars() {
        if character.is_ascii_alphabetic() {
            column_end += character.len_utf8();
        } else {
            break;
        }
    }

    if column_end == 0 || column_end == trimmed_locator.len() {
        return None;
    }

    let column = &trimmed_locator[..column_end];
    let row = &trimmed_locator[column_end..];
    if !row.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }

    Some((column_index(column)?, row.parse().ok()?))
}

// 2026-04-09 CST: 这里补 locator 归一化 helper，原因是 retrieval hygiene 只需要识别最常见的 `Sheet!A1` 与 `$A$1`
// 2026-04-09 CST: 两类变体；目的：把 sheet 前缀剥离和 `$` 清洗集中到单点，避免解析规则散落在多个 helper 中。
fn normalize_locator_cell_text(locator: &str) -> String {
    locator
        .trim()
        .rsplit_once('!')
        .map(|(_, cell_ref)| cell_ref)
        .unwrap_or(locator.trim())
        .replace('$', "")
}

// 2026-04-09 CST: 这里补范围 locator 归一化 helper，原因是范围解析需要先整体剥离 workbook/sheet 前缀，再识别真正的 `A1:B3`
// 2026-04-09 CST: 分隔符；目的：避免 Windows 路径里的 drive letter `:` 抢先打断最小范围解析，同时保持合同仍只覆盖 A1 风格 locator。
fn normalize_locator_range_text(locator: &str) -> String {
    locator
        .trim()
        .rsplit_once('!')
        .map(|(_, range_ref)| range_ref)
        .unwrap_or(locator.trim())
        .replace('$', "")
}

// 2026-04-09 CST: 这里补列标解析辅助函数，原因是 Excel 风格 locator 的列是字母进制，不适合直接做字符串比较。
// 2026-04-09 CST: 该函数只服务当前 retrieval locator 精度判断，不扩展到更广的表格语义层。
// 2026-04-09 CST: 目的：让范围面积计算具有稳定数值基础，保证 `A1`、`A1:B3` 这类 locator 的比较可预测。
fn column_index(column: &str) -> Option<usize> {
    let mut result = 0usize;

    for character in column.chars() {
        if !character.is_ascii_alphabetic() {
            return None;
        }

        let uppercase = character.to_ascii_uppercase() as usize;
        let letter_value = uppercase.checked_sub('A' as usize)?.checked_add(1)?;
        result = result.checked_mul(26)?.checked_add(letter_value)?;
    }

    Some(result)
}

// 2026-04-09 CST: 这里补 source_ref 固定分层规则，原因是本轮设计已经冻结为 primary > derived > planning。
// 2026-04-09 CST: 规则只服务 foundation 底座，不带业务词，也不开放成配置，避免 retrieval 过早演化成配置系统。
// 2026-04-09 CST: 目的：让来源偏好保持透明、稳定、可回归验证，同时继续把文本分数保留为第一优先级。
fn source_ref_priority(source_ref: &str) -> usize {
    let normalized_source_ref = normalized_text(source_ref);

    if contains_any_keyword(
        &normalized_source_ref,
        &["plan", "forecast", "scenario"],
    ) {
        2
    } else if contains_any_keyword(
        &normalized_source_ref,
        &["summary", "trend", "report", "analysis", "derived"],
    ) {
        1
    } else {
        0
    }
}

// 2026-04-09 CST: 这里补弱 source_ref 判定 helper，原因是排序层里的 source priority 只关心层级，
// 2026-04-09 CST: 不能表达“来源名本身有没有语义”；目的：用最小占位词规则先拦住明显低区分度来源。
fn is_weak_source_ref(source_ref: &str) -> bool {
    let normalized_source_ref = normalized_text(source_ref);
    if normalized_source_ref.is_empty() {
        return true;
    }

    let tokens = normalized_source_ref.split_whitespace().collect::<Vec<_>>();
    // 2026-04-09 CST: 这里扩弱 source_ref 的最小启发式，原因是本轮红灯样本已经证明仅拦单 token 占位词不够，
    // 2026-04-09 CST: `source data`、`table 1`、`sheet1` 这类全由占位词和编号组成的来源名仍然几乎没有区分度；目的：只补足最小诊断边界，不引入更重的来源分类系统。
    if tokens.len() == 1 && is_weak_source_ref_token(tokens[0]) {
        return true;
    }

    // 2026-04-09 CST: 这里把“全由弱 token 构成的多 token 来源名”纳入 weak source_ref，原因是这些名字虽然更长，
    // 2026-04-09 CST: 但仍然只是默认占位命名的堆叠；目的：拦住低区分度来源，同时保留含真实业务词的来源名不被误伤。
    if tokens.iter().any(|token| !token.chars().all(|character| character.is_ascii_digit()))
        && tokens.iter().all(|token| {
            is_weak_source_ref_token(token)
                || token.chars().all(|character| character.is_ascii_digit())
        })
    {
        return true;
    }

    false
}

// 2026-04-09 CST: 这里补弱 source_ref token helper，原因是单 token 和多 token 弱来源规则都需要复用同一组占位词判断；
// 2026-04-09 CST: 目的：避免把弱来源词表散落在多个 if 分支中，降低后续扩词表时的维护成本。
// 2026-04-09 CST: 追加修改，原因是 `sheet1` 这类紧凑默认命名和 `sheet 1` 本质相同，但此前会漏过弱来源诊断；
// 2026-04-09 CST: 目的：把“占位词 + 紧凑数字后缀”收口到同一个 helper，继续保持规则局部且可测试。
fn is_weak_source_ref_token(token: &str) -> bool {
    matches!(token, "sheet" | "data" | "table" | "source" | "file")
        || matches_placeholder_token_with_numeric_suffix(token)
}

// 2026-04-09 CST: 这里补紧凑编号占位 token 判断 helper，原因是现实中的默认来源名常写成 `sheet1`、`table01`，
// 2026-04-09 CST: 目的：只识别“占位词前缀 + 纯数字后缀”这一种最小新增形态，避免粗暴前缀匹配误伤有业务语义的来源名。
fn matches_placeholder_token_with_numeric_suffix(token: &str) -> bool {
    ["sheet", "data", "table", "source", "file"]
        .iter()
        .any(|placeholder| {
            token.strip_prefix(placeholder).is_some_and(|suffix| {
                !suffix.is_empty() && suffix.chars().all(|character| character.is_ascii_digit())
            })
        })
}

// 2026-04-09 CST: 这里补来源关键词匹配辅助函数，原因是 source_ref 层级判断只需要最小的子串包含能力，
// 2026-04-09 CST: 没必要为了这轮 tie-break 引入更复杂的分类器或额外依赖。
// 2026-04-09 CST: 目的：复用统一归一化后的 source_ref 文本，降低来源分类规则的重复代码。
fn contains_any_keyword(normalized_text: &str, keywords: &[&str]) -> bool {
    keywords
        .iter()
        .any(|keyword| normalized_text.contains(keyword))
}

// 2026-04-08 CST: 这里从候选域推导 seed concept，原因是当前设计不希望为了 Task 11
// 2026-04-08 CST: 直接修改 retrieve 签名或把 route 结果强塞进 retrieval 层。
// 2026-04-08 CST: 目的：在不改动 pipeline 数据流的前提下，最小表达“种子概念优先于漫游补全概念”的排序倾向。
fn seed_concept_ids(scope: &CandidateScope) -> BTreeSet<String> {
    let roamed_concept_ids = scope
        .path
        .iter()
        .map(|step| step.to_concept_id.clone())
        .collect::<BTreeSet<_>>();
    let mut seed_concept_ids = scope
        .concept_ids
        .iter()
        .filter(|concept_id| !roamed_concept_ids.contains(*concept_id))
        .cloned()
        .collect::<BTreeSet<_>>();

    // 2026-04-08 CST: 这里保留 path 为空时的回退规则，原因是没有漫游路径时无法再区分 seed 与扩展概念。
    // 2026-04-08 CST: 目的：在零漫游场景下仍保持检索可用，不让“推导不到 seed”变成额外失败源。
    if seed_concept_ids.is_empty() {
        seed_concept_ids.extend(scope.concept_ids.iter().cloned());
    }

    seed_concept_ids
}

// 2026-04-08 CST: 这里提取完整短语 bonus，原因是纯 token 交集无法表达“完整问题短语命中”的更强相关性。
// 2026-04-08 CST: 目的：让 phrase 命中在不引入复杂文本模型的前提下，先获得一个稳定、透明的排序优势。
fn phrase_bonus(normalized_question: &str, title: &str, body: &str) -> usize {
    if !normalized_question.contains(' ') {
        return 0;
    }

    let normalized_title = normalized_text(title);
    let normalized_body = normalized_text(body);
    if normalized_title.contains(normalized_question)
        || normalized_body.contains(normalized_question)
    {
        4
    } else {
        0
    }
}

// 2026-04-08 CST: 这里提取交集计数辅助函数，原因是 Task 11 之后标题与正文要分别计分，
// 2026-04-08 CST: 再继续把原本单一的 overlap 逻辑塞回一个函数里会让评分含义变得不清晰。
// 2026-04-08 CST: 目的：让 title/body 双通道计分保持可读，也让单测更容易定位具体分数来源。
// 2026-04-09 CST: 这里补命中 token 提取辅助函数，原因是 diagnostics 不仅要告诉调用方“命中了几个词”，
// 2026-04-09 CST: 还要告诉调用方“具体命中了哪些词”；目的：让 retrieval 解释能力可以直接服务排序排查和 AI 交接。
fn matched_tokens(question_tokens: &BTreeSet<String>, content_tokens: &BTreeSet<String>) -> Vec<String> {
    question_tokens
        .iter()
        .filter(|token| content_tokens.contains(*token))
        .cloned()
        .collect()
}

// 2026-04-07 CST: 这里提取最小分词函数，原因是当前评分只依赖大小写无关的关键词交集，
// 2026-04-07 CST: 不需要提前引入更重的文本分析策略，但仍要保证 title/body 与问题文本走同一套归一化规则。
// 2026-04-07 CST: 目的：让排序测试和无命中测试都依赖同一份可预测的文本切分逻辑，减少后续行为抖动。
fn tokenize(text: &str) -> BTreeSet<String> {
    normalized_text(text)
        .split_whitespace()
        .map(ToString::to_string)
        .collect()
}

// 2026-04-08 CST: 这里补统一文本归一化函数，原因是 Task 11 新增了短语命中检测，
// 2026-04-08 CST: 不能再只依赖 token 集合，否则 title/body 与问题文本无法共享一致的短语匹配空间。
// 2026-04-08 CST: 目的：把文本变成稳定的小写空格串，为 tokenize 与 phrase bonus 共用。
fn normalized_text(text: &str) -> String {
    text.chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
