# Axes schema

Status: empirical proposal; in progress.

This proposal instantiates the symbolic-axis part of the minimal schema inventory
for the architecture cases. It does not decide the concrete grammar or the
final extension namespace identifier. The governing sources are
[schema-format.md](./schema-format.md), [schema-inventory.md](./schema-inventory.md),
and [grimoire.md](../spec/grimoire.md).

## Purpose

An axis is a named symbolic dimension declared at an element's definition site.
A shape may reference the declaration by address. The axis schema gives that
addressed declaration a small machine-checkable value without inferring meaning
from a spelling.

The same schema covers:

- frame count in a video clip;
- spatial height and width;
- embedding or hidden width;
- token count;
- action or state width;
- speech chunk length; and
- any other dimension shared by selected interfaces.

The schema does not claim that an axis named `T` is time or that an axis named
`D` is an embedding dimension. That interpretation belongs to the consuming
architecture or shape schema.

## Candidate Contract

Candidate value body:

```text
product{
  name: text,
  description: optional(text)
}
```

The schema allows an empty description. A human name is required because an
address without a readable label is difficult to audit, but the label is not a
semantic type.

The schema does not include an extent. A fixed extent such as 16 frames or 1408
features is a shape value at a port. An axis can remain symbolic across model
variants and cuts even when one selected architecture supplies a literal extent.
This avoids making a declaration carry two competing sources of dimensional
truth.

## Attachment Convention

For the first fixture, an axis declaration attaches to the port that owns the
dimension at its definition site. A shape on another port references that axis
by address.

This is a fixture convention, not a frozen core rule. A group-level anchor may
be needed if several ports share a symbolic axis before any one port is defined;
that case remains a gap rather than an implicit exception.

The first fixture should therefore contain:

- one port defining `frames`;
- one port defining `features`;
- one output port whose shape references both declarations; and
- one unrelated layer that cannot see the declarations.

The validator must check the address and visibility rules, not the English names.

## Architecture Fit

V-JEPA 2 supplies direct cases:

- a temporal axis for 16- or 64-frame clips;
- spatial axes for 256, 384, or 512 resolution;
- a feature axis of 1408 for the ViT-g representation; and
- a sequence position axis for patch or token sequences.

The action-conditioned case supplies a literal vector dimension of 7. It does
not require a symbolic axis, but it can use one if several robot interfaces
share the same action width.

TiTok supplies a token-count axis without a spatial grid. Speech systems supply
chunk or frame axes. A bridge-based language layer can use separate visual-token
and language-hidden axes and connect them through a projector whose shapes make
the conversion explicit.

## Locality

An axis declaration follows the same definition-site and locality rules as any
other addressed element:

- an axis referenced by several layers belongs at the lowest site all references
  can see;
- a layer-local axis remains invisible to layers that do not declare that input;
- a tie in legal placement is authored, not a validator warning; and
- an unreferenced axis is allowed and is reportable only through explicit
  finalization, not as a structural error.

An axis address is flat and unique across the description. A shape reference to
an absent or below-scope axis is a validation failure with the axis identifier
and source location.

## Fixtures

Valid fixtures:

- one literal-only shape with no axis declarations;
- one port-declared axis referenced by an output shape;
- two ports at one site sharing one axis;
- an axis declared in the core and referenced by two layers;
- a layer-local axis referenced only by that layer; and
- a legal tied placement where either of two sites is visible to all
  referencers.

Invalid fixtures:

- duplicate axis address;
- a shape reference to a missing address;
- a shape reference from a layer below the axis definition site;
- an axis declaration with an absent name or non-text name;
- an axis value attached to an element kind outside the reviewed allowed set;
  and
- a layer that declares an input but cannot see an axis required by its shape.

The fixture must also distinguish a bad axis reference from a bad shape
arithmetic or a bad port connection. Each failure names the responsible check.

## Decision Record

This proposal records these decisions for the concrete schema work:

- An axis value is a labeled product of required `name` and optional
  `description`.
- Axis names are labels, not inferred semantic types.
- Fixed extents live in shape values; the axis declaration does not duplicate
  them.
- The initial anchoring fixture places an axis declaration on a dimension-owning
  port.
- Axis references are first-class address references and obey normal locality.
- No axis ontology for time, height, width, batch, token, or feature is added to
  the core specification.

## Sized Gaps

### Anchoring beyond ports

- Binds when: a real case needs a shared axis before a dimension-owning port
  exists or needs a group-level declaration.
- Cost of absence now: the initial fixture cannot represent that arrangement
  without choosing a port anchor.
- Candidate shape and rough size: allow `axes/1` on a group while retaining the
  same value body; one attachment rule and two fixtures.
- Entry trigger: the first reference description whose common axis has no
  natural port owner.

### Axis compatibility

- Binds when: two declarations with different addresses or names are proposed
  as the same dimension in one connection.
- Cost of absence now: address reference and connection checks do not establish
  semantic dimension equality.
- Candidate shape and rough size: a shape or architecture check over referenced
  axis addresses; do not add equality semantics to the schema body without a
  concrete mismatch fixture.
- Entry trigger: the first shape fixture that connects two symbolic dimensions.
