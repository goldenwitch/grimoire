use grimoire::{
    CutError, extract_cut, parse_description, serialize_description, validate_description,
};

mod common;
use common::schemas;

const VALID_DESCRIPTION: &str = r#"
    grimoire 1.0.0
    description @conformance "validator conformance" {
        core-spec 1.0.0;
        core {
            block @encoder "Encoder" {
                port @encoder/input;
                port @encoder/output;
            }
            block @head "Head" { port @head/input; }
            connection @flow @encoder/output -> @head/input;
            group @graph "graph" { @encoder, @head, @flow; }
        }
        layer "base" {
            inputs { core };
            consumes { projection-language 1.0.0; schemas { } }
            projection { select { use @encoder, @head, @flow; } }
        }
        layer "consumer" {
            inputs { core, "base" };
            consumes { projection-language 1.0.0; schemas { } }
            projection { select { use @encoder, @head; } }
        }
    }
"#;

#[test]
fn valid_reference_fixture_composes_parse_validate_serialize_and_cut() {
    let parsed = parse_description(VALID_DESCRIPTION).unwrap_or_else(|error| panic!("{error}"));
    validate_description(&parsed, &schemas())
        .unwrap_or_else(|errors| panic!("validation errors: {errors:?}"));

    let serialized = serialize_description(&parsed).unwrap_or_else(|error| panic!("{error}"));
    let reparsed = parse_description(&serialized).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(reparsed, parsed);
    assert_eq!(
        serialize_description(&reparsed).unwrap_or_else(|error| panic!("{error}")),
        serialized
    );

    let cut = extract_cut(&parsed, &["base", "consumer"], &schemas())
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(cut.layers.len(), 2);
    validate_description(&cut, &schemas())
        .unwrap_or_else(|errors| panic!("cut validation errors: {errors:?}"));
}

#[test]
fn non_cut_is_reported_without_running_projection_evaluation() {
    let parsed = parse_description(VALID_DESCRIPTION).unwrap_or_else(|error| panic!("{error}"));
    let error = extract_cut(&parsed, &["consumer"], &schemas())
        .expect_err("consumer without base should be unresolvable");
    assert_eq!(
        error,
        CutError::Unresolvable {
            layer: "consumer".to_owned(),
            missing: vec!["base".to_owned()],
        }
    );
    assert!(error.to_string().contains("C12"));
    assert!(error.to_string().contains("consumer"));
    assert!(error.to_string().contains("base"));
}
