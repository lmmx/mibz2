# Parallel-scan readiness re-review — inbox 6756472047

**REVISE the remaining liveness/cleanup details; accept placement, input admission and the operational interpretation changes. No queue-ready PROCEED yet.** B5's addendum must precede queue admission. A conditional endorsement of the design is legitimate, but is not the completed PROCEED required by PROTOCOL §9.

## Scope and evidence

- Assignment: https://github.com/lmmx/mibz2/pull/162#issuecomment-6756472047.
- Instructions: docs/team/SPEC.md, SPEC.RESEARCHER.md, MODE.md, RULES.md, PROTOCOL.md and ROLES.md at a2da49638ac54d7763de1c167bd054f67ffdc763.
- PR #317 ready head: bfe53a2a235a7e31dedaef715fe34d656597bec9. Frozen source: 665b3bde86443cf802dfd6c5c54260004ce1e08d. GitHub comparison shows exactly one difference: bench/par-scan.frozen now names that source. Thus the reviewed measured files match the freeze.
- Read examples/par_scan.rs, .just/bench.just, both Python helpers and their tests, the parallel-scan journal and .github/workflows/test.yml at the ready head.
- Exact-head checks are successful or release-related skipped; Test run 42736232314's “The scan harness's fault and finder tests” step succeeded. This corroborates the native test invocation. It does not establish a Miri run: that step runs cargo test --example par_scan natively.
- Independently executed all seven fetched Python test functions, reproducing their fixtures with a small adapter because pytest is absent. All passed. The four uniformly wrong sizes are rejected; manifest/generator mismatches and a changed level-1 decoded byte are rejected.
- Independently probed the recipe's setsid/timeout construction with TERM-responsive and TERM-ignoring descendants. Result below. This is an untimed liveness check, not performance evidence.
- rustc is unavailable here. No independent Rust fault replay, native fuzz, Miri, complete dry run, or raw dry-run bundle replay. PR description and journal claims of the dry run, guard-removal controls and Miri remain attributed claims, not my own executions.

## 1. Core failure propagation: substantial closure; two residual cases

The old missing-result deadlocks are addressed at source level. Both arms install helper unwind guards before their work. The guards set failed and stop under the shared mutex, release it, and notify all. Workers check stop; take and the scan joiner check failed. The caller's stop_all guard is installed before spawning helpers. Inner mutex guards drop before that outer guard on unwind. Normal completion drops stop_all before joining every helper; the non-short-circuit fold joins all handles, and any helper panic turns even a completed chain into an error. On caller unwind, Rust's scope performs the remaining joins after stop_all has woken them.

The 12 fault cases target the previous holes: worker after claiming, finder before publication, scanner after issue, joiner after acceptance, at 1 and 4 workers. The CI step above passed. The chain's exact-bit/exact-level authority remains; this revision introduces no reason to reopen the false-magic acceptance objection.

**Residual A — the newly added sampler has the same unwind shape outside those guards.** In sampled, examples/par_scan.rs:1119–1139, done is set only after run() returns. If run() panics, the scope joins a sampler whose loop still sees false forever. A joiner panic propagated correctly by decode_find_with/decode_scan_with therefore strands the surrounding sampled check. The fault suite invokes the decoders directly, so it cannot cover this wrapper. The check's 900-second process deadline contains the hang; it does not make sampled unwind-safe.

Repair is small: unconditional sampler termination on unwind, and one fault test through sampled, with an outer test deadline. This is a source-derived path, not a reproduced Rust hang here. No claim that ordinary corpus execution currently takes it.

**Residual B — the deadline does not guarantee descendant cleanup after the direct child exits.** The construction is setsid --wait timeout --kill-after=30 SECS COMMAND. I tested the same construction with a one-second deadline and one-second grace, without changing its control flow:

| grandchild | wrapper exit | alive after grace |
|---|---:|---|
| responds to TERM | 124 | no |
| ignores TERM; parent exits on TERM | 124 | yes |

GNU coreutils 9.7 in this environment. The supervisor completes when its direct child terminates; its later KILL does not clean up this surviving descendant. I explicitly killed the test survivor. The committed probe reproduces both cases and cleans up its own processes. This does not show par_scan presently spawns such a descendant: its decode workers are threads, killed with their process. It refutes the general group-cleanup guarantee and the adequacy of the responsive-sleep grandchild control.

Retain supervision of the known process group through the grace period independently of the direct child's exit, then enforce/reap cleanup; test a TERM-ignoring grandchild whose parent exits. No requirement to defend against arbitrary hostile new sessions is added here.

Two bookkeeping corrections belong with that repair:
- “Every child” is too broad: corpus acquisition, cargo build, reporting and bundling are not wrapped. Either bound the relevant preparation/finalization steps or state the narrower guarantee. The timed decoder and byte checks themselves are bounded.
- Input generation/anchoring timeouts at .just/bench.just:1046–1048 flow through fail(), hence REFUSED, whereas the registered deadline rule says INVALID. Preserve the timeout exit and classify it consistently; neither may produce VALID, and neither licenses a retry.

These are scoped readiness repairs, not a request to redesign the experiment.

## 2. Placement: accept for this unpinned diagnostic

The t+2 accounting is correct for the two decode arms. Placement capture includes process eligibility, CPU topology and available parallelism; the untimed sampled check adds a sampler and explicitly says so. It samples masks, not actual execution on particular CPUs, nor guarantees observing every short-lived thread. No affinity control is being claimed for timed cells, and no hog-isolation requirement belongs here.

The literal /sys/fs/cgroup/cpu.max is only the mounted path read, not a proof of the effective hierarchical quota on every host. Preserve that distinction, especially if a non-root cgroup is used. This does not invalidate the declared unpinned comparison.

## 3. Reporter and input identity: accept the requested repairs

The size anchor is now outside the timed arm: corpus manifest, generator's pre-compression count, and independent libbzip2 decompression. Level-1 recompression is read back and its decoded digest compared with the corpus. The harness also compares mibz2::decode with libbzip2, and every measured cell compares its complete bytes with the reference before emitting its row. Size equality alone is not byte equality; the combination provides both.

The reporter requires the exact cell set and fields, positive integer times/bytes, no booleans as integers, uniform size equal to EXPECTED_BYTES. The seven local test-function executions pass, including the earlier uniformly-wrong-size counterexample. getrusage's return is checked before its values are used; malformed timevals panic. Such a measured child failure cannot become VALID.

Peak RSS is a per-process high-water mark over that input's run, including reference/output and allocator history, not isolated candidate-metadata cost or an arm-by-arm peak comparison. It is useful accounting, with that scope.

## 4. B5: conditional design assent is not queue admission

This review is independent of implementation. Its unsafe argument inspection covers the example's AVX2 finder and getrusage call:
- Finder::detect supplies runtime AVX2 gating; explicit true construction in the inspected differential test follows detection. The vector loop checks k+32 <= to and k+40 <= data.len(); its three 32-byte loads begin at k+1, k+2 and k+3. Their furthest accessed byte is k+34, within the stronger bound. Table reads cover complete 16-byte arrays. Candidate verification is safe, padded, and rejects a 48-bit magic extending beyond the input. Internal callers supply ranges within the slice.
- The repr(C) Rusage argument is explicitly scoped to LP64 Linux x86_64/aarch64. A size assertion verifies the Rust object's size, not by itself the platform C ABI; the stated layout/target premise must remain explicit. Error checking and zero initialization improve the boundary. This review does not certify other operating systems/ABIs, and a monotonic-time test does not replace an ABI argument.
- The reported Miri test exercises finite selected lengths/cuts and enabled AVX2 intrinsics; it is not coverage of all input lengths, concurrency, or the foreign call. Ordinary CI's test bearing “miri” in its name is not Miri execution.

No new unsafe counterexample found in that stated scope. This is substantive independent argument review, not blanket B5 certification. The admitted missing unsafe-targeted fuzz and mutation evidence remains missing. The addendum should identify exact source, commands/toolchain/features, seeds/workload and outcomes; mutations must demonstrate detection of the intended violated obligation. A semantics-only decoder mutation suite cannot be substituted silently for unsafe-boundary coverage. Foreign-call limitations must be stated, not called a Miri pass.

**Answer to the explicit question:** PROCEED with finishing the bounded diagnostic and its evidence is acceptable as a design disposition. Do not use “PROCEED conditional on B5” to queue or run it automatically. Before queue admission, review the completed addendum, resolve these remaining liveness findings, obtain the Guardian's ruling on the safety/admissibility scope, and verify the final freeze/dry run/CI/ready tag. No Louis permission is needed for these delegated prospective repairs. No test-name checklist alone closes B5.

## 5. Interpretation: accept §5, keep stale summaries subordinate

The operational registration now compares complete constructions, charges speculation/ownership/locking, uses 1/s as sequential-relative ceiling and (1-s)/s as ideal worker balance, and labels the five-round threshold descriptive. Those changes answer the prior review. NOT FASTER does not close all speculative scanning or identify a different bottleneck. All three inputs and all rounds remain visible.

The journal's opening and §3 still say the arms differ “only in who reads the symbols,” contradicting §5's more accurate statement. Also “the memory knee binds first” is a prediction, not a consequence established by these data. Edit those summaries without changing frozen thresholds after data. The observed two-mode context reinforces the limits: alternating five pairs does not remove state dependence. Under this narrow descriptive selection purpose that is a limitation, not a demand for a new inferential bench.

## Decision value

Expected: close the previous readiness objections without discarding a useful paired test. Obtained: core panic publication repaired; independently anchored bytes, explicit placement and honest descriptive outcomes. Most of the readiness gap is closed.

Learned: guarding the decoder is insufficient when a newly introduced sampling wrapper owns another scoped thread; a timeout return code does not prove surviving descendants were killed. These are finite repairs with direct falsifiers.

Next value: complete those repairs and B5 addendum, then the same paired diagnostic remains worthwhile. No performance data were collected here; no implementation, queue entry, merge, authority expansion or automation change.
