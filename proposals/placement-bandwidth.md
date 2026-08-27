# Placement and bandwidth layer

Status: prototype contract; in progress.

This proposal records the smallest placement view needed by the observed
architecture cases. It uses finalized placement values on existing addressed
elements, keeps communication collectives as ordinary addressed blocks, and
derives wire traffic only from explicit shapes and placement assignments.

The governing sources are [grimoire.md](../spec/grimoire.md),
[architecture-vocabulary.md](./architecture-vocabulary.md),
[shape-schema.md](./shape-schema.md), and the observed case studies in
[frontier-architecture-case-studies.md](./frontier-architecture-case-studies.md)
and [v-jepa-2-case-studies.md](./v-jepa-2-case-studies.md).

## Placement values

The `placement/1` value is:

```text
product{
  location: text
}
```

It is a finalized value attached by `decorate`. A location is an opaque author
label such as `gpu-0`, `host-a`, or `robot-controller`; the prototype does not
resolve devices, hosts, links, or topology from that spelling.

An endpoint may receive a direct placement. When a port has no direct value,
the prototype may use the placement on its owning block. This is an explicit
structural ownership lookup, not inference from the port label. Missing
placement remains an error when a report needs the endpoint.

## Collectives

The frozen core vocabulary has no collective element kind. A placement layer
therefore selects an ordinary addressed block for a collective and supplies a
typed report record with:

- the collective block address;
- the payload shape address; and
- an explicit list of participant-to-participant transfers.

The transfer list is intentional. The prototype does not infer all-reduce,
all-gather, broadcast, topology, message fragmentation, or link contention
from a name such as `all-reduce`. A future collective schema can add those
semantics when an observed case requires them.

## Bytes on wire

For a directed connection whose endpoint locations differ, the report uses the
shape of the source port and records one transfer. A collective uses the same
rule for each explicit cross-location transfer. Same-location relations add no
wire traffic and therefore do not require a shape in the report.

The prototype shape used by the report has ordered literal or addressed-axis
dimensions and an explicit positive `bytes_per_element`. A literal dimension is
multiplied directly; an axis dimension requires a supplied extent binding.
Checked multiplication and addition make overflow visible. The existing
`shapes/1` schema intentionally does not infer element width from
`precision/1`, dtype text, or model family, so a caller must provide that width
for a byte calculation.

This is a static accounting result. It does not claim measured bandwidth,
latency, throughput, collective algorithm cost, or network utilization. Those
observations remain `measurement/1` values with their own source records.

## Observed-paper fit

The view is sufficient to exercise the recurring boundaries in the corpus:

- a bridge or projector can be placed separately from its visual encoder;
- a V-JEPA 2 action-conditioned predictor can be placed apart from visual and
  controller endpoints;
- a planning controller can be named as an external placement boundary; and
- BitNet-like precision variants can retain the same logical graph while a
  deployment placement view records where the operator resides.

The view does not decide whether an operator's precision changes its structural
interface. That remains the architecture and precision boundary already
recorded elsewhere.

## Sized gaps

- Device identity, network topology, collective algorithms, and contention are
  not defined by `placement/1`.
- Shape-to-wire accounting requires explicit element width and axis extents;
  no precision-to-byte mapping is inferred.
- Placement decoration parsing is available in the Rust prototype, while the
  report record is still a host-side analysis input rather than a new grammar
  production.