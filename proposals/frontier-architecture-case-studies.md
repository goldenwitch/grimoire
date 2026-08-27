# Frontier architecture case studies

Status: empirical proposal; in progress.

This document is a cross-paper consumption exercise for Grimoire. It asks
whether the language can represent the major architecture families in the
indexed paper set while keeping three things separate:

1. observed facts about a paper;
2. an illustrative Grimoire structural account; and
3. questions that require a language or schema decision.

The authority for the language remains [grimoire.md](../spec/grimoire.md).
The projection boundary remains [projection-language.md](./projection-language.md),
and the schema boundary remains [schema-format.md](./schema-format.md).
The V-JEPA 2-specific case is in
[v-jepa-2-case-studies.md](./v-jepa-2-case-studies.md).

Nothing in this document changes the core vocabulary or makes a paper's
terminology normative. Addresses, definition sites, layer names, and proposed
ports are illustrative until a reviewed grammar and a concrete fixture settle
their syntax.

## Why These Cases

The papers do not differ only in hyperparameters. They expose recurring
structural choices that a description language must keep distinct:

- one representation core with many local consumers;
- a vision encoder bridged into an independent language core;
- one multimodal transformer with discrete modality streams;
- decoupled modality frontends feeding one transformer;
- continuous latent generation and diffusion or flow objectives;
- compact discrete tokenization with a non-spatial latent sequence;
- streaming speech input and output around a language core;
- latent predictive dynamics and action-conditioned world models;
- low-bit operators and precision variants of a transformer; and
- parameter-space branches, deltas, and merges.

Precision, resolution, dataset composition, optimizer settings, benchmark
scores, and deployment measurements recur across the papers, but they do not
all change the structural graph. The case studies use them as decorations or
measurement values unless the paper makes the distinction graph-bearing.

## Evidence Set

The cases below draw from the indexed full-text papers. The links are the
primary arXiv records; the local Scry corpus contains the full-text origins
used for the empirical sweep.

### Representation learning

- [I-JEPA](https://arxiv.org/abs/2301.08243)
- [DINOv2](https://arxiv.org/abs/2304.07193)
- [SigLIP](https://arxiv.org/abs/2303.15343)
- [SigLIP 2](https://arxiv.org/abs/2502.14786)

### Multimodal language systems

- [Janus](https://arxiv.org/abs/2410.13848)
- [Janus-Pro](https://arxiv.org/abs/2501.17811)
- [Chameleon](https://arxiv.org/abs/2405.09818)
- [Emu3](https://arxiv.org/abs/2409.18869)
- [Show-o](https://arxiv.org/abs/2408.12528)
- [Transfusion](https://arxiv.org/abs/2408.11039)
- [Qwen2-VL](https://arxiv.org/abs/2409.12191)
- [Pixtral 12B](https://arxiv.org/abs/2410.07073)
- [InternVL 2.5](https://arxiv.org/abs/2412.05271)
- [MiniCPM-V](https://arxiv.org/abs/2408.01800)

### Generation and tokenization

- [Scaling Rectified Flow Transformers](https://arxiv.org/abs/2403.03206)
- [Movie Gen](https://arxiv.org/abs/2410.13720)
- [Scalable Diffusion Models with Transformers](https://arxiv.org/abs/2212.09748)
- [Flow Matching](https://arxiv.org/abs/2210.02747)
- [TiTok](https://arxiv.org/abs/2406.07550)

### Speech, world models, and adaptation

- [LLaMA-Omni](https://arxiv.org/abs/2409.06666)
- [Mini-Omni](https://arxiv.org/abs/2408.15585)
- [MC-JEPA](https://arxiv.org/abs/2310.12196)
- [Genie](https://arxiv.org/abs/2402.15391)
- [DreamerV3](https://arxiv.org/abs/2301.04104)
- [Continual Pre-Training](https://arxiv.org/abs/2308.04014)
- [DARE](https://arxiv.org/abs/2311.03099)
- [TIES-Merging](https://arxiv.org/abs/2306.01708)

### Low-bit variants

- [BitNet](https://arxiv.org/abs/2310.11453)
- [BitNet b1.58](https://arxiv.org/abs/2402.17764)
- [BitNet a4.8](https://arxiv.org/abs/2411.04965)

## Pattern Matrix

| Case | Representative papers | Structural question |
| --- | --- | --- |
| Shared representation core | I-JEPA, DINOv2, SigLIP, SigLIP 2 | Can one addressed encoder feed probes, decoders, and language consumers without duplicating identity? |
| Encoder-to-language bridge | Qwen2-VL, Pixtral, InternVL 2.5, MiniCPM-V | Can a local projector or compressor adapt one representation to an independent language core? |
| Unified discrete multimodal core | Chameleon, Emu3, Show-o | Can modality token streams and objective-specific attention remain visible inside one shared transformer? |
| Decoupled frontends, unified backbone | Janus, Janus-Pro | Can separate semantic and generative visual paths coexist without being folded into one representation? |
| Continuous latent generation | Transfusion, DiT, MM-DiT, Movie Gen, Flow Matching | Can a latent generator, conditioning path, and continuous objective be represented without pretending to be a runtime evaluator? |
| Compact one-dimensional tokenizer | TiTok | Can a latent token sequence be represented without imposing a false two-dimensional patch topology? |
| Streaming speech system | LLaMA-Omni, Mini-Omni | Can concurrent speech input, language reasoning, and speech output be represented without hiding stream semantics? |
| Latent predictive dynamics | MC-JEPA, Genie, DreamerV3, V-JEPA 2 | Can observation encoders, latent state, dynamics, action, and planning consumers be separated by site? |
| Low-bit operator variant | BitNet, BitNet b1.58, BitNet a4.8 | Which quantization facts are decorations, and when does replacing a linear operator change the structural interface? |
| Parameter-space adaptation | Continual Pre-Training, DARE, TIES-Merging | Can checkpoint lineage and parameter deltas be represented without misrepresenting them as activation connections? |

## Case 1: Shared Representation Core

### Observed facts

I-JEPA, DINOv2, SigLIP, and SigLIP 2 all provide evidence for treating a
learned visual representation as a reusable object rather than as a head tied
to one benchmark.

I-JEPA uses a context encoder, target encoder, predictor, and masked target
representations, then supports frozen representation evaluation. DINOv2 is
presented as a general visual representation with downstream linear, dense, and
other task heads. SigLIP and SigLIP 2 use visual encoders that can be consumed by
language-image systems and evaluate transfer or frozen representations across
tasks. SigLIP 2 additionally combines captioning, self-distillation, masked
prediction, and dense-feature objectives.

The recurring architectural fact is not that all four papers have the same
model. It is that the representation boundary is reusable and downstream heads
are separable from it.

### Illustrative structural account

A description for one of these systems can place an addressed visual encoder in
the core graph:

- a visual input port;
- a sequence or map of visual representation ports; and
- optional intermediate representation ports if a downstream consumer needs
  multilayer features.

A local understanding layer can select a probe. A dense-prediction layer can
select a task head. A language layer can select a projector and language model.
Each layer references the same encoder address, and each layer owns its own
consumer structure.

The encoder need not claim in the core that its representation is a motion
feature, an object feature, or a language-aligned feature. Those claims are
layer-local measurements or provenance statements.

### Candidate decorations

Architecture and representation schemas may attach:

- patch or token shapes;
- model family and scale;
- position encoding and patchification details;
- objective names and training data origins;
- frozen or trainable status for a particular downstream use; and
- benchmark measurements with their evaluation protocol.

The same encoder can therefore appear in several reprojections while the
attached values differ by consumer or evaluation setup.

### What this exercises

- Shared address folding.
- Local probes and decoders.
- Definition-site locality for a representation reused by several views.
- The difference between a representation contract and a benchmark result.
- The existing V-JEPA 2 gap around target, teacher, EMA, and freeze relations.

### Cut

An encoder-only cut contains the core graph and no downstream layer. A probe
cut contains the core graph and one task layer. A language-consumer cut contains
the core graph and its projector and language model, but not unrelated probes.

## Case 2: Encoder-to-Language Bridge

### Observed facts

Qwen2-VL, Pixtral 12B, InternVL 2.5, and MiniCPM-V use the broad pattern of a
visual encoder connected to a language model through an adaptation boundary.
The papers differ in the details:

- Qwen2-VL integrates a vision transformer and Qwen language model and supports
  dynamic visual resolution with multimodal positional treatment.
- Pixtral uses a native vision encoder and a bridge into a multimodal language
  model, with images handled at arbitrary aspect ratios and resolutions.
- InternVL 2.5 describes a ViT-MLP-LLM structure, dynamic high-resolution tiling,
  and staged training of visual and language components.
- MiniCPM-V uses a visual encoder, a compression layer with cross-attention, and
  an LLM; its adaptive visual encoding preserves high-resolution and aspect-ratio
  information while controlling visual token count.

These systems are not the same architecture, but their bridge is graph-bearing:
visual tokens are transformed before entering a separately structured language
core.

### Illustrative structural account

A bridge-based description can place the visual encoder in a shared core and
define the following in a language layer:

- a visual token or feature input;
- a projector, MLP bridge, or cross-attention compressor;
- a language-model input port;
- a language model block; and
- a text output port.

The bridge is an ordinary addressed block with input and output ports. Its input
shape and output shape are schema-governed values, not an informal statement
that the two spaces are compatible.

Dynamic resolution requires care. If tiling changes the number or arrangement of
visual tokens, the tile selection and token arrangement affect structural
connectivity. They cannot be treated only as a scalar resolution decoration.
The exact indexed form is deferred to grammar construction.

### Candidate layer choices

The same base encoder may feed several bridge layers:

- a Qwen2-VL-style bridge;
- a Pixtral-style bridge;
- an InternVL-style bridge; or
- a MiniCPM-style compressor.

Those alternatives should not be silently folded into one definition at one
address. A single worked description may place alternatives at a site visible to
all modes and let a mode layer select one. Independent paper descriptions should
instead keep their address spaces separate.

### What this exercises

- Shape references between separate blocks.
- Local projectors and compressors.
- Dynamic token counts and aspect-ratio-dependent structure.
- Frozen versus fine-tuned encoder variants as distinct layer decorations.
- Minimal cuts that include a language consumer but not unrelated training
  machinery.

### Cut

The visual cut contains the encoder. The multimodal cut contains the encoder,
bridge, language model, and their declared values. A downstream QA evaluation
layer can consume the multimodal reprojection without redefining the visual
encoder.

## Case 3: Unified Discrete Multimodal Core

### Observed facts

Chameleon, Emu3, and Show-o put multiple modalities into one transformer-centric
system, but their objective paths are not interchangeable.

Chameleon maps image and text into a shared discrete token space and supports
interleaved mixed-modal sequences. Emu3 tokenizes text, images, and video and
trains a decoder-only transformer with next-token prediction. Show-o uses one
transformer for multimodal understanding and combines autoregressive language
modeling with discrete diffusion for visual generation, including mixed-modal
editing tasks.

The important structural fact is that the modality streams enter a shared core,
not merely a projector feeding an otherwise independent language model. Show-o
adds a second fact: its attention regime changes by modality and objective.

### Illustrative structural account

The core graph can contain:

- one or more modality tokenizers;
- modality-specific token streams;
- a shared transformer;
- text and visual prediction heads; and
- modality-specific input and output ports.

A mode or objective layer selects the relevant input and output paths. For
example, a language mode selects causal text prediction, while a visual
synthesis mode selects the visual prediction head and its visual token path.

The causal or bidirectional visibility pattern is structural. In Show-o, an
omni-attention regime that lets visual tokens attend differently from text
cannot be attached as a decoration that later changes the graph. It must be
represented by selected or generated connections, or the grammar must provide a
concrete indexed form that expands to those connections.

### Discrete identity

The tokenizers and the shared transformer do not become one element merely
because they operate in one model. A tokenizer's output token address, the
transformer's input token address, and the prediction-head addresses remain
separate. Folding joins shared references by address; it does not erase a
modality boundary.

### What this exercises

- Multiple modality streams in one core.
- Mode-specific selection of heads and paths.
- Generated token-level connections.
- Causal and bidirectional attention as structure.
- Alternatives that share a transformer but differ in finalization values or
  objective paths.

### Cut

A text-only cut may contain the shared transformer and text tokenizer. A
multimodal generation cut includes the visual tokenizer, shared transformer,
visual head, and the selected attention structure. A cut that omits the visual
path must not silently retain a visual output that depends on it.

## Case 4: Decoupled Frontends with a Unified Backbone

### Observed facts

Janus and Janus-Pro sit between the bridge-based and early-fusion families.
They use separate visual encoding paths for understanding and generation while
retaining a shared autoregressive transformer backbone. In Janus, the
understanding path uses a semantic visual encoder and adaptor, while the
generation path uses a discrete visual tokenizer and generation adaptor. The
paths meet at the transformer, but they do not share the same visual
representation before that point.

This is not the same as Chameleon or Emu3, where a common tokenized modality
space is the central input representation. It is also not the same as a pure
bridge model where one visual path is the language model's only visual input.

### Illustrative structural account

The core graph can contain:

- an understanding encoder and its adaptor;
- a generation tokenizer and its adaptor;
- a shared autoregressive transformer;
- a text prediction head; and
- a visual generation head.

The understanding and generation paths carry distinct addresses and distinct
shape values until the shared transformer input boundary. A mode layer selects
one path or a permitted combination for a described task.

The two visual paths must not be folded merely because both consume images or
both reach the same transformer. Shared input modality is not shared identity.
The shared transformer is the address that folds across the modes.

### What this exercises

- Distinct representations for one raw modality.
- Shared backbone with local frontends and heads.
- Mode selection over alternatives.
- A concrete counterexample to treating all unified multimodal models as one
  architecture family.
- Locality for a generation-only tokenizer that understanding layers cannot see.

### Cut

An understanding cut contains the understanding encoder, adaptor, and shared
backbone. A generation cut contains the generation tokenizer, adaptor, shared
backbone, and generation head. A composite cut contains both paths and their
shared transformer.

## Case 5: Continuous Latent Generation and Flow Objectives

### Observed facts

Transfusion, DiT, the rectified-flow transformer work, Movie Gen, and Flow
Matching expose a family in which continuous latent states and vector-field or
diffusion-style objectives are central.

- DiT replaces a conventional U-Net with a transformer operating on latent
  visual patches and studies scaling through transformer compute.
- Flow Matching defines a simulation-free objective for learning continuous
  vector fields along conditional probability paths; it is an objective
  framework rather than one fixed network topology.
- The rectified-flow transformer work uses transformer blocks and rectified flow
  for high-resolution image synthesis, with modality-specific conditioning and
  positional treatment.
- Transfusion combines causal next-token prediction for discrete text with
  diffusion or flow-style prediction for continuous image patches in one
  multimodal transformer.
- Movie Gen uses a large transformer over joint spatiotemporal latents and flow
  matching for video and related media generation.

The recurring boundary is the latent representation and its denoising or flow
conditioning path. The objective may change without changing every structural
block, while a shared transformer may serve discrete and continuous modalities
through different heads or input paths.

### Illustrative structural account

A continuous-latent description can define:

- an encoder or autoencoder into a latent representation;
- a latent patch or spatiotemporal-token sequence;
- conditioning inputs such as text, time, noise, or modality labels;
- a transformer or denoiser;
- a continuous prediction head or vector-field output; and
- a decoder back to the observable modality.

The latent state, conditioning signal, and prediction target have separate
ports. A time or noise input that changes the model's predicted field is part of
structural input, even when the schedule itself is a decorated value.

For Transfusion, the discrete text prediction path and continuous image path
share the transformer but retain distinct output heads and objective values. The
fact that both are trained in one model does not make their target spaces
identical.

### Objective boundary

The Flow Matching paper is evidence that a training objective can be a reusable
layer over several architectures. The description may attach the path family,
noise schedule, solver-independent objective, and evaluation measurements as
values. It must not imply that a static projection executes an ODE solver or a
sampling run.

### What this exercises

- Continuous latent shapes.
- Decoder and autoencoder boundaries.
- Shared transformer with discrete and continuous objective paths.
- Conditioning and timestep inputs.
- Separation of a static generator graph from sampling runtime.
- Cost and resolution values attached after structural evaluation.

### Cut

A latent-generator cut contains the latent representation, conditioning path,
transformer, and prediction head. A reconstruction cut additionally contains
the decoder. A training-objective layer can consume the latent generator without
becoming the definition site of the generator's blocks.

## Case 6: Compact One-Dimensional Visual Tokenization

### Observed facts

TiTok studies image tokenization as a one-dimensional latent sequence, with
variants using as few as 32 tokens. The work evaluates reconstruction, linear
probing over frozen features, and image generation with a generator such as
MaskGIT.

The important architectural fact is not only token count. A 1D latent sequence
does not preserve the fixed two-dimensional correspondence between latent
positions and image patches that a conventional 2D visual tokenizer supplies.
That changes what a consumer may infer from token position.

### Illustrative structural account

A TiTok-style description can define:

- an image encoder;
- a one-dimensional latent token sequence;
- a decoder for reconstruction; and
- a separate generator consuming the latent sequence.

The latent sequence has a sequence axis and a token-feature axis. It should not
be assigned height and width axes unless a separate schema-governed value says
that a particular consumer has reconstructed such a topology.

A generation layer can reference the tokenizer and define a MaskGIT-like
consumer. A probing layer can reference frozen encoder features. The generator,
decoder, and probe are distinct local consumers of the tokenizer boundary.

### What this exercises

- Shape schemas without an assumed spatial topology.
- Compact latent representations.
- Tokenizer and generator as separate elements.
- A useful distinction between token count and token geometry.
- Erasure of a generator while retaining an encoder or reconstruction view.

### Cut

A reconstruction cut contains the image encoder, latent sequence, and decoder.
A generation cut contains the tokenizer and generator. A probe cut need contain
only the encoder output and probe, not the reconstruction decoder.

## Case 7: Streaming Speech Around a Language Core

### Observed facts

LLaMA-Omni and Mini-Omni use language-model-centered systems for real-time speech
interaction. LLaMA-Omni combines speech input, an LLM backbone, and a streaming
speech decoder so text and speech responses can be produced with low latency.
Mini-Omni combines speech understanding and speech-token generation with a
language model, including text-instructed speech token generation and
batch-parallel reasoning for real-time interaction.

The shared pattern is a speech input path, a language reasoning path, and a
speech output path. The output is not merely a final text string later sent to a
separate service; speech generation is part of the described model boundary.

### Illustrative structural account

A speech description can define:

- an audio or speech encoder;
- an optional text token path;
- a shared language reasoning block;
- a speech-token or acoustic decoder; and
- text and speech output ports.

The speech and text outputs may share the reasoning block but have distinct
prediction heads and representation schemas. Chunking, frame rate, receptive
window, and output latency attach as values to the relevant ports or stages.

A streaming layer can select the causal sequence structure used for incremental
input and output. It must not pretend that a static projection evaluates wall
clock timing, buffering, or an audio device. Those are deployment facts or an
external execution boundary.

### What this exercises

- Multiple output modalities from one reasoning core.
- Sequence chunking and causal visibility.
- Speech-token versus text-token shape references.
- Deployment measurements such as latency as measurement values.
- The boundary between static streaming structure and runtime scheduling.

### Cut

A text-only cut can omit the speech decoder. A speech-interaction cut includes
both speech paths and the shared language core. A decoder-only cut is not a
self-contained description unless its input representation and producer are
included in its input chain.

## Case 8: Latent Predictive Dynamics and World Models

### Observed facts

MC-JEPA, Genie, and DreamerV3 provide distinct examples of predictive latent
systems, alongside the V-JEPA 2 case.

- MC-JEPA separates content and motion dynamics in a joint-embedding predictive
  architecture by masking distinct temporal and spatial components.
- Genie combines a spatiotemporal video tokenizer, an autoregressive dynamics
  model, and a discrete latent action model to create interactive environments.
- DreamerV3 learns a recurrent state-space world model and optimizes actor and
  critic behavior through latent imagination across diverse domains.
- V-JEPA 2 adds a frozen observation encoder and a separately post-trained
  action-conditioned predictor for robot planning.

These papers show that "latent prediction" is not one topology. An action-free
predictor, a latent action model, a recurrent state-space model, and a frozen
encoder plus action-conditioned predictor have different structural interfaces.

### Illustrative structural account

A world-model description can separate:

- observation encoder;
- latent state or representation;
- dynamics predictor;
- action input, if present;
- reward, continuation, or task signal, if present;
- policy or planner consumer; and
- optional decoder used only for visualization or reconstruction.

The action input is structural when it conditions predicted future state. A
reward or task-success signal is structural only when the described model
actually consumes it. It must not be inferred from a paper's evaluation or
training metadata.

A planning layer can reference a dynamics reprojection and define a static goal
energy, candidate action representation, or planner structure. Closed-loop
execution, search iterations, and device timing remain outside pure static
projection evaluation.

### What this exercises

- Recurrent and autoregressive latent state paths.
- Action-free versus action-conditioned prediction.
- Optional decoder as an interpretability consumer rather than a generator core.
- Planning over imagined representations.
- Runtime loop boundaries and horizon-dependent measurements.
- Shared encoder cuts versus dynamics-inclusive cuts.

### Cut

An observation-representation cut contains only the encoder. A passive dynamics
cut contains the encoder and predictor. An action-conditioned planning cut also
contains action or proprioceptive inputs and the planner layer. A reward-free
world-model cut must not gain a reward port merely because another world-model
paper uses one.

## Case 9: Low-Bit Operator Variants

### Observed facts

BitNet, BitNet b1.58, and BitNet a4.8 retain the broad transformer arrangement
while changing the numerical representation and selected operators.

- BitNet replaces ordinary linear projections with low-precision operators and
  uses quantized weights and activations while retaining higher-precision
  optimizer state and gradients during training.
- BitNet b1.58 constrains every weight to the ternary set `{-1, 0, 1}`.
- BitNet a4.8 uses 1-bit or ternary weights with 4-bit activations and additional
  sparsification and 8-bit treatment for intermediate states.

The papers therefore test a boundary between architecture and extension values.
A low-bit operator can be a replacement for an ordinary linear operator, not
just a fact about a checkpoint file.

### Illustrative structural account

A conservative first account keeps the transformer graph explicit and attaches
schema-governed values to affected operator elements:

- weight representation;
- activation representation;
- accumulation representation;
- sparsity behavior;
- optimizer-state precision; and
- inference cost measurements.

If replacing `linear` with `bitlinear` changes the element's required ports or
its structural contract, the operator kind must be visible in the grammar or as
an ordinary element definition. If the ports remain identical and only the
numeric domain changes, an extension parameter may be enough.

This case should be allowed to expose that question. It must not silently choose
one side merely because the resulting graph is easier to write.

### What this exercises

- Architecture values that affect compute and memory without changing graph
  connectivity.
- Operator identity versus numeric decoration.
- Cost and bandwidth projections over precision values.
- Alternatives at one visible site.
- Preservation of unrecognized quantization namespaces.

### Cut

A logical transformer cut can omit deployment precision details. A deployed
low-bit cut includes the operator values and cost measurements. A variant cut
must preserve the relation to the common logical architecture without
pretending that two incompatible operator definitions are one element.

## Case 10: Parameter-Space Adaptation and Merge

### Observed facts

Continual pre-training, DARE, and TIES-Merging exercise a different kind of
composition. Their central objects are parameter states and parameter deltas,
not activation paths.

Continual pre-training studies how learning-rate re-warming and schedule shape
control adaptation to streaming data while reducing forgetting. DARE sparsifies
fine-tuning deltas from homologous models and rescales the remainder before
merging. TIES-Merging trims redundant parameter changes, resolves sign
conflicts, and merges aligned task vectors.

These operations may preserve the same logical model architecture while
changing parameter state. Treating a merge as an activation **connection** would
misstate what the paper does. Treating every checkpoint as a competing element
definition at one address may also collide with Grimoire's one-definition rule.

### Illustrative structural account

The first case-study account keeps the logical architecture and parameter-state
lineage separate:

- a base model architecture;
- one or more adapted parameter states;
- parameter delta artifacts;
- a merge or rescaling operation; and
- a resulting parameter state associated with the same logical architecture.

The model architecture may be represented in the core graph. The adaptation
history may be an external artifact or a layer-local structure only if the
grammar gives it an explicit non-activation relation. Delta sparsity, sign
agreement, rescaling, data stream, and evaluation results are finalized values
attached to that relation.

The proposal does not introduce a **checkpoint**, **parameter state**, or
**merge** vocabulary term. This case records the pressure for one and waits for
an explicit scope decision.

### What this exercises

- A composition that is not directed activation flow.
- Identity shared across parameter variants.
- Provenance of derived model states.
- Merge conflicts and visible errors.
- The limit of the current block/port/connection vocabulary.

### Cut

A logical-architecture cut should not require every parameter lineage artifact.
A reproducibility cut for a merged model must include the base state, all
selected deltas, the merge relation, and the resulting state, or report the
missing input as unresolvable.

## Cross-Case Address and Site Inventory

The following names are illustrative. They show where the recurring elements
might live if one worked description intentionally compared several architecture
families. They do not assert that unrelated papers should be merged into one
address space.

| Illustrative address | Candidate site | Reason |
| --- | --- | --- |
| `system/vision-encoder` | Core graph | Several layers consume the same representation in one described system. |
| `system/latent-representation` | Core graph or tokenizer layer | The representation is shared by generator, probe, or dynamics consumers. |
| `system/modality-tokenizer` | Core graph or local modality layer | It is core only when every selected view needs the same tokenizer. |
| `system/shared-transformer` | Core graph | Multiple modality or objective paths fold at the shared backbone. |
| `system/bridge` | Language layer | The projector or compressor serves one language consumer. |
| `system/dynamics-predictor` | World-model layer | Future-state prediction belongs above the observation representation. |
| `system/planner` | Planning layer | Goal scoring and candidate action structure belong to the planning viewport. |
| `system/speech-decoder` | Speech layer | Speech output is local to speech interaction unless all views consume it. |
| `system/quantized-operator` | Variant or deployment layer | Precision may be a value or a distinct operator definition; the case must decide visibly. |
| `system/parameter-lineage` | External artifact or future layer | Parameter-state composition is not ordinary activation structure. |

The locality rule remains the controlling test: an element is defined at the
lowest site all of its references can see. A paper comparison does not by
itself create a shared element. Shared addresses are justified only within one
description whose layers actually reuse that element.

## Cross-Case Tests

A later worked description or fixture should test the following without adding
new requirements to the core specification:

- A shared encoder folds into a probe layer, a bridge layer, and a dynamics
  layer without duplicate definitions.
- A bridge-based language model and an early-fusion language model remain
  distinguishable structural accounts.
- Janus-style independent frontends remain separate until the shared backbone.
- Discrete token streams and continuous latent streams do not collapse into one
  value shape.
- A one-dimensional tokenizer does not acquire a false spatial topology.
- A shared transformer can expose different objective paths without making the
  paths identical.
- Attention visibility, recurrent state, and action conditioning remain
  structural when they affect which inputs a prediction can use.
- Speech output and visual output can share a reasoning core while retaining
  distinct output contracts.
- Low-bit precision can be attached as values until an operator interface makes
  the distinction structural; the result must be visible either way.
- Parameter deltas and merges do not get represented as ordinary activation
  connections by accident.
- Every minimal cut contains the producer of each selected input and rejects a
  missing declared input visibly.
- Decorations, benchmark measurements, and deployment values never feed back
  into structural selection.

The cases intentionally do not force an example of `invert`. They also do not
claim that Grimoire evaluates token generation, diffusion sampling, recurrent
rollouts, speech streaming, robot execution, or parameter optimization at
runtime.

## Sized Gaps

### Indexed modality streams and attention visibility

- Binds when: a fixture expands a multimodal token sequence or an attention
  pattern beyond an architecture-level summary.
- Cost of absence now: Chameleon, Show-o, Janus, Transfusion, and speech cases
  cannot state their modality-specific visibility and prediction paths as
  checkable structure.
- Candidate shapes and rough size: generated ordinary connections indexed by
  token and modality; or a grammar-defined indexed form that expands before
  validation. This is a grammar and projection surface plus fixture families.
- Entry trigger: the first concrete unified-multimodal fixture with two
  modalities and two objective paths.

### Continuous and discrete representation contracts

- Binds when: a description combines Transfusion-like continuous and discrete
  paths or compares TiTok with a spatial tokenizer.
- Cost of absence now: shape schemas can name dimensions but cannot state which
  consumers may interpret token identity, spatial correspondence, or continuous
  fields.
- Candidate shapes and rough size: extend shape values with explicit topology
  and value-domain distinctions; or keep those distinctions as schema-governed
  extension values. This is schema work, not a new core graph primitive.
- Entry trigger: the first generator fixture with both a discrete text path and
  a continuous visual path.

### Streaming and recurrent execution boundaries

- Binds when: speech streaming or latent-dynamics cases are included in the
  worked description.
- Cost of absence now: static sequence structure can be described, but buffering,
  recurrent state, horizon, and closed-loop execution remain easy to imply
  without a checkable boundary.
- Candidate shapes and rough size: static causal and recurrent graph plus an
  explicit external-execution artifact; no runtime semantics in projections.
  This is a scope clarification and acceptance fixture.
- Entry trigger: the first fixture that distinguishes a static predictor graph
  from a runtime rollout.

### Parameter-state lineage

- Binds when: DARE, TIES-Merging, or continual adaptation is represented as
  more than provenance prose.
- Cost of absence now: parameter composition is either omitted or falsely
  rendered as activation flow; reproducibility of a merged state cannot be
  checked.
- Candidate shapes and rough size: a separate artifact relation for base state,
  delta, merge, and result; or an explicit future vocabulary extension. This
  requires a scope decision before grammar work.
- Entry trigger: a reference description needs to represent a merged or
  continually adapted checkpoint.

### Operator identity and quantization

- Binds when: a low-bit case needs to distinguish BitLinear from ordinary Linear
  for structural or cost reasons.
- Cost of absence now: the same graph can carry low-bit measurements, but the
  description cannot say whether the operator contract itself changed.
- Candidate shapes and rough size: schema-governed precision values; or distinct
  operator block kinds with a shared logical interface. This is one small case
  fixture and a schema decision.
- Entry trigger: a bandwidth or cost projection references the operator rather
  than only its attached precision.

### Training-update relations

- Binds when: the shared-representation, JEPA, low-bit, or continual-training
  cases need to distinguish activation flow from parameter updates.
- Cost of absence now: EMA, freeze, fine-tune, re-warming, and delta application
  remain ambiguous even though they affect the represented training system.
- Candidate shapes and rough size: a layer-local parameter relation over block
  ports; or schema-governed values attached to parameter-state artifacts. This
  is the same pressure identified by the V-JEPA 2 case study.
- Entry trigger: the first end-to-end fixture that validates both an activation
  connection and a parameter-update relation.

## Coverage Map

The proposed cases consume the indexed papers as follows:

- Shared representation core: I-JEPA, DINOv2, SigLIP, SigLIP 2.
- Encoder-to-language bridge: Qwen2-VL, Pixtral 12B, InternVL 2.5,
  MiniCPM-V.
- Unified discrete multimodal core: Chameleon, Emu3, Show-o.
- Decoupled frontends with unified backbone: Janus, Janus-Pro.
- Continuous latent generation: Transfusion, Scaling Rectified Flow
  Transformers, Movie Gen, DiT, Flow Matching.
- Compact one-dimensional tokenization: TiTok.
- Streaming speech: LLaMA-Omni, Mini-Omni.
- Latent predictive dynamics: MC-JEPA, Genie, DreamerV3, with V-JEPA 2 in
  the sibling case-study document.
- Low-bit operator variants: BitNet, BitNet b1.58, BitNet a4.8.
- Parameter-space adaptation: Continual Pre-Training, DARE, TIES-Merging.

This map is a coverage statement, not a ranking of the papers or a claim that
every paper in one row has identical architecture.

## Executable Consumer Checkpoint

The current Rust fixture in
[`consumer_targets.rs`](../crates/grimoire/tests/consumer_targets.rs) covers
the cross-paper consumer boundary with independent bridge, unified-token,
decoupled-frontend, continuous-latent, one-dimensional-tokenizer, streaming
speech, latent-dynamics, low-bit, and parameter-lineage layers. It validates
their cuts, checks that only justified shared backbone addresses fold, and
keeps precision and lineage facts out of activation structure.

The fixture is intentionally a coverage checkpoint rather than a claim that
the current grammar evaluates token generation, diffusion, streaming clocks,
recurrent rollouts, or parameter merges. Those remain static structure plus
finalized values or explicit external/deferred boundaries.

## Next Consumption Step

The next artifact should be one complete reference description that composes
the shared core, domain layers, pretraining and AC checkpoint, downstream
consumer checkpoint, placement, cost, provenance, and cut/serializer
validation. It should retain the paper-family distinctions already exercised
here instead of collapsing them into one universal multimodal architecture.
