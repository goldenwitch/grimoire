use grimoire::{
    ResourceBundle, ResourceCharge, ResourceError, ResourceFlow, ResourceKind, ResourceModel,
    ResourceScenario, evaluate_layer, parse_description, validate_description,
};

mod common;
use common::{address, schemas};

const INDEXED_SEARCH: &str = r#"
    grimoire 1.0.0
    description @search "Indexed search resource flow" {
        core-spec 1.0.0;
        core {
            block @search/input "Query input" { port @search/input/out; }
            block @search/encoder "Query encoder" {
                port @search/encoder/in;
                port @search/encoder/out;
            }
            block @search/index "Vector index" { port @search/index/in; }
            block @search/fallback "Fallback scan" { port @search/fallback/in; }
            connection @search/input-to-encoder @search/input/out -> @search/encoder/in;
            connection @search/encoder-to-index @search/encoder/out -> @search/index/in;
            connection @search/encoder-to-fallback @search/encoder/out -> @search/fallback/in;
            group @search/system "Search system" {
                @search/input,
                @search/encoder,
                @search/index,
                @search/fallback,
                @search/input-to-encoder,
                @search/encoder-to-index,
                @search/encoder-to-fallback;
            }
        }
        layer "search" {
            inputs { core };
            consumes { projection-language 1.0.0; schemas { } }
            projection { select { use @search/system; } }
        }
    }
"#;

const VJEPA_FRONTIERS: &str = r#"
    grimoire 1.0.0
    description @architectures "V-JEPA and frontier resource cases" {
        core-spec 1.0.0;
        core {
            block @vjepa2/encoder "Shared V-JEPA encoder" { port @vjepa2/encoder/out; }
            block @vjepa2/pretraining/predictor "Pretraining predictor" { port @vjepa2/pretraining/predictor/in; }
            block @vjepa2/action "Robot action" { port @vjepa2/action/out; }
            block @vjepa2/state "End-effector state" { port @vjepa2/state/out; }
            block @vjepa2/ac/predictor "Action-conditioned predictor" {
                port @vjepa2/ac/predictor/visual;
                port @vjepa2/ac/predictor/action;
                port @vjepa2/ac/predictor/state;
            }
            block @frontier/bridge "Visual-language bridge" {
                port @frontier/bridge/in;
                port @frontier/bridge/out;
            }
            block @frontier/transformer "Shared multimodal transformer" {
                port @frontier/transformer/in;
                port @frontier/transformer/out;
            }
            block @frontier/speech "Streaming speech decoder" { port @frontier/speech/in; }
            block @frontier/low-bit "Low-bit operator" { port @frontier/low-bit/in; }
            connection @flow/pretraining @vjepa2/encoder/out -> @vjepa2/pretraining/predictor/in;
            connection @flow/ac/visual @vjepa2/encoder/out -> @vjepa2/ac/predictor/visual;
            connection @flow/ac/action @vjepa2/action/out -> @vjepa2/ac/predictor/action;
            connection @flow/ac/state @vjepa2/state/out -> @vjepa2/ac/predictor/state;
            connection @flow/bridge @vjepa2/encoder/out -> @frontier/bridge/in;
            connection @flow/unified @frontier/bridge/out -> @frontier/transformer/in;
            connection @flow/speech @frontier/transformer/out -> @frontier/speech/in;
            connection @flow/low-bit @frontier/transformer/out -> @frontier/low-bit/in;
            group @architectures/system "Candidate architecture cases" {
                @vjepa2/encoder,
                @vjepa2/pretraining/predictor,
                @vjepa2/action,
                @vjepa2/state,
                @vjepa2/ac/predictor,
                @frontier/bridge,
                @frontier/transformer,
                @frontier/speech,
                @frontier/low-bit,
                @flow/pretraining,
                @flow/ac/visual,
                @flow/ac/action,
                @flow/ac/state,
                @flow/bridge,
                @flow/unified,
                @flow/speech,
                @flow/low-bit;
            }
        }
        layer "architecture-cases" {
            inputs { core };
            consumes { projection-language 1.0.0; schemas { } }
            projection { select { use @architectures/system; } }
        }
    }
"#;

fn bundle(entries: Vec<(ResourceKind, u64)>) -> ResourceBundle {
    ResourceBundle::new(entries).unwrap_or_else(|error| panic!("{error}"))
}

fn flow(
    relation: &str,
    source: &str,
    destination: &str,
    resources: ResourceBundle,
) -> ResourceFlow {
    ResourceFlow::new(
        address(relation),
        address(source),
        address(destination),
        resources,
    )
    .unwrap_or_else(|error| panic!("{error}"))
}

fn charge(target: &str, resources: ResourceBundle) -> ResourceCharge {
    ResourceCharge::new(address(target), resources).unwrap_or_else(|error| panic!("{error}"))
}

#[test]
fn indexed_search_tracks_non_fungible_resources_by_workload() {
    let description = parse_description(INDEXED_SEARCH).unwrap_or_else(|error| panic!("{error}"));
    validate_description(&description, &schemas())
        .unwrap_or_else(|errors| panic!("validation errors: {errors:?}"));
    let reprojection = evaluate_layer(&description, "search")
        .unwrap_or_else(|error| panic!("{error}"))
        .structural;

    let hit = ResourceScenario::new(
        "indexed-hit",
        0.75,
        "query resolves in the index",
        vec![flow(
            "@search/input-to-encoder",
            "@search/input/out",
            "@search/encoder/in",
            bundle(vec![(ResourceKind::Bytes, 4096)]),
        )],
        vec![
            charge(
                "@search/encoder",
                bundle(vec![
                    (ResourceKind::FlopWork, 1_000_000),
                    (ResourceKind::MemoryBytes, 8192),
                ]),
            ),
            charge(
                "@search/index",
                bundle(vec![(ResourceKind::LatencyNanoseconds, 500_000)]),
            ),
        ],
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let fallback = ResourceScenario::new(
        "fallback-scan",
        0.25,
        "query misses the index",
        vec![
            flow(
                "@search/input-to-encoder",
                "@search/input/out",
                "@search/encoder/in",
                bundle(vec![(ResourceKind::Bytes, 4096)]),
            ),
            flow(
                "@search/encoder-to-fallback",
                "@search/encoder/out",
                "@search/fallback/in",
                bundle(vec![(ResourceKind::Bytes, 2048)]),
            ),
        ],
        vec![charge(
            "@search/encoder",
            bundle(vec![(ResourceKind::FlopWork, 1_000_000)]),
        )],
    )
    .unwrap_or_else(|error| panic!("{error}"));

    let model = ResourceModel::new(vec![hit, fallback]).unwrap_or_else(|error| panic!("{error}"));
    let report = model
        .evaluate(&reprojection)
        .unwrap_or_else(|error| panic!("{error}"));

    assert_eq!(report.scenario_probability("indexed-hit"), Some(0.75));
    assert_eq!(
        report.assumption("fallback-scan"),
        Some("query misses the index")
    );
    assert_eq!(
        report
            .flow(&address("@search/encoder-to-fallback"))
            .and_then(|estimate| estimate.quantity(ResourceKind::Bytes)),
        Some(512.0)
    );
    assert_eq!(report.total_flow(ResourceKind::Bytes), Some(4608.0));
    assert_eq!(
        report.total_charge(ResourceKind::FlopWork),
        Some(1_000_000.0)
    );
    assert_eq!(report.total_charge(ResourceKind::MemoryBytes), Some(6144.0));
    assert_eq!(report.total_charge(ResourceKind::Bytes), None);
}

#[test]
fn invalid_probabilities_and_graph_flows_fail_visibly() {
    let invalid_probability =
        ResourceScenario::new("invalid", 1.5, "bad assumption", Vec::new(), Vec::new())
            .expect_err("probability outside the unit interval should fail");
    assert!(matches!(
        invalid_probability,
        ResourceError::InvalidProbability { .. }
    ));

    let first = ResourceScenario::new("first", 0.4, "first path", Vec::new(), Vec::new())
        .unwrap_or_else(|error| panic!("{error}"));
    let second = ResourceScenario::new("second", 0.4, "second path", Vec::new(), Vec::new())
        .unwrap_or_else(|error| panic!("{error}"));
    assert!(matches!(
        ResourceModel::new(vec![first, second]),
        Err(ResourceError::ProbabilityTotal { .. })
    ));

    let description = parse_description(INDEXED_SEARCH).unwrap_or_else(|error| panic!("{error}"));
    let reprojection = evaluate_layer(&description, "search")
        .unwrap_or_else(|error| panic!("{error}"))
        .structural;
    let scenario = ResourceScenario::new(
        "bad-flow",
        1.0,
        "endpoint mismatch",
        vec![flow(
            "@search/input-to-encoder",
            "@search/encoder/out",
            "@search/encoder/in",
            bundle(vec![(ResourceKind::Bytes, 1)]),
        )],
        Vec::new(),
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let model = ResourceModel::new(vec![scenario]).unwrap_or_else(|error| panic!("{error}"));
    assert!(matches!(
        model.evaluate(&reprojection),
        Err(ResourceError::FlowEndpointMismatch { .. })
    ));

    assert!(matches!(
        ResourceBundle::new(vec![(ResourceKind::Bytes, 1), (ResourceKind::Bytes, 2),]),
        Err(ResourceError::DuplicateResourceKind(ResourceKind::Bytes))
    ));
}

#[test]
fn vjepa_and_frontier_cases_keep_resource_kinds_componentwise() {
    let description = parse_description(VJEPA_FRONTIERS).unwrap_or_else(|error| panic!("{error}"));
    validate_description(&description, &schemas())
        .unwrap_or_else(|errors| panic!("validation errors: {errors:?}"));
    let reprojection = evaluate_layer(&description, "architecture-cases")
        .unwrap_or_else(|error| panic!("{error}"))
        .structural;

    let pretraining = ResourceScenario::new(
        "pretraining",
        0.4,
        "shared encoder predicts masked representations",
        vec![flow(
            "@flow/pretraining",
            "@vjepa2/encoder/out",
            "@vjepa2/pretraining/predictor/in",
            bundle(vec![(ResourceKind::Bytes, 2000)]),
        )],
        vec![charge(
            "@vjepa2/encoder",
            bundle(vec![(ResourceKind::FlopWork, 1_000_000)]),
        )],
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let action_conditioned = ResourceScenario::new(
        "action-conditioned",
        0.3,
        "V-JEPA 2-AC consumes visual, action, and state inputs",
        vec![
            flow(
                "@flow/ac/visual",
                "@vjepa2/encoder/out",
                "@vjepa2/ac/predictor/visual",
                bundle(vec![(ResourceKind::Bytes, 3000)]),
            ),
            flow(
                "@flow/ac/action",
                "@vjepa2/action/out",
                "@vjepa2/ac/predictor/action",
                bundle(vec![(ResourceKind::Bytes, 100)]),
            ),
            flow(
                "@flow/ac/state",
                "@vjepa2/state/out",
                "@vjepa2/ac/predictor/state",
                bundle(vec![(ResourceKind::Bytes, 100)]),
            ),
        ],
        vec![charge(
            "@vjepa2/ac/predictor",
            bundle(vec![(ResourceKind::FlopWork, 2_000_000)]),
        )],
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let bridge = ResourceScenario::new(
        "bridge",
        0.2,
        "visual tokens cross a bridge into a language core",
        vec![
            flow(
                "@flow/bridge",
                "@vjepa2/encoder/out",
                "@frontier/bridge/in",
                bundle(vec![(ResourceKind::Bytes, 4000)]),
            ),
            flow(
                "@flow/unified",
                "@frontier/bridge/out",
                "@frontier/transformer/in",
                bundle(vec![(ResourceKind::Bytes, 5000)]),
            ),
        ],
        vec![charge(
            "@frontier/transformer",
            bundle(vec![(ResourceKind::FlopWork, 3_000_000)]),
        )],
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let speech_low_bit = ResourceScenario::new(
        "speech-low-bit",
        0.1,
        "streaming speech and low-bit deployment are separate views",
        vec![
            flow(
                "@flow/speech",
                "@frontier/transformer/out",
                "@frontier/speech/in",
                bundle(vec![(ResourceKind::Bytes, 6000)]),
            ),
            flow(
                "@flow/low-bit",
                "@frontier/transformer/out",
                "@frontier/low-bit/in",
                bundle(vec![(ResourceKind::Bytes, 7000)]),
            ),
        ],
        vec![charge(
            "@frontier/low-bit",
            bundle(vec![(ResourceKind::MemoryBytes, 50_000)]),
        )],
    )
    .unwrap_or_else(|error| panic!("{error}"));

    let model = ResourceModel::new(vec![
        pretraining,
        action_conditioned,
        bridge,
        speech_low_bit,
    ])
    .unwrap_or_else(|error| panic!("{error}"));
    let report = model
        .evaluate(&reprojection)
        .unwrap_or_else(|error| panic!("{error}"));

    assert_eq!(
        report
            .flow(&address("@flow/ac/action"))
            .and_then(|estimate| estimate.quantity(ResourceKind::Bytes)),
        Some(30.0)
    );
    assert_eq!(report.total_flow(ResourceKind::Bytes), Some(4_860.0));
    assert_eq!(
        report.total_charge(ResourceKind::FlopWork),
        Some(1_600_000.0)
    );
    assert_eq!(
        report.total_charge(ResourceKind::MemoryBytes),
        Some(5_000.0)
    );
    assert_eq!(report.total_charge(ResourceKind::LatencyNanoseconds), None);
}
