use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

use crate::{Address, Element, StructuralReprojection};

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InformationQuantity {
    Entropy,
    MutualInformation,
    ConditionalMutualInformation,
    RetentionFraction,
}

#[derive(Clone, Debug, PartialEq)]
pub enum InformationDenominator {
    SourceEntropyBits(f64),
    Explicit { value: f64, unit: String },
}

#[derive(Clone, Debug, PartialEq)]
pub enum ClaimEstimate {
    Exact(f64),
    Bayesian(BayesianSummary),
}

impl ClaimEstimate {
    #[must_use]
    pub fn value(&self) -> f64 {
        match self {
            Self::Exact(value) => *value,
            Self::Bayesian(summary) => summary.estimate,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InformationClaim {
    pub source: Address,
    pub terminals: Vec<Address>,
    pub quantity: InformationQuantity,
    pub denominator: Option<InformationDenominator>,
    pub estimate: ClaimEstimate,
    pub method: String,
    pub evidence: String,
}

impl InformationClaim {
    pub fn new(
        source: Address,
        terminals: Vec<Address>,
        quantity: InformationQuantity,
        denominator: Option<InformationDenominator>,
        estimate: ClaimEstimate,
        method: String,
        evidence: String,
    ) -> Result<Self, InformationError> {
        if terminals.is_empty() {
            return Err(InformationError::EmptyClaimTerminals);
        }
        if let Some(duplicate) = find_duplicate(&terminals) {
            return Err(InformationError::DuplicateClaimTerminal(duplicate));
        }
        if method.trim().is_empty() {
            return Err(InformationError::EmptyClaimMethod);
        }
        if evidence.trim().is_empty() {
            return Err(InformationError::EmptyClaimEvidence);
        }
        validate_denominator(quantity, denominator.as_ref())?;
        validate_claim_estimate(quantity, &estimate)?;
        Ok(Self {
            source,
            terminals,
            quantity,
            denominator,
            estimate,
            method,
            evidence,
        })
    }

    pub fn exact(
        source: Address,
        terminals: Vec<Address>,
        quantity: InformationQuantity,
        denominator: Option<InformationDenominator>,
        value: f64,
        method: String,
        evidence: String,
    ) -> Result<Self, InformationError> {
        Self::new(
            source,
            terminals,
            quantity,
            denominator,
            ClaimEstimate::Exact(value),
            method,
            evidence,
        )
    }

    pub fn bayesian(
        source: Address,
        terminals: Vec<Address>,
        quantity: InformationQuantity,
        denominator: Option<InformationDenominator>,
        summary: BayesianSummary,
        method: String,
        evidence: String,
    ) -> Result<Self, InformationError> {
        Self::new(
            source,
            terminals,
            quantity,
            denominator,
            ClaimEstimate::Bayesian(summary),
            method,
            evidence,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RouteShare {
    pub route: Vec<Address>,
    pub estimate: ClaimEstimate,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RouteAllocationClaim {
    pub source: Address,
    pub denominator: InformationDenominator,
    pub partition: String,
    pub method: String,
    pub shares: Vec<RouteShare>,
}

impl RouteAllocationClaim {
    pub fn new(
        source: Address,
        denominator: InformationDenominator,
        partition: String,
        method: String,
        shares: Vec<RouteShare>,
    ) -> Result<Self, InformationError> {
        if partition.trim().is_empty() {
            return Err(InformationError::EmptyRoutePartition);
        }
        if method.trim().is_empty() {
            return Err(InformationError::EmptyClaimMethod);
        }
        if shares.is_empty() {
            return Err(InformationError::EmptyRouteShares);
        }
        validate_denominator(InformationQuantity::RetentionFraction, Some(&denominator))?;
        let mut share_sum = 0.0;
        let mut routes = Vec::new();
        for share in &shares {
            if share.route.is_empty() {
                return Err(InformationError::EmptyRoute);
            }
            if routes
                .iter()
                .any(|route: &Vec<Address>| route == &share.route)
            {
                return Err(InformationError::DuplicateRoute);
            }
            routes.push(share.route.clone());
            validate_claim_estimate_bounds(&share.estimate, 0.0, 1.0)?;
            share_sum += share.estimate.value();
        }
        if (share_sum - 1.0).abs() > PROBABILITY_TOLERANCE {
            return Err(InformationError::RouteSharesDoNotSum { sum: share_sum });
        }
        Ok(Self {
            source,
            denominator,
            partition,
            method,
            shares,
        })
    }
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

#[derive(Clone, Debug, PartialEq)]
pub struct ChannelNode {
    pub address: Address,
    pub block: Address,
    pub input_ports: Vec<Address>,
    pub output_port: Address,
    pub channel: Channel,
}

impl ChannelNode {
    pub fn new(
        address: Address,
        block: Address,
        input_ports: Vec<Address>,
        output_port: Address,
        channel: Channel,
    ) -> Result<Self, InformationError> {
        if input_ports.is_empty() {
            return Err(InformationError::EmptyNodeInputs(address));
        }
        if input_ports.contains(&output_port) {
            return Err(InformationError::DuplicateGraphPort(output_port));
        }
        if let Some(duplicate) = find_duplicate(&input_ports) {
            return Err(InformationError::DuplicateGraphPort(duplicate));
        }
        Ok(Self {
            address,
            block,
            input_ports,
            output_port,
            channel,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChannelLink {
    pub source: Address,
    pub destination: Address,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChannelGraph {
    nodes: BTreeMap<Address, ChannelNode>,
    links: Vec<ChannelLink>,
    visible_ports: Option<BTreeSet<Address>>,
}

impl ChannelGraph {
    pub fn new(nodes: Vec<ChannelNode>, links: Vec<ChannelLink>) -> Result<Self, InformationError> {
        if nodes.is_empty() {
            return Err(InformationError::EmptyGraph);
        }
        let mut node_map = BTreeMap::new();
        let mut graph_ports = BTreeSet::new();
        let mut input_ports = BTreeSet::new();
        for node in nodes {
            let node_address = node.address.clone();
            if node_map.insert(node_address.clone(), node).is_some() {
                return Err(InformationError::DuplicateGraphNode(node_address));
            }
        }
        for node in node_map.values() {
            if !graph_ports.insert(node.output_port.clone()) {
                return Err(InformationError::DuplicateGraphPort(
                    node.output_port.clone(),
                ));
            }
            for port in &node.input_ports {
                if !graph_ports.insert(port.clone()) {
                    return Err(InformationError::DuplicateGraphPort(port.clone()));
                }
                input_ports.insert(port.clone());
            }
        }
        let mut destinations = BTreeMap::new();
        let mut link_pairs = BTreeSet::new();
        for link in &links {
            if !input_ports.contains(&link.destination) {
                return Err(InformationError::UnknownDestinationPort(
                    link.destination.clone(),
                ));
            }
            if destinations
                .insert(link.destination.clone(), link.source.clone())
                .is_some()
            {
                return Err(InformationError::MultipleIncomingLinks(
                    link.destination.clone(),
                ));
            }
            if !link_pairs.insert((link.source.clone(), link.destination.clone())) {
                return Err(InformationError::DuplicateGraphLink {
                    source: link.source.clone(),
                    destination: link.destination.clone(),
                });
            }
        }
        Ok(Self {
            nodes: node_map,
            links,
            visible_ports: None,
        })
    }

    pub fn channel_to_terminal(
        &self,
        source_port: &Address,
        source: &Distribution,
        terminal: &Address,
    ) -> Result<Channel, InformationError> {
        self.check_visible_port(source_port)?;
        self.check_visible_port(terminal)?;
        let mut resolver = GraphResolver::new(self, source_port, source)?;
        resolver.resolve_port(terminal)
    }

    pub fn from_reprojection(
        reprojection: &StructuralReprojection,
        nodes: Vec<ChannelNode>,
    ) -> Result<Self, InformationError> {
        for node in &nodes {
            if !matches!(
                reprojection.elements.get(&node.block),
                Some(Element::Block(_))
            ) {
                return Err(InformationError::MissingGraphElement(node.block.clone()));
            }
            for port in node
                .input_ports
                .iter()
                .chain(std::iter::once(&node.output_port))
            {
                if !matches!(reprojection.elements.get(port), Some(Element::Port(_))) {
                    return Err(InformationError::MissingGraphElement(port.clone()));
                }
            }
        }
        let links = reprojection
            .elements
            .values()
            .filter_map(|element| match element {
                Element::Connection(connection) => Some(ChannelLink {
                    source: connection.source.clone(),
                    destination: connection.destination.clone(),
                }),
                Element::Description(_)
                | Element::Block(_)
                | Element::Port(_)
                | Element::Group(_) => None,
            })
            .collect();
        let mut graph = Self::new(nodes, links)?;
        graph.visible_ports = Some(
            reprojection
                .elements
                .iter()
                .filter(|(_, element)| matches!(element, Element::Port(_)))
                .map(|(address, _)| address.clone())
                .collect(),
        );
        Ok(graph)
    }

    pub fn from_layer(
        description: &crate::Description,
        layer_name: &str,
        nodes: Vec<ChannelNode>,
    ) -> Result<Self, InformationError> {
        let reprojection = crate::evaluate_layer(description, layer_name)
            .map_err(|error| InformationError::ProjectionEvaluation(error.to_string()))?;
        Self::from_reprojection(&reprojection.structural, nodes)
    }

    pub fn joint_channel_to_terminals(
        &self,
        source_port: &Address,
        source: &Distribution,
        terminals: &[Address],
    ) -> Result<Channel, InformationError> {
        if terminals.is_empty() {
            return Err(InformationError::EmptyTerminalSet);
        }
        if let Some(duplicate) = find_duplicate(terminals) {
            return Err(InformationError::DuplicateClaimTerminal(duplicate));
        }
        self.check_visible_port(source_port)?;
        for terminal in terminals {
            self.check_visible_port(terminal)?;
        }
        let mut resolver = GraphResolver::new(self, source_port, source)?;
        let mut joint = resolver.resolve_port(&terminals[0])?;
        for terminal in &terminals[1..] {
            let branch = resolver.resolve_port(terminal)?;
            joint = joint.conditionally_independent_branch(&branch)?;
        }
        Ok(joint)
    }

    pub fn information_claim(
        &self,
        source_port: &Address,
        source: &Distribution,
        terminal: &Address,
        method: String,
        evidence: String,
    ) -> Result<InformationClaim, InformationError> {
        let channel = self.channel_to_terminal(source_port, source, terminal)?;
        let value = channel.mutual_information_bits(source)?;
        InformationClaim::exact(
            source_port.clone(),
            vec![terminal.clone()],
            InformationQuantity::MutualInformation,
            None,
            value,
            method,
            evidence,
        )
    }

    pub fn joint_information_claim(
        &self,
        source_port: &Address,
        source: &Distribution,
        terminals: &[Address],
        method: String,
        evidence: String,
    ) -> Result<InformationClaim, InformationError> {
        let channel = self.joint_channel_to_terminals(source_port, source, terminals)?;
        let value = channel.mutual_information_bits(source)?;
        InformationClaim::exact(
            source_port.clone(),
            terminals.to_vec(),
            InformationQuantity::MutualInformation,
            None,
            value,
            method,
            evidence,
        )
    }

    fn check_visible_port(&self, port: &Address) -> Result<(), InformationError> {
        if self
            .visible_ports
            .as_ref()
            .is_some_and(|ports| !ports.contains(port))
        {
            return Err(InformationError::UnvisiblePort(port.clone()));
        }
        Ok(())
    }
}

struct GraphResolver<'graph> {
    graph: &'graph ChannelGraph,
    source_port: Address,
    source: &'graph Distribution,
    incoming: BTreeMap<Address, Address>,
    input_nodes: BTreeMap<Address, Address>,
    output_nodes: BTreeMap<Address, Address>,
    state: BTreeMap<Address, VisitState>,
    channels: BTreeMap<Address, Channel>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum VisitState {
    Visiting,
    Complete,
}

impl<'graph> GraphResolver<'graph> {
    fn new(
        graph: &'graph ChannelGraph,
        source_port: &Address,
        source: &'graph Distribution,
    ) -> Result<Self, InformationError> {
        let mut incoming = BTreeMap::new();
        for link in &graph.links {
            incoming.insert(link.destination.clone(), link.source.clone());
        }
        let mut input_nodes = BTreeMap::new();
        let mut output_nodes = BTreeMap::new();
        for node in graph.nodes.values() {
            for port in &node.input_ports {
                input_nodes.insert(port.clone(), node.address.clone());
            }
            output_nodes.insert(node.output_port.clone(), node.address.clone());
        }
        Ok(Self {
            graph,
            source_port: source_port.clone(),
            source,
            incoming,
            input_nodes,
            output_nodes,
            state: BTreeMap::new(),
            channels: BTreeMap::new(),
        })
    }

    fn resolve_port(&mut self, port: &Address) -> Result<Channel, InformationError> {
        if port == &self.source_port {
            return Channel::identity(self.source.cardinality());
        }
        if let Some(node_address) = self.output_nodes.get(port).cloned() {
            self.resolve_node(&node_address)?;
            return self
                .channels
                .get(port)
                .cloned()
                .ok_or_else(|| InformationError::UnreachableTerminal(port.clone()));
        }
        if self.input_nodes.contains_key(port) {
            let upstream = self
                .incoming
                .get(port)
                .cloned()
                .ok_or_else(|| InformationError::MissingGraphInput(port.clone()))?;
            return self.resolve_port(&upstream);
        }
        Err(InformationError::UnreachableTerminal(port.clone()))
    }

    fn resolve_node(&mut self, node_address: &Address) -> Result<(), InformationError> {
        match self.state.get(node_address) {
            Some(VisitState::Complete) => return Ok(()),
            Some(VisitState::Visiting) => return Err(InformationError::CyclicGraph),
            None => {}
        }
        self.state
            .insert(node_address.clone(), VisitState::Visiting);
        let node = self
            .graph
            .nodes
            .get(node_address)
            .ok_or_else(|| InformationError::UnreachableNode(node_address.clone()))?
            .clone();
        let mut input_channel: Option<Channel> = None;
        for input_port in &node.input_ports {
            let branch = self.resolve_port(input_port)?;
            input_channel = Some(match input_channel {
                None => branch,
                Some(existing) => existing.conditionally_independent_branch(&branch)?,
            });
        }
        let input_channel =
            input_channel.ok_or_else(|| InformationError::EmptyNodeInputs(node.address.clone()))?;
        let output_channel = input_channel.compose(&node.channel)?;
        self.channels.insert(node.output_port, output_channel);
        self.state
            .insert(node_address.clone(), VisitState::Complete);
        Ok(())
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

fn find_duplicate(values: &[Address]) -> Option<Address> {
    for (index, value) in values.iter().enumerate() {
        if values[..index].contains(value) {
            return Some(value.clone());
        }
    }
    None
}

fn validate_denominator(
    quantity: InformationQuantity,
    denominator: Option<&InformationDenominator>,
) -> Result<(), InformationError> {
    if quantity != InformationQuantity::RetentionFraction && denominator.is_some() {
        return Err(InformationError::UnexpectedDenominator);
    }
    if quantity == InformationQuantity::RetentionFraction && denominator.is_none() {
        return Err(InformationError::MissingDenominator);
    }
    if let Some(denominator) = denominator {
        let value = match denominator {
            InformationDenominator::SourceEntropyBits(value) => *value,
            InformationDenominator::Explicit { value, .. } => *value,
        };
        if !value.is_finite() || value <= PROBABILITY_TOLERANCE {
            return Err(InformationError::InvalidDenominator { value });
        }
    }
    Ok(())
}

fn validate_claim_estimate(
    quantity: InformationQuantity,
    estimate: &ClaimEstimate,
) -> Result<(), InformationError> {
    validate_claim_estimate_bounds(estimate, 0.0, f64::INFINITY)?;
    if quantity == InformationQuantity::RetentionFraction {
        validate_claim_estimate_bounds(estimate, 0.0, 1.0)?;
    }
    Ok(())
}

fn validate_claim_estimate_bounds(
    estimate: &ClaimEstimate,
    lower_bound: f64,
    upper_bound: f64,
) -> Result<(), InformationError> {
    match estimate {
        ClaimEstimate::Exact(value) => {
            if !value.is_finite() || *value < lower_bound || *value > upper_bound {
                return Err(InformationError::InvalidClaimValue { value: *value });
            }
        }
        ClaimEstimate::Bayesian(summary) => {
            if !summary.estimate.is_finite()
                || summary.estimate < lower_bound
                || summary.estimate > upper_bound
            {
                return Err(InformationError::InvalidClaimValue {
                    value: summary.estimate,
                });
            }
            if !summary.interval.lower.is_finite()
                || !summary.interval.upper.is_finite()
                || summary.interval.lower > summary.interval.upper
                || summary.interval.lower < lower_bound
                || summary.interval.upper > upper_bound
                || summary.estimate < summary.interval.lower - PROBABILITY_TOLERANCE
                || summary.estimate > summary.interval.upper + PROBABILITY_TOLERANCE
            {
                return Err(InformationError::InvalidClaimEstimate {
                    message: "posterior estimate is outside its credible interval",
                });
            }
            validate_credibility(summary.interval.credibility)?;
            if !summary.threshold.is_finite()
                || !summary.probability_at_least.is_finite()
                || !(0.0..=1.0).contains(&summary.probability_at_least)
            {
                return Err(InformationError::InvalidClaimEstimate {
                    message: "posterior decision probability is invalid",
                });
            }
        }
    }
    Ok(())
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
    EmptyClaimTerminals,
    DuplicateClaimTerminal(Address),
    EmptyClaimMethod,
    EmptyClaimEvidence,
    MissingDenominator,
    UnexpectedDenominator,
    InvalidDenominator {
        value: f64,
    },
    InvalidClaimValue {
        value: f64,
    },
    InvalidClaimEstimate {
        message: &'static str,
    },
    EmptyRoutePartition,
    EmptyRouteShares,
    EmptyRoute,
    DuplicateRoute,
    RouteSharesDoNotSum {
        sum: f64,
    },
    EmptyNodeInputs(Address),
    DuplicateGraphPort(Address),
    EmptyGraph,
    DuplicateGraphNode(Address),
    UnknownDestinationPort(Address),
    MultipleIncomingLinks(Address),
    DuplicateGraphLink {
        source: Address,
        destination: Address,
    },
    EmptyTerminalSet,
    UnreachableTerminal(Address),
    MissingGraphInput(Address),
    CyclicGraph,
    UnreachableNode(Address),
    MissingGraphElement(Address),
    ProjectionEvaluation(String),
    UnvisiblePort(Address),
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
            Self::EmptyClaimTerminals => formatter.write_str("information claim needs a terminal"),
            Self::DuplicateClaimTerminal(address) => {
                write!(formatter, "information claim repeats terminal `{address}`")
            }
            Self::EmptyClaimMethod => {
                formatter.write_str("information claim method must not be empty")
            }
            Self::EmptyClaimEvidence => {
                formatter.write_str("information claim evidence must not be empty")
            }
            Self::MissingDenominator => {
                formatter.write_str("retention claim needs a positive denominator")
            }
            Self::UnexpectedDenominator => {
                formatter.write_str("only retention claims may have a denominator")
            }
            Self::InvalidDenominator { value } => {
                write!(
                    formatter,
                    "information claim denominator is invalid: {value}"
                )
            }
            Self::InvalidClaimValue { value } => {
                write!(formatter, "information claim value is invalid: {value}")
            }
            Self::InvalidClaimEstimate { message } => formatter.write_str(message),
            Self::EmptyRoutePartition => formatter.write_str("route partition must not be empty"),
            Self::EmptyRouteShares => formatter.write_str("route allocation needs a share"),
            Self::EmptyRoute => formatter.write_str("route share must name an address route"),
            Self::DuplicateRoute => formatter.write_str("route allocation repeats a route"),
            Self::RouteSharesDoNotSum { sum } => {
                write!(formatter, "route shares sum to {sum}, not one")
            }
            Self::EmptyNodeInputs(address) => {
                write!(formatter, "channel node `{address}` needs an input port")
            }
            Self::DuplicateGraphPort(address) => {
                write!(formatter, "channel graph reuses port `{address}`")
            }
            Self::EmptyGraph => formatter.write_str("channel graph must not be empty"),
            Self::DuplicateGraphNode(address) => {
                write!(formatter, "channel graph repeats node `{address}`")
            }
            Self::UnknownDestinationPort(address) => {
                write!(
                    formatter,
                    "channel link targets unknown input port `{address}`"
                )
            }
            Self::MultipleIncomingLinks(address) => {
                write!(
                    formatter,
                    "channel input port `{address}` has multiple incoming links"
                )
            }
            Self::DuplicateGraphLink {
                source,
                destination,
            } => write!(
                formatter,
                "channel graph repeats link `{source}` -> `{destination}`"
            ),
            Self::EmptyTerminalSet => formatter.write_str("channel query needs a terminal"),
            Self::UnreachableTerminal(address) => {
                write!(formatter, "channel terminal `{address}` is unreachable")
            }
            Self::MissingGraphInput(address) => {
                write!(formatter, "channel input `{address}` has no source")
            }
            Self::CyclicGraph => formatter.write_str("channel query reaches a cycle"),
            Self::UnreachableNode(address) => {
                write!(formatter, "channel node `{address}` is not reachable")
            }
            Self::MissingGraphElement(address) => {
                write!(
                    formatter,
                    "channel graph references missing element `{address}`"
                )
            }
            Self::ProjectionEvaluation(message) => {
                write!(
                    formatter,
                    "channel graph layer evaluation failed: {message}"
                )
            }
            Self::UnvisiblePort(address) => {
                write!(
                    formatter,
                    "channel query uses port outside the selected layer: `{address}`"
                )
            }
        }
    }
}

impl std::error::Error for InformationError {}
