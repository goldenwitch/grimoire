# Measurement schema

Status: empirical proposal; in progress.

This proposal instantiates the sourced-literal part of the minimal schema
inventory. It is for values measured outside the description, such as benchmark
scores, latency, memory, bandwidth, or robot success rates. It does not define
symbolic cost expressions or runtime evaluation.

The governing sources are [grimoire.md](../spec/grimoire.md),
[schema-format.md](./schema-format.md), and
[schema-inventory.md](./schema-inventory.md).

## Purpose

A literal number in a description is not self-explanatory. A measurement must
carry its unit and the source and protocol that produced it. This lets a later
reader distinguish a reported paper result from a local reproduction, a
profiled number from a configured target, and a success rate from a latency.

The schema is intentionally small. It records a value and its origin; it does
not assert that two measurements are comparable merely because their units are
spelled alike.

## Candidate Contract

Candidate value body:

```text
product{
  value: alt{
    integer: positive-int,
    number: finite-number
  },
  unit: text,
  source: product{
    origin: text,
    locator: optional(text),
    protocol: optional(text)
  }
}
```

`origin` is a required text field. It may be a paper URI, repository path,
experiment record, or other source identifier. `locator` names a section,
table, figure, run, or byte range when one exists. `protocol` records the
measurement procedure or evaluation setup in a compact human-auditable form.

The value alternative distinguishes an integer literal from a finite number
without adding a general numeric union to the schema algebra. A zero-valued
measurement uses `number` until the scalar-domain rules define a more specific
representation. NaN, positive infinity, negative infinity, and an omitted value
are invalid.

The unit is text rather than an enum. The initial case set contains percent,
seconds, bytes, GPU-days, parameters, tokens, centimeters, and other domain
units. A standardized unit vocabulary can be added under a namespace later;
the base measurement schema does not silently convert units.

## Architecture Fit

The indexed papers provide several measurement classes:

- V-JEPA 2 benchmark accuracy, recall-at-5, success rates, and planning time;
- V-JEPA and DINOv2 frozen-probe results;
- Janus, MiniCPM-V, and other multimodal benchmark tables;
- TiTok reconstruction and generation metrics;
- BitNet latency, memory, throughput, and energy comparisons;
- LLaMA-Omni response latency; and
- robot distance-to-goal and manipulation success.

A measurement attaches to the block, port, group, or layer result whose value it
describes. The exact allowed element kinds should follow the reviewed layer and
schema definitions. A measured result does not define a new block and cannot
change structural selection.

## Relationship to Cost

A measured bandwidth or FLOP value is valid under this schema when it records an
observed result. A symbolic expression such as cost at width `D` is not a
measurement literal. It belongs to projection arithmetic over shape and
architecture values, with a schema-governed result once that boundary is
specified.

This distinction prevents the measurement schema from becoming an accidental
arithmetic language:

- `16 seconds per action` is a sourced measurement;
- `O(T * H * W * D)` is a symbolic cost expression; and
- `configured batch size 3072` is a training value, not an observation.

## Provenance Relationship

The source record here answers "where did this number come from?" The
provenance schema answers "what technical work or assumption does this element
carry?" A paper citation may appear in both, but the two records have different
responsibilities and should not be merged into one generic metadata product.

## Fixtures

Valid fixtures:

- integer count with a paper URL and section locator;
- finite percentage with a benchmark protocol;
- zero-valued finite number with a source;
- measured latency with seconds as unit;
- measured robot distance with centimeters as unit;
- same value and unit from two different origins; and
- a measurement attached to each of two folded references.

Invalid fixtures:

- missing source origin;
- missing unit;
- non-finite numeric literal;
- malformed value alternative;
- unsupported element attachment kind;
- an unresolvable source reference if the grammar makes source an address;
- an attempt to use a symbolic axis expression as the literal value; and
- a check that attempts to use a measurement to change structural selection.

The fixture suite should preserve exact source text for unknown extension
namespaces while canonicalizing recognized measurement values.

## Decision Record

This proposal records these decisions for concrete schema work:

- Every measurement has a literal value, unit, and mandatory source record.
- The source record contains required `origin` and optional `locator` and
  `protocol` fields.
- Integer and finite-number values are explicit alternatives.
- Unit text is not silently normalized or converted by the base schema.
- Measurements are finalized values and cannot feed structural selection.
- Symbolic costs and configured training values are not measurement values.

## Sized Gaps

### Unit identity and conversion

- Binds when: a cost or measurement check compares values from different unit
  spellings or scales.
- Cost of absence now: the schema can preserve the source and unit but cannot
  prove comparability or convert `ms` to `s`.
- Candidate shape and rough size: a namespaced unit vocabulary plus explicit
  conversion metadata; do not add implicit conversion to the base schema.
- Entry trigger: the first cross-paper cost or latency comparison fixture.

### Zero and signed values

- Binds when: a valid benchmark, delta, or measurement needs an integer zero or
  a signed literal that finite-number representation does not distinguish from
  other numeric forms.
- Cost of absence now: values can be represented as finite numbers, but the
  scalar contract is less precise than the surrounding field name suggests.
- Candidate shape and rough size: extend scalar refinements with nonnegative or
  signed integer only after a concrete fixture demonstrates the need.
- Entry trigger: the first measurement fixture that requires integer-domain
  validation rather than numeric finiteness.
