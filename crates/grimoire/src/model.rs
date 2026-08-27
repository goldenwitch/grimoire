use std::collections::BTreeMap;

use crate::{Address, Namespace, Value, Version};

#[derive(Clone, Debug, PartialEq)]
pub struct Description {
    pub address: Address,
    pub label: Option<String>,
    pub core_spec: Version,
    pub core: CoreGraph,
    pub extensions: Vec<ExtensionParameter>,
    pub layers: Vec<crate::Layer>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CoreGraph {
    pub blocks: BTreeMap<Address, Block>,
    pub connections: BTreeMap<Address, Connection>,
    pub groups: BTreeMap<Address, Group>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub address: Address,
    pub name: String,
    pub ports: BTreeMap<Address, Port>,
    pub extensions: Vec<ExtensionParameter>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Port {
    pub address: Address,
    pub label: Option<String>,
    pub extensions: Vec<ExtensionParameter>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Connection {
    pub address: Address,
    pub label: Option<String>,
    pub source: Address,
    pub destination: Address,
    pub extensions: Vec<ExtensionParameter>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Group {
    pub address: Address,
    pub label: Option<String>,
    pub members: Vec<Address>,
    pub extensions: Vec<ExtensionParameter>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExtensionParameter {
    pub namespace: Namespace,
    pub name: String,
    pub schema: String,
    pub version: Version,
    pub value: ExtensionValue,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExtensionValue {
    Known(Value),
    Opaque(Vec<u8>),
}

impl Description {
    #[must_use]
    pub fn new(address: Address, label: Option<String>, core_spec: Version) -> Self {
        Self {
            address,
            label,
            core_spec,
            core: CoreGraph::default(),
            extensions: Vec::new(),
            layers: Vec::new(),
        }
    }

    #[must_use]
    pub fn addresses(&self) -> Vec<&Address> {
        let mut addresses = vec![&self.address];
        addresses.extend(self.core.blocks.keys());
        addresses.extend(self.core.connections.keys());
        addresses.extend(self.core.groups.keys());
        for block in self.core.blocks.values() {
            addresses.extend(block.ports.keys());
        }
        addresses
    }
}

impl CoreGraph {
    #[must_use]
    pub fn addresses(&self) -> Vec<&Address> {
        let mut addresses = Vec::new();
        addresses.extend(self.blocks.keys());
        addresses.extend(self.connections.keys());
        addresses.extend(self.groups.keys());
        for block in self.blocks.values() {
            addresses.extend(block.ports.keys());
        }
        addresses
    }
}
