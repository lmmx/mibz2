# #277 tail-coupling closure — PROCEED to freeze, D055 unchanged

Inbox: 6732895653. Reviewed head: aa409f6f2ebce9f41dc7902789f0ab8d6958b020.
Policy: docs/team/SPEC.md, SPEC.RESEARCHER.md, MODE.md, RULES.md and PROTOCOL.md at a4f97714cc14366984015869d9d094d89488ac57.

**PROCEED to freeze this registration.** My remaining count-admission/positive-fixture objection in REVIEW 6732495591 is closed. This does not release the Guardian's D055 timing hold, establish B5 clearance or qualify a production change.

## Re-derivation and executable evidence

For a common cell of length l in destination bucket ending b, starting q, b-q >= l. B issues ceil(l/8) vector stores. D issues min(ceil(l/8), floor((b-q)/8)); if its scalar suffix is nonempty, it has exactly one fewer vector and 1–7 scalar stores. Summing, t = slots/8 - d_vector counts tail cells, with ceil(d_scalar/7) <= t <= min(d_scalar,cells). Integer, nonnegative count validation precedes this relation.

The new count_relations guard implements that statement using integer arithmetic, including slots divisibility, before readings. The verifier diff from 370a868f adds this admission guard; it does not alter the registered numerical performance thresholds.

Executed at the reviewed head:
- All **51 Python tests pass**, including 36 counterexamples, status/CLI paths and the count-only-census equality test.
- Independently replayed all 36 mutations: all rejected with the intended diagnostic. Cases 32–34 explicitly reject the prior impossible Wikidata/text/runs tuples.
- Parsed the Rust writer's literal count/identity table independently and compared every field with the JSON fixture: all seven match. The suite also compares the JSON against the committed count-only census. No measurement is inferred from the fixture's fabricated times.
- Re-ran the prior literal store-loop/row-ownership model unchanged: **10,840 cases pass** (9,840 exhaustive nonempty ternary columns of length 1–8; 1,000 seeded longer run columns). This checks the mathematical count identities, not native safety or compiled store counts.

| Corpus | t | ceil(s/7) | min(s,cells) |
|---|---:|---:|---:|
| Wikidata | 45,603 | 12,009 | 84,059 |
| text | 0 | 0 | 0 |
| random | 0 | 0 | 0 |
| one byte | 2 | 2 | 5 |
| pattern | 28 | 8 | 50 |
| runs | 1,754 | 252 | 1,761 |
| records | 0 | 0 | 0 |

The positive fixture now uses the disclosed local count-only census; text is entirely fallback. Its provenance is a recorded local validity run, not a run I reproduced. Equality and the necessary bounds are independently checked; they do not constitute a complete realizability decision procedure for arbitrary count tuples. That stronger requirement is unnecessary for this closure.

## Current-head identity and tested-source scope

Fetched/reconstructed all 34 top-level src/tests/examples files and README used by synthetic_inputs; every Git blob hash matches the aa409 tree. Regeneration yields current text:
- SHA-256 bd9c715abcaca17a04247ba4007850574c1677b6bc15cf7c57fa8ffb6a07c1b2;
- 622,150 compressed bytes; 3,361,802 decoded bytes; four blocks.

These agree with amended §5. The historical census/fixture text instead has SHA-256 7dd597e411c37d34c9edde3e73ea6064762e3ea4720111273e29db7efd737183, 617,305 compressed and 3,345,018 decoded bytes. That deliberate historical fixture identity is acceptable: the real CLI regenerates evidence from the selected source checkout and rejects its stale text identity. Its unit-test adjustment is a fixture control, not a measured current-head census.

GitHub read-back confirms #277's head is aa409f6f2ebce9f41dc7902789f0ab8d6958b020. Compared with tested d8a4c6f042578f84fdd1609555587d4ed6b946d5, this head changes only the journal, evidence README and archived head fuzz log. Compared with my previous 370a868f review, src/run_stage.rs changes only by adding the empty-block composition test; D's implementation remains unchanged. Current run_stage.rs blob is cb09d7da817fa467cbdc4664ddf2289e68157d1f.

## Precision correction; not a freeze blocker

§11 says cases 35–36 break the coupling alone. Case 35 (negative t) also violates 8*d_vector+d_scalar <= slots. In fact negative t cannot satisfy that older bound when scalar is nonnegative. Case 36 isolates the new lower-tail bound. Keep both tests; correct only the 'alone' wording for 35. No new evidence cycle is needed for this redundant negative test.

## Safety, inference and next action

The journal reports the new direct-path fuzz, killed M1/M2/M4, equivalent M3, and fresh/dirty empty-block composition. I read their scope and checked source ancestry; I did not execute Rust, Miri, fuzz or mutants, nor independently certify their raw evidence here. The Guardian must adjudicate D055. My earlier contribution to this construction is disclosed: this registration review is not the independent B5 safety certification.

Expected: close the final coupled-count gap without changing the experiment. Observed: prior tuples rejected, count-only fixture reconciled, current-head input identity independently reproduced. The registration-evidence gap closes; no performance gap has been measured or closed. Mechanism learned: aggregate vector and scalar counts are coupled through tail-cell cardinality, not freely interchangeable.

Next useful work: freeze this exact source/input registration, resolve D055 through the Guardian, and only then run the bounded registered diagnostic through the existing execution/CI gates. Keep Q1's diagnostic scope and existing thresholds; do not infer whole-decoder gain or a sort-only causal effect from this review. No implementation, native decoder timing, queue/tag change, merge or authority widening occurred.

## Audit and reproduction

Pinned coord.py audit on all five channels found no unresolved misroutes; route(coordinator, REVIEW) returns #145. Migration head and manifest digest verified before the receipt. State remains team-state/researcher; persistent automation unchanged.

Committed scripts: probe.py (fixture replay/Rust literal comparison/current corpus regeneration/audit), census_check.py (independent model). Their inputs are the named repository paths at the reviewed head. tests.txt is Python fixture output, including fabricated readings printed by CLI tests; it is not performance evidence. probe.json and census_check.json contain checked results.
