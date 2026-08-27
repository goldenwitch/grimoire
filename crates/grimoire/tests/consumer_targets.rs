use grimoire::{
    Element, ExtensionValue, Schema, evaluate_layer, extract_cut, parse_description,
    prototype_schemas, validate_description,
};

const CONSUMERS: &str = r#"
    grimoire 1.0.0
    description @frontier "Cross-paper consumer boundaries" {
        core-spec 1.0.0;
        core {
            block @frontier/vision-encoder "Shared visual encoder" {
                port @frontier/vision-encoder/input;
                port @frontier/vision-encoder/output extensions {
                    extension "https://github.com/goldenwitch/grimoire/extension/shapes" shape schema shapes @1.0.0 = {
                        layout: sequence,
                        dimensions: [literal(196), literal(1408)]
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
                        interface: present(ref(@frontier/vision-encoder/output))
                    };
                }
            }
            block @frontier/shared-transformer "Shared transformer" {
                port @frontier/shared-transformer/input;
                port @frontier/shared-transformer/output;
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
            block @frontier/action "Action input" { port @frontier/action/output; }
            block @frontier/state "State input" { port @frontier/state/output; }
            block @frontier/latent "Latent representation" {
                port @frontier/latent/input;
                port @frontier/latent/output;
            }
            group @frontier/shared "shared representation boundaries" {
                @frontier/vision-encoder,
                @frontier/shared-transformer,
                @frontier/latent;
            }
        }
        layer "bridge" {
            inputs { core };
            consumes {
                projection-language 1.0.0;
                schemas {
                    "https://github.com/goldenwitch/grimoire/extension/architecture" / architecture @1.0.0;
                    "https://github.com/goldenwitch/grimoire/extension/shapes" / shapes @1.0.0;
                }
            }
            projection {
                select {
                    use @frontier/vision-encoder, @frontier/vision-encoder/output;
                    block @bridge/projector "Visual projector" {
                        port @bridge/projector/input;
                        port @bridge/projector/output;
                    }
                    block @bridge/language "Language model" {
                        port @bridge/language/input;
                        port @bridge/language/output;
                    }
                    connection @bridge/encoder-to-projector @frontier/vision-encoder/output -> @bridge/projector/input;
                    connection @bridge/projector-to-language @bridge/projector/output -> @bridge/language/input;
                }
            }
        }
        layer "unified" {
            inputs { core };
            consumes { projection-language 1.0.0; schemas { } }
            projection {
                select {
                    use @frontier/shared-transformer, @frontier/shared-transformer/input,
                        @frontier/shared-transformer/output;
                    block @unified/visual-tokenizer "Visual tokenizer" {
                        port @unified/visual-tokenizer/input;
                        port @unified/visual-tokenizer/output;
                    }
                    block @unified/text-tokenizer "Text tokenizer" {
                        port @unified/text-tokenizer/input;
                        port @unified/text-tokenizer/output;
                    }
                    block @unified/visual-head "Visual head" {
                        port @unified/visual-head/input;
                        port @unified/visual-head/output;
                    }
                    block @unified/text-head "Text head" {
                        port @unified/text-head/input;
                        port @unified/text-head/output;
                    }
                    connection @unified/visual-to-core @unified/visual-tokenizer/output -> @frontier/shared-transformer/input;
                    connection @unified/text-to-core @unified/text-tokenizer/output -> @frontier/shared-transformer/input;
                    connection @unified/core-to-visual @frontier/shared-transformer/output -> @unified/visual-head/input;
                    connection @unified/core-to-text @frontier/shared-transformer/output -> @unified/text-head/input;
                }
            }
        }
        layer "decoupled" {
            inputs { core };
            consumes { projection-language 1.0.0; schemas { } }
            projection {
                select {
                    use @frontier/vision-encoder, @frontier/vision-encoder/output,
                        @frontier/shared-transformer, @frontier/shared-transformer/input;
                    block @decoupled/semantic-encoder "Semantic encoder" {
                        port @decoupled/semantic-encoder/input;
                        port @decoupled/semantic-encoder/output;
                    }
                    block @decoupled/semantic-adaptor "Semantic adaptor" {
                        port @decoupled/semantic-adaptor/input;
                        port @decoupled/semantic-adaptor/output;
                    }
                    block @decoupled/generation-tokenizer "Generation tokenizer" {
                        port @decoupled/generation-tokenizer/input;
                        port @decoupled/generation-tokenizer/output;
                    }
                    block @decoupled/generation-adaptor "Generation adaptor" {
                        port @decoupled/generation-adaptor/input;
                        port @decoupled/generation-adaptor/output;
                    }
                    connection @decoupled/semantic-input @frontier/vision-encoder/output -> @decoupled/semantic-encoder/input;
                    connection @decoupled/semantic-path @decoupled/semantic-encoder/output -> @decoupled/semantic-adaptor/input;
                    connection @decoupled/semantic-to-core @decoupled/semantic-adaptor/output -> @frontier/shared-transformer/input;
                    connection @decoupled/generation-input @frontier/vision-encoder/output -> @decoupled/generation-tokenizer/input;
                    connection @decoupled/generation-path @decoupled/generation-tokenizer/output -> @decoupled/generation-adaptor/input;
                    connection @decoupled/generation-to-core @decoupled/generation-adaptor/output -> @frontier/shared-transformer/input;
                }
            }
        }
        layer "latent" {
            inputs { core };
            consumes { projection-language 1.0.0; schemas { } }
            projection {
                select {
                    use @frontier/latent, @frontier/latent/input, @frontier/latent/output;
                    block @latent/conditioner "Conditioner" {
                        port @latent/conditioner/input;
                        port @latent/conditioner/output;
                    }
                    block @latent/denoiser "Continuous denoiser" {
                        port @latent/denoiser/input;
                        port @latent/denoiser/output;
                    }
                    block @latent/decoder "Latent decoder" {
                        port @latent/decoder/input;
                        port @latent/decoder/output;
                    }
                    connection @latent/condition-to-denoiser @latent/conditioner/output -> @latent/denoiser/input;
                    connection @latent/denoiser-to-latent @latent/denoiser/output -> @frontier/latent/input;
                    connection @latent/latent-to-decoder @frontier/latent/output -> @latent/decoder/input;
                }
            }
        }
        layer "tokenizer" {
            inputs { core };
            consumes {
                projection-language 1.0.0;
                schemas { "https://github.com/goldenwitch/grimoire/extension/shapes" / shapes @1.0.0; }
            }
            projection {
                select {
                    use @frontier/vision-encoder, @frontier/vision-encoder/output;
                    block @tokenizer/encoder "Image encoder" {
                        port @tokenizer/encoder/input;
                        port @tokenizer/encoder/output;
                    }
                    block @tokenizer/latent "One-dimensional latent" {
                        port @tokenizer/latent/input;
                        port @tokenizer/latent/output extensions {
                            extension "https://github.com/goldenwitch/grimoire/extension/shapes" shape schema shapes @1.0.0 = {
                                layout: sequence,
                                dimensions: [literal(32), literal(1024)]
                            };
                        };
                    }
                    block @tokenizer/decoder "Reconstruction decoder" {
                        port @tokenizer/decoder/input;
                        port @tokenizer/decoder/output;
                    }
                    block @tokenizer/generator "Latent generator" {
                        port @tokenizer/generator/input;
                        port @tokenizer/generator/output;
                    }
                    connection @tokenizer/encoder-to-latent @frontier/vision-encoder/output -> @tokenizer/encoder/input;
                    connection @tokenizer/encoder-path @tokenizer/encoder/output -> @tokenizer/latent/input;
                    connection @tokenizer/latent-to-decoder @tokenizer/latent/output -> @tokenizer/decoder/input;
                    connection @tokenizer/latent-to-generator @tokenizer/latent/output -> @tokenizer/generator/input;
                }
            }
        }
        layer "speech" {
            inputs { core };
            consumes {
                projection-language 1.0.0;
                schemas { "https://github.com/goldenwitch/grimoire/extension/execution" / execution @1.0.0; }
            }
            projection {
                select {
                    use @frontier/shared-transformer, @frontier/shared-transformer/input,
                        @frontier/shared-transformer/output;
                    block @speech/encoder "Speech encoder" {
                        port @speech/encoder/input;
                        port @speech/encoder/output;
                    }
                    block @speech/decoder "Streaming speech decoder" {
                        port @speech/decoder/input;
                        port @speech/decoder/output;
                    }
                    connection @speech/encoder-to-core @speech/encoder/output -> @frontier/shared-transformer/input;
                    connection @speech/core-to-decoder @frontier/shared-transformer/output -> @speech/decoder/input;
                }
                decorate {
                    on @speech/decoder extension "https://github.com/goldenwitch/grimoire/extension/execution" execution schema execution @1.0.0 = {
                        regime: streaming,
                        horizon: absent,
                        rate: present(50.0),
                        external_consumer: yes
                    };
                }
            }
        }
        layer "dynamics" {
            inputs { core };
            consumes { projection-language 1.0.0; schemas { } }
            projection {
                select {
                    use @frontier/vision-encoder, @frontier/vision-encoder/output,
                        @frontier/action, @frontier/action/output,
                        @frontier/state, @frontier/state/output;
                    block @dynamics/predictor "Latent dynamics predictor" {
                        port @dynamics/predictor/visual;
                        port @dynamics/predictor/action;
                        port @dynamics/predictor/state;
                        port @dynamics/predictor/output;
                    }
                    block @dynamics/planner "Dynamics planner" {
                        port @dynamics/planner/input;
                        port @dynamics/planner/output;
                    }
                    connection @dynamics/visual-input @frontier/vision-encoder/output -> @dynamics/predictor/visual;
                    connection @dynamics/action-input @frontier/action/output -> @dynamics/predictor/action;
                    connection @dynamics/state-input @frontier/state/output -> @dynamics/predictor/state;
                    connection @dynamics/predictor-to-planner @dynamics/predictor/output -> @dynamics/planner/input;
                }
            }
        }
        layer "low-bit" {
            inputs { core };
            consumes {
                projection-language 1.0.0;
                schemas { "https://github.com/goldenwitch/grimoire/extension/precision" / precision @1.0.0; }
            }
            projection {
                select {
                    use @frontier/shared-transformer, @frontier/shared-transformer/output;
                    block @low-bit/operator "Quantized operator" {
                        port @low-bit/operator/input;
                        port @low-bit/operator/output;
                    }
                    connection @low-bit/transformer-to-operator @frontier/shared-transformer/output -> @low-bit/operator/input;
                }
                decorate {
                    on @low-bit/operator extension "https://github.com/goldenwitch/grimoire/extension/precision" precision schema precision @1.0.0 = {
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
                schemas { "https://github.com/goldenwitch/grimoire/extension/lineage" / lineage @1.0.0; }
            }
            projection {
                select {
                    block @lineage/base "Base parameters" { }
                    block @lineage/delta "Fine-tuning delta" { }
                    block @lineage/merged "Merged parameters" { }
                    group @lineage/states "Parameter states" {
                        @lineage/base, @lineage/delta, @lineage/merged;
                    }
                }
                decorate {
                    on @lineage/merged extension "https://github.com/goldenwitch/grimoire/extension/lineage" lineage schema lineage @1.0.0 = {
                        base: ref(@lineage/base),
                        deltas: [ref(@lineage/delta)],
                        operation: trim-sign-merge,
                        result: ref(@lineage/merged)
                    };
                }
            }
        }
    }
"#;

fn schemas() -> Vec<Schema> {
    prototype_schemas().unwrap_or_else(|error| panic!("{error}"))
}

fn parsed() -> grimoire::Description {
    parse_description(CONSUMERS).unwrap_or_else(|error| panic!("{error}"))
}

#[test]
fn cross_paper_consumers_validate_and_evaluate() {
    let description = parsed();
    validate_description(&description, &schemas())
        .unwrap_or_else(|errors| panic!("validation errors: {errors:?}"));
    for layer in [
        "bridge",
        "unified",
        "decoupled",
        "latent",
        "tokenizer",
        "speech",
        "dynamics",
        "low-bit",
        "lineage",
    ] {
        evaluate_layer(&description, layer).unwrap_or_else(|error| panic!("{layer}: {error}"));
    }
    assert_eq!(description.layers.len(), 9);
}

#[test]
fn bridge_and_unified_consumer_boundaries_stay_distinct() {
    let description = parsed();
    let bridge = evaluate_layer(&description, "bridge").unwrap_or_else(|error| panic!("{error}"));
    let unified = evaluate_layer(&description, "unified").unwrap_or_else(|error| panic!("{error}"));
    let decoupled =
        evaluate_layer(&description, "decoupled").unwrap_or_else(|error| panic!("{error}"));

    assert!(
        bridge
            .structural
            .elements
            .contains_key(&grimoire::Address::parse("@bridge/language").unwrap())
    );
    assert!(
        !bridge
            .structural
            .elements
            .contains_key(&grimoire::Address::parse("@frontier/shared-transformer").unwrap())
    );
    assert!(
        unified
            .structural
            .elements
            .contains_key(&grimoire::Address::parse("@frontier/shared-transformer").unwrap())
    );
    assert!(
        decoupled
            .structural
            .elements
            .contains_key(&grimoire::Address::parse("@decoupled/semantic-encoder").unwrap())
    );
    assert!(
        decoupled
            .structural
            .elements
            .contains_key(&grimoire::Address::parse("@decoupled/generation-tokenizer").unwrap())
    );
}

#[test]
fn consumer_cuts_keep_their_local_producers() {
    let description = parsed();
    for layer in ["bridge", "unified", "tokenizer", "dynamics", "lineage"] {
        let cut = extract_cut(&description, &[layer], &schemas())
            .unwrap_or_else(|error| panic!("{layer}: {error}"));
        validate_description(&cut, &schemas())
            .unwrap_or_else(|errors| panic!("{layer} cut errors: {errors:?}"));
        assert_eq!(cut.layers.len(), 1);
    }
}

#[test]
fn precision_and_lineage_remain_values_not_activation_edges() {
    let description = parsed();
    let low_bit = evaluate_layer(&description, "low-bit").unwrap_or_else(|error| panic!("{error}"));
    let lineage = evaluate_layer(&description, "lineage").unwrap_or_else(|error| panic!("{error}"));

    assert_eq!(low_bit.decorations.len(), 1);
    assert!(lineage.decorations.len() == 1);
    assert!(
        lineage
            .structural
            .elements
            .values()
            .all(|element| !matches!(element, Element::Connection(_)))
    );
    let parameter = &lineage.decorations[0].parameter;
    assert!(matches!(parameter.value, ExtensionValue::Known(_)));
}

#[test]
fn sequence_latents_and_runtime_boundaries_are_explicit() {
    let description = parsed();
    let tokenizer =
        evaluate_layer(&description, "tokenizer").unwrap_or_else(|error| panic!("{error}"));
    let speech = evaluate_layer(&description, "speech").unwrap_or_else(|error| panic!("{error}"));
    assert!(
        tokenizer
            .structural
            .elements
            .contains_key(&grimoire::Address::parse("@tokenizer/latent").unwrap())
    );
    assert_eq!(speech.decorations.len(), 1);
    assert!(
        !speech
            .structural
            .elements
            .contains_key(&grimoire::Address::parse("@speech/runtime-buffer").unwrap())
    );
}
