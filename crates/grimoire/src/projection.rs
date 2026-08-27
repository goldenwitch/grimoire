use crate::{Address, Block, Connection, ExtensionParameter, Group, Namespace, Version};

#[derive(Clone, Debug, PartialEq)]
pub struct Layer {
    pub name: String,
    pub inputs: Vec<LayerInput>,
    pub projection_language: Version,
    pub schemas: Vec<SchemaUse>,
    pub projection: Projection,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerFile {
    pub description: Address,
    pub layer: Layer,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LayerInput {
    Core,
    Layer(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct SchemaUse {
    pub namespace: Namespace,
    pub name: String,
    pub version: Version,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Projection {
    pub select: Vec<SelectItem>,
    pub invert: Vec<Address>,
    pub decorate: Vec<Decoration>,
    pub checks: Vec<Check>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SelectItem {
    Use(Vec<Address>),
    GenerateBlock(Block),
    GenerateConnection(Connection),
    GenerateGroup(Group),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Decoration {
    pub target: Address,
    pub parameter: ExtensionParameter,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Check {
    pub name: String,
    pub expected: ExpectedCardinality,
    pub namespace: Namespace,
    pub parameter: String,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ExpectedCardinality {
    Empty,
    Nonempty,
}
