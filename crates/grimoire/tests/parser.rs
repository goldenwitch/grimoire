use grimoire::{Address, ParseError, parse_description};

const MINIMAL: &str = r#"
# The first complete core description.
grimoire 1.0.0
description @system "V-JEPA 2" {
    core-spec 1.0.0;
    core {
        block @vision-encoder "Vision encoder" {
            port @vision-encoder/input "input";
            port @vision-encoder/output "output";
        }
        block @probe "Probe" {
            port @probe/input;
            port @probe/output "output";
        }
        connection @encoder-to-probe @vision-encoder/output -> @probe/input;
        group @model "model" {
            @vision-encoder, @probe, @encoder-to-probe;
        }
    }
}
"#;

#[test]
fn parses_minimal_description_with_core_elements() {
    let description = parse_description(MINIMAL).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(description.address, address("@system"));
    assert_eq!(description.label.as_deref(), Some("V-JEPA 2"));
    assert_eq!(description.core.blocks.len(), 2);
    assert_eq!(description.core.connections.len(), 1);
    assert_eq!(description.core.groups.len(), 1);
    assert_eq!(description.core.blocks[&address("@probe")].ports.len(), 2);
    let connection = &description.core.connections[&address("@encoder-to-probe")];
    assert_eq!(connection.source, address("@vision-encoder/output"));
    assert_eq!(connection.destination, address("@probe/input"));
}

#[test]
fn parses_escaped_labels_and_unicode_text() {
    let source = r#"
        grimoire 1.0.0
        description @d "quoted \"label\" / \u03bb" {
            core-spec 1.0.0;
            core { block @b "name" { port @b/p; } }
        }
    "#;
    let description = parse_description(source).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        description.label.as_deref(),
        Some("quoted \"label\" / \u{03bb}")
    );
}

#[test]
fn reports_unterminated_string_at_its_start() {
    let source = "grimoire 1.0.0 description @d \"broken { core-spec 1.0.0; core {} }";
    let error = parse_description(source).expect_err("unterminated string should fail");
    assert!(error.offset > 0);
    assert!(error.message.contains("unterminated string"));
}

#[test]
fn reports_duplicate_addresses() {
    let source = r#"
        grimoire 1.0.0
        description @d {
            core-spec 1.0.0;
            core {
                block @b "one" { port @b/p; }
                block @b "two" { port @b/q; }
            }
        }
    "#;
    let error = parse_description(source).expect_err("duplicate address should fail");
    assert!(error.message.contains("duplicate block address"));
}

#[test]
fn reports_trailing_input() {
    let error =
        parse_description("grimoire 1.0.0 description @d { core-spec 1.0.0; core {} } trailing")
            .expect_err("trailing input should fail");
    assert!(error.offset > 0);
    assert!(error.message.contains("trailing input"));
}

fn address(value: &str) -> Address {
    Address::parse(value).unwrap_or_else(|error| panic!("{error}"))
}

fn _parse_error_is_public(_: ParseError) {}
