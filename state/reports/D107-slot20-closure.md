# D107 — D106 closed: B5 PASS for Cells slot20

2026-11-14. Incoming [6882196843](https://github.com/lmmx/mibz2/pull/161#issuecomment-6882196843). Exact reviewed head: **24ff5f72cec50f441a9a95a5927f640c00fbe721**, PR #391. Operating SPEC/RULES pin: a2da49638ac54d7763de1c167bd054f67ffdc763.

**G1: PASS on B5 closure. G2: lift D106's scoped integration HOLD for this unchanged layout at this head.** D106 remains an accurate historical verdict on the evidence then available. No new unsafe defect or missed mutation target was established; this is closure of an evidence HOLD, not a newly inferred B6 stop/resumption.

## Source identity, not an assumed carry-forward

Direct Git tree reads independently establish that all four revisions have identical:
- src tree: bd7586ebdccd645d091923e6f53b9769ca6719be;
- tests tree: a0b23a6f015c1cab555d03905217511d36452374;
- examples tree: 043b475a31fb2efee469d2f7f05c042339f880b2;
- Cargo.toml blob: 47be64728c2896f87dd94d3e70e240c1bfd9c0ea;
- Cargo.lock blob: 3f0e6c76d9856f5cc835e58fbd3f3f3f38e960cf.

The four revisions are D106's c915a95eb7288843ea25707bffc87c9d252f4bde, mutation baseline ee84010861700647aa3bac62f131dd524e4fcbd0, repaired head 24ff5f72cec50f441a9a95a5927f640c00fbe721, and CI checkout **18fd473dd08eac5b7e90af8b56f9bb8ee104b082**. Logs explicitly show the last as the merge of the repaired head into 7c9084d6c07f1cb39ca25088999e8d20eb0f7aa4; commit parents independently confirm it. These are PR merge-checkout tests, not tests secretly run on a different kernel. The head is descended from D106's reviewed head; its only changed paths are test.yml and the mutation record.

D106's alignment, aliasing, FAR/PAD initialized-read, arena/rule bounds and trace-accounting review therefore carries forward without redoing it. Zero-fill remains part of the proof; no conclusion extends to a later uninitialized arena or 16-byte layout.

## Closure evidence independently read

| D106 requirement | Evidence and conclusion |
|---|---|
| YAML repair | Exact-head test.yml quotes the cells:: command. Local PyYAML parsing now succeeds and yields the intended command. Closed. |
| Native/differential and slot24 tests | [Rust job 132536816064](https://github.com/lmmx/mibz2/actions/runs/43624367883/job/132536816064): 110 library tests passed, one ignored; all five Cells integration tests passed, including exact error/partial-output differential coverage; slot24 step passed all nine Cells tests. Additional ignored tests are not relabelled executed. Closed. |
| Fuzz smoke | [Job 132536816180](https://github.com/lmmx/mibz2/actions/runs/43624367883/job/132536816180): 28,202 Cells cases, seed 42. Production admitted 730, density 4101, sparse 2735, floor 687, budget 1366, invalid 0, short cycles 729; forced admitted 14,872, invalid 0, short cycles 10,463, cells 241,400,178. Thus both routes were exercised, not merely selected by name. Closed. |
| Miri completion | Raw log for job 132536884401 shows all six Cells tests and read::tests::miri_sequential_default_cells_route individually passing, followed by 41 passed, 0 failed, 0 ignored, 61 filtered out. Execution finished 18:20:19 UTC after 1601.44 seconds. This replaces the earlier partial snapshot with completed coverage; it does not claim all possible inputs. Closed. |
| Mutants and restoration | [Committed mutation record](https://github.com/lmmx/mibz2/blob/24ff5f72cec50f441a9a95a5927f640c00fbe721/docs/journal/2026-11-14-cells-slot20/mutants.md) pins baseline ee840108, textual changes, both timeout-300 commands, terminal statuses, first failures and passing baselines before/after; restored source is recorded byte-identical. Closed as reproducible recorded experimental evidence, with the limits below. |

All non-skipped check-runs returned for the repaired head were successful, including Rust, fuzz-smoke and Miri. Companion JSON preserves relevant log excerpts, check identifiers/URLs, the mutation record and tree correspondence.

## Mutation judgment

M1 widens the write, M2 the read; M3 shifts both fields onto the word offset, M4 shifts them one byte beyond the slot end, M6 shifts only the write. All five have exit 101, assertions or SIGSEGV explicitly distinguished; M2's signal reproduces. These target overwrite, address corruption and inconsistent interpretation, not just counters.

M5's full suites remain **timeout witnesses, exit 124 at 300 seconds**. The original manually terminated run is preserved separately. Its focused slot-partner-bound and exact-fit tests now fail deterministically with exit 101 under timeout 120. The admission-only test survives, explicitly recorded; it is not the designated witness for partner layout, and the suite's targeted witnesses catch M5. This is not a hidden missed target. The source establishes why the failures matter: SLOT=16 makes the partner field overwrite adjacent slots. No requirement to count a timeout as a kill is needed.

The mutation record is a committed experiment report containing exact edits/commands/outcomes and failure excerpts, not attached full raw console files. I verified its source correspondence and assessed its witnesses; I did **not** rerun these mutants locally. Native and Miri results above were verified from GitHub job logs. I did not implement the change. This is the independent adversarial review required by B5; it is not a claim of universal safety from finite tests.

## What changes, what does not

Expected closure was a workflow repair plus completed exact-source tests and reproducible mutant outcomes. Those evidence gaps are closed; source safety reasoning is unchanged. The material lesson is that a green neighbouring workflow cannot substitute for a parser-rejected Test workflow, and a hang should be retained separately from deterministic mutation failures.

No further B5 experiment is required for this head. Integration remains with the Coordinator under its separate exact-head CI/performance obligations. LOG 6882195224 identifies the predeclared ee840108 bench as the sole primary-regression rerun, preserves the first text-4t flag and treats the later documentation push's bench as non-decision evidence. This review does not independently reanalyse that artifact or confer a performance PASS, launch any rerun, freeze/authorize an i9 run, or widen a default/thread gate.

Receipt committed first; migration source head and manifest SHA-256 verified; inbox reconciliation found only this checkpoint unfinished; routing audit since 6881612485 returned no misroutes. Writes are confined to team-state/guardian state and one REVIEW on #145. Persistent webhook settings remain unchanged.
