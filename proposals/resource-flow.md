# Typed probabilistic resource flow

This proposal defines a machine-agnostic resource analysis over an evaluated
Grimoire structure. It extends the explicit-input cost and placement boundaries
without turning resource analysis into a second graph language or a runtime
simulator.

The governing sources are the [core specification](../spec/grimoire.md), the
[cost boundary](./cost-layer.md), the [placement and bandwidth boundary](./placement-bandwidth.md),
the [measurement schema](./measurement-schema.md), and the architecture
case studies in [V-JEPA 2](./v-jepa-2-case-studies.md) and
[frontier architectures](./frontier-architecture-case-studies.md).

## Purpose

A computer-science architecture can consume several resources at once. FLOP
work, activation bytes, persistent storage, memory residency, bandwidth, and
latency are useful dimensions, but they are not interchangeable currencies.
Adding them into one score hides the question being answered and invents an
exchange rate that the description does not supply.

This contract therefore reports a typed resource vector. Each resource kind has
its own quantity, unit meaning, assumptions, and aggregation. A caller may
compare two reports component by component or provide an explicitly authored
multi-objective decision rule outside this contract.

## Resource kinds

The first bounded vocabulary is:

- `flop-work`: arithmetic work, counted in operations;
- `bytes`: bytes transferred by an explicitly named flow;
- `memory-bytes`: bytes resident in a memory account;
- `bandwidth-bytes-per-second`: a rate, not a byte count; and
- `latency-nanoseconds`: elapsed time under an explicitly named protocol.

These kinds are mutually non-fungible. A byte is not memory merely because it
is stored, bandwidth is not a transfer merely because both mention bytes, and
latency is not converted into FLOPs. Conversions require a separate authored
rule and are not part of the initial analysis.

The implementation uses nonnegative quantities. A quantity is attached to one
resource kind; a bundle is a set of such quantities. Bundle addition is
componentwise only when the resource kinds match. There is no untyped total.

## Flows and charges

A resource flow records an explicit directed movement through the addressed
graph:

```text
flow {
  relation: @system/encoder-to-index,
  source: @system/encoder/output,
  destination: @system/index/input,
  resources: { bytes: 4096 }
}
```

The relation is an addressed connection in the evaluated reprojection. Its
source and destination must agree with the connection endpoints. The resource
analysis does not infer flows from graph reachability or labels.

A resource charge records work or residency attributed to an addressed element
or relation without claiming that the resource itself travels:

```text
charge {
  target: @system/encoder,
  resources: { flop-work: 1000000, memory-bytes: 8192 }
}
```

Flows and charges are separate report sections. A flow may carry `bytes`,
`bandwidth-bytes-per-second`, or another explicitly authored kind; a charge may
record `flop-work`, `memory-bytes`, or `latency-nanoseconds`. The initial
implementation validates the addressed target and preserves the authored
classification. It does not infer which kinds are legal for a given element.

A node consumes, produces, or transforms a resource only when the description
contains an explicit event for that operation. The first bounded implementation
reports authored flow and charge events; automatic conservation, transformation,
resource propagation, scheduling, and hardware behavior remain outside it.

## Probabilistic assumptions

A workload scenario is a complete finite accounting case with:

- a stable name;
- a probability in `[0, 1]`;
- its explicit resource flows; and
- its explicit resource charges.

Scenario probabilities must be finite, nonnegative, and sum to one. Mutually
exclusive branches are represented as separate scenarios. Shared work appears
in each scenario in which it occurs; expected values then weight the complete
scenario rather than double-counting an inferred route.

The first bounded regime uses probabilities for workload or route selection and
keeps each scenario's resource quantities deterministic. A later extension may
model uncertain resource amounts, but it must remain a separate distribution
from workload probability. Measurement uncertainty is a third account carried
by sourced observations and must not be folded into scenario probability.

The expected report is computed independently for every resource kind:

```text
E[resource(kind)] = sum_scenario P(scenario) * resource(kind, scenario)
```

An expected FLOP value and an expected byte value remain separate fields. The
report does not expose a single expected-resource scalar.

## CLI analysis input

The CLI accepts a tab-delimited analysis file so resource events remain separate
from the Grimoire text grammar. Blank lines and lines beginning with `#` are
ignored. The records are:

```text
scenario<TAB>name<TAB>probability<TAB>assumption
flow<TAB>scenario<TAB>relation<TAB>source<TAB>destination<TAB>resource<TAB>quantity
charge<TAB>scenario<TAB>target<TAB>resource<TAB>quantity
```

Several `flow` or `charge` records may describe different resource kinds for
one event. The CLI groups those records into one typed bundle and then invokes
the same `ResourceModel` used by the library. The file is an explicit analysis
input, not a second architecture description and not an extension to the
Grimoire grammar.

## Evaluation boundary

Resource analysis consumes a `StructuralReprojection` and explicit events. It
checks that:

- every flow relation is visible and is a connection;
- flow endpoints match that connection exactly;
- every charge target is visible;
- every resource bundle has unique kinds and valid quantities;
- every scenario probability is finite and valid;
- scenario names are unique; and
- scenario probabilities normalize to one.

It returns either a typed report or a visible error. It does not mutate the
reprojection, select elements, evaluate runtime loops, infer device topology,
or derive probabilities from labels, architecture names, or connection paths.

The existing `CostModel` remains the deterministic symbolic expression API.
`bytes_on_wire` remains the deterministic placement-and-shape transfer API.
This resource contract composes with both APIs but does not replace or silently
reinterpret them.

## Architecture coverage

The resource model must be exercised against the architecture families already
represented by the case studies:

| Architecture family | Resource-flow pressure |
| --- | --- |
| Model-relative indexed artifact pipeline | Separate model, index, checkpoint, residual, access, and integrity byte accounts, plus explicit build/read workload scenarios. |
| Persistent semantic corpus and search | Separate model-cache, source, slicing, embedding, persistent-corpus, query, scan, handle, and context resources, plus explicit cold-ingest, warm-ingest, search, and handle-lookup workload scenarios. |
| V-JEPA 2 shared encoder and pretraining | Shared encoder work and activation movement reused across pretraining, anticipation, VidQA, and planning views. |
| V-JEPA 2-AC | FLOP work over frame and hidden dimensions, action/state input movement, and explicit recurrent-horizon workload assumptions without executing rollout. |
| Bridge-based multimodal systems | Visual-token movement and bridge work kept distinct from language-core work. |
| Unified discrete multimodal systems | Shared-transformer work and modality-specific paths represented as explicit branches, not inferred from one total. |
| Decoupled frontends | Separate semantic and generative frontend resources until their explicitly shared backbone. |
| Continuous latent generation | Conditioning, denoising, latent movement, and deferred sampling resources kept separate. |
| TiTok-style tokenization | Sequence resources use authored token dimensions and do not acquire spatial cost from labels. |
| Streaming speech | Byte-rate and latency resources remain distinct from static model work; scheduling is external. |
| Latent dynamics and planning | Observation, action, state, and prediction resources can be scenario-weighted without claiming runtime control. |
| Low-bit operators | Precision may change an authored expression, but dtype width and hardware throughput are never inferred. |
| Parameter-space lineage | State, delta, merge, and checkpoint storage remain separate from activation flows. |

The architecture proposals own the observed structural distinctions. This
proposal supplies a common analysis account for their explicit resource facts;
it does not make any paper family a core vocabulary term.

The public [Scry architecture fixture](../examples/scry.grimoire) and its
[resource sidecar](../examples/scry-resources.tsv) exercise this row. Their
measurement decorations cite Scry's public implementation and architecture
records for the model assets, token window, embedding dimension, source-read
bound, and corpus-scan behavior. The fixture shares the indexed-artifact
pressure for separate model, data, access, and integrity accounts, while its
addresses and operation names remain local example structure rather than new
Grimoire primitives. Local corpus files, caches, checkout paths, and the Scry
MCP configuration are not inputs to the public fixture.

## Report and failure shape

A caller pairs the report with the evaluated description or layer identity. A
report includes:

- the scenario names and normalized probabilities;
- expected flow quantities by resource kind;
- expected charge quantities by resource kind; and
- the explicit scenario assumptions supplied by the caller.

Source and protocol context for measured constants remains on the attached
`measurement/1` values when present. The resource report does not silently
merge those description decorations into its sidecar event assumptions.

Failures identify the responsible boundary and relevant address or scenario:
missing elements, non-connection flow relations, endpoint mismatches, duplicate
resource kinds, invalid probabilities, non-normalized scenarios, unsupported
conversions, and arithmetic overflow are never silently repaired.

A report is analysis output. It cannot feed back into `select`, `invert`,
`decorate`, or checks that determine structural identity. A sourced measurement
may qualify a report, but it does not become an exact calculation merely because
it is attached to the same element.

## Sized extensions

The first implementation intentionally leaves these boundaries open:

- uncertain resource amounts as distributions separate from workload
  probabilities;
- conditional resource events with explicit node-state predicates;
- automatic flow conservation or transformation laws;
- multi-hop route attribution and shared-subgraph allocation;
- platform-specific hardware throughput and device topology;
- serialized resource events in the Grimoire text grammar; and
- a general optimization or Pareto-selection rule across unlike resources.

Each extension requires a concrete architecture fixture and a report contract
before it enters the implementation.
