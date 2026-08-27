use grimoire::{Address, ExtensionValue, Value, parse_description};

const PROTOTYPE_NAMESPACE: &str = "https://github.com/goldenwitch/grimoire/extension/shapes";

#[test]
fn parses_known_extension_values_into_the_closed_algebra() {
    let source = r#"
        grimoire 1.0.0
        description @d {
            core-spec 1.0.0;
            extensions {
                extension "https://github.com/goldenwitch/grimoire/extension/shapes" shape schema shapes @1.0.0 = {
                    layout: sequence,
                    dimensions: [1, 0, ref(@d)]
                };
            }
            core { block @b "block" { port @b/p; } }
        }
    "#;
    let description = parse_description(source).unwrap_or_else(|error| panic!("{error}"));
    let extension = &description.extensions[0];
    assert_eq!(extension.namespace.as_str(), PROTOTYPE_NAMESPACE);
    assert_eq!(extension.name, "shape");
    let ExtensionValue::Known(Value::Product(fields)) = &extension.value else {
        panic!("known namespace should parse a typed product");
    };
    assert_eq!(
        fields.get("layout"),
        Some(&Value::Enum("sequence".to_owned()))
    );
    let Some(Value::Sequence(values)) = fields.get("dimensions") else {
        panic!("shape dimensions should be a sequence");
    };
    assert_eq!(values.len(), 3);
    assert_eq!(values[0], Value::PositiveInteger(1));
    let Value::Number(number) = &values[1] else {
        panic!("zero should parse as a finite number");
    };
    assert_eq!(number.get(), 0.0);
    assert_eq!(
        values[2],
        Value::AddressReference(Address::parse("@d").unwrap())
    );
}

#[test]
fn preserves_unknown_extension_source_span() {
    let source = r#"grimoire 1.0.0 description @d {
    core-spec 1.0.0;
    extensions {
        extension "https://other.example/opaque" payload schema unknown @1.0.0 = { value: "keep" # retained
        };
    }
    core { block @b "block" { port @b/p; } }
}"#;
    let description = parse_description(source).unwrap_or_else(|error| panic!("{error}"));
    let expected = "extension \"https://other.example/opaque\" payload schema unknown @1.0.0 = { value: \"keep\" # retained\n        };";
    let ExtensionValue::Opaque(raw) = &description.extensions[0].value else {
        panic!("unknown namespace should remain opaque");
    };
    assert_eq!(String::from_utf8_lossy(raw), expected);
}

#[test]
fn rejects_schema_version_without_at_sign() {
    let source = r#"
        grimoire 1.0.0
        description @d {
            core-spec 1.0.0;
            extensions {
                extension "https://other.example/opaque" payload schema unknown 1.0.0 = true;
            }
            core {}
        }
    "#;
    let error = parse_description(source).expect_err("schema version must carry an at sign");
    assert!(error.offset > 0);
    assert!(error.message.contains("schema version requires `@`"));
}
