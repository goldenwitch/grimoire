# Provenance schema

Status: prototype schema checkpoint with open granularity gaps.

This proposal instantiates the citation, assumption, and novelty value used by
the provenance layer and the architecture case studies. It does not make
citations or novelty claims part of the frozen core graph.

The governing sources are [grimoire.md](../spec/grimoire.md),
[schema-format.md](./schema-format.md), and
[schema-inventory.md](./schema-inventory.md).

## Purpose

The architecture cases mix established components, adaptations of prior work,
and novel combinations. A description needs to carry those distinctions next
to the group or element they qualify, rather than relying on a bibliography
outside the description.

Provenance is a finalized value. It can be inspected by a provenance check, but
it cannot choose graph structure, change a connection, or alter a projection's
structural result.

## Candidate Contract

Candidate value body:

```text
product{
  citations: seq(text),
  assumptions: seq(text),
  novelty: enum{novel, existing, adapted, unclassified}
}
```

`citations` and `assumptions` are homogeneous sequences and may be empty. The
schema does not impose a citation style. A citation may be an arXiv identifier,
DOI, repository URL, or a local source record.

`novelty` is required so a provenance check can identify an unclassified group
without confusing absent data with a known negative claim. `existing` means the
represented technique is claimed as prior work, `adapted` means the described
use changes or composes prior work, and `novel` is an authored claim. The schema
does not verify the truth of those claims; it preserves them for review.

## Attachment Convention

The core specification names groups as the initial provenance target. The first
fixture therefore attaches `provenance/1` to groups only. A group may contain
blocks, ports, connections, or nested groups, and a provenance value on a group
can cover the named technique or composition represented by that group.

Architecture facts that need element-level source traceability may use
`measurement/1` source records or a later extension namespace. The provenance
schema should not broaden its attachment kinds just to duplicate measurement
origin.

## Architecture Fit

The indexed papers provide direct provenance cases:

- the V-JEPA 2 encoder and action-conditioned world model as an adaptation of
  joint-embedding predictive architecture;
- Janus's decoupled visual encoding as a claimed architectural contribution;
- Chameleon and Emu3 as prior unified multimodal approaches;
- BitNet's low-bit operator and BitNet a4.8's hybrid quantization; and
- DARE and TIES-Merging as adaptations of parameter-space model combination.

A worked description can group a shared encoder, a predictor, or an entire
training stage and decorate the group with citations and assumptions. A local
combination may be marked `adapted` without claiming every component is novel.

The schema also supports the novelty-surface check in the core specification:
select every group whose provenance value is `unclassified` and expect an empty
result. Groups with no references or no provenance are not structural errors;
they become visible through the explicit finalization check chosen by the
provenance layer.

## Fixtures

Valid fixtures:

- empty citations and assumptions with `unclassified` novelty;
- one citation and one assumption with `existing` novelty;
- multiple citations with `adapted` novelty;
- a nested group with provenance on both parent and child;
- a group folded from references carrying the same provenance namespace; and
- provenance values attached after structural folding.

Invalid fixtures:

- missing novelty field;
- novelty outside the closed enumeration;
- a scalar where a citation or assumption sequence is required;
- provenance attached to an element kind outside the reviewed allowed set;
- a structural selector that branches on novelty before finalization; and
- a check that observes an uncited block directly rather than a finalized
  provenance value.

Fixtures should prove that changing a citation or novelty value does not change
the selected addresses or connections.

## Decision Record

This proposal records these decisions for concrete schema work:

- Provenance is a product of citation sequence, assumption sequence, and
  required novelty state.
- Citation and assumption sequences may be empty.
- Novelty uses the closed values `novel`, `existing`, `adapted`, and
  `unclassified`.
- The initial attachment target is groups, matching the core provenance layer.
- Provenance values are finalized and cannot feed structural evaluation.
- Measurement source records and technical provenance remain separate schemas.

## Sized Gaps

### Element-level provenance

- Binds when: a case requires a citation or assumption on one block or port
  that cannot be meaningfully grouped without losing locality.
- Cost of absence now: the group-level schema can still record the claim, but
  the exact element carrying it is less precise.
- Candidate shape and rough size: permit attachment to blocks and groups while
  retaining the same value body; one attachment-rule update and fixtures.
- Entry trigger: the first reference description that needs a provenance check
  over individual operators rather than groups.

### Citation identity

- Binds when: two textual citations should be checked as the same source or a
  source must be traced to a local Scry handle.
- Cost of absence now: text preserves the citation but does not give it a
  machine-stable identity.
- Candidate shape and rough size: replace or supplement citation text with an
  address reference to a source element; this requires a source artifact
  decision before schema change.
- Entry trigger: the first provenance fixture requiring source-handle
  round-tripping.
