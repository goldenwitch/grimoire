# Grimoire concrete grammar

Status: draft for grammar/review.

This document is the candidate concrete grammar for the Grimoire description
format. The normative semantic source is [grimoire.md](../spec/grimoire.md).
The schema-body decisions are in [schema-format.md](../proposals/schema-format.md)
and the concrete schema inventory is in
[schema-inventory.md](../proposals/schema-inventory.md). Projection semantics
are in [projection-language.md](../proposals/projection-language.md).

The grammar is written as EBNF plus semantic constraints. A production is not a
new semantic requirement: it is a spelling for a requirement already present in
the source documents. This draft must be reviewed against those documents
before the validator or a producer is implemented.

## Design Boundary

The format has three document forms:

- a bundled description document containing one description, its core graph, and
  zero or more layers;
- a standalone layer document containing one layer and the description address
  it belongs to; and
- a schema document containing one versioned schema body.

The bundled form is useful for validation and cut extraction. The standalone
layer form preserves the layer's one-file human viewport. Both forms parse into
the same description and layer values after the surrounding description is
provided. A standalone layer without its referenced description is an
unresolvable document, not a partial description.

This packaging choice is a draft implementation boundary. It does not make a
layer an element and does not give it an address. The description itself is the
addressed element; a layer has a unique name within that description.

## Lexical Conventions

The grammar is UTF-8 text. The ASCII spellings shown here are keywords and
punctuation. Text values may contain Unicode according to the string production.

Whitespace is insignificant between tokens. A line comment begins with `#` and
runs to the end of the line. Comments are not part of recognized parsed values
and are not emitted by canonical serialization. When an extension namespace is
unrecognized, the complete extension-parameter span is opaque and its source
comments and whitespace are retained as part of that span.

```ebnf
letter       = "A".."Z" | "a".."z" ;
digit        = "0".."9" ;
nonzero      = "1".."9" ;
hex          = digit | "A".."F" | "a".."f" ;

identifier   = letter , { letter | digit | "_" | "-" } ;
segment      = ( letter | digit | "_" | "-" ) ,
               { letter | digit | "_" | "-" } ;
address      = "@" , segment , { "/" , segment } ;
version      = digit , { digit } , "." , digit , { digit } , "." , digit , { digit } ;
integer      = "-" , nonzero , { digit } | "0" | nonzero , { digit } ;
number       = integer , [ "." , digit , { digit } ] ,
               [ ( "e" | "E" ) , [ "+" | "-" ] , digit , { digit } ] ;
string       = '"' , { string-character | escape } , '"' ;
escape       = "\\" , ( '"' | "\\" | "/" | "b" | "f" | "n" | "r" | "t"
                         | "u" , hex , hex , hex , hex ) ;
uri-string   = string ;
```

`string` uses the JSON string escape set. `number` is accepted only when it is
finite; infinities and NaN have no lexical production. The semantic validator
applies the positive-integer, finite-number, and provisional boolean
`finite-scalar` refinements from the schema format after parsing.

Addresses are flat identifiers. The slash is part of an address, not a nesting
operator. Address comparison is exact and case-sensitive.

## Document Forms

```ebnf
document       = description-document
               | layer-document
               | schema-document ;

description-document = "grimoire" , version , description , end-of-file ;
layer-document       = "grimoire-layer" , version , layer-file , end-of-file ;
schema-document      = "grimoire-schema" , version , schema-definition , end-of-file ;
end-of-file          = ? no non-comment token ? ;
```

The version after `grimoire` is the format grammar version. The version carried
inside a description is its declared core spec version and is separate from the
syntax version.

## Description and Core Graph

```ebnf
description      = "description" , address , [ string ] , "{" ,
                   "core-spec" , version , ";" ,
                   description-extension-block , core-graph ,
                   { layer } , "}" ;

description-extension-block = [ extensions ] ;

core-graph       = "core" "{" ,
                   { block-definition },
                   { connection-definition },
                   { group-definition },
                   "}" ;

block-definition = "block" , address , string , "{" ,
                   { port-definition },
                   [ extensions ],
                   "}" ;

port-definition  = "port" , address , [ string ] ,
                   [ extensions ] , ";" ;

connection-definition = "connection" , address , address , "->" , address ,
                        [ extensions ] , ";" ;

group-definition = "group" , address , [ string ] , "{" ,
                   [ address-list ],
                   [ extensions ],
                   "}" ;

address-list     = address , { "," , address } , ";" ;
```

A block's nested ports are definitions at the block's containing definition
site, not a new layer. A port belongs to exactly the block that contains its
production. A block has a required human name; port, connection, and group
labels are optional because the core vocabulary does not require them. A
connection names exactly two port addresses, with the left port as source and
the right port as destination. A group names a set of addressed elements;
group members may include groups.

The grammar does not put a direction field on a port. Direction belongs to the
connection production. A port may therefore participate in connections in
multiple directions if the described graph permits it.

A group may be empty. An empty group is still an addressed element and is not an
error merely because it has no members.

## Layers

```ebnf
layer            = "layer" , string , "{" ,
                   layer-inputs , layer-consumption , projection ,
                   "}" ;

layer-inputs     = "inputs" , "{" ,
                   ( "core" | layer-name ) ,
                   { "," , ( "core" | layer-name ) } ,
                   "}" , ";" ;

layer-name       = string ;

layer-consumption = "consumes" , "{" ,
                    "projection-language" , version , ";" ,
                    "schemas" , "{" ,
                    { schema-use },
                    "}" ,
                    "}" ;

schema-use       = uri-string , "/" , identifier , "@" , version , ";" ;
```

Layer names are unique within one description. A layer's declared inputs name
the core graph and other layers; duplicate inputs are invalid. The input graph
must be a DAG and every named layer must exist.

The `core-spec` value is declared by the enclosing description. The
`projection-language` value is the version of the sibling projection language
consumed by the layer. Schema uses name a namespace URI, local schema name, and
schema version. The concrete separator in `schema-use` is provisional and must
be checked against namespace URI grammar during review.

A standalone layer document uses the following wrapper:

```ebnf
layer-file       = "for" , address , ";" , layer , end-of-file ;
```

The `layer` production in a standalone file is the same production as in a
bundled description. Its inputs may refer only to layers in the surrounding
description. A standalone layer cannot validate its references without that
surrounding description.

## Extensions

```ebnf
extensions       = "extensions" , "{" , { extension-parameter } , "}" ;

extension-parameter = "extension" , uri-string , identifier ,
                      "schema" , identifier , "@" , version ,
                      "=" , value , ";" ;
```

An extension parameter is attached to the nearest enclosing addressed element.
The description, block, port, connection, and group productions may carry an
extension block. A layer is not an element, so the bundled layer wrapper does
not acquire extension parameters through this production.

The namespace URI is an exact identifier. It is validated as an absolute
`https` URI by the namespace rule, but it is not dereferenced. In the prototype,
the string payload must begin with `https://`, contain a nonempty authority,
and contain no whitespace or control characters. Schema name and version are
subordinate to the namespace and are not global names.

For a namespace recognized by the consumer, the parameter is parsed as a value
and validated against the named schema. For an unrecognized namespace, the
consumer retains the complete source span of the extension parameter as opaque
data. The serializer emits that span byte-for-byte, including comments and
insignificant whitespace inside the span. This opaque-span exception is the
only place where canonical serialization retains source formatting; it is a
parser and serializer obligation in addition to the grammar production.

The exact raw-byte boundary for an unknown parameter remains a review point:
the current candidate boundary is the first `extension` keyword through its
terminating semicolon, including all bytes between them.

## Projection

The projection grammar fixes the global phase order by making the stages
separate, ordered sections. A projection cannot put a decoration before
structural sections or a check before decoration.

```ebnf
projection       = "projection" , "{" ,
                   select-stage ,
                   [ invert-stage ],
                   [ decorate-stage ],
                   [ checks-stage ],
                   "}" ;

select-stage     = "select" , "{" ,
                   { select-reference | generated-definition },
                   "}" ;

select-reference = "use" , address-list ;

generated-definition = block-definition
                     | connection-definition
                     | group-definition ;

invert-stage     = "invert" , "{" ,
                   { "group" , address , ";" },
                   "}" ;

decorate-stage   = "decorate" , "{" ,
                   { decoration },
                   "}" ;

decoration       = "on" , address , extension-parameter ;

checks-stage     = "checks" , "{" ,
                   { check },
                   "}" ;

check            = "check" , identifier , "expect" ,
                   ( "empty" | "nonempty" ) ,
                   "over" , decoration-selector , ";" ;

decoration-selector = uri-string , identifier ;
```

`use` is the explicit address-list form of `select`. A generated definition is
an ordinary element definition at the current layer's definition site. It must
supply all required fields and obey the same reference and locality rules as a
core definition or any other layer definition.

A generated definition may reference an existing element in a group member
list, a connection endpoint, or an address-valued extension field. Every such
reference is resolved from the current layer's visible input chain or from an
element defined at the current layer's definition site; textual definition
order does not change visibility, and generated structure cannot see finalized
decoration values. Generated addresses participate in the same single global
address-uniqueness check as authored addresses; a duplicate is a C1 failure
rather than a second definition. This follows the pure and static projection
requirement: structural evaluation depends on declared inputs and structural
definitions only.

`invert` names groups. It reverses the direction sign of every connection in the
selected group. Exclusions are represented by the selected group membership and
selection result; the projection language adds no separate stop-gradient
primitive in this draft.

`decorate` attaches extension parameters after all `select` and `invert`
evaluation. `decoration` uses an extension parameter's ordinary namespace,
parameter, schema, version, and value form. A decoration target must be a
folded element address visible to the layer. It resolves from the layer's
declared input chain or from an element defined at the layer's own definition
site; an out-of-scope target is a validation failure under the ordinary
reference rules.

A check names an expected cardinality and selects only finalized decoration
values. The check result is not an element and does not discard or alter the
finalized structure or values. An empty result is valid when the expected
cardinality is `empty`; a nonempty result is valid when it is `nonempty`.

The concrete syntax for a check's selector is provisional. It names one
namespace and parameter. The selector aggregates all finalized values with that
namespace and parameter on folded elements visible to the layer. Its semantic
restriction is fixed: it cannot inspect structural data except through values
attached by `decorate`.

## Values and Schema Expressions

### Runtime Values

```ebnf
value            = string
                 | integer
                 | number
                 | enum-value
                 | tagged-value
                 | product-value
                 | sequence-value
                 | address-value
                 | "absent"
                 | "present" , "(" , value , ")" ;

enum-value       = identifier ;
tagged-value     = identifier , "(" , value , ")" ;

product-value    = "{" , [ field-value , { "," , field-value } ] , "}" ;
field-value      = identifier , ":" , value ;

sequence-value   = "[" , [ value , { "," , value } ] , "]" ;
address-value    = "ref" , "(" , address , ")" ;
```

The schema selected for an extension parameter determines whether a text,
integer, number, enum tag, product, sequence, address reference, or presence
value is valid. An untyped bare `identifier` is not accepted as a runtime value
unless the selected schema expects a closed enumeration.

Products are semantically labeled, not positional. Sequences are ordered and
homogeneous according to their schema. `absent` and `present(T)` are explicit;
absence is not represented by a missing product field when the schema declares
the field. A tagged alternative is represented as `tag(value)` and selects
exactly one arm named by `tag`; the selected value must validate against that
arm's schema.

The serializer emits product fields in schema declaration order and sequence
items in value order. It emits a closed enumeration using its schema spelling.
Address references are emitted using the exact address string.

### Schema Documents

```ebnf
schema-definition = "schema" , "{" ,
                    "namespace" , uri-string , ";" ,
                    "name" , identifier , ";" ,
                    "version" , version , ";" ,
                    "allows" , "{" ,
                    element-kind , { "," , element-kind },
                    "}" , ";" ,
                    "value" , schema-expression , ";" ,
                    "}" ;

element-kind     = "description" | "block" | "port" | "connection" | "group" ;

schema-expression = scalar-expression
                  | product-expression
                  | sequence-expression
                  | alternative-expression
                  | reference-expression
                  | presence-expression ;

scalar-expression = "finite-scalar"
                   | "positive-integer"
                   | "finite-number"
                   | "text"
                   | "enumeration" , "{" ,
                     identifier , { "," , identifier },
                     "}" ;

product-expression = "product" , "{" ,
                     field-schema , { "," , field-schema },
                     "}" ;
field-schema      = identifier , ":" , schema-expression ;

sequence-expression = "sequence" , "<" , schema-expression , ">" ;

alternative-expression = "alternative" , "{" ,
                          alternative-arm , { "," , alternative-arm },
                          "}" ;
alternative-arm   = identifier , ":" , schema-expression ;

reference-expression = "address-reference" ;
presence-expression  = "presence" , "<" , schema-expression , ">" ;
```

A schema document declares one namespace, local name, version, allowed element
kinds, and one value expression. The schema body is closed: no arbitrary
keyword or external dialect is admitted by this grammar.

The bootstrap schema is represented by the same productions. The validator
loads the bootstrap contract before validating ordinary schema documents; it
does not invoke a second schema language.

The exact production and meaning of `finite-scalar` is a review gap inherited
from the schema-format proposal. It must be resolved before a schema validator
can claim complete bootstrap conformance.

## Canonical Serialization

Canonical serialization is part of the format, not an implementation preference.
The following ordering is the candidate canonical order.

### Documents

1. Emit the syntax header and version.
2. Emit the description, layer, or schema header.
3. Emit each fixed section in grammar order.
4. Emit one final newline.

Comments and insignificant whitespace are omitted outside opaque unknown
extension-parameter spans. Strings use the shortest valid JSON escape spelling
accepted by the serializer. No URI is dereferenced or normalized.

### Description

1. Emit the description address and optional human label.
2. Emit description extensions in namespace, parameter, and schema-version
   order, except that unknown namespace parameter spans retain their exact
   source bytes and relative order.
3. Emit the core graph.
4. Within the core graph, emit blocks ordered lexicographically by address;
   ports inside each block ordered lexicographically by address; connections
   ordered lexicographically by address; and groups ordered lexicographically
   by address.
5. Emit layers ordered lexicographically by name.
6. Within each layer, emit inputs in the order `core` followed by lexicographic
   layer name; schema uses by namespace URI, schema name, and version; and
   projection sections in their fixed structural order.

The core graph's element ordering is serialization only. It does not change
whether a reference can see a definition. Definition site and visibility are
validated from the enclosing core or layer.

### Values and Schemas

Known products use schema field order. Known alternatives use schema arm order.
Sequences preserve order. Known extension parameters use namespace, parameter,
and schema-version order. Unknown extension parameter source spans are emitted
without parsing or normalization, including their internal comments and
whitespace. Unknown spans retain their relative order among unknown spans;
canonical ordering of known parameters does not rewrite their bytes.

Canonical serialization must satisfy:

```text
parse(serialize(parse(x))) == parse(x)
serialize(parse(x)) == serialize(parse(serialize(parse(x))))
```

The first equality is parsed-value preservation; the second is the serializer
fixpoint expressed without assuming that an input was already canonical.

## Semantic Constraints Carried by Validation

The grammar alone does not encode all structural requirements. The validator
applies these constraints after parsing:

- every addressed element, including the description, has one unique address;
- every connection is directed and joins exactly two existing ports;
- the description declares its core spec version;
- every element has exactly one definition site;
- every definition has the grammar-defined form for its site;
- references occur only at their definition site or above it;
- referenced definitions satisfy the maximal-site locality rule;
- unreferenced elements are legal unless an explicit finalization check reports
  them;
- declared layer inputs form a DAG and resolve to existing layers or core;
- layer input names are unique within each layer;
- extension namespaces are qualified, recognized values validate against their
  schema, and unrecognized payloads are preserved; and
- a cut extracts a standalone description that revalidates after erasure.

A non-cut subset is not treated as an empty description. The validator reports
which layer has an absent declared input and returns an unresolvable outcome.

## Draft Decisions for Review

This grammar draft records these provisional architecture decisions:

- A block, port, connection, or group is the same addressed element whether it
  is authored in the core or generated in a layer.
- A layer name is not an element address; addresses remain flat across core and
  layer definition sites.
- Nested ports inherit the containing block's definition site.
- Explicit address lists are the minimal valid `select` form.
- Generated `select` definitions use ordinary block, connection, and group
  productions.
- Projection section order makes select and invert precede decorate and checks
  syntactically.
- Architecture facts, shapes, training settings, execution regimes, precision,
  measurements, provenance, and candidate lineage are extension values.
- Unknown namespace payloads are opaque source spans, not guessed values.
- Bundled descriptions and standalone layers are serialization forms of the
  same semantic model.
- Canonical ordering is deterministic, but it does not change definition-site
  visibility or structural identity.

These are grammar decisions, not additions to the frozen core vocabulary.

## Review Gaps

### Layer-file packaging

The draft supplies both bundled and standalone forms, but the exact manifest
that tells a consumer where to find standalone layers is not specified. The
reference implementation can validate a bundled description first; standalone
layer loading remains a packaging boundary.

### Namespace URI production

The namespace rule requires an absolute `https` URI, but the exact URI lexical
production and the treatment of escaped string bytes must be fixed before
unknown namespace preservation can be tested byte-for-byte.

### Indexed structural generation

The draft supports explicit addresses and ordinary generated definitions. It
does not yet provide a compact indexed form for V-JEPA 2-AC block-causal
attention, Show-o mixed attention, dynamic visual tiles, or long token streams.
The first fixture should expand a small instance into ordinary connections
before a compact production is considered.

### Parameter-update relations

EMA, freeze, fine-tuning, continual updates, deltas, and merges are represented
as extension values or candidate lineage values. The grammar does not invent an
activation connection for them. A future structural relation requires a fixture
that demonstrates validator-level necessity.

### Symbolic values and cost

The runtime value grammar contains literals and references but no symbolic
arithmetic. Cost expressions over declared axes remain a projection-language
and schema decision, not an unreviewed grammar addition.

### Shannon channel semantics

The concrete grammar does not yet serialize channel kernels, source
distributions, information queries, or uncertainty claims. Those are defined by
the information-flow proposal as a semantic interpretation over selected
addressed structure. The first executable regime is finite, discrete, and
acyclic, with finite-horizon unrolling for recurrent examples; continuous
estimators, fixed points, and route-attribution methods remain explicit
follow-up decisions.

### Finite scalar kind

The schema-format proposal names `finite scalar kind`, but its exact value set
is not defined by this draft. Bootstrap validation must stop at this discrepancy
rather than silently treating it as an enum or boolean.

## Initial Grammar Fixtures

The grammar review should begin with these small documents:

1. A minimal valid description with one addressed description, one block, two
   ports, one directed connection, one group, and no layers.
2. A description with one core axis and shape extension values on ports.
3. A description with one layer that uses a core address and defines one local
   generated block.
4. A two-layer chain with one layer consuming the other.
5. A layer whose `select`, `invert`, `decorate`, and `checks` sections are in
   canonical order.
6. A schema document for `axes/1` using product, text, and presence.
7. A schema document for `shapes/1` using sequence, alternative, positive
   integer, and address reference.
8. A known extension value and an unknown namespace payload in one description.
9. A malformed connection, duplicate address, below-scope reference, cyclic
   layer input, and unavailable schema version.
10. A V-JEPA 2 boundary fixture with a shared encoder, pretraining predictor,
    action-conditioned predictor, and two cuts.

The fixtures must be reviewed against [v-jepa-2-case-studies.md](../proposals/v-jepa-2-case-studies.md)
and [frontier-architecture-case-studies.md](../proposals/frontier-architecture-case-studies.md)
so the grammar is tested against actual architecture structure rather than an
invented toy vocabulary.
