use std::collections::BTreeMap;

use crate::ops::foundation::knowledge_record::{KnowledgeEdge, KnowledgeNode};

// 2026-04-07 CST: 这里把 knowledge graph store 设计成纯内存只读查询层，原因是 Task 4 的目标
// 是先让 foundation 主链拥有稳定的图谱读取入口，而不是提前引入持久化或业务型仓储。
// 目的：把节点、概念映射和边读取统一收口到 store，给后续 roaming / retrieval 提供干净依赖面。
#[derive(Debug, Clone)]
pub struct KnowledgeGraphStore {
    nodes: Vec<KnowledgeNode>,
    edges: Vec<KnowledgeEdge>,
    node_index: BTreeMap<String, usize>,
    concept_index: BTreeMap<String, Vec<usize>>,
    outgoing_edge_index: BTreeMap<String, Vec<usize>>,
}

impl KnowledgeGraphStore {
    // 2026-04-07 CST: 这里新增 store 构造函数，原因是 record 层只负责定义数据结构，
    // 真正的节点索引、概念聚合和边查询入口应该统一在 store 层建立。
    // 目的：先把 Task 4 的“模型”和“查询”职责分开，避免后续图谱能力继续堆回 record 模块。
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

    // 2026-04-07 CST: 这里提供按 node id 读取节点详情的入口，原因是 retrieval 和 evidence
    // 组装都会消费节点正文与证据引用，不能直接窥探 store 的内部向量索引。
    // 目的：给上层提供稳定只读接口，同时把节点存储细节继续封装在 store 内部。
    pub fn node(&self, node_id: &str) -> Option<&KnowledgeNode> {
        self.node_index
            .get(node_id)
            .and_then(|index| self.nodes.get(*index))
    }

    // 2026-04-07 CST: 这里按 concept ids 聚合节点，原因是 roaming 输出的候选概念集
    // 下一步就需要映射到候选节点集，Task 4 必须先把这条基础路径建起来。
    // 目的：先以节点声明顺序返回去重后的 node ids，为后续 scoped retrieval 提供稳定输入。
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

    // 2026-04-07 CST: 这里提供按 node id 读取出边的最小能力，原因是后续 evidence 路径保留、
    // 节点关系追踪和更高层候选扩展都需要从 graph store 读取节点之间的连接。
    // 目的：先把“节点 -> 边集合”的读取契约固定下来，后续即使换索引策略也不影响上层接口。
    pub fn outgoing_edges<'a>(&'a self, node_id: &str) -> Vec<&'a KnowledgeEdge> {
        self.outgoing_edge_index
            .get(node_id)
            .into_iter()
            .flat_map(|positions| positions.iter())
            .filter_map(|position| self.edges.get(*position))
            .collect()
    }
}
