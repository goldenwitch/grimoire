use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

use crate::{Address, Decoration, Element, ExtensionValue, StructuralReprojection, Value};

pub const PLACEMENT_NAMESPACE: &str = "https://github.com/goldenwitch/grimoire/extension/placement";
pub const PLACEMENT_PARAMETER: &str = "placement";
pub const PLACEMENT_SCHEMA: &str = "placement";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShapeDimension {
    Literal(u64),
    Axis(Address),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TensorShape {
    dimensions: Vec<ShapeDimension>,
    bytes_per_element: u64,
}

impl TensorShape {
    pub fn new(
        dimensions: Vec<ShapeDimension>,
        bytes_per_element: u64,
    ) -> Result<Self, PlacementError> {
        if bytes_per_element == 0 {
            return Err(PlacementError::InvalidElementWidth);
        }
        for (index, dimension) in dimensions.iter().enumerate() {
            if let ShapeDimension::Literal(value) = dimension
                && *value == 0
            {
                return Err(PlacementError::ZeroShapeDimension { index });
            }
        }
        Ok(Self {
            dimensions,
            bytes_per_element,
        })
    }

    #[must_use]
    pub fn dimensions(&self) -> &[ShapeDimension] {
        &self.dimensions
    }

    #[must_use]
    pub const fn bytes_per_element(&self) -> u64 {
        self.bytes_per_element
    }

    pub fn byte_size(&self, axes: &BTreeMap<Address, u64>) -> Result<u64, PlacementError> {
        let mut elements = 1u64;
        for dimension in &self.dimensions {
            let extent = match dimension {
                ShapeDimension::Literal(value) => *value,
                ShapeDimension::Axis(address) => *axes
                    .get(address)
                    .ok_or_else(|| PlacementError::MissingAxis(address.clone()))?,
            };
            if extent == 0 {
                let ShapeDimension::Axis(address) = dimension else {
                    unreachable!("zero literal dimensions are rejected by TensorShape::new");
                };
                return Err(PlacementError::ZeroResolvedAxis(address.clone()));
            }
            elements = elements
                .checked_mul(extent)
                .ok_or(PlacementError::ShapeSizeOverflow)?;
        }
        elements
            .checked_mul(self.bytes_per_element)
            .ok_or(PlacementError::ShapeSizeOverflow)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Placement {
    locations: BTreeMap<Address, String>,
}

impl Placement {
    pub fn new(assignments: Vec<(Address, String)>) -> Result<Self, PlacementError> {
        let mut locations = BTreeMap::new();
        for (address, location) in assignments {
            if location.trim().is_empty() {
                return Err(PlacementError::EmptyLocation(address));
            }
            if locations.insert(address.clone(), location).is_some() {
                return Err(PlacementError::DuplicatePlacement(address));
            }
        }
        Ok(Self { locations })
    }

    pub fn from_decorations(decorations: &[Decoration]) -> Result<Self, PlacementError> {
        let mut assignments = Vec::new();
        for decoration in decorations {
            let parameter = &decoration.parameter;
            if parameter.namespace.as_str() != PLACEMENT_NAMESPACE
                || parameter.name != PLACEMENT_PARAMETER
                || parameter.schema != PLACEMENT_SCHEMA
            {
                continue;
            }
            let ExtensionValue::Known(Value::Product(fields)) = &parameter.value else {
                return Err(PlacementError::InvalidPlacementValue(
                    decoration.target.clone(),
                ));
            };
            let Some(Value::Text(location)) = fields.get("location") else {
                return Err(PlacementError::InvalidPlacementValue(
                    decoration.target.clone(),
                ));
            };
            assignments.push((decoration.target.clone(), location.clone()));
        }
        Self::new(assignments)
    }

    #[must_use]
    pub fn assigned_location(&self, address: &Address) -> Option<&str> {
        self.locations.get(address).map(String::as_str)
    }

    fn location_for(
        &self,
        address: &Address,
        reprojection: &StructuralReprojection,
    ) -> Result<String, PlacementError> {
        if !reprojection.elements.contains_key(address) {
            return Err(PlacementError::UnknownAddress(address.clone()));
        }
        if let Some(location) = self.locations.get(address) {
            return Ok(location.clone());
        }
        if let Some(owner) = owner_block(address, reprojection)
            && let Some(location) = self.locations.get(&owner)
        {
            return Ok(location.clone());
        }
        Err(PlacementError::MissingPlacement(address.clone()))
    }
}

fn owner_block(address: &Address, reprojection: &StructuralReprojection) -> Option<Address> {
    reprojection.elements.values().find_map(|element| {
        if let Element::Block(block) = element
            && block.ports.contains_key(address)
        {
            Some(block.address.clone())
        } else {
            None
        }
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollectiveTransfer {
    pub source: Address,
    pub destination: Address,
}

impl CollectiveTransfer {
    #[must_use]
    pub const fn new(source: Address, destination: Address) -> Self {
        Self {
            source,
            destination,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Collective {
    pub address: Address,
    pub payload: Address,
    pub transfers: Vec<CollectiveTransfer>,
}

impl Collective {
    pub fn new(
        address: Address,
        payload: Address,
        transfers: Vec<CollectiveTransfer>,
    ) -> Result<Self, PlacementError> {
        if transfers.is_empty() {
            return Err(PlacementError::EmptyCollectiveTransfers(address));
        }
        let mut seen = BTreeSet::new();
        for transfer in &transfers {
            if !seen.insert((transfer.source.clone(), transfer.destination.clone())) {
                return Err(PlacementError::DuplicateCollectiveTransfer {
                    collective: address,
                    source: transfer.source.clone(),
                    destination: transfer.destination.clone(),
                });
            }
        }
        Ok(Self {
            address,
            payload,
            transfers,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WireTransfer {
    pub relation: Address,
    pub source: Address,
    pub destination: Address,
    pub bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BandwidthReport {
    pub transfers: Vec<WireTransfer>,
    total_bytes: u64,
}

impl BandwidthReport {
    #[must_use]
    pub const fn total_bytes(&self) -> u64 {
        self.total_bytes
    }
}

pub fn bytes_on_wire(
    reprojection: &StructuralReprojection,
    placement: &Placement,
    shapes: &BTreeMap<Address, TensorShape>,
    axes: &BTreeMap<Address, u64>,
    collectives: &[Collective],
) -> Result<BandwidthReport, PlacementError> {
    let mut transfers = Vec::new();
    let mut relation_addresses = BTreeSet::new();
    let mut total_bytes = 0u64;

    for (relation, element) in &reprojection.elements {
        let Element::Connection(connection) = element else {
            continue;
        };
        relation_addresses.insert(relation.clone());
        let source_location = placement.location_for(&connection.source, reprojection)?;
        let destination_location = placement.location_for(&connection.destination, reprojection)?;
        if source_location == destination_location {
            continue;
        }
        let shape = shapes
            .get(&connection.source)
            .ok_or_else(|| PlacementError::MissingShape(connection.source.clone()))?;
        let bytes = shape.byte_size(axes)?;
        total_bytes = total_bytes
            .checked_add(bytes)
            .ok_or(PlacementError::WireSizeOverflow)?;
        transfers.push(WireTransfer {
            relation: relation.clone(),
            source: connection.source.clone(),
            destination: connection.destination.clone(),
            bytes,
        });
    }

    for collective in collectives {
        if !relation_addresses.insert(collective.address.clone()) {
            return Err(PlacementError::DuplicateRelation(
                collective.address.clone(),
            ));
        }
        if !matches!(
            reprojection.elements.get(&collective.address),
            Some(Element::Block(_))
        ) {
            return Err(PlacementError::CollectiveElementMissing(
                collective.address.clone(),
            ));
        }
        let shape = shapes
            .get(&collective.payload)
            .ok_or_else(|| PlacementError::MissingShape(collective.payload.clone()))?;
        let bytes = shape.byte_size(axes)?;
        for transfer in &collective.transfers {
            let source_location = placement.location_for(&transfer.source, reprojection)?;
            let destination_location =
                placement.location_for(&transfer.destination, reprojection)?;
            if source_location == destination_location {
                continue;
            }
            total_bytes = total_bytes
                .checked_add(bytes)
                .ok_or(PlacementError::WireSizeOverflow)?;
            transfers.push(WireTransfer {
                relation: collective.address.clone(),
                source: transfer.source.clone(),
                destination: transfer.destination.clone(),
                bytes,
            });
        }
    }

    transfers.sort_by(|left, right| {
        left.relation
            .cmp(&right.relation)
            .then_with(|| left.source.cmp(&right.source))
            .then_with(|| left.destination.cmp(&right.destination))
    });
    Ok(BandwidthReport {
        transfers,
        total_bytes,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlacementError {
    InvalidElementWidth,
    ZeroShapeDimension {
        index: usize,
    },
    MissingAxis(Address),
    ZeroResolvedAxis(Address),
    ShapeSizeOverflow,
    EmptyLocation(Address),
    DuplicatePlacement(Address),
    InvalidPlacementValue(Address),
    UnknownAddress(Address),
    MissingPlacement(Address),
    EmptyCollectiveTransfers(Address),
    DuplicateCollectiveTransfer {
        collective: Address,
        source: Address,
        destination: Address,
    },
    DuplicateRelation(Address),
    CollectiveElementMissing(Address),
    MissingShape(Address),
    WireSizeOverflow,
}

impl fmt::Display for PlacementError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidElementWidth => {
                formatter.write_str("tensor element width must be positive")
            }
            Self::ZeroShapeDimension { index } => {
                write!(formatter, "tensor shape dimension {index} must be positive")
            }
            Self::MissingAxis(address) => {
                write!(formatter, "shape axis `{address}` has no extent")
            }
            Self::ZeroResolvedAxis(address) => {
                write!(formatter, "shape axis `{address}` resolved to zero")
            }
            Self::ShapeSizeOverflow => formatter.write_str("tensor shape byte size overflowed"),
            Self::EmptyLocation(address) => {
                write!(formatter, "placement for `{address}` is empty")
            }
            Self::DuplicatePlacement(address) => {
                write!(
                    formatter,
                    "placement for `{address}` is declared more than once"
                )
            }
            Self::InvalidPlacementValue(address) => {
                write!(
                    formatter,
                    "placement decoration on `{address}` has no text location"
                )
            }
            Self::UnknownAddress(address) => {
                write!(formatter, "placement address `{address}` is not visible")
            }
            Self::MissingPlacement(address) => {
                write!(
                    formatter,
                    "no placement is assigned to `{address}` or its owning block"
                )
            }
            Self::EmptyCollectiveTransfers(address) => {
                write!(
                    formatter,
                    "collective `{address}` needs an explicit transfer"
                )
            }
            Self::DuplicateCollectiveTransfer {
                collective,
                source,
                destination,
            } => write!(
                formatter,
                "collective `{collective}` repeats transfer `{source}` -> `{destination}`"
            ),
            Self::DuplicateRelation(address) => {
                write!(
                    formatter,
                    "placement relation `{address}` is declared more than once"
                )
            }
            Self::CollectiveElementMissing(address) => write!(
                formatter,
                "collective `{address}` does not name a visible block element"
            ),
            Self::MissingShape(address) => {
                write!(formatter, "no tensor shape is supplied for `{address}`")
            }
            Self::WireSizeOverflow => formatter.write_str("wire byte total overflowed"),
        }
    }
}

impl std::error::Error for PlacementError {}
