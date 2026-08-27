use grimoire::{
    Address, Block, Check, Connection, CoreGraph, Decoration, Description, Element,
    ExpectedCardinality, ExtensionParameter, ExtensionValue, FiniteNumber, Group, Layer,
    LayerInput, Namespace, Port, Projection, SelectItem, Value, Version, evaluate_layer,
};

fn address(value: &str) -> Address {
    Address::parse(value).unwrap_or_else(|error| panic!("{error}"))
}

fn namespace(value: &str) -> Namespace {
    Namespace::parse(value).unwrap_or_else(|error| panic!("{error}"))
}

fn base_description(layers: Vec<Layer>) -> Description {
    Description {
        address: address("@d"),
        label: Some("evaluation".to_owned()),
        core_spec: Version::new(1, 0, 0),
        core: CoreGraph {
            blocks: std::collections::BTreeMap::from([
                (
                    address("@source"),
                    Block {
                        address: address("@source"),
                        name: "Source".to_owned(),
                        ports: std::collections::BTreeMap::from([(
                            address("@source/out"),
                            Port {
                                address: address("@source/out"),
                                label: None,
                                extensions: Vec::new(),
                            },
                        )]),
                        extensions: Vec::new(),
                    },
                ),
                (
                    address("@target"),
                    Block {
                        address: address("@target"),
                        name: "Target".to_owned(),
                        ports: std::collections::BTreeMap::from([(
                            address("@target/in"),
                            Port {
                                address: address("@target/in"),
                                label: None,
                                extensions: Vec::new(),
                            },
                        )]),
                        extensions: Vec::new(),
                    },
                ),
            ]),
            connections: std::collections::BTreeMap::from([(
                address("@flow"),
                Connection {
                    address: address("@flow"),
                    label: None,
                    source: address("@source/out"),
                    destination: address("@target/in"),
                    extensions: Vec::new(),
                },
            )]),
            groups: std::collections::BTreeMap::from([(
                address("@flow-group"),
                Group {
                    address: address("@flow-group"),
                    label: None,
                    members: vec![address("@flow")],
                    extensions: Vec::new(),
                },
            )]),
        },
        extensions: Vec::new(),
        layers,
    }
}

fn layer(name: &str, inputs: Vec<LayerInput>, projection: Projection) -> Layer {
    Layer {
        name: name.to_owned(),
        inputs,
        projection_language: Version::new(1, 0, 0),
        schemas: Vec::new(),
        projection,
    }
}

fn architecture_decoration(target: &str, family: &str) -> Decoration {
    Decoration {
        target: address(target),
        parameter: ExtensionParameter {
            namespace: namespace("https://github.com/goldenwitch/grimoire/extension/architecture"),
            name: "family".to_owned(),
            schema: "architecture".to_owned(),
            version: Version::new(1, 0, 0),
            value: ExtensionValue::Known(Value::Product(std::collections::BTreeMap::from([(
                "family".to_owned(),
                Value::Text(family.to_owned()),
            )]))),
        },
    }
}

#[test]
fn folds_shared_references_and_applies_group_inversion() {
    let description = base_description(vec![layer(
        "backward",
        vec![LayerInput::Core],
        Projection {
            select: vec![SelectItem::Use(vec![
                address("@source"),
                address("@target"),
                address("@source/out"),
                address("@target/in"),
                address("@flow"),
                address("@flow-group"),
            ])],
            invert: vec![address("@flow-group")],
            ..Projection::default()
        },
    )]);
    let result = evaluate_layer(&description, "backward").unwrap_or_else(|error| panic!("{error}"));
    let Element::Connection(connection) = &result.structural.elements[&address("@flow")] else {
        panic!("expected a connection");
    };
    assert_eq!(connection.source, address("@target/in"));
    assert_eq!(connection.destination, address("@source/out"));
}

#[test]
fn can_select_the_addressed_description_element() {
    let description = base_description(vec![layer(
        "description",
        vec![LayerInput::Core],
        Projection {
            select: vec![SelectItem::Use(vec![address("@d")])],
            ..Projection::default()
        },
    )]);
    let result =
        evaluate_layer(&description, "description").unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        result.structural.elements.get(&address("@d")),
        Some(&Element::Description(address("@d")))
    );
}

#[test]
fn selecting_a_group_stands_for_its_members_during_inversion() {
    let description = base_description(vec![layer(
        "group-target",
        vec![LayerInput::Core],
        Projection {
            select: vec![SelectItem::Use(vec![address("@flow-group")])],
            invert: vec![address("@flow-group")],
            ..Projection::default()
        },
    )]);
    let result =
        evaluate_layer(&description, "group-target").unwrap_or_else(|error| panic!("{error}"));
    let Element::Connection(connection) = &result.structural.elements[&address("@flow")] else {
        panic!("expected the group member connection");
    };
    assert_eq!(connection.source, address("@target/in"));
    assert_eq!(connection.destination, address("@source/out"));
}

#[test]
fn generated_definitions_are_structural_elements() {
    let generated = Block {
        address: address("@generated"),
        name: "Generated".to_owned(),
        ports: std::collections::BTreeMap::from([(
            address("@generated/in"),
            Port {
                address: address("@generated/in"),
                label: None,
                extensions: Vec::new(),
            },
        )]),
        extensions: Vec::new(),
    };
    let description = base_description(vec![layer(
        "generated",
        vec![LayerInput::Core],
        Projection {
            select: vec![SelectItem::GenerateBlock(generated)],
            ..Projection::default()
        },
    )]);
    let result =
        evaluate_layer(&description, "generated").unwrap_or_else(|error| panic!("{error}"));
    assert!(matches!(
        result.structural.elements[&address("@generated")],
        Element::Block(_)
    ));
    assert!(matches!(
        result.structural.elements[&address("@generated/in")],
        Element::Port(_)
    ));
}

#[test]
fn structural_equality_does_not_expose_definition_site() {
    let core_result = evaluate_layer(
        &base_description(vec![layer(
            "core",
            vec![LayerInput::Core],
            Projection {
                select: vec![SelectItem::Use(vec![
                    address("@source"),
                    address("@source/out"),
                ])],
                ..Projection::default()
            },
        )]),
        "core",
    )
    .unwrap_or_else(|error| panic!("{error}"));

    let mut layer_description = base_description(Vec::new());
    layer_description.core = CoreGraph::default();
    layer_description.layers.push(layer(
        "local",
        vec![LayerInput::Core],
        Projection {
            select: vec![SelectItem::GenerateBlock(Block {
                address: address("@source"),
                name: "Source".to_owned(),
                ports: std::collections::BTreeMap::from([(
                    address("@source/out"),
                    Port {
                        address: address("@source/out"),
                        label: None,
                        extensions: Vec::new(),
                    },
                )]),
                extensions: Vec::new(),
            })],
            ..Projection::default()
        },
    ));
    let layer_result =
        evaluate_layer(&layer_description, "local").unwrap_or_else(|error| panic!("{error}"));

    assert_eq!(core_result.structural, layer_result.structural);
}

#[test]
fn generated_connections_and_groups_are_structural_elements() {
    let generated_connection = Connection {
        address: address("@generated-flow"),
        label: None,
        source: address("@source/out"),
        destination: address("@target/in"),
        extensions: Vec::new(),
    };
    let generated_group = Group {
        address: address("@generated-group"),
        label: None,
        members: vec![address("@generated-flow")],
        extensions: Vec::new(),
    };
    let description = base_description(vec![layer(
        "generated-structure",
        vec![LayerInput::Core],
        Projection {
            select: vec![
                SelectItem::Use(vec![
                    address("@source"),
                    address("@target"),
                    address("@source/out"),
                    address("@target/in"),
                ]),
                SelectItem::GenerateConnection(generated_connection),
                SelectItem::GenerateGroup(generated_group),
            ],
            invert: vec![address("@generated-group")],
            ..Projection::default()
        },
    )]);
    let result = evaluate_layer(&description, "generated-structure")
        .unwrap_or_else(|error| panic!("{error}"));
    let Element::Connection(connection) = &result.structural.elements[&address("@generated-flow")]
    else {
        panic!("expected a generated connection");
    };
    assert_eq!(connection.source, address("@target/in"));
    assert_eq!(connection.destination, address("@source/out"));
    assert!(matches!(
        result.structural.elements[&address("@generated-group")],
        Element::Group(_)
    ));
}

#[test]
fn competing_definitions_fail_at_fold() {
    let generated = Block {
        address: address("@source"),
        name: "Competing".to_owned(),
        ports: std::collections::BTreeMap::new(),
        extensions: Vec::new(),
    };
    let description = base_description(vec![layer(
        "competing",
        vec![LayerInput::Core],
        Projection {
            select: vec![
                SelectItem::Use(vec![address("@source")]),
                SelectItem::GenerateBlock(generated),
            ],
            ..Projection::default()
        },
    )]);
    let error =
        evaluate_layer(&description, "competing").expect_err("competing definition should fail");
    assert_eq!(error.stage, "fold");
    assert!(error.message.contains("competing definitions"));
    assert_eq!(error.identifier.as_deref(), Some("@source"));
}

#[test]
fn decorations_and_checks_happen_after_structure() {
    let decoration = architecture_decoration("@source", "encoder");
    let description = base_description(vec![layer(
        "decorated",
        vec![LayerInput::Core],
        Projection {
            select: vec![SelectItem::Use(vec![address("@source")])],
            decorate: vec![decoration],
            checks: vec![Check {
                name: "has-family".to_owned(),
                expected: ExpectedCardinality::Nonempty,
                namespace: namespace(
                    "https://github.com/goldenwitch/grimoire/extension/architecture",
                ),
                parameter: "family".to_owned(),
            }],
            ..Projection::default()
        },
    )]);
    let result =
        evaluate_layer(&description, "decorated").unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(result.structural.elements.len(), 1);
    assert_eq!(result.decorations.len(), 1);
    assert_eq!(result.checks.len(), 1);
    assert!(result.checks[0].passed);
}

#[test]
fn empty_and_nonempty_checks_are_visible_without_mutating_structure() {
    let description = base_description(vec![layer(
        "checks",
        vec![LayerInput::Core],
        Projection {
            select: vec![SelectItem::Use(vec![address("@target")])],
            checks: vec![
                Check {
                    name: "nothing".to_owned(),
                    expected: ExpectedCardinality::Empty,
                    namespace: namespace(
                        "https://github.com/goldenwitch/grimoire/extension/architecture",
                    ),
                    parameter: "family".to_owned(),
                },
                Check {
                    name: "also-nothing".to_owned(),
                    expected: ExpectedCardinality::Nonempty,
                    namespace: namespace(
                        "https://github.com/goldenwitch/grimoire/extension/architecture",
                    ),
                    parameter: "family".to_owned(),
                },
            ],
            ..Projection::default()
        },
    )]);
    let result = evaluate_layer(&description, "checks").unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(result.structural.elements.len(), 1);
    assert!(result.checks[0].passed);
    assert!(!result.checks[1].passed);
    assert!(result.decorations.is_empty());
}

#[test]
fn chained_layers_compose_by_address() {
    let description = base_description(vec![
        layer(
            "first",
            vec![LayerInput::Core],
            Projection {
                select: vec![SelectItem::Use(vec![address("@source")])],
                ..Projection::default()
            },
        ),
        layer(
            "second",
            vec![LayerInput::Core, LayerInput::Layer("first".to_owned())],
            Projection {
                select: vec![SelectItem::Use(vec![
                    address("@source"),
                    address("@target"),
                ])],
                ..Projection::default()
            },
        ),
    ]);
    let result = evaluate_layer(&description, "second").unwrap_or_else(|error| panic!("{error}"));
    assert!(result.structural.elements.contains_key(&address("@source")));
    assert!(result.structural.elements.contains_key(&address("@target")));
}

#[test]
fn chained_layers_preserve_upstream_finalized_decorations() {
    let description = base_description(vec![
        layer(
            "annotated",
            vec![LayerInput::Core],
            Projection {
                select: vec![SelectItem::Use(vec![address("@source")])],
                decorate: vec![architecture_decoration("@source", "encoder")],
                ..Projection::default()
            },
        ),
        layer(
            "consumer",
            vec![LayerInput::Core, LayerInput::Layer("annotated".to_owned())],
            Projection {
                select: vec![SelectItem::Use(vec![address("@source")])],
                checks: vec![Check {
                    name: "inherited-family".to_owned(),
                    expected: ExpectedCardinality::Nonempty,
                    namespace: namespace(
                        "https://github.com/goldenwitch/grimoire/extension/architecture",
                    ),
                    parameter: "family".to_owned(),
                }],
                ..Projection::default()
            },
        ),
    ]);
    let result = evaluate_layer(&description, "consumer").unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(result.decorations.len(), 1);
    assert_eq!(result.checks[0].observed, 1);
    assert!(result.checks[0].passed);
}

#[test]
fn errors_name_the_projection_stage_and_identifier() {
    let description = base_description(vec![layer(
        "missing",
        vec![LayerInput::Core],
        Projection {
            select: vec![SelectItem::Use(vec![address("@absent")])],
            ..Projection::default()
        },
    )]);
    let error = evaluate_layer(&description, "missing").expect_err("missing selection should fail");
    assert_eq!(error.stage, "select");
    assert_eq!(error.identifier.as_deref(), Some("@absent"));
}

#[test]
fn missing_decoration_target_fails_at_finalize() {
    let description = base_description(vec![layer(
        "missing-decoration",
        vec![LayerInput::Core],
        Projection {
            select: vec![SelectItem::Use(vec![address("@source")])],
            decorate: vec![architecture_decoration("@target", "target")],
            ..Projection::default()
        },
    )]);
    let error = evaluate_layer(&description, "missing-decoration")
        .expect_err("missing decoration target should fail");
    assert_eq!(error.stage, "decorate");
    assert_eq!(error.identifier.as_deref(), Some("@target"));
}

#[test]
fn finite_number_fixture_remains_available_for_future_costs() {
    let number = FiniteNumber::new(1.0).unwrap();
    assert_eq!(number.get(), 1.0);
}
