use grimoire::{
    Element, Schema, evaluate_layer, parse_description, prototype_schemas, validate_description,
};

const MODES: &str = r#"
    grimoire 1.0.0
    description @d "modes" {
        core-spec 1.0.0;
        core {
            block @shared/backbone "Shared transformer backbone" {
                port @shared/backbone/input;
                port @shared/backbone/output;
            }
            block @mode/understanding "Understanding path" {
                port @mode/understanding/input;
                port @mode/understanding/output;
            }
            block @mode/generation "Generation path" {
                port @mode/generation/input;
                port @mode/generation/output;
            }
            group @mode/alternatives "mode alternatives" {
                @mode/understanding, @mode/generation;
            }
        }
        layer "understanding" {
            inputs { core };
            consumes { projection-language 1.0.0; schemas { } }
            projection {
                select {
                    use @shared/backbone, @mode/understanding;
                    block @understanding/probe "Understanding probe" {
                        port @understanding/probe/input;
                    }
                }
            }
        }
        layer "generation" {
            inputs { core };
            consumes { projection-language 1.0.0; schemas { } }
            projection {
                select {
                    use @shared/backbone, @mode/generation;
                    block @generation/head "Generation head" {
                        port @generation/head/input;
                    }
                }
            }
        }
    }
"#;

fn schemas() -> Vec<Schema> {
    prototype_schemas().unwrap_or_else(|error| panic!("{error}"))
}

#[test]
fn mode_layers_validate_and_select_distinct_alternatives() {
    let description = parse_description(MODES).unwrap_or_else(|error| panic!("{error}"));
    validate_description(&description, &schemas())
        .unwrap_or_else(|errors| panic!("validation errors: {errors:?}"));

    let understanding =
        evaluate_layer(&description, "understanding").unwrap_or_else(|error| panic!("{error}"));
    let generation =
        evaluate_layer(&description, "generation").unwrap_or_else(|error| panic!("{error}"));
    assert!(
        understanding
            .structural
            .elements
            .contains_key(&grimoire::Address::parse("@shared/backbone").unwrap())
    );
    assert!(
        generation
            .structural
            .elements
            .contains_key(&grimoire::Address::parse("@shared/backbone").unwrap())
    );
    assert!(
        understanding
            .structural
            .elements
            .contains_key(&grimoire::Address::parse("@mode/understanding").unwrap())
    );
    assert!(
        !understanding
            .structural
            .elements
            .contains_key(&grimoire::Address::parse("@mode/generation").unwrap())
    );
    assert!(
        generation
            .structural
            .elements
            .contains_key(&grimoire::Address::parse("@mode/generation").unwrap())
    );
    assert!(
        !generation
            .structural
            .elements
            .contains_key(&grimoire::Address::parse("@mode/understanding").unwrap())
    );
}

#[test]
fn mode_local_elements_are_owned_by_their_mode() {
    let description = parse_description(MODES).unwrap_or_else(|error| panic!("{error}"));
    let understanding =
        evaluate_layer(&description, "understanding").unwrap_or_else(|error| panic!("{error}"));
    let generation =
        evaluate_layer(&description, "generation").unwrap_or_else(|error| panic!("{error}"));
    assert!(matches!(
        understanding.structural.elements
            [&grimoire::Address::parse("@understanding/probe").unwrap()],
        Element::Block(_)
    ));
    assert!(
        !generation
            .structural
            .elements
            .contains_key(&grimoire::Address::parse("@understanding/probe").unwrap())
    );
    assert!(matches!(
        generation.structural.elements[&grimoire::Address::parse("@generation/head").unwrap()],
        Element::Block(_)
    ));
    assert!(
        !understanding
            .structural
            .elements
            .contains_key(&grimoire::Address::parse("@generation/head").unwrap())
    );
}

#[test]
fn an_unselected_alternative_remains_available_to_another_mode() {
    let description = parse_description(MODES).unwrap_or_else(|error| panic!("{error}"));
    let group = &description.core.groups[&grimoire::Address::parse("@mode/alternatives").unwrap()];
    assert_eq!(group.members.len(), 2);
    assert!(
        description
            .core
            .blocks
            .contains_key(&grimoire::Address::parse("@mode/understanding").unwrap())
    );
    assert!(
        description
            .core
            .blocks
            .contains_key(&grimoire::Address::parse("@mode/generation").unwrap())
    );
}
