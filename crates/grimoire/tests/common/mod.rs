#![allow(dead_code)]

use grimoire::{
    Address, Channel, ChannelLink, ChannelNode, Distribution, Schema, prototype_schemas,
};

pub(crate) fn address(value: &str) -> Address {
    Address::parse(value).unwrap_or_else(|error| panic!("{error}"))
}

pub(crate) fn schemas() -> Vec<Schema> {
    prototype_schemas().unwrap_or_else(|error| panic!("{error}"))
}

pub(crate) fn binary_source() -> Distribution {
    Distribution::uniform(2).unwrap_or_else(|error| panic!("{error}"))
}

pub(crate) fn node(
    address_value: &str,
    block: &str,
    input_ports: &[&str],
    output_port: &str,
    channel: Channel,
) -> ChannelNode {
    ChannelNode::new(
        address(address_value),
        address(block),
        input_ports.iter().map(|value| address(value)).collect(),
        address(output_port),
        channel,
    )
    .unwrap_or_else(|error| panic!("{error}"))
}

pub(crate) fn link(source: &str, destination: &str) -> ChannelLink {
    ChannelLink {
        source: address(source),
        destination: address(destination),
    }
}
