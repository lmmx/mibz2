# C31 / #277: closure review, count-admission gap remains

**REVISE before freeze, narrowly.** Reviewed `7f88388b938528f22088df79387f309eb112fe1c`, answering inbox #162 comment **6732258318** and the repairs to REVIEW **6732062626**. The six original witnesses are rejected, the timed D path is checked, and the accounting repair is sound. Do not roll those closures back. The new count admission still accepts physically impossible evidence, including its own nominally valid fixture. Complete the necessary inequalities below; no kernel redesign or new threshold is requested.

## Closed

- `bench/run_fed_stage.py` rejects missing, boolean, zero and negative GHz before readings; its finite [0.1,10] range also rejects wrong-unit input. Output denominators, compressed size, hash and block count are checked against independent expected corpus metadata. The six previous mutations and the 19 additional registered fixture mutations fail admission. All **50 Python verifier tests pass** here, including CLI rejection before readings.
- `examples/run_fed_stage.rs` now checks `arm == 0 || by_runs == runs` after every timed entry call. The phase path already requires successful placement and does not silently charge fallback as placement. This closes the prior missing-D guard.
- The per-round identity is correct: `D_i−A_i = K_i+δ_i−e_i−f_i`, with `K_i=B_i−A_i−sort_B,i`. Both readers compute each round before taking medians; the historical sensitivity now takes the median of `.218+δ_i−e_i−f_i`. Neither sum-of-medians nor a cross-run constant is claimed as the paired entry verdict. The identity is algebraic bookkeeping, not independent proof of the clock brackets or causal isolation.
- `e` and `f` are explicitly measured controls. Q1 excludes the following walk/RLE1/CRC. D/B compares complete constructors. The 0.95 prospective diagnostic bar, old #269 0.90 STOP, and dropping old C/store-floor controls remain as assessed previously. No outcome or production gate changes.
- Source-generated text independently regenerated from the exact head: **3,324,678 decoded bytes; 611,579 compressed bytes; 4 blocks; SHA-256 `b4c2767e731f9be033d841e2f23d3bcbb7ba11335a60404a4a3af51c458916a5`**. All five head-independent synthetic identities pass the verifier tests. Python/libbzip2 variability fails admission rather than silently substituting an input. Header-magic scanning is a count cross-check under these pinned hashes, not a general structural bzip2 parser.
- GitHub's full comparison against `705323290b5470dea4a31910c22522c342e9d064` lists seven changed files, all harness/fixture/journal; no `src/` or `tests/` change. `src/mtf.rs` and `src/bwt.rs` blobs match #269 `fd60fffada25d475bbcc2b54165b96cf5fd7a92f`. Their whole-file blobs differ from master `6b36fd8c43c1b834e69ee62cd6b5054dc99c571c` because of research additions; source comparison leaves the incumbent MTF/build bodies unchanged. Do not describe the entire files as master-identical.

## Remaining admission defect

`bench/run_fed_stage.py:202–231` applies necessary but incomplete count relations. Write `b_f=blocks_fallback`, `b_r=blocks_runs`, `n_f=rows−rows_runs`, `r=records`, `c=cells`, `v=d_vector`, `s=d_scalar`, `σ=d_streams`. Four omitted necessary relations are:

1. **`b_f ≤ n_f ≤ 900000 b_f`.** The lower bound alone leaves rows assigned to a nonexistent fallback, or too many rows for its number of blocks. In particular `b_f=0` requires `rows=rows_runs`.
2. **`b_r ≤ σ ≤ min(256 b_r,r)`.** Every nonempty byte bucket contains at least one maximal run. A block with one run cannot have 256 nonempty buckets.
3. **`8c ≤ slots_B`.** Each positive-length cell performs at least one 8-slot store in B, even where that overstore overlaps another cell. Existing `rows_runs ≤ slots_B` is weaker when cells are short; both bounds are necessary.
4. **`c ≤ v+s`.** Each positive-length D cell enters its store loops from its own start and performs at least one vector or scalar store. An earlier overstore does not skip the next cell's emission. This complements the written-slot lower bound `rows_runs ≤ 8v+s`.

These are summed necessary bounds, not a request to authenticate every count by implementing a second decoder. They follow directly from this harness's already registered representation and store loops. Their check belongs before `readings()`.

### Replayed witnesses

`witnesses.py` reads the exact-head verifier and its formula-time fixture, regenerates corpus evidence independently, and adjusts the text identity to this checkout as the repository's own CLI test does. It leaves timing arrays unchanged. All four cases below return **no admission problems and Q1 FEASIBLE**. These are contradictory test records, not real measurements:

| Mutation | Contradiction still admitted |
|---|---|
| One-byte corpus `rows=96531`, retaining `rows_runs=96530`, `blocks_fallback=0` | One fallback row with zero fallback blocks |
| Wikidata `rows=206000000`, retaining `rows_runs=163600000`, `blocks_fallback=47` | 42,400,000 fallback rows exceed the 42,300,000-row capacity |
| One-byte corpus `d_streams=256`, retaining `records=1` | 256 nonempty byte buckets from one maximal run |
| Wikidata `cells=20000000`, `slots=159999992`, `d_vector=19000000`, `d_scalar=0`, `rows_runs=150000000` | B slots below 160,000,000; D's 19,000,000 stores below 20,000,000 cells; fallback also over capacity |

More directly, the **unmutated synthetic-text fixture** (`examples/run_fed_stage.rs` fixture writer; `bench/fixtures/run_fed_stage.json`) declares `cells=1500000`, `slots=5000000`, `d_vector=560000`, `d_scalar=2000`. It needs at least **12,000,000 B slots** and **1,500,000 D stores**, but declares 5,000,000 and 562,000. “Made up but within the checked relations” is not a physically feasible positive fixture. Repair the writer and JSON fixture together; do not weaken the necessary bounds to retain these constants.

**Materiality:** the primary time ratio is unaffected by these specific count edits. No actual captured result is shown corrupt. But admission claims consistent paths and uses the new census to explain mechanism. It currently accepts nonexistent path rows and impossible store/stream work. Preserve the count-integrity requirement already under review instead of labeling these records VALID and explaining them afterwards. Add these inequalities and negative cases, retain a feasible passing fixture, rerun the existing suite. That is the remaining freeze change.

## Census and interpretation

For a cell `[q,q+ℓ)` in a bucket ending at `b`, the independently proposed formula remains correct:

`v=min(ceil(ℓ/8),floor((b−q)/8)); s=max(0,ℓ−8v)`.

I rebuilt cells from a row-ownership grid, without the implementation's resumed cursors or binary-search seeds, and compared the formula to literal vector/scalar-loop counting. **10,840 columns pass**: all 9,840 nonempty ternary columns through length 8, plus 1,000 deterministic longer-run columns. They also satisfy all four relevant store/stream inequalities. This checks the mathematical census, not native machine code or B5 safety.

`direct_census` is outside timed brackets, as intended. Its `v+s` is a count of source-level table-store operations. Until disassembly, call it predicted table stores rather than total machine store instructions: compiler spills, scratch state, and instruction lowering are not counted. Likewise `σ` counts nonempty destination buckets in a block; it is not a measurement of simultaneously active hardware write streams or write-buffer pressure.

Two stale lines remain in the journal: §6 says “the same 8-lane stores”; §8 says “D isolates the sort's removal.” They conflict with the corrected §§1–3. Remove or qualify them during the count repair. No additional experiment follows from this editorial cleanup.

## Scope and next step

Expected: previous six invalid inputs would fail and census/count evidence would be consistent. Observed: original failures closed, 50 tests pass, census model agrees, remaining necessary bounds absent and positive fixture impossible. No measured performance gap closes in this review. The next useful step is the small admission/fixture repair, followed by a frozen diagnostic at its exact head; not another speculative performance model or a relaxed gate.

B5 remains separate: written safety proof, fuzz coverage, Miri/differential/mutation evidence and the queued independent review are not waived. I proposed the direct-F construction and disclose that involvement. I did not rerun Rust tests/Miri, execute native timing, inspect compiled disassembly, validate a Wikidata binary on disk, implement the repair, merge, or authorise dispatch. The referenced prior native checks are the drafter's evidence, not fresh observations here. Registration closure is not production qualification or whole-decoder accounting.

Policy read at `a4f97714cc14366984015869d9d094d89488ac57`: SPEC, SPEC.RESEARCHER, MODE, RULES, PROTOCOL. Migration manifest hash and legacy branch head match the cursor. Routing audit: five previously resolved historical entries, no unresolved misroutes; REVIEW routes to #145. Publication verification is recorded separately; pinned `coord.py verify` hard-codes the Coordinator sender and requires an additional role-aware exact-body/#145 check for Researcher output.

## Reproduction

At a clean checkout of `7f88388b938528f22088df79387f309eb112fe1c`, place `witnesses.py` in the checkout root and run `python witnesses.py`; it imports that checkout's bench verifier/test helpers and regenerates synthetic input from its sources. Run `python bench/test_run_fed_stage.py` for the existing 50 tests. Run `python census_check.py` independently for the mathematical model. The archived `tests.log` contains formula-fixture readings only, never performance evidence. `witnesses.json` and `census_check.json` retain the observed outputs.
