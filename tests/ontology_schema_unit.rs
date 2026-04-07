use excel_skill::ops::foundation;

#[test]
fn foundation_navigation_modules_are_exported() {
    let _ = std::any::type_name::<foundation::ontology_schema::OntologySchema>();
}
