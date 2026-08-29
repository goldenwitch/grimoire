use grimoire::{
    ElementKind, SchemaExpr, parse_schema_document, prototype_schemas, serialize_schema_document,
};

const SHAPES: &str = r#"
    grimoire-schema 1.0.0
    schema {
        namespace "https://github.com/goldenwitch/grimoire/extension/shapes";
        name shapes;
        version 1.0.0;
        allows { port };
        value product {
            layout: enumeration { scalar, vector, sequence, grid, volume },
            dimensions: sequence<alternative {
                literal: positive-integer,
                symbolic: address-reference
            }>
        };
    }
"#;

#[test]
fn parses_the_shapes_schema_document() {
    let schema = parse_schema_document(SHAPES).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        schema.namespace.as_str(),
        "https://github.com/goldenwitch/grimoire/extension/shapes"
    );
    assert_eq!(schema.name, "shapes");
    assert!(schema.allowed_elements.contains(&ElementKind::Port));
    let SchemaExpr::Product(fields) = schema.value else {
        panic!("shapes should be a product schema");
    };
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].name, "layout");
    assert_eq!(fields[1].name, "dimensions");
}

#[test]
fn parsed_schema_matches_the_registry_entry() {
    let parsed = parse_schema_document(SHAPES).unwrap_or_else(|error| panic!("{error}"));
    let registered = prototype_schemas()
        .unwrap_or_else(|error| panic!("{error}"))
        .into_iter()
        .find(|schema| schema.name == "shapes")
        .unwrap_or_else(|| panic!("missing registered shapes schema"));
    assert_eq!(parsed, registered);
}

#[test]
fn parses_nested_closed_schema_constructors() {
    let source = r#"
        grimoire-schema 1.0.0
        schema {
            namespace "https://github.com/goldenwitch/grimoire/extension/test";
            name test;
            version 1.0.0;
            allows { block, group };
            value alternative {
                enabled: finite-scalar,
                absent-name: presence<text>
            };
        }
    "#;
    let schema = parse_schema_document(source).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(schema.allowed_elements.len(), 2);
    assert!(matches!(schema.value, SchemaExpr::Alternative(_)));
}

#[test]
fn rejects_duplicate_schema_fields_and_arms() {
    let fields = r#"
        grimoire-schema 1.0.0 schema {
            namespace "https://example.org/test";
            name test;
            version 1.0.0;
            allows { block };
            value product { one: text, one: text };
        }
    "#;
    let field_error = parse_schema_document(fields).expect_err("duplicate field should fail");
    assert!(field_error.message.contains("duplicate product field"));

    let arms = r#"
        grimoire-schema 1.0.0 schema {
            namespace "https://example.org/test";
            name test;
            version 1.0.0;
            allows { block };
            value alternative { one: text, one: finite-scalar };
        }
    "#;
    let arm_error = parse_schema_document(arms).expect_err("duplicate arm should fail");
    assert!(arm_error.message.contains("duplicate alternative tag"));
}

#[test]
fn rejects_unknown_schema_expression() {
    let source = r#"
        grimoire-schema 1.0.0 schema {
            namespace "https://example.org/test";
            name test;
            version 1.0.0;
            allows { block };
            value open-ended;
        }
    "#;
    let error = parse_schema_document(source).expect_err("unknown constructor should fail");
    assert!(error.message.contains("unknown schema expression"));
}

#[test]
fn schema_serialization_round_trips() {
    let schema = parse_schema_document(SHAPES).unwrap_or_else(|error| panic!("{error}"));
    let serialized = serialize_schema_document(&schema).unwrap_or_else(|error| panic!("{error}"));
    let reparsed = parse_schema_document(&serialized).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(reparsed, schema);
}
