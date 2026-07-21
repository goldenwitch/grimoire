# tools/validator: Reference validator and canonical serializer

Guidance for a mechanical pass. The specification (grimoire.md) is normative; the grammar document (artifact of grammar/build, accepted at grammar/review) defines the concrete syntax. The validator conforms to both and is never the definition of either.

## Objective
A tool that parses descriptions, checks every structural MUST in the specification, extracts cuts, and emits the canonical serialization.

## Inputs
- The reviewed grammar document.
- grimoire.md.
- Implementation language: as named at dispatch. Do not choose one yourself.

## Steps
1. Implement a parser exactly per the grammar. No error recovery, no accepted dialects: input either parses or is rejected with a location.
2. Implement the structural checks. Each check cites its spec section and has the fixtures appropriate to its outcome:
   - C1. Every element carries an address, unique within the description.
   - C2. Every connection is directed and joins exactly two ports.
   - C3. The description declares its core spec version and instance revision.
   - C4. Every element has exactly one definition site.
   - C5. Each element conforms to its grammar-defined form at its definition site.
   - C6. Every reference to an element occurs at its definition site or in a layer above it.
   - C7. Locality: each referenced element is defined at a maximal site among those at-or-below all of its references. Ties: accept any maximal legal site; never warn on an authored tie.
   - C8. Unreferenced elements may sit at any site; report them only via the dead-structure query, never as errors.
   - C9. Layer declared inputs form a DAG and every declared input resolves.
   - C10. Extension parameters: namespace present; preserve unrecognized namespaces byte-exact through parse and serialize.
   - C11. Cut extraction: given a downward-closed subset of layers, emit it as a standalone description and re-validate it (erasure).
   - C12. A non-cut subset is reported as unresolvable at the layers whose inputs are absent — a distinct, visible outcome, not a crash and not silence.
3. Implement the canonical serializer per the grammar. Properties: parse(serialize(parse(x))) preserves every valid parsed value, and serialize(parse(x)) is a fixpoint for every valid x.
4. Fixtures: one minimal valid description; failing fixtures for invalid checks; valid query fixtures for C8; successful transformation fixtures for C11; unresolvable fixtures for C12; both serializer properties asserted over all valid fixtures.

## Acceptance
- All fixtures behave as specified.
- Both serializer properties hold on every valid fixture.
- Every check failure message names the check, the location, and the relevant identifier when one exists.

## Do not decide — escalate instead
- Any point where the grammar and the specification disagree, or where either is ambiguous: stop and file the discrepancy. Do not improvise a resolution; ambiguity here propagates into every downstream document.
- Projection evaluation is out of scope. It belongs to the projection language.
- No constraints beyond the specification's MUSTs, however reasonable they seem.
