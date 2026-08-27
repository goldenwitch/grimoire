# Minimal schema inventory for the case set

Status: empirical proposal; in progress.

This document instantiates the closed schema algebra from
[schema-format.md](./schema-format.md) against the architecture cases in
[v-jepa-2-case-studies.md](./v-jepa-2-case-studies.md) and
[frontier-architecture-case-studies.md](./frontier-architecture-case-studies.md).
It is a schema inventory, not concrete grammar syntax.

The goal is one small shared family of schemas, not one schema per paper or
architecture brand. Paper names, benchmark names, and implementation
vocabularies remain values and provenance. They do not become Grimoire
primitives.

The core authority remains [grimoire.md](../spec/grimoire.md). The schema body
remains the closed Grimoire-native algebra already recorded in
[schema-format.md](./schema-format.md). Extension namespace identifiers are
still owned by namespace minting; the local names below are placeholders until
that work assigns collision-resistant identifiers.

## Value Notation

The following notation is explanatory shorthand for the existing constructors:

- `text` means the text scalar refinement.
- `positive-int` means the positive integer scalar refinement.
- `finite-number` means the finite number scalar refinement.
- `enum{...}` means a closed enumeration.
- `seq(T)` means a homogeneous sequence of `T`.
- `product{...}` means a labeled product.
- `alt{...}` means a tagged alternative.
- `optional(T)` means the existing `absent | present(T)` constructor.
- `ref(address)` means the first-class address reference constructor.

The notation does not add an optional, union, reference, or scalar feature to the
schema algebra. It only makes the proposed value bodies readable before the
concrete grammar exists.

## Minimal Family

| Schema | Attaches to | Purpose | Required by |
| --- | --- | --- | --- |
| `axes/1` | An element chosen by the anchoring convention | Names symbolic axes and carries their human description. | All tensor, token, video, speech, and schedule shapes. |
| `shapes/1` | Primarily ports; exact allowed kinds await the anchoring fixture | Describes ordered dimensions and coarse layout without imposing a paper-specific topology. | Every encoder, tokenizer, latent, action, state, and output interface. |
| `architecture/1` | Blocks, ports, and groups | Records model family, scale, operator facts, attention regime, and interface references. | All architecture families, including low-bit variants. |
| `training/1` | Blocks or groups representing training stages/objectives | Records objective, optimizer, phases, trainable targets, data references, and precision. | Pretraining, fine-tuning, distillation, flow, diffusion, speech, and continual adaptation. |
| `execution/1` | Predictors, planners, streaming paths, or controllers | Makes the static versus streaming, recurrent, or closed-loop boundary explicit. | Speech systems, world models, planning, and autoregressive generation. |
| `precision/1` | Operator blocks or relevant ports | Separates weight, activation, accumulation, optimizer-state, and sparsity facts. | BitNet family, quantized deployment, and cost views. |
| `measurement/1` | The element whose attached value is measured | Carries a literal value, unit, and source record. | Benchmark scores, latency, memory, bandwidth, success rates, and profiled facts. |
| `provenance/1` | Groups, and possibly other elements if a later layer requires it | Carries citations, assumptions, and novelty state. | Every architecture comparison and the provenance layer. |
| `lineage/1` | A model block or future parameter-state artifact | Records parameter-state ancestry and merge operation without pretending it is activation flow. | DARE, TIES-Merging, continual adaptation, and checkpoint reproducibility. |

There are no separate `janus`, `chameleon`, `jepa`, `bitnet`, `qwen`, or
`dreamer` schemas. Those names are values of `architecture.family`,
`training.objective`, or provenance records.

## Candidate channel-claim family

The initial executable registry intentionally remains the nine families above.
Shannon channel claims are the next candidate family, but they do not fit the
current single-element attachment model without an explicit source-terminal
relation boundary.

A candidate information claim, as described in the
[information-flow proposal](./information-flow.md), would name a source port, one terminal port or a
joint terminal set, a quantity such as mutual information, a denominator when
reporting normalized retention, and the distribution, channel or estimator,
method, uncertainty, confidence, and evidence context used to obtain it.

This is not yet an `information/1` registry entry. Adding it before the
relation and uncertainty contracts are reviewed would make a source-terminal
fact look like an ordinary scalar decoration and would obscure branching and
joint-information semantics.

## Schema Bodies

### `axes/1`

Candidate value body:

```text
product{
  name: text,
  description: optional(text)
}
```

The value declares a symbolic axis at an element's definition site. A shape
references that declaration by address. The schema does not infer an axis from
a dimension's spelling and does not assign a semantic role such as time,
height, width, batch, or feature unless a consuming schema explicitly records
one as a value.

This keeps `T`, `H`, `W`, token count, embedding width, action width, and speech
chunk length in one mechanism while avoiding a universal axis ontology.

### `shapes/1`

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

`dimensions` is ordered. A literal dimension covers fixed values such as `7`
for a robot action or `1408` for the V-JEPA 2 ViT-g embedding width. A symbolic
reference covers shared or substituted dimensions such as frame count, spatial
size, token count, or model width.

`layout` distinguishes a TiTok-like one-dimensional latent sequence from a
spatial grid or a spatiotemporal volume without requiring a new topology
constructor. It is deliberately coarse. A layout value does not by itself
assert that token positions correspond to image coordinates; that assertion
requires a separate schema value or a structural relation.

Examples:

- V-JEPA 2 frame representation: `layout:grid`, dimensions `H, W, D`.
- V-JEPA 2 video input: `layout:volume`, dimensions `T, H, W, C`.
- TiTok latent: `layout:sequence`, dimensions `K, D`.
- Droid action: `layout:vector`, dimensions `7`.
- A text token stream: `layout:sequence`, dimensions `N, D`.

### `architecture/1`

Candidate value body:

```text
product{
  family: text,
  parameter_count: optional(positive-int),
  width: optional(positive-int),
  depth: optional(positive-int),
  head_count: optional(positive-int),
  mlp_width: optional(positive-int),
  activation: optional(text),
  position_encoding: optional(text),
  attention_regime: optional(
    enum{causal, bidirectional, block-causal, mixed, unspecified}
  ),
  operator: optional(text),
  interface: optional(ref(address))
}
```

The family is text rather than a closed list of paper names. It can hold
`vision-transformer`, `language-model`, `projector`, `tokenizer`,
`autoencoder`, `diffusion-transformer`, `world-model`, `speech-decoder`,
`probe`, or a future family without changing this schema's algebra.

`attention_regime` is a closed list because the case set repeatedly needs to
state whether visibility is causal, bidirectional, block-causal, or mixed. The
value describes the regime; it does not change structural connections. The
connections or a grammar-defined indexed form must still represent the actual
visibility pattern.

`operator` carries a name such as `linear`, `bitlinear`, `cross-attention`, or
`perceiver-resampler`. Whether that name changes the element kind or remains a
value is intentionally tested by the low-bit fixture.

### `training/1`

Candidate value body:

```text
product{
  objective: text,
  optimizer: optional(text),
  batch_size: optional(positive-int),
  steps: optional(positive-int),
  phases: seq(
    product{
      name: text,
      steps: optional(positive-int),
      learning_rate: optional(finite-number),
      frame_count: optional(positive-int),
      resolution: optional(ref(address))
    }
  ),
  trainable_targets: seq(ref(address)),
  frozen_targets: seq(ref(address)),
  data_sources: seq(text)
}
```

`phases` covers warmup, constant-rate, cooldown, staged alignment,
teacher-forcing, rollout, and continual-adaptation schedules without making one
schedule family normative. A phase's name is a value; its structural effect is
not inferred by the schema.

The target lists record which addressed elements a training stage updates or
freezes. They are intentionally values in this first inventory. They do not
replace a future parameter-update relation when the description needs to make
EMA, delta application, or merge semantics structural.

Examples:

- V-JEPA 2 pretraining: mask-denoising objective, VideoMix22M source values,
  warmup/constant/cooldown phases, and encoder plus predictor targets.
- V-JEPA 2-AC: teacher-forcing plus rollout objective, frozen encoder, Droid
  source, and the action-conditioned predictor target.
- Janus: staged adaptor, unified-pretraining, and supervised-fine-tuning phases.
- BitNet: quantization-aware objective with higher-precision optimizer state.
- DARE/TIES: adaptation objective and source model references, with the actual
  parameter lineage carried separately by `lineage/1`.

### `execution/1`

Candidate value body:

```text
product{
  regime: enum{static, streaming, recurrent, closed-loop},
  horizon: optional(positive-int),
  rate: optional(finite-number),
  external_consumer: enum{yes, no}
}
```

This schema records the boundary without binding a projection to a run. A
static description may say that a predictor is recurrent or that a planner is
closed-loop, but the projection language still does not execute the recurrence,
stream, controller, or replanning loop.

Examples:

- LLaMA-Omni and Mini-Omni: `streaming`.
- Genie and DreamerV3 dynamics: `recurrent` or `static` depending on whether
  the description is of the model graph or a deployment loop.
- V-JEPA 2-AC planning: `closed-loop`, `external_consumer: yes`.
- Flow Matching training: `static`, with sampling runtime remaining external.

### `precision/1`

Candidate value body:

```text
product{
  weights: optional(text),
  activations: optional(text),
  accumulation: optional(text),
  optimizer_state: optional(text),
  sparsity: optional(text)
}
```

Text is used for the representation name because the case set includes FP16,
BF16, ternary weights, INT4, FP4, and mixed strategies. A later standardized
precision vocabulary can constrain these values without changing the schema
shape.

This schema allows the logical transformer graph and deployment precision to
remain separate. A `bitlinear` operator can be a decoration while its ports
remain compatible, or it can be a distinct structural block if a fixture proves
that its interface differs.

### `measurement/1`

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

The source record is mandatory. It prevents a benchmark score, latency, GPU
count, success rate, or profiled norm from appearing as an unexplained literal.
The schema carries the value; a check can inspect decorated measurements, but a
measurement cannot alter structural selection.

A zero-valued measurement is a remaining scalar-domain question because the
initial scalar refinements currently distinguish positive integers from finite
numbers. The fixture should use `finite-number` for quantities where zero is a
legitimate result and should not invent a special zero constructor.

### `provenance/1`

Candidate value body:

```text
product{
  citations: seq(text),
  assumptions: seq(text),
  novelty: enum{novel, existing, adapted, unclassified}
}
```

The provenance layer's novelty check can select groups whose `novelty` value is
`unclassified`. Empty citations and assumptions are representable; the schema
does not make a citation mandatory for every element because the core
specification does not require that constraint.

For the paper cases, citation text can be a DOI, arXiv identifier, repository
origin, or a human-readable reference. The measurement schema's source record
is separate because origin of a measured number is not the same thing as
technical provenance of an element.

### `lineage/1`

Candidate value body:

```text
product{
  base: ref(address),
  deltas: seq(ref(address)),
  operation: enum{continual-update, sparsify-rescale, trim-sign-merge},
  result: ref(address)
}
```

This is the smallest candidate that can name the base, delta inputs, operation,
and resulting parameter state for continual pretraining, DARE, and
TIES-Merging. It is not yet a settled core schema because the current Grimoire
vocabulary has no parameter-state element kind.

Until that boundary is ruled, a description may carry the lineage as an
external artifact or as a value attached to a model block, but it must not
represent parameter merging as an activation connection. This candidate schema
exists to make the missing decision concrete and testable.

## Coverage Matrix

| Case family | Shapes | Axes | Architecture | Training | Execution | Precision | Measurement | Provenance | Lineage |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| V-JEPA 2 encoder and pretraining | yes | yes | yes | yes | optional | optional | yes | yes | no |
| V-JEPA 2-AC and planning | yes | yes | yes | yes | yes | optional | yes | yes | no |
| Shared representation encoders | yes | yes | yes | optional | optional | optional | yes | yes | no |
| Encoder-to-language bridges | yes | yes | yes | yes | optional | optional | yes | yes | no |
| Unified discrete multimodal cores | yes | yes | yes | yes | optional | optional | yes | yes | no |
| Janus-style decoupled frontends | yes | yes | yes | yes | optional | optional | yes | yes | no |
| Continuous latent generation | yes | yes | yes | yes | static boundary | optional | yes | yes | no |
| TiTok one-dimensional tokenization | yes | yes | yes | yes | optional | optional | yes | yes | no |
| Streaming speech | yes | yes | yes | yes | yes | optional | yes | yes | no |
| Latent predictive dynamics | yes | yes | yes | yes | yes | optional | yes | yes | no |
| BitNet low-bit variants | yes | optional | yes | yes | optional | yes | yes | yes | no |
| DARE/TIES/continual adaptation | optional | optional | yes | yes | optional | optional | yes | yes | candidate |

`yes` means the schema body can carry the case's stated facts. It does not mean
that grammar syntax, structural relations, or validation rules for the case are
already settled.

## What Is Deliberately Not a Schema

The following remain structural or projection concerns and must not be hidden
inside metadata:

- whether a connection is directed and which two ports it joins;
- token-level causal, bidirectional, mixed, or block-causal visibility;
- selection and generation of elements;
- inversion of selected connections;
- address folding and competing definitions;
- the fact that a generated element is an ordinary definition;
- a planner's runtime observation and action loop;
- a diffusion, flow, or autoregressive sampling run;
- a check's expected empty or nonempty cardinality.

Schemas describe finalized values attached after structural evaluation. They do
not become a second graph language.

## Minimality Argument

The inventory is minimal at the family level for the current case set:

- `axes` and `shapes` are the common interface contract for all token, latent,
  action, state, and speech streams.
- `architecture` carries repeated block-level facts without one namespace per
  paper.
- `training` carries objective and schedule facts that cannot be inferred from
  architecture alone.
- `execution` prevents a static architecture description from implying runtime
  semantics.
- `precision` keeps low-bit deployment facts separate from logical connectivity.
- `measurement` gives every external number an origin.
- `provenance` supports the existing novelty-surface requirement.
- `lineage` makes parameter-state composition explicit as a bounded candidate
  instead of misrepresenting it as activation flow.

Removing any of these families either drops an observed cross-paper distinction
or forces a different family to carry facts outside its responsibility. Adding a
paper-specific family would duplicate vocabulary without improving the
structural account.

## Sized Gaps

### Axis anchoring

- Binds when: the first shape fixture attaches an axis declaration to a
  definition site.
- Cost of absence now: axis references can be named but the validator cannot
  check the intended anchoring convention.
- Candidate shape: attach `axes/1` to a port that owns the dimension; permit a
  group-level declaration only if a fixture needs shared axes without a port.
- Entry trigger: the first V-JEPA 2 shape fixture.

### Parameter updates

- Binds when: EMA, freeze, fine-tune, or delta application must be checked as a
  relation rather than recorded as a training value.
- Cost of absence now: activation flow and parameter flow remain distinguishable
  only by convention.
- Candidate shape: retain `training/1` target lists for v1 and add a structural
  relation only after a fixture requires validator-level checking.
- Entry trigger: the first fixture containing both an EMA target encoder and an
  activation connection.

### Indexed visibility

- Binds when: a fixture expands more than one time step, token, or modality.
- Cost of absence now: block-causal attention and mixed multimodal visibility
  cannot be checked at the level where the papers claim it matters.
- Candidate shape: generated ordinary connections or a grammar-defined indexed
  form that expands before validation.
- Entry trigger: the first unified-multimodal or V-JEPA 2-AC fixture.

### Symbolic cost values

- Binds when: the cost layer needs a value such as bytes or FLOPs as an
  expression over declared axes rather than a measured literal.
- Cost of absence now: `measurement/1` covers observed values but not symbolic
  substitutions.
- Candidate shape: projection-language symbolic arithmetic over shape values,
  with a schema-governed result; do not add arithmetic constructors to the
  schema body without a concrete expression fixture.
- Entry trigger: the first bytes-on-wire or symbolic-cost fixture.

### Parameter-state elements

- Binds when: a reproducibility cut must contain a base checkpoint, deltas, a
  merge operation, and a result as first-class elements.
- Cost of absence now: `lineage/1` can name the relationship only as a
  candidate value or external artifact.
- Candidate shape: a future artifact or element kind with explicit parameter
  lineage; no activation connection semantics.
- Entry trigger: the first reference description that includes DARE or
  TIES-Merging as a reproducible transformation.

## Fixture Sequence

The smallest validating sequence is:

1. One encoder block with two port shapes and two axis declarations.
2. One pretraining group with architecture, training, measurement, and
   provenance values.
3. One action-conditioned predictor with action/state shapes, execution regime,
   and a frozen encoder target.
4. One bridge-based language consumer and one unified discrete consumer sharing
   only the addresses the description intentionally shares.
5. One TiTok sequence shape and one spatial tokenizer shape to test topology.
6. One speech path with streaming execution values.
7. One BitNet operator with precision values.
8. One parameter-lineage candidate kept external or value-attached, with the
   expected unresolved outcome recorded.
9. One symbolic-cost fixture only after projection arithmetic is concrete.

Every fixture should be cut at least once. Each cut must either validate after
erasure or report the absent declared input at the layer that needs it.
