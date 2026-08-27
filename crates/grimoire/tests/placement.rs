use std::collections::BTreeMap;

use grimoire::{
    Address, Collective, CollectiveTransfer, Element, Placement, PlacementError, Schema,
    ShapeDimension, TensorShape, bytes_on_wire, evaluate_layer, parse_description,
    prototype_schemas, validate_description,
};

const PLACEMENT_LAYER: &str = r#"
    grimoire 1.0.0
    description @d "placement" {
        core-spec 1.0.0;
        core {
            block @encoder "Encoder" {
                port @encoder/output extensions {
                    extension "https://github.com/goldenwitch/grimoire/extension/shapes" shape schema shapes @1.0.0 = {
                        layout: vector,
                        dimensions: [literal(4)]
                    };
                };
            }
            block @head "Head" { port @head/input; }
            connection @flow @encoder/output -> @head/input;
            group @graph "graph" { @encoder, @head, @flow; }
        }
        layer "placement" {
            inputs { core };
            consumes {
                projection-language 1.0.0;
                schemas {
                    "https://github.com/goldenwitch/grimoire/extension/placement" / placement @1.0.0;
                    "https://github.com/goldenwitch/grimoire/extension/shapes" / shapes @1.0.0;
                }
            }
            projection {
                select {
                    use @encoder, @encoder/output, @head, @head/input, @flow, @graph;
                    block @placement/all-reduce "All reduce" { }
                }
                decorate {
                    on @encoder extension "https://github.com/goldenwitch/grimoire/extension/placement" placement schema placement @1.0.0 = { location: "gpu-0" };
                    on @head extension "https://github.com/goldenwitch/grimoire/extension/placement" placement schema placement @1.0.0 = { location: "gpu-1" };
                    on @placement/all-reduce extension "https://github.com/goldenwitch/grimoire/extension/placement" placement schema placement @1.0.0 = { location: "gpu-0" };
                }
            }
        }
    }
"#;

fn address(value: &str) -> Address {
    Address::parse(value).unwrap_or_else(|error| panic!("{error}"))
}

fn schemas() -> Vec<Schema> {
    prototype_schemas().unwrap_or_else(|error| panic!("{error}"))
}

fn parsed() -> grimoire::Description {
    parse_description(PLACEMENT_LAYER).unwrap_or_else(|error| panic!("{error}"))
}

fn shape() -> TensorShape {
    TensorShape::new(vec![ShapeDimension::Literal(4)], 2).unwrap_or_else(|error| panic!("{error}"))
}

#[test]
fn placement_decorations_validate_and_extract_locations() {
    let description = parsed();
    validate_description(&description, &schemas())
        .unwrap_or_else(|errors| panic!("validation errors: {errors:?}"));
    let result =
        evaluate_layer(&description, "placement").unwrap_or_else(|error| panic!("{error}"));
    let placement =
        Placement::from_decorations(&result.decorations).unwrap_or_else(|error| panic!("{error}"));

    assert_eq!(
        placement.assigned_location(&address("@encoder")),
        Some("gpu-0")
    );
    assert_eq!(
        placement.assigned_location(&address("@head")),
        Some("gpu-1")
    );
    assert!(matches!(
        result
            .structural
            .elements
            .get(&address("@placement/all-reduce")),
        Some(Element::Block(_))
    ));
}

#[test]
fn bytes_on_wire_uses_block_placement_and_explicit_shapes() {
    let description = parsed();
    let result =
        evaluate_layer(&description, "placement").unwrap_or_else(|error| panic!("{error}"));
    let placement =
        Placement::from_decorations(&result.decorations).unwrap_or_else(|error| panic!("{error}"));
    let shapes = BTreeMap::from([(address("@encoder/output"), shape())]);
    let collective = Collective::new(
        address("@placement/all-reduce"),
        address("@encoder/output"),
        vec![CollectiveTransfer::new(
            address("@encoder"),
            address("@head"),
        )],
    )
    .unwrap_or_else(|error| panic!("{error}"));

    let report = bytes_on_wire(
        &result.structural,
        &placement,
        &shapes,
        &BTreeMap::new(),
        &[collective],
    )
    .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(report.total_bytes(), 16);
    assert_eq!(report.transfers.len(), 2);
    assert_eq!(report.transfers[0].bytes, 8);
    assert_eq!(report.transfers[1].bytes, 8);
}

#[test]
fn same_location_links_need_no_shape_and_missing_location_fails() {
    let description = parsed();
    let result =
        evaluate_layer(&description, "placement").unwrap_or_else(|error| panic!("{error}"));
    let same_location = Placement::new(vec![
        (address("@encoder"), "gpu-0".to_owned()),
        (address("@head"), "gpu-0".to_owned()),
    ])
    .unwrap_or_else(|error| panic!("{error}"));
    let report = bytes_on_wire(
        &result.structural,
        &same_location,
        &BTreeMap::new(),
        &BTreeMap::new(),
        &[],
    )
    .unwrap_or_else(|error| panic!("{error}"));
    assert!(report.transfers.is_empty());
    assert_eq!(report.total_bytes(), 0);

    let missing = Placement::new(vec![(address("@encoder"), "gpu-0".to_owned())])
        .unwrap_or_else(|error| panic!("{error}"));
    let error = bytes_on_wire(
        &result.structural,
        &missing,
        &BTreeMap::new(),
        &BTreeMap::new(),
        &[],
    )
    .expect_err("cross-device traffic needs both endpoint placements");
    assert_eq!(
        error,
        PlacementError::MissingPlacement(address("@head/input"))
    );
}

#[test]
fn symbolic_shape_extents_are_explicit_and_checked() {
    let axis = address("@axis/frames");
    let shape = TensorShape::new(vec![ShapeDimension::Axis(axis.clone())], 4)
        .unwrap_or_else(|error| panic!("{error}"));
    let bytes = shape
        .byte_size(&BTreeMap::from([(axis.clone(), 16)]))
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(bytes, 64);
    assert_eq!(
        shape.byte_size(&BTreeMap::new()),
        Err(PlacementError::MissingAxis(axis))
    );
}
