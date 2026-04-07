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
