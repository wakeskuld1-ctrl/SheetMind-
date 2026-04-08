use excel_skill::ops::foundation;
use excel_skill::ops::foundation::ontology_schema::{OntologyConcept, OntologySchema};

// 2026-04-08 CST: 这里先补 foundation 导航内核的首条失败测试，原因是方案 C 第一阶段需要先把
// `src/ops/foundation.rs` 变成稳定的知识导航子模块入口，而不是直接跳到实现细节。
// 目的：先用最小导出测试钉死 `ontology_schema` 模块对外可见，再按 TDD 逐层补数据结构与行为。
#[test]
fn foundation_navigation_modules_are_exported() {
    let _ = std::any::type_name::<foundation::ontology_schema::OntologySchema>();
}

// 2026-04-08 CST: 这里补 concept 名称与 alias 建索引测试，原因是导航内核的第一条真实语义能力
// 就是把问题文本稳定映射到 concept id，而不是直接跳到检索阶段。
// 目的：先钉死 schema 对 name/alias 的最小查找契约，后续 router 才能在这个基础上继续实现。
#[test]
fn ontology_schema_indexes_concepts_and_aliases() {
    let schema = OntologySchema::new(
        vec![OntologyConcept::new("revenue", "Revenue").with_alias("sales")],
        vec![],
    )
    .expect("schema should be valid");

    assert_eq!(schema.find_concept_id("Revenue"), Some("revenue"));
    assert_eq!(schema.find_concept_id("sales"), Some("revenue"));
}
