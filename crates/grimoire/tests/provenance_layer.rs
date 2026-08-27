use grimoire::{
    Schema, Value, evaluate_layer, parse_description, prototype_schemas, validate_description,
};

const PROVENANCE_LAYER: &str = r#"
    grimoire 1.0.0
    description @d "provenance" {
        core-spec 1.0.0;
        core {
            block @encoder "Encoder" { port @encoder/output; }
            group @model "model" { @encoder; }
        }
        layer "provenance" {
            inputs { core };
            consumes {
                projection-language 1.0.0;
                schemas {
                    "https://github.com/goldenwitch/grimoire/extension/provenance" / provenance @1.0.0;
                }
            }
            projection {
                select { use @model; }
                decorate {
                    on @model extension "https://github.com/goldenwitch/grimoire/extension/provenance" provenance schema provenance @1.0.0 = {
                        citations: ["arXiv:2506.09985"],
                        assumptions: ["the represented composition is authored"],
                        novelty: adapted
                    };
                }
                checks {
                    check provenance-present expect nonempty over "https://github.com/goldenwitch/grimoire/extension/provenance" provenance;
                }
            }
        }
    }
"#;

fn schemas() -> Vec<Schema> {
    prototype_schemas().unwrap_or_else(|error| panic!("{error}"))
}

#[test]
fn provenance_layer_validates_and_checks_finalized_group_data() {
    let description = parse_description(PROVENANCE_LAYER).unwrap_or_else(|error| panic!("{error}"));
    validate_description(&description, &schemas())
        .unwrap_or_else(|errors| panic!("validation errors: {errors:?}"));
    let result =
        evaluate_layer(&description, "provenance").unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(result.structural.elements.len(), 1);
    assert_eq!(result.decorations.len(), 1);
    assert_eq!(result.checks.len(), 1);
    assert_eq!(result.checks[0].observed, 1);
    assert!(result.checks[0].passed);
}

#[test]
fn provenance_values_do_not_change_selected_structure() {
    let original = parse_description(PROVENANCE_LAYER).unwrap_or_else(|error| panic!("{error}"));
    let changed_source = PROVENANCE_LAYER.replace("novelty: adapted", "novelty: existing");
    let changed = parse_description(&changed_source).unwrap_or_else(|error| panic!("{error}"));
    let original_result =
        evaluate_layer(&original, "provenance").unwrap_or_else(|error| panic!("{error}"));
    let changed_result =
        evaluate_layer(&changed, "provenance").unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(original_result.structural, changed_result.structural);
    assert_ne!(original_result.decorations, changed_result.decorations);
}

#[test]
fn provenance_on_a_block_is_rejected_by_the_group_only_schema() {
    let source = PROVENANCE_LAYER.replace("on @model extension", "on @encoder extension");
    let description = parse_description(&source).unwrap_or_else(|error| panic!("{error}"));
    let errors = validate_description(&description, &schemas())
        .expect_err("provenance on a block should fail");
    assert!(errors.iter().any(|error| {
        error.check == "C10"
            && error.identifier.as_deref() == Some("provenance")
            && error.message.contains("does not allow Block")
    }));
}

#[test]
fn provenance_value_uses_the_closed_runtime_algebra() {
    let description = parse_description(PROVENANCE_LAYER).unwrap_or_else(|error| panic!("{error}"));
    let value = &description.layers[0].projection.decorate[0].parameter.value;
    let grimoire::ExtensionValue::Known(Value::Product(fields)) = value else {
        panic!("expected a known provenance value");
    };
    assert!(matches!(fields.get("citations"), Some(Value::Sequence(_))));
}
