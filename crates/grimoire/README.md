# Grimoire

Grimoire is a Rust library and command-line tool for static, addressed
descriptions of machine-learning systems. The library preserves one structural
model with layered views; the CLI validates, canonicalizes, evaluates, cuts,
and reports explicit typed resources over that model.

## Install

Install the command-line tool from crates.io:

```text
cargo install grimoire --version 1.0.0
```

Use the library from Rust:

```toml
[dependencies]
grimoire = "1.0.0"
```

## Included examples

From the root of an unpacked crate archive, the included examples can be
checked with the installed CLI:

```text
grimoire validate examples/reference.grimoire
grimoire canonicalize examples/reference.grimoire
grimoire evaluate examples/reference.grimoire architecture
grimoire resources examples/reference.grimoire cost examples/reference-resources.tsv
```

The Scry architecture fixture uses the same commands with
`examples/scry.grimoire` and `examples/scry-resources.tsv`. Resource events are
explicit analysis inputs; FLOP work, bytes, memory, bandwidth, and latency
remain separate dimensions.

The library and CLI are static and offline. They do not execute training,
sampling, controllers, recurrent processes, or network operations.