# D108 — PR #444: staged rules, ordering evidence HOLD

Observed 2026-11-15 21:53 UTC. Inbox [6901903478](https://github.com/lmmx/mibz2/pull/161#issuecomment-6901903478). Reviewed head **12ae88b03d2f7ce2114b71e98dffc69455a2c51c**, base 827c3dd55ec3e3ca1214a90411340a26284b5536. Authority: SPEC / SPEC.GUARDIAN / MODE / RULES / PROTOCOL at a2da49638ac54d7763de1c167bd054f67ffdc763.

## Decision

**G1: source bounds and flush/fence control flow concur; full B5 qualification is not established. G2: HOLD adoption/integration of this NT-store candidate until the ordering mutation gap is closed.** This is a scoped B5/B6 evidence hold, not a demonstrated defect in the unmutated implementation and not a stop of the Cells line. Corrective evidence work may proceed. No merge, timing, infrastructure or policy change is authorized here.

The surviving removal of the redundant alignment check is explained by a stronger established precondition. The surviving removal of the fence removes an essential safety operation. These survivors are not equivalent. A mutation check aimed at that operation currently misses its target; RULES B6 cannot be discharged by noting that ordinary output tests do not expose it.

I did not implement this candidate. I independently inspected the source delta and surrounding callers, reproduced the interval/alignment arithmetic in a separate model, inspected the mutation script, compared head/CI source trees, and read completed fuzz logs and the Rust intrinsic contracts. No local Rust compiler was available on PATH; I did not run Rust, Miri, the candidate mutation script, or timing. Implementer claims are labelled below.

## Bounds and representation

- Safe entry bounds budget <= n, records <= n and n < 2^24. The existing private layout/induction coupling supplies the same budget. The new round-up adds at most 15 bytes before the rules and is included in allocation and the u32-offset refusal. No new unchecked external layout enters.
- Rules begin at a multiple of 16; rp increases by 16 only for rules. STAGE is 128 in tests/Miri and 1024 otherwise. Thus q = rp % STAGE is at most STAGE-16: both 8-byte stage writes are in bounds.
- Every full send is [max(rp+16-STAGE, rules), rp+16). The final send is [max(rp-rp%STAGE, rules), rp). These intervals partition exactly the rules produced, including the empty final interval after an exact fill. They never cross a ring boundary or include unstaged bytes. The ring is reused only after the previous range has been copied out.
- send_rules computes alignment from the actual destination pointer, not the offset alone. Its plain prefix reaches a 64-byte boundary, its four unaligned source vector loads cover a complete staged line, and its four stream stores land on 16-byte boundaries. The plain suffix is disjoint. Consecutive sends do not rewrite bytes with outstanding NT stores.
- Independent model: **807,936** combinations of ring size, every 16-byte start residue, rule counts through three fills plus one, and all 64 destination base residues; **2,026,880** send intervals and **17,281,470** whole lines. Partition, source bounds and alignment assertions pass. See D108-geometry.py. This is arithmetic validation, not native execution or a memory-model test.
- The budget countdown checks zero before decrementing, preserves the old per-cell limit and returns budget-left. Rule and arena consumption remain bounded by that count.

## Reads and ordering

Small-word materialization runs only when la+lb <= 16, where la/lb include MARK. Therefore neither factor can denote a long or marked staged rule. The test hook now reconstructs a rule from its factors; the independent comparison additionally checks the completed rule region byte-for-byte.

The widened fetch-ahead load is rd64(top+8), covering bytes +8..+15; the old offset-only load covered +12..+15. FAR's padding still covers the lower endpoint. Bottom's arbitrary initialized offset is clamped to last_top, so even a non-slot offset reads inside the initialized allocation; its resulting address is only a wrapping prefetch hint. Valid long/marked words hint the single-byte region. This does not assert that arbitrary stale hint values are semantically valid words.

**The fence is required before same-thread reads too.** The journal's statement that a thread reads its own NT stores correctly is not the Rust safety argument. Rust's [_mm_stream_si128 contract](https://doc.rust-lang.org/core/arch/x86_64/fn._mm_stream_si128.html) requires a same-thread sfence before subsequent access to the affected memory; [_mm_sfence](https://doc.rust-lang.org/core/arch/x86_64/fn._mm_sfence.html) explains that NT writes are modelled asynchronously and can race with same-thread accesses.

The actual code installs a named Fence guard before any send. It drops after the success-path final send, on budget/error returns after a send, and during ordinary unwinding. try_invert checks canaries and invokes the writer only after induce_cells has returned, hence after that drop. Initial refusals precede all stores. No normal induction read of streamed rules was found. An abort terminates rather than reusing the buffer. This source argument supports the present code, but does not turn a surviving fence-deletion mutant into a killed mutant.

On x86-64 the used intrinsics require only baseline SSE2/SSE; there is no AVX requirement or runtime CPU dispatch obligation for ordinary x86-64 targets. Other architectures use direct scalar writes (STAGED=false). Miri on x86-64 exercises the stage and copies but compiles out streaming stores and the fence. Miri cannot establish their native ordering. The registration's prose suggesting staging on every non-x86 target differs from this scalar fallback, which is explicitly described in the implementation journal; no unsafe intrinsic leaks into that path.

## Evidence and the two survivors

The committed mutation TSV reports 14/16 killed. The script uses debug cargo test --lib cells, checks compilation separately, and restores the source after the loop. Its restoration is not protected by finally against interruption; no interruption is claimed in the submitted result. Raw mutant process logs/exit codes are not in this evidence bundle, so the TSV is an implementer report, not my reproduced run.

1. **Removed lay.rules alignment check:** all current callers obtain the aligned offset from slots; rounding remains. This mutant does not invalidate the reachable invariant. The separate rounding-removal mutant is reported killed. Classify the surviving redundant check explicitly; do not claim its specific defensive rejection was tested.
2. **Removed sfence:** the mutated program lacks the intrinsic's required synchronization. Deterministic output equality can still pass. This is a material missed safety target, even if no native stress test happens to observe stale bytes.

Close the hold with a corrected ordering argument and a reproducible check that rejects fence removal (and loss/early drop of its guard on post-store success/error/unwind paths). An instrumented contract model or compiler-output/control-flow check may complement native tests; it must state its assumptions and limits. A flaky cross-thread stale-read demonstration is not required. Record baseline success and targeted negative failure; retain the present survivor outcome as history. Supply completed exact-source Miri/native evidence rather than relabelling the pending jobs.

At the observed CI snapshot, Rust job 132970010145 and Miri job 132970009903 were in progress; log fetches returned BlobNotFound. They are **unverified here**, not failures and not passes. The journal reports 113 library tests, nine Miri Cells tests, and corpus equality at 1/4/12 threads. The checked-in Miri selector includes all nine and the multi-send reach assertion. Those claims await inspectable completion evidence.

Completed fuzz-smoke job [132970009701](https://github.com/lmmx/mibz2/actions/runs/43765647089/job/132970009701) checked out merge 77ab87be12559df2da463aa033dca680308caf01. Head and merge have identical src/tests/examples trees. Its log records 33,498 Cells cases, seed 42; production admitted 852, forced admitted 17,700, with 290,681,155 forced cells. Classic/pipeline/service/teams smoke modes also report zero failures. This is actual native coverage of production-size staging, not a substitute for Miri or fence detection.

## Registered outcome and limits

LOG 6901430556 made +5% an expectation to report, explicitly not a rejection gate. +16% induction and +3.4% whole-decoder instructions therefore do not violate a count gate. Staging alone at +5% does not replace the complete candidate's count. CI paired bench rule 3 still decides performance; no timing verdict is made here. The expected cache-traffic reduction remains a mechanism hypothesis, and extra fetch-ahead work explains the count cost without proving a gain. The next useful safety work is the bounded ordering check; the next performance evidence is the already-registered paired bench.

Migration manifest and legacy branch head match the cursor. Routing audit after the previous checkpoint found no violations; REVIEW destination is #145. The pinned coord.py verifier hardcodes FROM coordinator even for reviews; exact Guardian header, destination and body are independently checked, with that sole tooling limitation recorded. State history is preserved. No automation settings changed.
