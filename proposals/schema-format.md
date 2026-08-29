# Schema definition format

## Ruling

A **schema** body is a closed Grimoire-native algebra. Every constructor, constraint, and validation behavior is explicitly defined by the format. A **schema** body does not delegate arbitrary semantics to an external dialect or extension keyword.

This is the boundary that keeps invalid value states out of the represented system rather than asking every consumer to interpret an open-ended **schema** language.

## Inherited contract

- A **schema** is versioned and machine-checkable.
- A **schema** defines the structure and constraints of **extension parameters** in one **extension namespace**.
- **Decorate** attaches **schema**-governed values only after structural evaluation.
- **Extension namespace** naming is owned by **extension namespace** minting, not by this **schema** body design.
- Producer and consumer must be able to validate the same value against the same **schema** without depending on the producer implementation language.

## Prior art signals

- JSON Schema separates a meta-schema, vocabularies, and arbitrary extension keywords. Its modularity is useful prior art, but the open vocabulary mechanism is outside this ruling.
- Protobuf gives fields stable numeric identity and evolves by reserved identifiers. Its compatibility discipline is useful, but Grimoire has not chosen a binary wire format or field tags.
- Avro defines a closed type algebra and canonical form. Its content identity and fingerprinting are useful input to **extension namespace** minting, not a decision about **schema** constructors here.
- The archived tape contract stamps a version and rejects producer/consumer mismatch. This is evidence for fail-visible validation rather than fallback interpretation.

## First-case test matrix

The initial algebra is adequate only if these value shapes require no escape hatch:

| Consumer **schema** | Required value shape |
| --- | --- |
| Axes | symbolic declaration, **absent | present(text)** description, and an **address** reference form |
| Shapes | ordered dimensions: positive literal or axis reference |
| Measurement | literal value, unit, and source record |
| Provenance | citations, assumptions, and novelty state |

## Closed initial algebra

Every **schema** declares the **element** kinds to which its values may attach and one value expression built from these constructors:

- scalar with closed refinements;
- labeled product fields;
- homogeneous sequence;
- tagged alternative;
- **address** reference;
- explicit presence: **absent | present(T)**.

An **address** reference is a first-class constructor. It validates an **address** reference into the **description**; individual **schemas** may further constrain the referenced declaration.

The initial closed scalar refinements are:

- finite scalar kind, provisionally the boolean values `true` and `false`;
- positive integer;
- finite number;
- text;
- closed enumeration.

Richer constraints are expressed by composing these constructors inside an individual **schema**. They do not add meta-algebra features.

For the executable implementation, `finite scalar kind` is the closed boolean
kind. This is a reversible engineering choice made to give the bootstrap
validator one executable meaning for the previously undefined constructor. A
fixture must reject non-boolean values and preserve boolean values through
canonical serialization. If the empirical case set later requires another
finite scalar kind, the fixture is the trigger for revising this contract.

## Bootstrap

The **schema** definition format offers the necessary bootstrap productions to describe and validate these same constructors. It does not invoke a second **schema** language or an open escape hatch.

Bootstrap productions are part of the closed initial algebra. They validate the **schema** declaration, its allowed **element** kinds, and its value expression using the same constructors the declaration makes available to later **schemas**.