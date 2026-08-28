# Static execution boundary

Status: prototype boundary checkpoint with open runtime gaps.

This proposal defines the static account for execution regimes found in the
indexed architecture cases. It is deliberately a boundary document: it says
what a Grimoire description can represent and what remains an external
consumer. It does not add runtime semantics to the projection language.

The governing sources are [grimoire.md](../spec/grimoire.md),
[schema-inventory.md](./schema-inventory.md),
[architecture-vocabulary.md](./architecture-vocabulary.md), and
[frontier-architecture-case-studies.md](./frontier-architecture-case-studies.md).

## The Boundary

A Grimoire description represents static structure:

- addressed blocks, ports, connections, and groups;
- layer-local structure and declared input chains;
- the values and constraints attached by schemas; and
- the declared regime in which an external consumer uses the structure.

It does not evaluate:

- a training run or optimizer update;
- a diffusion, flow, or autoregressive sampling trajectory;
- a recurrent state across wall-clock steps;
- an audio buffer or stream scheduler;
- a robot controller or physical action; or
- a model-predictive-control replanning loop.

An execution value marks this distinction. It is not a hidden instruction to
run the model.

## Candidate Contract

Candidate `execution/1` value body:

```text
product{
  regime: enum{static, streaming, recurrent, closed-loop},
  horizon: optional(positive-int),
  rate: optional(finite-number),
  external_consumer: enum{yes, no}
}
```

`regime` is required. `horizon` records a finite context or planning horizon
when the paper or deployment protocol gives one. `rate` records a frame,
chunk, or control rate when its unit is supplied by a separate value or the
containing element's interface. `external_consumer` is required so the
representation cannot ambiguously claim that a description both is and is not
bound to a runtime process.

The candidate does not include a callback, clock, buffer, state transition
program, or action executor. Those would be runtime language features rather
than finalized schema values.

## Static Structure Versus Regime

The same static predictor graph may receive different execution values in
different descriptions or deployment layers. For example:

- an autoregressive transformer can be described as `static` when documenting
  its architecture;
- its token-by-token deployment can be described as `streaming` when the
  external consumer supplies incremental context; and
- a latent dynamics block can be described as `recurrent` when its output is
  fed back across model steps.

The regime does not alter connections. If feedback, causal visibility, or an
explicit state port is part of the model's architecture, those are structural
connections and ports. If the repeated use is only a deployment loop, the
execution value and external-consumer flag record the boundary.

This distinction prevents an execution decoration from changing a projection's
structural result.

## Speech Cases

LLaMA-Omni and Mini-Omni expose speech input and output around a language core.
A static description includes:

- speech or audio input ports;
- any text-token or language-reasoning ports;
- speech-token or acoustic output ports; and
- directed connections between those blocks.

A streaming execution value can record the reported low-latency or chunked
regime. It does not say how an audio device schedules chunks, how output is
buffered, or when a user hears a token. A measurement schema records latency or
throughput with its source and protocol.

If the model's architecture contains a causal attention path over audio chunks,
that path is structural. If the deployed system merely invokes a static model
repeatedly, the repetition belongs to the external consumer.

## Latent Dynamics Cases

Genie and DreamerV3 contain learned latent dynamics used for imagination or
interaction. V-JEPA 2-AC contains an action-conditioned predictor over frozen
visual representations.

A static world-model account includes:

- an observation or image encoder;
- a latent state or representation port;
- a dynamics predictor;
- action and optional reward or continuation ports;
- a predicted next-state port; and
- any decoder or planner that the description intentionally includes.

A recurrent value may mark that predicted state is supplied again at a later
model step. It does not itself create a feedback connection. A feedback
connection is required only when the described model interface explicitly
contains that state path.

A reward port is included only when the model consumes reward or task signal.
Dreamer-style actor-critic imagination and V-JEPA 2 reward-free prediction must
not be conflated because both are called world models.

## Closed-Loop Planning

V-JEPA 2-AC planning provides the clearest boundary case. The static structure
contains:

- current visual and proprioceptive inputs;
- a goal representation;
- a world-model rollout block;
- an energy or distance value;
- an action candidate representation; and
- an action output toward an external controller.

A `closed-loop` execution value records that the external consumer observes,
executes the selected action, and presents a new state before replanning. It
does not evaluate CEM, choose an action, or access a camera. The planner's
candidate structure and goal energy are static elements or finalized values;
the runtime loop is outside the projection.

If a selected layer requires a runtime-produced input that is absent from a cut,
the cut is a static description whose layer is unresolvable. It is not silently
converted into an empty or open-loop result.

## Static Generation and Sampling

Diffusion, flow matching, and autoregressive generation need the same boundary.
A description may represent:

- a noise or timestep input port;
- a conditioning port;
- a denoiser, vector-field, or prediction block;
- a latent representation; and
- a decoder or output head.

A `static` execution value can say that this is the architecture being
specified. It does not bind the projection to a solver, random seed, number of
sampling steps, or generated sample. Those are training or deployment values,
measurements, or external execution artifacts.

The distinction is important for Transfusion and Show-o: their shared
transformer and modality-specific objective paths are structural, while the
actual diffusion or next-token run is not.

## Runtime Inputs and Cuts

A cut contains a layer and every layer in its declared input chain. Runtime
inputs are not special exceptions to that rule.

- A layer-local state port defined in the cut is present.
- A state port defined in an absent layer is an absent declared input.
- A controller outside the description is an external consumer, not an
  implicitly available block.
- A standalone layer that names a missing runtime-facing producer remains
  unresolvable until its surrounding description supplies that producer.

The validator should report the layer and missing declared input. It should not
try to infer a suitable sensor, clock, controller, or prior state.

## Candidate Checks

The execution schema supports visible checks over finalized values, for example:

- a streaming block has a positive declared rate;
- a closed-loop planner has `external_consumer: yes`;
- a static architecture does not claim a runtime-produced input in its own
  layer; and
- a recurrent predictor has a declared state interface when the architecture
  explicitly feeds its output back.

These are schema or layer checks over decorations. They do not cause execution
or change structural selection.

The base schema should not require every static block to carry an execution
value. The absence of an execution decoration is not itself a structural error;
a layer may add one when a deployment or runtime boundary is relevant.

## Fixtures

Valid fixtures:

- a static image encoder with no runtime binding;
- a streaming speech encoder and decoder with a positive rate;
- a recurrent latent dynamics predictor with an explicit state port;
- a closed-loop V-JEPA 2-AC planner with an external controller boundary;
- a diffusion denoiser with a static sampling boundary;
- a language model used in both static and streaming descriptions; and
- a cut containing a runtime-facing layer and all of its static producers.

Invalid or visible-outcome fixtures:

- an unknown execution regime;
- a zero or non-finite rate where a rate is present;
- a closed-loop planner with `external_consumer: no` when its layer requires
  physical action execution;
- a recurrent relation described only by an execution value while the model
  claims a structural feedback port is required;
- a cut whose planner input layer is absent; and
- a projection that tries to invoke runtime state, a clock, or an external
  action inside `select` or `decorate`.

The last three outcomes must distinguish schema invalidity, structural
unresolvability, and out-of-scope runtime evaluation. They must not collapse
into one generic failure.

## Decision Record

This proposal records these decisions:

- Execution regime is a finalized schema value, not a runtime instruction.
- The initial regimes are `static`, `streaming`, `recurrent`, and
  `closed-loop`.
- Horizon and rate are optional values; a rate requires an explicit unit on the
  relevant interface or measurement context.
- `external_consumer` is explicit and required within an execution value.
- Causal, recurrent-state, and action-conditioning paths are structural when
  the model interface contains them; execution values do not create paths.
- Runtime observation, buffering, state progression, action execution, and
  replanning remain outside pure static projection evaluation.
- Missing runtime-facing producers in a cut produce a visible unresolvable
  outcome at the dependent layer.
- A reward or task signal is represented only when the model consumes it.

## Sized Gaps

### State-transition relation

- Prototype status: the recurrent fixture distinguishes an explicit state
  feedback connection from an `execution/1` regime value and keeps runtime
  progression external.
- Remaining gap: the validator does not check a state-transition relation
  beyond the ordinary addressed connection.
- Candidate shape and rough size: an ordinary directed state connection with a
  layer-local relation value; one two-step fixture. Do not add implicit
  recurrence to the execution schema.
- Entry trigger: a recurrent case whose state update must be validated
  independently of runtime execution.

### Streaming chunk identity

- Binds when: a speech or streaming multimodal case relies on chunk overlap,
  order, or concurrent input/output ownership.
- Cost of absence now: causal sequence structure and rate can be recorded, but
  chunk correspondence remains uncheckable.
- Candidate shape and rough size: explicit sequence axes and generated chunk
  connections; one bounded fixture before adding any runtime scheduler.
- Entry trigger: the first streaming fixture that compares input and output
  chunks rather than only attaching latency.

### Runtime artifact packaging

- Binds when: a description must ship a controller, scheduler, solver, or
  external device contract alongside static architecture.
- Cost of absence now: static structures are representable, but a reproducible
  deployment cut cannot include the external process contract.
- Candidate shape and rough size: a separately versioned execution artifact
  referenced by provenance or a future package manifest; no projection runtime.
- Entry trigger: the first end-to-end worked description claiming deployment
  reproducibility.
