use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

use crate::{Description, Layer, LayerInput, Schema, ValidationError, validate_description};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CutError {
    UnknownLayer(String),
    Unresolvable { layer: String, missing: Vec<String> },
    Invalid(Vec<ValidationError>),
}

pub fn extract_cut(
    description: &Description,
    selected_layers: &[&str],
    schemas: &[Schema],
) -> Result<Description, CutError> {
    let layers_by_name: BTreeMap<&str, &Layer> = description
        .layers
        .iter()
        .map(|layer| (layer.name.as_str(), layer))
        .collect();
    let selected: BTreeSet<&str> = selected_layers.iter().copied().collect();
    for name in &selected {
        if !layers_by_name.contains_key(name) {
            return Err(CutError::UnknownLayer((*name).to_owned()));
        }
    }
    for name in &selected {
        let Some(layer) = layers_by_name.get(name) else {
            continue;
        };
        let missing: Vec<String> = layer
            .inputs
            .iter()
            .filter_map(|input| match input {
                LayerInput::Core => None,
                LayerInput::Layer(input_name) if selected.contains(input_name.as_str()) => None,
                LayerInput::Layer(input_name) => Some(input_name.clone()),
            })
            .collect();
        if !missing.is_empty() {
            return Err(CutError::Unresolvable {
                layer: (*name).to_owned(),
                missing,
            });
        }
    }
    let mut cut = description.clone();
    cut.layers = description
        .layers
        .iter()
        .filter(|layer| selected.contains(layer.name.as_str()))
        .cloned()
        .collect();
    validate_description(&cut, schemas).map_err(CutError::Invalid)?;
    Ok(cut)
}

impl fmt::Display for CutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownLayer(layer) => write!(formatter, "C11 unknown layer `{layer}`"),
            Self::Unresolvable { layer, missing } => write!(
                formatter,
                "C12 layer `{layer}` is unresolvable; absent inputs: {}",
                missing
                    .iter()
                    .map(|name| format!("`{name}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Invalid(errors) => write!(
                formatter,
                "C11 extracted cut is invalid: {}",
                errors
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        }
    }
}

impl std::error::Error for CutError {}
