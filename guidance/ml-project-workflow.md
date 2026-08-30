# Plan an ML project with Grimoire

Grimoire is a design-time and review-time description of an ML system. Use it
to make the system's structure, training surfaces, deployment choices, evidence,
and open boundaries inspectable in one address space. It does not run training,
bind a projection to a batch, execute a controller, or infer measurements from
architecture labels.

This guide is procedural. The [core specification](../spec/grimoire.md) is the
authority for meaning, the [concrete grammar](../grammar/grimoire.md) is the
authority for syntax, and the [schema inventory](../proposals/schema-inventory.md)
records the current schema families. When this guide and those documents differ,
the specification and grammar win.

## The workflow

The shortest useful loop is:

1. State the planning question and the boundary of the system.
2. Model stable, shared structure once in the core graph.
3. Add a layer for each distinct view, objective, mode, or review question.
4. Attach typed facts with schemas, keeping evidence and assumptions visible.
5. Validate, evaluate, and canonicalize the description.
6. Extract downward-closed cuts for individual reviews or deliverables.
7. Run cost, placement, or information analysis only with explicit inputs.
8. Record absent, opaque, unresolved, and deferred facts instead of filling them
   in from intuition.

The result is a set of related views over one structure, not a collection of
independent model diagrams.

## Two primary workflows

The front door presents two workflows. They share one addressed description but
answer different questions:

1. **Cost and resource analysis** calculates or estimates what the described
    system requires.
2. **Architecture substrate and diagrams** preserves and communicates what the
    system is.

Keep the workflows connected through shared addresses, but do not make a
resource result part of structural identity or make a diagram a second model.

### Cost and resource analysis

Use this workflow when the question is how much computation, memory, bandwidth,
or another resource a described structure uses or is expected to use.

1. Define the structure and evaluate the layer or cut being analyzed.
2. For a deterministic calculation, provide explicit cost expressions, axis
    extents, shapes, placements, and element widths as required by the analysis.
3. For an observed or estimated value, record its unit, source, protocol,
    assumptions, and uncertainty context when available.
4. Report missing inputs and unsupported inference visibly. Do not derive
    hardware throughput, device topology, contention, dtype width, or runtime
    behavior from names, labels, or graph topology.

The [typed resource-flow proposal](../proposals/resource-flow.md),
[cost-layer proposal](../proposals/cost-layer.md),
[placement and bandwidth proposal](../proposals/placement-bandwidth.md), and
[measurement schema](../proposals/measurement-schema.md) define the relevant
boundaries. The implementation's `CostModel` and `bytes_on_wire` APIs provide
the deterministic calculation path. The [reference example](../examples/reference.grimoire)
provides a small addressed description with separate `cost` and `deployment`
layers to use as a starting point. Its [resource events](../examples/reference-resources.tsv)
file shows the explicit scenario input format for the probabilistic report.

The [Scry architecture example](../examples/scry.grimoire) exercises the same
workflow against a persistent semantic corpus and search substrate. Its
[resource events](../examples/scry-resources.tsv) distinguish cold and warm
ingestion from indexed search and handle lookup, while keeping model-cache,
source, corpus, memory, bandwidth, and latency quantities separate. The
fixture's measurements cite public Scry implementation facts; its event
quantities are explicit analysis inputs rather than runtime traces or inferred
hardware behavior.

### Architecture substrate and diagrams

Use this workflow when the question is how to preserve, compare, or communicate
a computer-science architecture over time.

1. Put stable shared structure in the core graph with exact addressed blocks,
    ports, connections, and groups.
2. Add layers for distinct consumers, objectives, modes, or review questions.
3. Validate definitions, references, visibility, and layer inputs; then use
    canonical serialization to make the model portable and reviewable.
4. Extract downward-closed cuts when a review needs a self-contained slice.
5. Inspect evaluated layers in the read-only viewer, comparing structure by
    address and keeping finalized values and resource overlays separate.

The [core specification](../spec/grimoire.md) and
[concrete grammar](../grammar/grimoire.md) define the substrate. The
[visualization boundary](../proposals/visualization.md) defines the diagram
surface. Diagram coordinates, labels, and overlays are presentation data; they
cannot create, merge, or reinterpret addressed elements. The [reference
example](../examples/reference.grimoire) provides a compact model to validate,
canonicalize, cut, and inspect.

The Scry example is another static substrate, not a runtime integration. Its
MCP block is modeled as an outer transport boundary; the current Grimoire
release does not ship an MCP adapter.

## 1. Start with the planning question

Write down what the project needs to decide before writing blocks. Examples:

- Can one representation support both supervised prediction and generation?
- Which parts are trained, frozen, or adapted for each objective?
- What must cross a device boundary at deployment?
- Which claims are measured, and what source or protocol supports them?
- What is the smallest self-contained artifact for an architecture review?

Give the description one stable address and a short human label. Decide what
the description includes: an architecture, a training recipe, a deployment
shape, or a static account of all three. A runtime loop may be represented as
static structure plus an `execution/1` value, but its actual execution remains
outside the description.

Keep a small inventory while planning:

| Planning need | Grimoire account |
| --- | --- |
| Shared computational identity | An addressed `block` in the core graph |
| Interface crossing a block boundary | An addressed `port` |
| Data or control route | A directed `connection` |
| A reusable stage or review unit | A `group` |
| A training, mode, deployment, or analysis view | A `layer` |
| A typed fact about an element | A schema-governed extension parameter |
| A reviewable subset | A downward-closed `cut` |

Do not make a paper name, benchmark, optimizer, or runtime event a new core
element kind. First ask whether it is structure, a finalized value, an external
measurement, or a boundary that is not represented yet.

## 2. Build the shared core graph

Put only the structure that multiple views genuinely share in the core graph.
The core graph contains blocks, ports, connections, and groups. It does not
decide whether a connection is a forward activation, a gradient route, a
parameter update, or a network transfer; layers and explicit analyses supply
those meanings.

Choose addresses for identity, not for visual indentation. Addresses are flat,
exact, and case-sensitive even when they contain slash-separated segments. A
shared address means actual reuse by multiple layers in this description. Two
unrelated encoders do not become one element because both are called
`vision-encoder`.

A small core might look like this:

```text
grimoire 1.0.0
description @demo "classification project" {
    core-spec 1.0.0;
    core {
        block @demo/input "Input" {
            port @demo/input/output;
        }
        block @demo/encoder "Encoder" {
            port @demo/encoder/input;
            port @demo/encoder/output;
        }
        block @demo/head "Classifier" {
            port @demo/head/input;
            port @demo/head/output;
        }
        connection @demo/input-to-encoder
            @demo/input/output -> @demo/encoder/input;
        connection @demo/encoder-to-head
            @demo/encoder/output -> @demo/head/input;
        group @demo/model "Model" {
            @demo/encoder,
            @demo/head,
            @demo/input-to-encoder,
            @demo/encoder-to-head;
        }
    }
    layer "architecture" {
        inputs { core };
        consumes {
            projection-language 1.0.0;
            schemas { }
        }
        projection {
            select {
                use @demo/model;
            }
        }
    }
}
```

The `architecture` layer selects the group, so the current evaluator
materializes its visible members. You can select an explicit address list when
that is clearer for a smaller view. A group is organization and selection
context; it is not an implicit new computation.

Before adding detail, check the core graph locally:

- Every block has the ports needed by its visible connections.
- Every connection names exactly two existing ports.
- Every element has one unique address.
- A group contains only the elements that its name and review purpose require.
- A tensor, token stream, or scalar is a port value unless it must itself be
  selected or referenced as an element.

## 3. Add layers by question

A layer is one human viewport over declared inputs. Its input declarations form
a DAG. A layer can see the core graph and the reprojections of the layers it
declares; it cannot silently reach into an undeclared layer.

Use one layer when a question needs a distinct structural or finalized view. A
typical project may grow these layers:

| Layer | What it answers |
| --- | --- |
| `architecture` | What is the stable computation and interface? |
| `pretraining` | What structure and training facts define representation learning? |
| `finetuning` | Which target is adapted, with what objective and data? |
| `evaluation` | Which consumer and measurement boundary is being reviewed? |
| `inference` or `generation` | Which alternative path is selected in this mode? |
| `deployment` | Which addressed elements have authored placement and external boundaries? |
| `provenance` | Which groups have citations, assumptions, and novelty state? |

These names are workflow conventions, not new vocabulary. Use names that match
the actual review surfaces. A mode-local cache or probe belongs in its mode
layer. An alternative implementation shared by all modes can be an ordinary
core block, with each mode selecting the appropriate address.

Layer inputs also determine cuts. If `planning` consumes `ac`, then a cut
containing `planning` must contain `ac` as well as the core graph. Selecting
`planning` alone is intentionally reported as an unresolvable C12 result.

## 4. Use projection stages deliberately

Every layer projection has one global order:

```text
select + invert -> decorate -> checks
```

### Select structure

Use `select` to reference visible addresses or to generate ordinary blocks,
ports, connections, and groups that are local to the layer. Generated
definitions obey the same address, visibility, and locality rules as core
definitions. They cannot read finalized values, and the order of generated
definitions in the file does not create extra visibility.

### Invert topology

Use `invert` when a view needs the direction sign of every connection in a
selected group reversed, such as a structural backward view. `invert` is not a
Shannon-information reversal and does not execute a gradient calculation. A
stopped or excluded route is represented by the selected structure and its
connections, not by guessing from an optimizer or loss label.

### Decorate facts

Use `decorate` after structural selection is complete. Decoration can attach a
configuration, source record, placement, measurement, or other schema-governed
value to a folded address. Changing a dial or citation must not change the
selected structure.

### Check coverage

Use `checks` for questions over finalized values, such as whether every
training-stage group has a `training/1` value or whether a provenance value is
present. A check returns an expected empty or nonempty result; it does not alter
the structure or discard decorations.

## 5. Attach typed facts and evidence

The schema inventory defines ten families for recurring planning facts:

- `axes/1` and `shapes/1` for symbolic dimensions and coarse interfaces;
- `architecture/1` for model and operator facts;
- `training/1` for objectives, dials, phases, targets, and data sources;
- `execution/1` for static, streaming, recurrent, and closed-loop boundaries;
- `precision/1` for weights, activations, accumulation, optimizer state, and
  sparsity;
- `measurement/1` for literal observations with units and source records;
- `provenance/1` for citations, assumptions, and novelty;
- `lineage/1` for parameter-state ancestry, without pretending it is activation
  flow; and
- `placement/1` for authored deployment locations.

Paper and product names remain values inside these schemas. They do not become
schema families or structural primitives merely because they are important to
the project.

For example, a training layer can attach a typed training record to the group
that represents its stage:

```text
layer "finetuning" {
    inputs { core };
    consumes {
        projection-language 1.0.0;
        schemas {
            "https://github.com/goldenwitch/grimoire/extension/training"
                / training @1.0.0;
        }
    }
    projection {
        select {
            use @demo/model;
        }
        decorate {
            on @demo/model extension
                "https://github.com/goldenwitch/grimoire/extension/training"
                training schema training @1.0.0 = {
                    objective: "supervised classification",
                    optimizer: present("adamw"),
                    batch_size: present(128),
                    steps: present(10000),
                    phases: [],
                    trainable_targets: [ref(@demo/head)],
                    frozen_targets: [ref(@demo/encoder)],
                    data_sources: ["dataset-v1"]
                };
        }
        checks {
            check training-covered expect nonempty
                over "https://github.com/goldenwitch/grimoire/extension/training" training;
        }
    }
}
```

Use `absent` when a field is known to have no value in this account, not when
the field was forgotten. Use `present(...)` for a supplied value. For an
external observation, include its unit and source context. For an unfamiliar
extension namespace, preserve the opaque payload and do not reinterpret it as
one of the known schemas.

## 6. Validate the description early

Validation is part of planning, not a final formatting step. The Rust crate
exposes the current static path directly:

```rust
use grimoire::{
    evaluate_layer, parse_description, prototype_schemas, serialize_description,
    validate_description,
};

let description = parse_description(source)?;
let schemas = prototype_schemas()?;
validate_description(&description, &schemas)
    .map_err(|errors| format!("validation failed: {errors:?}"))?;

for layer in &description.layers {
    evaluate_layer(&description, &layer.name)?;
}

let canonical = serialize_description(&description)?;
```

The validator checks address uniqueness, connection endpoints, definition-site
visibility, locality, layer input resolution and acyclicity, extension
namespaces, schema values, and the cut rules. The evaluator keeps structural
results separate from finalized decorations and reports projection-stage errors
with their stage and relevant identifier.

From the repository root, the focused executable baseline is:

```text
cargo test --workspace --locked
```

The tests under `crates/grimoire/tests/` are also precise examples of the API.
The whole-system reference is
[reference_validation.rs](../crates/grimoire/tests/reference_validation.rs).
It composes shared structure with pretraining, action-conditioned, consumer,
planning, mode, placement, cost, provenance, information, execution, precision,
and lineage boundaries.

Canonical serialization is useful in review: a valid description should round
trip without losing recognized values or opaque extension data. Keep the
canonical output in the change under review when it is the artifact that other
people will inspect.

## 7. Produce review cuts

Use a cut when a reviewer needs one self-contained slice of the project. A cut
contains the core graph, the selected layers, and every layer in their input
chains. It is not an arbitrary subset of files.

The Rust API is:

```rust
use grimoire::extract_cut;

let cut = extract_cut(&description, &["core-view", "finetuning"], &schemas)?;
```

The selected names are layer names; the core graph is included automatically.
Choose a downward-closed set such as `ac` plus `planning` when planning
declares `ac` as an input. A missing dependency produces visible C12
unresolvability. A successful cut is revalidated, so a reviewer can treat it as
a standalone description rather than trusting that the extraction happened to
preserve enough context.

Useful cuts for an ML project often include:

- a core architecture cut for interface review;
- a training cut for objective, data, frozen-target, and dial review;
- a deployment cut for placement and external execution boundaries; and
- an evaluation cut containing the consumer and the measurements it actually
  supports.

## 8. Run analyses only with their missing inputs made explicit

The description supplies addressed structure and authored values. Several
analyses deliberately live at a host-side boundary because the description
does not contain enough information to calculate them honestly.

### Cost

Use `CostModel` with explicit constants, addressed axes, sums, and products.
Provide positive axis extents at evaluation time. A cost expression does not
infer FLOPs, sparsity, throughput, or dtype width from a block label.

```rust
use grimoire::{Address, CostExpression, CostModel};
use std::collections::BTreeMap;

let model = CostModel::new(vec![
    (
        Address::parse("@demo/encoder")?,
        CostExpression::product(vec![
            CostExpression::axis(Address::parse("@demo/frames")?),
            CostExpression::constant(1408),
        ]),
    ),
])?;
let mut axes = BTreeMap::new();
    axes.insert(Address::parse("@demo/frames")?, 16);
let report = model.evaluate(&reprojection.structural, &axes)?;
```

### Placement and bandwidth

Decorate authored locations, then provide explicit tensor dimensions and
`bytes_per_element` to `bytes_on_wire`. Same-location links report no traffic;
cross-location links are counted only when placement and shape inputs exist.
The API does not infer a device topology, collective algorithm, contention, or
latency.

### Information flow

Use an explicit channel interpretation for information claims: identify the
source port, terminal ports, finite distributions, block kernels, quantity,
method, and evidence context. Structural reach is not mutual information, and
route percentages are not obtained by adding branch mutual informations. A
continuous estimator, cyclic channel, causal intervention, or correlated source
remains unresolved or deferred until its contract is supplied.

## 9. Inspect and compare views

Open the [static viewer](../viz/index.html) directly from the workspace to
inspect the current reference corpus. Its primary workflow is layer comparison
and single-reprojection inspection:

- compare by exact address, so shared identity and local additions are visible;
- inspect finalized metadata separately from the graph;
- opt into placement, cost, channel, measurement, provenance, precision, or
  lineage overlays only when their records are supplied; and
- keep exact, measured, posterior, opaque, deferred, unresolved, and absent
  states distinct.

The viewer is a static artifact. It does not execute projections, training,
sampling, recurrence, controllers, or network calls. Its current embedded
model is a curated presentation snapshot; it is a way to inspect the workflow,
not a replacement for validating a source description.

The viewer's pure presentation step has the shape
`renderPresentation(model, uiState) -> presentation`. It reads no DOM or global
state; DOM updates and event handlers apply its returned presentation outside
the pure step.

## 10. Use the V-JEPA 2 path as a worked pattern

The [V-JEPA 2 case study](../proposals/v-jepa-2-case-studies.md) is the
canonical worked system because it exercises several planning surfaces without
duplicating the shared representation:

| Project concern | Addressing pattern |
| --- | --- |
| Shared visual representation | Put the encoder in the core graph. |
| Masked representation pretraining | Define the target encoder, mask token, predictor, and objective in a pretraining layer. |
| Action-conditioned prediction | Define action, state, and its distinct predictor in an action-conditioned layer. |
| Downstream language consumer | Define the bridge and language model in a VidQA layer. |
| Planning | Define goal and controller structure in a planning layer; mark the closed-loop consumer with `execution/1`. |
| Training and provenance | Decorate the relevant stage groups rather than changing the graph. |

The pretraining predictor and the action-conditioned predictor have different
addresses because they have different inputs and roles. The encoder has one
shared address because the views genuinely consume the same representation.
That distinction is the core planning discipline: fold identity by address, not
by a similar name.

## Review checklist

Before treating a description as a project plan, check:

- The planning question and system boundary are stated.
- The core graph contains only structure shared by the intended views.
- Addresses are stable and unique; reused addresses mean actual reuse.
- Connections join existing ports and have the intended structural direction.
- Each layer declares every input it needs, and its input chain is acyclic.
- Mode-local or objective-local elements are not leaked into unrelated layers.
- `select` and `invert` determine structure before any decoration is read.
- Training, architecture, placement, measurement, and provenance facts use the
  appropriate schema and attachment target.
- Evidence includes a source or protocol where a claim depends on an external
  observation.
- Cuts contain their full layer input chains and revalidate successfully.
- Cost, bandwidth, and information reports name the explicit inputs used.
- Runtime behavior, parameter updates, continuous estimates, and causal claims
  are labeled as external, unresolved, or deferred when no settled contract
  represents them.

The final artifact is ready for review when another person can answer "what is
shared, what changes by view, what is known, and what is still a boundary?" by
following addresses, declared inputs, schemas, checks, and cuts rather than
reconstructing the project from prose alone.