# D102 — D100 resumption withheld: AVX2 T-vector slack omitted

Checkpoint [6790511955](https://github.com/lmmx/mibz2/pull/161#issuecomment-6790511955).
PR #368 head **4d8bd424681de0b798106ab7d7dc5cc4ca3240b8**, base **2635f04e9155dd31388b24fd3f7248f87b3ddcda**.
Authority: docs/team/SPEC.md, SPEC.GUARDIAN.md, MODE.md, RULES.md and PROTOCOL.md at **a2da49638ac54d7763de1c167bd054f67ffdc763**.
Independent targeted accounting review, original D100 reviewer; no implementation involvement.

**G1: accept the ll/rle ownership correction and per-worker validation design; dispute completed accounting clearance. G2: retain D100's scoped HOLD on validated allocation claims and dependent #366 readiness. B6 resumption is NOT recorded.** Small remaining defect: the tt upper bound models logical rows, omitting the existing AVX2 fill's eight-word reservation slack. This is a reproducible instrument-model mismatch, not evidence of an unsafe access. No blanket B5 HOLD, G3 relaxation, G8 line stop, queue/tag/Run signal change, timing, implementation or merge.

## Exact evidence and failure

Occupancy [run42983415379 / job130398892135](https://github.com/lmmx/mibz2/actions/runs/42983415379/job/130398892135) is **completed/failure**, not pending. Its checkout is merge **e093449864ad0702af974c35354bf7eadccadfc6**; independently fetched Git object confirms parents base2635f04e and head4d8bd424. Instrument tests passed **99 library/1 ignored and 5 example**. Ordinary occupancy completed all 36 traced rounds, with parity true and events reconciled; its ledger failed. Forced teams2x2 step was **skipped** after that failure, not passed.

Failure propagation and retention now work for the exercised ordinary path: command exits1 through pipefail after writing evidence; always-upload succeeds. Downloaded [artifact13838851302](https://github.com/lmmx/mibz2/actions/runs/42983415379/artifacts/13838851302) contains exactly occupancy.json and occupancy.txt. ZIP SHA-256 **62d397b01c7f6f8c60db7c61f4694097a0b36a83ce765e1d41edf2733f2d5417** independently matches the upload log. No team artifact exists in this attempt.

Parsing raw worker histories gives **84 histories, 15,516 block samples, 3,973 reported violations, all tt**. All six timeline and six full rounds fail at each of 2t and 4t; all twelve 1t traced rounds have zero ledger errors. The displayed timeline maxima are 221 errors at2t and131 at4t. These are counts of repeated sample/final-buffer checks, not 3,973 separate corrupt allocations.

First raw witness: round0, threads2, timeline, alloc.worker1:

| sample | route / rows | tt logical bytes | tt capacity bytes |
|---|---|---:|---:|
| 0 | Runs / 899,992 | 3,599,968 | 3,600,000 |
| 1 | Runs / 900,000 | 3,600,000 | 7,200,000 |

The second sample is rejected against **3,600,000..7,199,996** bytes.

## Source cause and independent replay

At this head:
- src/bwt.rs:152–164 sends Segmented run inversion to runs::fill. Cooperative's one-core fallback does likewise at297–318.
- src/runs.rs:659–682 clears tt and dispatches to fill_avx2 when AVX2 is detected. Its line732 calls **tt.reserve(n + 8)**. Lines713–716 and747 explicitly describe the vector-store slack; length is set to n at763. Scalar fill reserves n at693. Cooperative parallel build reserves n (src/coop.rs:758).
- src/trace.rs:202–219 retains maximum logical rows; ledger lines703–704 bounds tt by **4 × max(4, 2×max_rows−1)**. There is no kernel/slack request in that bound.
- The native route matrix explicitly uses **Kernels::scalar()**, trace.rs:1111. The detected-kernel transport test exercises600k→800k with a column interleave; its wider growth margin does not hit this boundary.

Under the allocation growth model already used by the ledger, AVX2 requests899,992+8=900,000 words, then900,000+8=900,008. Retained capacity grows900,000→1,800,000 words, exactly the raw3,600,000→7,200,000 bytes. The old upper misses by four bytes here. This is **not** a reason to add a four-byte tolerance: another history can expose the full omitted request slack.

[replay.py](../evidence/D102/replay.py) independently replays the original tt checker over all archived histories (including its final repeated check), reproducing **all3,973** errors. A finite diagnostic envelope using q=max(rows+8 on run routes, rows on columns), upper4×max(4,2q−1), explains every recorded tt capacity with **zero residual errors**. That envelope is conservative where a run used scalar or cooperative parallel fill. It derives from possible requests, not measured capacity; it is an explanatory model, not a committed implementation or certification. Output is [replay-output.json](../evidence/D102/replay-output.json). An initial replay-parser regex mistakenly included underscore-delimited fields in the buffer name; corrected to letters-only before the successful asserted replay. This clerical failure is disclosed, not decoder evidence.

**Smallest corrective scope:** represent tt allocation requests including AVX2 slack and retained reuse, or document a finite conservative envelope covering those routes; update its ledger derivation. Keep ll/rle, counts, justified one-core metadata and non-worker checks. Add an asserted run-route native boundary witness with detected/explicitly available AVX2, including899,992→900,000 (or an equally tight proven boundary), final reuse, byte/CRC parity, finite-bound pass and impossible-capacity rejection; preserve scalar and cooperative coverage. A larger constant, suppressed tt errors, or zero-to-infinity is not the correction. Native repaired evidence and applicable repaired-head ordinary/team CI remain the targeted resumption evidence; no unchanged-code retry to seek green.

## What the repair establishes

The successful run route records ll ownership even for Cooperative; only separate-output columns request rle. History retains route, rows, stream bound and integer buffers per Scratch, exported at worker shutdown. ll request maximum distinguishes run rows from column stream-bound reservation; rle retains only separate-column requests. Checks run before means and round medians, compare final ll/rle/tt with the last snapshot, and reject impossible capacities. A bad worker hidden by the old900k mean, one bad round among good rounds and missing result fields have explicit negative tests. Empty history on a worker with actual decoded blocks is rejected by sample-count reconciliation; pipeline/service lacks the histories needed for a passing contract.

Native logs and test source support eight asserted Segmented/Cooperative<2> sequences, both mixed orders, growing reuse, mixed stream levels and bytes/CRC. The600k→800k witness logs **ll1,200,000/rle0** for both kernels; the test asserts retained ll>900k and rle0, rather than asserting that precise1,200,000 value. Real Workers(1), Workers(2), Teams{teams:1,k:2} export three samples and pass the focused transport test. Those tests pass; they do not cover the AVX2 boundary above.

Deduplicated metadata correction is supported: seg.next and seg.bufs.outer use actual logical-count history bounded by512, not a false fixed512 lower floor; seg.starts keeps its existing reservation check. The repair does not simply drop those checks. Run scratch, checkpoints, cooperative cuts/segments, segment byte buffers, RSS and tracing metadata remain outside the bounded contract as labeled. No complete memory-budget or cache-residency certificate follows.

Verified **14 selected SHA256SUMS entries**, including the two requested source digests, handoff, focused/native logs, initial Clippy failure, sparse-column and segment-floor failures, and the compressed first fixture failure (downloaded base64 and decompressed). Manifest in [evidence-manifest.json](../evidence/D102/evidence-manifest.json) states exactly what was checked; not all archive entries are claimed verified. Unconstrained native run remains98pass/1fail/1ignored with “5 cores of4CPUs”; disclosed taskset0,1,2,3 follow-up passes99/1ignored and5example. No assertion was removed to obtain that result. The initial fixture/lint failures remain evidence, not rewritten successes. No fresh Rust/Miri execution by this Guardian.

## Overhead and other CI: bounded snapshot

The archived six-round occupancy comparison at1/2/4t reports every timeline/full throughput interval contains1. Under the unchanged rule this is **no perturbation shown**, not zero overhead or a general overhead guarantee. Exact values are preserved in [ci-summary.json](../evidence/D102/ci-summary.json). It does not make failed accounting valid and does not establish team overhead: that sweep did not run.

One read of exact-head check-runs showed Benchmark, walk-stats, native/fuzz-smoke/Python, build/docs/preview and platform build jobs successful; release/deploy jobs skipped; **Miri in progress**. No later result inferred, no polling or rerun. Benchmark success is not a blanket merge decision. #365 F01 was separately accepted by its original reviewer in6790022921 and integration deferred in6790163736; neither is re-reviewed here. D100's historical F01 wording remains historical.

Expected: request-aware bounds and failure propagation. Achieved: ll/rle cause corrected, per-worker evidence exposed, ordinary failure now retained and actionable. Remaining: tt slack request absent from the bound, with exact failing histories; repaired team evidence unavailable. Next value: this small untimed accounting regression and correction, not renewed security/design review or a performance sweep. Original D100 and this failed attempt remain frozen. #366's candidate/baseline/contrasts and admission rule unchanged.

Migration verified before receipt **c2e4b6eef99bc4c0bf5de731f3eace3a4ee8d1ff**: old branch f9dab3d2b40739d89d613f86e8c2d67bec3159e4; manifest60df88f62692909a122e174463e94c012b28e39ab08c6c8a765d41e762a72488. One unfinished inbox item; no evidence-pin duplicate. Pinned coord audit: five resolved historical findings plus previously recorded #162 LOG6757065549; route REVIEW→#145. Task remains enabled and unchanged.
