# Architecture vocabulary and decisions

Status: prototype contract; in progress.

This ledger records how the architecture terms found in the indexed papers map
to the existing Grimoire vocabulary. It is deliberately conservative: a paper
term becomes a new Grimoire vocabulary term only when the existing primitives
cannot represent its identity, scope, or structural relation.

The core specification remains frozen in [grimoire.md](../spec/grimoire.md).
This document does not change that vocabulary. It records proposed mappings and
identifies decisions that must be made before grammar or validator work can
claim complete coverage.

## Existing Vocabulary Is Enough for Most Architecture

| Paper concept | Grimoire account | Reason |
| --- | --- | --- |
| Encoder, transformer, language model, predictor, tokenizer, decoder, projector, probe, planner, controller | `block` | Each is a named computational element with ports and connections. The name and architecture family are values. |
| Tensor, token stream, latent state, action, end-effector state, image goal, logits | Port value described by `shapes/1` and other schemas | These are values crossing structural boundaries, not independently addressable structure unless a case needs them to be selected or referenced. |
| Input, output, target, mask token, query token | `port` or a block-local element | The choice depends on whether the item is a stable interface or only an internal generated element. |
| Data flow, attention visibility, modality path, encoder-to-projector path | Directed `connection` | The relation determines what a downstream block can receive. It cannot be only a decoration. |
| Transformer stage, tokenizer family, probe, language alignment, planning view | `layer` | These are human viewports or consumer-specific structures over shared inputs. |
| Model family, parameter count, width, depth, heads, activation, position encoding | `architecture/1` value | They describe a block without changing its identity by themselves. |
| Objective, optimizer, schedule, frozen target, trainable target, data mixture | `training/1` value | They describe a training stage and must not make a static projection run training. |
| Static, streaming, recurrent, closed-loop | `execution/1` value | The value marks the boundary without introducing runtime evaluation. |
| Weight, activation, accumulation, optimizer-state precision, sparsity | `precision/1` value | These describe numeric realization and cost; they become structural only if the operator interface differs. |
| Benchmark score, latency, memory, bandwidth, success rate | `measurement/1` value | External observations require a unit and source record. |
| Citation, assumption, novelty state | `provenance/1` value | These are finalized facts attached to groups or other permitted elements. |
| Variant, checkpoint, delta, merge | `lineage/1` candidate or external artifact | Parameter-state composition is not activation flow and is not yet represented by a settled core relation. |

The same term may occupy different accounts in different descriptions. For
example, a mask token can be a layer-local block or generated port when the
case only needs its position, but it can be an ordinary addressed element when
a downstream layer references the learned pretraining predictor's masking
interface. The description must make that choice visible; the validator must not
infer it from a name.

## Prototype Decisions

The first executable prototype makes three deliberately reversible choices:

- Rust 2024 is the implementation language, using Cargo and the installed Rust
	1.94 toolchain.
- Indexed modality, time, and block-causal visibility expands into ordinary
	addressed elements and directed connections before validation. No compact
	indexed grammar is introduced yet.
- Parameter states, deltas, checkpoints, and merges remain external lineage
	artifacts or finalized values. They are not activation connections and do not
	become new frozen element kinds.

Each choice is paired with a fixture. A fixture may replace the choice when it
shows that the current account cannot represent an empirical case without
losing identity, locality, cut erasure, or the structural/finalization
boundary.

## Architecture Decisions

### A shared address means actual reuse

A visual encoder gets one address in the core graph only when multiple layers
in the same description consume that representation. A paper comparison does
not create shared identity. If two papers use unrelated encoders, their blocks
remain distinct even when both are called `vision-encoder`.

This is the controlling rule for the V-JEPA 2, DINOv2, SigLIP, Qwen2-VL, and
bridge-based case studies. The shared encoder folds into consumer reprojections;
its probes, projectors, language models, decoders, and planners remain local.

### A similar name does not imply a shared element

The V-JEPA 2 pretraining predictor and V-JEPA 2-AC predictor have different
inputs, objectives, attention regimes, and parameterizations. They therefore
receive distinct addresses even though both are predictors of visual
representations.

The same rule separates a visual tokenizer from a decoder, a projector from an
LLM, and a language prediction head from a visual generation head. Folding is by
address, never by paper terminology or role label.

### Consumer layers own adaptation

A projector, MLP bridge, perceiver compressor, attentive pooler, classification
probe, action-anticipation probe, or speech decoder is defined in the layer that
needs it. It enters the core graph only if every layer in the description needs
the same addressed element.

This keeps Qwen2-VL, Pixtral, InternVL, MiniCPM-V, and V-JEPA 2 VidQA
consumers comparable without claiming that their bridges are one primitive or
one shared model.

### Shared backbones may have separate frontends

Janus-style understanding and generation frontends remain separate addressed
paths until their shared transformer backbone. A shared raw modality does not
collapse them. A shared backbone may fold across mode layers while the semantic
encoder, generation tokenizer, adaptors, and output heads remain local or
mode-selected.

This distinguishes decoupled-frontends systems from Chameleon and Emu3-style
shared token-space systems. The distinction is structural, not a model-family
label.

### Objective paths remain structural where visibility changes

Causal text prediction, bidirectional visual processing, Show-o-style mixed
attention, block-causal action prediction, and modality-specific output paths
must be represented as selected or generated connections when they change which
inputs a block can see.

An objective name, loss name, or attention-regime decoration can describe the
path, but it cannot retroactively change a folded graph. The projection language
must complete structural selection before decoration.

### Shapes describe topology only at the level actually known

A shape carries ordered dimensions and a coarse layout. It does not infer
semantics from axis names and does not assert spatial correspondence merely
because a value has two dimensions.

A TiTok latent sequence is therefore a sequence, not a grid. A V-JEPA feature
map may be a grid, while a video input may be a volume. A token sequence's
relationship to image coordinates is a separate structural or schema-governed
fact and must not be fabricated from the shape alone.

### Training facts do not become runtime behavior

Warmup, constant-rate, cooldown, teacher forcing, rollout loss, EMA, frozen
encoders, and staged alignment are facts about how a system was trained. They
are attached values or parameter relations, not instructions for a projection
to execute.

Likewise, flow matching, diffusion, next-token prediction, and CEM are static
objective or planner structures. The language describes their inputs, outputs,
and attached facts; it does not bind to a training run, sampling trajectory,
robot controller, audio clock, or optimizer process.

### Parameter flow is not activation flow

An EMA target, frozen encoder, fine-tuned block, continual update, task vector,
delta, or merge describes parameter state or update lineage. It must not be
encoded as an ordinary activation connection simply because both objects are
represented by blocks.

The current inventory carries target and freeze lists as `training/1` values and
holds richer lineage in the `lineage/1` candidate. A future structural relation
requires a fixture that proves validator-level checking is needed.

### Precision is a value until the interface changes

BitNet-style low-bit weights and activations can be attached as `precision/1`
values when the logical input and output contract remains the same. If a
quantized operator has a different required port or connection contract, it is a
distinct block definition at the appropriate site and may still carry precision
values.

The case study must expose which situation holds. It must not choose a new core
operator kind merely to make a cost calculation convenient.

### Runtime boundaries are explicit

A planner, recurrent world model, or streaming speech path can be fully
represented as static structure plus an `execution/1` value that names its
regime and external consumer. Runtime observation, buffering, recurrent state
updates, action execution, and replanning remain outside the projection
language.

A cut containing a runtime-facing layer must still be a well-formed static
description. Missing runtime inputs are an external execution concern, not a
reason to silently invent a run binding.

## Vocabulary That Remains Deferred

The following names are useful in case-study prose but are not yet additions to
the frozen core vocabulary:

- parameter state;
- parameter delta;
- checkpoint;
- merge;
- objective;
- schedule;
- execution regime;
- modality stream;
- latent topology;
- attention mask; and
- model family.

Most of these are already representable as schema-governed values or ordinary
blocks, ports, and groups. `parameter state`, `delta`, and `merge` are the
exception: they name a relation that is not activation flow and therefore remain
an explicit architecture gap rather than an improvised core primitive.

## Architecture Family Decisions

| Family | Decision recorded here | What remains empirical |
| --- | --- | --- |
| Shared representation | Reuse one addressed encoder and define consumers locally. | Whether intermediate-layer outputs need stable addresses in the reference description. |
| Encoder-to-language bridge | Make bridge/projector/compressor an ordinary local block with shape-checked ports. | Whether dynamic tiling needs generated token elements or can remain a value. |
| Unified discrete multimodal | Keep modality tokenizers, shared transformer, heads, and objective paths distinct. | Exact indexed attention grammar. |
| Decoupled frontends | Keep independent frontends separate until a shared backbone address. | Which mode combinations the reference description permits. |
| Continuous latent generation | Keep latent state, conditioning, denoiser/vector field, and decoder distinct. | Symbolic noise/time values and their projection syntax. |
| One-dimensional tokenization | Represent sequence shape without inferred spatial correspondence. | Whether a tokenizer's latent identity needs a dedicated schema value. |
| Streaming speech | Keep speech input and output paths distinct around a shared reasoning block. | Static representation of chunking and concurrent streams. |
| Latent dynamics | Separate observation encoding, dynamics, action/state input, planner, and optional decoder. | Recurrent-state relation and runtime rollout boundary. |
| Low-bit operators | Start with precision decoration; promote operator identity only when interface differs. | Cost projection's need to inspect operator precision. |
| Parameter adaptation | Keep architecture identity and parameter lineage separate. | Whether lineage must become first-class for reproducibility cuts. |

## Decision Record Format

When a new case or fixture changes one of these decisions, record four pieces of
data together:

1. the empirical source and exact architecture fact;
2. the existing Grimoire primitive or schema that would carry it;
3. the observed failure if that account is insufficient; and
4. the smallest candidate change, with the fixture that would discriminate it.

A decision is not complete merely because a representation can be written. It is
complete when the chosen account preserves address identity, layer locality,
cut erasure, and the structural/finalization boundary under validation.
