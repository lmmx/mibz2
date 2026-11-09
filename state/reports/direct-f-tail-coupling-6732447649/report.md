# #277 second closure: requested guards fixed; fixture still violates coupled store accounting

**REVISE, limited to a coupled store-count check and a consistent positive fixture.** Inbox **6732447649**, exact head **370a868f46e4a7326557859043e5e44960e7b592**. Preserve the earlier closures. My previous four bounds were necessary, not sufficient: satisfying them alone does not certify physical feasibility. This review finds a direct B/D coupling, not another independent threshold.

## Verified closures

All four requested bounds are implemented in `count_relations` before readings. The previous four witnesses and all **31** supplied counterexamples fail admission with their expected messages. **50 verifier tests pass**, including the positive fixture and CLI checks. The source-label corrections now distinguish predicted table stores and nonempty buckets from machine instructions and hardware streams. The stale sort-only and same-store claims are corrected. No threshold, timing array, entry implementation, phase accounting, kernel or selector changed in the five-file diff from `7f88388b938528f22088df79387f309eb112fe1c`.

The source-generated text independently regenerates as registered: **3,326,302 decoded bytes, 612,142 compressed bytes, 4 blocks**, SHA-256 **5996a32dcec1cb7899cd2dd2e2cb122a3de7eeead4c2096729575e3aec79e6ff**. The change is prospective and explicit. The fixture's historical text identity is intentionally a separate formula-test input; the CLI test substitutes this checkout's identity. No result is reinterpreted.

## The missing coupled invariant

Let `S=slots_B`, `V=d_vector`, `s=d_scalar`, and `t` be the number of D cells that use any scalar tail store. These count only accepted blocks under the AVX2 condition.

On each cell of length ℓ, B emits `ceil(ℓ/8)` vector stores. D emits `v=min(ceil(ℓ/8),floor((bucket_end−q)/8))` vector stores and `s_cell=max(0,ℓ−8v)` scalar stores. Because the cell ends within its bucket, when the vector loop stops before the cell ends there are **1–7** rows left. Therefore:

- if `s_cell=0`, B and D emit the same number of vector stores;
- if `s_cell>0`, B emits exactly **one more** vector store than D.

Summing gives an exact relation and necessary bounds:

`t = S/8 − V`, and `ceil(s/7) ≤ t ≤ min(s,cells)`.

The existing `S % 8 == 0` check makes `S/8` integral. In particular, the removed written slots obey `0 ≤ S−(8V+s) ≤ 7s`. This relates B and D on their common cells; separate lower bounds on B slots and D store instructions cannot replace it. It also forces `S=8V` when no scalar tails occur.

The current positive fixture passes `check()` with no problems but violates this bound in **three** corpora:

| Corpus | B slots S | D vectors V | D scalars s | Implied tail cells S/8−V | Permitted tail cells |
|---|---:|---:|---:|---:|---:|
| Wikidata | 176,000,000 | 21,500,000 | 60,000 | 500,000 | 8,572–60,000 |
| Synthetic text | 12,800,000 | 1,500,000 | 2,000 | 100,000 | 286–2,000 |
| Synthetic runs | 200,000 | 24,000 | 500 | 1,000 | 72–500 |

For text, the fixture claims 798,000 written slots saved by only 2,000 scalar-tail rows. Their maximum saving is **14,000**. Neither compression variability nor instruction lowering explains it: these fields are explicitly source-loop counts of the same cells.

The row-ownership census model from the preceding review now also checks this coupling. It passes **10,840 columns** (9,840 exhaustive nonempty ternary columns through length 8 plus 1,000 deterministic longer-run columns). This independently constructed grid and literal-loop calculation support the derivation; they do not establish native safety or performance.

## Concrete closure

Add the coupled invariant to admission, keep these three fixture tuples as negative cases, and repair the writer/JSON positive fixture together. Prefer copying a mutually consistent tuple of counts from an untimed validation census over adjusting each number to pass individual inequalities. Preserve that count-only evidence; the deleted 2-round records cannot be independently inspected here. No new timing is needed or requested. A fixture can retain artificial timing arrays, but its counts should come from a coherent census. This is not a request for a second decoder or exhaustive proof of every conceivable count relation.

This defect does **not** reverse Q1: its time ratio remains unchanged. Nor is any real run demonstrated corrupt. It prevents the current claimed feasible fixture and mechanism census admission from being accepted as such. The prior construction, budget, timed path guard, corpus identities, diagnostic 0.95 gate and removal of old controls remain closed. After the count coupling is repaired, this review requests no further changes to the registration's estimand or experiment design.

Expected: four repairs and a feasible positive tuple. Observed: repairs pass, tuple remains impossible because its B/D tails disagree. Performance gap closed: none measured. Mechanism learned: B's lost vector stores count the D scalar-tail cells exactly, so the census admits a much tighter cross-arm sanity check. Next useful work: count-only reconciliation, then the bounded diagnostic under its frozen head and remaining safety conditions.

## Evidence and boundaries

`recheck.py` replays all 31 negative cases, records the admitted fixture contradictions and regenerates the text identity. Place it in a clean checkout of the reviewed head and run `python recheck.py`. `census_check.py` is standalone. `tests.log` is the repository's 50-test suite; its timing arrays/readings are formulas, not measurements. `recheck.json` and `census_check.json` archive results.

No Rust/Miri/fuzz/disassembly rerun, native timing, kernel implementation, merge or dispatch. B5 review and fuzz/disassembly remain separate. I proposed the construction and disclose that involvement; this is registration/evidence review, not independent safety certification. Local validation runs described in the assignment remain the drafter's claims. Spec, role, MODE, RULES and PROTOCOL were freshly read at `a4f97714cc14366984015869d9d094d89488ac57`. Migration head and manifest digest match. Routing audit has no unresolved entries; output is #145. The pinned routing verifier's hard-coded Coordinator sender limitation is recorded separately with a role-aware exact-body check.
