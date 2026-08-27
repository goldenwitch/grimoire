use grimoire::{
    Schema, evaluate_layer, parse_description, prototype_schemas, validate_description,
};

const HYPERPARAMETERS: &str = r#"
    grimoire 1.0.0
    description @d "hyperparameters" {
        core-spec 1.0.0;
        core {
            block @encoder "Encoder" { port @encoder/output; }
            block @predictor "Predictor" { port @predictor/input; }
            group @training-stage "training stage" { @encoder, @predictor; }
        }
        layer "hyperparameters" {
            inputs { core };
            consumes {
                projection-language 1.0.0;
                schemas {
                    "https://github.com/goldenwitch/grimoire/extension/training" / training @1.0.0;
                }
            }
            projection {
                select { use @training-stage; }
                decorate {
                    on @training-stage extension "https://github.com/goldenwitch/grimoire/extension/training" training schema training @1.0.0 = {
                        objective: "masked representation prediction",
                        optimizer: present("adamw"),
                        batch_size: present(2048),
                        steps: present(100000),
                        phases: [{
                            name: "warmup",
                            steps: present(1000),
                            learning_rate: present(0.0001),
                            frame_count: present(16),
                            resolution: absent
                        }],
                        trainable_targets: [ref(@predictor)],
                        frozen_targets: [ref(@encoder)],
                        data_sources: ["VideoMix22M"]
                    };
                }
                checks {
                    check training-covered expect nonempty over "https://github.com/goldenwitch/grimoire/extension/training" training;
                }
            }
        }
    }
"#;

fn schemas() -> Vec<Schema> {
    prototype_schemas().unwrap_or_else(|error| panic!("{error}"))
}

#[test]
fn hyperparameter_layer_validates_typed_training_values() {
    let description = parse_description(HYPERPARAMETERS).unwrap_or_else(|error| panic!("{error}"));
    validate_description(&description, &schemas())
        .unwrap_or_else(|errors| panic!("validation errors: {errors:?}"));
    let result =
        evaluate_layer(&description, "hyperparameters").unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(result.structural.elements.len(), 3);
    assert_eq!(result.decorations.len(), 1);
    assert_eq!(result.checks[0].observed, 1);
    assert!(result.checks[0].passed);
}

#[test]
fn changing_a_training_dial_does_not_change_structure() {
    let original = parse_description(HYPERPARAMETERS).unwrap_or_else(|error| panic!("{error}"));
    let changed_source =
        HYPERPARAMETERS.replace("batch_size: present(2048)", "batch_size: present(4096)");
    let changed = parse_description(&changed_source).unwrap_or_else(|error| panic!("{error}"));
    let original_result =
        evaluate_layer(&original, "hyperparameters").unwrap_or_else(|error| panic!("{error}"));
    let changed_result =
        evaluate_layer(&changed, "hyperparameters").unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(original_result.structural, changed_result.structural);
    assert_ne!(original_result.decorations, changed_result.decorations);
}

#[test]
fn training_schema_rejects_a_port_attachment() {
    let source = HYPERPARAMETERS.replace(
        "on @training-stage extension",
        "on @encoder/output extension",
    );
    let description = parse_description(&source).unwrap_or_else(|error| panic!("{error}"));
    let errors =
        validate_description(&description, &schemas()).expect_err("training on a port should fail");
    assert!(errors.iter().any(|error| {
        error.check == "C10"
            && error.identifier.as_deref() == Some("training")
            && error.message.contains("does not allow Port")
    }));
}
