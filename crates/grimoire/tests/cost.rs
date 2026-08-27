use std::collections::BTreeMap;

use grimoire::{
    Address, CostError, CostExpression, CostModel, Schema, evaluate_layer, parse_description,
    prototype_schemas, validate_description,
};

const COST_DESCRIPTION: &str = r#"
    grimoire 1.0.0
    description @d "cost" {
        core-spec 1.0.0;
        core {
            block @encoder "Encoder" { port @encoder/output; }
            block @bridge "Bridge" { port @bridge/input; }
            connection @flow @encoder/output -> @bridge/input;
            group @pipeline "pipeline" { @encoder, @bridge, @flow; }
        }
        layer "cost" {
            inputs { core };
            consumes { projection-language 1.0.0; schemas { } }
            projection { select { use @encoder, @bridge, @flow, @pipeline; } }
        }
    }
"#;

fn address(value: &str) -> Address {
    Address::parse(value).unwrap_or_else(|error| panic!("{error}"))
}

fn schemas() -> Vec<Schema> {
    prototype_schemas().unwrap_or_else(|error| panic!("{error}"))
}

#[test]
fn symbolic_cost_evaluates_from_explicit_axis_bindings() {
    let expression = CostExpression::product(vec![
        CostExpression::constant(2),
        CostExpression::axis(address("@axis/frames")),
        CostExpression::axis(address("@axis/features")),
    ]);
    let value = expression
        .evaluate(&BTreeMap::from([
            (address("@axis/frames"), 16),
            (address("@axis/features"), 1408),
        ]))
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(value, 45056);
}

#[test]
fn group_cost_is_the_sum_of_authored_member_expressions() {
    let description = parse_description(COST_DESCRIPTION).unwrap_or_else(|error| panic!("{error}"));
    validate_description(&description, &schemas())
        .unwrap_or_else(|errors| panic!("validation errors: {errors:?}"));
    let reprojection =
        evaluate_layer(&description, "cost").unwrap_or_else(|error| panic!("{error}"));
    let model = CostModel::new(vec![
        (address("@encoder"), CostExpression::constant(10)),
        (
            address("@bridge"),
            CostExpression::product(vec![
                CostExpression::constant(2),
                CostExpression::axis(address("@axis/tokens")),
            ]),
        ),
        (address("@flow"), CostExpression::constant(3)),
    ])
    .unwrap_or_else(|error| panic!("{error}"));
    let report = model
        .evaluate(
            &reprojection.structural,
            &BTreeMap::from([(address("@axis/tokens"), 4)]),
        )
        .unwrap_or_else(|error| panic!("{error}"));

    assert_eq!(report.value(&address("@bridge")), Some(8));
    assert_eq!(
        report
            .group_total(&reprojection.structural, &address("@pipeline"))
            .unwrap_or_else(|error| panic!("{error}")),
        21
    );
}

#[test]
fn unresolved_cost_inputs_fail_without_inference() {
    let missing_axis = CostExpression::axis(address("@axis/missing"));
    assert_eq!(
        missing_axis.evaluate(&BTreeMap::new()),
        Err(CostError::MissingAxis(address("@axis/missing")))
    );

    let description = parse_description(COST_DESCRIPTION).unwrap_or_else(|error| panic!("{error}"));
    let reprojection =
        evaluate_layer(&description, "cost").unwrap_or_else(|error| panic!("{error}"));
    let model = CostModel::new(vec![(address("@encoder"), CostExpression::constant(10))])
        .unwrap_or_else(|error| panic!("{error}"));
    let report = model
        .evaluate(&reprojection.structural, &BTreeMap::new())
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        report.group_total(&reprojection.structural, &address("@pipeline")),
        Err(CostError::MissingCost(address("@bridge")))
    );
}

#[test]
fn cost_overflow_and_duplicate_group_members_are_visible() {
    let overflow = CostExpression::product(vec![
        CostExpression::constant(u64::MAX),
        CostExpression::constant(2),
    ]);
    assert_eq!(
        overflow.evaluate(&BTreeMap::new()),
        Err(CostError::ExpressionOverflow)
    );

    let duplicate = CostModel::new(vec![
        (address("@encoder"), CostExpression::constant(1)),
        (address("@encoder"), CostExpression::constant(2)),
    ]);
    assert_eq!(
        duplicate,
        Err(CostError::DuplicateCost(address("@encoder")))
    );
}
