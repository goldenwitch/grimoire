use grimoire::{Schema, parse_description, prototype_schemas, validate_description};

const REFERENCE_CORE: &str = r#"
    grimoire 1.0.0
    description @frontier-reference "Shared V-JEPA 2 and frontier boundaries" {
        core-spec 1.0.0;
        core {
            block @shared/vision-encoder "Shared vision encoder" {
                port @shared/vision-encoder/input "video input" extensions {
                    extension "https://github.com/goldenwitch/grimoire/extension/axes" axis schema axes @1.0.0 = { name: "frames", description: absent };
                };
                port @shared/vision-encoder/output "patch representation" extensions {
                    extension "https://github.com/goldenwitch/grimoire/extension/shapes" shape schema shapes @1.0.0 = {
                        layout: sequence,
                        dimensions: [symbolic(ref(@shared/vision-encoder/input)), literal(1408)]
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
                        interface: present(ref(@shared/vision-encoder/output))
                    };
                }
            }
            block @shared/vision-tokenizer "Visual tokenizer" {
                port @shared/vision-tokenizer/input "image input";
                port @shared/vision-tokenizer/output "visual tokens";
            }
            block @shared/bridge "Encoder to language bridge" {
                port @shared/bridge/input "visual features";
                port @shared/bridge/output "language features";
            }
            block @shared/transformer-backbone "Shared transformer backbone" {
                port @shared/transformer-backbone/input "token input";
                port @shared/transformer-backbone/output "token output";
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
            block @shared/predictor "Representation predictor" {
                port @shared/predictor/input "context input";
                port @shared/predictor/output "predicted representation";
            }
            block @shared/latent-state "Continuous latent state" {
                port @shared/latent-state/input "latent input";
                port @shared/latent-state/output "latent output";
            }
            block @shared/speech-input "Speech input" {
                port @shared/speech-input/input "audio input";
                port @shared/speech-input/output "speech features";
            }
            block @shared/speech-decoder "Speech decoder" {
                port @shared/speech-decoder/input "decoder input";
                port @shared/speech-decoder/output "audio output";
            }
            block @shared/operator "Low-bit operator boundary" {
                port @shared/operator/input "operator input";
                port @shared/operator/output "operator output";
            }
            connection @shared/encoder-to-tokenizer @shared/vision-encoder/output -> @shared/vision-tokenizer/input;
            connection @shared/tokenizer-to-bridge @shared/vision-tokenizer/output -> @shared/bridge/input;
            connection @shared/bridge-to-backbone @shared/bridge/output -> @shared/transformer-backbone/input;
            connection @shared/backbone-to-predictor @shared/transformer-backbone/output -> @shared/predictor/input;
            connection @shared/predictor-to-latent @shared/predictor/output -> @shared/latent-state/input;
            connection @shared/speech-to-backbone @shared/speech-input/output -> @shared/transformer-backbone/input;
            connection @shared/latent-to-operator @shared/latent-state/output -> @shared/operator/input;
            connection @shared/operator-to-speech @shared/operator/output -> @shared/speech-decoder/input;
            group @shared/core-boundaries "shared architecture boundaries" {
                @shared/vision-encoder,
                @shared/vision-tokenizer,
                @shared/bridge,
                @shared/transformer-backbone,
                @shared/predictor,
                @shared/latent-state,
                @shared/speech-input,
                @shared/speech-decoder,
                @shared/operator,
                @shared/encoder-to-tokenizer,
                @shared/tokenizer-to-bridge,
                @shared/bridge-to-backbone,
                @shared/backbone-to-predictor,
                @shared/predictor-to-latent,
                @shared/speech-to-backbone,
                @shared/latent-to-operator,
                @shared/operator-to-speech;
                extensions {
                    extension "https://github.com/goldenwitch/grimoire/extension/provenance" provenance schema provenance @1.0.0 = {
                        citations: ["arXiv:2506.09985", "arXiv:2405.09818"],
                        assumptions: ["shared addresses denote actual reuse"],
                        novelty: adapted
                    };
                }
            }
        }
    }
"#;

fn schemas() -> Vec<Schema> {
    prototype_schemas().unwrap_or_else(|error| panic!("{error}"))
}

#[test]
fn parses_the_shared_reference_core() {
    let description = parse_description(REFERENCE_CORE).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(description.core.blocks.len(), 9);
    assert_eq!(description.core.connections.len(), 8);
    assert_eq!(description.core.groups.len(), 1);
    assert_eq!(
        description.core.blocks[&grimoire::Address::parse("@shared/vision-encoder")
            .unwrap_or_else(|error| panic!("{error}"))]
            .ports
            .len(),
        2
    );
}

#[test]
fn the_shared_reference_core_validates_without_new_element_kinds() {
    let description = parse_description(REFERENCE_CORE).unwrap_or_else(|error| panic!("{error}"));
    validate_description(&description, &schemas())
        .unwrap_or_else(|errors| panic!("validation errors: {errors:?}"));
    assert_eq!(description.addresses().len(), 37);
}
