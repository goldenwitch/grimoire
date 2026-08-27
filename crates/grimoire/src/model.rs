use std::collections::BTreeMap;

use crate::{Address, Namespace, Version};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Description {
    pub address: Address,
    pub label: Option<String>,
    pub core_spec: Version,
    pub core: CoreGraph,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CoreGraph {
    pub blocks: BTreeMap<Address, Block>,
    pub connections: BTreeMap<Address, Connection>,
    pub groups: BTreeMap<Address, Group>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Block {
    pub address: Address,
    pub name: String,
    pub ports: BTreeMap<Address, Port>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Port {
    pub address: Address,
    pub label: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Connection {
    pub address: Address,
    pub label: Option<String>,
    pub source: Address,
    pub destination: Address,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Group {
    pub address: Address,
    pub label: Option<String>,
    pub members: Vec<Address>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtensionParameter {
    pub namespace: Namespace,
    pub name: String,
    pub schema: String,
    pub version: Version,
}

impl Description {
    #[must_use]
    pub fn new(address: Address, label: Option<String>, core_spec: Version) -> Self {
        Self {
            address,
            label,
            core_spec,
            core: CoreGraph::default(),
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
