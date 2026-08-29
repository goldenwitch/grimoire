# Visualization design

This proposal defines the first visualization boundary for Grimoire. It is a
rendering contract, not a new core vocabulary or a runtime execution model.
The implementation is a static HTML viewer with a self-contained presentation
model. Its data is a curated snapshot and is not a second executable source of
structural truth; the rendering contract does not depend on how the snapshot is
produced.

The governing sources are [grimoire.md](../spec/grimoire.md),
[projection-language.md](./projection-language.md), the reference fixture in
[reference_validation.rs](../crates/grimoire/tests/reference_validation.rs),
and the observed cases in [v-jepa-2-case-studies.md](./v-jepa-2-case-studies.md)
and [frontier-architecture-case-studies.md](./frontier-architecture-case-studies.md).

## Current artifact

[`viz/index.html`](../viz/index.html) is the first browser-openable viewer. It
contains the static view model for the V-JEPA 2 reference, the cross-paper
consumer cases, and the Fauxlden Retriever fixture. It requires no build
step, server, dependency, or network request. Open it directly from the
workspace to inspect the views.

## First view

The primary workflow is layer comparison with single-reprojection inspection.
A viewer opens one description, chooses a left reprojection and an optional
right reprojection, and renders the selected addressed elements. The default
worked case is the V-JEPA 2 reference:

- shared core;
- action-free pretraining;
- action-conditioned prediction;
- downstream consumers; and
- static planning.

The cross-paper consumer fixture is a secondary acceptance case. It exercises
bridge-based, unified, decoupled, latent, tokenizer, speech, low-bit, and
lineage boundaries without making those terms viewer primitives.

A viewer may render a core graph directly or render an evaluated layer result.
A layer view is the `FinalizedReprojection` produced by the static evaluator,
not an unevaluated projection and not a live run. The viewer never executes a
training step, sampling loop, recurrent rollout, controller, or network call.

## Structural rendering

The structural canvas renders only addressed elements present in the chosen
reprojection:

- blocks are primary computational nodes;
- ports are visible attachment points inside their blocks;
- connections are directed edges between ports;
- groups are collapsible containment or membership boundaries; and
- the description element is available as document context, not as a fake
  computational node.

Addresses are stable DOM and interaction identifiers. Human labels are display
text only. The renderer must not infer a connection, tensor shape, location,
attention pattern, or semantic role from a label.

Groups may be collapsed for navigation, but collapsing is a presentation state.
It must not remove members from the underlying result or change an edge's
meaning. A selected group exposes its direct members and nested groups without
changing the structural result.

The viewer does not expose the evaluator's private definition-origin metadata.
When two reprojections are compared, it may say that an address is present only
on the left or right, but it must not claim that a legal definition-site choice
changes projection identity.

## Comparison rendering

Comparison is address-based:

- common addresses receive a shared identity;
- left-only and right-only addresses receive distinct presence states;
- common connections are compared by address and endpoint addresses; and
- a changed finalized value is reported in metadata, not as a structural edge
  change.

The comparison view must make the shared encoder and distinct local predictors
legible in the V-JEPA case. It must also show that a bridge-based language
consumer and a shared-transformer consumer have different structural paths.
Comparison is not a graph merge and does not manufacture a union reprojection.

## Finalized metadata

Finalized decorations appear in a separate metadata surface for the selected
address. The structural canvas remains unchanged when a decoration changes.
The metadata surface shows, where present:

- schema namespace, parameter, and version;
- the decoded value for recognized schemas;
- source/protocol context for measurements;
- placement location labels;
- training and execution boundary values;
- precision values; and
- provenance and lineage values.

Unknown extension parameters remain an opaque record. The viewer may show their
namespace, parameter identity, and preserved source text, but it must not parse
or reinterpret the payload as a known value.

## Analysis overlays

Analysis overlays are opt-in and are visually distinct from structure. They
are attached by address or relation identity and must carry their analysis
status:

- placement may tint or group addressed elements by authored location;
- cost may display an explicitly supplied expression or evaluated total;
- channel claims may identify source and terminal addresses and show exact or
  posterior estimates with their intervals; and
- route allocations may show their declared partition and uncertainty.

An overlay with missing inputs is shown as unresolved with its reason. The
viewer never derives cost from a shape label, information from topology,
placement from an address string, or route percentages by summing branch
mutual informations.

The following statuses are first-class display states: exact, measured,
posterior, opaque, deferred, unresolved, and absent. A deferred continuous
neural estimate, EMA parameter relation, fixed-point cycle, causal intervention,
or runtime controller is visible as deferred/unresolved context rather than a
blank field or a fabricated number.

## Interaction contract

The first viewer needs these interactions:

- choose a layer or core result;
- choose an optional comparison result;
- focus an address from the canvas or an address list;
- expand/collapse groups;
- toggle metadata and analysis overlays; and
- inspect the reason attached to a deferred or unresolved outcome.

Every interactive address has a non-color identity. Focus, selection, and
comparison state must be available through keyboard navigation and readable
text. Color may reinforce presence or status, but it cannot be the only carrier
of meaning.

The layout must remain stable when labels, metadata, or status text changes.
Long addresses and labels wrap or move to an adjacent detail surface rather
than changing node dimensions unpredictably or overlapping neighboring content.

## Acceptance fixtures

The current implementation consumes a frozen, curated view model without
introducing a visualization-specific input format. The Rust fixtures remain the
executable validation source; the viewer is a presentation artifact over the
same public cases. The acceptance cases are:

1. V-JEPA 2: compare `pretraining`, `ac`, `vidqa`, and `planning` against the
   shared core. The encoder address remains common; the pretraining and AC
   predictors remain distinct; action, state, and visual ports remain visible;
   the planning controller is marked as an external execution boundary.
2. Cross-paper consumers: compare `bridge`, `unified`, `decoupled`, `latent`,
   `tokenizer`, `speech`, `dynamics`, `low-bit`, and `lineage`. Bridge and
   unified paths remain distinct; the one-dimensional tokenizer remains a
   sequence; precision and lineage remain metadata/analysis rather than
   activation edges.
3. Opaque data: the Fauxlden fixture displays the unknown hot-path facts as
   opaque text and preserves them through serialization.
4. Empty and unresolved states: an empty reprojection, an incomplete cut, and
   an unavailable analysis input have visible statuses and do not crash or
   acquire inferred structure.

The acceptance checks compare the rendered address set and relation set with
the evaluated reprojection. They do not test a particular graph layout or
force one visual style.

## Explicit non-goals

The first viewer does not:

- edit descriptions or projections;
- expose definition sites as semantic identity;
- execute projections against runs;
- animate training, diffusion, recurrence, CEM, or robot control;
- infer tensor correspondence, topology, placement, cost, or information;
- normalize unknown extension payloads; or
- require a paper-specific visual primitive.

A later implementation node may choose the HTML transport, graph layout
algorithm, and static asset packaging, provided those choices preserve this
contract.
