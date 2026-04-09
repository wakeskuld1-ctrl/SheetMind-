use excel_skill::ops::foundation;
use excel_skill::ops::foundation::ontology_schema::{OntologyConcept, OntologySchema};

#[test]
fn foundation_navigation_modules_are_exported() {
    let _ = std::any::type_name::<foundation::ontology_schema::OntologySchema>();
}

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

// 2026-04-09 CST: 这里补重复 lookup key 的多候选测试，原因是 foundation 路由准备引入最小标签约束后，
// 同一短语命中多个概念将不再是非法状态，而应交给上层路由结合 scope 决策。
// 目的：先把 schema 层的多候选收口能力钉死，避免后续还停留在“lookup key 必须全局唯一”的限制。
#[test]
fn ontology_schema_keeps_multiple_candidates_for_same_lookup_key() {
    let schema = OntologySchema::new(
        vec![
            OntologyConcept::new("gross_margin", "MarginSpread")
                .with_alias("margin")
                .with_tag("finance"),
            OntologyConcept::new("layout_margin", "LayoutMargin")
                .with_alias("margin")
                .with_tag("ui"),
        ],
        vec![],
    )
    .expect("schema should allow duplicate lookup keys across concepts");

    assert_eq!(schema.find_concept_id("margin"), Some("gross_margin"));
    assert_eq!(
        schema.find_concept_ids("margin"),
        vec!["gross_margin", "layout_margin"]
    );
}
