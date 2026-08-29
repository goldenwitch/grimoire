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

## Documentation

- [Core specification](spec/grimoire.md): normative semantics and vocabulary.
- [Concrete grammar](grammar/grimoire.md): document syntax and serialization.
- [ML project workflow](guidance/ml-project-workflow.md): a practical modeling
  workflow.
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