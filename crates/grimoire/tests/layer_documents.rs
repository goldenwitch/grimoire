use grimoire::{
    ExpectedCardinality, LayerInput, SelectItem, parse_layer_document, serialize_layer_document,
};

const STANDALONE_LAYER: &str = r#"
    grimoire-layer 1.0.0
    for @system;
    layer "pretraining" {
        inputs { core };
        consumes {
            projection-language 1.0.0;
            schemas {
                "https://github.com/goldenwitch/grimoire/extension/shapes" / shapes @1.0.0;
            }
        }
        projection {
            select {
                use @encoder;
                block @predictor "Predictor" { port @predictor/input; }
            }
            invert { }
            decorate {
                on @encoder extension "https://github.com/goldenwitch/grimoire/extension/architecture" family schema architecture @1.0.0 = { family: "vision" };
            }
            checks {
                check architecture-present expect nonempty over "https://github.com/goldenwitch/grimoire/extension/architecture" family;
            }
        }
    }
"#;

#[test]
fn parses_a_standalone_layer_document() {
    let document = parse_layer_document(STANDALONE_LAYER).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(document.description.as_str(), "@system");
    assert_eq!(document.layer.name, "pretraining");
    assert_eq!(document.layer.inputs, vec![LayerInput::Core]);
    assert_eq!(document.layer.schemas[0].name, "shapes");
    assert_eq!(document.layer.projection.select.len(), 2);
    assert_eq!(document.layer.projection.decorate.len(), 1);
    assert_eq!(document.layer.projection.checks.len(), 1);
    assert_eq!(
        document.layer.projection.checks[0].expected,
        ExpectedCardinality::Nonempty
    );
    assert!(matches!(
        document.layer.projection.select[0],
        SelectItem::Use(_)
    ));
}

#[test]
fn rejects_trailing_tokens_after_a_standalone_layer() {
    let source = format!("{STANDALONE_LAYER} trailing");
    let error = parse_layer_document(&source).expect_err("trailing tokens should fail");
    assert!(error.message.contains("trailing input"));
}

#[test]
fn standalone_layer_serialization_round_trips() {
    let document = parse_layer_document(STANDALONE_LAYER).unwrap_or_else(|error| panic!("{error}"));
    let serialized = serialize_layer_document(&document).unwrap_or_else(|error| panic!("{error}"));
    let reparsed = parse_layer_document(&serialized).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(reparsed, document);
}
