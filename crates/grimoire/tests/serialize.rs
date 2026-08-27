use grimoire::{ExtensionValue, Value, parse_description, serialize_description};

const CORE: &str = r#"
    grimoire 1.0.0
    description @d "core" {
        core-spec 1.0.0;
        core {
            block @b "Block" {
                port @b/input "input";
                port @b/output "output";
            }
            block @c "Consumer" { port @c/input; }
            connection @flow @b/output -> @c/input;
            group @all "all" { @b, @c, @flow; }
        }
    }
"#;

const KNOWN_EXTENSION: &str = r#"
    grimoire 1.0.0
    description @d {
        core-spec 1.0.0;
        extensions {
            extension "https://github.com/goldenwitch/grimoire/extension/axes" axis schema axes @1.0.0 = {
                name: "frames",
                description: absent
            };
        }
        core { block @b "Block" { port @b/input; } }
    }
"#;

const UNKNOWN_EXTENSION: &str = "grimoire 1.0.0 description @d {\n    core-spec 1.0.0;\n    extensions {\n        extension \"https://other.example/opaque\" payload schema unknown @1.0.0 = { value: \"keep\" # retained\n        };\n    }\n    core { block @b \"Block\" { port @b/input; } }\n}\n";

const LAYERED: &str = r#"
    grimoire 1.0.0
    description @d {
        core-spec 1.0.0;
        core { block @b "Block" { port @b/input; } }
        layer "view" {
            inputs { core };
            consumes {
                projection-language 1.0.0;
                schemas {
                    "https://github.com/goldenwitch/grimoire/extension" / shapes @1.0.0;
                }
            }
            projection {
                select { use @b; }
                decorate {
                    on @b extension "https://github.com/goldenwitch/grimoire/extension/architecture" family schema architecture @1.0.0 = { family: "encoder" };
                }
                checks {
                    check family-present expect nonempty over "https://github.com/goldenwitch/grimoire/extension/architecture" family;
                }
            }
        }
    }
"#;

#[test]
fn canonical_core_serialization_round_trips() {
    let parsed = parse_description(CORE).unwrap_or_else(|error| panic!("{error}"));
    let serialized = serialize_description(&parsed).unwrap_or_else(|error| panic!("{error}"));
    let reparsed = parse_description(&serialized).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(reparsed, parsed);
    assert_eq!(serialize_description(&reparsed).unwrap(), serialized);
}

#[test]
fn known_extension_values_are_canonicalized_and_round_trip() {
    let parsed = parse_description(KNOWN_EXTENSION).unwrap_or_else(|error| panic!("{error}"));
    let serialized = serialize_description(&parsed).unwrap_or_else(|error| panic!("{error}"));
    let reparsed = parse_description(&serialized).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(reparsed, parsed);
    let ExtensionValue::Known(Value::Product(fields)) = &reparsed.extensions[0].value else {
        panic!("expected a typed extension value");
    };
    assert_eq!(fields.get("name"), Some(&Value::Text("frames".to_owned())));
}

#[test]
fn unknown_extension_bytes_survive_serialization() {
    let parsed = parse_description(UNKNOWN_EXTENSION).unwrap_or_else(|error| panic!("{error}"));
    let ExtensionValue::Opaque(raw) = &parsed.extensions[0].value else {
        panic!("expected an opaque extension value");
    };
    let expected = String::from_utf8_lossy(raw).to_string();
    let serialized = serialize_description(&parsed).unwrap_or_else(|error| panic!("{error}"));
    assert!(serialized.contains(&expected));
    let reparsed = parse_description(&serialized).unwrap_or_else(|error| panic!("{error}"));
    let ExtensionValue::Opaque(reparsed_raw) = &reparsed.extensions[0].value else {
        panic!("expected an opaque extension value after round-trip");
    };
    assert_eq!(reparsed_raw, raw);
}

#[test]
fn layered_projection_serialization_round_trips() {
    let parsed = parse_description(LAYERED).unwrap_or_else(|error| panic!("{error}"));
    let serialized = serialize_description(&parsed).unwrap_or_else(|error| panic!("{error}"));
    let reparsed = parse_description(&serialized).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(reparsed, parsed);
    assert_eq!(serialize_description(&reparsed).unwrap(), serialized);
}

#[test]
fn invalid_opaque_bytes_are_rejected_by_text_serializer() {
    let mut parsed = parse_description(CORE).unwrap_or_else(|error| panic!("{error}"));
    parsed.extensions.push(grimoire::ExtensionParameter {
        namespace: grimoire::Namespace::parse("https://other.example/opaque").unwrap(),
        name: "payload".to_owned(),
        schema: "unknown".to_owned(),
        version: grimoire::Version::new(1, 0, 0),
        value: ExtensionValue::Opaque(vec![0xff]),
    });
    assert!(serialize_description(&parsed).is_err());
}
