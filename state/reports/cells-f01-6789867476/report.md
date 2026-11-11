# F01 targeted recheck: resolved at d4f0e8fb

Inbox 6789867476; exact reviewed correction **d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2**, parent **57f51bbc0ca62ac3ee1cc148a1010842eea6222a**. **F01 RESOLVED.** I accept the targeted coverage correction and withdraw my F01 B5 objection from REVIEW [6782367271](https://github.com/lmmx/mibz2/pull/145#issuecomment-6782367271). This resolves that objection, not CI, performance qualification, adoption or D100.

## Independence and method

Same original reviewer; no implementation of PR365 or this correction. Prior mathematical research, manuscript/figures and my proposed F01 witnesses are disclosed. This is independent implementation/evidence review, not blind review of the idea or independent authorship of the new Rust fixtures. Authenticated task pin remains **a2da49638ac54d7763de1c167bd054f67ffdc763** (SPEC, SPEC.RESEARCHER, MODE, RULES, PROTOCOL).

Inspected the parent-to-correction diff, full kernel, new tests, shared decoder checker, mutation driver, SAFETY correction, handoff and archived evidence. Independently recomputed all 21 listed evidence hashes and all four source hashes. Kernel SHA-256 is **42e377d8a980c94d95a4b162b0f7ba6466ff6b3844cbc8b10fce933fd25692b2**. Retrieved binary Miri archive through base64, decompressed it and verified that removing trailing whitespace on each line reproduces the readable log exactly. Raw compressed and normalized digests match their distinct manifest entries. No inaccessible requested evidence bytes remain.

Executable review record: `python check.py` beside [inputs.json](inputs.json), [replay.py](replay.py), [checks.json](checks.json). The model is my prior checked-index grammar/writer model with separate stable-sort Psi oracle; this run repeats only the four targeted witnesses, not the full earlier 22,172-case review. No Rust/Cargo exists on this runtime's PATH; I did not rerun native Rust, Miri, decoder integration, fuzz or timing. Archived Rust results below are inspected evidence, not my execution.

## Reach is now demonstrated

The new WriterReach field/type and updates are cfg(test). Counters sit inside the actual seed, descending propagation, ascending expansion, conditional long-copy, final spine-recopy and repetition branches. Reset occurs at writer entry. No release-path operations, admission predicates, default route or team path changed in this diff.

| n / primary | cells / period | seeds | descending / propagated | expanded | long copy | spine recopy | repetition |
|---|---|---|---|---|---|---|---|
| 31 / 16 | 17 / 31 | 1 | 7 / 6 | 7 | 0 | 0 | 0 |
| 55 / 0 | 37 / 55 | 2 | 2 / 1 | 2 | 0 | 1 | 0 |
| 55 / 3 | 38 / 55 | 2 | 6 / 5 | 6 | 1 | 0 | 0 |
| 550 / 0 | 57 / 55 | 2 | 2 / 1 | 2 | 0 | 1 | 4 |

These are the completed Miri trace counts. My model independently matches cells, period, seed, descending, expansion and copy/recopy counts; its counter vocabulary does not separately count propagated child stores or repetition. The 550 witness has eight capped records, budget floor(4n/25)=88, seven marked-split letters, and calls ordinary Cells::invert_runs. Assertions require one admission, 57 cells, one short cycle and zero density/floor/budget/invalid refusals. It therefore closes the production-reach gap.

Both fixtures poison expected output with byte complements, check byte equality before reach assertions, then run a smaller block in the retained arena/position/output vectors. Capacity checks plus inspected resize/clear behavior establish no intervening shrink/reallocation in that reuse sequence. Per-call expanded count resets to zero. This exercises retained stale storage as well as long rules.

## Execution and falsifier

The raw archive completes **39/39 Miri tests**, zero failures, in 1155.64s, including both new tests and the counts above. The retained read-only-cache setup failure happens before testing; it is not misreported as coverage. The initial native probe preceding the period-counter addition is distinguished from final-source native **7/7**. Decoder differential **5/5** includes successful libbzip2 decoding and exact output/error/CRC parity with Segmented under scalar and detected kernels. The ordinary 550 decode asserts actual admission; the smaller 55 cases also exercise forced cells. Detected kernels mean the tested host's dispatch, not every architecture.

Six mutants compile before execution. Five fail with test panics; uncounted-cell aborts with allocator corruption (SIGABRT), a runtime kill rather than an assertion or compile failure. Baseline passes. The crucial sixth mutation removes only the final copy_long statement, leaving the recopy counter intact. Both new fixtures fail **byte equality**, at source lines 977 and 1027, before counter assertions. Poisoned bytes 158/157 remain where 97/98 belong. Independently omitting recopy in my model fails both n55 and n550 witnesses with unwritten output. This is a real writer falsifier, not instrumentation self-confirmation.

## Disposition and limits

F01's missing coverage is remedied on this exact source. F02's prose correction is also accepted: the implemented floor is marked-split letters−1, not union-of-cuts r2−1; the cited 1^51 0^50 example remains 2 versus 3, with admission unchanged.

This is a bounded extension of the prior review, not a universal safety proof or another full B5 campaign. Prior fuzz/benchmark verdicts remain frozen. No new automatic-CI completion or merge readiness is certified; no timing, implementation or authority change. D100 remains separate. Further repetition of this same targeted review has no identified information value absent a source change or counterexample.

Migration check passed before receipt. Pinned routing audit over comments since the prior assignment returned no misroutes; route sends REVIEW to #145. The pinned verify helper is Coordinator-specific (requires FROM coordinator even for REVIEW); publication will be checked by exact author/body/issue and role-aware routing, recording that known limitation rather than forging the sender.

