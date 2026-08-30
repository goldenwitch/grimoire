use std::collections::{BTreeMap, BTreeSet};

use grimoire::{
    Address, Block, Description, ElementKind, Namespace, Schema, SchemaExpr, SchemaExprArm,
    SchemaExprField, Value, Version,
};

mod common;
use common::address;

fn namespace(value: &str) -> Namespace {
    Namespace::parse(value).unwrap_or_else(|error| panic!("{error}"))
}

#[test]
fn addresses_are_exact_flat_identifiers() {
    let parsed = address("@system/vision-encoder");
    assert_eq!(parsed.as_str(), "@system/vision-encoder");
    assert!(Address::parse("system/vision-encoder").is_err());
    assert!(Address::parse("@system//encoder").is_err());
    assert!(Address::parse("@system/vision encoder").is_err());
}

#[test]
fn namespaces_are_exact_https_identifiers() {
    let parsed = namespace("https://github.com/goldenwitch/grimoire/extension/shapes");
    assert_eq!(
        parsed.as_str(),
        "https://github.com/goldenwitch/grimoire/extension/shapes"
    );
    assert!(Namespace::parse("http://example.org/extension").is_err());
    assert!(Namespace::parse("shapes").is_err());
    assert!(Namespace::parse("https:// example.org/extension").is_err());
}

#[test]
fn finite_scalar_is_provisionally_boolean() {
    let schema = SchemaExpr::FiniteScalar;
    assert!(schema.validate(&Value::Bool(true)).is_ok());
    assert!(schema.validate(&Value::Bool(false)).is_ok());
    assert!(schema.validate(&Value::Text("true".to_owned())).is_err());
}

#[test]
fn closed_products_alternatives_presence_and_references_validate() {
    let mut fields = BTreeMap::new();
    fields.insert("name".to_owned(), Value::Text("frames".to_owned()));
    fields.insert(
        "description".to_owned(),
        Value::Present(Box::new(Value::Text("video time".to_owned()))),
    );
    let product = SchemaExpr::Product(vec![
        SchemaExprField {
            name: "name".to_owned(),
            schema: Box::new(SchemaExpr::Text),
        },
        SchemaExprField {
            name: "description".to_owned(),
            schema: Box::new(SchemaExpr::Presence(Box::new(SchemaExpr::Text))),
        },
    ]);
    assert!(product.validate(&Value::Product(fields)).is_ok());

    let alternative = SchemaExpr::Alternative(vec![SchemaExprArm {
        tag: "integer".to_owned(),
        schema: Box::new(SchemaExpr::PositiveInteger),
    }]);
    assert!(
        alternative
            .validate(&Value::Tagged {
                tag: "integer".to_owned(),
                value: Box::new(Value::PositiveInteger(7)),
            })
            .is_ok()
    );
    assert!(alternative.validate(&Value::Bool(true)).is_err());

    let reference = SchemaExpr::AddressReference;
    assert!(
        reference
            .validate(&Value::AddressReference(address("@system/encoder")))
            .is_ok()
    );
}

#[test]
fn description_collects_core_and_nested_port_addresses() {
    let encoder = address("@system/vision-encoder");
    let input = address("@system/vision-encoder/input");
    let description = Description {
        address: address("@system"),
        label: Some("system".to_owned()),
        core_spec: Version::new(1, 0, 0),
        extensions: Vec::new(),
        layers: Vec::new(),
        core: grimoire::CoreGraph {
            blocks: BTreeMap::from([(
                encoder.clone(),
                Block {
                    address: encoder.clone(),
                    name: "Vision encoder".to_owned(),
                    ports: BTreeMap::from([(
                        input.clone(),
                        grimoire::Port {
                            address: input.clone(),
                            label: Some("input".to_owned()),
                            extensions: Vec::new(),
                        },
                    )]),
                    extensions: Vec::new(),
                },
            )]),
            ..grimoire::CoreGraph::default()
        },
    };
    let addresses = description.addresses();
    assert!(addresses.contains(&&address("@system")));
    assert!(addresses.contains(&&encoder));
    assert!(addresses.contains(&&input));
}

#[test]
fn schema_metadata_keeps_namespace_version_and_attachment_kinds() {
    let schema = Schema {
        namespace: namespace("https://github.com/goldenwitch/grimoire/extension/shapes"),
        name: "shapes".to_owned(),
        version: Version::new(1, 0, 0),
        allowed_elements: BTreeSet::from([ElementKind::Port]),
        value: SchemaExpr::Text,
    };
    assert_eq!(schema.version, Version::new(1, 0, 0));
    assert!(schema.allowed_elements.contains(&ElementKind::Port));
}
