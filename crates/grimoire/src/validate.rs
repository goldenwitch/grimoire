use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    Address, Block, Connection, Description, ElementKind, ExtensionParameter, ExtensionValue,
    Group, Layer, LayerInput, Namespace, PROTOTYPE_NAMESPACE_ROOT, Port, Schema, SelectItem, Value,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationError {
    pub check: &'static str,
    pub location: String,
    pub identifier: Option<String>,
    pub message: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Site {
    Core,
    Layer(String),
}

#[derive(Clone, Debug)]
struct Definition {
    kind: ElementKind,
    site: Site,
}

#[derive(Clone, Debug)]
struct Reference {
    target: Address,
    site: Site,
    location: String,
}

struct Context<'description> {
    description: &'description Description,
    layers: BTreeMap<String, &'description Layer>,
    definitions: BTreeMap<Address, Definition>,
    references: Vec<Reference>,
    errors: Vec<ValidationError>,
}

pub fn validate_description(
    description: &Description,
    schemas: &[Schema],
) -> Result<(), Vec<ValidationError>> {
    let mut context = Context::new(description);
    context.check_layers();
    context.collect_definitions();
    context.collect_references();
    context.check_connections();
    context.check_references();
    context.check_locality();
    context.check_extensions(schemas);
    if context.errors.is_empty() {
        Ok(())
    } else {
        Err(context.errors)
    }
}

impl<'description> Context<'description> {
    fn new(description: &'description Description) -> Self {
        let mut layers = BTreeMap::new();
        for layer in &description.layers {
            if layers.insert(layer.name.clone(), layer).is_some() {
                layers.remove(&layer.name);
            }
        }
        Self {
            description,
            layers,
            definitions: BTreeMap::new(),
            references: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn check_layers(&mut self) {
        let mut names = BTreeSet::new();
        for layer in &self.description.layers {
            if !names.insert(layer.name.clone()) {
                self.error(
                    "C1",
                    format!("layer/{}", layer.name),
                    Some(layer.name.clone()),
                    "duplicate layer name",
                );
            }
            if !self.layer_reaches_core(&layer.name, &mut BTreeSet::new()) {
                self.error(
                    "C9",
                    format!("layer/{}/inputs", layer.name),
                    Some(layer.name.clone()),
                    "layer input chain does not reach the core graph",
                );
            }
            for input in &layer.inputs {
                if let LayerInput::Layer(input_name) = input
                    && !self.layers.contains_key(input_name)
                {
                    self.error(
                        "C9",
                        format!("layer/{}/inputs", layer.name),
                        Some(input_name.clone()),
                        "declared layer input does not resolve",
                    );
                }
            }
        }
        for layer in &self.description.layers {
            let mut visiting = BTreeSet::new();
            self.visit_layer_cycle(&layer.name, &mut visiting);
        }
    }

    fn visit_layer_cycle(&mut self, name: &str, visiting: &mut BTreeSet<String>) -> bool {
        if !visiting.insert(name.to_owned()) {
            self.error(
                "C9",
                format!("layer/{name}/inputs"),
                Some(name.to_owned()),
                "declared layer inputs contain a cycle",
            );
            return true;
        }
        let inputs = self
            .layers
            .get(name)
            .map(|layer| layer.inputs.clone())
            .unwrap_or_default();
        let mut has_cycle = false;
        for input in inputs {
            if let LayerInput::Layer(input_name) = input
                && self.layers.contains_key(&input_name)
                && self.visit_layer_cycle(&input_name, visiting)
            {
                has_cycle = true;
            }
        }
        visiting.remove(name);
        has_cycle
    }

    fn collect_definitions(&mut self) {
        self.add_definition(
            self.description.address.clone(),
            ElementKind::Description,
            Site::Core,
            "description",
        );
        for (address, block) in &self.description.core.blocks {
            self.add_block_definition(address, block, Site::Core, "core");
        }
        for (address, connection) in &self.description.core.connections {
            self.add_connection_definition(address, connection, Site::Core, "core");
        }
        for (address, group) in &self.description.core.groups {
            self.add_group_definition(address, group, Site::Core, "core");
        }
        for layer in &self.description.layers {
            let site = Site::Layer(layer.name.clone());
            for item in &layer.projection.select {
                match item {
                    SelectItem::Use(_) => {}
                    SelectItem::GenerateBlock(block) => {
                        self.add_block_definition(
                            &block.address,
                            block,
                            site.clone(),
                            &format!("layer/{}/select", layer.name),
                        );
                    }
                    SelectItem::GenerateConnection(connection) => {
                        self.add_connection_definition(
                            &connection.address,
                            connection,
                            site.clone(),
                            &format!("layer/{}/select", layer.name),
                        );
                    }
                    SelectItem::GenerateGroup(group) => {
                        self.add_group_definition(
                            &group.address,
                            group,
                            site.clone(),
                            &format!("layer/{}/select", layer.name),
                        );
                    }
                }
            }
        }
    }

    fn add_block_definition(
        &mut self,
        address: &Address,
        block: &Block,
        site: Site,
        location: &str,
    ) {
        if address != &block.address {
            self.error(
                "C5",
                format!("{location}/block/{address}"),
                Some(address.to_string()),
                "block map key and block address differ",
            );
        }
        if block.name.is_empty() {
            self.error(
                "C5",
                format!("{location}/block/{address}"),
                Some(address.to_string()),
                "block requires a human name",
            );
        }
        self.add_definition(
            block.address.clone(),
            ElementKind::Block,
            site.clone(),
            &format!("{location}/block/{address}"),
        );
        for (port_address, port) in &block.ports {
            self.add_port_definition(
                port_address,
                port,
                site.clone(),
                &format!("{location}/block/{address}"),
            );
        }
    }

    fn add_port_definition(&mut self, address: &Address, port: &Port, site: Site, location: &str) {
        if address != &port.address {
            self.error(
                "C5",
                format!("{location}/port/{address}"),
                Some(address.to_string()),
                "port map key and port address differ",
            );
        }
        self.add_definition(
            port.address.clone(),
            ElementKind::Port,
            site,
            &format!("{location}/port/{address}"),
        );
    }

    fn add_connection_definition(
        &mut self,
        address: &Address,
        connection: &Connection,
        site: Site,
        location: &str,
    ) {
        if address != &connection.address {
            self.error(
                "C5",
                format!("{location}/connection/{address}"),
                Some(address.to_string()),
                "connection map key and connection address differ",
            );
        }
        self.add_definition(
            connection.address.clone(),
            ElementKind::Connection,
            site,
            &format!("{location}/connection/{address}"),
        );
    }

    fn add_group_definition(
        &mut self,
        address: &Address,
        group: &Group,
        site: Site,
        location: &str,
    ) {
        if address != &group.address {
            self.error(
                "C5",
                format!("{location}/group/{address}"),
                Some(address.to_string()),
                "group map key and group address differ",
            );
        }
        self.add_definition(
            group.address.clone(),
            ElementKind::Group,
            site,
            &format!("{location}/group/{address}"),
        );
    }

    fn add_definition(&mut self, address: Address, kind: ElementKind, site: Site, location: &str) {
        if self.definitions.contains_key(&address) {
            self.error(
                "C1",
                location.to_owned(),
                Some(address.to_string()),
                "address is defined more than once",
            );
            self.error(
                "C4",
                location.to_owned(),
                Some(address.to_string()),
                "element has more than one definition site",
            );
            return;
        }
        self.definitions.insert(address, Definition { kind, site });
    }

    fn collect_references(&mut self) {
        let core_site = Site::Core;
        for connection in self.description.core.connections.values() {
            self.reference(
                connection.source.clone(),
                core_site.clone(),
                format!("core/connection/{}/source", connection.address),
            );
            self.reference(
                connection.destination.clone(),
                core_site.clone(),
                format!("core/connection/{}/destination", connection.address),
            );
        }
        for group in self.description.core.groups.values() {
            for (index, member) in group.members.iter().enumerate() {
                self.reference(
                    member.clone(),
                    core_site.clone(),
                    format!("core/group/{}/member/{index}", group.address),
                );
            }
        }
        self.collect_element_extensions(
            &self.description.extensions,
            core_site.clone(),
            "description/extensions",
        );
        for (address, block) in &self.description.core.blocks {
            self.collect_element_extensions(
                &block.extensions,
                core_site.clone(),
                &format!("core/block/{address}/extensions"),
            );
            for (port_address, port) in &block.ports {
                self.collect_element_extensions(
                    &port.extensions,
                    core_site.clone(),
                    &format!("core/block/{address}/port/{port_address}/extensions"),
                );
            }
        }
        for (address, connection) in &self.description.core.connections {
            self.collect_element_extensions(
                &connection.extensions,
                core_site.clone(),
                &format!("core/connection/{address}/extensions"),
            );
        }
        for (address, group) in &self.description.core.groups {
            self.collect_element_extensions(
                &group.extensions,
                core_site.clone(),
                &format!("core/group/{address}/extensions"),
            );
        }
        for layer in &self.description.layers {
            let site = Site::Layer(layer.name.clone());
            for (index, item) in layer.projection.select.iter().enumerate() {
                match item {
                    SelectItem::Use(addresses) => {
                        for (address_index, address) in addresses.iter().enumerate() {
                            self.reference(
                                address.clone(),
                                site.clone(),
                                format!("layer/{}/select/use/{index}/{address_index}", layer.name),
                            );
                        }
                    }
                    SelectItem::GenerateBlock(block) => {
                        self.collect_block_references(
                            block,
                            site.clone(),
                            &format!("layer/{}/select/block/{index}", layer.name),
                        );
                    }
                    SelectItem::GenerateConnection(connection) => {
                        self.reference(
                            connection.source.clone(),
                            site.clone(),
                            format!("layer/{}/select/connection/{index}/source", layer.name),
                        );
                        self.reference(
                            connection.destination.clone(),
                            site.clone(),
                            format!("layer/{}/select/connection/{index}/destination", layer.name),
                        );
                        self.collect_element_extensions(
                            &connection.extensions,
                            site.clone(),
                            &format!("layer/{}/select/connection/{index}/extensions", layer.name),
                        );
                    }
                    SelectItem::GenerateGroup(group) => {
                        for (member_index, member) in group.members.iter().enumerate() {
                            self.reference(
                                member.clone(),
                                site.clone(),
                                format!(
                                    "layer/{}/select/group/{index}/member/{member_index}",
                                    layer.name
                                ),
                            );
                        }
                        self.collect_element_extensions(
                            &group.extensions,
                            site.clone(),
                            &format!("layer/{}/select/group/{index}/extensions", layer.name),
                        );
                    }
                }
            }
            for (index, group) in layer.projection.invert.iter().enumerate() {
                self.reference(
                    group.clone(),
                    site.clone(),
                    format!("layer/{}/invert/{index}", layer.name),
                );
            }
            for (index, decoration) in layer.projection.decorate.iter().enumerate() {
                self.reference(
                    decoration.target.clone(),
                    site.clone(),
                    format!("layer/{}/decorate/{index}/target", layer.name),
                );
                self.collect_element_extensions(
                    std::slice::from_ref(&decoration.parameter),
                    site.clone(),
                    &format!("layer/{}/decorate/{index}/parameter", layer.name),
                );
            }
        }
    }

    fn collect_block_references(&mut self, block: &Block, site: Site, location: &str) {
        self.collect_element_extensions(
            &block.extensions,
            site.clone(),
            &format!("{location}/extensions"),
        );
        for (port_address, port) in &block.ports {
            self.collect_element_extensions(
                &port.extensions,
                site.clone(),
                &format!("{location}/port/{port_address}/extensions"),
            );
        }
    }

    fn collect_element_extensions(
        &mut self,
        extensions: &[ExtensionParameter],
        site: Site,
        location: &str,
    ) {
        for (index, extension) in extensions.iter().enumerate() {
            if let ExtensionValue::Known(value) = &extension.value {
                self.collect_value_references(
                    value,
                    site.clone(),
                    &format!("{location}/{index}/value"),
                );
            }
        }
    }

    fn collect_value_references(&mut self, value: &Value, site: Site, location: &str) {
        match value {
            Value::AddressReference(address) => {
                self.reference(address.clone(), site, location.to_owned());
            }
            Value::Product(fields) => {
                for (name, field) in fields {
                    self.collect_value_references(
                        field,
                        site.clone(),
                        &format!("{location}/{name}"),
                    );
                }
            }
            Value::Sequence(values) => {
                for (index, item) in values.iter().enumerate() {
                    self.collect_value_references(
                        item,
                        site.clone(),
                        &format!("{location}/{index}"),
                    );
                }
            }
            Value::Present(value) => self.collect_value_references(value, site, location),
            Value::Tagged { value, .. } => self.collect_value_references(value, site, location),
            Value::Bool(_)
            | Value::PositiveInteger(_)
            | Value::Number(_)
            | Value::Text(_)
            | Value::Enum(_)
            | Value::Absent => {}
        }
    }

    fn reference(&mut self, target: Address, site: Site, location: String) {
        self.references.push(Reference {
            target,
            site,
            location,
        });
    }

    fn check_connections(&mut self) {
        for connection in self.description.core.connections.values() {
            self.check_connection(
                connection,
                &format!("core/connection/{}", connection.address),
            );
        }
        for layer in &self.description.layers {
            for item in &layer.projection.select {
                if let SelectItem::GenerateConnection(connection) = item {
                    self.check_connection(
                        connection,
                        &format!("layer/{}/connection/{}", layer.name, connection.address),
                    );
                }
            }
        }
    }

    fn check_connection(&mut self, connection: &Connection, location: &str) {
        for (role, address) in [
            ("source", &connection.source),
            ("destination", &connection.destination),
        ] {
            match self.definitions.get(address) {
                Some(definition) if definition.kind == ElementKind::Port => {}
                Some(definition) => self.error(
                    "C2",
                    format!("{location}/{role}"),
                    Some(address.to_string()),
                    format!("connection endpoint is {:?}, not a port", definition.kind),
                ),
                None => self.error(
                    "C2",
                    format!("{location}/{role}"),
                    Some(address.to_string()),
                    "connection endpoint does not resolve to a port",
                ),
            }
        }
    }

    fn check_references(&mut self) {
        for reference in self.references.clone() {
            let Some(definition) = self.definitions.get(&reference.target) else {
                self.error(
                    "C6",
                    reference.location,
                    Some(reference.target.to_string()),
                    "reference does not resolve to an element",
                );
                continue;
            };
            if !self.site_visible(&definition.site, &reference.site) {
                self.error(
                    "C6",
                    reference.location,
                    Some(reference.target.to_string()),
                    "reference is below the definition site or outside declared inputs",
                );
            }
        }
    }

    fn check_locality(&mut self) {
        let mut references_by_address: BTreeMap<Address, Vec<Reference>> = BTreeMap::new();
        for reference in &self.references {
            references_by_address
                .entry(reference.target.clone())
                .or_default()
                .push(reference.clone());
        }
        for (address, references) in references_by_address {
            let Some(definition_site) = self
                .definitions
                .get(&address)
                .map(|definition| definition.site.clone())
            else {
                continue;
            };
            if definition_site == Site::Core {
                continue;
            }
            let mut candidate_sites = BTreeSet::from([Site::Core]);
            candidate_sites.extend(
                self.description
                    .layers
                    .iter()
                    .map(|layer| Site::Layer(layer.name.clone())),
            );
            let candidate_sites: Vec<Site> = candidate_sites.into_iter().collect();
            let legal_sites: Vec<&Site> = candidate_sites
                .iter()
                .filter(|candidate| {
                    references
                        .iter()
                        .all(|reference| self.site_visible(candidate, &reference.site))
                })
                .collect();
            let maximal = legal_sites
                .iter()
                .filter(|candidate| {
                    let candidate: Site = (**candidate).clone();
                    !legal_sites.iter().any(|other| {
                        let other: Site = (**other).clone();
                        candidate != other && self.site_below(&candidate, &other)
                    })
                })
                .any(|candidate| *candidate == &definition_site);
            if !legal_sites.is_empty() && !maximal {
                self.error(
                    "C7",
                    "definition-sites".to_owned(),
                    Some(address.to_string()),
                    "element is defined below a maximal site visible to every reference",
                );
            }
        }
    }

    fn check_extensions(&mut self, schemas: &[Schema]) {
        let schema_map: BTreeMap<(&Namespace, &str, crate::Version), &Schema> = schemas
            .iter()
            .map(|schema| {
                (
                    (&schema.namespace, schema.name.as_str(), schema.version),
                    schema,
                )
            })
            .collect();
        self.check_extension_list(
            &self.description.extensions,
            ElementKind::Description,
            "description/extensions",
            &schema_map,
        );
        for (address, block) in &self.description.core.blocks {
            self.check_extension_list(
                &block.extensions,
                ElementKind::Block,
                &format!("core/block/{address}/extensions"),
                &schema_map,
            );
            for (port_address, port) in &block.ports {
                self.check_extension_list(
                    &port.extensions,
                    ElementKind::Port,
                    &format!("core/block/{address}/port/{port_address}/extensions"),
                    &schema_map,
                );
            }
        }
        for (address, connection) in &self.description.core.connections {
            self.check_extension_list(
                &connection.extensions,
                ElementKind::Connection,
                &format!("core/connection/{address}/extensions"),
                &schema_map,
            );
        }
        for (address, group) in &self.description.core.groups {
            self.check_extension_list(
                &group.extensions,
                ElementKind::Group,
                &format!("core/group/{address}/extensions"),
                &schema_map,
            );
        }
        for layer in &self.description.layers {
            for (index, item) in layer.projection.select.iter().enumerate() {
                match item {
                    SelectItem::GenerateBlock(block) => self.check_block_extensions(
                        block,
                        &format!("layer/{}/select/block/{index}", layer.name),
                        &schema_map,
                    ),
                    SelectItem::GenerateConnection(connection) => self.check_extension_list(
                        &connection.extensions,
                        ElementKind::Connection,
                        &format!("layer/{}/select/connection/{index}", layer.name),
                        &schema_map,
                    ),
                    SelectItem::GenerateGroup(group) => self.check_extension_list(
                        &group.extensions,
                        ElementKind::Group,
                        &format!("layer/{}/select/group/{index}", layer.name),
                        &schema_map,
                    ),
                    SelectItem::Use(_) => {}
                }
            }
            for (index, decoration) in layer.projection.decorate.iter().enumerate() {
                self.check_extension_parameter(
                    &decoration.parameter,
                    self.definitions
                        .get(&decoration.target)
                        .map_or(ElementKind::Block, |definition| definition.kind),
                    &format!("layer/{}/decorate/{index}", layer.name),
                    &schema_map,
                );
            }
        }
    }

    fn check_block_extensions(
        &mut self,
        block: &Block,
        location: &str,
        schemas: &BTreeMap<(&Namespace, &str, crate::Version), &Schema>,
    ) {
        self.check_extension_list(&block.extensions, ElementKind::Block, location, schemas);
        for (address, port) in &block.ports {
            self.check_extension_list(
                &port.extensions,
                ElementKind::Port,
                &format!("{location}/port/{address}"),
                schemas,
            );
        }
    }

    fn check_extension_list(
        &mut self,
        extensions: &[ExtensionParameter],
        kind: ElementKind,
        location: &str,
        schemas: &BTreeMap<(&Namespace, &str, crate::Version), &Schema>,
    ) {
        for (index, extension) in extensions.iter().enumerate() {
            self.check_extension_parameter(
                extension,
                kind,
                &format!("{location}/{index}"),
                schemas,
            );
        }
    }

    fn check_extension_parameter(
        &mut self,
        extension: &ExtensionParameter,
        kind: ElementKind,
        location: &str,
        schemas: &BTreeMap<(&Namespace, &str, crate::Version), &Schema>,
    ) {
        if !is_identifier(&extension.name) || !is_identifier(&extension.schema) {
            self.error(
                "C10",
                location.to_owned(),
                Some(extension.name.clone()),
                "extension parameter or schema name is not a valid identifier",
            );
            return;
        }
        let known_namespace = extension.namespace.as_str() == PROTOTYPE_NAMESPACE_ROOT
            || extension
                .namespace
                .as_str()
                .strip_prefix(PROTOTYPE_NAMESPACE_ROOT)
                .is_some_and(|rest| rest.starts_with('/'));
        if !known_namespace {
            if !matches!(extension.value, ExtensionValue::Opaque(_)) {
                self.error(
                    "C10",
                    location.to_owned(),
                    Some(extension.name.clone()),
                    "unrecognized namespace must remain opaque",
                );
            }
            return;
        }
        let Some(schema) = schemas.get(&(
            &extension.namespace,
            extension.schema.as_str(),
            extension.version,
        )) else {
            self.error(
                "C10",
                location.to_owned(),
                Some(extension.name.clone()),
                "recognized extension schema does not resolve",
            );
            return;
        };
        let ExtensionValue::Known(value) = &extension.value else {
            self.error(
                "C10",
                location.to_owned(),
                Some(extension.name.clone()),
                "recognized namespace cannot use an opaque value",
            );
            return;
        };
        if let Err(error) = schema.validate(kind, value) {
            self.error(
                "C10",
                format!("{location}/value{}", error.path),
                Some(extension.name.clone()),
                error.message,
            );
        }
    }

    fn site_visible(&self, definition: &Site, reference: &Site) -> bool {
        match reference {
            Site::Core => definition == &Site::Core,
            Site::Layer(reference_name) => match definition {
                Site::Core => self.layer_reaches_core(reference_name, &mut BTreeSet::new()),
                Site::Layer(definition_name) => {
                    definition_name == reference_name
                        || self.layer_reaches_layer(
                            reference_name,
                            definition_name,
                            &mut BTreeSet::new(),
                        )
                }
            },
        }
    }

    fn site_below(&self, below: &Site, above: &Site) -> bool {
        below != above && self.site_visible(below, above)
    }

    fn layer_reaches_core(&self, name: &str, visiting: &mut BTreeSet<String>) -> bool {
        if !visiting.insert(name.to_owned()) {
            return false;
        }
        let Some(layer) = self.layers.get(name) else {
            return false;
        };
        let reaches = layer.inputs.iter().any(|input| match input {
            LayerInput::Core => true,
            LayerInput::Layer(input_name) => self.layer_reaches_core(input_name, visiting),
        });
        visiting.remove(name);
        reaches
    }

    fn layer_reaches_layer(
        &self,
        reference_name: &str,
        definition_name: &str,
        visiting: &mut BTreeSet<String>,
    ) -> bool {
        if !visiting.insert(reference_name.to_owned()) {
            return false;
        }
        let Some(layer) = self.layers.get(reference_name) else {
            return false;
        };
        let reaches = layer.inputs.iter().any(|input| match input {
            LayerInput::Core => false,
            LayerInput::Layer(input_name) => {
                input_name == definition_name
                    || self.layer_reaches_layer(input_name, definition_name, visiting)
            }
        });
        visiting.remove(reference_name);
        reaches
    }

    fn error(
        &mut self,
        check: &'static str,
        location: String,
        identifier: Option<String>,
        message: impl Into<String>,
    ) {
        self.errors.push(ValidationError {
            check,
            location,
            identifier,
            message: message.into(),
        });
    }
}

fn is_identifier(value: &str) -> bool {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    first.is_ascii_alphabetic()
        && characters.all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '-'
        })
}

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.identifier {
            Some(identifier) => write!(
                formatter,
                "{} at {} for `{}`: {}",
                self.check, self.location, identifier, self.message
            ),
            None => write!(
                formatter,
                "{} at {}: {}",
                self.check, self.location, self.message
            ),
        }
    }
}

impl std::error::Error for ValidationError {}
