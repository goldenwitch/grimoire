# Extension namespace minting

This proposal defines how an author names an extension namespace so independent
authors do not collide unknowingly. It also records the preservation boundary
needed by the parser and canonical serializer.

The core requirement is in [grimoire.md](../spec/grimoire.md): every extension
parameter is qualified by an extension namespace, and namespace identifiers
must be constructed so independent authors cannot collide unknowingly. The
closed schema algebra is described in [schema-format.md](./schema-format.md).
This proposal does not add a new element kind or schema constructor.

## Minting Rule

An extension namespace identifier is an author-controlled absolute `https` URI.
The author chooses a path below an origin it controls, and owns the obligation
to keep the path unique within that origin.

Illustrative identifiers:

```text
https://example.org/grimoire/extension/architecture
https://github.com/example/project/grimoire/extension/shapes
https://research.example.edu/lab/model/extension/measurements
```

The origin is the collision boundary. Two independent authors controlling
different origins cannot unknowingly mint the same absolute URI. A shared
organization or project origin is one authoring domain for this rule; that
organization must allocate unique paths below it.

The identifier is compared as an exact valid URI string. Grimoire does not:

- dereference the URI;
- resolve redirects or DNS records;
- consult a registry;
- normalize aliases, case, or percent-encoding; or
- infer ownership from a URL at validation time.

Those choices keep namespace validation deterministic and offline. They also
make an accidentally duplicated spelling visible instead of silently merging
two namespaces that only appear equivalent after external normalization.

## Schema Identity

A schema belongs to one extension namespace and carries its own local name and
version. The conceptual identity is:

```text
(namespace URI, schema name, schema version)
```

The concrete spelling of that identity belongs to grammar construction. The
namespace URI must not be repeated as an unqualified schema name or parameter
key. A schema version changes the governed value contract; consumers must not
fall back to a different version when the requested one is unavailable.

Namespace identity and schema identity have different lifetimes:

- the namespace URI names the author's extension vocabulary;
- the schema name identifies one contract in that vocabulary; and
- the schema version identifies the exact contract a value uses.

A new schema contract gets a new version under the same namespace when the
namespace remains the correct owner. A different owner or independent
vocabulary gets a different namespace URI.

Compatibility policy for schema versions is not decided here. The validator
must still report an unavailable or incompatible requested version visibly
rather than interpreting another version by guesswork.

## Parameter Ownership

An extension parameter is addressed by its namespace URI and a name governed by
a schema in that namespace. The namespace owns the meaning of the parameter;
the core language owns only its qualification and preservation rules.

A consumer that recognizes the namespace may validate the parameter against the
referenced schema. A consumer that does not recognize it must retain:

- the exact namespace identifier;
- the exact parameter name;
- the exact schema/version reference; and
- the exact parameter value bytes accepted by the grammar.

Unknown extension data is opaque to the consumer. It is not parsed into a
second guessed dialect and is not reformatted by the canonical serializer.
Known data may be parsed and emitted canonically according to its reviewed
schema and grammar.

This creates two serializer obligations:

1. recognized values serialize canonically;
2. unrecognized namespace payloads survive parse and serialize byte-exactly.

The second obligation is deliberately stronger than semantic equivalence. It
allows a consumer to inspect and re-emit a description without damaging data it
cannot interpret.

## Authoring Practice

A namespace author should keep a small public record at the namespace origin
that names:

- the owner or project;
- the namespace's purpose;
- the schemas currently under it;
- each schema version and its closed value contract; and
- the date or revision at which a version was minted.

The record is provenance and documentation, not a runtime dependency. A
validator does not fetch it. Its purpose is to make ownership and allocation
human-auditable when two projects exchange descriptions.

The reference implementation uses this namespace root:

```text
https://github.com/goldenwitch/grimoire/extension
```

Schema families use paths below this root, for example
`https://github.com/goldenwitch/grimoire/extension/shapes`. A published schema
contract should keep one owned root and must not mix alternate spellings.

## Collision Cases

The following are distinct namespaces because exact URI identity is used:

```text
https://example.org/model/extension/shapes
https://example.org/model/extension/architecture
https://other.example/model/extension/shapes
```

The first two share an owner but have different allocated paths. The third has
a different owner even though its path suffix is the same.

The following are not acceptable namespace identifiers under this proposal:

```text
shapes
example.org/model/extension/shapes
urn:example:shapes
http://example.org/model/extension/shapes
```

A bare name has no authoring boundary, a relative-looking host is ambiguous,
URN allocation is not the selected minting rule, and `http` does not meet the
required `https` form.

## Architecture Fit

The namespace rule supports the empirical architecture cases without making
their names core vocabulary:

- `architecture/1` can carry model family and operator facts from V-JEPA 2,
  Janus, Qwen2-VL, BitNet, and other papers.
- `shapes/1` and `axes/1` can carry dimensions and symbolic references for
  video, speech, latent, action, and token interfaces.
- `training/1` and `execution/1` can carry stage and runtime-boundary facts.
- `measurement/1` and `provenance/1` can carry sourced results and citations.
- `lineage/1` can remain a candidate until parameter-state relations are ruled.

The paper name, model family, or benchmark does not become a namespace merely
because it appears in a value. Namespaces identify owners of extension
contracts, not topics in the corpus.

## Fixtures

The namespace and serializer fixtures should include:

- two valid namespaces with different controlled origins;
- two valid namespaces with the same origin and distinct allocated paths;
- a duplicate exact namespace used by two parameters, which joins one
  namespace rather than creating two;
- invalid relative, non-`https`, and malformed identifiers;
- an unrecognized namespace with values containing ordering, quoting, and
  whitespace that the consumer must preserve exactly;
- a recognized namespace whose schema version is unavailable, which fails
  visibly;
- a known parameter and an unknown parameter in the same description; and
- canonical serialization of a description containing both known and unknown
  namespace payloads.

The fixture must prove that an unknown value is not silently dropped, parsed by
an invented dialect, or normalized by a serializer that cannot recognize it.

## Decision Record

This proposal records these architecture and format decisions:

- Namespace collision resistance comes from an author-controlled absolute
  `https` URI and a unique path below that origin.
- The reference implementation namespace root is
  `https://github.com/goldenwitch/grimoire/extension`; it is provisional until
  repository ownership is verified.
- Namespace identity is exact and offline; no network lookup or alias
  normalization participates in validation.
- Schema name and version are subordinate to namespace identity and are not
  global bare names.
- Unknown namespace payloads are opaque and byte-preserved through parse and
  serialize.
- Recognized schema values are validated by the closed schema algebra and
  emitted canonically.
- Paper, model, benchmark, and architecture names remain values or provenance,
  not namespace identifiers.

## Sized Gaps

### URI grammar and canonical spelling

- Implementation boundary: the parser accepts a restricted ASCII `https` URI with
  exact-string identity, and malformed-identifier fixtures cover the current
  failure behavior.
- Remaining gap: a public grammar may still need a fuller URI production and a
  compatibility policy for escaped string bytes.
- Candidate shape and rough size: a versioned URI production with exact-string
  identity and serializer preservation. This is one grammar production plus
  malformed-identifier fixtures.
- Entry trigger: a public schema contract requires URI forms beyond the
  implementation restriction.

### Unknown-value byte boundary

- Implementation boundary: parser and serializer fixtures preserve the complete
  unknown parameter span from `extension` through its terminating semicolon,
  including internal comments and whitespace.
- Remaining gap: the public contract may need to refine the exact source-span
  boundary if a future grammar permits nested or streamed extension payloads.
- Candidate shape and rough size: retain the raw source slice for the complete
  unknown parameter payload and emit it without normalization. The current
  parser value type and round-trip fixtures already implement this shape.
- Entry trigger: a concrete grammar change makes the current raw-span boundary
  insufficient.

### Schema version compatibility

- Binds when: a description requests a schema version that differs from the
  consumer's available version.
- Cost of absence now: unavailable versions can fail, but compatible evolution
  and migration are not defined.
- Candidate shape and rough size: fail-closed exact version matching for v1;
  later compatibility policy only after a versioned fixture demonstrates a
  need. This is a validator rule and one failure fixture.
- Entry trigger: `schemas/instances` introduces the first versioned concrete
  schema.
