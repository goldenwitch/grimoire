use grimoire::{
    Address, Element, ExtensionValue, Schema, Value, evaluate_layer, parse_description,
    prototype_schemas, validate_description,
};

const INDEXED_VISIBILITY: &str = r#"
    grimoire 1.0.0
    description @d "indexed visibility" {
        core-spec 1.0.0;
        core {
            block @token/visual/0 "Visual token at time zero" { port @token/visual/0/out; }
            block @token/action/0 "Action token at time zero" { port @token/action/0/out; }
            block @token/visual/1 "Visual token at time one" { port @token/visual/1/out; }
            block @token/action/1 "Action token at time one" { port @token/action/1/out; }
            block @attention/visual/0 "Visual attention at time zero" { port @attention/visual/0/in; }
            block @attention/action/0 "Action attention at time zero" { port @attention/action/0/in; }
            block @attention/visual/1 "Visual attention at time one" { port @attention/visual/1/in; }
            block @attention/action/1 "Action attention at time one" { port @attention/action/1/in; }

            connection @visibility/block-causal/v0-v0 @token/visual/0/out -> @attention/visual/0/in;
            connection @visibility/block-causal/a0-v0 @token/action/0/out -> @attention/visual/0/in;
            connection @visibility/block-causal/v0-a0 @token/visual/0/out -> @attention/action/0/in;
            connection @visibility/block-causal/a0-a0 @token/action/0/out -> @attention/action/0/in;
            connection @visibility/block-causal/v0-v1 @token/visual/0/out -> @attention/visual/1/in;
            connection @visibility/block-causal/a0-v1 @token/action/0/out -> @attention/visual/1/in;
            connection @visibility/block-causal/v1-v1 @token/visual/1/out -> @attention/visual/1/in;
            connection @visibility/block-causal/a1-v1 @token/action/1/out -> @attention/visual/1/in;
            connection @visibility/block-causal/v0-a1 @token/visual/0/out -> @attention/action/1/in;
            connection @visibility/block-causal/a0-a1 @token/action/0/out -> @attention/action/1/in;
            connection @visibility/block-causal/v1-a1 @token/visual/1/out -> @attention/action/1/in;
            connection @visibility/block-causal/a1-a1 @token/action/1/out -> @attention/action/1/in;
            group @visibility/block-causal "block causal visibility" {
                @visibility/block-causal/v0-v0,
                @visibility/block-causal/a0-v0,
                @visibility/block-causal/v0-a0,
                @visibility/block-causal/a0-a0,
                @visibility/block-causal/v0-v1,
                @visibility/block-causal/a0-v1,
                @visibility/block-causal/v1-v1,
                @visibility/block-causal/a1-v1,
                @visibility/block-causal/v0-a1,
                @visibility/block-causal/a0-a1,
                @visibility/block-causal/v1-a1,
                @visibility/block-causal/a1-a1;
                extensions {
                    extension "https://github.com/goldenwitch/grimoire/extension/architecture" architecture schema architecture @1.0.0 = {
                        family: "attention-visibility",
                        parameter_count: absent,
                        width: absent,
                        depth: absent,
                        head_count: absent,
                        mlp_width: absent,
                        activation: absent,
                        position_encoding: absent,
                        attention_regime: present(block-causal),
                        operator: absent,
                        interface: absent
                    };
                }
            }

            connection @visibility/mixed/v0-v0 @token/visual/0/out -> @attention/visual/0/in;
            connection @visibility/mixed/a0-v0 @token/action/0/out -> @attention/visual/0/in;
            connection @visibility/mixed/v1-v0 @token/visual/1/out -> @attention/visual/0/in;
            connection @visibility/mixed/a1-v0 @token/action/1/out -> @attention/visual/0/in;
            connection @visibility/mixed/v0-a0 @token/visual/0/out -> @attention/action/0/in;
            connection @visibility/mixed/a0-a0 @token/action/0/out -> @attention/action/0/in;
            connection @visibility/mixed/v1-a0 @token/visual/1/out -> @attention/action/0/in;
            connection @visibility/mixed/a1-a0 @token/action/1/out -> @attention/action/0/in;
            connection @visibility/mixed/v0-v1 @token/visual/0/out -> @attention/visual/1/in;
            connection @visibility/mixed/a0-v1 @token/action/0/out -> @attention/visual/1/in;
            connection @visibility/mixed/v1-v1 @token/visual/1/out -> @attention/visual/1/in;
            connection @visibility/mixed/a1-v1 @token/action/1/out -> @attention/visual/1/in;
            connection @visibility/mixed/v0-a1 @token/visual/0/out -> @attention/action/1/in;
            connection @visibility/mixed/a0-a1 @token/action/0/out -> @attention/action/1/in;
            connection @visibility/mixed/v1-a1 @token/visual/1/out -> @attention/action/1/in;
            connection @visibility/mixed/a1-a1 @token/action/1/out -> @attention/action/1/in;
            group @visibility/mixed "mixed attention visibility" {
                @visibility/mixed/v0-v0,
                @visibility/mixed/a0-v0,
                @visibility/mixed/v1-v0,
                @visibility/mixed/a1-v0,
                @visibility/mixed/v0-a0,
                @visibility/mixed/a0-a0,
                @visibility/mixed/v1-a0,
                @visibility/mixed/a1-a0,
                @visibility/mixed/v0-v1,
                @visibility/mixed/a0-v1,
                @visibility/mixed/v1-v1,
                @visibility/mixed/a1-v1,
                @visibility/mixed/v0-a1,
                @visibility/mixed/a0-a1,
                @visibility/mixed/v1-a1,
                @visibility/mixed/a1-a1;
                extensions {
                    extension "https://github.com/goldenwitch/grimoire/extension/architecture" architecture schema architecture @1.0.0 = {
                        family: "attention-visibility",
                        parameter_count: absent,
                        width: absent,
                        depth: absent,
                        head_count: absent,
                        mlp_width: absent,
                        activation: absent,
                        position_encoding: absent,
                        attention_regime: present(mixed),
                        operator: absent,
                        interface: absent
                    };
                }
            }
        }
        layer "block-causal" {
            inputs { core };
            consumes { projection-language 1.0.0; schemas { } }
            projection {
                select {
                    use @token/visual/0, @token/action/0, @token/visual/1, @token/action/1,
                       @attention/visual/0, @attention/action/0, @attention/visual/1, @attention/action/1,
                       @token/visual/0/out, @token/action/0/out, @token/visual/1/out, @token/action/1/out,
                       @attention/visual/0/in, @attention/action/0/in, @attention/visual/1/in, @attention/action/1/in,
                       @visibility/block-causal, @visibility/block-causal/v0-v0, @visibility/block-causal/a0-v0,
                       @visibility/block-causal/v0-a0, @visibility/block-causal/a0-a0, @visibility/block-causal/v0-v1,
                       @visibility/block-causal/a0-v1, @visibility/block-causal/v1-v1, @visibility/block-causal/a1-v1,
                       @visibility/block-causal/v0-a1, @visibility/block-causal/a0-a1, @visibility/block-causal/v1-a1,
                       @visibility/block-causal/a1-a1;
                }
            }
        }
        layer "mixed" {
            inputs { core };
            consumes { projection-language 1.0.0; schemas { } }
            projection {
                select {
                    use @token/visual/0, @token/action/0, @token/visual/1, @token/action/1,
                       @attention/visual/0, @attention/action/0, @attention/visual/1, @attention/action/1,
                       @token/visual/0/out, @token/action/0/out, @token/visual/1/out, @token/action/1/out,
                       @attention/visual/0/in, @attention/action/0/in, @attention/visual/1/in, @attention/action/1/in,
                       @visibility/mixed, @visibility/mixed/v0-v0, @visibility/mixed/a0-v0,
                       @visibility/mixed/v1-v0, @visibility/mixed/a1-v0, @visibility/mixed/v0-a0,
                       @visibility/mixed/a0-a0, @visibility/mixed/v1-a0, @visibility/mixed/a1-a0,
                       @visibility/mixed/v0-v1, @visibility/mixed/a0-v1, @visibility/mixed/v1-v1,
                       @visibility/mixed/a1-v1, @visibility/mixed/v0-a1, @visibility/mixed/a0-a1,
                       @visibility/mixed/v1-a1, @visibility/mixed/a1-a1;
                }
            }
        }
    }
"#;

fn schemas() -> Vec<Schema> {
    prototype_schemas().unwrap_or_else(|error| panic!("{error}"))
}

fn address(value: &str) -> Address {
    Address::parse(value).unwrap_or_else(|error| panic!("{error}"))
}

#[test]
fn expanded_visibility_fixture_validates() {
    let description =
        parse_description(INDEXED_VISIBILITY).unwrap_or_else(|error| panic!("{error}"));
    validate_description(&description, &schemas())
        .unwrap_or_else(|errors| panic!("validation errors: {errors:?}"));
    assert_eq!(description.core.connections.len(), 28);
}

#[test]
fn block_causal_and_mixed_layers_have_distinct_connection_sets() {
    let description =
        parse_description(INDEXED_VISIBILITY).unwrap_or_else(|error| panic!("{error}"));
    let block_causal =
        evaluate_layer(&description, "block-causal").unwrap_or_else(|error| panic!("{error}"));
    let mixed = evaluate_layer(&description, "mixed").unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(block_causal.structural.elements.len(), 29);
    assert_eq!(mixed.structural.elements.len(), 33);
    assert!(matches!(
        block_causal.structural.elements[&address("@visibility/block-causal/v0-v0")],
        Element::Connection(_)
    ));
    assert!(
        !block_causal
            .structural
            .elements
            .contains_key(&address("@visibility/mixed/v1-v0"))
    );
    assert!(matches!(
        mixed.structural.elements[&address("@visibility/mixed/v1-v0")],
        Element::Connection(_)
    ));
    assert!(
        !mixed
            .structural
            .elements
            .contains_key(&address("@visibility/block-causal/v1-v1"))
    );
}

#[test]
fn attention_regime_metadata_is_separate_from_visibility_structure() {
    let description =
        parse_description(INDEXED_VISIBILITY).unwrap_or_else(|error| panic!("{error}"));
    let block_causal = &description.core.groups[&address("@visibility/block-causal")];
    let mixed = &description.core.groups[&address("@visibility/mixed")];
    let ExtensionValue::Known(Value::Product(block_fields)) = &block_causal.extensions[0].value
    else {
        panic!("expected block-causal architecture metadata");
    };
    let ExtensionValue::Known(Value::Product(mixed_fields)) = &mixed.extensions[0].value else {
        panic!("expected mixed architecture metadata");
    };
    assert!(matches!(
        block_fields.get("attention_regime"),
        Some(Value::Present(value)) if matches!(value.as_ref(), Value::Enum(regime) if regime == "block-causal")
    ));
    assert!(matches!(
        mixed_fields.get("attention_regime"),
        Some(Value::Present(value)) if matches!(value.as_ref(), Value::Enum(regime) if regime == "mixed")
    ));
}
