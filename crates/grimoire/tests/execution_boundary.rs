use grimoire::{
    Address, CutError, Schema, evaluate_layer, extract_cut, parse_description, prototype_schemas,
    validate_description,
};

const EXECUTION_BOUNDARY: &str = r#"
    grimoire 1.0.0
    description @d "execution boundaries" {
        core-spec 1.0.0;
        core {
            block @speech/input "Speech input" { port @speech/input/out; }
            block @speech/encoder "Speech encoder" {
                port @speech/encoder/input;
                port @speech/encoder/output;
            }
            block @speech/decoder "Speech decoder" {
                port @speech/decoder/input;
                port @speech/decoder/output;
            }
            block @dynamics "Latent dynamics" {
                port @dynamics/state-in;
                port @dynamics/state-out;
            }
            block @planner "Planner" {
                port @planner/input;
                port @planner/output;
            }
            connection @speech-input @speech/input/out -> @speech/encoder/input;
            connection @speech-output @speech/encoder/output -> @speech/decoder/input;
            connection @state-feedback @dynamics/state-out -> @dynamics/state-in;
            connection @dynamics-to-planner @dynamics/state-out -> @planner/input;
        }
        layer "streaming-speech" {
            inputs { core };
            consumes {
                projection-language 1.0.0;
                schemas {
                    "https://github.com/goldenwitch/grimoire/extension/execution" / execution @1.0.0;
                }
            }
            projection {
                select {
                    use @speech/input, @speech/encoder, @speech/decoder,
                       @speech/input/out, @speech/encoder/input, @speech/encoder/output,
                       @speech/decoder/input, @speech/decoder/output,
                       @speech-input, @speech-output;
                }
                decorate {
                    on @speech/encoder extension "https://github.com/goldenwitch/grimoire/extension/execution" execution schema execution @1.0.0 = {
                        regime: streaming,
                        horizon: absent,
                        rate: present(50.0),
                        external_consumer: yes
                    };
                }
                checks {
                    check stream-boundary expect nonempty over "https://github.com/goldenwitch/grimoire/extension/execution" execution;
                }
            }
        }
        layer "recurrent" {
            inputs { core };
            consumes {
                projection-language 1.0.0;
                schemas {
                    "https://github.com/goldenwitch/grimoire/extension/execution" / execution @1.0.0;
                }
            }
            projection {
                select {
                    use @dynamics, @planner, @dynamics/state-in, @dynamics/state-out,
                       @planner/input, @planner/output, @state-feedback, @dynamics-to-planner;
                }
                decorate {
                    on @dynamics extension "https://github.com/goldenwitch/grimoire/extension/execution" execution schema execution @1.0.0 = {
                        regime: recurrent,
                        horizon: present(16),
                        rate: absent,
                        external_consumer: yes
                    };
                }
            }
        }
        layer "closed-loop-planning" {
            inputs { "recurrent" };
            consumes {
                projection-language 1.0.0;
                schemas {
                    "https://github.com/goldenwitch/grimoire/extension/execution" / execution @1.0.0;
                }
            }
            projection {
                select { use @dynamics, @planner; }
                decorate {
                    on @planner extension "https://github.com/goldenwitch/grimoire/extension/execution" execution schema execution @1.0.0 = {
                        regime: closed-loop,
                        horizon: present(5),
                        rate: present(10.0),
                        external_consumer: yes
                    };
                }
                checks {
                    check controller-boundary expect nonempty over "https://github.com/goldenwitch/grimoire/extension/execution" execution;
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
fn execution_regimes_validate_as_static_metadata() {
    let description =
        parse_description(EXECUTION_BOUNDARY).unwrap_or_else(|error| panic!("{error}"));
    validate_description(&description, &schemas())
        .unwrap_or_else(|errors| panic!("validation errors: {errors:?}"));
    let streaming =
        evaluate_layer(&description, "streaming-speech").unwrap_or_else(|error| panic!("{error}"));
    let recurrent =
        evaluate_layer(&description, "recurrent").unwrap_or_else(|error| panic!("{error}"));
    let planning = evaluate_layer(&description, "closed-loop-planning")
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(streaming.structural.elements.len(), 10);
    assert_eq!(recurrent.structural.elements.len(), 8);
    assert_eq!(planning.structural.elements.len(), 2);
    assert!(streaming.checks[0].passed);
    assert!(planning.checks[0].passed);
}

#[test]
fn recurrent_state_feedback_is_structural_and_runtime_remains_external() {
    let description =
        parse_description(EXECUTION_BOUNDARY).unwrap_or_else(|error| panic!("{error}"));
    let result =
        evaluate_layer(&description, "recurrent").unwrap_or_else(|error| panic!("{error}"));
    assert!(
        result
            .structural
            .elements
            .contains_key(&address("@state-feedback"))
    );
    assert_eq!(result.decorations.len(), 1);
    assert!(result.checks.is_empty());
}

#[test]
fn planning_cut_without_recurrent_producer_is_c12() {
    let description =
        parse_description(EXECUTION_BOUNDARY).unwrap_or_else(|error| panic!("{error}"));
    let error = extract_cut(&description, &["closed-loop-planning"], &schemas())
        .expect_err("planning cut should require recurrent producer");
    assert_eq!(
        error,
        CutError::Unresolvable {
            layer: "closed-loop-planning".to_owned(),
            missing: vec!["recurrent".to_owned()],
        }
    );
    assert!(error.to_string().contains("C12"));
}

#[test]
fn unknown_execution_regime_is_a_visible_schema_failure() {
    let source = EXECUTION_BOUNDARY.replace("regime: streaming", "regime: bursty");
    let description = parse_description(&source).unwrap_or_else(|error| panic!("{error}"));
    let errors = validate_description(&description, &schemas())
        .expect_err("unknown execution regime should fail");
    assert!(errors.iter().any(|error| {
        error.check == "C10" && error.message.contains("unknown enumeration value `bursty`")
    }));
}
