use grimoire::{Schema, parse_description, prototype_schemas, validate_description};

fn schemas() -> Vec<Schema> {
    prototype_schemas().unwrap_or_else(|error| panic!("{error}"))
}

fn errors(source: &str) -> Vec<String> {
    let description = parse_description(source).unwrap_or_else(|error| panic!("{error}"));
    validate_description(&description, &schemas())
        .expect_err("fixture should fail")
        .into_iter()
        .map(|error| error.to_string())
        .collect()
}

const VALID: &str = r#"
    grimoire 1.0.0
    description @d "address references" {
        core-spec 1.0.0;
        core {
            block @axis "Axis" {
                port @axis/frames extensions {
                    extension "https://github.com/goldenwitch/grimoire/extension/axes" axis schema axes @1.0.0 = {
                        name: "frames",
                        description: absent
                    };
                };
            }
            block @encoder "Encoder" {
                port @encoder/features extensions {
                    extension "https://github.com/goldenwitch/grimoire/extension/shapes" shape schema shapes @1.0.0 = {
                        layout: grid,
                        dimensions: [symbolic(ref(@axis/frames))]
                    };
                };
            }
        }
    }
"#;

#[test]
fn accepts_a_visible_address_reference_inside_a_schema_value() {
    let description = parse_description(VALID).unwrap_or_else(|error| panic!("{error}"));
    assert!(validate_description(&description, &schemas()).is_ok());
}

#[test]
fn reports_a_missing_schema_value_reference_with_c6() {
    let source = VALID.replace("ref(@axis/frames)", "ref(@axis/missing)");
    let failures = errors(&source);
    assert!(failures.iter().any(|failure| {
        failure.contains("C6") && failure.contains("@axis/missing") && failure.contains("value")
    }));
}

#[test]
fn reports_a_below_scope_schema_value_reference_with_c6() {
    let source = r#"
        grimoire 1.0.0
        description @d "scope" {
            core-spec 1.0.0;
            core { block @root "Root" { port @root/p; } }
            layer "axis" {
                inputs { core };
                consumes { projection-language 1.0.0; schemas { } }
                projection {
                    select {
                        block @axis "Axis" {
                            port @axis/frames extensions {
                                extension "https://github.com/goldenwitch/grimoire/extension/axes" axis schema axes @1.0.0 = {
                                    name: "frames",
                                    description: absent
                                };
                            };
                        }
                    }
                }
            }
            layer "consumer" {
                inputs { core };
                consumes { projection-language 1.0.0; schemas { } }
                projection {
                    select {
                        block @consumer "Consumer" {
                            port @consumer/features extensions {
                                extension "https://github.com/goldenwitch/grimoire/extension/shapes" shape schema shapes @1.0.0 = {
                                    layout: grid,
                                    dimensions: [symbolic(ref(@axis/frames))]
                                };
                            };
                        }
                    }
                }
            }
        }
    "#;
    let failures = errors(source);
    assert!(failures.iter().any(|failure| {
        failure.contains("C6") && failure.contains("@axis/frames") && failure.contains("consumer")
    }));
}

#[test]
fn duplicate_generated_addresses_report_c1_and_c4() {
    let source = r#"
        grimoire 1.0.0
        description @d "duplicate generated addresses" {
            core-spec 1.0.0;
            core {}
            layer "one" {
                inputs { core };
                consumes { projection-language 1.0.0; schemas { } }
                projection {
                    select {
                        block @first "First" { port @shared/p; }
                    }
                }
            }
            layer "two" {
                inputs { core };
                consumes { projection-language 1.0.0; schemas { } }
                projection {
                    select {
                        block @second "Second" { port @shared/p; }
                    }
                }
            }
        }
    "#;
    let failures = errors(source);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("C1") && failure.contains("@shared/p"))
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("C4") && failure.contains("@shared/p"))
    );
}
