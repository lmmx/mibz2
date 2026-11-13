# Selective Cells B5 review — assignment 6857053711

**Verdict: HOLD / REVISE the exact-source mutation evidence.** Reviewed only f2dc198ee22ad13b69a72d42744fb1a0d135b95f. I found no changed-path defect in the selected materialization logic. The available three-new-mutant evidence does not identify this source, so I cannot close its requested B5 obligation yet. This is a bounded evidence finding, not a claim of observed undefined behavior or a performance verdict.

## H1: three new mutation runs are not bound to the reviewed source

The producer bundle at [5df5a2](https://github.com/lmmx/mibz2/tree/5df5a2eeeaea99c1b07f94c8a5f3ca46666b9c05/state/reports/cells-selective-6856176834) contains:

| Object | src/cells.rs SHA-256 |
|---|---|
| Reviewed head, independently hashed; cells-selective-source.json | 3bbc63745068713b75bd6cbaac4217d374fbca6d1318a55c27bf9e788858aeee |
| Six original mutations' results.json | 3bbc63745068713b75bd6cbaac4217d374fbca6d1318a55c27bf9e788858aeee |
| Three selected-write mutations' results.json | ca1b9116309fcb140d3aa27917cbfe68ff8581bae7eb1d4553a7c954bb6406aa |

The new runner computes this digest from the unmutated copied source before applying any substitution. It is not a mutant hash. Its baseline and three mutants demonstrably compile and run, but that alone cannot establish correspondence to this head. The preserved fmt failure changes examples/ab_cells/mod.rs only; it does not explain this src/cells.rs discrepancy. The bundle does not supply the differing source or a mechanical correspondence. I verified the fetched result/log digests against SHA256.json: this discrepancy is present in the immutable evidence, not introduced by retrieval.

**Smallest closure:** rerun just the new suite's positive baseline and three compiled mutants from the exact reviewed head, recording that head, clean source identity and relevant test/helper identities with the logs; alternatively supply the original ca1b... source plus a mechanical comparison proving every difference irrelevant to these tests and the changed production path. Do not relabel the existing digest or silently substitute a successor. No new algorithm, performance attempt or workstation run is needed to answer H1. Any successor needs explicit correspondence.

## Independent changed-path reasoning

The production difference is the movement of the unchanged MARK-inclusive predicate before the alternative writes. Both unmarked short factors have positive lengths whose sum is at most16. Thus the short branch retains the same two copy16 calls, in their original order; la.min(16) equals la here. MARK must not be stripped: a marked short composite is a rule, with the initial marked byte as a possible terminal singleton. Arithmetic, selected descriptors, cell counting and all three slot-transition cases are unchanged.

The key dependency question is stronger than equality of final bytes. Does an omitted speculative write establish memory later used meaningfully? I find it does not under the existing constructor/grammar invariants:

- slots creates a fully initialized Vec of the allocation's length, not merely reserved capacity. Retention preserves initialized bytes; byte table and canaries are reset. Initialized does not mean zero or semantically live.
- Before a short join, both meaningful literal prefixes end at or before top. The first complete 16-byte load precedes its overlapping store; that store starts at top, so it cannot damage B's meaningful prefix before the second load. It can change B's irrelevant tail. The second store overwrites the first overcopy at the exact A-length boundary. Only A's prefix followed by B's prefix is published.
- A discarded literal write on a long/marked join starts at top or later. It cannot establish an earlier live literal's prefix. top is unchanged; a later selected short join writes its entire meaningful prefix before publishing it.
- A discarded rule write on a short join targets the next unused rp. rp is unchanged. The next retained rule writes both ordered u64 descriptors, all16 bytes, before publishing its pointer. No live rule depends on the unused pair.
- Every retained child descriptor precedes its parent; selected rules advance rp by16, selected literals advance top by their true length. The budget check still precedes each cell, including equal steps. Per-cell slot, arena and rule reservations therefore suffice exactly as before. The removed accesses cannot require additional slack: retained operations still have initialized16-byte source windows and the existing32-byte arena destination allowance. Debug/Miri canaries supplement, rather than prove, those bounds.
- Padding prefetch reads/hints remain in the unchanged slot regions; they do not turn dead arena tails into grammar. Meaningful pointer/length fields and marks are unchanged.
- Refusal paths, output mutation timing, allocation high-water policy and pos reset are unchanged. Retained large-small-large calls remain subject to initialization and live-prefix invariants, not equality of all stale scratch bytes.

I independently compared extracted functions: slots, write_cells and try_invert are byte-exact against corrected365 d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2. Writer propagation, first-position reuse, F01 final marked-spine recopy, SHIFT=1 rotation, periodic repetition and Segmented fallback consequently receive the same live inputs; this review does not reopen or globally certify every inherited algorithm premise. No guarded379 writer or equal-unit forwarding enters the delta.

## Comparator and counterchecks

I removed exactly one record_materialization hook and three record_selected_slots hooks from the eager comparator, reversed its function rename, and compared whitespace-normalized text to365's induction. It matches. These are read-only snapshots: no production repair or alternate algorithm is hidden in the comparator. Selected snapshots include descriptors, both frontiers/cursors, count, retained payload and the two written slot records. Subsequent snapshots, returned root/refusal, rule_end, writer pos and independent-oracle bytes add observability. Initial letter slots are compared too. Fresh-slot auxiliary writes are not all snapshotted immediately, but they are unchanged and their later consumed fields feed the comparison. This is strong finite regression coverage, not an all-state proof.

The test loop count is independently 9601: binary columns of lengths1..8 and ternary1..6, every origin. Boundary tests assert unmarked15/16/17, marked-short/long and all three step orders, rather than merely listing examples. Budgets0/B-1/B, arbitrary cycles, periodic cases and differing0x39/0xC6 retained-buffer poison are exercised. Initial allocation can replace poison with zeros; subsequent retained calls supply the differential-tail check. The n55/n550 writer tests and production-admission test retain actual branch reach.

Executed locally: mechanical source checks; verification of21 retrieved evidence files against the producer manifest; and **88,860** short-copy placement cases covering every positive length pair summing to<=16 and every operand-prefix placement before top=32. Old/new initialized buffers have different dead tails and overlapping source windows; the exact sequential16-byte load/store model always preserves the expected concatenation prefix. This model probes the disputed tail dependency. It is not Rust execution, a full grammar interpreter, or a substitute for Miri. Rust/cargo are unavailable here; I did not rerun native, Miri or fuzz binaries.

See countercheck.py and countercheck.json. Its local input names head.rs, original.rs and comparator.rs correspond respectively to the reviewed head's src/cells.rs,365's src/cells.rs, and the reviewed head's src/cells_original_test.rs. evidence.json preserves the21 verified text artifacts plus their manifest; hashes identify all inputs.

## What the raw evidence establishes

- Native log: nine Cells tests pass, including the exhaustive selected-state test. The displayed canary panic is caught by its passing negative test.
- Targeted Miri log: six named tests pass, zero failures,120.85s, including selected boundaries/reuse and both long-writer routes. This verifies the logged completion; its attribution to the reviewed checkout remains the producer's report/source manifest, not an independently reproduced binary identity.
- Original mutation baseline: nine tests pass. All six mutant logs show compilation followed by execution failures. Uncounted-cell is an allocator SIGABRT, not an assertion or output mismatch; lost-mark is a mark assertion; wrong-backlink reaches an unexpected Budget refusal; wrong-rotation is an oracle mismatch; region-overrun trips the region bound; missing-spine-recopy fails both long-writer tests. Do not call all six output kills.
- New positive baseline passes. Missing-short and missing-rule fail selected state/payload comparison; masked-mark fails selected representation classification. They are three compiled assertion kills on the recorded ca1b... source, **not three output-byte kills**, and remain subject to H1.
- Fuzz log: seed42,26348 cases; production2607 admitted/3816 density/637 floor/1913 budget,2109 short cycles,1781648 cells; forced13890 admitted,9772 short cycles,225403031 cells. The report says30 seconds; the text log does not independently timestamp that duration. It shows no failure, not exhaustive correctness.
- Full selection ends106 library+69 integration passes, one ignored and two excluded large Room-walk tests. Preserve the prior105-pass/one-fail affinity result (“5 cores of4 CPUs”), and constrained run's two service failures at four-block reach. The final selection is not a claim that those earlier attempts passed.
- tests/service.rs changes only the two relevant corpora(1) calls to corpora(2) with comments; the >=4 assertions are retained. build_original.py archives the experiment HEAD and restores only src/cells.rs from365, so both planned arms inherit the same fixture corrections and harness. That recipe establishes intended matching, not evidence that both builds' final oracle runs completed. Source-generated fixtures are distinct from the separately pinned seven-family benchmark inputs.

## Scope and completion

The frozen PLAN2a27255 and exact-head SAFETY argument were read. The source-saving formula does not establish traffic, optimized code shape or runtime benefit. At dispatch this head had no associated PR, zero check runs and zero legacy statuses, aggregate pending: not green. Both-build seven-family full-reader oracles/route parity, optimized codegen and matched CI were explicitly unfinished in the supplied checkpoint. They remain uncredited here; H1 is separate from those integration/performance obligations.

The pinned task and migration checks passed; receipt ab9ad8fbc941cb0095dbc51fa677a2790671381c precedes this work. Pinned coordination helper audit since prior assignment6853304419 found no routing defects in fresh connector snapshots; route returns REVIEW on145. No implementation, other-role state, inbox write, merge, queue, workstation execution or automation setting was changed.
