# Shapes schema

Status: prototype schema checkpoint with open topology gaps.

This proposal instantiates the tensor and token shape contract for the indexed
architecture cases. It depends on [axes-schema.md](./axes-schema.md),
[schema-format.md](./schema-format.md), and
[schema-inventory.md](./schema-inventory.md).

The purpose is to make interfaces checkable without making shape values a
second graph language or claiming that every dimension has a universal semantic
name.

## Candidate Contract

Candidate value body:

```text
product{
  layout: enum{scalar, vector, sequence, grid, volume},
  dimensions: seq(
    alt{
      literal: positive-int,
      symbolic: ref(address)
    }
  )
}
```

`dimensions` is ordered. A literal dimension is a positive integer. A symbolic
dimension is a first-class address reference to an axis declaration. A shape
with `layout: scalar` has no dimensions; every other layout has at least one
dimension in the fixture contract.

The base schema does not allow a dimension to be a free text label. A consumer
must either use a literal or reference an addressed axis. This makes shared
symbolic dimensions visible to the locality and cut checks.

## Layout

The initial layout enumeration is intentionally small:

- `scalar`: one value with no tensor axes;
- `vector`: an ordered non-spatial collection, such as a 7D action;
- `sequence`: an ordered token or latent sequence;
- `grid`: a two-dimensional spatial feature map with any explicitly listed
  feature or channel dimensions; and
- `volume`: a three-dimensional or higher structured extent such as video with
  temporal and spatial dimensions.

The layout is a declared coarse fact. It does not infer the meaning of an axis
from its name or prove that a grid coordinate corresponds to an input pixel.

A later schema may describe coordinate correspondence, patchification, or
permutation explicitly. The shape schema must not smuggle those claims into
`layout`.

## Architecture Examples

### V-JEPA 2

A frame-level ViT-g representation can be represented as:

```text
layout: grid
dimensions: [height, width, feature]
```

where `height`, `width`, and `feature` are addressed axes or fixed literals.
The paper reports a feature map of `16 x 16 x 1408` for the action-conditioned
use of the ViT-g encoder.

A video input can be represented as:

```text
layout: volume
dimensions: [frames, height, width, channels]
```

The input contract does not assert a particular frame count or resolution; the
training or evaluation layer supplies those values.

### Bridge-based multimodal models

A visual token stream entering a projector can be represented as:

```text
layout: sequence
dimensions: [visual_tokens, visual_width]
```

The projector output can use a different width:

```text
layout: sequence
dimensions: [visual_tokens, language_width]
```

The connection between those ports is structural. The shape values make the
adaptation boundary explicit but do not define the projector's computation.

Dynamic tiling can alter `visual_tokens`. If the tile count or order affects
which token is connected to which consumer, the tile structure must be selected
or generated structurally. A changing token count alone may remain a finalized
value when no structural identity is claimed.

### TiTok

A TiTok latent is represented as:

```text
layout: sequence
dimensions: [latent_tokens, latent_width]
```

It is not represented as a grid merely because it encodes an image. Its shape
contains no hidden height and width axes, so a consumer cannot infer a fixed
spatial correspondence that the one-dimensional tokenizer intentionally removes.

### Robot actions and states

The V-JEPA 2-AC action and end-effector state can each be represented as:

```text
layout: vector
dimensions: [7]
```

The seven components' meanings are architecture or schema values: three
Cartesian position values, three extrinsic Euler-angle values, and one gripper
value. The shape itself only carries the dimension.

### Speech

A speech token or acoustic feature stream can use `sequence` or `volume`
depending on whether the described interface includes only ordered chunks or an
explicit time-frequency structure. The choice is authored by the description;
the shape schema does not inspect the media type and decide for it.

## Axis References

A symbolic dimension reference must resolve to an addressed axis declaration
that is visible from the shape's definition site. The reference is checked by
address, not by comparing the axis's human-readable `name`.

The following conditions are distinct:

- an absent axis address is a reference-resolution failure;
- an axis below the shape's definition site is a scope failure;
- a duplicate axis address is an address uniqueness failure; and
- two visible axes with different addresses are different declarations even if
  their names match.

A shape may use the same axis address in several dimensions only when the
architecture actually gives those dimensions the same extent. The base schema
does not infer equality merely because two axis references are repeated; the
shape value makes the equality explicit by reuse of one address.

## Shape Compatibility

The shape schema describes values; it does not create a general type-checking
system for every connection. A later structural or layer check may require
compatible source and destination shapes for a particular block, projector,
or decoder.

The first compatibility fixture should be narrow:

- a projector whose input and output sequence lengths share one axis address;
- a projector whose feature widths differ but are declared at both ports; and
- a deliberately incompatible connection with a missing or invisible axis.

Do not reject a shape merely because a model's paper uses a non-square image,
variable token count, or a different feature width. Those are valid shape values
when their axes and ports are explicit.

## Topology and Correspondence

The following are separate claims and must not be collapsed into `layout`:

- a sequence has an order;
- a grid has two indexed spatial dimensions;
- a volume has temporal or additional structured dimensions;
- a latent token corresponds to a patch or pixel;
- a tokenizer preserves spatial order; and
- a model's positional encoding represents one or more axes.

V-JEPA 2's 3D-RoPE and tubelet patchification are architecture values and
structural tokenization facts. TiTok's removal of fixed 2D correspondence is a
counterexample that keeps topology and correspondence separate.

## Fixtures

Valid fixtures:

- scalar with empty dimensions;
- one literal vector shape of dimension 7;
- one sequence shape with a token axis and a width axis;
- one grid shape with two symbolic spatial axes and a literal feature width;
- one volume shape with temporal, height, width, and channel dimensions;
- two ports sharing one axis address;
- a TiTok-style sequence shape with no spatial axes; and
- a bridge whose input and output sequence shapes use different width axes.

Invalid fixtures:

- a symbolic dimension that references no address;
- a symbolic dimension that references an axis below the shape definition site;
- a duplicate or malformed layout value;
- a scalar shape with a dimension;
- a non-scalar shape with no dimension;
- a zero or non-finite literal dimension;
- a free text dimension label;
- a shape reference to an element that is not an axis declaration; and
- a consumer that branches structurally on a finalized shape value before
  structural evaluation completes.

The fixtures must include a cut where a shared axis is retained and a cut where
a layer-local axis is erased with its defining layer. The latter must validate
or report a visible absent input, never retain a dangling shape reference.

## Decision Record

This proposal records these decisions for concrete schema work:

- Shape values are products of a coarse layout and ordered dimensions.
- Dimensions are literal positive integers or first-class references to addressed
  axis declarations.
- The initial layouts are `scalar`, `vector`, `sequence`, `grid`, and `volume`.
- Layout does not infer axis semantics or spatial correspondence.
- TiTok's latent is a sequence, not a grid, unless a separate structural or
  schema value says otherwise.
- Shape compatibility is a narrow later check, not a new core graph primitive.
- Dynamic token count is a value until token identity or connectivity changes.

## Sized Gaps

### Axis ownership

- Binds when: a shape needs an axis that no single port naturally owns.
- Cost of absence now: the first fixture is constrained to port-anchored axis
  declarations.
- Candidate shape and rough size: allow a group-level axis anchor with the same
  value body; one attachment rule and one cut fixture.
- Entry trigger: a shared architecture group with a common axis but no
  dimension-owning port.

### Shape equality and transforms

- Binds when: a connection performs reshape, flatten, pooling, tiling, or
  permutation and a validator must check its shape relation.
- Cost of absence now: shapes can be attached to ports, but the transform's
  dimension relation remains prose.
- Candidate shape and rough size: schema-governed transform values or ordinary
  addressed transform blocks with explicit input/output ports. Do not add
  implicit shape inference.
- Entry trigger: the first bridge, tokenizer, or projector fixture that needs
  validator-level transform checking.

### Token correspondence

- Binds when: a downstream consumer relies on spatial or temporal identity of a
  token rather than only its ordered shape.
- Cost of absence now: TiTok and spatial tokenizers can be distinguished
  coarsely, but correspondence cannot be checked.
- Candidate shape and rough size: a separate schema value naming coordinate
  topology and permutation; one schema fixture family.
- Entry trigger: a cost, placement, or attention projection references token
  coordinates.
