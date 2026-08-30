use std::collections::BTreeMap;

use grimoire::{
    CostExpression, CostModel, Placement, ShapeDimension, TensorShape, bytes_on_wire,
    evaluate_layer, extract_cut, parse_description, serialize_description, validate_description,
};

mod common;
use common::{address, schemas};

const EXAMPLE: &str = include_str!("../../../examples/reference.grimoire");

#[test]
fn public_example_covers_both_primary_workflows() {
    let description = parse_description(EXAMPLE).unwrap_or_else(|error| panic!("{error}"));
    let schemas = schemas();
    validate_description(&description, &schemas)
        .unwrap_or_else(|errors| panic!("validation errors: {errors:?}"));

    let architecture =
        evaluate_layer(&description, "architecture").unwrap_or_else(|error| panic!("{error}"));
    let deployment =
        evaluate_layer(&description, "deployment").unwrap_or_else(|error| panic!("{error}"));
    let cost = evaluate_layer(&description, "cost").unwrap_or_else(|error| panic!("{error}"));

    let placement = Placement::from_decorations(&deployment.decorations)
        .unwrap_or_else(|error| panic!("{error}"));
    let report = bytes_on_wire(
        &deployment.structural,
        &placement,
        &BTreeMap::from([(
            address("@example/encoder/output"),
            TensorShape::new(vec![ShapeDimension::Literal(4)], 2)
                .unwrap_or_else(|error| panic!("{error}")),
        )]),
        &BTreeMap::new(),
        &[],
    )
    .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(report.total_bytes(), 8);

    let model = CostModel::new(vec![
        (address("@example/input"), CostExpression::constant(1)),
        (address("@example/encoder"), CostExpression::constant(10)),
        (address("@example/head"), CostExpression::constant(3)),
        (
            address("@example/input-to-encoder"),
            CostExpression::constant(2),
        ),
        (
            address("@example/encoder-to-head"),
            CostExpression::constant(2),
        ),
    ])
    .unwrap_or_else(|error| panic!("{error}"));
    let cost_report = model
        .evaluate(&cost.structural, &BTreeMap::new())
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        cost_report
            .group_total(&cost.structural, &address("@example/model"))
            .unwrap_or_else(|error| panic!("{error}")),
        18
    );

    assert!(
        architecture
            .structural
            .elements
            .contains_key(&address("@example/model"))
    );
    let cut = extract_cut(&description, &["deployment"], &schemas)
        .unwrap_or_else(|error| panic!("{error}"));
    validate_description(&cut, &schemas)
        .unwrap_or_else(|errors| panic!("cut validation errors: {errors:?}"));
    let serialized = serialize_description(&description).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        parse_description(&serialized).unwrap_or_else(|error| panic!("{error}")),
        description
    );
}
