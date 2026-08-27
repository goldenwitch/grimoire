use core::fmt;

use crate::{
    Address, Block, Check, Connection, CoreGraph, Description, ExpectedCardinality,
    ExtensionParameter, ExtensionValue, Group, Layer, LayerInput, Port, Projection, SchemaUse,
    SelectItem, Value,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SerializeError {
    pub message: String,
}

pub fn serialize_description(description: &Description) -> Result<String, SerializeError> {
    let mut output = String::new();
    output.push_str("grimoire 1.0.0\n");
    output.push_str("description ");
    write_address(&mut output, &description.address);
    if let Some(label) = &description.label {
        output.push(' ');
        write_string(&mut output, label);
    }
    output.push_str(" {\n");
    write_indent(&mut output, 1);
    output.push_str("core-spec ");
    output.push_str(&description.core_spec.to_string());
    output.push_str(";\n");
    write_extensions(&mut output, &description.extensions, 1)?;
    write_core(&mut output, &description.core, 1)?;
    let mut layers = description.layers.clone();
    layers.sort_by(|left, right| left.name.cmp(&right.name));
    for layer in &layers {
        write_layer(&mut output, layer, 1)?;
    }
    output.push_str("}\n");
    Ok(output)
}

fn write_core(output: &mut String, core: &CoreGraph, indent: usize) -> Result<(), SerializeError> {
    write_indent(output, indent);
    output.push_str("core {\n");
    for block in core.blocks.values() {
        write_block(output, block, indent + 1)?;
    }
    for connection in core.connections.values() {
        write_connection(output, connection, indent + 1)?;
    }
    for group in core.groups.values() {
        write_group(output, group, indent + 1)?;
    }
    write_indent(output, indent);
    output.push_str("}\n");
    Ok(())
}

fn write_block(output: &mut String, block: &Block, indent: usize) -> Result<(), SerializeError> {
    write_indent(output, indent);
    output.push_str("block ");
    write_address(output, &block.address);
    output.push(' ');
    write_string(output, &block.name);
    output.push_str(" {\n");
    for port in block.ports.values() {
        write_port(output, port, indent + 1)?;
    }
    write_extensions(output, &block.extensions, indent + 1)?;
    write_indent(output, indent);
    output.push_str("}\n");
    Ok(())
}

fn write_port(output: &mut String, port: &Port, indent: usize) -> Result<(), SerializeError> {
    write_indent(output, indent);
    output.push_str("port ");
    write_address(output, &port.address);
    if let Some(label) = &port.label {
        output.push(' ');
        write_string(output, label);
    }
    if port.extensions.is_empty() {
        output.push_str(";\n");
    } else {
        output.push('\n');
        write_extensions(output, &port.extensions, indent + 1)?;
        write_indent(output, indent);
        output.push_str(";\n");
    }
    Ok(())
}

fn write_connection(
    output: &mut String,
    connection: &Connection,
    indent: usize,
) -> Result<(), SerializeError> {
    write_indent(output, indent);
    output.push_str("connection ");
    write_address(output, &connection.address);
    output.push(' ');
    write_address(output, &connection.source);
    output.push_str(" -> ");
    write_address(output, &connection.destination);
    if connection.extensions.is_empty() {
        output.push_str(";\n");
    } else {
        output.push('\n');
        write_extensions(output, &connection.extensions, indent + 1)?;
        write_indent(output, indent);
        output.push_str(";\n");
    }
    Ok(())
}

fn write_group(output: &mut String, group: &Group, indent: usize) -> Result<(), SerializeError> {
    write_indent(output, indent);
    output.push_str("group ");
    write_address(output, &group.address);
    if let Some(label) = &group.label {
        output.push(' ');
        write_string(output, label);
    }
    output.push_str(" {\n");
    if !group.members.is_empty() {
        write_indent(output, indent + 1);
        write_address_list(output, &group.members);
        output.push_str(";\n");
    }
    write_extensions(output, &group.extensions, indent + 1)?;
    write_indent(output, indent);
    output.push_str("}\n");
    Ok(())
}

fn write_layer(output: &mut String, layer: &Layer, indent: usize) -> Result<(), SerializeError> {
    write_indent(output, indent);
    output.push_str("layer ");
    write_string(output, &layer.name);
    output.push_str(" {\n");
    write_indent(output, indent + 1);
    output.push_str("inputs { ");
    for (index, input) in layer.inputs.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        match input {
            LayerInput::Core => output.push_str("core"),
            LayerInput::Layer(name) => write_string(output, name),
        }
    }
    output.push_str(" };\n");
    write_indent(output, indent + 1);
    output.push_str("consumes {\n");
    write_indent(output, indent + 2);
    output.push_str("projection-language ");
    output.push_str(&layer.projection_language.to_string());
    output.push_str(";\n");
    write_indent(output, indent + 2);
    output.push_str("schemas {\n");
    let mut schemas = layer.schemas.clone();
    schemas.sort_by(schema_use_order);
    for schema in &schemas {
        write_schema_use(output, schema, indent + 3);
    }
    write_indent(output, indent + 2);
    output.push_str("}\n");
    write_indent(output, indent + 1);
    output.push_str("}\n");
    write_projection(output, &layer.projection, indent + 1)?;
    write_indent(output, indent);
    output.push_str("}\n");
    Ok(())
}

fn write_schema_use(output: &mut String, schema: &SchemaUse, indent: usize) {
    write_indent(output, indent);
    write_string(output, schema.namespace.as_str());
    output.push_str(" / ");
    output.push_str(&schema.name);
    output.push_str(" @");
    output.push_str(&schema.version.to_string());
    output.push_str(";\n");
}

fn schema_use_order(left: &SchemaUse, right: &SchemaUse) -> std::cmp::Ordering {
    left.namespace
        .cmp(&right.namespace)
        .then_with(|| left.name.cmp(&right.name))
        .then_with(|| left.version.cmp(&right.version))
}

fn write_projection(
    output: &mut String,
    projection: &Projection,
    indent: usize,
) -> Result<(), SerializeError> {
    write_indent(output, indent);
    output.push_str("projection {\n");
    write_indent(output, indent + 1);
    output.push_str("select {\n");
    for item in &projection.select {
        write_select_item(output, item, indent + 2)?;
    }
    write_indent(output, indent + 1);
    output.push_str("}\n");
    if !projection.invert.is_empty() {
        write_indent(output, indent + 1);
        output.push_str("invert {\n");
        for group in &projection.invert {
            write_indent(output, indent + 2);
            output.push_str("group ");
            write_address(output, group);
            output.push_str(";\n");
        }
        write_indent(output, indent + 1);
        output.push_str("}\n");
    }
    if !projection.decorate.is_empty() {
        write_indent(output, indent + 1);
        output.push_str("decorate {\n");
        for decoration in &projection.decorate {
            write_indent(output, indent + 2);
            output.push_str("on ");
            write_address(output, &decoration.target);
            output.push(' ');
            write_extension_parameter(output, &decoration.parameter)?;
            output.push('\n');
        }
        write_indent(output, indent + 1);
        output.push_str("}\n");
    }
    if !projection.checks.is_empty() {
        write_indent(output, indent + 1);
        output.push_str("checks {\n");
        for check in &projection.checks {
            write_check(output, check, indent + 2);
        }
        write_indent(output, indent + 1);
        output.push_str("}\n");
    }
    write_indent(output, indent);
    output.push_str("}\n");
    Ok(())
}

fn write_select_item(
    output: &mut String,
    item: &SelectItem,
    indent: usize,
) -> Result<(), SerializeError> {
    match item {
        SelectItem::Use(addresses) => {
            write_indent(output, indent);
            output.push_str("use ");
            write_address_list(output, addresses);
            output.push_str(";\n");
        }
        SelectItem::GenerateBlock(block) => write_block(output, block, indent)?,
        SelectItem::GenerateConnection(connection) => write_connection(output, connection, indent)?,
        SelectItem::GenerateGroup(group) => write_group(output, group, indent)?,
    }
    Ok(())
}

fn write_check(output: &mut String, check: &Check, indent: usize) {
    write_indent(output, indent);
    output.push_str("check ");
    output.push_str(&check.name);
    output.push_str(" expect ");
    output.push_str(match check.expected {
        ExpectedCardinality::Empty => "empty",
        ExpectedCardinality::Nonempty => "nonempty",
    });
    output.push_str(" over ");
    write_string(output, check.namespace.as_str());
    output.push(' ');
    output.push_str(&check.parameter);
    output.push_str(";\n");
}

fn write_extensions(
    output: &mut String,
    extensions: &[ExtensionParameter],
    indent: usize,
) -> Result<(), SerializeError> {
    if extensions.is_empty() {
        return Ok(());
    }
    write_indent(output, indent);
    output.push_str("extensions {\n");
    for extension in extensions {
        write_indent(output, indent + 1);
        write_extension_parameter(output, extension)?;
        output.push('\n');
    }
    write_indent(output, indent);
    output.push_str("}\n");
    Ok(())
}

fn write_extension_parameter(
    output: &mut String,
    extension: &ExtensionParameter,
) -> Result<(), SerializeError> {
    match &extension.value {
        ExtensionValue::Known(value) => {
            output.push_str("extension ");
            write_string(output, extension.namespace.as_str());
            output.push(' ');
            output.push_str(&extension.name);
            output.push_str(" schema ");
            output.push_str(&extension.schema);
            output.push_str(" @");
            output.push_str(&extension.version.to_string());
            output.push_str(" = ");
            write_value(output, value)?;
            output.push(';');
        }
        ExtensionValue::Opaque(bytes) => {
            let value = std::str::from_utf8(bytes).map_err(|_| SerializeError {
                message: "opaque extension payload is not valid UTF-8".to_owned(),
            })?;
            output.push_str(value);
        }
    }
    Ok(())
}

fn write_value(output: &mut String, value: &Value) -> Result<(), SerializeError> {
    match value {
        Value::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
        Value::PositiveInteger(value) => output.push_str(&value.to_string()),
        Value::Number(value) => output.push_str(&value.get().to_string()),
        Value::Text(value) => write_string(output, value),
        Value::Enum(value) => output.push_str(value),
        Value::Product(fields) => {
            output.push('{');
            for (index, (name, value)) in fields.iter().enumerate() {
                if index > 0 {
                    output.push_str(", ");
                }
                output.push_str(name);
                output.push_str(": ");
                write_value(output, value)?;
            }
            output.push('}');
        }
        Value::Sequence(values) => {
            output.push('[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    output.push_str(", ");
                }
                write_value(output, value)?;
            }
            output.push(']');
        }
        Value::AddressReference(address) => {
            output.push_str("ref(");
            write_address(output, address);
            output.push(')');
        }
        Value::Absent => output.push_str("absent"),
        Value::Present(value) => {
            output.push_str("present(");
            write_value(output, value)?;
            output.push(')');
        }
        Value::Tagged { tag, value } => {
            output.push_str(tag);
            output.push('(');
            write_value(output, value)?;
            output.push(')');
        }
    }
    Ok(())
}

fn write_address_list(output: &mut String, addresses: &[Address]) {
    for (index, address) in addresses.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        write_address(output, address);
    }
}

fn write_address(output: &mut String, address: &Address) {
    output.push_str(address.as_str());
}

fn write_string(output: &mut String, value: &str) {
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{0008}' => output.push_str("\\b"),
            '\u{000c}' => output.push_str("\\f"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character.is_control() => {
                output.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => output.push(character),
        }
    }
    output.push('"');
}

fn write_indent(output: &mut String, indent: usize) {
    for _ in 0..indent {
        output.push_str("    ");
    }
}

impl fmt::Display for SerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for SerializeError {}
