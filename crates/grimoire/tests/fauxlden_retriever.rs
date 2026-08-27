use grimoire::{
    Element, ExtensionValue, Placement, Schema, evaluate_layer, parse_description,
    prototype_schemas, serialize_description, validate_description,
};

const FAUXLDEN_RETRIEVER: &str = r#"
    grimoire 1.0.0
    description @fauxlden "Fauxlden Retriever consumption fixture" {
        core-spec 1.0.0;
        core {
            block @fauxlden/input "Observation input" {
                port @fauxlden/input/output;
            }
            block @fauxlden/encoder "Hot-path encoder" {
                port @fauxlden/encoder/input;
                port @fauxlden/encoder/output;
                extensions {
                    extension "https://github.com/goldenwitch/grimoire/extension/precision" precision schema precision @1.0.0 = {
                        weights: present("bf16"),
                        activations: present("bf16"),
                        accumulation: present("fp32"),
                        optimizer_state: present("fp32"),
                        sparsity: absent
                    };
                }
            }
            block @fauxlden/ranker "Paired spacetime ranker" {
                port @fauxlden/ranker/candidate-0-frame-0;
                port @fauxlden/ranker/candidate-0-frame-1;
                port @fauxlden/ranker/candidate-1-frame-0;
                port @fauxlden/ranker/candidate-1-frame-1;
                port @fauxlden/ranker/score-0;
                port @fauxlden/ranker/score-1;
                extensions {
                    extension "https://github.com/goldenwitch/grimoire/extension/measurement" bandwidth schema measurement @1.0.0 = {
                        value: integer(4096),
                        unit: "bytes",
                        source: {
                            origin: "synthetic/fauxlden",
                            locator: present("paired scorer"),
                            protocol: present("all candidate spacetime")
                        }
                    };
                }
            }
            block @fauxlden/selector "Exactly-one candidate selector" {
                port @fauxlden/selector/score-0;
                port @fauxlden/selector/score-1;
                port @fauxlden/selector/selected;
            }
            block @fauxlden/executor "Execution boundary" {
                port @fauxlden/executor/input;
                port @fauxlden/executor/output;
            }
            connection @fauxlden/input-to-encoder @fauxlden/input/output -> @fauxlden/encoder/input;
            connection @fauxlden/encoder-to-ranker @fauxlden/encoder/output -> @fauxlden/ranker/candidate-0-frame-0;
            connection @fauxlden/score-0-to-selector @fauxlden/ranker/score-0 -> @fauxlden/selector/score-0;
            connection @fauxlden/score-1-to-selector @fauxlden/ranker/score-1 -> @fauxlden/selector/score-1;
            connection @fauxlden/selector-to-executor @fauxlden/selector/selected -> @fauxlden/executor/input;
            group @fauxlden/core "retriever core" {
                @fauxlden/input,
                @fauxlden/encoder,
                @fauxlden/ranker,
                @fauxlden/selector,
                @fauxlden/executor;
            }
        }
        layer "retrieval" {
            inputs { core };
            consumes {
                projection-language 1.0.0;
                schemas {
                    "https://github.com/goldenwitch/grimoire/extension/execution" / execution @1.0.0;
                    "https://github.com/goldenwitch/grimoire/extension/measurement" / measurement @1.0.0;
                    "https://github.com/goldenwitch/grimoire/extension/placement" / placement @1.0.0;
                    "https://github.com/goldenwitch/grimoire/extension/provenance" / provenance @1.0.0;
                    "https://github.com/goldenwitch/grimoire/extension/training" / training @1.0.0;
                }
            }
            projection {
                select {
                    use @fauxlden/core,
                        @fauxlden/input/output,
                        @fauxlden/encoder/input, @fauxlden/encoder/output,
                        @fauxlden/ranker/candidate-0-frame-0,
                        @fauxlden/ranker/candidate-0-frame-1,
                        @fauxlden/ranker/candidate-1-frame-0,
                        @fauxlden/ranker/candidate-1-frame-1,
                        @fauxlden/ranker/score-0, @fauxlden/ranker/score-1,
                        @fauxlden/selector/score-0, @fauxlden/selector/score-1,
                        @fauxlden/selector/selected,
                        @fauxlden/executor/input, @fauxlden/executor/output,
                        @fauxlden/input-to-encoder, @fauxlden/encoder-to-ranker,
                        @fauxlden/score-0-to-selector, @fauxlden/score-1-to-selector,
                        @fauxlden/selector-to-executor;
                    block @fauxlden/candidate/0/lead "Candidate zero lead" {
                        port @fauxlden/candidate/0/lead/output;
                    }
                    block @fauxlden/candidate/0/location/0 "Candidate zero location zero" {
                        port @fauxlden/candidate/0/location/0/output;
                    }
                    block @fauxlden/candidate/0/location/1 "Candidate zero location one" {
                        port @fauxlden/candidate/0/location/1/output;
                    }
                    block @fauxlden/candidate/0/tube/frame/0 "Candidate zero frame zero" {
                        port @fauxlden/candidate/0/tube/frame/0/output;
                    }
                    block @fauxlden/candidate/0/tube/frame/1 "Candidate zero frame one" {
                        port @fauxlden/candidate/0/tube/frame/1/input;
                        port @fauxlden/candidate/0/tube/frame/1/output;
                    }
                    block @fauxlden/candidate/1/lead "Candidate one lead" {
                        port @fauxlden/candidate/1/lead/output;
                    }
                    block @fauxlden/candidate/1/location/0 "Candidate one location zero" {
                        port @fauxlden/candidate/1/location/0/output;
                    }
                    block @fauxlden/candidate/1/location/1 "Candidate one location one" {
                        port @fauxlden/candidate/1/location/1/output;
                    }
                    block @fauxlden/candidate/1/tube/frame/0 "Candidate one frame zero" {
                        port @fauxlden/candidate/1/tube/frame/0/output;
                    }
                    block @fauxlden/candidate/1/tube/frame/1 "Candidate one frame one" {
                        port @fauxlden/candidate/1/tube/frame/1/input;
                        port @fauxlden/candidate/1/tube/frame/1/output;
                    }
                    connection @fauxlden/candidate/0/tube/advance @fauxlden/candidate/0/tube/frame/0/output -> @fauxlden/candidate/0/tube/frame/1/input;
                    connection @fauxlden/candidate/1/tube/advance @fauxlden/candidate/1/tube/frame/0/output -> @fauxlden/candidate/1/tube/frame/1/input;
                    connection @fauxlden/candidate/0/frame-0-to-ranker @fauxlden/candidate/0/tube/frame/0/output -> @fauxlden/ranker/candidate-0-frame-0;
                    connection @fauxlden/candidate/0/frame-1-to-ranker @fauxlden/candidate/0/tube/frame/1/output -> @fauxlden/ranker/candidate-0-frame-1;
                    connection @fauxlden/candidate/1/frame-0-to-ranker @fauxlden/candidate/1/tube/frame/0/output -> @fauxlden/ranker/candidate-1-frame-0;
                    connection @fauxlden/candidate/1/frame-1-to-ranker @fauxlden/candidate/1/tube/frame/1/output -> @fauxlden/ranker/candidate-1-frame-1;
                    group @fauxlden/candidate/0/locations "Candidate zero locations" {
                        @fauxlden/candidate/0/location/0,
                        @fauxlden/candidate/0/location/1;
                    }
                    group @fauxlden/candidate/0/tube "Candidate zero spacetime tube" {
                        @fauxlden/candidate/0/tube/frame/0,
                        @fauxlden/candidate/0/tube/frame/1,
                        @fauxlden/candidate/0/tube/advance;
                    }
                    group @fauxlden/candidate/0 "Candidate zero" {
                        @fauxlden/candidate/0/lead,
                        @fauxlden/candidate/0/locations,
                        @fauxlden/candidate/0/tube;
                    }
                    group @fauxlden/candidate/1/locations "Candidate one locations" {
                        @fauxlden/candidate/1/location/0,
                        @fauxlden/candidate/1/location/1;
                    }
                    group @fauxlden/candidate/1/tube "Candidate one spacetime tube" {
                        @fauxlden/candidate/1/tube/frame/0,
                        @fauxlden/candidate/1/tube/frame/1,
                        @fauxlden/candidate/1/tube/advance;
                    }
                    group @fauxlden/candidate/1 "Candidate one" {
                        @fauxlden/candidate/1/lead,
                        @fauxlden/candidate/1/locations,
                        @fauxlden/candidate/1/tube;
                    }
                    group @fauxlden/candidates "Candidate plurality" {
                        @fauxlden/candidate/0,
                        @fauxlden/candidate/1;
                    }
                    group @fauxlden/hot-path "Fauxlden hot path" {
                        @fauxlden/encoder,
                        @fauxlden/ranker,
                        @fauxlden/selector,
                        @fauxlden/executor,
                        @fauxlden/candidates;
                        extensions {
                            extension "https://github.com/goldenwitch/grimoire/extension/training" training schema training @1.0.0 = {
                                objective: "candidate retrieval and selection",
                                optimizer: present("adamw"),
                                batch_size: present(32),
                                steps: present(1000),
                                phases: [],
                                trainable_targets: [ref(@fauxlden/ranker), ref(@fauxlden/selector)],
                                frozen_targets: [ref(@fauxlden/encoder)],
                                data_sources: ["synthetic observations"]
                            };
                            extension "https://github.com/goldenwitch/grimoire/extension/provenance" provenance schema provenance @1.0.0 = {
                                citations: ["synthetic/fauxlden"],
                                assumptions: ["candidate and location axes are separate"],
                                novelty: unclassified
                            };
                            extension "https://example.com/fauxlden/hot-path" facts schema notes @1.0.0 = {
                                optimizer_flag: "adamw",
                                gradient_intervention: "stop-on-selected-target",
                                seed: 17,
                                evidence_artifacts: ["synthetic-observation-1", "synthetic-score-1"],
                                bandwidth_cost: "4096 bytes",
                                intentionally_undefined: "tie-breaking policy"
                            };
                        }
                    }
                }
                decorate {
                    on @fauxlden/candidate/0/location/0 extension "https://github.com/goldenwitch/grimoire/extension/placement" placement schema placement @1.0.0 = { location: "site-a" };
                    on @fauxlden/candidate/0/location/1 extension "https://github.com/goldenwitch/grimoire/extension/placement" placement schema placement @1.0.0 = { location: "site-b" };
                    on @fauxlden/candidate/1/location/0 extension "https://github.com/goldenwitch/grimoire/extension/placement" placement schema placement @1.0.0 = { location: "site-c" };
                    on @fauxlden/candidate/1/location/1 extension "https://github.com/goldenwitch/grimoire/extension/placement" placement schema placement @1.0.0 = { location: "site-d" };
                    on @fauxlden/executor extension "https://github.com/goldenwitch/grimoire/extension/execution" execution schema execution @1.0.0 = {
                        regime: closed-loop,
                        horizon: present(1),
                        rate: absent,
                        external_consumer: yes
                    };
                }
            }
        }
    }
"#;

fn address(value: &str) -> grimoire::Address {
    grimoire::Address::parse(value).unwrap_or_else(|error| panic!("{error}"))
}

fn schemas() -> Vec<Schema> {
    prototype_schemas().unwrap_or_else(|error| panic!("{error}"))
}

fn parsed() -> grimoire::Description {
    parse_description(FAUXLDEN_RETRIEVER).unwrap_or_else(|error| panic!("{error}"))
}

fn members(description: &grimoire::FinalizedReprojection, group: &str) -> Vec<grimoire::Address> {
    let Element::Group(group) = &description.structural.elements[&address(group)] else {
        panic!("expected group")
    };
    group.members.clone()
}

#[test]
fn fauxlden_consumes_dense_facts_without_new_core_kinds() {
    let description = parsed();
    validate_description(&description, &schemas())
        .unwrap_or_else(|errors| panic!("validation errors: {errors:?}"));
    let result =
        evaluate_layer(&description, "retrieval").unwrap_or_else(|error| panic!("{error}"));

    let candidate_members = members(&result, "@fauxlden/candidates");
    assert_eq!(candidate_members.len(), 2);
    for candidate in ["@fauxlden/candidate/0", "@fauxlden/candidate/1"] {
        assert_eq!(members(&result, candidate).len(), 3);
        let tube = format!("{candidate}/tube");
        assert_eq!(members(&result, &tube).len(), 3);
        let locations = format!("{candidate}/locations");
        assert_eq!(members(&result, &locations).len(), 2);
    }

    let ranker = &result.structural.elements[&address("@fauxlden/ranker")];
    let Element::Block(ranker) = ranker else {
        panic!("expected ranker block")
    };
    assert_eq!(ranker.ports.len(), 6);
    let selector = &result.structural.elements[&address("@fauxlden/selector")];
    let Element::Block(selector) = selector else {
        panic!("expected selector block")
    };
    assert_eq!(
        selector
            .ports
            .keys()
            .filter(|port| port.as_str().ends_with("/selected"))
            .count(),
        1
    );
    assert!(matches!(
        result.structural.elements[&address("@fauxlden/selector-to-executor")],
        Element::Connection(_)
    ));

    let hot_path = &result.structural.elements[&address("@fauxlden/hot-path")];
    let Element::Group(hot_path) = hot_path else {
        panic!("expected hot path group")
    };
    assert_eq!(hot_path.extensions.len(), 3);
    assert!(
        hot_path
            .extensions
            .iter()
            .any(|extension| { matches!(extension.value, ExtensionValue::Opaque(_)) })
    );
    assert!(
        hot_path
            .extensions
            .iter()
            .any(|extension| { extension.name == "training" && extension.schema == "training" })
    );
    assert!(
        hot_path.extensions.iter().any(|extension| {
            extension.name == "provenance" && extension.schema == "provenance"
        })
    );
}

#[test]
fn fauxlden_preserves_multi_location_placement_and_static_execution_boundary() {
    let description = parsed();
    let result =
        evaluate_layer(&description, "retrieval").unwrap_or_else(|error| panic!("{error}"));
    let placement =
        Placement::from_decorations(&result.decorations).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        placement.assigned_location(&address("@fauxlden/candidate/0/location/0")),
        Some("site-a")
    );
    assert_eq!(
        placement.assigned_location(&address("@fauxlden/candidate/0/location/1")),
        Some("site-b")
    );
    assert_eq!(
        placement.assigned_location(&address("@fauxlden/candidate/1/location/0")),
        Some("site-c")
    );
    assert_eq!(
        placement.assigned_location(&address("@fauxlden/candidate/1/location/1")),
        Some("site-d")
    );
    assert_eq!(result.decorations.len(), 5);
    assert!(result.decorations.iter().any(|decoration| {
        decoration.target == address("@fauxlden/executor")
            && decoration.parameter.schema == "execution"
    }));
}

#[test]
fn fauxlden_unknown_facts_round_trip_as_opaque_data() {
    let description = parsed();
    let serialized = serialize_description(&description).unwrap_or_else(|error| panic!("{error}"));
    assert!(serialized.contains("intentionally_undefined"));
    let reparsed = parse_description(&serialized).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(reparsed, description);
    assert_eq!(serialize_description(&reparsed).unwrap(), serialized);
}

#[test]
fn fauxlden_uses_only_existing_element_kinds() {
    let description = parsed();
    validate_description(&description, &schemas())
        .unwrap_or_else(|errors| panic!("validation errors: {errors:?}"));
    assert_eq!(description.core.blocks.len(), 5);
    assert_eq!(description.core.connections.len(), 5);
    assert_eq!(description.core.groups.len(), 1);
    let result =
        evaluate_layer(&description, "retrieval").unwrap_or_else(|error| panic!("{error}"));
    assert!(result.structural.elements.values().all(|element| {
        matches!(
            element,
            Element::Description(_)
                | Element::Block(_)
                | Element::Port(_)
                | Element::Connection(_)
                | Element::Group(_)
        )
    }));
}
