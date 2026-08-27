use std::collections::BTreeSet;

use crate::{
    ElementKind, Namespace, NamespaceError, Schema, SchemaExpr, SchemaExprArm, SchemaExprField,
    Version,
};

pub const PROTOTYPE_NAMESPACE_ROOT: &str = "https://github.com/goldenwitch/grimoire/extension";

pub fn prototype_schemas() -> Result<Vec<Schema>, NamespaceError> {
    let version = Version::new(1, 0, 0);
    Ok(vec![
        Schema {
            namespace: namespace("axes")?,
            name: "axes".to_owned(),
            version,
            allowed_elements: kinds([ElementKind::Port]),
            value: product(vec![
                field("name", SchemaExpr::Text),
                field("description", presence(SchemaExpr::Text)),
            ]),
        },
        Schema {
            namespace: namespace("shapes")?,
            name: "shapes".to_owned(),
            version,
            allowed_elements: kinds([ElementKind::Port]),
            value: product(vec![
                field(
                    "layout",
                    SchemaExpr::Enumeration(vec![
                        "scalar".to_owned(),
                        "vector".to_owned(),
                        "sequence".to_owned(),
                        "grid".to_owned(),
                        "volume".to_owned(),
                    ]),
                ),
                field(
                    "dimensions",
                    SchemaExpr::Sequence(Box::new(SchemaExpr::Alternative(vec![
                        arm("literal", SchemaExpr::PositiveInteger),
                        arm("symbolic", SchemaExpr::AddressReference),
                    ]))),
                ),
            ]),
        },
        Schema {
            namespace: namespace("architecture")?,
            name: "architecture".to_owned(),
            version,
            allowed_elements: kinds([ElementKind::Block, ElementKind::Port, ElementKind::Group]),
            value: product(vec![
                field("family", SchemaExpr::Text),
                optional_field("parameter_count", SchemaExpr::PositiveInteger),
                optional_field("width", SchemaExpr::PositiveInteger),
                optional_field("depth", SchemaExpr::PositiveInteger),
                optional_field("head_count", SchemaExpr::PositiveInteger),
                optional_field("mlp_width", SchemaExpr::PositiveInteger),
                optional_field("activation", SchemaExpr::Text),
                optional_field("position_encoding", SchemaExpr::Text),
                optional_field(
                    "attention_regime",
                    SchemaExpr::Enumeration(vec![
                        "causal".to_owned(),
                        "bidirectional".to_owned(),
                        "block-causal".to_owned(),
                        "mixed".to_owned(),
                        "unspecified".to_owned(),
                    ]),
                ),
                optional_field("operator", SchemaExpr::Text),
                optional_field("interface", SchemaExpr::AddressReference),
            ]),
        },
        Schema {
            namespace: namespace("training")?,
            name: "training".to_owned(),
            version,
            allowed_elements: kinds([ElementKind::Block, ElementKind::Group]),
            value: product(vec![
                field("objective", SchemaExpr::Text),
                optional_field("optimizer", SchemaExpr::Text),
                optional_field("batch_size", SchemaExpr::PositiveInteger),
                optional_field("steps", SchemaExpr::PositiveInteger),
                field(
                    "phases",
                    SchemaExpr::Sequence(Box::new(product(vec![
                        field("name", SchemaExpr::Text),
                        optional_field("steps", SchemaExpr::PositiveInteger),
                        optional_field("learning_rate", SchemaExpr::FiniteNumber),
                        optional_field("frame_count", SchemaExpr::PositiveInteger),
                        optional_field("resolution", SchemaExpr::AddressReference),
                    ]))),
                ),
                field(
                    "trainable_targets",
                    SchemaExpr::Sequence(Box::new(SchemaExpr::AddressReference)),
                ),
                field(
                    "frozen_targets",
                    SchemaExpr::Sequence(Box::new(SchemaExpr::AddressReference)),
                ),
                field(
                    "data_sources",
                    SchemaExpr::Sequence(Box::new(SchemaExpr::Text)),
                ),
            ]),
        },
        Schema {
            namespace: namespace("execution")?,
            name: "execution".to_owned(),
            version,
            allowed_elements: kinds([ElementKind::Block, ElementKind::Port, ElementKind::Group]),
            value: product(vec![
                field(
                    "regime",
                    SchemaExpr::Enumeration(vec![
                        "static".to_owned(),
                        "streaming".to_owned(),
                        "recurrent".to_owned(),
                        "closed-loop".to_owned(),
                    ]),
                ),
                optional_field("horizon", SchemaExpr::PositiveInteger),
                optional_field("rate", SchemaExpr::FiniteNumber),
                field(
                    "external_consumer",
                    SchemaExpr::Enumeration(vec!["yes".to_owned(), "no".to_owned()]),
                ),
            ]),
        },
        Schema {
            namespace: namespace("precision")?,
            name: "precision".to_owned(),
            version,
            allowed_elements: kinds([ElementKind::Block, ElementKind::Port]),
            value: product(vec![
                optional_field("weights", SchemaExpr::Text),
                optional_field("activations", SchemaExpr::Text),
                optional_field("accumulation", SchemaExpr::Text),
                optional_field("optimizer_state", SchemaExpr::Text),
                optional_field("sparsity", SchemaExpr::Text),
            ]),
        },
        Schema {
            namespace: namespace("placement")?,
            name: "placement".to_owned(),
            version,
            allowed_elements: kinds([
                ElementKind::Description,
                ElementKind::Block,
                ElementKind::Port,
                ElementKind::Connection,
                ElementKind::Group,
            ]),
            value: product(vec![field("location", SchemaExpr::Text)]),
        },
        Schema {
            namespace: namespace("measurement")?,
            name: "measurement".to_owned(),
            version,
            allowed_elements: kinds([
                ElementKind::Description,
                ElementKind::Block,
                ElementKind::Port,
                ElementKind::Connection,
                ElementKind::Group,
            ]),
            value: product(vec![
                field(
                    "value",
                    SchemaExpr::Alternative(vec![
                        arm("integer", SchemaExpr::PositiveInteger),
                        arm("number", SchemaExpr::FiniteNumber),
                    ]),
                ),
                field("unit", SchemaExpr::Text),
                field(
                    "source",
                    product(vec![
                        field("origin", SchemaExpr::Text),
                        optional_field("locator", SchemaExpr::Text),
                        optional_field("protocol", SchemaExpr::Text),
                    ]),
                ),
            ]),
        },
        Schema {
            namespace: namespace("provenance")?,
            name: "provenance".to_owned(),
            version,
            allowed_elements: kinds([ElementKind::Group]),
            value: product(vec![
                field(
                    "citations",
                    SchemaExpr::Sequence(Box::new(SchemaExpr::Text)),
                ),
                field(
                    "assumptions",
                    SchemaExpr::Sequence(Box::new(SchemaExpr::Text)),
                ),
                field(
                    "novelty",
                    SchemaExpr::Enumeration(vec![
                        "novel".to_owned(),
                        "existing".to_owned(),
                        "adapted".to_owned(),
                        "unclassified".to_owned(),
                    ]),
                ),
            ]),
        },
        Schema {
            namespace: namespace("lineage")?,
            name: "lineage".to_owned(),
            version,
            allowed_elements: kinds([ElementKind::Block, ElementKind::Group]),
            value: product(vec![
                field("base", SchemaExpr::AddressReference),
                field(
                    "deltas",
                    SchemaExpr::Sequence(Box::new(SchemaExpr::AddressReference)),
                ),
                field(
                    "operation",
                    SchemaExpr::Enumeration(vec![
                        "continual-update".to_owned(),
                        "sparsify-rescale".to_owned(),
                        "trim-sign-merge".to_owned(),
                    ]),
                ),
                field("result", SchemaExpr::AddressReference),
            ]),
        },
    ])
}

fn namespace(name: &str) -> Result<Namespace, NamespaceError> {
    Namespace::parse(&format!("{PROTOTYPE_NAMESPACE_ROOT}/{name}"))
}

fn kinds<const N: usize>(values: [ElementKind; N]) -> BTreeSet<ElementKind> {
    values.into_iter().collect()
}

fn field(name: &str, schema: SchemaExpr) -> SchemaExprField {
    SchemaExprField {
        name: name.to_owned(),
        schema: Box::new(schema),
    }
}

fn optional_field(name: &str, schema: SchemaExpr) -> SchemaExprField {
    field(name, presence(schema))
}

fn presence(schema: SchemaExpr) -> SchemaExpr {
    SchemaExpr::Presence(Box::new(schema))
}

fn product(fields: Vec<SchemaExprField>) -> SchemaExpr {
    SchemaExpr::Product(fields)
}

fn arm(tag: &str, schema: SchemaExpr) -> SchemaExprArm {
    SchemaExprArm {
        tag: tag.to_owned(),
        schema: Box::new(schema),
    }
}
