use grimoire::{
    ExpectedCardinality, ExtensionValue, LayerInput, SelectItem, Value, parse_description,
};

const LAYERED: &str = r#"
    grimoire 1.0.0
    description @system "layered system" {
        core-spec 1.0.0;
        core {
            block @encoder "Encoder" { port @encoder/output; }
            group @selected "selected" { @encoder; }
        }
        layer "pretraining" {
            inputs { core };
            consumes {
                projection-language 1.0.0;
                schemas {
                    "https://github.com/goldenwitch/grimoire/extension/shapes" / shapes @1.0.0;
                }
            }
            projection {
                select {
                    use @encoder;
                    block @predictor "Predictor" { port @predictor/input; }
                }
                invert { group @selected; }
                decorate {
                    on @encoder extension "https://github.com/goldenwitch/grimoire/extension/architecture" family schema architecture @1.0.0 = { family: "vision" };
                }
                checks {
                    check architecture-present expect nonempty over "https://github.com/goldenwitch/grimoire/extension/architecture" family;
                }
            }
        }
        layer "consumer" {
            inputs { core, "pretraining" };
            consumes {
                projection-language 1.0.0;
                schemas { }
            }
            projection { select { use @predictor; } }
        }
    }
"#;

#[test]
fn parses_layer_inputs_schema_uses_and_all_projection_stages() {
    let description = parse_description(LAYERED).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(description.layers.len(), 2);

    let pretraining = description
        .layers
        .iter()
        .find(|layer| layer.name == "pretraining")
        .unwrap();
    assert_eq!(pretraining.inputs, vec![LayerInput::Core]);
    assert_eq!(pretraining.schemas[0].name, "shapes");
    assert_eq!(pretraining.projection.select.len(), 2);
    assert_eq!(pretraining.projection.invert.len(), 1);
    assert_eq!(pretraining.projection.decorate.len(), 1);
    assert_eq!(pretraining.projection.checks.len(), 1);
    assert_eq!(
        pretraining.projection.checks[0].expected,
        ExpectedCardinality::Nonempty
    );
    let parameter = &pretraining.projection.decorate[0].parameter;
    assert!(matches!(
        parameter.value,
        ExtensionValue::Known(Value::Product(_))
    ));

    let consumer = description
        .layers
        .iter()
        .find(|layer| layer.name == "consumer")
        .unwrap();
    assert_eq!(
        consumer.inputs,
        vec![
            LayerInput::Core,
            LayerInput::Layer("pretraining".to_owned())
        ]
    );
    assert!(matches!(consumer.projection.select[0], SelectItem::Use(_)));
}

#[test]
fn generated_layer_definition_is_retained_as_an_ordinary_definition() {
    let description = parse_description(LAYERED).unwrap_or_else(|error| panic!("{error}"));
    let pretraining = description
        .layers
        .iter()
        .find(|layer| layer.name == "pretraining")
        .unwrap();
    let SelectItem::GenerateBlock(block) = &pretraining.projection.select[1] else {
        panic!("expected generated block");
    };
    assert_eq!(block.address.as_str(), "@predictor");
    assert_eq!(block.name, "Predictor");
}

#[test]
fn rejects_duplicate_layer_inputs() {
    let source = r#"
        grimoire 1.0.0 description @d {
            core-spec 1.0.0;
            core {}
            layer "one" {
                inputs { core, core };
                consumes { projection-language 1.0.0; schemas { } }
                projection { select { } }
            }
        }
    "#;
    let error = parse_description(source).expect_err("duplicate input should fail");
    assert!(error.message.contains("duplicate layer input"));
}

#[test]
fn rejects_projection_phase_out_of_order() {
    let source = r#"
        grimoire 1.0.0 description @d {
            core-spec 1.0.0;
            core {}
            layer "one" {
                inputs { core };
                consumes { projection-language 1.0.0; schemas { } }
                projection {
                    decorate { }
                    select { }
                }
            }
        }
    "#;
    let error = parse_description(source).expect_err("phase order should fail");
    assert!(error.message.contains("expected `select`"));
}
