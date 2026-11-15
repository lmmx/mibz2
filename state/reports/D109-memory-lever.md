# D109 — #447 memory-lever: source review complete; validation HOLD

2026-11-15. Checkpoint [6902547995](https://github.com/lmmx/mibz2/pull/161#issuecomment-6902547995).
Head **81aa4b1dfa3333754c77baca975948c3cd0121b8**.
Authority: docs/team SPEC, SPEC.GUARDIAN, MODE, RULES and PROTOCOL at a2da49638ac54d7763de1c167bd054f67ffdc763.

## Ruling

**G1: accept the corrected ordering argument and the design of the complementary fence checks. G2: retain a narrow B5 evidence HOLD on integrating/running the experimental arms until completed, exact-source validation is inspectable. D108 is not yet fully closed.** No new unsafe-access defect was identified in this source review. This does not stop implementation of corrective checks, ordinary CI, unrelated work or the Cells line.

What remains is evidence completion, not a demand for another kernel redesign: completed native feature-on tests/fence check, the four feature Miri tests, feature fuzz, and baseline/targeted mutation diagnostics. Existing completed local logs pinned to this source can serve; no timing rerun is requested. The current journal and TSV report success but do not provide those execution transcripts. At the single CI snapshot the relevant jobs were still in progress; their log endpoints returned BlobNotFound. This is unavailable evidence, not failed tests. RULES B7 prevents converting it into PASS.

Independent involvement: I did not implement these arms. I reviewed their source and surrounding delta, the test reach assertions, mutation harness, feature gating and compiled-check script. Python independently confirmed that all 38 mutation replacements match the reviewed source exactly once. I did not execute Rust/Miri/native disassembly here (cargo unavailable on PATH), and do not claim to reproduce the reported 35 kills.

## D108: what has been repaired

The journal now correctly requires sfence before subsequent access to streamed memory, including on the issuing thread. A named Fence guard exists before any send and drops on success, post-store refusal and ordinary unwind; writer access follows induction return. Initial refusal precedes stores. Stage layout and disjoint full/final send arithmetic retain D108's source argument.

The test model marks whole-line sends, tracks fence count and pending-clear count, and checks success, budget refusal and a caught panic after an actually marked send. Requiring CLEARED to increase prevents a vacuous all-false model. A final reuse run checks recovery. Miri runs scalar copies and this model, not the hardware intrinsics.

The compiled script supplies the complementary missing test: it requires sfence in the staged induction or its called Fence drop. The TSV distinguishes no guard (model and compiled check fail), early guard drop (model fails, compiled check survives), and no sfence (model survives, compiled check fails). This is the right division of obligations. Instruction presence alone does not prove dominance of every exit; the source/RAII argument and exit model supply that component. The pending marker does not itself prove hardware ordering.

The mutation runner has an evidentiary limitation: any nonzero exit from the compiled-check shell is called a fence kill, including cargo/objdump/tool failure. Its TSV has no exact exit diagnostics or separately recorded baseline row. To close the hold, retain a successful baseline and the failing check's actual diagnostic for the fence deletion, plus the named model-test failures for missing/early guard and absent marker. This can be supplied from the existing run; a claim of 35/38 alone cannot distinguish infrastructure failure from detection. The script also restores source only on normal completion, not interruption; record restoration if rerunning.

The three current survivors have different premises from a missing fence: aligned slots establish the redundant rule-offset condition; live-letter ownership bounds pool demand; the marked row terminates traversal before stream exhaustion. These are source arguments, not claimed test kills. Keep all survivor outcomes, including D108's earlier failed target, as history.

## Lazy pool and stream proof scope

The safe entry bounds budget and record count by n, then lazy layout refuses n >= 2^22. Census validates positive record lengths, their total and primary row before constructing streams. The second pass splits exactly the same marked run counted by the first; each letter contributes one byte and one entry.

The output allocation is partitioned into entries [BPAD, BPAD+4R), top bytes [BPAD+4R, BPAD+5R), a gap, then R bottom slots and a trailing gap. The top pool and words are in the separate kernel allocation. Both allocations are resized before raw pointers are taken; induction does not resize them. Mutable output storage therefore does not alias the input records or kernel storage in the reviewed safe caller. The writer overwrites output only after induction ends.

The 22-bit length, 8-bit byte, ROW bit30 and TAG bit31 do not overlap. Both slot allocations are refused at 2^31, so tagged pointers and ENTRY markers remain distinguishable from offsets. All sizing arithmetic is bounded by n/budget before the threshold checks, including on 32-bit usize.

A top-stream byte selects that byte's descending entry counter. It consumes exactly one of the counted letters; unused wrapping counters for absent/exhausted bytes are never dereferenced. Both streams check their remaining counts. An entry tagged by one front is read by the other front: hence its offset belongs to the correct pool. The original front has already passed that entry, so it does not misinterpret its own tag.

take either advances a checked pool end or removes one free pair. give records the dead top slot's free-list link and matching bottom slot. On an equal step the retained cell is installed in the opposite slots before the front pair is released; on a closed unmarked letter a pair is released only if it is not an ENTRY marker. The asymmetric steps reuse the ended front's slot through the same promised-slot invariant as shipped Cells. No live link is intended to retain a released pair. All tests that inspect offsets do so against the appropriate separate pool.

These ownership arguments are substantial new unsafe scope; the checked-in tests exercise all three comparisons, closed cycles, pool reuse, exact/short budgets, long lengths and public fallback. Their existence and targeted mutations support the design, but do not replace completed Miri/differential/fuzz execution evidence. The new feature fuzz compares partial output, exact errors and route counters for all four experimental admission variants.

## Feature-off claim

Source isolation is supported: experimental fields, include, dispatch, layout rounding, exports and reader/worker constructors are feature-gated; test hooks are test-only. The shipped induction is unchanged. Without memory-lever the original dispatch and layout remain. With it, variant zero selects the shipped induction, while the NT arm has a separate copied function.

**Byte-for-byte binary identity is not established.** Equal callgrind counts and absence of selected symbols/sfence are not a binary comparison. The symbol script is a useful exclusion check, not a hash of all production machine code. This is a correction of the claim's strength, not an additional requirement for whole-archive identity: the opt-in contract can be stated as source/behavior isolation, with its actual checks. A literal binary-identity claim needs matched-build bytes or disassembly.

## Evidence snapshot and boundaries

Reviewed the journal, mutation TSV/script, cells_lever.rs, cells_lever_test.rs, affected cells/caller/test diffs, workflows and registration LOG 6902022809. CI jobs at head: Rust 132984705700, Miri 132984705396 and fuzz 132984705747 in progress; each log fetch returned BlobNotFound at 22:41 UTC. Claimed local results (four new Miri tests, nine shipped tests, 22,913 fuzz cases, corpus hashes and 35 mutation kills) remain attributed to Pathfinder.

The experiment is opt-in; #444's shipping rejection is not reversed by moving its code here. No production/default/threshold qualification follows. Registration and workstation readiness remain separate, including green CI, frozen recipe and Louis's own run under C5; repository comments do not independently grant that authority. No workstation run, readiness tag, merge or automation change was performed.

Migration head and manifest matched; inbox reconciliation found only this unfinished item. Routing audit was clean. REVIEW goes to #145. Pinned coord.py has the known coordinator-only FROM check; its sole failure is checked separately against the exact Guardian header, body and destination.
