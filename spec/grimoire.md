# Grimoire — a layered description language for ML systems

*Grimoire is the project name. This document records design intent — current best guesses, not settled fact. Version 1.0.0.*

## Conventions

- The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT, RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are to be interpreted as described in BCP 14 (RFC 2119, RFC 8174) when, and only when, they appear in all capitals.
- Words in **bold** are vocabulary. Every bold term has exactly one definition, given in the Vocabulary section, and every use of a defined term is bold.

## Vocabulary

Each term is defined once, in terms of primitives and other vocabulary. These definitions are intended to graduate into formal productions of the language grammar.

- **description** — a **core graph** together with any set of **layers** over it.
- **address** — a stable identifier, unique within a **description**.
- **element** — anything that carries an **address**: a **block**, **port**, **connection**, **group**, or the **description** itself.
- **block** — an **element** with a human name and zero or more **ports**.
- **port** — an **element** belonging to exactly one **block**, at which **connections** attach.
- **connection** — a directed **element** joining exactly two **ports**.
- **group** — an **element** naming a set of **elements**; **groups** may contain **groups**.
- **schema** — a versioned, machine-checkable definition of the structure and constraints of the **extension parameters** in one **extension namespace**.
- **extension namespace** — a qualifier for **extension parameters**.
- **extension parameter** — a structured value attached to an **element**, qualified by an **extension namespace** and governed by a **schema**.
- **projection language** — the companion specification defining how **projections** are written and evaluated; versioned in its own right.
- **projection** — a pure, static function from **description**-shaped input to **description**-shaped output, written in the **projection language**, which is specified separately.
- **layer** — one file and human viewport in a **description**, with a name unique within that **description**, a set of declared inputs, the **schemas** and **projection language** version it consumes, and one **projection**.
- **core graph** — the root **definition site** of a **description**: the **blocks**, **ports**, **connections**, and **groups** visible to every **layer**, with their **extension parameters**.
- **definition site** — the one place an **element** is defined: the **core graph**, or exactly one **layer**.
- **input chain** — a **layer**'s declared inputs, their declared inputs, and so on, down to and including the **core graph**.
- **reprojection** — the result of applying a **layer**'s **projection** to its declared inputs.
- **select** — the fragment of the **projection language** that defines a **layer**'s **elements** by generating them, taking references to existing **elements**, or both.
- **invert** — the fragment of the **projection language** that takes a **group** of **elements** and reverses the direction sign of every **connection** in it.
- **decorate** — the finalization fragment of the **projection language** that attaches **schema**-governed values after structural evaluation.
- **check** — a lossy finalization operation over values attached by **decorate**, with an expected cardinality of its result, empty or nonempty.
- **core spec version** — the version of this specification that a **description** conforms to.
- **cut** — the **core graph** together with selected **layers** and every **layer** in their **input chains**.

## Purpose

A Grimoire **description** is one **core graph** of **blocks** and **connections**, plus **layers** that project it. Gradient-flow views, hyperparameter records, placement plans, cost accounting, provenance reports, and per-mode views are all **reprojections** of the same shared structure. **Layers** fold: **reprojections** join on shared **addresses** and chain by composition, so a composite view is an ordinary **projection**. Each **layer** carries only the context its view requires, so a **cut** of a **description** is self-contained and minimal.

Grimoire is a design-time and review-time artifact.

The property we design for: after any change to a **definition site**, every **layer** either updates coherently or fails visibly. This is the goal, not yet a guarantee.

## Why it exists

A training system is several graphs superimposed on shared structure: activations forward; gradients backward, edited independently by stop-gradients and estimators; optimizer writes into stores; slow copies between them; per-mode state. Good notation exists for pieces of this — tensor and string diagrams handle shape and composition well, compiler IRs are exact, and the categorical literature gives forward and backward the right mathematics — but we have not found a notation that holds the whole training-time picture in one human-authored, checkable **description**, alongside the configuration, cost, and provenance that drift when kept elsewhere. We may have missed one, and the gap may persist because the problem is harder than it looks.

## Scope of this document

- Computation over the data belongs to the **projection language**, specified separately; this document states only the requirements that language MUST satisfy.
- The aim is that every claim in a **description** is checkable from the **description** alone.
- Visualization is specified separately from this document.

## The core graph

The **core graph** is the abstract definition of the **blocks** — a versioned interface that every **layer** consumes. Its semantics are graph semantics — identity, direction, connectivity, containment — and nothing more. It does not know what a gradient, a store, or a clock is; domain meaning is conferred by **layers**.

- Every **element** MUST carry an **address**, unique within its **description**.
- A **connection** MUST be directed and MUST join exactly two **ports**. Direction is structure; what a direction means is conferred by **layers**.
- A **group** MAY contain any **elements**, including **groups**, and MAY serve as a **projection** target standing for its members.
- The **core graph** MAY contain cycles.
- A **description** MUST declare its **core spec version**.
- The format MUST define a canonical serialization.

## Definition sites and scoping

Structure is defined where it is needed. The **core graph** is the bottom of a **description**; each **layer** sits above its declared inputs; references look down.

- Every **element** MUST have exactly one **definition site**.
- Every reference to an **element** MUST occur at that **element**'s **definition site** or in a **layer** above it. Nothing below a **definition site** can see it.
- **Addresses** stay flat: unique across every **definition site** in a **description**.
- Locality: a referenced **element** MUST be defined at a maximal site among those at-or-below every reference to it — equivalently, it MUST NOT be defined below a site its references can already all see. Unrelated referencers force a definition down; the **core graph** is where sharing bottoms out — the meet of the views, not their union.
- Ties: the maximal site need not be unique. Where several sites are legal, placement is authored. Every legal site is, by construction, at-or-below every referencer, so any **cut** containing a referencer contains the **element** under every legal placement; a tie choice affects only **cuts** in which the **element** is unreferenced.
- A persistent tie SHOULD be resolved by defining the missing shared **layer** — the meet of the referencers — which is strictly better for **cut** minimality than either corner.
- An **element** with no references MAY be defined at any site; it burdens only the **cuts** that include its **definition site**. Dead structure MAY be represented as a value attached by **decorate** and observed by a **check**.
- Illustration: an inference-only cache is defined in the mode **layer** that views it; a collective is defined in the placement **layer**; a residual stream that every view references lives in the **core graph**.

## Extension parameters

- Any **element** MAY carry **extension parameters** beyond what this specification defines.
- Every **extension parameter** MUST be qualified by an **extension namespace**. **Extension namespace** identifiers MUST be constructed so that independent authors cannot collide unknowingly.
- A consumer that does not recognize an **extension namespace** MUST preserve its parameters unmodified.
- Shared vocabularies SHOULD be standardized **schemas** rather than core primitives; tensor shapes are the intended first case.
- A symbolic axis is a declaration at a **definition site**, governed by a standardized **schema** and referenced by **address**, subject to the same locality rule as any other declaration: an axis only one view uses lives in that view's **layer**; an axis every view uses lives in the **core graph**.
- A value measured outside the **description** — a profiled number, an observed norm — MAY be carried as a literal **extension parameter** under a **schema** that records its origin.

## The projection language

A **layer**'s **projection** is written in the **projection language** — a companion specification, versioned in its own right. This document does not define that language; it requires the following of it:

- A **projection** MUST be pure and static: it MUST evaluate against its declared inputs alone. No binding to runs is defined.
- The algebra MUST be closed: **description**-shaped input, **description**-shaped output, so **projections** compose.
- The language MUST NOT expose **definition sites**: a **projection** cannot distinguish where an **element** is defined, so every legal placement yields identical **reprojections**.
- The language MUST provide **select** and **invert** for structural evaluation and **decorate** for finalization. An explicit **address** list MUST be a valid **select**.
- The language MUST have one global finalization phase: all **select** and **invert** evaluation MUST precede every **decorate** and **check** evaluation.
- Values attached by **decorate** MUST NOT affect structural evaluation. They MAY be computed, and results MAY be symbolic in declared axes.
- An empty result MUST be a first-class outcome, not an error.
- A **check** MUST operate only over values attached by **decorate**. It is lossy only in its own result and MUST NOT discard or alter finalized data.
- The structural results of **reprojections** MUST fold: join references by shared **address**, and chain by composition. A competing definition at one **address** is an error. Finalization attaches **extension parameters** to the folded **elements**; their identity and constraints are governed by their **extension namespaces** and **schemas**.

Worked case, illustrative: the backward view is a **projection** — **select** the differentiable subgraph, **invert** its **connections**, exclude the stopped **connections** (a stop-gradient is exactly an exclusion), **decorate** the rest. Reach is reachability in the **reprojection**.

## Layers

A **layer**, as data:

- One file: one human viewport.
- A name.
- Declared inputs: a **core graph**, and zero or more other **layers**. Declaring a **layer** grants use of its **reprojection** and visibility of its **definition sites**. Declared inputs MUST form a DAG.
- The **schemas** and **projection language** version it consumes.
- One **projection**, whose **select** defines that **layer** file's **elements**. The **core graph** remains authored base structure; a **layer**'s **select** does not define the **core graph** or any other **layer**.

Intended invariants:

- Coherence: a change to any declared input MUST either leave a **layer** valid or visibly invalidate it. Staleness is detectable, never silent.
- Erasure: any **cut** MUST be a well-formed **description**. A non-**cut** subset degrades visibly: a **layer** whose declared inputs are absent is unresolvable, never silently wrong.

## What each layer requires of the language

The initial **layers**, stated as the features Grimoire must support to express them. The list reflects the systems in front of us today; it is surely incomplete, and **layers** are meant to be cheap enough to add that being wrong here is survivable.

### Information flow

Forward and backward views of one system, with honest quantities.

- **Invert** over **selected** subgraphs, with exclusions composing with **invert** to express stop-gradients.
- The structural graph establishes which addressed routes are available; it does not by itself establish a Shannon quantity.
- A channel interpretation MAY assign random variables to addressed ports and conditional channel behavior to blocks, then query information from an explicit source port to explicit terminal ports.
- A Shannon claim MUST identify its source distribution, channel or estimator, quantity, and evidence context. An approximate percentage MUST distinguish normalized terminal retention from a route-allocation attribution.
- Confidence qualifies an estimated claim and records how it was obtained; it is not probability mass in the channel and does not turn an uncomputed quantity into an exact bit count.
- Declared and derived structural reach may remain a separate check from Shannon information; the two must not be conflated.

### Hyperparameters

Every dial attached to the structure it modulates.

- **Decorate** on any **address** with typed, domained, defaulted values.
- Schedules as declared functions of symbolic variables — data describing intended variation, with no evaluation against a run defined anywhere in the format.
- Coverage as a **check** over dial values attached by **decorate**: no dial without an **address**.

### Placement and bandwidth

Where structure lives and what crosses links.

- **Decorate** placement on **addresses**.
- Collectives as **elements** defined in this **layer**.
- Bytes-on-wire as **projections** combining the shapes **schema** with placement.

### Cost

Compositional, symbolic accounting.

- **Projections** with symbolic arithmetic over declared axes, so "cost at 8× width" is a substitution.
- Aggregation over **groups** so cost composes: a **group**'s cost is the sum over its members.

### Technique provenance

What each part is, and whether it is new.

- **Decorate** on **groups**: citations, stated assumptions, or a novel flag.
- The novelty surface as a **check** over provenance values attached by **decorate**, expected empty: every **group** lacking provenance.

### Mode

The same system, per regime.

- **Select** per mode: each mode is a **layer** **selecting** its own view.
- Mode-local structure as **elements**: an inference-only cache is defined in the **layer** that views it, invisible to training views.
- Alternatives as structure: variant implementations coexist at a site all modes can see; a mode's **layer** **selects** one.

## Non-goals

- Not bound to any ML stack.
