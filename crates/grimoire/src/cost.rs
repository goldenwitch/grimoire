use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

use crate::{Address, Element, StructuralReprojection};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CostExpression {
    Constant(u64),
    Axis(Address),
    Sum(Vec<CostExpression>),
    Product(Vec<CostExpression>),
}

impl CostExpression {
    #[must_use]
    pub const fn constant(value: u64) -> Self {
        Self::Constant(value)
    }

    #[must_use]
    pub fn axis(address: Address) -> Self {
        Self::Axis(address)
    }

    #[must_use]
    pub fn sum(terms: Vec<Self>) -> Self {
        Self::Sum(terms)
    }

    #[must_use]
    pub fn product(factors: Vec<Self>) -> Self {
        Self::Product(factors)
    }

    pub fn evaluate(&self, axes: &BTreeMap<Address, u64>) -> Result<u64, CostError> {
        match self {
            Self::Constant(value) => Ok(*value),
            Self::Axis(address) => {
                let value = *axes
                    .get(address)
                    .ok_or_else(|| CostError::MissingAxis(address.clone()))?;
                if value == 0 {
                    return Err(CostError::InvalidAxisExtent(address.clone()));
                }
                Ok(value)
            }
            Self::Sum(terms) => {
                let mut total = 0u64;
                for term in terms {
                    total = total
                        .checked_add(term.evaluate(axes)?)
                        .ok_or(CostError::ExpressionOverflow)?;
                }
                Ok(total)
            }
            Self::Product(factors) => {
                let mut total = 1u64;
                for factor in factors {
                    total = total
                        .checked_mul(factor.evaluate(axes)?)
                        .ok_or(CostError::ExpressionOverflow)?;
                }
                Ok(total)
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CostModel {
    expressions: BTreeMap<Address, CostExpression>,
}

impl CostModel {
    pub fn new(assignments: Vec<(Address, CostExpression)>) -> Result<Self, CostError> {
        let mut expressions = BTreeMap::new();
        for (address, expression) in assignments {
            if expressions.insert(address.clone(), expression).is_some() {
                return Err(CostError::DuplicateCost(address));
            }
        }
        Ok(Self { expressions })
    }

    #[must_use]
    pub fn expression(&self, address: &Address) -> Option<&CostExpression> {
        self.expressions.get(address)
    }

    pub fn evaluate(
        &self,
        reprojection: &StructuralReprojection,
        axes: &BTreeMap<Address, u64>,
    ) -> Result<CostReport, CostError> {
        let mut values = BTreeMap::new();
        for (address, expression) in &self.expressions {
            if !reprojection.elements.contains_key(address) {
                return Err(CostError::UnknownElement(address.clone()));
            }
            values.insert(address.clone(), expression.evaluate(axes)?);
        }
        Ok(CostReport { values })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CostReport {
    values: BTreeMap<Address, u64>,
}

impl CostReport {
    #[must_use]
    pub fn value(&self, address: &Address) -> Option<u64> {
        self.values.get(address).copied()
    }

    pub fn group_total(
        &self,
        reprojection: &StructuralReprojection,
        group_address: &Address,
    ) -> Result<u64, CostError> {
        let Some(Element::Group(_)) = reprojection.elements.get(group_address) else {
            return Err(CostError::NotAGroup(group_address.clone()));
        };
        let mut visiting = BTreeSet::new();
        self.accumulate_group(reprojection, group_address, &mut visiting)
    }

    fn accumulate_group(
        &self,
        reprojection: &StructuralReprojection,
        address: &Address,
        visiting: &mut BTreeSet<Address>,
    ) -> Result<u64, CostError> {
        if !visiting.insert(address.clone()) {
            return Err(CostError::CyclicGroup(address.clone()));
        }
        let Some(Element::Group(group)) = reprojection.elements.get(address) else {
            visiting.remove(address);
            return self
                .values
                .get(address)
                .copied()
                .ok_or_else(|| CostError::MissingCost(address.clone()));
        };
        let mut members_seen = BTreeSet::new();
        let mut total = 0u64;
        for member in &group.members {
            if !members_seen.insert(member.clone()) {
                visiting.remove(address);
                return Err(CostError::DuplicateGroupMember {
                    group: address.clone(),
                    member: member.clone(),
                });
            }
            if !reprojection.elements.contains_key(member) {
                visiting.remove(address);
                return Err(CostError::UnknownElement(member.clone()));
            }
            total = total
                .checked_add(self.accumulate_group(reprojection, member, visiting)?)
                .ok_or(CostError::TotalOverflow)?;
        }
        visiting.remove(address);
        Ok(total)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CostError {
    DuplicateCost(Address),
    MissingAxis(Address),
    InvalidAxisExtent(Address),
    ExpressionOverflow,
    UnknownElement(Address),
    NotAGroup(Address),
    MissingCost(Address),
    CyclicGroup(Address),
    DuplicateGroupMember { group: Address, member: Address },
    TotalOverflow,
}

impl fmt::Display for CostError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateCost(address) => {
                write!(formatter, "cost for `{address}` is declared more than once")
            }
            Self::MissingAxis(address) => write!(formatter, "cost axis `{address}` has no extent"),
            Self::InvalidAxisExtent(address) => {
                write!(formatter, "cost axis `{address}` has a non-positive extent")
            }
            Self::ExpressionOverflow => formatter.write_str("cost expression overflowed"),
            Self::UnknownElement(address) => {
                write!(formatter, "cost address `{address}` is not visible")
            }
            Self::NotAGroup(address) => write!(formatter, "cost target `{address}` is not a group"),
            Self::MissingCost(address) => write!(formatter, "no cost is assigned to `{address}`"),
            Self::CyclicGroup(address) => write!(formatter, "cost group `{address}` is cyclic"),
            Self::DuplicateGroupMember { group, member } => {
                write!(formatter, "cost group `{group}` repeats member `{member}`")
            }
            Self::TotalOverflow => formatter.write_str("cost group total overflowed"),
        }
    }
}

impl std::error::Error for CostError {}
