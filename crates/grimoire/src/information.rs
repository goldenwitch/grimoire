use core::fmt;

const PROBABILITY_TOLERANCE: f64 = 1e-12;

#[derive(Clone, Debug, PartialEq)]
pub struct Distribution {
    probabilities: Vec<f64>,
}

impl Distribution {
    pub fn new(probabilities: Vec<f64>) -> Result<Self, InformationError> {
        if probabilities.is_empty() {
            return Err(InformationError::EmptyDistribution);
        }
        validate_probability_vector(&probabilities, "distribution")?;
        Ok(Self { probabilities })
    }

    pub fn uniform(cardinality: usize) -> Result<Self, InformationError> {
        if cardinality == 0 {
            return Err(InformationError::EmptyDistribution);
        }
        Self::new(vec![1.0 / cardinality as f64; cardinality])
    }

    #[must_use]
    pub fn cardinality(&self) -> usize {
        self.probabilities.len()
    }

    #[must_use]
    pub fn probabilities(&self) -> &[f64] {
        &self.probabilities
    }

    #[must_use]
    pub fn entropy_bits(&self) -> f64 {
        let entropy = self
            .probabilities
            .iter()
            .filter(|probability| **probability > 0.0)
            .map(|probability| -probability * probability.log2())
            .sum::<f64>();
        entropy.max(0.0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Channel {
    rows: Vec<Vec<f64>>,
    target_cardinality: usize,
}

impl Channel {
    pub fn new(rows: Vec<Vec<f64>>) -> Result<Self, InformationError> {
        if rows.is_empty() {
            return Err(InformationError::EmptyChannel);
        }
        let target_cardinality = rows[0].len();
        if target_cardinality == 0 {
            return Err(InformationError::EmptyChannel);
        }
        for (row_index, row) in rows.iter().enumerate() {
            if row.len() != target_cardinality {
                return Err(InformationError::RowWidthMismatch {
                    row: row_index,
                    expected: target_cardinality,
                    actual: row.len(),
                });
            }
            validate_probability_vector(row, "channel row")?;
        }
        Ok(Self {
            rows,
            target_cardinality,
        })
    }

    pub fn deterministic(
        mapping: Vec<usize>,
        target_cardinality: usize,
    ) -> Result<Self, InformationError> {
        if target_cardinality == 0 {
            return Err(InformationError::EmptyChannel);
        }
        let mut rows = Vec::with_capacity(mapping.len());
        for target_index in mapping {
            if target_index >= target_cardinality {
                return Err(InformationError::OutputIndex {
                    index: target_index,
                    cardinality: target_cardinality,
                });
            }
            let mut row = vec![0.0; target_cardinality];
            row[target_index] = 1.0;
            rows.push(row);
        }
        Self::new(rows)
    }

    pub fn identity(cardinality: usize) -> Result<Self, InformationError> {
        Self::deterministic((0..cardinality).collect(), cardinality)
    }

    #[must_use]
    pub fn source_cardinality(&self) -> usize {
        self.rows.len()
    }

    #[must_use]
    pub fn target_cardinality(&self) -> usize {
        self.target_cardinality
    }

    #[must_use]
    pub fn row(&self, source_index: usize) -> Option<&[f64]> {
        self.rows.get(source_index).map(Vec::as_slice)
    }

    pub fn output_distribution(
        &self,
        source: &Distribution,
    ) -> Result<Distribution, InformationError> {
        self.check_source(source)?;
        let mut probabilities = vec![0.0; self.target_cardinality];
        for (source_index, source_probability) in source.probabilities().iter().enumerate() {
            for (target_index, channel_probability) in self.rows[source_index].iter().enumerate() {
                probabilities[target_index] += source_probability * channel_probability;
            }
        }
        Distribution::new(probabilities)
    }

    pub fn compose(&self, next: &Self) -> Result<Self, InformationError> {
        if self.target_cardinality != next.source_cardinality() {
            return Err(InformationError::DimensionMismatch {
                context: "channel composition",
                expected: self.target_cardinality,
                actual: next.source_cardinality(),
            });
        }
        let mut rows = vec![vec![0.0; next.target_cardinality]; self.source_cardinality()];
        for (source_index, row) in self.rows.iter().enumerate() {
            for (middle_index, first_probability) in row.iter().enumerate() {
                for (target_index, second_probability) in next.rows[middle_index].iter().enumerate()
                {
                    rows[source_index][target_index] += first_probability * second_probability;
                }
            }
        }
        Self::new(rows)
    }

    pub fn conditionally_independent_branch(&self, other: &Self) -> Result<Self, InformationError> {
        if self.source_cardinality() != other.source_cardinality() {
            return Err(InformationError::DimensionMismatch {
                context: "conditional branch",
                expected: self.source_cardinality(),
                actual: other.source_cardinality(),
            });
        }
        let target_cardinality = self
            .target_cardinality
            .checked_mul(other.target_cardinality)
            .ok_or(InformationError::CardinalityOverflow)?;
        let mut rows = Vec::with_capacity(self.source_cardinality());
        for source_index in 0..self.source_cardinality() {
            let mut row = vec![0.0; target_cardinality];
            for (first_index, first_probability) in self.rows[source_index].iter().enumerate() {
                for (second_index, second_probability) in
                    other.rows[source_index].iter().enumerate()
                {
                    let joint_index = first_index * other.target_cardinality + second_index;
                    row[joint_index] = first_probability * second_probability;
                }
            }
            rows.push(row);
        }
        Self::new(rows)
    }

    pub fn mutual_information_bits(&self, source: &Distribution) -> Result<f64, InformationError> {
        let output = self.output_distribution(source)?;
        let mut mutual_information = 0.0;
        for (source_index, source_probability) in source.probabilities().iter().enumerate() {
            for (target_index, channel_probability) in self.rows[source_index].iter().enumerate() {
                let joint_probability = source_probability * channel_probability;
                if joint_probability <= 0.0 {
                    continue;
                }
                let denominator = source_probability * output.probabilities()[target_index];
                mutual_information += joint_probability * (joint_probability / denominator).log2();
            }
        }
        Ok(mutual_information.max(0.0))
    }

    pub fn retention_fraction(&self, source: &Distribution) -> Result<f64, InformationError> {
        let entropy = source.entropy_bits();
        if entropy <= PROBABILITY_TOLERANCE {
            return Err(InformationError::ZeroEntropy);
        }
        Ok((self.mutual_information_bits(source)? / entropy).clamp(0.0, 1.0))
    }

    fn check_source(&self, source: &Distribution) -> Result<(), InformationError> {
        if self.source_cardinality() != source.cardinality() {
            return Err(InformationError::DimensionMismatch {
                context: "source distribution",
                expected: self.source_cardinality(),
                actual: source.cardinality(),
            });
        }
        Ok(())
    }
}

pub fn data_processing_holds(
    source: &Distribution,
    first: &Channel,
    second: &Channel,
    tolerance: f64,
) -> Result<bool, InformationError> {
    if tolerance < 0.0 || !tolerance.is_finite() {
        return Err(InformationError::InvalidTolerance { value: tolerance });
    }
    let composed = first.compose(second)?;
    let before = first.mutual_information_bits(source)?;
    let after = composed.mutual_information_bits(source)?;
    Ok(after <= before + tolerance)
}

#[derive(Clone, Debug, PartialEq)]
pub struct PosteriorSamples {
    values: Vec<f64>,
    weights: Vec<f64>,
}

impl PosteriorSamples {
    pub fn new(values: Vec<f64>, weights: Vec<f64>) -> Result<Self, InformationError> {
        if values.is_empty() || values.len() != weights.len() {
            return Err(InformationError::PosteriorShapeMismatch {
                values: values.len(),
                weights: weights.len(),
            });
        }
        for (index, value) in values.iter().enumerate() {
            if !value.is_finite() {
                return Err(InformationError::InvalidPosteriorValue {
                    index,
                    value: *value,
                });
            }
        }
        validate_probability_vector(&weights, "posterior weights")?;
        Ok(Self { values, weights })
    }

    pub fn equal_weight(values: Vec<f64>) -> Result<Self, InformationError> {
        if values.is_empty() {
            return Err(InformationError::PosteriorShapeMismatch {
                values: 0,
                weights: 0,
            });
        }
        let weight = 1.0 / values.len() as f64;
        Self::new(values.clone(), vec![weight; values.len()])
    }

    pub fn exact(value: f64) -> Result<Self, InformationError> {
        Self::new(vec![value], vec![1.0])
    }

    #[must_use]
    pub fn values(&self) -> &[f64] {
        &self.values
    }

    #[must_use]
    pub fn weights(&self) -> &[f64] {
        &self.weights
    }

    #[must_use]
    pub fn posterior_mean(&self) -> f64 {
        self.values
            .iter()
            .zip(&self.weights)
            .map(|(value, weight)| value * weight)
            .sum()
    }

    pub fn credible_interval(
        &self,
        credibility: f64,
    ) -> Result<CredibleInterval, InformationError> {
        validate_credibility(credibility)?;
        let mut weighted_values: Vec<(f64, f64)> = self
            .values
            .iter()
            .copied()
            .zip(self.weights.iter().copied())
            .collect();
        weighted_values.sort_by(|left, right| left.0.total_cmp(&right.0));
        let tail = (1.0 - credibility) / 2.0;
        Ok(CredibleInterval {
            lower: weighted_quantile(&weighted_values, tail),
            upper: weighted_quantile(&weighted_values, 1.0 - tail),
            credibility,
        })
    }

    #[must_use]
    pub fn probability_at_least(&self, threshold: f64) -> f64 {
        self.values
            .iter()
            .zip(&self.weights)
            .filter(|(value, _)| **value >= threshold)
            .map(|(_, weight)| weight)
            .sum()
    }

    pub fn summarize(
        &self,
        credibility: f64,
        threshold: f64,
    ) -> Result<BayesianSummary, InformationError> {
        Ok(BayesianSummary {
            estimate: self.posterior_mean(),
            interval: self.credible_interval(credibility)?,
            threshold,
            probability_at_least: self.probability_at_least(threshold),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CredibleInterval {
    pub lower: f64,
    pub upper: f64,
    pub credibility: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BayesianSummary {
    pub estimate: f64,
    pub interval: CredibleInterval,
    pub threshold: f64,
    pub probability_at_least: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChannelScenario {
    pub source: Distribution,
    pub channel: Channel,
    pub weight: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChannelPosterior {
    scenarios: Vec<ChannelScenario>,
}

impl ChannelPosterior {
    pub fn new(scenarios: Vec<ChannelScenario>) -> Result<Self, InformationError> {
        if scenarios.is_empty() {
            return Err(InformationError::EmptyPosterior);
        }
        let weights: Vec<f64> = scenarios.iter().map(|scenario| scenario.weight).collect();
        validate_probability_vector(&weights, "posterior scenario weights")?;
        Ok(Self { scenarios })
    }

    #[must_use]
    pub fn scenario_count(&self) -> usize {
        self.scenarios.len()
    }

    pub fn mutual_information_posterior(&self) -> Result<PosteriorSamples, InformationError> {
        let mut values = Vec::with_capacity(self.scenarios.len());
        let mut weights = Vec::with_capacity(self.scenarios.len());
        for scenario in &self.scenarios {
            values.push(scenario.channel.mutual_information_bits(&scenario.source)?);
            weights.push(scenario.weight);
        }
        PosteriorSamples::new(values, weights)
    }

    pub fn retention_posterior(&self) -> Result<PosteriorSamples, InformationError> {
        let mut values = Vec::with_capacity(self.scenarios.len());
        let mut weights = Vec::with_capacity(self.scenarios.len());
        for scenario in &self.scenarios {
            values.push(scenario.channel.retention_fraction(&scenario.source)?);
            weights.push(scenario.weight);
        }
        PosteriorSamples::new(values, weights)
    }
}

fn validate_probability_vector(
    probabilities: &[f64],
    context: &'static str,
) -> Result<(), InformationError> {
    let mut sum = 0.0;
    for (index, probability) in probabilities.iter().enumerate() {
        if !probability.is_finite() || *probability < 0.0 {
            return Err(InformationError::InvalidProbability {
                context,
                index,
                value: *probability,
            });
        }
        sum += probability;
    }
    if (sum - 1.0).abs() > PROBABILITY_TOLERANCE {
        return Err(InformationError::ProbabilitySum { context, sum });
    }
    Ok(())
}

fn validate_credibility(credibility: f64) -> Result<(), InformationError> {
    if !(credibility.is_finite() && 0.0 < credibility && credibility <= 1.0) {
        return Err(InformationError::InvalidCredibility { value: credibility });
    }
    Ok(())
}

fn weighted_quantile(weighted_values: &[(f64, f64)], quantile: f64) -> f64 {
    let mut cumulative = 0.0;
    for (value, weight) in weighted_values {
        cumulative += weight;
        if cumulative + PROBABILITY_TOLERANCE >= quantile {
            return *value;
        }
    }
    weighted_values.last().map_or(0.0, |(value, _)| *value)
}

#[derive(Clone, Debug, PartialEq)]
pub enum InformationError {
    EmptyDistribution,
    EmptyChannel,
    EmptyPosterior,
    InvalidProbability {
        context: &'static str,
        index: usize,
        value: f64,
    },
    ProbabilitySum {
        context: &'static str,
        sum: f64,
    },
    RowWidthMismatch {
        row: usize,
        expected: usize,
        actual: usize,
    },
    OutputIndex {
        index: usize,
        cardinality: usize,
    },
    DimensionMismatch {
        context: &'static str,
        expected: usize,
        actual: usize,
    },
    CardinalityOverflow,
    ZeroEntropy,
    InvalidTolerance {
        value: f64,
    },
    PosteriorShapeMismatch {
        values: usize,
        weights: usize,
    },
    InvalidPosteriorValue {
        index: usize,
        value: f64,
    },
    InvalidCredibility {
        value: f64,
    },
}

impl fmt::Display for InformationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDistribution => formatter.write_str("distribution must not be empty"),
            Self::EmptyChannel => formatter.write_str("channel must not be empty"),
            Self::EmptyPosterior => formatter.write_str("posterior must not be empty"),
            Self::InvalidProbability {
                context,
                index,
                value,
            } => write!(
                formatter,
                "{context} probability {index} is invalid: {value}"
            ),
            Self::ProbabilitySum { context, sum } => {
                write!(formatter, "{context} probabilities sum to {sum}, not one")
            }
            Self::RowWidthMismatch {
                row,
                expected,
                actual,
            } => write!(
                formatter,
                "channel row {row} has width {actual}, expected {expected}"
            ),
            Self::OutputIndex { index, cardinality } => write!(
                formatter,
                "channel output index {index} is outside cardinality {cardinality}"
            ),
            Self::DimensionMismatch {
                context,
                expected,
                actual,
            } => write!(
                formatter,
                "{context} expects dimension {expected}, got {actual}"
            ),
            Self::CardinalityOverflow => formatter.write_str("channel cardinality overflow"),
            Self::ZeroEntropy => formatter.write_str("source entropy is zero"),
            Self::InvalidTolerance { value } => write!(formatter, "invalid tolerance {value}"),
            Self::PosteriorShapeMismatch { values, weights } => write!(
                formatter,
                "posterior has {values} values and {weights} weights"
            ),
            Self::InvalidPosteriorValue { index, value } => {
                write!(formatter, "posterior value {index} is invalid: {value}")
            }
            Self::InvalidCredibility { value } => {
                write!(formatter, "credibility must be in (0, 1], got {value}")
            }
        }
    }
}

impl std::error::Error for InformationError {}
