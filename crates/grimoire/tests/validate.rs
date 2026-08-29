use grimoire::{
    Address, Block, CoreGraph, Description, ElementKind, ExtensionParameter, ExtensionValue,
    Namespace, Port, Version, parse_description, validate_description,
};

mod common;
use common::schemas;

const VALID: &str = r#"
    grimoire 1.0.0
    description @d "valid" {
        core-spec 1.0.0;
        core {
            block @encoder "Encoder" {
                port @encoder/input;
                port @encoder/output;
            }
            block @consumer "Consumer" { port @consumer/input; }
            connection @flow @encoder/output -> @consumer/input;
            group @graph "graph" { @encoder, @consumer, @flow; }
        }
        layer "view" {
            inputs { core };
            consumes {
                projection-language 1.0.0;
                schemas { }
            }
            projection { select { use @encoder, @consumer, @flow; } }
        }
    }
"#;

fn parsed(source: &str) -> Description {
    parse_description(source).unwrap_or_else(|error| panic!("{error}"))
}

fn checks(source: &str) -> Vec<String> {
    validate_description(&parsed(source), &schemas())
        .expect_err("fixture should fail")
        .into_iter()
        .map(|error| error.to_string())
        .collect()
}

#[test]
fn accepts_core_and_layer_references_with_unreferenced_elements() {
    assert!(validate_description(&parsed(VALID), &schemas()).is_ok());
}

#[test]
fn reports_non_port_connection_endpoints_with_c2() {
    let source = r#"
        grimoire 1.0.0 description @d {
            core-spec 1.0.0;
            core {
                block @b "Block" { port @b/p; }
                connection @c @b -> @b/p;
            }
        }
    "#;
    let errors = checks(source);
    assert!(
        errors
            .iter()
            .any(|error| error.contains("C2") && error.contains("@b"))
    );
}

#[test]
fn reports_missing_layer_visibility_with_c6() {
    let source = r#"
        grimoire 1.0.0 description @d {
            core-spec 1.0.0;
            core {}
            layer "one" {
                inputs { core };
                consumes { projection-language 1.0.0; schemas { } }
                projection {
                    select {
                        use @missing;
                    }
                }
            }
        }
    "#;
    let errors = checks(source);
    assert!(
        errors
            .iter()
            .any(|error| error.contains("C6") && error.contains("@missing"))
    );
}

#[test]
fn reports_a_layer_definition_below_a_maximal_referencer_with_c7() {
    let source = r#"
        grimoire 1.0.0
        description @d "locality" {
            core-spec 1.0.0;
            core { block @root "Root" { port @root/p; } }
            layer "base" {
                inputs { core };
                consumes { projection-language 1.0.0; schemas { } }
                projection {
                    select {
                        block @local "Local" { port @local/p; }
                    }
                }
            }
            layer "consumer" {
                inputs { "base" };
                consumes { projection-language 1.0.0; schemas { } }
                projection { select { use @local; } }
            }
        }
    "#;
    let failures = checks(source);
    assert!(
        failures
            .iter()
            .any(|failure| { failure.contains("C7") && failure.contains("@local") })
    );
}

#[test]
fn accepts_a_layer_whose_declared_chain_reaches_core() {
    let source = r#"
        grimoire 1.0.0
        description @d "chained inputs" {
            core-spec 1.0.0;
            core {
                block @root "Root" { port @root/p; }
                group @root-group { @root; }
            }
            layer "base" {
                inputs { core };
                consumes { projection-language 1.0.0; schemas { } }
                projection { select { use @root; } }
            }
            layer "child" {
                inputs { "base" };
                consumes { projection-language 1.0.0; schemas { } }
                projection { select { use @root; } }
            }
        }
    "#;
    validate_description(&parsed(source), &schemas())
        .unwrap_or_else(|errors| panic!("validation errors: {errors:?}"));
}

#[test]
fn accepts_layer_names_as_grammar_strings() {
    let source = r#"
        grimoire 1.0.0
        description @d "layer names" {
            core-spec 1.0.0;
            core {
                block @root "Root" { port @root/p; }
                group @root-group { @root; }
            }
            layer "video language" {
                inputs { core };
                consumes { projection-language 1.0.0; schemas { } }
                projection { select { use @root; } }
            }
        }
    "#;
    validate_description(&parsed(source), &schemas())
        .unwrap_or_else(|errors| panic!("validation errors: {errors:?}"));
}

#[test]
fn generated_block_extension_references_use_the_actual_layer_name() {
    let source = r#"
        grimoire 1.0.0
        description @d "layer location" {
            core-spec 1.0.0;
            core {
                block @root "Root" { port @root/p; }
                group @root-group { @root, @root/p; }
            }
            layer "video/language" {
                inputs { core };
                consumes { projection-language 1.0.0; schemas { } }
                projection {
                    select {
                        block @consumer "Consumer" {
                            port @consumer/features extensions {
                                extension "https://github.com/goldenwitch/grimoire/extension/shapes" shape schema shapes @1.0.0 = {
                                    layout: sequence,
                                    dimensions: [symbolic(ref(@root/p))]
                                };
                            };
                        }
                    }
                }
            }
        }
    "#;
    validate_description(&parsed(source), &schemas())
        .unwrap_or_else(|errors| panic!("validation errors: {errors:?}"));
}

#[test]
fn reports_missing_and_cyclic_layer_inputs_with_c9() {
    let missing = r#"
        grimoire 1.0.0 description @d {
            core-spec 1.0.0;
            core {}
            layer "one" {
                inputs { core, "absent" };
                consumes { projection-language 1.0.0; schemas { } }
                projection { select { } }
            }
        }
    "#;
    let missing_errors = checks(missing);
    assert!(
        missing_errors
            .iter()
            .any(|error| error.contains("C9") && error.contains("absent"))
    );

    let cyclic = r#"
        grimoire 1.0.0 description @d {
            core-spec 1.0.0;
            core {}
            layer "one" {
                inputs { core, "two" };
                consumes { projection-language 1.0.0; schemas { } }
                projection { select { } }
            }
            layer "two" {
                inputs { core, "one" };
                consumes { projection-language 1.0.0; schemas { } }
                projection { select { } }
            }
        }
    "#;
    let cycle_errors = checks(cyclic);
    assert!(
        cycle_errors
            .iter()
            .any(|error| error.contains("C9") && error.contains("cycle"))
    );
}

#[test]
fn reports_known_schema_attachment_failure_with_c10() {
    let source = r#"
        grimoire 1.0.0 description @d {
            core-spec 1.0.0;
            extensions {
                extension "https://github.com/goldenwitch/grimoire/extension/architecture" family schema architecture @1.0.0 = { family: "encoder" };
            }
            core {}
        }
    "#;
    let errors = checks(source);
    assert!(
        errors
            .iter()
            .any(|error| error.contains("C10") && error.contains("family"))
    );
}

#[test]
fn accepts_unknown_namespace_as_opaque_data() {
    let source = r#"grimoire 1.0.0 description @d {
        core-spec 1.0.0;
        extensions {
            extension "https://other.example/opaque" payload schema unknown @1.0.0 = { value: "kept" };
        }
        core {}
    }"#;
    assert!(validate_description(&parsed(source), &schemas()).is_ok());
}

#[test]
fn error_records_name_check_location_and_identifier() {
    let source = r#"
        grimoire 1.0.0 description @d {
            core-spec 1.0.0;
            core {
                block @b "Block" { port @b/p; }
                connection @bad @b/p -> @missing;
            }
        }
    "#;
    let errors = checks(source);
    assert!(errors.iter().any(|error| {
        error.contains("C2")
            && error.contains("core/connection/@bad/destination")
            && error.contains("@missing")
    }));
}

#[test]
fn manually_constructed_empty_block_name_is_c5() {
    let block_address = Address::parse("@b").unwrap();
    let port_address = Address::parse("@b/p").unwrap();
    let description = Description {
        address: Address::parse("@d").unwrap(),
        label: None,
        core_spec: Version::new(1, 0, 0),
        core: CoreGraph {
            blocks: std::collections::BTreeMap::from([(
                block_address.clone(),
                Block {
                    address: block_address,
                    name: String::new(),
                    ports: std::collections::BTreeMap::from([(
                        port_address.clone(),
                        Port {
                            address: port_address,
                            label: None,
                            extensions: Vec::new(),
                        },
                    )]),
                    extensions: Vec::new(),
                },
            )]),
            ..CoreGraph::default()
        },
        extensions: Vec::<ExtensionParameter>::new(),
        layers: Vec::new(),
    };
    let errors =
        validate_description(&description, &schemas()).expect_err("empty name should fail");
    assert!(
        errors
            .iter()
            .any(|error| error.check == "C5" && error.identifier.as_deref() == Some("@b"))
    );
}

#[test]
fn schema_namespace_and_kind_are_available_to_validation() {
    let namespace =
        Namespace::parse("https://github.com/goldenwitch/grimoire/extension/axes").unwrap();
    let extension = ExtensionParameter {
        namespace,
        name: "axis".to_owned(),
        schema: "axes".to_owned(),
        version: Version::new(1, 0, 0),
        value: ExtensionValue::Known(grimoire::Value::Product(std::collections::BTreeMap::from(
            [
                (
                    "name".to_owned(),
                    grimoire::Value::Text("frames".to_owned()),
                ),
                ("description".to_owned(), grimoire::Value::Absent),
            ],
        ))),
    };
    let port = Port {
        address: Address::parse("@b/p").unwrap(),
        label: None,
        extensions: vec![extension],
    };
    let ExtensionValue::Known(value) = &port.extensions[0].value else {
        panic!("expected known value");
    };
    assert!(
        schemas()
            .iter()
            .find(|schema| schema.name == "axes")
            .unwrap()
            .validate(ElementKind::Port, value)
            .is_ok()
    );
}
