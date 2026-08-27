use std::collections::BTreeMap;

use crate::Address;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct FiniteNumber(f64);

impl FiniteNumber {
    pub fn new(value: f64) -> Option<Self> {
        value.is_finite().then_some(Self(value))
    }

    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    PositiveInteger(u64),
    Number(FiniteNumber),
    Text(String),
    Enum(String),
    Product(BTreeMap<String, Value>),
    Sequence(Vec<Value>),
    AddressReference(Address),
    Absent,
    Present(Box<Value>),
    Tagged { tag: String, value: Box<Value> },
}
