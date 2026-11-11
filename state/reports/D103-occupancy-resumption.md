# D103 — targeted accounting resumption at bfd5d322

Checkpoint [6790746386](https://github.com/lmmx/mibz2/pull/161#issuecomment-6790746386), dispatching QUEUED6790684381.
Reviewed PR #368 head **bfd5d322651974f9803f4c03942cf3ba3b5161d2**, base **2635f04e9155dd31388b24fd3f7248f87b3ddcda**.
Authority: SPEC/SPEC.GUARDIAN, MODE, RULES and PROTOCOL at **a2da49638ac54d7763de1c167bd054f67ffdc763**.
Original D100/D102 reviewer, independent of implementation. This is the one-commit delta from4d8bd424, not a full review repeated.

**G1: PROCEED for the corrected, explicitly limited allocation accounting. Record B6 resumption at this source pin: D100/D102's accounting defect is resolved. Lift the scoped accounting restriction for these repaired ordinary-worker/team paths.** The old source heads and failed attempts remain failed; they do not acquire this result retroactively. No new G2/G3/G8 restriction.

**This does not make #366 ready.** Its old exact-head Occupancy failure remains. The Coordinator must reconcile incorporation of the repair and all applicable exact-head integration/readiness requirements before any admission. Miri was still in progress at this run's bounded snapshot. No all-green declaration, merge, ready tag, queue entry, Run signal or workstation permission follows. Routine work does not need another Guardian approval merely because unrelated CI is unfinished.

## Request bound: cause fixed

The delta changes only tracing/accounting, its tests, documentation and preserved evidence. No decoder allocation/kernel code changes. D102's accepted ll/rle ownership, retained history, integer validation before aggregation, metadata checks and failure propagation remain intact.

src/lib.rs now passes the actual Kernels value to record_allocation; the snapshot records **avx2 = target x86_64 AND k.avx2**. This is selected configuration, not a claim that every cooperative block executed the one-core AVX2 fill. In src/trace.rs, WorkerNeeds keeps:
- maximum logical rows n;
- independent ll and rle request maxima, unchanged;
- maximum possible tt request **u = max(rows +8 for Runs with selected AVX2; rows otherwise)**.

The tt lower bound remains **4n**; upper becomes **4×max(4,2u−1)**, with zero request requiring zero capacity. This is the right asymmetry: cooperative parallel build reserves n; its possible one-core fallback reserves n+8 when AVX2 is selected. Requiring the slack in the lower bound would falsely reject the parallel path. Including it only in the upper covers both without using observed capacity to set its own bound. A later column/smaller block retains u, so reuse cannot erase earlier slack.

The finite upper follows the same current Vec growth model as the existing ledger: growth requires old_capacity<request≤u, hence doubling cannot exceed2u−2; a direct request is≤u; the minimum is4 words. This is a source/toolchain model, explicitly documented as such, not a Rust API guarantee. Logical live/hot rows still exclude the slack. Schema route-history-v2 exports the AVX2 flag, making the bound independently reproducible. Successful run and column route capture and validation ordering remain as reviewed in D102. The source diff introduces no tolerance and suppresses no errors.

## Native regression reach

The matrix now covers **20 sequences**: two decoder types × scalar/detected × five route/reuse cases. It keeps route/stream-bound assertions, bytes/CRC, independent row counts, impossible ll/rle/tt capacity rejection and the previous mixed-route witnesses. New99,992→100,000-row case is below the cooperative threshold and checks detected-AVX2 tt capacity **800,000 bytes**. The passing native log covers both Segmented and Cooperative<2> detected variants; focused ledger4/4 and occupancy5/5 pass. Native transport Workers1/Workers2/Teams{1,2} remains checked.

The old-bound mutant log shows compilation completed, then a runtime assertion on Segmented/simd-slack/detected: capacity800,000 exceeds799,996, including the final repeated check. It is not a compile-fail kill. The handoff identifies the mutation as restoring only the old upper expression; the archived log demonstrates the claimed rejection and its location. I did not rerun Rust or independently reconstruct that mutant binary.

Independent arithmetic reproduces requests100,000→100,008 words and retained capacity100,000→200,000 words. Corrected upper is800,060 bytes; old799,996 rejects the actual800,000. The observed mutant and this derivation address the exact D102 omission, not an unrelated corruption test.

Nine selected source/text-evidence SHA256SUMS entries independently verified, including trace **4a7228090164c24dad5f5b24064b847476d327fd3e120633433ddbe01ac87ff3**, occupancy **c98921bbeb958cd2bb9144d22002031af6690b3699333dd7935a72cc8af3e5c8**, lib.rs, documentation, native and mutant logs. The [digest record](../evidence/D103/source-digests.json) lists exactly what was verified; no blanket archive-validation claim.

## Exact repaired CI and independent raw replay

[Occupancy42984639389/job130402788066](https://github.com/lmmx/mibz2/actions/runs/42984639389/job/130402788066) completed **SUCCESS**. Log checkout is **fa03b46de14cd507e9672462e653f5343f806365**; fetched Git commit confirms parents2635f04e andbfd5d322. Instrument tests pass99library/1ignored and5example. Ordinary occupancy, forced teams and upload all succeed.

Downloaded [artifact13838404688](https://github.com/lmmx/mibz2/actions/runs/42984639389/artifacts/13838404688), containing ordinary/team JSON and text. ZIP SHA-256 **f2e7c1d0d421a858e8efde116aa9bf3d2e06650552746a036022589d251af2e7** matches the job's upload digest.

Independent [replay.py](../evidence/D103/replay.py) parses every raw run, checks plan/output/error identity, completeness, zero reported ledger violations, worker-history counts and431samples per traced run, and recalculates ll/rle/tt integer bounds and nondecreasing capacities, plus exact counts1024/1024. [Result](../evidence/D103/replay-output.json):

| evidence | traced rounds | worker histories | block snapshots | independent ll/rle/tt errors |
|---|---:|---:|---:|---:|
| ordinary workers1/2/4, six rounds each mode | 36 | 84 | 15,516 | 0 |
| forced teams2x2, two rounds each mode | 4 | 8 | 1,724 | 0 |

Both arms report output209,700,196 bytes, hash a8830caee5c511a6, no decode errors, parity true; summaries reconcile events. Ordinary traces record13,788Runs/1,728ColumnInPlace; team traces1,532Runs/192ColumnSeparate. All snapshots record selected AVX2. The ordinary new sample set would produce3,761 old-upper snapshot failures; corrected bounds produce zero. That count belongs to this new attempt; it does not replace the old attempt's3,949 snapshot or3,973 total-check failures.

The independent script does not separately reconstruct segment metadata or final buffer equality from a field not exported in raw history; those unchanged source checks run in the native instrument and report zero errors. Prior D102 metadata findings stand. This is raw-evidence replay plus source review and inspected native/CI evidence, not a fresh native decoder run or timing by the Guardian.

## Failures preserved; overhead remains bounded

The old failed artifact is now committed gzip-wrapped. Independently verified its wrapper SHA256 **2a069af2a0328c7ef849aeeacad7a1d4d5a450d3a239ea206878ab08b64ddd86**; decompression recovers exactly D102's ZIP digest **62d397b01c7f6f8c60db7c61f4694097a0b36a83ce765e1d41edf2733f2d5417**. Initial fixture/lint/topology logs and prior native logs are not modified by this delta. D100 and D102 remain adjacent historical verdicts. This is a changed-source run, not an unchanged-head retry.

All eight ordinary/team timeline/full throughput intervals contain1 under the **unchanged** overhead rule. Ordinary six-round intervals span0.939964–1.130328 across configurations. The two-round team intervals are very wide: timeline **[0.817027,1.236774]**, full **[0.571351,1.621146]**. Classification is **no perturbation shown**, not zero or negligible overhead, not equivalence, and not evidence of a performance gain. No threshold, workload, estimator or repeat rule was relaxed. Exact estimates remain in replay-output.json.

One fresh exact-head check-runs snapshot showed Benchmark, walk-stats, native/fuzz/Python, build/docs and platform jobs successful, release/deploy jobs skipped, **Miri in progress**. Those are check statuses, not an independent benchmark-policy adjudication or blanket B5 certification. No poll or discretionary rerun followed.

Bounds remain limited to ll/rle/tt/counts and the justified one-core metadata. Run scratch, checkpoints, cooperative cuts/segments, segment byte buffers, tracing metadata, allocator/RSS and cache residency remain outside this certification. Pipeline/service routes lack the required histories and gain no passing verdict. #365 and security/bridge design are outside this delta.

Expected: fix the request model and prove native reach with repaired ordinary/team CI. Observed: request-correct bound, compiling regression kill, native pass and complete zero-violation raw evidence. Gap closed: D100/D102's accounting cause. Next consequential action belongs to the Coordinator: complete remaining integration/readiness checks at the appropriate exact head. This review authorises no timing, merge, signal, cap/default/threshold change or new infrastructure.

## Publication provenance

Receipt **b534f2b88b93358c792d2dfb8fa135a11f692cab** follows successful migration check: old branch f9dab3d2b40739d89d613f86e8c2d67bec3159e4 and manifest60df88f62692909a122e174463e94c012b28e39ab08c6c8a765d41e762a72488. D102 is finished; the dispatch's d87ff14b85e11b01fa3ee7bd5081ee47122cbcd2 is its cursor blob, not a branch commit. Current pre-receipt head8c171ce9adf7b301675a8bbb4425ad5a10084397 confirms that state. Only this item unfinished; corrected evidence pin is new. Routing audit retains five resolved historical findings and previously recorded #162 LOG6757065549; REVIEW destination#145. Task unchanged.
