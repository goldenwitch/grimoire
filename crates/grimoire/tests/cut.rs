use grimoire::{
    CutError, Schema, extract_cut, parse_description, prototype_schemas, serialize_description,
    validate_description,
};

const DESCRIPTION: &str = r#"
    grimoire 1.0.0
    description @d "cuts" {
        core-spec 1.0.0;
        core {
            block @encoder "Encoder" { port @encoder/output; }
        }
        layer "base" {
            inputs { core };
            consumes { projection-language 1.0.0; schemas { } }
            projection { select { use @encoder; } }
        }
        layer "consumer" {
            inputs { core, "base" };
            consumes { projection-language 1.0.0; schemas { } }
            projection { select { use @encoder; } }
        }
    }
"#;

fn schemas() -> Vec<Schema> {
    prototype_schemas().unwrap_or_else(|error| panic!("{error}"))
}

#[test]
fn downward_closed_cut_extracts_and_revalidates() {
    let description = parse_description(DESCRIPTION).unwrap_or_else(|error| panic!("{error}"));
    let cut = extract_cut(&description, &["base", "consumer"], &schemas())
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(cut.layers.len(), 2);
    assert!(validate_description(&cut, &schemas()).is_ok());
    let serialized = serialize_description(&cut).unwrap_or_else(|error| panic!("{error}"));
    let reparsed = parse_description(&serialized).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(reparsed, cut);
}

#[test]
fn a_core_only_cut_is_self_contained() {
    let description = parse_description(DESCRIPTION).unwrap_or_else(|error| panic!("{error}"));
    let cut = extract_cut(&description, &[], &schemas()).unwrap_or_else(|error| panic!("{error}"));
    assert!(cut.layers.is_empty());
    assert!(validate_description(&cut, &schemas()).is_ok());
}

#[test]
fn non_cut_reports_the_layer_and_absent_input() {
    let description = parse_description(DESCRIPTION).unwrap_or_else(|error| panic!("{error}"));
    let error = extract_cut(&description, &["consumer"], &schemas())
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

#[test]
fn unknown_selected_layer_is_visible() {
    let description = parse_description(DESCRIPTION).unwrap_or_else(|error| panic!("{error}"));
    let error = extract_cut(&description, &["missing"], &schemas())
        .expect_err("unknown selected layer should fail");
    assert_eq!(error, CutError::UnknownLayer("missing".to_owned()));
}
