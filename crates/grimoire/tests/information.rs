use grimoire::{
    Channel, ChannelPosterior, ChannelScenario, Distribution, InformationError, PosteriorSamples,
    data_processing_holds,
};

fn binary_source() -> Distribution {
    Distribution::uniform(2).unwrap_or_else(|error| panic!("{error}"))
}

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
