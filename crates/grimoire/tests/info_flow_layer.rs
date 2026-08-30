use grimoire::{Element, evaluate_layer, parse_description, validate_description};

mod common;
use common::{address, schemas};

const INFO_FLOW: &str = r#"
    grimoire 1.0.0
    description @d "information flow" {
        core-spec 1.0.0;
        core {
            block @source "Source" { port @source/out; }
            block @encoder "Encoder" {
                port @encoder/input;
                port @encoder/output;
            }
            block @head "Head" { port @head/input; }
            block @stopped "Stopped consumer" { port @stopped/input; }
            connection @source-to-encoder @source/out -> @encoder/input;
            connection @encoder-to-head @encoder/output -> @head/input;
            connection @encoder-to-stopped @encoder/output -> @stopped/input;
            group @differentiable "differentiable path" {
                @source-to-encoder, @encoder-to-head;
            }
        }
        layer "forward" {
            inputs { core };
            consumes { projection-language 1.0.0; schemas { } }
            projection {
                select {
                    use @source, @encoder, @head, @stopped,
                       @source/out, @encoder/input, @encoder/output, @head/input,
                       @stopped/input, @source-to-encoder, @encoder-to-head,
                       @encoder-to-stopped, @differentiable;
                }
            }
        }
        layer "backward" {
            inputs { core };
            consumes { projection-language 1.0.0; schemas { } }
            projection {
                select {
                    use @source, @encoder, @head,
                       @source/out, @encoder/input, @encoder/output, @head/input,
                       @source-to-encoder, @encoder-to-head, @differentiable;
                }
                invert { group @differentiable; }
            }
        }
    }
"#;

#[test]
fn forward_and_backward_views_validate() {
    let description = parse_description(INFO_FLOW).unwrap_or_else(|error| panic!("{error}"));
    validate_description(&description, &schemas())
        .unwrap_or_else(|errors| panic!("validation errors: {errors:?}"));
    assert_eq!(description.layers.len(), 2);
}

#[test]
fn backward_view_inverts_selected_flow_and_excludes_stopped_flow() {
    let description = parse_description(INFO_FLOW).unwrap_or_else(|error| panic!("{error}"));
    let result = evaluate_layer(&description, "backward").unwrap_or_else(|error| panic!("{error}"));
    let Element::Connection(connection) =
        &result.structural.elements[&address("@source-to-encoder")]
    else {
        panic!("expected source-to-encoder connection");
    };
    assert_eq!(connection.source, address("@encoder/input"));
    assert_eq!(connection.destination, address("@source/out"));
    assert!(
        !result
            .structural
            .elements
            .contains_key(&address("@stopped"))
    );
    assert!(
        !result
            .structural
            .elements
            .contains_key(&address("@encoder-to-stopped"))
    );
}

#[test]
fn forward_view_keeps_the_stopped_connection() {
    let description = parse_description(INFO_FLOW).unwrap_or_else(|error| panic!("{error}"));
    let result = evaluate_layer(&description, "forward").unwrap_or_else(|error| panic!("{error}"));
    assert!(
        result
            .structural
            .elements
            .contains_key(&address("@encoder-to-stopped"))
    );
}
