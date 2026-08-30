use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

use crate::{Address, Element, StructuralReprojection};

const PROBABILITY_TOLERANCE: f64 = 1e-12;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ResourceKind {
    FlopWork,
    Bytes,
    MemoryBytes,
    BandwidthBytesPerSecond,
    LatencyNanoseconds,
}

impl ResourceKind {
    pub const ALL: [Self; 5] = [
        Self::FlopWork,
        Self::Bytes,
        Self::MemoryBytes,
        Self::BandwidthBytesPerSecond,
        Self::LatencyNanoseconds,
    ];

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "flop-work" => Some(Self::FlopWork),
            "bytes" => Some(Self::Bytes),
            "memory-bytes" => Some(Self::MemoryBytes),
            "bandwidth-bytes-per-second" => Some(Self::BandwidthBytesPerSecond),
            "latency-nanoseconds" => Some(Self::LatencyNanoseconds),
            _ => None,
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FlopWork => "flop-work",
            Self::Bytes => "bytes",
            Self::MemoryBytes => "memory-bytes",
            Self::BandwidthBytesPerSecond => "bandwidth-bytes-per-second",
            Self::LatencyNanoseconds => "latency-nanoseconds",
        }
    }
}

impl fmt::Display for ResourceKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceBundle {
    quantities: BTreeMap<ResourceKind, u64>,
}

impl ResourceBundle {
    pub fn new(assignments: Vec<(ResourceKind, u64)>) -> Result<Self, ResourceError> {
        let mut quantities = BTreeMap::new();
        for (kind, quantity) in assignments {
            if quantities.insert(kind, quantity).is_some() {
                return Err(ResourceError::DuplicateResourceKind(kind));
            }
        }
        Ok(Self { quantities })
    }

    #[must_use]
    pub fn quantity(&self, kind: ResourceKind) -> Option<u64> {
        self.quantities.get(&kind).copied()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.quantities.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceFlow {
    pub relation: Address,
    pub source: Address,
    pub destination: Address,
    pub resources: ResourceBundle,
}

impl ResourceFlow {
    pub fn new(
        relation: Address,
        source: Address,
        destination: Address,
        resources: ResourceBundle,
    ) -> Result<Self, ResourceError> {
        if resources.is_empty() {
            return Err(ResourceError::EmptyResourceBundle { context: "flow" });
        }
        Ok(Self {
            relation,
            source,
            destination,
            resources,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceCharge {
    pub target: Address,
    pub resources: ResourceBundle,
}

impl ResourceCharge {
    pub fn new(target: Address, resources: ResourceBundle) -> Result<Self, ResourceError> {
        if resources.is_empty() {
            return Err(ResourceError::EmptyResourceBundle { context: "charge" });
        }
        Ok(Self { target, resources })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResourceScenario {
    name: String,
    probability: f64,
    assumption: String,
    flows: Vec<ResourceFlow>,
    charges: Vec<ResourceCharge>,
}

impl ResourceScenario {
    pub fn new(
        name: impl Into<String>,
        probability: f64,
        assumption: impl Into<String>,
        flows: Vec<ResourceFlow>,
        charges: Vec<ResourceCharge>,
    ) -> Result<Self, ResourceError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(ResourceError::EmptyScenarioName);
        }
        validate_probability(&name, probability)?;
        let assumption = assumption.into();
        if assumption.trim().is_empty() {
            return Err(ResourceError::EmptyScenarioAssumption(name));
        }
        Ok(Self {
            name,
            probability,
            assumption,
            flows,
            charges,
        })
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn probability(&self) -> f64 {
        self.probability
    }

    #[must_use]
    pub fn assumption(&self) -> &str {
        &self.assumption
    }

    #[must_use]
    pub fn flows(&self) -> &[ResourceFlow] {
        &self.flows
    }

    #[must_use]
    pub fn charges(&self) -> &[ResourceCharge] {
        &self.charges
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResourceModel {
    scenarios: Vec<ResourceScenario>,
}

impl ResourceModel {
    /// Creates a model with scenarios in canonical name order.
    ///
    /// Canonical storage order makes floating-point aggregation reproducible
    /// when callers provide the same named scenarios in different orders.
    pub fn new(scenarios: Vec<ResourceScenario>) -> Result<Self, ResourceError> {
        if scenarios.is_empty() {
            return Err(ResourceError::EmptyResourceModel);
        }
        let mut scenarios = scenarios;
        scenarios.sort_by(|left, right| left.name.cmp(&right.name));
        let mut names = BTreeSet::new();
        let mut probability_total = 0.0;
        for scenario in &scenarios {
            if !names.insert(scenario.name.clone()) {
                return Err(ResourceError::DuplicateScenario(scenario.name.clone()));
            }
            probability_total += scenario.probability;
        }
        if !probability_total.is_finite() || (probability_total - 1.0).abs() > PROBABILITY_TOLERANCE
        {
            return Err(ResourceError::ProbabilityTotal {
                actual: probability_total,
            });
        }
        Ok(Self { scenarios })
    }

    /// The scenarios in canonical name order.
    #[must_use]
    pub fn scenarios(&self) -> &[ResourceScenario] {
        &self.scenarios
    }

    pub fn evaluate(
        &self,
        reprojection: &StructuralReprojection,
    ) -> Result<ResourceReport, ResourceError> {
        let mut scenario_probabilities = BTreeMap::new();
        let mut assumptions = BTreeMap::new();
        let mut flow_totals = BTreeMap::new();
        let mut charge_totals = BTreeMap::new();

        for scenario in &self.scenarios {
            scenario_probabilities.insert(scenario.name.clone(), scenario.probability);
            assumptions.insert(scenario.name.clone(), scenario.assumption.clone());
            for flow in &scenario.flows {
                validate_flow(reprojection, flow)?;
                flow_totals
                    .entry(flow.relation.clone())
                    .or_insert_with(ResourceEstimate::default)
                    .add_scaled(&flow.resources, scenario.probability)?;
            }
            for charge in &scenario.charges {
                if !reprojection.elements.contains_key(&charge.target) {
                    return Err(ResourceError::UnknownAddress(charge.target.clone()));
                }
                charge_totals
                    .entry(charge.target.clone())
                    .or_insert_with(ResourceEstimate::default)
                    .add_scaled(&charge.resources, scenario.probability)?;
            }
        }

        Ok(ResourceReport {
            scenario_probabilities,
            assumptions,
            flow_totals,
            charge_totals,
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResourceEstimate {
    quantities: BTreeMap<ResourceKind, f64>,
}

impl ResourceEstimate {
    pub fn quantities(&self) -> impl Iterator<Item = (ResourceKind, f64)> + '_ {
        self.quantities
            .iter()
            .map(|(kind, quantity)| (*kind, *quantity))
    }

    #[must_use]
    pub fn quantity(&self, kind: ResourceKind) -> Option<f64> {
        self.quantities.get(&kind).copied()
    }

    fn add_scaled(
        &mut self,
        resources: &ResourceBundle,
        probability: f64,
    ) -> Result<(), ResourceError> {
        for (kind, quantity) in &resources.quantities {
            let scaled = *quantity as f64 * probability;
            let total = self.quantities.get(kind).copied().unwrap_or(0.0) + scaled;
            if !total.is_finite() {
                return Err(ResourceError::ArithmeticOverflow { kind: *kind });
            }
            self.quantities.insert(*kind, total);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResourceReport {
    scenario_probabilities: BTreeMap<String, f64>,
    assumptions: BTreeMap<String, String>,
    flow_totals: BTreeMap<Address, ResourceEstimate>,
    charge_totals: BTreeMap<Address, ResourceEstimate>,
}

impl ResourceReport {
    pub fn scenarios(&self) -> impl Iterator<Item = (&str, f64, &str)> + '_ {
        self.scenario_probabilities
            .iter()
            .map(|(name, probability)| {
                let assumption = self.assumptions.get(name).map(String::as_str).unwrap_or("");
                (name.as_str(), *probability, assumption)
            })
    }

    pub fn flow_estimates(&self) -> impl Iterator<Item = (&Address, &ResourceEstimate)> + '_ {
        self.flow_totals.iter()
    }

    pub fn charge_estimates(&self) -> impl Iterator<Item = (&Address, &ResourceEstimate)> + '_ {
        self.charge_totals.iter()
    }

    #[must_use]
    pub fn scenario_probability(&self, name: &str) -> Option<f64> {
        self.scenario_probabilities.get(name).copied()
    }

    #[must_use]
    pub fn assumption(&self, name: &str) -> Option<&str> {
        self.assumptions.get(name).map(String::as_str)
    }

    #[must_use]
    pub fn flow(&self, relation: &Address) -> Option<&ResourceEstimate> {
        self.flow_totals.get(relation)
    }

    #[must_use]
    pub fn charge(&self, target: &Address) -> Option<&ResourceEstimate> {
        self.charge_totals.get(target)
    }

    #[must_use]
    pub fn total_flow(&self, kind: ResourceKind) -> Option<f64> {
        total_quantity(&self.flow_totals, kind)
    }

    #[must_use]
    pub fn total_charge(&self, kind: ResourceKind) -> Option<f64> {
        total_quantity(&self.charge_totals, kind)
    }
}

fn total_quantity(
    estimates: &BTreeMap<Address, ResourceEstimate>,
    kind: ResourceKind,
) -> Option<f64> {
    let mut total = 0.0;
    let mut found = false;
    for estimate in estimates.values() {
        if let Some(quantity) = estimate.quantity(kind) {
            total += quantity;
            found = true;
        }
    }
    found.then_some(total)
}

fn validate_flow(
    reprojection: &StructuralReprojection,
    flow: &ResourceFlow,
) -> Result<(), ResourceError> {
    let Some(element) = reprojection.elements.get(&flow.relation) else {
        return Err(ResourceError::UnknownAddress(flow.relation.clone()));
    };
    let Element::Connection(connection) = element else {
        return Err(ResourceError::FlowRelationNotConnection(
            flow.relation.clone(),
        ));
    };
    if connection.source != flow.source || connection.destination != flow.destination {
        return Err(ResourceError::FlowEndpointMismatch {
            relation: flow.relation.clone(),
            expected_source: connection.source.clone(),
            expected_destination: connection.destination.clone(),
            actual_source: flow.source.clone(),
            actual_destination: flow.destination.clone(),
        });
    }
    Ok(())
}

fn validate_probability(name: &str, probability: f64) -> Result<(), ResourceError> {
    if !probability.is_finite() || !(0.0..=1.0).contains(&probability) {
        return Err(ResourceError::InvalidProbability {
            scenario: name.to_owned(),
            value: probability,
        });
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq)]
pub enum ResourceError {
    EmptyResourceModel,
    EmptyScenarioName,
    EmptyScenarioAssumption(String),
    DuplicateScenario(String),
    InvalidProbability {
        scenario: String,
        value: f64,
    },
    ProbabilityTotal {
        actual: f64,
    },
    DuplicateResourceKind(ResourceKind),
    EmptyResourceBundle {
        context: &'static str,
    },
    UnknownAddress(Address),
    FlowRelationNotConnection(Address),
    FlowEndpointMismatch {
        relation: Address,
        expected_source: Address,
        expected_destination: Address,
        actual_source: Address,
        actual_destination: Address,
    },
    ArithmeticOverflow {
        kind: ResourceKind,
    },
}

impl fmt::Display for ResourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyResourceModel => formatter.write_str("resource model has no scenarios"),
            Self::EmptyScenarioName => formatter.write_str("resource scenario name is empty"),
            Self::EmptyScenarioAssumption(name) => {
                write!(formatter, "resource scenario `{name}` has no assumption")
            }
            Self::DuplicateScenario(name) => {
                write!(
                    formatter,
                    "resource scenario `{name}` is declared more than once"
                )
            }
            Self::InvalidProbability { scenario, value } => {
                write!(
                    formatter,
                    "resource scenario `{scenario}` has invalid probability {value}"
                )
            }
            Self::ProbabilityTotal { actual } => {
                write!(
                    formatter,
                    "resource scenario probabilities sum to {actual}, not one"
                )
            }
            Self::DuplicateResourceKind(kind) => {
                write!(formatter, "resource bundle repeats `{kind}`")
            }
            Self::EmptyResourceBundle { context } => {
                write!(formatter, "resource {context} has no quantities")
            }
            Self::UnknownAddress(address) => {
                write!(formatter, "resource address `{address}` is not visible")
            }
            Self::FlowRelationNotConnection(address) => {
                write!(
                    formatter,
                    "resource flow relation `{address}` is not a connection"
                )
            }
            Self::FlowEndpointMismatch {
                relation,
                expected_source,
                expected_destination,
                actual_source,
                actual_destination,
            } => write!(
                formatter,
                "resource flow `{relation}` expects `{expected_source} -> {expected_destination}`, got `{actual_source} -> {actual_destination}`"
            ),
            Self::ArithmeticOverflow { kind } => {
                write!(formatter, "resource estimate for `{kind}` overflowed")
            }
        }
    }
}

impl std::error::Error for ResourceError {}
