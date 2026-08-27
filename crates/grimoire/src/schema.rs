use core::fmt;
use std::collections::BTreeSet;

use crate::{Address, Namespace, Value, Version};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ElementKind {
    Description,
    Block,
    Port,
    Connection,
    Group,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SchemaExprField {
    pub name: String,
    pub schema: Box<SchemaExpr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SchemaExprArm {
    pub tag: String,
    pub schema: Box<SchemaExpr>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SchemaExpr {
    FiniteScalar,
    PositiveInteger,
    FiniteNumber,
    Text,
    Enumeration(Vec<String>),
    Product(Vec<SchemaExprField>),
    Sequence(Box<SchemaExpr>),
    Alternative(Vec<SchemaExprArm>),
    AddressReference,
    Presence(Box<SchemaExpr>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Schema {
    pub namespace: Namespace,
    pub name: String,
    pub version: Version,
    pub allowed_elements: BTreeSet<ElementKind>,
    pub value: SchemaExpr,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchemaError {
    pub path: String,
    pub message: String,
}

impl SchemaExpr {
    pub fn validate(&self, value: &Value) -> Result<(), SchemaError> {
        self.validate_at(value, "$".to_owned())
    }

    fn validate_at(&self, value: &Value, path: String) -> Result<(), SchemaError> {
        match (self, value) {
            (Self::FiniteScalar, Value::Bool(_))
            | (Self::PositiveInteger, Value::PositiveInteger(_))
            | (Self::FiniteNumber, Value::Number(_))
            | (Self::Text, Value::Text(_))
            | (Self::AddressReference, Value::AddressReference(_)) => Ok(()),
            (Self::Enumeration(allowed), Value::Enum(actual))
                if allowed.iter().any(|item| item == actual) =>
            {
                Ok(())
            }
            (Self::Enumeration(_), Value::Enum(actual)) => Err(SchemaError {
                path,
                message: format!("unknown enumeration value `{actual}`"),
            }),
            (Self::Product(fields), Value::Product(values)) => {
                for field in fields {
                    let Some(field_value) = values.get(&field.name) else {
                        return Err(SchemaError {
                            path: format!("{path}.{}", field.name),
                            message: "missing product field".to_owned(),
                        });
                    };
                    field
                        .schema
                        .validate_at(field_value, format!("{path}.{}", field.name))?;
                }
                if let Some(extra) = values
                    .keys()
                    .find(|key| !fields.iter().any(|field| &field.name == *key))
                {
                    return Err(SchemaError {
                        path,
                        message: format!("unknown product field `{extra}`"),
                    });
                }
                Ok(())
            }
            (Self::Sequence(item_schema), Value::Sequence(values)) => {
                values.iter().enumerate().try_for_each(|(index, item)| {
                    item_schema.validate_at(item, format!("{path}[{index}]"))
                })
            }
            (Self::Alternative(arms), Value::Tagged { tag, value }) => {
                let Some(arm) = arms.iter().find(|arm| arm.tag == *tag) else {
                    return Err(SchemaError {
                        path,
                        message: format!("unknown alternative tag `{tag}`"),
                    });
                };
                arm.schema.validate_at(value, format!("{path}.{tag}"))
            }
            (Self::Presence(_), Value::Absent) => Ok(()),
            (Self::Presence(inner), Value::Present(value)) => {
                inner.validate_at(value, format!("{path}.present"))
            }
            (expected, actual) => Err(SchemaError {
                path,
                message: format!("expected {}, got {}", expected.kind(), actual.kind()),
            }),
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            Self::FiniteScalar => "finite scalar",
            Self::PositiveInteger => "positive integer",
            Self::FiniteNumber => "finite number",
            Self::Text => "text",
            Self::Enumeration(_) => "enumeration",
            Self::Product(_) => "product",
            Self::Sequence(_) => "sequence",
            Self::Alternative(_) => "alternative",
            Self::AddressReference => "address reference",
            Self::Presence(_) => "presence",
        }
    }
}

impl Value {
    fn kind(&self) -> &'static str {
        match self {
            Self::Bool(_) => "finite scalar",
            Self::PositiveInteger(_) => "positive integer",
            Self::Number(_) => "finite number",
            Self::Text(_) => "text",
            Self::Enum(_) => "enumeration",
            Self::Product(_) => "product",
            Self::Sequence(_) => "sequence",
            Self::AddressReference(_) => "address reference",
            Self::Absent => "absent",
            Self::Present(_) => "present",
            Self::Tagged { .. } => "alternative",
        }
    }
}

impl fmt::Display for SchemaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.path, self.message)
    }
}

impl std::error::Error for SchemaError {}

#[allow(dead_code)]
fn _address_type_is_used(_: Address) {}
