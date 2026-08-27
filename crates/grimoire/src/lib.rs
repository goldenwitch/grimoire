mod address;
mod cut;
mod model;
mod namespace;
mod parser;
mod projection;
mod schema;
mod schemas;
mod validate;
mod value;
mod version;

pub use address::{Address, AddressError};
pub use cut::{CutError, extract_cut};
pub use model::{
    Block, Connection, CoreGraph, Description, ExtensionParameter, ExtensionValue, Group, Port,
};
pub use namespace::{Namespace, NamespaceError};
pub use parser::{ParseError, parse_description};
pub use projection::{
    Check, Decoration, ExpectedCardinality, Layer, LayerInput, Projection, SchemaUse, SelectItem,
};
pub use schema::{ElementKind, Schema, SchemaError, SchemaExpr, SchemaExprArm, SchemaExprField};
mod serialize;
pub use schemas::{PROTOTYPE_NAMESPACE_ROOT, prototype_schemas};
pub use serialize::{SerializeError, serialize_description};
pub use validate::{ValidationError, validate_description};
pub use value::{FiniteNumber, Value};
pub use version::{Version, VersionError};
