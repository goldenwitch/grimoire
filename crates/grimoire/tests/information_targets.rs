use grimoire::{
    Channel, ChannelGraph, ChannelNode, ClaimEstimate, Distribution, JointSource, Schema,
    evaluate_layer, parse_description, prototype_schemas, validate_description,
};

fn address(value: &str) -> grimoire::Address {
    grimoire::Address::parse(value).unwrap_or_else(|error| panic!("{error}"))
}

fn node(
    address_value: &str,
    block: &str,
    input_ports: &[&str],
    output_port: &str,
    channel: Channel,
) -> ChannelNode {
    ChannelNode::new(
        address(address_value),
        address(block),
        input_ports.iter().map(|value| address(value)).collect(),
        address(output_port),
        channel,
    )
    .unwrap_or_else(|error| panic!("{error}"))
}

fn schemas() -> Vec<Schema> {
    prototype_schemas().unwrap_or_else(|error| panic!("{error}"))
}

fn binary_source() -> Distribution {
    Distribution::uniform(2).unwrap_or_else(|error| panic!("{error}"))
}

fn joint_robot_source() -> JointSource {
    JointSource::new(
        vec![
            address("@vjepa2/observation/output"),
            address("@vjepa2/action/output"),
            address("@vjepa2/state/output"),
        ],
        vec![2, 2, 2],
        Distribution::new(vec![0.225, 0.225, 0.025, 0.025, 0.025, 0.025, 0.225, 0.225])
            .unwrap_or_else(|error| panic!("{error}")),
    )
    .unwrap_or_else(|error| panic!("{error}"))
}

const TARGET: &str = r#"
    grimoire 1.0.0
    description @vjepa2 "V-JEPA 2 target scenarios" {
        core-spec 1.0.0;
        core {
            block @vjepa2/observation "Video observation" {
                port @vjepa2/observation/output;
            }
            block @vjepa2/encoder "Shared V-JEPA 2 encoder" {
                port @vjepa2/encoder/input;
                port @vjepa2/encoder/output;
            }
            block @vjepa2/action "Robot action" {
                port @vjepa2/action/output;
            }
            block @vjepa2/state "End-effector state" {
                port @vjepa2/state/output;
            }
            connection @vjepa2/observation-to-encoder @vjepa2/observation/output -> @vjepa2/encoder/input;
        }
        layer "pretraining" {
            inputs { core };
            consumes { projection-language 1.0.0; schemas { } }
            projection {
                select {
                    use @vjepa2/observation, @vjepa2/observation/output,
                        @vjepa2/encoder, @vjepa2/encoder/input, @vjepa2/encoder/output,
                        @vjepa2/observation-to-encoder;
                    block @vjepa2/pretraining/predictor "Representation predictor" {
                        port @vjepa2/pretraining/predictor/input;
                        port @vjepa2/pretraining/predictor/output;
                    }
                    connection @vjepa2/pretraining/encoder-to-predictor @vjepa2/encoder/output -> @vjepa2/pretraining/predictor/input;
                }
            }
        }
        layer "ac" {
            inputs { core };
            consumes { projection-language 1.0.0; schemas { } }
            projection {
                select {
                    use @vjepa2/observation, @vjepa2/observation/output,
                        @vjepa2/encoder, @vjepa2/encoder/input, @vjepa2/encoder/output,
                        @vjepa2/action, @vjepa2/action/output,
                        @vjepa2/state, @vjepa2/state/output,
                        @vjepa2/observation-to-encoder;
                    block @vjepa2/ac/predictor "Action-conditioned predictor" {
                        port @vjepa2/ac/predictor/visual;
                        port @vjepa2/ac/predictor/action;
                        port @vjepa2/ac/predictor/state;
                        port @vjepa2/ac/predictor/output;
                    }
                    connection @vjepa2/ac/encoder-to-predictor @vjepa2/encoder/output -> @vjepa2/ac/predictor/visual;
                    connection @vjepa2/ac/action-to-predictor @vjepa2/action/output -> @vjepa2/ac/predictor/action;
                    connection @vjepa2/ac/state-to-predictor @vjepa2/state/output -> @vjepa2/ac/predictor/state;
                    group @vjepa2/ac/structure "Action-conditioned structure" {
                        @vjepa2/ac/predictor,
                        @vjepa2/ac/predictor/output,
                        @vjepa2/ac/encoder-to-predictor,
                        @vjepa2/ac/action-to-predictor,
                        @vjepa2/ac/state-to-predictor;
                    }
                }
            }
        }
        layer "vidqa" {
            inputs { core };
            consumes { projection-language 1.0.0; schemas { } }
            projection {
                select {
                    use @vjepa2/observation, @vjepa2/observation/output,
                        @vjepa2/encoder, @vjepa2/encoder/input, @vjepa2/encoder/output,
                        @vjepa2/observation-to-encoder;
                    block @vjepa2/vidqa/projector "Visual-language projector" {
                        port @vjepa2/vidqa/projector/input;
                        port @vjepa2/vidqa/projector/output;
                    }
                    block @vjepa2/vidqa/language "Language model" {
                        port @vjepa2/vidqa/language/input;
                        port @vjepa2/vidqa/language/output;
                    }
                    connection @vjepa2/vidqa/encoder-to-projector @vjepa2/encoder/output -> @vjepa2/vidqa/projector/input;
                    connection @vjepa2/vidqa/projector-to-language @vjepa2/vidqa/projector/output -> @vjepa2/vidqa/language/input;
                }
            }
        }
        layer "planning" {
            inputs { core, "ac" };
            consumes { projection-language 1.0.0; schemas { } }
            projection {
                select {
                    use @vjepa2/observation, @vjepa2/observation/output,
                        @vjepa2/encoder, @vjepa2/encoder/input, @vjepa2/encoder/output,
                        @vjepa2/action, @vjepa2/action/output,
                        @vjepa2/state, @vjepa2/state/output,
                        @vjepa2/observation-to-encoder,
                        @vjepa2/ac/predictor, @vjepa2/ac/predictor/visual,
                        @vjepa2/ac/predictor/action, @vjepa2/ac/predictor/state,
                        @vjepa2/ac/predictor/output,
                        @vjepa2/ac/encoder-to-predictor,
                        @vjepa2/ac/action-to-predictor,
                        @vjepa2/ac/state-to-predictor;
                    block @vjepa2/planning/controller "External controller boundary" {
                        port @vjepa2/planning/controller/input;
                        port @vjepa2/planning/controller/output;
                    }
                    connection @vjepa2/planning/predictor-to-controller @vjepa2/ac/predictor/output -> @vjepa2/planning/controller/input;
                }
            }
        }
    }
"#;

#[test]
fn target_scenarios_parse_and_validate_as_existing_grimoire_structure() {
    let description = parse_description(TARGET).unwrap_or_else(|error| panic!("{error}"));
    validate_description(&description, &schemas())
        .unwrap_or_else(|errors| panic!("validation errors: {errors:?}"));
    assert_eq!(description.layers.len(), 4);
    assert_eq!(description.core.blocks.len(), 4);
}

#[test]
fn shared_encoder_and_distinct_terminals_are_queryable() {
    let description = parse_description(TARGET).unwrap_or_else(|error| panic!("{error}"));
    let pretraining =
        evaluate_layer(&description, "pretraining").unwrap_or_else(|error| panic!("{error}"));
    let ac = evaluate_layer(&description, "ac").unwrap_or_else(|error| panic!("{error}"));
    let vidqa = evaluate_layer(&description, "vidqa").unwrap_or_else(|error| panic!("{error}"));
    let planning =
        evaluate_layer(&description, "planning").unwrap_or_else(|error| panic!("{error}"));

    assert!(
        pretraining
            .structural
            .elements
            .contains_key(&address("@vjepa2/encoder"))
    );
    assert!(
        ac.structural
            .elements
            .contains_key(&address("@vjepa2/encoder"))
    );
    assert!(
        vidqa
            .structural
            .elements
            .contains_key(&address("@vjepa2/encoder"))
    );
    assert!(
        planning
            .structural
            .elements
            .contains_key(&address("@vjepa2/encoder"))
    );
    assert!(
        pretraining
            .structural
            .elements
            .contains_key(&address("@vjepa2/pretraining/predictor"))
    );
    assert!(
        ac.structural
            .elements
            .contains_key(&address("@vjepa2/ac/predictor"))
    );
    assert!(
        vidqa
            .structural
            .elements
            .contains_key(&address("@vjepa2/vidqa/language"))
    );
    assert!(
        planning
            .structural
            .elements
            .contains_key(&address("@vjepa2/planning/controller"))
    );
    assert!(
        !ac.structural
            .elements
            .contains_key(&address("@vjepa2/pretraining/predictor"))
    );
}

#[test]
fn pretraining_vidqa_and_planning_have_explicit_source_to_terminal_claims() {
    let description = parse_description(TARGET).unwrap_or_else(|error| panic!("{error}"));
    let source = binary_source();
    let identity = Channel::identity(2).unwrap_or_else(|error| panic!("{error}"));

    let pretraining = ChannelGraph::from_layer(
        &description,
        "pretraining",
        vec![
            node(
                "@channel/pretraining-encoder",
                "@vjepa2/encoder",
                &["@vjepa2/encoder/input"],
                "@vjepa2/encoder/output",
                identity.clone(),
            ),
            node(
                "@channel/pretraining-predictor",
                "@vjepa2/pretraining/predictor",
                &["@vjepa2/pretraining/predictor/input"],
                "@vjepa2/pretraining/predictor/output",
                Channel::new(vec![vec![0.9, 0.1], vec![0.1, 0.9]])
                    .unwrap_or_else(|error| panic!("{error}")),
            ),
        ],
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let pretraining_claim = pretraining
        .information_claim(
            &address("@vjepa2/observation/output"),
            &source,
            &address("@vjepa2/pretraining/predictor/output"),
            "finite-reference-channel".to_owned(),
            "V-JEPA 2 action-free pretraining fixture".to_owned(),
        )
        .unwrap_or_else(|error| panic!("{error}"));
    assert!(matches!(pretraining_claim.estimate, ClaimEstimate::Exact(value) if value > 0.5));

    let vidqa = ChannelGraph::from_layer(
        &description,
        "vidqa",
        vec![
            node(
                "@channel/vidqa-encoder",
                "@vjepa2/encoder",
                &["@vjepa2/encoder/input"],
                "@vjepa2/encoder/output",
                identity.clone(),
            ),
            node(
                "@channel/vidqa-projector",
                "@vjepa2/vidqa/projector",
                &["@vjepa2/vidqa/projector/input"],
                "@vjepa2/vidqa/projector/output",
                identity.clone(),
            ),
            node(
                "@channel/vidqa-language",
                "@vjepa2/vidqa/language",
                &["@vjepa2/vidqa/language/input"],
                "@vjepa2/vidqa/language/output",
                identity,
            ),
        ],
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let vidqa_claim = vidqa
        .information_claim(
            &address("@vjepa2/observation/output"),
            &source,
            &address("@vjepa2/vidqa/language/output"),
            "finite-reference-channel".to_owned(),
            "V-JEPA 2 VidQA fixture".to_owned(),
        )
        .unwrap_or_else(|error| panic!("{error}"));
    assert!(matches!(vidqa_claim.estimate, ClaimEstimate::Exact(value) if value == 1.0));

    let planning = ChannelGraph::from_layer(
        &description,
        "planning",
        vec![
            node(
                "@channel/planning-encoder",
                "@vjepa2/encoder",
                &["@vjepa2/encoder/input"],
                "@vjepa2/encoder/output",
                Channel::identity(2).unwrap(),
            ),
            node(
                "@channel/planning-ac",
                "@vjepa2/ac/predictor",
                &[
                    "@vjepa2/ac/predictor/visual",
                    "@vjepa2/ac/predictor/action",
                    "@vjepa2/ac/predictor/state",
                ],
                "@vjepa2/ac/predictor/output",
                Channel::deterministic(vec![0, 0, 1, 1, 0, 0, 1, 1], 2)
                    .unwrap_or_else(|error| panic!("{error}")),
            ),
            node(
                "@channel/planning-controller",
                "@vjepa2/planning/controller",
                &["@vjepa2/planning/controller/input"],
                "@vjepa2/planning/controller/output",
                Channel::identity(2).unwrap(),
            ),
        ],
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let planning_claim = planning
        .information_claim_with_joint_source(
            &joint_robot_source(),
            &address("@vjepa2/observation/output"),
            &address("@vjepa2/planning/controller/output"),
            "finite-joint-reference-channel".to_owned(),
            "V-JEPA 2 planning static boundary fixture".to_owned(),
        )
        .unwrap_or_else(|error| panic!("{error}"));
    assert!(
        matches!(planning_claim.estimate, ClaimEstimate::Exact(value) if value > 0.4 && value < 0.7)
    );
}
