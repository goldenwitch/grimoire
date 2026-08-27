# V-JEPA 2 architecture case studies

Status: empirical proposal; in progress.

This document is a consumption exercise for Grimoire. It asks whether the
language can represent the structures and training views of V-JEPA 2 without
turning the paper's vocabulary, metrics, or implementation choices into
Grimoire requirements.

The authority for the language remains [grimoire.md](../spec/grimoire.md).
The semantic boundary for projections remains
[projection-language.md](./projection-language.md), and the schema boundary
remains [schema-format.md](./schema-format.md).

## Sources

The primary source is [V-JEPA 2: Self-Supervised Video Models Enable
Understanding, Prediction and Planning](https://arxiv.org/abs/2506.09985),
version 1. The readable full-text mirror is
[the ar5iv rendering](https://ar5iv.labs.arxiv.org/html/2506.09985). The
inherited representation-space training pattern is described in
[V-JEPA: Revisiting Feature Prediction for Learning Visual Representations from
Video](https://arxiv.org/abs/2404.08471).

The paper's relevant sections are:

- Section 2 and Appendix A: action-free V-JEPA 2 pretraining.
- Section 3 and Appendix B: the V-JEPA 2-AC action-conditioned world model.
- Section 4: image-goal planning and closed-loop robot control.
- Section 5: probe-based visual classification.
- Section 6: probe-based human action anticipation.
- Section 7 and Appendix E: video question answering and language alignment.

Facts in this document are marked as observations in prose. Address labels and
site placements are proposed case-study choices, not concrete grammar syntax.
No serialized Grimoire form is implied until the grammar is reviewed.

## Observed System

The paper gives us several distinct consumers of one learned representation.
They should be represented as layers over shared structure, not as unrelated
model descriptions.

| Surface | Observed structure | Relevant boundary |
| --- | --- | --- |
| Observation pretraining | An action-free encoder and predictor learn to predict masked video representations. An EMA copy of the encoder supplies stopped-gradient targets. | Image and video observations, mask tokens, representation-space L1 loss. |
| V-JEPA 2 encoder | A ViT encoder is scaled from ViT-L at 300M parameters through ViT-H at 600M to ViT-g at 1B. The ViT-g has width 1408, depth 40, 22 heads, and MLP width 6144. | The encoder representation is reused by every downstream surface below. |
| Pretraining predictor | The pretraining predictor is held at a ViT-s-like 22M architecture with width 384, depth 12, 12 heads, and MLP width 1536. | It is used by the masked prediction objective and by action anticipation, but not by V-JEPA 2-AC. |
| Action anticipation | The frozen encoder and the pretraining predictor are used together. A future mask token produces a predicted future representation, which is concatenated with encoder output and sent to a three-query attentive probe for action, verb, and noun. | The prediction task is a downstream layer over the learned representation and pretraining predictor. |
| Video question answering | Encoder patch embeddings are projected into an LLM input space. The paper uses staged image and video alignment with Qwen2-7B-Instruct or Llama 3.1 8B, with frozen and unfrozen encoder variants. | The projector and LLM are downstream structure, not additions to the shared encoder. |
| V-JEPA 2-AC | The encoder is frozen and applied independently to each frame. A new approximately 300M transformer predicts the next frame representation from prior representations, 7D actions, and 7D end-effector states. | This is a separate action-conditioned predictor, trained above the shared encoder. |
| Planning | An image goal is encoded into the same representation space. Candidate actions are rolled forward through V-JEPA 2-AC, scored by L1 distance to the goal representation, optimized with the Cross-Entropy Method, and executed in a receding-horizon loop. | The static architecture can be described; runtime execution is outside a pure static projection. |

## Address and Site Inventory

This is the first proposed placement of the paper's elements. It is meant to
exercise locality and cuts. The labels are illustrative addresses, not a
commitment about address spelling.

| Illustrative address | Proposed definition site | Why this site is visible |
| --- | --- | --- |
| `vjepa2/vision-encoder` | Core graph | Pretraining, action conditioning, classification, anticipation, VidQA, and planning all reference it. |
| `vjepa2/pretraining/predictor` | `vjepa2/pretraining` layer | Pretraining defines it; action anticipation consumes its learned representation-prediction behavior. |
| `vjepa2/pretraining/target-encoder` | `vjepa2/pretraining` layer | It exists to produce EMA targets for the action-free objective. |
| `vjepa2/pretraining/mask-token` | `vjepa2/pretraining` layer | It belongs to masked representation prediction and is not needed by V-JEPA 2-AC. |
| `vjepa2/ac/action-input` | `vjepa2/ac` layer | It is specific to the robot interaction data and the action-conditioned predictor. |
| `vjepa2/ac/end-effector-state` | `vjepa2/ac` layer | It is a 7D proprioceptive input to V-JEPA 2-AC. |
| `vjepa2/ac/predictor` | `vjepa2/ac` layer | It is a new predictor with a different architecture from the pretraining predictor. |
| `vjepa2/anticipation/probe` | `vjepa2/anticipation` layer | The probe is trained for the EK100 task and is not shared model structure. |
| `vjepa2/vidqa/projector` | `vjepa2/vidqa` layer | The projector adapts visual embeddings to one language backbone and training recipe. |
| `vjepa2/vidqa/language-model` | `vjepa2/vidqa` layer | The LLM is a downstream consumer and may vary independently of the encoder. |
| `vjepa2/planning/goal` | `vjepa2/planning` layer | Goal encoding and action scoring belong to the planning viewport. |
| `vjepa2/planning/controller` | `vjepa2/planning` layer | The controller is part of deployment structure, not representation pretraining. |

The proposed sites yield these declared-input chains:

- `vjepa2/pretraining` consumes the core graph.
- `vjepa2/anticipation` consumes the core graph and `vjepa2/pretraining`.
- `vjepa2/ac` consumes the core graph.
- `vjepa2/vidqa` consumes the core graph.
- `vjepa2/planning` consumes `vjepa2/ac` and the core graph.

The pretraining predictor is deliberately not placed in the core graph. Its
use by action anticipation is visible through the declared input chain, while
V-JEPA 2-AC receives only the shared encoder representation. This is the
locality choice exercised by the case study.

## Case 1: Shared Representation Core

### Question

Can the core graph carry the stable representation contract while each layer
owns the machinery that exists only for one training or deployment surface?

### Proposed structural account

The core graph contains a vision encoder with ports for visual input and patch
representations. At the architecture level, the encoder is one addressed block.
If a future description needs transformer-block-level accounting, the encoder
can be expanded into a group of addressed blocks without changing the external
encoder address used by consuming layers.

The encoder's factual interface includes:

- video tubelets of size `2 x 16 x 16`;
- 3D RoPE over temporal, height, and width axes;
- a sequence of patch embeddings as output;
- the ViT-L, ViT-H, and ViT-g scale points reported by the paper.

The core graph does not need to claim that the representation means motion,
objects, or robot state. Those meanings are supplied by layers. The same
encoder address can therefore be consumed by an understanding layer, a
language-alignment layer, and an action-conditioned layer without duplicating
identity.

### Proposed finalized data

Shape and architecture values attach to the encoder and its ports through
schema-governed extension parameters. Candidate value families are:

- input and output shapes, including symbolic temporal and spatial axes;
- parameter count, width, depth, head count, and MLP width;
- position encoding kind and tubelet size;
- source provenance for the reported architecture and checkpoint.

These are values about the structure. They do not decide which elements are
selected by a downstream layer.

### What this exercises

- One shared address folded into several reprojections.
- An architecture-level block that can later be expanded without changing
  downstream references.
- Shapes and architecture facts as decorations rather than graph identity.
- Minimal cuts for encoder-only consumers.

No use of `invert` is claimed here. The V-JEPA 2 paper does not describe an
inverted connection view of this architecture.

## Case 2: Action-Free Representation Pretraining

### Observed facts

The pretraining objective masks a subset of video tokens. The context encoder
processes the unmasked tokens. Learnable mask tokens identify the dropped
positions and are processed with the context output by the predictor. An EMA
encoder computes target representations from the unmasked full input. The
predictor output is compared with the stopped-gradient target only at masked
positions using an L1 loss.

The main training recipe uses VideoMix22M: 22 million samples assembled from
SSv2, Kinetics, HowTo100M, curated YT-Temporal-1B, and ImageNet images treated
as repeated-frame video. The recipe starts with 16 frames at 256 x 256, then
uses a cooldown phase that increases frame count and resolution. These facts are
training values, not new core primitives.

### Proposed layer account

`vjepa2/pretraining` declares the core graph as an input. Its structural
selection references `vjepa2/vision-encoder` and defines:

- `vjepa2/pretraining/predictor`;
- `vjepa2/pretraining/target-encoder`;
- `vjepa2/pretraining/mask-token`;
- the masked-target objective and its ports.

The structural path joins the encoder output to the predictor and the target
encoder. The mask token joins the predictor at the masked positions. The
objective joins predicted masked representations to target representations.

The following values are finalized on the selected elements:

- the L1 objective and masked-position reduction;
- EMA coefficient and stop-gradient status;
- VideoMix22M source composition and sampling weights;
- frame count, frame rate, crop size, tubelet size, and mask ranges;
- warmup, constant-rate, and cooldown schedule parameters;
- the paper and section that provide each fact.

### Parameter-update boundary

The EMA target encoder is a structural participant in pretraining, but the
statement that its weights are an exponential moving average of the context
encoder is not an ordinary data-flow edge. This case study records it as a
layer-local update relation attached to the encoder pair. Whether that relation
is represented as a directed **connection**, a generated structural element, or
a schema-governed finalized value is left as a sized gap below. The proposal
does not hide it inside an ordinary tensor port.

### Cut

The minimal pretraining cut contains the core graph and
`vjepa2/pretraining`. It is self-contained because the target encoder, mask
tokens, predictor, and objective are all defined in that layer or below it.

## Case 3: V-JEPA 2-AC Action-Conditioned World Model

### Observed facts

V-JEPA 2-AC is post-trained on less than 62 hours of unlabeled Droid robot
videos. A training sample contains 16 frames at 4 fps, 256 x 256 resolution,
15 actions, and 16 end-effector states. Each state is a real-valued 7D vector:
three Cartesian position values, three extrinsic Euler-angle values, and one
gripper value. Each action is a real-valued 7D change in end-effector state.

The frozen V-JEPA 2 encoder is applied independently to each frame. With the
ViT-g encoder, each frame representation has shape `16 x 16 x 1408`. The new
predictor is approximately 300M parameters, with 24 layers, 16 heads, hidden
width 1024, and GELU activations. Separate learned affine maps bring the action,
state, and flattened feature-map inputs into the predictor hidden dimension; an
output affine map returns predictions to the encoder embedding dimension.

The predictor consumes temporally interleaved action, state, and feature inputs.
Its block-causal attention allows a patch at time `t` to attend to action, state,
and patch features at time `t` and at earlier times. It predicts the next frame
representation autoregressively. Training combines teacher forcing with a
2-step rollout loss.

### Proposed layer account

`vjepa2/ac` declares the core graph as an input. It references the shared
encoder and defines:

- an action input block with a 7D action port;
- an end-effector state block with a 7D state port;
- input affine maps for action, state, and visual features;
- `vjepa2/ac/predictor`;
- an output affine map to the encoder representation shape;
- teacher-forcing and rollout objective elements.

The block-causal attention pattern is structural. It should therefore be part of
selection or generated structural elements, not a decoration that later changes
which tokens can see one another. A future concrete grammar must be able to
express the token-indexed connections as ordinary addressed elements or provide
a grammar-defined generated form for them.

The following values are finalized after the structural fold:

- the encoder is frozen during action-conditioned post-training;
- action and state dimensions and component meanings;
- affine-map dimensions;
- teacher-forcing horizon and rollout horizon;
- optimizer and schedule values;
- Droid provenance and the absence of reward or task-success labels.

### Shape relation

The predictor's output must be understood as a representation of the same
addressed visual state kind as the frozen encoder output. A shapes schema may
express that relationship if the schema defines it. The core specification does
not acquire a new shape-compatibility primitive from this case study.

### Cut

The minimal action-conditioned cut contains the core graph and `vjepa2/ac`.
It does not need the pretraining predictor, target encoder, or mask-token
structure. This is an empirical reason to keep V-JEPA 2-AC as a separate layer
rather than treating it as a replacement definition of the pretraining
predictor.

## Case 4: Downstream Understanding Layers

V-JEPA 2 exposes more than one downstream consumer. These should be separate
layers because their local structure and declared outputs differ.

### Visual classification

A classification layer references the shared encoder and defines a four-block
attentive probe. The final probe block uses cross-attention with a learnable
query token, followed by a classifier. The encoder remains frozen in the
reported frozen-evaluation protocol.

The layer decorates the probe with task identity, input frame protocol,
resolution, and measurement results. The six reported tasks include three
motion tasks and three appearance tasks. The measurements belong to the layer's
provenance and measurement values; they do not become graph structure.

Its minimal cut is the core graph plus the classification layer.

### Human action anticipation

An anticipation layer declares the core graph and `vjepa2/pretraining` as
inputs. It references the shared encoder and the pretraining predictor. The
predictor receives a future mask token corresponding to the anticipated frame.
The encoder output and predicted future representation are concatenated before
entering a three-query attentive probe. The three query outputs feed separate
classifiers for action, verb, and noun, and the paper trains them with summed
focal losses.

This layer demonstrates why two elements with similar names must not be
silently joined: the anticipation layer uses the original representation-space
predictor, while V-JEPA 2-AC defines a different action-conditioned predictor.
Their addresses remain distinct and their shared encoder reference remains one
address.

Its minimal cut is the core graph, `vjepa2/pretraining`, and the anticipation
layer. The cut includes the pretraining predictor because the anticipation layer
can see it only through that declared input chain.

### Video question answering

A VidQA layer references the shared encoder and defines a projector plus a
language model. The projector maps visual patch embeddings into the LLM input
space. The paper reports both frozen-encoder and end-to-end variants, as well as
progressive alignment stages over image captioning, image question answering,
and video captioning or question answering.

The projector and language model are local elements of this layer. They are not
placed in the core graph because a different language consumer may use a
different projector, LLM, pooling ratio, alignment dataset, or training stage.
The layer decorates these choices and the resulting benchmark measurements.

Its minimal cut is the core graph plus the VidQA layer. The action-conditioned
predictor is not required by this case.

## Case 5: Goal-Conditioned Planning Boundary

### Observed facts

At deployment, the current frame and an image goal are encoded by the shared
V-JEPA 2 encoder. V-JEPA 2-AC rolls candidate action sequences forward from the
current representation and end-effector state. The planner scores a candidate
by L1 distance between its imagined future representation and the goal
representation. The Cross-Entropy Method refines candidate action
 distributions, the first action is executed, and the process repeats in a
model-predictive-control loop.

The paper uses image goals and, for pick-and-place, intermediate sub-goal
images. It does not define language goals for this system. It reports a
sensitivity to camera position and a degradation of prediction accuracy over
longer autoregressive rollouts.

### Proposed static account

`vjepa2/planning` declares `vjepa2/ac` and the core graph as inputs. Its static
structure references:

- current visual observation and current end-effector state;
- a goal image and its encoder representation;
- a candidate action sequence;
- the V-JEPA 2-AC rollout;
- an L1 goal energy;
- a CEM refinement block;
- an action output and a low-level controller interface.

The action constraint, planning horizon, number of CEM samples, refinement
steps, goal schedule, camera arrangement, and controller details are finalized
values. They are not inputs to structural selection.

The closed-loop execution itself is not represented as a pure static
**projection**. The description can state the static planner structure and
attach its deployment facts, but runtime observation, action execution, and
replanning belong to an external consumer. This boundary is a feature of the
case study: representing the architecture must not quietly introduce run
binding into the projection language.

### Cut

The planning cut contains the core graph, `vjepa2/ac`, and `vjepa2/planning`.
It is larger than the encoder-only and VidQA cuts because its declared inputs
include the action-conditioned predictor.

## What the Case Studies Test

A successful later implementation of this worked description should show that:

- the encoder's one address folds into pretraining, AC, classification,
  anticipation, VidQA, and planning reprojections;
- the pretraining predictor and AC predictor remain distinct definitions;
- action anticipation can see the pretraining predictor through its declared
  input chain without making that predictor global core structure;
- layer-local projectors, probes, goals, and controllers do not leak into other
  layers;
- shapes, architecture facts, freeze status, schedules, measurements, and
  provenance remain finalized values rather than structural inputs;
- token-indexed block-causal attention can be represented as structure;
- the minimal cuts above erase cleanly and remain valid descriptions;
- the planning layer records static architecture without pretending to evaluate
  a runtime loop;
- checks, when used, observe only finalized decoration values.

The case studies intentionally do not force an example of `invert`. They also do
not introduce a new core primitive for tensors, training, robot actions, or
runtime control.

## Sized Gaps

### Parameter update and freeze relations

- Binds when: the first reviewed grammar fixture represents EMA targets or a
  frozen encoder as more than prose.
- Cost of absence now: the V-JEPA 2-AC and pretraining cases can be described,
  but the distinction between data flow and parameter-update semantics remains
  underspecified.
- Candidate shapes and rough size: a layer-local directed relation over
  parameter ports; or a schema-governed value attached to a block pair. This is
  a small semantic decision plus one fixture family.
- Entry trigger: grammar construction reaches training relations or the
  validator needs to distinguish the two cases.

### Generated token-level attention structure

- Binds when: a description expands block-causal attention or masked-token
  selection beyond an architecture-level summary.
- Cost of absence now: the paper's most important causal visibility rule would
  be prose or an uncheckable value rather than structure.
- Candidate shapes and rough size: explicit generated connections with ordinary
  addresses; or a grammar-defined indexed structural form that expands to
  ordinary elements before validation. This is a grammar and projection surface
  plus conformance fixtures.
- Entry trigger: the first concrete V-JEPA 2-AC fixture that includes more than
  one time step and more than one patch.

### Static descriptions of runtime planning

- Binds when: the worked description includes CEM, receding-horizon control, or
  another loop that consumes observations at execution time.
- Cost of absence now: the architecture can be recorded, but no claim can be
  made that Grimoire represents or evaluates the deployed controller.
- Candidate shapes and rough size: a static planner layer with an explicit
  external-execution boundary; or a separate execution artifact attached to the
  description but excluded from projections. This is a scope clarification and
  an acceptance fixture, not a runtime-language expansion.
- Entry trigger: planning is included in the first end-to-end worked
  description.

### Schemas for model and robot facts

- Binds when: the case study is converted into a machine-checked description.
- Cost of absence now: shapes, 7D action/state semantics, schedules, freeze
  status, and benchmark values can be named but not validated uniformly.
- Candidate shapes and rough size: one architecture schema, one robot-state
  schema, one training-schedule schema, and reuse of shapes, axes, measurement,
  and provenance schemas. The exact extension namespaces and anchoring remain
  design work.
- Entry trigger: schema work begins for the reference description.

## Next Consumption Step

The next artifact should be a small reviewed-grammar fixture for the
encoder/pretraining/AC boundary only. It should include one encoder, one
pretraining predictor, one EMA target path, one action-conditioned predictor,
and two declared cuts. The fixture should be small enough to expose address,
locality, shape, and parameter-update questions before the full downstream
surface is attempted.
