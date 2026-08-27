use grimoire::{
    Channel, ChannelGraph, ChannelLink, ChannelNode, ClaimEstimate, Distribution, InformationError,
    JointSource, evaluate_layer, parse_description, prototype_schemas, validate_description,
};

fn address(value: &str) -> grimoire::Address {
    grimoire::Address::parse(value).unwrap_or_else(|error| panic!("{error}"))
}

fn binary_source() -> Distribution {
    Distribution::uniform(2).unwrap_or_else(|error| panic!("{error}"))
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

fn link(source: &str, destination: &str) -> ChannelLink {
    ChannelLink {
        source: address(source),
        destination: address(destination),
    }
}

const DESCRIPTION: &str = r#"
    grimoire 1.0.0
    description @d "channel graph" {
        core-spec 1.0.0;
        core {
            block @source "Source" { port @source/out; }
            block @encoder "Encoder" {
                port @encoder/in;
                port @encoder/out;
            }
            block @head "Head" {
                port @head/in;
                port @head/out;
            }
            connection @source-to-encoder @source/out -> @encoder/in;
            connection @encoder-to-head @encoder/out -> @head/in;
        }
        layer "forward" {
            inputs { core };
            consumes { projection-language 1.0.0; schemas { } }
            projection {
                select {
                    use @source, @source/out, @encoder, @encoder/in, @encoder/out,
                       @head, @head/in, @head/out, @source-to-encoder, @encoder-to-head;
                }
            }
        }
    }
"#;

#[test]
fn addressed_dag_composes_channels_to_a_terminal() {
    let graph = ChannelGraph::new(
        vec![node(
            "@encoder",
            "@encoder",
            &["@encoder/in"],
            "@encoder/out",
            Channel::identity(2).unwrap(),
        )],
        vec![link("@source/out", "@encoder/in")],
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let channel = graph
        .channel_to_terminal(
            &address("@source/out"),
            &binary_source(),
            &address("@encoder/out"),
        )
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(channel.source_cardinality(), 2);
    assert_eq!(channel.target_cardinality(), 2);
    assert_eq!(
        channel
            .mutual_information_bits(&binary_source())
            .unwrap_or_else(|error| panic!("{error}")),
        1.0
    );
}

#[test]
fn evaluated_grimoire_reprojection_supplies_channel_wiring() {
    let description = parse_description(DESCRIPTION).unwrap_or_else(|error| panic!("{error}"));
    validate_description(
        &description,
        &prototype_schemas().unwrap_or_else(|error| panic!("{error}")),
    )
    .unwrap_or_else(|errors| panic!("validation errors: {errors:?}"));
    let reprojection = evaluate_layer(&description, "forward")
        .unwrap_or_else(|error| panic!("{error}"))
        .structural;
    let identity = Channel::identity(2).unwrap_or_else(|error| panic!("{error}"));
    let graph = ChannelGraph::from_reprojection(
        &reprojection,
        vec![
            node(
                "@encoder-channel",
                "@encoder",
                &["@encoder/in"],
                "@encoder/out",
                identity.clone(),
            ),
            node(
                "@head-channel",
                "@head",
                &["@head/in"],
                "@head/out",
                identity,
            ),
        ],
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let claim = graph
        .information_claim(
            &address("@source/out"),
            &binary_source(),
            &address("@head/out"),
            "finite-channel".to_owned(),
            "evaluated reprojection fixture".to_owned(),
        )
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(claim.source, address("@source/out"));
    assert_eq!(claim.terminals, vec![address("@head/out")]);
    assert!(matches!(claim.estimate, ClaimEstimate::Exact(value) if value == 1.0));
}

#[test]
fn layer_bound_channel_queries_reject_hidden_terminals() {
    let description = parse_description(DESCRIPTION).unwrap_or_else(|error| panic!("{error}"));
    let identity = Channel::identity(2).unwrap_or_else(|error| panic!("{error}"));
    let graph = ChannelGraph::from_layer(
        &description,
        "forward",
        vec![
            node(
                "@encoder-channel",
                "@encoder",
                &["@encoder/in"],
                "@encoder/out",
                identity.clone(),
            ),
            node(
                "@head-channel",
                "@head",
                &["@head/in"],
                "@head/out",
                identity,
            ),
        ],
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let error = graph
        .channel_to_terminal(
            &address("@source/out"),
            &binary_source(),
            &address("@hidden/out"),
        )
        .expect_err("hidden terminal should fail");
    assert!(
        matches!(error, InformationError::UnvisiblePort(port) if port == address("@hidden/out"))
    );
}

#[test]
fn finite_horizon_recurrence_is_an_acyclic_addressed_graph() {
    let graph = ChannelGraph::new(
        vec![
            node(
                "@dynamics/0",
                "@dynamics/0",
                &["@state/0/in"],
                "@state/0/out",
                Channel::identity(2).unwrap(),
            ),
            node(
                "@dynamics/1",
                "@dynamics/1",
                &["@state/1/in"],
                "@state/1/out",
                Channel::new(vec![vec![0.9, 0.1], vec![0.1, 0.9]]).unwrap(),
            ),
        ],
        vec![
            link("@source/out", "@state/0/in"),
            link("@state/0/out", "@state/1/in"),
        ],
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let source = binary_source();
    let channel = graph
        .channel_to_terminal(&address("@source/out"), &source, &address("@state/1/out"))
        .unwrap_or_else(|error| panic!("{error}"));
    assert!(
        channel
            .mutual_information_bits(&source)
            .unwrap_or_else(|error| panic!("{error}"))
            < 1.0
    );
}

#[test]
fn action_conditioned_graph_keeps_visual_information_distinct_from_side_inputs() {
    let graph = ChannelGraph::new(
        vec![node(
            "@ac/predictor",
            "@ac/predictor",
            &["@ac/visual", "@ac/action", "@ac/state"],
            "@ac/output",
            Channel::deterministic(vec![0, 0, 1, 1, 0, 0, 1, 1], 2)
                .unwrap_or_else(|error| panic!("{error}")),
        )],
        vec![
            link("@visual/out", "@ac/visual"),
            link("@action/out", "@ac/action"),
            link("@state/out", "@ac/state"),
        ],
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let source = JointSource::new(
        vec![
            address("@visual/out"),
            address("@action/out"),
            address("@state/out"),
        ],
        vec![2, 2, 2],
        Distribution::new(vec![0.225, 0.225, 0.025, 0.025, 0.025, 0.025, 0.225, 0.225])
            .unwrap_or_else(|error| panic!("{error}")),
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let channel = graph
        .channel_to_terminal_with_joint_source(&source, &address("@ac/output"))
        .unwrap_or_else(|error| panic!("{error}"));
    let information = source
        .mutual_information_bits(&channel, &address("@visual/out"))
        .unwrap_or_else(|error| panic!("{error}"));
    assert!(information > 0.4);
    assert!(information < 0.7);
    let claim = graph
        .information_claim_with_joint_source(
            &source,
            &address("@visual/out"),
            &address("@ac/output"),
            "finite-joint-channel".to_owned(),
            "action-conditioned graph fixture".to_owned(),
        )
        .unwrap_or_else(|error| panic!("{error}"));
    assert!(matches!(claim.estimate, ClaimEstimate::Exact(value) if value == information));
}

#[test]
fn branch_terminals_use_joint_information_instead_of_summing_marginals() {
    let identity = Channel::identity(2).unwrap_or_else(|error| panic!("{error}"));
    let graph = ChannelGraph::new(
        vec![
            node(
                "@left",
                "@left",
                &["@left/in"],
                "@left/out",
                identity.clone(),
            ),
            node("@right", "@right", &["@right/in"], "@right/out", identity),
        ],
        vec![
            link("@source/out", "@left/in"),
            link("@source/out", "@right/in"),
        ],
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let source = binary_source();
    let joint = graph
        .joint_channel_to_terminals(
            &address("@source/out"),
            &source,
            &[address("@left/out"), address("@right/out")],
        )
        .unwrap_or_else(|error| panic!("{error}"));
    let claim = graph
        .joint_information_claim(
            &address("@source/out"),
            &source,
            &[address("@left/out"), address("@right/out")],
            "finite-channel".to_owned(),
            "branch fixture".to_owned(),
        )
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        joint
            .mutual_information_bits(&source)
            .unwrap_or_else(|error| panic!("{error}")),
        1.0
    );
    assert!(matches!(claim.estimate, ClaimEstimate::Exact(value) if value == 1.0));
}

#[test]
fn conditional_independence_is_explicit_at_a_merge() {
    let identity = Channel::identity(2).unwrap_or_else(|error| panic!("{error}"));
    let merge_channel =
        Channel::deterministic(vec![0, 0, 1, 1], 2).unwrap_or_else(|error| panic!("{error}"));
    let graph = ChannelGraph::new(
        vec![
            node(
                "@left",
                "@left",
                &["@left/in"],
                "@left/out",
                identity.clone(),
            ),
            node("@right", "@right", &["@right/in"], "@right/out", identity),
            node(
                "@merge",
                "@merge",
                &["@merge/left", "@merge/right"],
                "@merge/out",
                merge_channel,
            ),
        ],
        vec![
            link("@source/out", "@left/in"),
            link("@source/out", "@right/in"),
            link("@left/out", "@merge/left"),
            link("@right/out", "@merge/right"),
        ],
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let source = binary_source();
    let claim = graph
        .information_claim(
            &address("@source/out"),
            &source,
            &address("@merge/out"),
            "finite-channel".to_owned(),
            "merge fixture".to_owned(),
        )
        .unwrap_or_else(|error| panic!("{error}"));
    assert!(matches!(claim.estimate, ClaimEstimate::Exact(value) if value == 1.0));
}

#[test]
fn graph_reports_cycles_and_invalid_wiring() {
    let identity = Channel::identity(2).unwrap_or_else(|error| panic!("{error}"));
    let invalid = ChannelGraph::new(
        vec![
            node(
                "@first",
                "@first",
                &["@first/in"],
                "@first/out",
                identity.clone(),
            ),
            node(
                "@second",
                "@second",
                &["@second/in"],
                "@second/out",
                identity,
            ),
        ],
        vec![
            link("@source/out", "@first/in"),
            link("@first/out", "@second/in"),
            link("@second/out", "@unknown/in"),
        ],
    );
    assert!(matches!(
        invalid,
        Err(InformationError::UnknownDestinationPort(_))
    ));

    let cycle = ChannelGraph::new(
        vec![
            node(
                "@first",
                "@first",
                &["@first/in", "@first/feedback"],
                "@first/out",
                Channel::deterministic(vec![0, 1, 0, 1], 2).unwrap(),
            ),
            node(
                "@second",
                "@second",
                &["@second/in"],
                "@second/out",
                Channel::identity(2).unwrap(),
            ),
        ],
        vec![
            link("@source/out", "@first/in"),
            link("@first/out", "@second/in"),
            link("@second/out", "@first/feedback"),
        ],
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let error = cycle
        .channel_to_terminal(
            &address("@source/out"),
            &binary_source(),
            &address("@second/out"),
        )
        .expect_err("the terminal depends on a cycle");
    assert!(matches!(error, InformationError::CyclicGraph));
}
