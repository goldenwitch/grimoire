# Projection language design

Status: in progress.

This is the sibling language that defines a **layer**'s **projection**. It is not a runtime language: it describes the represented system statically.

## Evaluation order

A **projection** has one ordered evaluation:

```text
select + invert
    ->
decorate
    ->
checks
```

Structural evaluation completes before finalization begins. Finalization completes before **checks** begin.

## Structural evaluation

### Select

**Select** defines a **layer**'s **elements**. It may generate **elements**, take references to existing **elements**, or both.

A generated **element** is an ordinary definition at the **layer**'s **definition site**. It supplies everything required of any other **element** definition and obeys the same reference rules.

An explicit **address** list is a valid **select**. Other generated selection forms remain to be specified.

### Invert

**Invert** takes a **group** of **elements** and reverses the direction sign of every **connection** in it.

## Finalization

**Decorate** is the one global finalization phase. It attaches **schema**-governed values after structural evaluation.

Decorations do not feed structural evaluation. They compose the representational account of the finalized structure: configuration, placement, cost, measurement, provenance, and other **schema**-governed facts.

Values may be computed and symbolic in declared axes.

## Checks

A **check** operates only over values attached by **decorate**, with an expected empty or nonempty cardinality.

A **check** is lossy only in its own result. It does not discard or alter finalized structure or finalized values.

## Folding

Structural **reprojections** join references by shared **address** and chain by composition. Each **address** has one **definition site**; a competing definition is invalid. Finalization attaches **extension parameters** to the folded **elements**.

The identity and constraints of finalization values are governed by their **extension namespaces** and **schemas**. The **projection language** adds no second decoration merge rule.

## Deferred

The following are not unresolved **projection language** semantics:

- Concrete syntax for generated **select** forms belongs to grammar construction when a described example requires it.
- The internal value shape and constraints of **decorate** values belong to the **schema** definition format and their consuming **schemas**.
- Concrete symbolic-value syntax and aggregation notation wait for a described value example that requires them.
