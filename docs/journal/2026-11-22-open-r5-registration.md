# bench-open-r5: registration

*22 November 2026. Written and frozen before any data, on `bench/open-r5`. The recipe is
`just bench-open-r5` (`.just/bench.just`). The verdict is `bench/open_r5.py`'s, tested by
`bench/test_open_r5.py`. The recipe, the rule and the verdict script are bench-open-stack's
(#720, `docs/journal/2026-11-22-open-stack-registration.md` on `bench/open-stack`), with its
median peak RSS report. Those were bench-open-graded's (#715), bench-open-relay8's (#707),
bench-open-diet's (#693) and bench-open-r4's (#680) before it. The builds differ. bench-open-stack's
nobi pass is left out, because this candidate has no single commit worth isolating that way
(below).*

**The decision it settles:** whether #734 (`perf/open-r5-stack`, "perf(open): Round-5
construction stack (Neon cycles 36–45) on master") reaches the decoder's default on the i9. It is
an ordinary merge under PERFORMANCE-POLICY.md, measured on the one machine where production runs
`OpenCells` on the i9's cache. CI's bench never runs Open there
(`docs/journal/2026-11-20-open-stage-attribution.md`).

## The hypothesis

**What changed.** #734's head is master's `23c1e283` plus one commit. It ports Carbon Kitchen
Sink's `cand_r5_head` (`research/carbon-kitchen-sink` at 25be0973,
`docs/journal/2026-11-21-carbon-kitchen-sink-cycle45.md`). It changes `src/open.rs`,
`src/open/endpoint.rs` and `src/open/outq.rs`, with tests in `src/open/checks.rs` and
`src/open/spirals.rs` (1,116 lines added, 317 removed). The code is safe Rust, with no allocator,
routing or admission change. The ten Round-5 survivors cover every construction stage:
- **S1:** Cap127 with Pt64, S1Trim, OneBr;
- **S2a:** S2aFuse;
- **S2b:** Inline2 with Theorem 45.1's key, FastJoin, S2b-NI;
- **S3a:** NoStats, and S3Tag12, S3Fast, S3Arr and S3a-C8;
- **S4:** BandW and Wire16.

Emission (S5) is unchanged: two-ended `wire_xor` and emit stay as #719 shipped them. #734's body
lists every change, its deviations from the candidate, and its correctness review.

**The native evidence (the handoff's §3, Gemini Neon, `run_c45.rs`).** It was measured against
verbatim `53f7ecf93` in the same binary, on the six Wikidata panel blocks. Its src matches the
base's `src/open*` files. The figures:
- **construction (S1–S4):** 0.8895×;
- **one thread:** 0.9304× in total;
- **two threads:** 0.9507× at p10 and 0.9332× at p50;
- **four threads:** 0.9366× at p50;
- **per block, one thread:** 0.926–0.934×;
- **persistent working set:** +0 B.

These are claims from another machine, not evidence here. The port also differs from the
measured candidate in a few places, which #734 lists:
- a long record merges every following record of its byte;
- the step count is kept under `cfg(test)` only;
- the test-only paths are corrected.

None of these is expected to move the figures. CI's paired bench on #734 is a second, independent
look; it never runs Open on the i9's cache.

**Why it should hold under contention.** All of this work is removed from construction, on the
block's own arrays, at +0 B of retained scratch. That is the kind of gain that held on the i9
before:
- #692's memory diet: 0.952, 0.917 and 0.931 at 6, 8 and 10 threads (ADOPTS);
- #719's construction stack: 0.952, 0.987 and 0.975 at 6, 8 and 10 threads
  (`open-stack/run-20261122T022924Z`).

Construction is about half of an Open block's time natively (the handoff's §6), so if the native
gain carries, the whole-decoder step at 10 threads is several times smaller than 0.89. The cause
is a hypothesis, not shown.

**The risk, named.** The stack trades loads and branches for wider registers and fixed tables, and
two of its parts could fail to carry under contention:
- **The smaller dictionary tables:** 256 pair buckets and 2,048 tagged entries, where master has
  1,024 and 4,096 (S3Arr, S3Tag12). They give fewer cache misses natively, but they evict more, so
  a few more codes per block are made.
- **The 16-wide wiring (Wire16):** a burst of scattered XOR writes per group.

Either could erode under shared-L3 contention at 7 to 8 workers, as #719's last step did
(0.941 natively, smaller on the i9). If it does, the judged count may land INCONCLUSIVE.

## What is compared

- **Head:** #734's head, `perf/open-r5-stack` at `8be1f140f7df62614ce7f23dcac4d6ac92b5e87e`. Its
  src differs from the base's in the five `src/open*` files above and nothing else.
- **Base:** `23c1e283483ca465f92e060b1c0028f1d0404378`, #734's merge-base with master (#730's
  merge), whose src is v0.5.12's. It is an ancestor of HEAD; the recipe checks it with
  `git merge-base --is-ancestor "$BASE" "$HEAD"`, as before.
- **Base and master agree.** When the recipe was written, master's head *was* the base, so this
  run measures #734 against master's head. Should master move before the run, the run still
  measures #734 against its own base, and the Coordinator weighs any later master change to `src`
  at the merge.
- **The contrast, checked by the recipe:** the two builds' `src`, `examples`, `Cargo.toml`,
  `Cargo.lock` and `build.rs` may differ only in `src/open.rs` and files under `src/open/`, or the
  run stops.
  - At these SHAs the changed files are `src/open.rs`, `src/open/checks.rs`,
    `src/open/endpoint.rs`, `src/open/outq.rs` and `src/open/spirals.rs`. The bundle keeps the
    list as `src-changed.txt`.
  - `rust-toolchain.toml` (rustc 1.99.1), `examples`, `Cargo.toml`, `Cargo.lock` and `build.rs`
    are identical at the two builds. The head's harness drives the run.
- **Admission:** unchanged. `OpenCells::cap` and the first pass's refusal (`over`,
  `f.m > cap.m`, `ns > cap.strips`) are as they were, so the same blocks take Open in both builds.
  The single-hash check over every run's output guards the decoded bytes.
- **What it predicts:** a whole-decoder wall gain at 10 threads (7 workers), 8 (8 workers) and 6
  (6 workers), smaller than the native kernel's. For scale, on the same grid:
  - bench-open-diet: 0.952, 0.917 and 0.931 (ADOPTS);
  - bench-open-r4: 0.956, 0.982 and 0.985 [0.970, 1.001] (NOT ADOPTED by 0.001 at 10);
  - bench-open-stack: 0.952, 0.987 and 0.975.
- **Memory, reported:** expected flat. The handoff measured +0 B of steady-state working set.
  The 8,192-band reservation (S3a-C8, 64 KiB a worker) only bounds a capacity the large blocks
  already reach. The 1.10 gate is not expected to bind. The verdict prints each count's median
  peak RSS at the base and the head, in MiB, with their difference. Only the 1.10 gate decides
  anything.

## The run

- **The machine:** the maintainer's i9-10900K (10 cores, 20 CPUs, no CPU quota), under C5.
  Any other machine voids the run.
- **The corpus:** `corpora/wikidata.bz2`, the whole bench corpus (431 blocks, 209.7 MB decoded),
  checked by `bench/wikidata.json`'s SHA-256. Four copies end to end make the timed file
  (`wikidata-x4`).
- **The cells:** `ab run --arms mibz2@build-base,mibz2` runs production's decoder at each build,
  unpinned on every CPU (no `--spread`, no spinner), at 6, 8 and 10 threads. Production runs
  `OpenCells` on the i9 at each of these:
  - 6 threads: 6 lazy workers;
  - 8 threads: 8 lazy workers;
  - 10 threads (the default): 7 workers, `trimmed`, unpinned.
- **Rounds:** 16. Each round runs both arms at each count, interleaved, with the arms' order
  rotated (`arm_order`) and a 2 s settle gate before each run.
- **The lazy diagnostic** (decides nothing): `mibz2-workers-lazy` at each build, at 6 and 8
  threads, 16 rounds, reported as steps only, after the judged run.
- **No nobi pass.** bench-open-stack's third build isolated one commit, two-ended emission. #734
  is one commit, and its ten changes interlock: Pt64 is only a win with Cap127, S3Fast only with
  NoStats. A single "without X" build would not separate them cleanly, and no other comparison is
  registered here.
- **Cost:** about 15 minutes of workstation time with the two builds. bench-open-stack's judged
  run and lazy diagnostic took under 10 minutes.

## The rule

R4's, unchanged. A count's **step** is the 16 rounds' paired wall ratios, head over base: their
geometric mean and its 95% interval (Student's t on the logs, 15 degrees of freedom). A step is
FASTER if the interval lies below 1, SLOWER if it lies above 1, and INCONCLUSIVE otherwise.

- **ADOPTS:** all three hold:
  - the step at 10 threads is FASTER (its interval lies below 1);
  - neither 6 nor 8 threads is SLOWER;
  - at each of 6, 8 and 10 threads, the median peak RSS over the rounds, head over base, is at
    most 1.10.
- **HELD ON MEMORY:** as ADOPTS, but with peak RSS above 1.10 at some count. The Coordinator
  weighs the growth under PERFORMANCE-POLICY rule 4.
- **NOT ADOPTED:** any other outcome: 10 threads not FASTER, or 6 or 8 SLOWER.
- 6 and 8 threads are **supporting**: they are reported with their intervals, and only a loss
  there counts against adoption.
- **VOID**, with no verdict, if any of these holds:
  - the machine is not the i9's profile;
  - any run is unsuccessful, or has a non-finite or non-positive wall, CPU or peak RSS;
  - any run is pinned or beside a spinner;
  - any record is duplicated, unexpected or missing;
  - the corpus is not `wikidata-x4`;
  - any output is not 838,800,784 bytes, or the outputs' hashes are not all one.

  A VOID run may be repeated once, with the reason logged. A verdict is never re-run to change
  it.
- **Reported beside each step, deciding nothing:** CPU time, and the median peak RSS at the base
  and the head in MiB, with their difference.
- **The diagnostic decides nothing.** The verdict is the judged run's alone (`verdict.txt`). A
  VOID lazy diagnostic leaves the verdict as it is.

**Limits:**
- **The corpus:** Wikidata alone. `text` is not in this run.
- **What a result separates:** an ADOPTS shows the whole Round-5 stack's whole-decoder gain, not
  its cause or any one change's share. A NOT ADOPTED does not say which change failed to carry.
- **The head's branch:** the recipe fetches HEAD from `perf/open-r5-stack` if the workstation
  lacks it. A force-push that drops 8be1f140 from every ref on origin would stop the run at the
  fetch, not change what it measures. A later push to #734 is not what this run measures, and
  needs its own pin.
