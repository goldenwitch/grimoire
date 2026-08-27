use grimoire::{
    Address, ElementKind, ExtensionParameter, ExtensionValue, Schema, Value, Version,
    prototype_schemas,
};

fn schemas() -> Vec<Schema> {
    prototype_schemas().unwrap_or_else(|error| panic!("{error}"))
}

fn schema<'schemas>(schemas: &'schemas [Schema], name: &str) -> &'schemas Schema {
    schemas
        .iter()
        .find(|schema| schema.name == name)
        .unwrap_or_else(|| panic!("missing schema {name}"))
}

fn product(fields: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    Value::Product(
        fields
            .into_iter()
            .map(|(name, value)| (name.to_owned(), value))
            .collect(),
    )
}

fn text(value: &str) -> Value {
    Value::Text(value.to_owned())
}

fn sequence(values: Vec<Value>) -> Value {
    Value::Sequence(values)
}

fn tagged(tag: &str, value: Value) -> Value {
    Value::Tagged {
        tag: tag.to_owned(),
        value: Box::new(value),
    }
}

fn reference(value: &str) -> Value {
    Value::AddressReference(Address::parse(value).unwrap_or_else(|error| panic!("{error}")))
}

#[test]
fn registry_has_one_schema_per_shared_family() {
    let schemas = schemas();
    assert_eq!(schemas.len(), 9);
    assert!(
        schemas
            .iter()
            .all(|schema| schema.version == Version::new(1, 0, 0))
    );
    assert!(schemas.iter().all(|schema| {
        schema
            .namespace
            .as_str()
            .starts_with("https://github.com/goldenwitch/grimoire/extension/")
    }));
}

#[test]
fn axes_and_shapes_cover_symbolic_dimensions() {
    let schemas = schemas();
    let axes = schema(&schemas, "axes");
    let axis_value = product([("name", text("frames")), ("description", Value::Absent)]);
    assert!(axes.validate(ElementKind::Port, &axis_value).is_ok());

    let shapes = schema(&schemas, "shapes");
    let shape_value = product([
        ("layout", Value::Enum("grid".to_owned())),
        (
            "dimensions",
            sequence(vec![
                tagged("symbolic", reference("@axis/frames")),
                tagged("literal", Value::PositiveInteger(1408)),
            ]),
        ),
    ]);
    assert!(shapes.validate(ElementKind::Port, &shape_value).is_ok());
}

#[test]
fn execution_precision_and_provenance_cover_runtime_cases() {
    let schemas = schemas();
    let execution = schema(&schemas, "execution");
    let execution_value = product([
        ("regime", Value::Enum("closed-loop".to_owned())),
        (
            "horizon",
            Value::Present(Box::new(Value::PositiveInteger(1))),
        ),
        (
            "rate",
            Value::Present(Box::new(Value::Number(
                grimoire::FiniteNumber::new(4.0).unwrap(),
            ))),
        ),
        ("external_consumer", Value::Enum("yes".to_owned())),
    ]);
    assert!(
        execution
            .validate(ElementKind::Block, &execution_value)
            .is_ok()
    );

    let precision = schema(&schemas, "precision");
    let precision_value = product([
        ("weights", Value::Present(Box::new(text("ternary")))),
        ("activations", Value::Present(Box::new(text("int4")))),
        ("accumulation", Value::Absent),
        ("optimizer_state", Value::Present(Box::new(text("bf16")))),
        ("sparsity", Value::Absent),
    ]);
    assert!(
        precision
            .validate(ElementKind::Block, &precision_value)
            .is_ok()
    );

    let provenance = schema(&schemas, "provenance");
    let provenance_value = product([
        ("citations", sequence(vec![text("arXiv:2506.09985")])),
        (
            "assumptions",
            sequence(vec![text("image goal is available")]),
        ),
        ("novelty", Value::Enum("adapted".to_owned())),
    ]);
    assert!(
        provenance
            .validate(ElementKind::Group, &provenance_value)
            .is_ok()
    );
}

#[test]
fn measurement_and_lineage_require_their_closed_shapes() {
    let schemas = schemas();
    let measurement = schema(&schemas, "measurement");
    let measurement_value = product([
        (
            "value",
            tagged(
                "number",
                Value::Number(grimoire::FiniteNumber::new(16.0).unwrap()),
            ),
        ),
        ("unit", text("seconds")),
        (
            "source",
            product([
                ("origin", text("https://arxiv.org/abs/2506.09985")),
                ("locator", Value::Present(Box::new(text("Table 3")))),
                ("protocol", Value::Absent),
            ]),
        ),
    ]);
    assert!(
        measurement
            .validate(ElementKind::Block, &measurement_value)
            .is_ok()
    );

    let lineage = schema(&schemas, "lineage");
    let lineage_value = product([
        ("base", reference("@model/base")),
        ("deltas", sequence(vec![reference("@model/task-a")])),
        ("operation", Value::Enum("trim-sign-merge".to_owned())),
        ("result", reference("@model/merged")),
    ]);
    assert!(lineage.validate(ElementKind::Block, &lineage_value).is_ok());
}

#[test]
fn invalid_attachment_and_value_are_visible() {
    let schemas = schemas();
    let provenance = schema(&schemas, "provenance");
    let empty = product([
        ("citations", sequence(Vec::new())),
        ("assumptions", sequence(Vec::new())),
        ("novelty", Value::Enum("unclassified".to_owned())),
    ]);
    assert!(provenance.validate(ElementKind::Block, &empty).is_err());
    assert!(
        provenance
            .validate(ElementKind::Group, &Value::Bool(true))
            .is_err()
    );
}

#[test]
fn extension_payload_can_hold_a_registered_schema_value() {
    let schemas = schemas();
    let axes = schema(&schemas, "axes");
    let value = product([
        ("name", text("width")),
        (
            "description",
            Value::Present(Box::new(text("embedding width"))),
        ),
    ]);
    assert!(axes.validate(ElementKind::Port, &value).is_ok());
    let extension = ExtensionParameter {
        namespace: axes.namespace.clone(),
        name: "axis".to_owned(),
        schema: axes.name.clone(),
        version: axes.version,
        value: ExtensionValue::Known(value),
    };
    assert_eq!(extension.namespace, axes.namespace);
}
