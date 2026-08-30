# Grimoire

Grimoire is a design-time and review-time description language for machine
learning systems. It represents one addressed structural graph with layered
views, typed finalized values, sourced measurements, and explicit analysis and
runtime boundaries.

The repository contains a Rust reference implementation and a browser-openable
static architecture viewer. The implementation is intentionally static: it
does not run training, sampling, controllers, recurrent processes, or network
operations.

## Quick start

```text
cargo test --workspace
```

Open [viz/index.html](viz/index.html) directly to inspect the static viewer.

## Choose a workflow

### [Cost and resource analysis](guidance/ml-project-workflow.md#cost-and-resource-analysis)

Use explicit inputs to calculate symbolic costs and bytes on wire. Use typed
resource events and finite workload scenarios to report non-fungible resource
quantities. Use sourced measurements to record observed or estimated values,
including their units, protocols, assumptions, and uncertainty when the claim
format supports it. Neither path infers hardware, runtime behavior, or resource
values from names or topology.

The command-line path is intentionally thin:

```text
cargo run --package grimoire -- validate examples/reference.grimoire
cargo run --package grimoire -- canonicalize examples/reference.grimoire
cargo run --package grimoire -- evaluate examples/reference.grimoire architecture
cargo run --package grimoire -- resources examples/reference.grimoire cost examples/reference-resources.tsv
```

The resource events file is an explicit analysis input. FLOP work, bytes, and
memory remain separate report dimensions; scenario probabilities describe the
workload cases and do not express measurement uncertainty.

For a grounded architecture sample, run the same path against the public Scry
fixture:

```text
cargo run --package grimoire -- validate examples/scry.grimoire
cargo run --package grimoire -- evaluate examples/scry.grimoire architecture
cargo run --package grimoire -- resources examples/scry.grimoire cost examples/scry-resources.tsv
```

The Scry fixture uses public implementation facts as sourced measurements and
keeps workload assumptions and resource quantities in the explicit sidecar.
Grimoire's MCP adapter is deferred from this release. Local Scry MCP was used
only as private development evidence; it is not required for the core
workflows and does not add another resource or architecture language.

### [Architecture substrate and diagrams](guidance/ml-project-workflow.md#architecture-substrate-and-diagrams)

Define stable addressed structure once, add layers for distinct views, validate
and canonicalize it, then derive cuts or inspect it in the read-only viewer.
The addressed description is the model; the diagram is a presentation of that
model and does not become a second source of identity.

The viewer's presentation function is pure over its model and UI state. DOM
writes and interaction wiring are effects outside that function.

## Documentation

- [Core specification](spec/grimoire.md): normative semantics and vocabulary.
- [Concrete grammar](grammar/grimoire.md): document syntax and serialization.
- [ML project workflow](guidance/ml-project-workflow.md): a practical modeling
  workflow.
- [Reference example](examples/reference.grimoire): a compact description that
  demonstrates both primary workflows.
- [Resource events](examples/reference-resources.tsv): explicit workload and
  typed-resource inputs for the reference example.
- [Scry architecture example](examples/scry.grimoire): a public addressed
  ingestion, search, handle, and transport substrate.
- [Scry resource events](examples/scry-resources.tsv): explicit workload and
  typed-resource inputs for the Scry example.
- [Resource-flow contract](proposals/resource-flow.md): typed, probabilistic,
  machine-agnostic resource accounting.
- [Schema inventory](proposals/schema-inventory.md): the shared schema family
  contracts.
- [Architecture vocabulary](proposals/architecture-vocabulary.md): mappings
  from recurring architecture concepts to Grimoire primitives.
- [Case studies](proposals/v-jepa-2-case-studies.md): the primary worked system.
- [Visualization boundary](proposals/visualization.md): the viewer contract.

The remaining proposals record focused contracts and explicit boundaries for
projection, schemas, information flow, placement, cost, provenance, execution,
and extension namespaces. The Rust fixtures under
[crates/grimoire/tests](crates/grimoire/tests) provide executable examples.