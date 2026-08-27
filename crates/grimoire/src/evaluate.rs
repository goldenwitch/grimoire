use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    Address, Block, Connection, Decoration, Description, Group, Layer, LayerInput, Projection,
    SelectItem,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Element {
    Description(Address),
    Block(Block),
    Port(crate::Port),
    Connection(Connection),
    Group(Group),
}

#[derive(Clone, Debug)]
pub struct StructuralReprojection {
    pub elements: BTreeMap<Address, Element>,
    origins: BTreeMap<Address, DefinitionOrigin>,
}

impl PartialEq for StructuralReprojection {
    fn eq(&self, other: &Self) -> bool {
        self.elements == other.elements
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FinalizedReprojection {
    pub structural: StructuralReprojection,
    pub decorations: Vec<Decoration>,
    pub checks: Vec<CheckResult>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CheckResult {
    pub name: String,
    pub expected: crate::ExpectedCardinality,
    pub observed: usize,
    pub passed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionError {
    pub stage: &'static str,
    pub identifier: Option<String>,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum DefinitionOrigin {
    Core,
    Layer(String),
}

pub fn evaluate_layer(
    description: &Description,
    layer_name: &str,
) -> Result<FinalizedReprojection, ProjectionError> {
    let mut stack = BTreeSet::new();
    evaluate_reprojection(description, layer_name, &mut stack)
}

impl StructuralReprojection {
    fn empty() -> Self {
        Self {
            elements: BTreeMap::new(),
            origins: BTreeMap::new(),
        }
    }

    fn from_core(description: &Description) -> Result<Self, ProjectionError> {
        let mut result = Self::empty();
        result.add(
            description.address.clone(),
            Element::Description(description.address.clone()),
            DefinitionOrigin::Core,
        )?;
        for block in description.core.blocks.values() {
            result.add(
                block.address.clone(),
                Element::Block(block.clone()),
                DefinitionOrigin::Core,
            )?;
            for port in block.ports.values() {
                result.add(
                    port.address.clone(),
                    Element::Port(port.clone()),
                    DefinitionOrigin::Core,
                )?;
            }
        }
        for connection in description.core.connections.values() {
            result.add(
                connection.address.clone(),
                Element::Connection(connection.clone()),
                DefinitionOrigin::Core,
            )?;
        }
        for group in description.core.groups.values() {
            result.add(
                group.address.clone(),
                Element::Group(group.clone()),
                DefinitionOrigin::Core,
            )?;
        }
        Ok(result)
    }

    fn add(
        &mut self,
        address: Address,
        element: Element,
        origin: DefinitionOrigin,
    ) -> Result<(), ProjectionError> {
        if let Some(existing) = self.elements.get(&address) {
            let same_origin = self.origins.get(&address) == Some(&origin);
            if same_origin && existing == &element {
                return Ok(());
            }
            return Err(ProjectionError::new(
                "fold",
                Some(address.to_string()),
                "competing definitions at one address",
            ));
        }
        self.origins.insert(address.clone(), origin);
        self.elements.insert(address, element);
        Ok(())
    }

    fn merge(&mut self, other: Self) -> Result<(), ProjectionError> {
        for (address, element) in other.elements {
            let Some(origin) = other.origins.get(&address).cloned() else {
                return Err(ProjectionError::new(
                    "fold",
                    Some(address.to_string()),
                    "structural element has no definition origin",
                ));
            };
            self.add(address, element, origin)?;
        }
        Ok(())
    }

    fn selected(
        &self,
        address: &Address,
    ) -> Result<(Address, Element, DefinitionOrigin), ProjectionError> {
        let Some(element) = self.elements.get(address) else {
            return Err(ProjectionError::new(
                "select",
                Some(address.to_string()),
                "selected address is not present in the declared inputs",
            ));
        };
        let Some(origin) = self.origins.get(address) else {
            return Err(ProjectionError::new(
                "select",
                Some(address.to_string()),
                "selected address has no definition origin",
            ));
        };
        Ok((address.clone(), element.clone(), origin.clone()))
    }
}

fn evaluate_reprojection(
    description: &Description,
    layer_name: &str,
    stack: &mut BTreeSet<String>,
) -> Result<FinalizedReprojection, ProjectionError> {
    if !stack.insert(layer_name.to_owned()) {
        return Err(ProjectionError::new(
            "select",
            Some(layer_name.to_owned()),
            "layer input cycle during projection evaluation",
        ));
    }
    let layer = find_layer(description, layer_name)?;
    let mut inputs = StructuralReprojection::empty();
    let mut inherited_decorations = Vec::new();
    for input in &layer.inputs {
        let input_result = match input {
            LayerInput::Core => FinalizedReprojection {
                structural: StructuralReprojection::from_core(description)?,
                decorations: Vec::new(),
                checks: Vec::new(),
            },
            LayerInput::Layer(name) => evaluate_reprojection(description, name, stack)?,
        };
        inputs.merge(input_result.structural)?;
        inherited_decorations.extend(input_result.decorations);
    }
    let mut selected = StructuralReprojection::empty();
    for item in &layer.projection.select {
        match item {
            SelectItem::Use(addresses) => {
                let mut expanded_groups = BTreeSet::new();
                for address in addresses {
                    select_address(&inputs, &mut selected, address, &mut expanded_groups)?;
                }
            }
            SelectItem::GenerateBlock(block) => {
                let origin = DefinitionOrigin::Layer(layer.name.clone());
                selected.add(
                    block.address.clone(),
                    Element::Block(block.clone()),
                    origin.clone(),
                )?;
                for port in block.ports.values() {
                    selected.add(
                        port.address.clone(),
                        Element::Port(port.clone()),
                        origin.clone(),
                    )?;
                }
            }
            SelectItem::GenerateConnection(connection) => selected.add(
                connection.address.clone(),
                Element::Connection(connection.clone()),
                DefinitionOrigin::Layer(layer.name.clone()),
            )?,
            SelectItem::GenerateGroup(group) => selected.add(
                group.address.clone(),
                Element::Group(group.clone()),
                DefinitionOrigin::Layer(layer.name.clone()),
            )?,
        }
    }
    apply_invert(&mut selected, &layer.projection)?;
    stack.remove(layer_name);
    finalize(selected, inherited_decorations, &layer.projection)
}

fn select_address(
    inputs: &StructuralReprojection,
    selected: &mut StructuralReprojection,
    address: &Address,
    expanded_groups: &mut BTreeSet<Address>,
) -> Result<(), ProjectionError> {
    let (address, element, origin) = inputs.selected(address)?;
    let group_members = match &element {
        Element::Group(group) if expanded_groups.insert(address.clone()) => {
            Some(group.members.clone())
        }
        _ => None,
    };
    selected.add(address, element, origin)?;
    if let Some(group_members) = group_members {
        for member in group_members {
            select_address(inputs, selected, &member, expanded_groups)?;
        }
    }
    Ok(())
}

fn apply_invert(
    structural: &mut StructuralReprojection,
    projection: &Projection,
) -> Result<(), ProjectionError> {
    let mut connections = BTreeSet::new();
    for group_address in &projection.invert {
        let Some(Element::Group(group)) = structural.elements.get(group_address) else {
            return Err(ProjectionError::new(
                "invert",
                Some(group_address.to_string()),
                "invert target is not a selected group",
            ));
        };
        collect_group_connections(structural, group, &mut connections);
    }
    for address in connections {
        let Some(Element::Connection(connection)) = structural.elements.get_mut(&address) else {
            continue;
        };
        std::mem::swap(&mut connection.source, &mut connection.destination);
    }
    Ok(())
}

fn collect_group_connections(
    structural: &StructuralReprojection,
    group: &Group,
    connections: &mut BTreeSet<Address>,
) {
    for member in &group.members {
        match structural.elements.get(member) {
            Some(Element::Connection(_)) => {
                connections.insert(member.clone());
            }
            Some(Element::Group(group)) => {
                collect_group_connections(structural, group, connections)
            }
            _ => {}
        }
    }
}

fn finalize(
    structural: StructuralReprojection,
    inherited_decorations: Vec<Decoration>,
    projection: &Projection,
) -> Result<FinalizedReprojection, ProjectionError> {
    let mut decorations: Vec<Decoration> = inherited_decorations
        .into_iter()
        .filter(|decoration| structural.elements.contains_key(&decoration.target))
        .collect();
    for decoration in &projection.decorate {
        if !structural.elements.contains_key(&decoration.target) {
            return Err(ProjectionError::new(
                "decorate",
                Some(decoration.target.to_string()),
                "decoration target is not in the folded structural result",
            ));
        }
        decorations.push(decoration.clone());
    }
    let checks = projection
        .checks
        .iter()
        .map(|check| {
            let observed = decorations
                .iter()
                .filter(|decoration| {
                    decoration.parameter.namespace == check.namespace
                        && decoration.parameter.name == check.parameter
                })
                .count();
            let passed = match check.expected {
                crate::ExpectedCardinality::Empty => observed == 0,
                crate::ExpectedCardinality::Nonempty => observed > 0,
            };
            CheckResult {
                name: check.name.clone(),
                expected: check.expected,
                observed,
                passed,
            }
        })
        .collect();
    Ok(FinalizedReprojection {
        structural,
        decorations,
        checks,
    })
}

fn find_layer<'description>(
    description: &'description Description,
    name: &str,
) -> Result<&'description Layer, ProjectionError> {
    description
        .layers
        .iter()
        .find(|layer| layer.name == name)
        .ok_or_else(|| {
            ProjectionError::new("select", Some(name.to_owned()), "layer does not exist")
        })
}

impl ProjectionError {
    fn new(
        stage: &'static str,
        identifier: Option<impl Into<String>>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            stage,
            identifier: identifier.map(Into::into),
            message: message.into(),
        }
    }
}

impl fmt::Display for ProjectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.identifier {
            Some(identifier) => write!(
                formatter,
                "projection {} for `{}`: {}",
                self.stage, identifier, self.message
            ),
            None => write!(formatter, "projection {}: {}", self.stage, self.message),
        }
    }
}

impl std::error::Error for ProjectionError {}
