# Parallel scan: chain acceptance supported; REVISE operational readiness

Assignment [6755779354](https://github.com/lmmx/mibz2/pull/162#issuecomment-6755779354), bench-par-scan only. Reviewed frozen source **eed6cc7705c329ed685c00411cf2330e11b95752**. Authority: docs/team SPEC, SPEC.RESEARCHER, MODE, RULES and PROTOCOL at **a2da49638ac54d7763de1c167bd054f67ffdc763**. Receipt committed after migration head/digest verification.

**REVISE before queuing.** The chain-acceptance design answers the false-magic objection for the inspected in-memory, deterministic parser path. The prototype does not inherit the titration's hog-placement confound. It does inherit failure-path waiting without a deadline. Resolve that and narrow the result interpretation; the comparative diagnostic then has good information value.

## 1. Why false magics do not become output

The invariant is stronger than “valid CRC means a block.” At the start of each chain iteration, `bit`, stream `level`, block index and combined CRC equal the sequential parser's state after the already accepted prefix.

In `chain` (examples/par_scan.rs:487):
- stream headers are parsed at the authoritative byte boundary;
- a candidate is considered only when the input at the authoritative bit is a block magic;
- `take` discards all earlier candidates and accepts only the candidate at that exact bit;
- a worker outcome decoded at the wrong level is discarded and reread inline;
- both paths use `read_item` then `decode_block`; their end bit advances the chain;
- the combined CRC, end marker, next byte alignment and next stream header are handled by the chain.

I compared this with src/stream.rs and src/lib.rs at the same SHA. `decode_block` rolls back appended output on RLE or CRC failure. Worker errors are surfaced only at the corresponding authoritative bit/index; speculative errors elsewhere are discarded. Thus a complete valid block embedded inside another block cannot redirect the chain just because its own CRC succeeds. A missed candidate falls back to the authoritative parse once the finder has passed that position.

This supports semantic equivalence **assuming the finder progresses, tasks/results remain correctly indexed, the shared parser terminates normally, and resource failure does not intervene**. It does not certify streaming I/O, cancellation or all allocation/failure paths. The journal already excludes several of those. I did not rerun the claimed six adversarial files or 4,000 mutations: rustc is unavailable here. Those remain reported coverage, not independent native replay.

The useful conceptual prior art is already in the repository's speculative-scan plan: discover candidates cheaply, validate by exact parsing. The advance is this concrete authoritative-chain implementation and its comparison, not the assertion that a 48-bit magic cannot occur in payloads. No external priority claim is established by this review.

## 2. Shutdown is correct on ordinary returned errors, not on panic

Both arms set their stop flag and notify waiters after their ordinary joiner result, including returned decode errors. This is better than an unconditional infinite worker loop. Nevertheless:

- In `decode_find`, a worker can panic after taking a task and before inserting its outcome. `take` waits indefinitely for that exact result. There is no worker-failure state or notification that resolves it.
- A finder panic can leave `found_upto` permanently short of the chain's next position, with the same result.
- In `decode_scan`, a worker panic can leave a missing result at `taken`; a scanner panic can leave `end` unset.
- A joiner panic bypasses the normal stop assignment. Scoped-thread joining then waits for workers which can remain asleep on their condition variable.

Recovering a poisoned mutex does not repair missing work or announce thread termination. The spawned handles are not explicitly monitored. These are source-level failure paths, not a reproduced native hang.

The recipe calls the check and knee children without timeouts. Its EXIT trap bundles only after control returns; it cannot bound a child deadlock. Required revision: propagate worker/finder/scanner failure into shared terminal state, notify all waiters, arrange stop on unwind and ordinary exit, and enforce a finite external deadline with process-group cleanup and preserved failure evidence. Fault-inject a panic at each producer boundary and a stalled child in the untimed recipe test. This is bounded-failure evidence, not another performance run.

A second resource qualification: slots bounds issued speculative jobs, **not all discovered candidate metadata**. The finder appends to `FindState.tasks` over the entire input and never erases its prefix. Crafted candidate-dense input can grow that vector with file length. For the three bounded in-memory diagnostic inputs, record this limitation and peak allocation; for production, add actual discovery backpressure and retained-input bounds. Do not claim “false magic costs only time” without the memory and liveness qualification.

## 3. Placement and timing: which earlier objections apply?

There are no hog threads, CPU pinning calls or claims of constant isolated interference here. Unpinned scheduling is an explicit limit of the registered harness comparison. I do **not** require importing the titration's core-isolation design merely because this is also threaded.

At t workers there is a caller/joiner plus a finder or scanner thread, as well as the workers. Ten workers is not a ten-thread CPU budget. Record allowed CPUs, topology, quota and the actual arm/worker/slot settings; retain the unpinned scope. The two arms also differ in symbol-buffer ownership, candidate storage, locking and fallback work. Their wall ratio measures the complete constructions, not a pure causal estimate of moving Huffman work.

Every timed cell checks its full output against the sequential reference outside the measured interval, and child failure propagates through the shell. These are positive features. New threads, joins and allocation are charged. `cpu_micros` calls getrusage without checking its return: make failed CPU collection explicit rather than trusting partially meaningful deltas. The file contains that FFI unsafe boundary as well as the AVX2 filter, contrary to the journal's narrow unsafe summary.

The AVX2 loads have explicit end checks and runtime feature detection, and scalar verification checks full marker bits. Source inspection is not completion of RULES B5. Supply the scoped safety argument and test evidence, including unsupported Miri coverage limitations, plus the liveness fault tests. No concrete UB is alleged.

## 4. Readiness and result meaning

Three reporter tests pass locally by direct invocation. Type, nonfinite, missing/duplicate-cell and inconsistent-size checks work in those tests. A uniform wrong-size fixture still passes: every cell with bytes=1 yields FASTER. This does not evade the native per-cell output comparison, so it is an independent artifact-admission gap, not a demonstrated wrong decode. Pass the independently recorded expected size into reporting and check it.

The recipe records the original corpus's decoded digest while recompressing it, but does not independently decompress the generated level-1 file and assert its decoded digest equals that original. Its harness compares each input with that input's own sequential decode. Add the promised cross-input identity assertion, and retain compressor/version and generated input digests. The input directory is removed before bundling, so reproducibility rests on those recorded identities and generator details.

The reviewed anchor does not itself contain bench/par-scan.frozen. That can legitimately be added in a subsequent commit pointing back to the anchor; verify the complete ready commit's measured-file equivalence. After revisions, re-freeze, complete the untimed end-to-end recipe test and exact-ready-head CI. No successful dry run or green-CI claim is made here.

FASTER at median r≥1.15 can be a useful **descriptive construction-selection threshold** for this diagnostic, not statistical proof of a population gain. NOT FASTER at r≤1.05 includes small gains and losses; five-round extrema are not confidence intervals. Keep all rounds and all three input outcomes visible, without retry or selective reporting.

Most importantly, NOT FASTER on two inputs cannot prove “something else holds the workers” or close every speculative-scanner construction. The added parsing, scheduling, allocation or speculative work may hide the scanner saving. Close this measured arm's claim; inspect the excess work before closing the idea. Similarly, Wikidata FASTER would challenge the model's prediction, not prove that memory pressure is absent. The preceding bandwidth review explicitly left the memory cause unresolved.

The ideal scanner throughput ceiling relative to a sequential decoder is 1/s if scanning has share s and everything else is free to scale. The balance in **workers** is (1−s)/s because their work excludes scanning; it is not exactly 1/s workers. Neither quantity establishes the real multicore ceiling without charging worker supply, join/copy work and memory effects.

## 5. Consequence

Expected: worker-side symbol reading removes a serial supply limit, especially at level 1 and on records. Observed here: no new timing; chain source checks and three reporter tests pass, while failure handling remains unbounded. No production gap is demonstrated closed.

Next value: retain the paired scan/find sweep after the bounded-shutdown, evidence and interpretation revisions. It directly compares two implementable constructions and offers better decision value than inferring a unique bottleneck from an artificial load. A positive result would justify a separately reviewed production integration candidate with streaming/resource/error obligations; it would not qualify that candidate.

Research only. No source implementation, timing, queue/tag action or merge. Routing audit since the prior REVIEW found no violations. Attached replay.py and replay-result.json document the reporter replay; use reporter and tests from the exact reviewed SHA.
