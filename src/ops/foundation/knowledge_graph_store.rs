use std::collections::BTreeMap;

use crate::ops::foundation::knowledge_record::{KnowledgeEdge, KnowledgeNode};

// 2026-04-08 CST: 这里把 knowledge graph store 设计成纯内存只读查询层，原因是第一阶段目标
// 是先打通知识图谱查询入口，而不是立刻引入持久化知识库或业务仓储。
// 目的：统一封装 node、concept 聚合和出边读取，为 retrieval 与后续装配层提供干净依赖面。
#[derive(Debug, Clone)]
pub struct KnowledgeGraphStore {
    nodes: Vec<KnowledgeNode>,
    edges: Vec<KnowledgeEdge>,
    node_index: BTreeMap<String, usize>,
    concept_index: BTreeMap<String, Vec<usize>>,
    outgoing_edge_index: BTreeMap<String, Vec<usize>>,
}

impl KnowledgeGraphStore {
    // 2026-04-08 CST: 这里构建最小 graph store，原因是 record 层只负责定义数据结构，
    // 真正的按 node id、按 concept id 和按出边查询应该统一在 store 层建立。
    // 目的：把模型定义与读取逻辑拆开，避免职责继续混在一起。
    pub fn new(nodes: Vec<KnowledgeNode>, edges: Vec<KnowledgeEdge>) -> Self {
        let mut node_index = BTreeMap::new();
        let mut concept_index: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        let mut outgoing_edge_index: BTreeMap<String, Vec<usize>> = BTreeMap::new();

        for (position, node) in nodes.iter().enumerate() {
            node_index.insert(node.id.clone(), position);
            for concept_id in &node.concept_ids {
                concept_index
                    .entry(concept_id.clone())
                    .or_default()
                    .push(position);
            }
        }

        for (position, edge) in edges.iter().enumerate() {
            outgoing_edge_index
                .entry(edge.from_node_id.clone())
                .or_default()
                .push(position);
        }

        Self {
            nodes,
            edges,
            node_index,
            concept_index,
            outgoing_edge_index,
        }
    }

    // 2026-04-08 CST: 这里按 node id 读取节点，原因是 retrieval 和 evidence assembly
    // 都需要安全读取节点正文与证据，但不该知道内部向量位置。
    // 目的：为上层提供稳定只读接口，隐藏具体存储实现。
    pub fn node(&self, node_id: &str) -> Option<&KnowledgeNode> {
        self.node_index
            .get(node_id)
            .and_then(|index| self.nodes.get(*index))
    }

    // 2026-04-08 CST: 这里按 concept ids 聚合节点，原因是 retrieval 阶段的候选 node 域
    // 需要由 roaming 输出的候选 concept 域映射而来。
    // 目的：先提供去重且稳定顺序的最小 concept -> node 查询契约。
    pub fn node_ids_for_concepts<'a>(&'a self, concept_ids: &[&str]) -> Vec<&'a str> {
        let mut matched_positions = Vec::new();

        for concept_id in concept_ids {
            if let Some(positions) = self.concept_index.get(*concept_id) {
                for position in positions {
                    if !matched_positions.contains(position) {
                        matched_positions.push(*position);
                    }
                }
            }
        }

        matched_positions.sort_unstable();

        matched_positions
            .into_iter()
            .filter_map(|position| self.nodes.get(position))
            .map(|node| node.id.as_str())
            .collect()
    }

    // 2026-04-08 CST: 这里读取节点全部出边，原因是后续证据路径保留和更复杂路径解释
    // 都需要从图谱 store 读取节点关系，而不应直接遍历原始边集合。
    // 目的：先固定 “node -> outgoing edges” 的读取契约。
    pub fn outgoing_edges<'a>(&'a self, node_id: &str) -> Vec<&'a KnowledgeEdge> {
        self.outgoing_edge_index
            .get(node_id)
            .into_iter()
            .flat_map(|positions| positions.iter())
            .filter_map(|position| self.edges.get(*position))
            .collect()
    }
}
