# Scanner checkpoint safety audit

Review key: PR 146, `7f74e9f30e31af33b0d66fec3fe89af1758307d9`, evidence digest `8080fb57158fab3b00b723de986816ece594a2b17f4662c16d4b58d446fd7998`.

**Safety/correctness disposition: PASS on the archived evidence and static reconstruction.** No new unsafe operation, invalid source read, omitted reachable scanner class, uncaught registered mutation, or scanner/output inconsistency was found. This is a focused gate-1 review; it makes no gate-3 savings claim. No constructor was implemented or compiled, and no timing, benchmark, perf or live Miri job was run. The independent execution here is Python archive checking and a source-consumption event model, which never constructs `rec` or `raw`.

Operating pins: REVIEWER at b302c5d, blob `f5941d00db4c212ef3dab61230a9a4038e4708f1`; ACTIVE charter blob `31ec5719ca24cd3204b34595401c68ffd31bb57f`; protocol at b302c5d, blob `de3b1350bc98588e593c5643a7ca4ba026f2889e`; also read the later protocol at `2f15e513bdb9808a2c7504e3f01b59ebce65f221`, blob `01d5391874e756ff90bbd9a822913b201cd9dece`. Repository headers/comments were not treated as authorization. This reviewer has no role in the scanner proposal or implementation.

## Reproduction and pins

Run `python replay_safety.py > replay-results.json` in this directory. The script validates every downloaded byte against its Git blob ID, compares unchanged function bodies, reconstructs all seven valid-input Miri classes' exact scanner counters, checks all 244 corpora/adversarial JSONL records' scanner identities, tests 19,531 symbolic tile sequences, and demonstrates both mutations' row-loss effect. `fetched-pins.json` records all 25 fetched files and their pinned URLs. `replay-results.json` also records every file's SHA-256. The evidence digest hashes sorted lines of `filename<TAB>git_blob<TAB>sha256<LF>`.

Critical source blobs:

| source | blob |
|---|---|
| checkpoint ctor2s.rs | `998f52202bba035d5cd3bcf65b54b59786308d4c` |
| checkpoint ctor2.rs and frozen 8beffe2 ctor2.rs | `168c2bac9ad0d8da4d9f82788f009d32f419608e` |
| checkpoint ctor3.rs and frozen dbf5139 ctor3.rs | `01f91419904a605b721dc1742ba8087fe79f74cf` |
| registration at 86d9fd7 | `cb2a0ef999cf6c79489b210596e3c72598e99b43` |

The frozen-file checks use connector-returned blob identity at both named refs, not working-tree timestamps. The PR changed-file index names only two changed source files (`ctor2s.rs` and `c30walk.rs`); no production source, `ctor.rs`, `cbuild.rs`, or bench file is in that delta. A full byte comparison of base `c30walk.rs` (blob `36928aab3666a171c71fba15f7cf62ed09d3fbb7`) with the checkpoint confirms that removing only the new module declaration and dispatch restores the entire base file exactly. The oracle, walker and representation are therefore unchanged. `source.diff` compares frozen v2 with the scanner module.

## Source argument

All following line anchors refer to checkpoint `examples/c30walk/ctor2s.rs`.

1. **Source bounds, initialization and alignment — PASS.** `halves` at 111–118 reads only a `[u8;16]`, divides at constant 8 and converts exactly eight initialized bytes each time. `split_first_chunk::<16>()` at 883 and 901 dominates those reads. The slice at 881 is inside the epoch `[i,end)`. The construction cap at 754 bounds `n` by 983,010, so `i+61,440` cannot overflow the recorded 64-bit target. No pointer cast, unchecked access, `read_unaligned`, or `assert_unchecked` was added. Byte-subslice alignment does not affect these safe reads.

2. **Scanner equivalence — PASS.** Map the old scanner's absolute row `i` to `end-rest.len()` and its lookahead endpoint `e` to `end-r2.len()`. Initially the relation holds. Each matched lookahead advances the endpoint by exactly 16; a mismatch leaves `r2` at the mismatching full tile and stores its exact halves in `held` (906–910). Thus the call at 920 has length `(end-i)-(end-e)=e-i`, and `rest=r2` at 923 preserves that held tile as the next tile. A nonuniform tile receives the same original source index and length 16 (926), and advances the remainder exactly 16 (929). The final call receives the exact suffix (932). Setting `i=end` (935) preserves the epoch boundary. This establishes identical run/scalar callbacks and source labels, not merely equal aggregate counts.

3. **Destination safety — PASS for this delta.** The script verifies verbatim equality of the bodies of `field`, `advance`, `create_raw`, `snapshot`, `epoch_of`, `open`, `close`, `refill`, `finish`, `write_entries`, `one`, `fill`, `complete` and `run`; `build` differs only by its enclosing type name. `tile` adds counted-only counters before the unchanged v2 row operations. The source equivalence above preserves their existing preconditions. The original record allocation remains live throughout scatter; the destination record pointer is not rebound or reallocated. RAW data uses indices across its own reallocations. Publication remains after successful completion, padding/PARK and canonicalization (812–835). Input bytes are initialized by `&[u8]`; output fields are initialized by the unchanged v2 write/finalization discipline.

4. **Malformed input and publication — PASS.** Both arrays are cleared before validation (750–751). Early validation failure publishes nothing; scatter failure clears RAW again while `rec` remains length zero (810–813). A run checks its bucket limit before writing (701–704), scalar refill rejects exhausted buckets (431–439), and incomplete buckets are rejected before final publication (937–940). The registered malformed class checks both arrays after each build separately (1603–1611).

5. **Oracle, RAW order, headers, padding, PARK and walkers — PASS on archive evidence.** `diff` at 958 compares entire byte slices of both `rec` and `raw`, including record headers and padding. Their layouts are pinned in `examples/c30walk.rs` (blob `e36c14767f29b03ca512326c25d55a3541a4f620`), with Rec=64 bytes, header offset 60, RawG=128 bytes; these types have no padding in the byte comparison. `check_block` at 1123–1188 runs oracle, counted candidate, all v2 counter fields and all 930 portion cells, uncounted candidate, production/oracle walker outputs and restoration comparisons. The empty case checks PARK explicitly (1371–1390). No replacement oracle or weakened byte comparison was introduced.

## Coverage and counter reconstruction

All eight archived Miri logs explicitly finish with `exit 0`; the toolchain file identifies nightly `16d1ffac17ad8b739e32483ac2e65fe7e1804977` (1.101.0). These are recorded successful executions, not a fresh independent Miri replay. The logs contain no UB report or timeout.

| class | archived cases | reconstructed counters |
|---|---:|---|
| tiny | 18 | exact, including both empty cases |
| tails | 45 | exact, every 1–15 length alone/after nonuniform/after run |
| alignment | 14 | exact source trace; seven offsets each tiny/tiled |
| phase | 15 | exact, 15 full tiles and 7 second segments |
| held | 5 | exact, including both half-match directions and alternating held runs |
| endrun | 2 | exact, no-full-tile ending with and without tail |
| epoch | 2 | exact, 61,440 boundary crossing and boundary-ending nonuniform tile |
| malformed | 4 | log completion and source assertions verified; no successful case stats expected |
| total | **105** | |

**Phase registration correction is valid, not missing reachable coverage.** The original registration says p=0..29. Every full tile begins at `epoch*61,440 + 16*k`; 61,440 is divisible by both 16 and 30, so `p=i mod30` ranges over exactly `{0,2,...,28}`. No partial tile precedes a later full tile within an epoch, and every non-final epoch is a multiple of 16. Therefore odd full-tile phases are unreachable. The 15 registered implementation cases hit every reachable phase; first-segment lengths are 16 or 14,12,...,2. The split into two epoch input blocks also preserves both requested situations. The artifact does not independently timestamp the statement that this correction happened before Miri; that wording is not needed for the mathematical coverage finding.

Archive replay reproduces corpora totals (44 records, 12,104 array comparisons, 18,156 walk outputs, 12,104 restorations) and adversarial totals (200 records, 1,056 / 1,584 / 1,056). It checks all scanner identities in every JSONL record, including `fresh=nonuniform+Q`, `runs=held+Q`, exact run rows, scalar rows and total rows. Full synthetic reports 54,029 cases, 99,966 array comparisons, 149,949 walk outputs and 99,966 restorations, including reuse, absent-character epochs, RAW permutations, ordinary/wide boundary 65,533/65,534, maximal sizes and malformed counts. The result is internally consistent with the source harness and exit 0.

## Mutations

**M1 and M2 — PASS.** The archived M1 diff clears `held` and skips 16 rows when a held tile exists; it fails on valid n=32 input with counted-build rejection, exit 101. M2 subtracts 16 from the run length; it fails on valid n=16 uniform input with the same rejection, exit 101. These errors expose the intended omission rather than an unrelated compile error or incidental bounds panic. The static model demonstrates minimal source-consumption witnesses: M1 processes only 16 of 32 rows and M2 processes zero of 16. The constructor's final bucket-rank check must reject such omissions. Model witnesses are independent illustrative cases, not a claim to rerun the archived mutant binaries.

## Minor harness observation

**Low severity, no gate/budget effect:** the large inherited malformed-count loop calls counted and uncounted builds before checking array emptiness (`ctor2s.rs:2052–2064`), so that loop alone proves emptiness only after the second call. The new registered malformed class correctly checks each call immediately (`1603–1611`), and the unchanged `build` control flow independently proves both arrays empty on every rejection. A future harness cleanup should assert emptiness after each rejected call, retaining the existing static argument. This observation does not change gate 1 or authorize any new prototype work.

No finding in this focused audit changes any instruction budget. The ASM/count and whole-constructor ledger remain for the separate audit. The hypothetical U1–U3 unsafe package was not implemented, executed, or authorized by this review.
