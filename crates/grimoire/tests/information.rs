use grimoire::{
    BayesianSummary, Channel, ChannelObservation, ChannelPosterior, ChannelScenario, ClaimEstimate,
    CredibleInterval, Distribution, InformationClaim, InformationDenominator, InformationError,
    InformationQuantity, JointSource, PosteriorSamples, RouteAllocationClaim, RouteShare,
    data_processing_holds,
};

mod common;
use common::{address, binary_source};

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-10, "{actual} != {expected}");
}

#[test]
fn identity_channel_preserves_one_bit() {
    let source = binary_source();
    let channel = Channel::identity(2).unwrap_or_else(|error| panic!("{error}"));
    assert_close(source.entropy_bits(), 1.0);
    assert_close(
        channel
            .mutual_information_bits(&source)
            .unwrap_or_else(|error| panic!("{error}")),
        1.0,
    );
    assert_close(
        channel
            .retention_fraction(&source)
            .unwrap_or_else(|error| panic!("{error}")),
        1.0,
    );
}

#[test]
fn constant_channel_preserves_no_information() {
    let source = binary_source();
    let channel = Channel::deterministic(vec![0, 0], 1).unwrap_or_else(|error| panic!("{error}"));
    assert_close(
        channel
            .mutual_information_bits(&source)
            .unwrap_or_else(|error| panic!("{error}")),
        0.0,
    );
    assert_close(
        channel
            .retention_fraction(&source)
            .unwrap_or_else(|error| panic!("{error}")),
        0.0,
    );
}

#[test]
fn noisy_channel_reduces_mutual_information() {
    let source = binary_source();
    let channel = Channel::new(vec![vec![0.9, 0.1], vec![0.1, 0.9]])
        .unwrap_or_else(|error| panic!("{error}"));
    let mutual_information = channel
        .mutual_information_bits(&source)
        .unwrap_or_else(|error| panic!("{error}"));
    assert!(mutual_information > 0.5);
    assert!(mutual_information < source.entropy_bits());
    assert!(
        channel
            .retention_fraction(&source)
            .unwrap_or_else(|error| panic!("{error}"))
            < 1.0
    );
}

#[test]
fn channel_composition_is_associative() {
    let first = Channel::new(vec![vec![0.8, 0.2], vec![0.2, 0.8]])
        .unwrap_or_else(|error| panic!("{error}"));
    let second = Channel::new(vec![vec![0.7, 0.3], vec![0.4, 0.6]])
        .unwrap_or_else(|error| panic!("{error}"));
    let third = Channel::new(vec![vec![0.6, 0.4], vec![0.1, 0.9]])
        .unwrap_or_else(|error| panic!("{error}"));
    let left = first
        .compose(&second)
        .and_then(|channel| channel.compose(&third))
        .unwrap_or_else(|error| panic!("{error}"));
    let right = second
        .compose(&third)
        .and_then(|channel| first.compose(&channel))
        .unwrap_or_else(|error| panic!("{error}"));
    for source_index in 0..left.source_cardinality() {
        for target_index in 0..left.target_cardinality() {
            assert_close(
                left.row(source_index).unwrap()[target_index],
                right.row(source_index).unwrap()[target_index],
            );
        }
    }
}

#[test]
fn data_processing_holds_after_a_channel() {
    let source = binary_source();
    let first = Channel::identity(2).unwrap_or_else(|error| panic!("{error}"));
    let second = Channel::new(vec![vec![0.9, 0.1], vec![0.1, 0.9]])
        .unwrap_or_else(|error| panic!("{error}"));
    assert!(
        data_processing_holds(&source, &first, &second, 1e-10)
            .unwrap_or_else(|error| panic!("{error}"))
    );
}

#[test]
fn joint_branch_information_is_not_the_sum_of_marginals() {
    let source = binary_source();
    let branch = Channel::identity(2).unwrap_or_else(|error| panic!("{error}"));
    let joint = branch
        .conditionally_independent_branch(&branch)
        .unwrap_or_else(|error| panic!("{error}"));
    let marginal_information = 2.0
        * branch
            .mutual_information_bits(&source)
            .unwrap_or_else(|error| panic!("{error}"));
    let joint_information = joint
        .mutual_information_bits(&source)
        .unwrap_or_else(|error| panic!("{error}"));
    assert_close(joint_information, 1.0);
    assert!(joint_information < marginal_information);
}

#[test]
fn joint_source_keeps_correlated_side_inputs_explicit() {
    let joint_source = JointSource::new(
        vec![address("@visual"), address("@action")],
        vec![2, 2],
        Distribution::new(vec![0.45, 0.05, 0.05, 0.45]).unwrap_or_else(|error| panic!("{error}")),
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let channel =
        Channel::deterministic(vec![0, 1, 0, 1], 2).unwrap_or_else(|error| panic!("{error}"));
    let visual = joint_source
        .component_distribution(&address("@visual"))
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(visual.probabilities(), &[0.5, 0.5]);
    let information = joint_source
        .mutual_information_bits(&channel, &address("@visual"))
        .unwrap_or_else(|error| panic!("{error}"));
    assert!(information > 0.4);
    assert!(information < 0.7);
    let retention = joint_source
        .retention_fraction(&channel, &address("@visual"))
        .unwrap_or_else(|error| panic!("{error}"));
    assert!(retention > 0.4);
    assert!(retention < 0.7);
}

#[test]
fn posterior_channels_produce_credible_decisions() {
    let source = binary_source();
    let identity = Channel::identity(2).unwrap_or_else(|error| panic!("{error}"));
    let constant = Channel::deterministic(vec![0, 0], 1).unwrap_or_else(|error| panic!("{error}"));
    let posterior = ChannelPosterior::new(vec![
        ChannelScenario {
            source: source.clone(),
            channel: identity,
            weight: 0.75,
        },
        ChannelScenario {
            source,
            channel: constant,
            weight: 0.25,
        },
    ])
    .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(posterior.scenario_count(), 2);
    let information = posterior
        .mutual_information_posterior()
        .unwrap_or_else(|error| panic!("{error}"));
    assert_close(information.posterior_mean(), 0.75);
    assert_close(information.probability_at_least(0.5), 0.75);
    let summary = information
        .summarize(0.9, 0.5)
        .unwrap_or_else(|error| panic!("{error}"));
    assert_close(summary.estimate, 0.75);
    assert_close(summary.probability_at_least, 0.75);
    assert_close(summary.interval.lower, 0.0);
    assert_close(summary.interval.upper, 1.0);
}

#[test]
fn posterior_update_reweights_channel_hypotheses_from_observations() {
    let source = binary_source();
    let prior = ChannelPosterior::new(vec![
        ChannelScenario {
            source: source.clone(),
            channel: Channel::identity(2).unwrap_or_else(|error| panic!("{error}")),
            weight: 0.6,
        },
        ChannelScenario {
            source,
            channel: Channel::deterministic(vec![1, 0], 2)
                .unwrap_or_else(|error| panic!("{error}")),
            weight: 0.4,
        },
    ])
    .unwrap_or_else(|error| panic!("{error}"));
    let updated = prior
        .update(&[ChannelObservation::new(0, 0), ChannelObservation::new(1, 1)])
        .unwrap_or_else(|error| panic!("{error}"));
    let weights = updated.scenario_weights();
    assert_close(weights[0], 1.0);
    assert_close(weights[1], 0.0);
    let posterior = updated
        .mutual_information_posterior()
        .unwrap_or_else(|error| panic!("{error}"));
    assert_close(posterior.posterior_mean(), 1.0);
    assert_close(posterior.probability_at_least(0.9), 1.0);
}

#[test]
fn empty_evidence_preserves_the_prior_and_impossible_evidence_fails() {
    let source = binary_source();
    let prior = ChannelPosterior::new(vec![ChannelScenario {
        source,
        channel: Channel::identity(2).unwrap_or_else(|error| panic!("{error}")),
        weight: 1.0,
    }])
    .unwrap_or_else(|error| panic!("{error}"));
    let unchanged = prior.update(&[]).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(unchanged.scenario_weights(), vec![1.0]);
    let error = unchanged
        .update(&[ChannelObservation::new(0, 2)])
        .expect_err("impossible observation should collapse the posterior");
    assert!(matches!(
        error,
        grimoire::InformationError::ImpossibleObservations { count: 1 }
    ));
}

#[test]
fn posterior_samples_support_exact_claims() {
    let exact = PosteriorSamples::exact(0.75).unwrap_or_else(|error| panic!("{error}"));
    let interval = exact
        .credible_interval(0.95)
        .unwrap_or_else(|error| panic!("{error}"));
    assert_close(interval.lower, 0.75);
    assert_close(interval.upper, 0.75);
    assert_close(exact.probability_at_least(0.5), 1.0);
}

#[test]
fn invalid_probability_and_posterior_shapes_fail_visibly() {
    assert!(matches!(
        Distribution::new(vec![0.5]),
        Err(InformationError::ProbabilitySum { .. })
    ));
    assert!(matches!(
        Distribution::new(vec![-0.1, 1.1]),
        Err(InformationError::InvalidProbability { .. })
    ));
    assert!(matches!(
        Channel::new(vec![vec![1.0], vec![0.5, 0.5]]),
        Err(InformationError::RowWidthMismatch { .. })
    ));
    assert!(matches!(
        PosteriorSamples::new(vec![0.5], vec![0.5]),
        Err(InformationError::ProbabilitySum { .. })
    ));
}

#[test]
fn zero_entropy_and_dimension_mismatch_are_distinct_errors() {
    let source = Distribution::new(vec![1.0]).unwrap_or_else(|error| panic!("{error}"));
    let channel = Channel::identity(1).unwrap_or_else(|error| panic!("{error}"));
    assert!(matches!(
        channel.retention_fraction(&source),
        Err(InformationError::ZeroEntropy)
    ));
    let binary = binary_source();
    assert!(matches!(
        channel.mutual_information_bits(&binary),
        Err(InformationError::DimensionMismatch { .. })
    ));
}

#[test]
fn invalid_credible_level_fails_visibly() {
    let samples = PosteriorSamples::exact(1.0).unwrap_or_else(|error| panic!("{error}"));
    assert!(matches!(
        samples.credible_interval(0.0),
        Err(InformationError::InvalidCredibility { .. })
    ));
}

#[test]
fn information_claims_bind_estimates_to_source_and_terminals() {
    let exact = InformationClaim::exact(
        address("@input/out"),
        vec![address("@head/out")],
        InformationQuantity::MutualInformation,
        None,
        0.75,
        "finite-channel".to_owned(),
        "synthetic law fixture".to_owned(),
    )
    .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(exact.source, address("@input/out"));
    assert_eq!(exact.terminals, vec![address("@head/out")]);
    assert!(matches!(exact.estimate, ClaimEstimate::Exact(value) if value == 0.75));

    let summary = BayesianSummary {
        estimate: 0.75,
        interval: CredibleInterval {
            lower: 0.0,
            upper: 1.0,
            credibility: 0.9,
        },
        threshold: 0.5,
        probability_at_least: 0.75,
    };
    let bayesian = InformationClaim::bayesian(
        address("@input/out"),
        vec![address("@head/out"), address("@decoder/out")],
        InformationQuantity::RetentionFraction,
        Some(InformationDenominator::SourceEntropyBits(1.0)),
        summary,
        "posterior-scenarios".to_owned(),
        "finite posterior fixture".to_owned(),
    )
    .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(bayesian.terminals.len(), 2);
    assert!(matches!(bayesian.estimate, ClaimEstimate::Bayesian(_)));
}

#[test]
fn retention_claims_require_a_positive_source_entropy_denominator() {
    let missing = InformationClaim::exact(
        address("@input/out"),
        vec![address("@head/out")],
        InformationQuantity::RetentionFraction,
        None,
        0.75,
        "finite-channel".to_owned(),
        "fixture".to_owned(),
    );
    assert!(matches!(missing, Err(InformationError::MissingDenominator)));

    let unexpected = InformationClaim::exact(
        address("@input/out"),
        vec![address("@head/out")],
        InformationQuantity::MutualInformation,
        Some(InformationDenominator::SourceEntropyBits(1.0)),
        0.75,
        "finite-channel".to_owned(),
        "fixture".to_owned(),
    );
    assert!(matches!(
        unexpected,
        Err(InformationError::UnexpectedDenominator)
    ));
}

#[test]
fn route_allocation_claims_require_a_declared_partition_and_sum() {
    let claim = RouteAllocationClaim::new(
        address("@input/out"),
        InformationDenominator::SourceEntropyBits(1.0),
        "two terminal routes".to_owned(),
        "conditional-ablation".to_owned(),
        vec![
            RouteShare {
                route: vec![address("@left")],
                estimate: ClaimEstimate::Exact(0.6),
            },
            RouteShare {
                route: vec![address("@right")],
                estimate: ClaimEstimate::Exact(0.4),
            },
        ],
    )
    .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(claim.shares.len(), 2);

    let invalid = RouteAllocationClaim::new(
        address("@input/out"),
        InformationDenominator::SourceEntropyBits(1.0),
        "two terminal routes".to_owned(),
        "conditional-ablation".to_owned(),
        vec![
            RouteShare {
                route: vec![address("@left")],
                estimate: ClaimEstimate::Exact(0.6),
            },
            RouteShare {
                route: vec![address("@right")],
                estimate: ClaimEstimate::Exact(0.5),
            },
        ],
    );
    assert!(matches!(
        invalid,
        Err(InformationError::RouteSharesDoNotSum { .. })
    ));
}

#[test]
fn claim_validation_rejects_duplicate_terminals_and_bad_posterior_intervals() {
    let duplicate = InformationClaim::exact(
        address("@input/out"),
        vec![address("@head/out"), address("@head/out")],
        InformationQuantity::MutualInformation,
        None,
        0.75,
        "finite-channel".to_owned(),
        "fixture".to_owned(),
    );
    assert!(matches!(
        duplicate,
        Err(InformationError::DuplicateClaimTerminal(_))
    ));

    let bad_summary = InformationClaim::bayesian(
        address("@input/out"),
        vec![address("@head/out")],
        InformationQuantity::MutualInformation,
        None,
        BayesianSummary {
            estimate: 0.8,
            interval: CredibleInterval {
                lower: 0.1,
                upper: 0.2,
                credibility: 0.9,
            },
            threshold: 0.5,
            probability_at_least: 0.8,
        },
        "posterior".to_owned(),
        "fixture".to_owned(),
    );
    assert!(matches!(
        bad_summary,
        Err(InformationError::InvalidClaimEstimate { .. })
    ));
}
