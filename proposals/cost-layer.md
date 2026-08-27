# Cost layer

Status: prototype contract; in progress.

This proposal records the smallest symbolic cost view needed by the observed
architecture papers. It evaluates authored expressions over explicit axis
bindings and aggregates priced members through addressed groups. It does not
turn a projection into a profiler or infer a model's cost from names,
topology, precision labels, or paper family.

The governing sources are [grimoire.md](../spec/grimoire.md),
[architecture-vocabulary.md](./architecture-vocabulary.md),
[axes-schema.md](./axes-schema.md), [shape-schema.md](./shape-schema.md), and
the observed cases in [frontier-architecture-case-studies.md](./frontier-architecture-case-studies.md).

## Expression algebra

The first Rust prototype exposes four constructors:

```text
constant(n)
axis(address)
sum(expressions)
product(expressions)
```

An axis is a first-class address reference to a caller-supplied positive
extent. Empty sums and products use the ordinary identities zero and one. All
arithmetic is checked and overflow is an error. The expression result is a
unitless authored cost until the caller names its interpretation, such as
multiply-add count or a symbolic proxy for memory traffic.

The prototype intentionally keeps this algebra separate from the current text
grammar. Adding symbolic expressions to projection syntax would require a
separate grammar and value contract; the current API is a reversible analysis
boundary over an evaluated static reprojection.

## Group aggregation

A `CostModel` assigns one expression to each priced addressed element. A
`CostReport` evaluates those expressions and can recursively sum a selected
group's members. Group membership is structural and addressed. Repeated
members, cyclic groups, missing elements, and unpriced leaves fail visibly;
there is no silent partial total.

This keeps aggregation explicit for groups such as a visual encoder pipeline,
a bridge, a low-bit operator variant, or a planning subgraph. The group does
not acquire a hidden cost field and a decoration never changes structural
selection.

## Observed-paper fit

The algebra can express the recurring dimensions in the corpus:

- V-JEPA 2 attention or projection proxies over frame, token, and feature
  axes;
- bridge and projector costs where token count and hidden widths differ;
- low-bit variants where the logical graph is shared but an authored operator
  expression changes; and
- aggregate costs for a selected architecture group.

The expression alone does not assert the correctness of an implementation's
FLOP convention, memory layout, sparsity behavior, or hardware throughput.
Those require a method and, when observed rather than symbolic, a sourced
`measurement/1` value.

## Sized gaps

- The text grammar has no symbolic arithmetic production yet.
- No universal FLOP, byte, sparsity, precision, or hardware cost convention is
  selected.
- Shape compatibility and precision-to-cost rules remain explicit caller
  inputs rather than inferred transformations.