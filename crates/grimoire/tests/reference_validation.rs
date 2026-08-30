use std::collections::BTreeMap;

use grimoire::{
    CostExpression, CostModel, CutError, Placement, ShapeDimension, TensorShape, bytes_on_wire,
    evaluate_layer, extract_cut, parse_description, serialize_description, validate_description,
};

mod common;
use common::{address, schemas};

const REFERENCE_DESCRIPTION: &str = r#"
    grimoire 1.0.0
    description @reference "V-JEPA 2 and frontier reference" {
        core-spec 1.0.0;
        core {
            block @reference/observation "Video observation" {
                port @reference/observation/output;
            }
            block @reference/encoder "Shared visual encoder" {
                port @reference/encoder/input extensions {
                    extension "https://github.com/goldenwitch/grimoire/extension/axes" axis schema axes @1.0.0 = {
                        name: "frames",
                        description: present("video time axis")
                    };
                };
                port @reference/encoder/output extensions {
                    extension "https://github.com/goldenwitch/grimoire/extension/shapes" shape schema shapes @1.0.0 = {
                        layout: grid,
                        dimensions: [literal(16), literal(16), literal(1408)]
                    };
                };
                extensions {
                    extension "https://github.com/goldenwitch/grimoire/extension/architecture" architecture schema architecture @1.0.0 = {
                        family: "vision-transformer",
                        parameter_count: present(1000000000),
                        width: present(1408),
                        depth: present(40),
                        head_count: present(22),
                        mlp_width: present(6144),
                        activation: present("gelu"),
                        position_encoding: present("3d-rope"),
                        attention_regime: present(bidirectional),
                        operator: absent,
                        interface: present(ref(@reference/encoder/output))
                    };
                    extension "https://github.com/goldenwitch/grimoire/extension/measurement" benchmark schema measurement @1.0.0 = {
                        value: integer(1000000000),
                        unit: "parameters",
                        source: {
                            origin: "arXiv:2506.09985",
                            locator: present("Section 2"),
                            protocol: absent
                        }
                    };
                }
            }
            block @reference/action "Robot action" {
                port @reference/action/output extensions {
                    extension "https://github.com/goldenwitch/grimoire/extension/shapes" shape schema shapes @1.0.0 = {
                        layout: vector,
                        dimensions: [literal(7)]
                    };
                };
            }
            block @reference/state "End-effector state" {
                port @reference/state/output extensions {
                    extension "https://github.com/goldenwitch/grimoire/extension/shapes" shape schema shapes @1.0.0 = {
                        layout: vector,
                        dimensions: [literal(7)]
                    };
                };
            }
            block @reference/bridge "Visual-language bridge" {
                port @reference/bridge/input;
                port @reference/bridge/output;
            }
            block @reference/transformer "Shared multimodal transformer" {
                port @reference/transformer/input;
                port @reference/transformer/output;
                extensions {
                    extension "https://github.com/goldenwitch/grimoire/extension/architecture" architecture schema architecture @1.0.0 = {
                        family: "multimodal-transformer",
                        parameter_count: absent,
                        width: absent,
                        depth: absent,
                        head_count: absent,
                        mlp_width: absent,
                        activation: present("gelu"),
                        position_encoding: absent,
                        attention_regime: present(mixed),
                        operator: present("transformer"),
                        interface: absent
                    };
                }
            }
            block @reference/latent "Latent representation" {
                port @reference/latent/input;
                port @reference/latent/output;
            }
            block @reference/planner "Planner boundary" {
                port @reference/planner/input;
                port @reference/planner/output;
            }
            block @reference/understanding "Understanding path" {
                port @reference/understanding/input;
                port @reference/understanding/output;
            }
            block @reference/generation "Generation path" {
                port @reference/generation/input;
                port @reference/generation/output;
            }
            connection @reference/observation-to-encoder @reference/observation/output -> @reference/encoder/input;
            connection @reference/encoder-to-bridge @reference/encoder/output -> @reference/bridge/input;
            connection @reference/bridge-to-transformer @reference/bridge/output -> @reference/transformer/input;
            connection @reference/transformer-to-latent @reference/transformer/output -> @reference/latent/input;
            connection @reference/latent-to-planner @reference/latent/output -> @reference/planner/input;
            connection @reference/action-to-planner @reference/action/output -> @reference/planner/input;
            connection @reference/state-to-planner @reference/state/output -> @reference/planner/input;
            group @reference/pipeline "shared pipeline" {
                @reference/encoder,
                @reference/bridge,
                @reference/encoder-to-bridge;
            }
            group @reference/modes "mode alternatives" {
                @reference/understanding,
                @reference/generation;
            }
            group @reference/core "reference core" {
                @reference/observation,
                @reference/encoder,
                @reference/action,
                @reference/state,
                @reference/bridge,
                @reference/transformer,
                @reference/latent,
                @reference/planner,
                @reference/understanding,
                @reference/generation,
                @reference/observation-to-encoder,
                @reference/encoder-to-bridge,
                @reference/bridge-to-transformer,
                @reference/transformer-to-latent,
                @reference/latent-to-planner,
                @reference/action-to-planner,
                @reference/state-to-planner,
                @reference/pipeline,
                @reference/modes;
            }
        }
        layer "pretraining" {
            inputs { core };
            consumes {
                projection-language 1.0.0;
                schemas {
                    "https://github.com/goldenwitch/grimoire/extension/training" / training @1.0.0;
                }
            }
            projection {
                select {
                    use @reference/observation, @reference/observation/output,
                        @reference/encoder, @reference/encoder/input, @reference/encoder/output,
                        @reference/observation-to-encoder;
                    block @reference/pretraining/target-encoder "EMA target encoder" {
                        port @reference/pretraining/target-encoder/input;
                        port @reference/pretraining/target-encoder/output;
                    }
                    block @reference/pretraining/mask-token "Learned mask token" {
                        port @reference/pretraining/mask-token/output;
                    }
                    block @reference/pretraining/predictor "Representation predictor" {
                        port @reference/pretraining/predictor/input;
                        port @reference/pretraining/predictor/mask;
                        port @reference/pretraining/predictor/output;
                    }
                    block @reference/pretraining/objective "Masked objective" {
                        port @reference/pretraining/objective/prediction;
                        port @reference/pretraining/objective/target;
                    }
                    connection @reference/pretraining/encoder-to-predictor @reference/encoder/output -> @reference/pretraining/predictor/input;
                    connection @reference/pretraining/mask-to-predictor @reference/pretraining/mask-token/output -> @reference/pretraining/predictor/mask;
                    connection @reference/pretraining/predictor-to-objective @reference/pretraining/predictor/output -> @reference/pretraining/objective/prediction;
                    connection @reference/pretraining/target-to-objective @reference/pretraining/target-encoder/output -> @reference/pretraining/objective/target;
                    group @reference/pretraining/structure "pretraining structure" {
                        @reference/pretraining/target-encoder,
                        @reference/pretraining/mask-token,
                        @reference/pretraining/predictor,
                        @reference/pretraining/objective,
                        @reference/pretraining/encoder-to-predictor,
                        @reference/pretraining/mask-to-predictor,
                        @reference/pretraining/predictor-to-objective,
                        @reference/pretraining/target-to-objective;
                    }
                }
                decorate {
                    on @reference/pretraining/structure extension "https://github.com/goldenwitch/grimoire/extension/training" training schema training @1.0.0 = {
                        objective: "masked representation prediction",
                        optimizer: absent,
                        batch_size: absent,
                        steps: absent,
                        phases: [],
                        trainable_targets: [ref(@reference/pretraining/predictor)],
                        frozen_targets: [ref(@reference/encoder)],
                        data_sources: ["VideoMix22M"]
                    };
                }
            }
        }
        layer "ac" {
            inputs { core };
            consumes {
                projection-language 1.0.0;
                schemas {
                    "https://github.com/goldenwitch/grimoire/extension/training" / training @1.0.0;
                    "https://github.com/goldenwitch/grimoire/extension/execution" / execution @1.0.0;
                }
            }
            projection {
                select {
                    use @reference/encoder, @reference/encoder/output,
                        @reference/action, @reference/action/output,
                        @reference/state, @reference/state/output;
                    block @reference/ac/predictor "Action-conditioned predictor" {
                        port @reference/ac/predictor/visual;
                        port @reference/ac/predictor/action;
                        port @reference/ac/predictor/state;
                        port @reference/ac/predictor/output;
                    }
                    block @reference/ac/teacher-forcing "Teacher forcing objective" {
                        port @reference/ac/teacher-forcing/input;
                        port @reference/ac/teacher-forcing/output;
                    }
                    block @reference/ac/rollout "Two-step rollout objective" {
                        port @reference/ac/rollout/input;
                        port @reference/ac/rollout/output;
                    }
                    connection @reference/ac/encoder-to-predictor @reference/encoder/output -> @reference/ac/predictor/visual;
                    connection @reference/ac/action-to-predictor @reference/action/output -> @reference/ac/predictor/action;
                    connection @reference/ac/state-to-predictor @reference/state/output -> @reference/ac/predictor/state;
                    connection @reference/ac/predictor-to-teacher @reference/ac/predictor/output -> @reference/ac/teacher-forcing/input;
                    connection @reference/ac/predictor-to-rollout @reference/ac/predictor/output -> @reference/ac/rollout/input;
                    group @reference/ac/structure "action-conditioned structure" {
                        @reference/ac/predictor,
                        @reference/ac/teacher-forcing,
                        @reference/ac/rollout,
                        @reference/ac/encoder-to-predictor,
                        @reference/ac/action-to-predictor,
                        @reference/ac/state-to-predictor,
                        @reference/ac/predictor-to-teacher,
                        @reference/ac/predictor-to-rollout;
                    }
                }
                decorate {
                    on @reference/ac/structure extension "https://github.com/goldenwitch/grimoire/extension/training" training schema training @1.0.0 = {
                        objective: "action-conditioned representation prediction",
                        optimizer: absent,
                        batch_size: absent,
                        steps: absent,
                        phases: [],
                        trainable_targets: [ref(@reference/ac/predictor)],
                        frozen_targets: [ref(@reference/encoder)],
                        data_sources: ["Droid"]
                    };
                    on @reference/ac/structure extension "https://github.com/goldenwitch/grimoire/extension/execution" execution schema execution @1.0.0 = {
                        regime: recurrent,
                        horizon: present(2),
                        rate: absent,
                        external_consumer: no
                    };
                }
            }
        }
        layer "anticipation" {
            inputs { core, "pretraining" };
            consumes { projection-language 1.0.0; schemas { } }
            projection {
                select {
                    use @reference/encoder, @reference/encoder/output,
                        @reference/pretraining/predictor, @reference/pretraining/predictor/output;
                    block @reference/anticipation/probe "Action anticipation probe" {
                        port @reference/anticipation/probe/encoder;
                        port @reference/anticipation/probe/prediction;
                        port @reference/anticipation/probe/output;
                    }
                    connection @reference/anticipation/encoder-to-probe @reference/encoder/output -> @reference/anticipation/probe/encoder;
                    connection @reference/anticipation/predictor-to-probe @reference/pretraining/predictor/output -> @reference/anticipation/probe/prediction;
                }
            }
        }
        layer "vidqa" {
            inputs { core };
            consumes {
                projection-language 1.0.0;
                schemas {
                    "https://github.com/goldenwitch/grimoire/extension/shapes" / shapes @1.0.0;
                }
            }
            projection {
                select {
                    use @reference/encoder, @reference/encoder/output;
                    block @reference/vidqa/projector "Visual-language projector" {
                        port @reference/vidqa/projector/input;
                        port @reference/vidqa/projector/output;
                    }
                    block @reference/vidqa/language "Language model" {
                        port @reference/vidqa/language/input;
                        port @reference/vidqa/language/output;
                    }
                    connection @reference/vidqa/encoder-to-projector @reference/encoder/output -> @reference/vidqa/projector/input;
                    connection @reference/vidqa/projector-to-language @reference/vidqa/projector/output -> @reference/vidqa/language/input;
                }
            }
        }
        layer "planning" {
            inputs { core, "ac" };
            consumes {
                projection-language 1.0.0;
                schemas {
                    "https://github.com/goldenwitch/grimoire/extension/execution" / execution @1.0.0;
                }
            }
            projection {
                select {
                    use @reference/encoder, @reference/action, @reference/state,
                        @reference/ac/predictor, @reference/ac/predictor/output;
                    block @reference/planning/controller "External controller" {
                        port @reference/planning/controller/input;
                        port @reference/planning/controller/output;
                    }
                    connection @reference/planning/predictor-to-controller @reference/ac/predictor/output -> @reference/planning/controller/input;
                }
                decorate {
                    on @reference/planning/controller extension "https://github.com/goldenwitch/grimoire/extension/execution" execution schema execution @1.0.0 = {
                        regime: closed-loop,
                        horizon: present(5),
                        rate: present(10.0),
                        external_consumer: yes
                    };
                }
            }
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
                select { use @reference/pipeline; }
                decorate {
                    on @reference/pipeline extension "https://github.com/goldenwitch/grimoire/extension/training" training schema training @1.0.0 = {
                        objective: "reference analysis",
                        optimizer: present("adamw"),
                        batch_size: present(2048),
                        steps: present(100000),
                        phases: [],
                        trainable_targets: [],
                        frozen_targets: [ref(@reference/encoder)],
                        data_sources: ["reference fixture"]
                    };
                }
            }
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
                select { use @reference/pipeline; }
                decorate {
                    on @reference/pipeline extension "https://github.com/goldenwitch/grimoire/extension/provenance" provenance schema provenance @1.0.0 = {
                        citations: ["arXiv:2506.09985", "arXiv:2405.09818"],
                        assumptions: ["shared addresses denote actual reuse"],
                        novelty: adapted
                    };
                }
            }
        }
        layer "placement" {
            inputs { core };
            consumes {
                projection-language 1.0.0;
                schemas {
                    "https://github.com/goldenwitch/grimoire/extension/placement" / placement @1.0.0;
                }
            }
            projection {
                select {
                    use @reference/encoder, @reference/encoder/output,
                        @reference/bridge, @reference/bridge/input,
                        @reference/encoder-to-bridge;
                }
                decorate {
                    on @reference/encoder extension "https://github.com/goldenwitch/grimoire/extension/placement" placement schema placement @1.0.0 = { location: "gpu-0" };
                    on @reference/bridge extension "https://github.com/goldenwitch/grimoire/extension/placement" placement schema placement @1.0.0 = { location: "gpu-1" };
                }
            }
        }
        layer "cost" {
            inputs { core };
            consumes { projection-language 1.0.0; schemas { } }
            projection { select { use @reference/pipeline; } }
        }
        layer "mode" {
            inputs { core };
            consumes { projection-language 1.0.0; schemas { } }
            projection {
                select {
                    use @reference/transformer, @reference/transformer/input,
                        @reference/transformer/output, @reference/understanding;
                    block @reference/mode/probe "Understanding probe" {
                        port @reference/mode/probe/input;
                    }
                }
            }
        }
        layer "execution" {
            inputs { core };
            consumes {
                projection-language 1.0.0;
                schemas {
                    "https://github.com/goldenwitch/grimoire/extension/execution" / execution @1.0.0;
                }
            }
            projection {
                select { use @reference/latent, @reference/planner; }
                decorate {
                    on @reference/planner extension "https://github.com/goldenwitch/grimoire/extension/execution" execution schema execution @1.0.0 = {
                        regime: static,
                        horizon: absent,
                        rate: absent,
                        external_consumer: no
                    };
                }
            }
        }
        layer "info-flow" {
            inputs { core };
            consumes { projection-language 1.0.0; schemas { } }
            projection {
                select {
                    use @reference/observation, @reference/observation/output,
                        @reference/encoder, @reference/encoder/input, @reference/encoder/output,
                        @reference/observation-to-encoder;
                }
            }
        }
        layer "low-bit" {
            inputs { core };
            consumes {
                projection-language 1.0.0;
                schemas {
                    "https://github.com/goldenwitch/grimoire/extension/precision" / precision @1.0.0;
                }
            }
            projection {
                select {
                    use @reference/transformer, @reference/transformer/output;
                    block @reference/low-bit/operator "Quantized operator" {
                        port @reference/low-bit/operator/input;
                        port @reference/low-bit/operator/output;
                    }
                    connection @reference/low-bit/transformer-to-operator @reference/transformer/output -> @reference/low-bit/operator/input;
                }
                decorate {
                    on @reference/low-bit/operator extension "https://github.com/goldenwitch/grimoire/extension/precision" precision schema precision @1.0.0 = {
                        weights: present("ternary"),
                        activations: present("int4"),
                        accumulation: present("bf16"),
                        optimizer_state: present("bf16"),
                        sparsity: present("sparse")
                    };
                }
            }
        }
        layer "lineage" {
            inputs { core };
            consumes {
                projection-language 1.0.0;
                schemas {
                    "https://github.com/goldenwitch/grimoire/extension/lineage" / lineage @1.0.0;
                }
            }
            projection {
                select {
                    block @reference/lineage/base "Base parameters" { }
                    block @reference/lineage/delta "Fine-tuning delta" { }
                    block @reference/lineage/merged "Merged parameters" { }
                    group @reference/lineage/states "Parameter states" {
                        @reference/lineage/base,
                        @reference/lineage/delta,
                        @reference/lineage/merged;
                    }
                }
                decorate {
                    on @reference/lineage/merged extension "https://github.com/goldenwitch/grimoire/extension/lineage" lineage schema lineage @1.0.0 = {
                        base: ref(@reference/lineage/base),
                        deltas: [ref(@reference/lineage/delta)],
                        operation: trim-sign-merge,
                        result: ref(@reference/lineage/merged)
                    };
                }
            }
        }
    }
"#;

fn parsed() -> grimoire::Description {
    parse_description(REFERENCE_DESCRIPTION).unwrap_or_else(|error| panic!("{error}"))
}

#[test]
fn reference_description_composes_all_static_surfaces() {
    let description = parsed();
    validate_description(&description, &schemas())
        .unwrap_or_else(|errors| panic!("validation errors: {errors:?}"));
    assert_eq!(description.layers.len(), 14);
    for layer in [
        "pretraining",
        "ac",
        "anticipation",
        "vidqa",
        "planning",
        "hyperparameters",
        "provenance",
        "placement",
        "cost",
        "mode",
        "execution",
        "info-flow",
        "low-bit",
        "lineage",
    ] {
        evaluate_layer(&description, layer).unwrap_or_else(|error| panic!("{layer}: {error}"));
    }
}

#[test]
fn reference_description_has_serializer_and_cut_properties() {
    let description = parsed();
    validate_description(&description, &schemas())
        .unwrap_or_else(|errors| panic!("validation errors: {errors:?}"));
    let serialized = serialize_description(&description).unwrap_or_else(|error| panic!("{error}"));
    let reparsed = parse_description(&serialized).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(reparsed.address, description.address);
    assert_eq!(reparsed.label, description.label);
    assert_eq!(reparsed.core_spec, description.core_spec);
    assert_eq!(reparsed.core, description.core);
    assert_eq!(reparsed.extensions, description.extensions);
    assert_eq!(reparsed.layers.len(), description.layers.len());
    for (reparsed_layer, original_layer) in reparsed.layers.iter().zip(&description.layers) {
        assert_eq!(reparsed_layer.name, original_layer.name);
        assert_eq!(reparsed_layer.inputs, original_layer.inputs);
        assert_eq!(
            reparsed_layer.projection_language,
            original_layer.projection_language
        );
        assert_eq!(reparsed_layer.schemas, original_layer.schemas);
        assert_eq!(
            reparsed_layer.projection, original_layer.projection,
            "layer mismatch"
        );
    }
    assert_eq!(serialize_description(&reparsed).unwrap(), serialized);

    for selected in [
        vec!["pretraining"],
        vec!["ac"],
        vec!["ac", "planning"],
        vec!["pretraining", "anticipation"],
        vec!["placement", "cost"],
    ] {
        let cut = extract_cut(&description, &selected, &schemas())
            .unwrap_or_else(|error| panic!("{selected:?}: {error}"));
        validate_description(&cut, &schemas())
            .unwrap_or_else(|errors| panic!("{selected:?} cut errors: {errors:?}"));
        let cut_text = serialize_description(&cut).unwrap_or_else(|error| panic!("{error}"));
        let reparsed_cut = parse_description(&cut_text).unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(reparsed_cut, cut);
    }

    let error = extract_cut(&description, &["planning"], &schemas())
        .expect_err("planning without AC must be unresolvable");
    assert_eq!(
        error,
        CutError::Unresolvable {
            layer: "planning".to_owned(),
            missing: vec!["ac".to_owned()],
        }
    );
    assert!(error.to_string().contains("C12"));
}

#[test]
fn reference_description_exercises_explicit_placement_and_cost_inputs() {
    let description = parsed();
    let placement_result =
        evaluate_layer(&description, "placement").unwrap_or_else(|error| panic!("{error}"));
    let placement = Placement::from_decorations(&placement_result.decorations)
        .unwrap_or_else(|error| panic!("{error}"));
    let report = bytes_on_wire(
        &placement_result.structural,
        &placement,
        &BTreeMap::from([(
            address("@reference/encoder/output"),
            TensorShape::new(vec![ShapeDimension::Literal(4)], 2).unwrap(),
        )]),
        &BTreeMap::new(),
        &[],
    )
    .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(report.total_bytes(), 8);
    assert_eq!(report.transfers.len(), 1);

    let cost_result =
        evaluate_layer(&description, "cost").unwrap_or_else(|error| panic!("{error}"));
    let model = CostModel::new(vec![
        (address("@reference/encoder"), CostExpression::constant(10)),
        (address("@reference/bridge"), CostExpression::constant(20)),
        (
            address("@reference/encoder-to-bridge"),
            CostExpression::product(vec![
                CostExpression::constant(2),
                CostExpression::axis(address("@axis/tokens")),
            ]),
        ),
    ])
    .unwrap_or_else(|error| panic!("{error}"));
    let report = model
        .evaluate(
            &cost_result.structural,
            &BTreeMap::from([(address("@axis/tokens"), 4)]),
        )
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        report
            .group_total(&cost_result.structural, &address("@reference/pipeline"))
            .unwrap_or_else(|error| panic!("{error}")),
        38
    );
}
