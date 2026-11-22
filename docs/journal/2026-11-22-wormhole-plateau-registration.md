# bench-wormhole-plateau: registration

*22 November 2026. Written and frozen before any data, on `bench/wormhole-plateau`. The recipe
is `just bench-wormhole-plateau` (`.just/bench.just`). The verdict is
`bench/wormhole_plateau.py`'s, tested by `bench/test_wormhole_plateau.py`. The recipe is
bench-open-r5's (#735) with the builds, the thread counts and the rounds changed and its lazy
diagnostic left out; the verdict script is `bench/open_r5.py` with its nobi pass removed and the
memory route added after the wall route.*

**The decision it settles:** whether #739 (`perf/wormhole-plateau`, Wormhole's Phase 1 + Phase 2
handoff and output stack on master) reaches the decoder's default on the i9, where the release
plateaus at 6 to 20 threads. It is an ordinary merge under PERFORMANCE-POLICY.md, judged on the one
machine where the plateau is. CI's paired bench (1 to 4 threads) is #739's own check that nothing
regresses elsewhere.

## The hypothesis

**What changed.** #739's head is master's `96a24d3e` (whose build inputs are v0.5.12's) plus the
port of Wormhole's Interim Handoff #2 (`research/carbon-wormhole` at 8aae99dd7,
`docs/journal/2026-11-21-wormhole-cycle40-interim-handoff2-phase2-stack.md`), the `P2c_Full_T64`
arm. It changes `src/lib.rs`, `src/crc.rs`, `src/rle.rs`, `src/read.rs`, `src/parallel.rs` and
`src/parallel/**`, and `examples/bzcat.rs`. #739's body lists every change, its deviations from
the source, and its correctness review. On the plain workers, the route production takes at every
count of this run:
- **each block's own output buffer:** the run-route block is inverted straight into it (no `ll`
  copy), its RLE1 undone there in place and its CRC taken in 64 KiB tiles as they are written;
- **the output pool kept for full blocks** (64 Ki rows or more); a smaller block takes a buffer of
  its own;
- **the symbols' buffer handed back** as soon as the block's rows are known;
- **a 12 MiB cap on the decoded bytes waiting for the reader:** past it the scanner runs only the
  workers plus 2 blocks ahead, not the window's 28 to 40;
- **the handoff:** ring ready slots, the reader and the scanner each on their own condition and
  woken only by what they wait for, and a job queue that wakes one waiting worker;
- **the input chunks recycled** to the reader thread.

The whole-block reads (`read_block`, `BufRead`) are not on the bench's route: `ab`'s `mibz2` arm
reads through `Read` into a 4 MiB buffer at both builds.

**The source's evidence** (a 2-vCPU Xeon with a 105 MiB L3, not evidence here), against v0.5.12
in the same binary, 15 rotated fresh-process rounds at 4 workers and 5-round medians above:
- **4 workers, Wikidata:** 0.9441 [0.9328, 0.9556] of the wall; peak RSS 44.7 MiB against 60.1;
- **7, 10 and 16 workers (wall, peak RSS):** -5.5% and -34%, -4.6% and -30%, -4.9% and -45%;
  minor faults -50% to -56%, system time -41% to -58%;
- **the source's own harness** drove the arms, with a zero-copy consumer that master's arm did not
  have. Part of its 4-worker wall gain is that consumer's, which the bench here does not take.

**The prior: cycle 31's occupancy at 10 threads.** Wormhole's anatomy of v0.5.12's plateau
(`research/carbon-wormhole` at cee7c3290, cycle 31) found the workers idle about 44% of the time
at 10 threads (its 10-worker, 40-slot cell; its table splits the idle time as `i:rd` 43% and `i:q`
28%), with the reader waiting 72% of the time, mostly on the scanner's reads: the serial scan (about 56 ms of the
modelled 68-73 ms wall) is the critical path. This stack does not touch the scan; Mechanisms'
`ParScan`, which would, is not in it.

**The prediction.** Without `ParScan` the serial scan still caps the plateau, so the stack's gains
here should be mostly memory and system time: peak RSS clearly lower at 10 threads and above (the
cap and the dropped `ll` buffer scale with the workers), fewer page faults, less system time. A
wall gain, if any, should be small: the work removed from the workers is off the critical path
while they wait for blocks. So the memory route is the likelier one to decide. A wall loss is
possible where the cap's shorter lookahead lets a slow block hold up the workers (head-of-line);
the source's heterogeneous corpus did not show it, but this corpus is the judge.

**The risk, named.** The ready cap binds hardest where many workers decode ahead of the reader:
at 16 and 20 threads asked, production runs 7 workers, so its floor (9 blocks in flight) is under
a third of the window (28). If the scan were not the bottleneck, that could cost wall time there.

## What is compared

- **Head:** #739's head, `perf/wormhole-plateau` at
  `23efeae76a9831e19523ca5eba01d1674cbb722b`. *Changed before any data, on 22 November 2026,
  from the first head `8857f9fd`.* The new head adds three commits after CI's first look:
  - the footprint ledger's route-history check for the owned output buffers (`src/trace.rs`,
    `examples/occupancy.rs`, `docs/ledger.md`, and `trace`/test-only sampling in
    `src/lib.rs`), with Miri's path filters widened;
  - the reader's `Taken` trace mark back under the lock (`src/parallel.rs`; trace only);
  - **one change to the decoder:** the input feeder allocates a new chunk zeroed by the
    allocator (`vec![0; n]`, as master does) instead of resizing an empty spare, and keeps
    one spare chunk, not three (`src/read.rs`, `src/parallel.rs`, `examples/bzcat.rs`). CI's
    bench on the second head had shown small inputs (onebyte, runs, pattern) slower at 2 and 4
    threads and records' peak RSS higher; this undoes the cause.
- **Base:** `96a24d3e4f764b3918089b0d5bcd5fe4b9bc1fea`, #739's merge-base with master (#738's
  merge). Its build inputs (`src`, `examples`, `Cargo.toml`, `Cargo.lock`, `build.rs`,
  `rust-toolchain.toml`) are v0.5.12's (`6a36b0e1`), the source's baseline. It is an ancestor of
  HEAD (`git merge-base --is-ancestor "$BASE" "$HEAD"`).
- **Base and master agree.** When the recipe was written, master's head was the base. Should
  master move before the run, the run still measures #739 against its own base, and the
  Coordinator weighs any later master change to `src` at the merge.
- **The contrast, checked by the recipe:** the two builds' `src`, `examples`, `Cargo.toml`,
  `Cargo.lock`, `build.rs` and `rust-toolchain.toml` may differ only in `src/lib.rs`,
  `src/crc.rs`, `src/rle.rs`, `src/read.rs`, `src/trace.rs`, `src/parallel.rs`,
  `src/parallel/**`, `examples/bzcat.rs` and `examples/occupancy.rs`, or the run stops. The bundle keeps the list as `src-changed.txt`.
  `examples/ab.rs`, which drives the run, is identical at the two builds.
- **Routing and admission:** unchanged. The plan, the trimmed workers (6 lazy workers at 6 threads,
  8 at 8, 7 from 10 threads asked) and the windows are as they were; the same blocks take each
  kernel. The single-hash check over every run's output guards the decoded bytes.

## The run

- **The machine:** the maintainer's i9-10900K (10 cores, 20 CPUs, no CPU quota), under C5. Any
  other machine voids the run.
- **The corpus:** `corpora/wikidata.bz2`, the whole bench corpus (431 blocks, 209.7 MB decoded),
  checked by `bench/wikidata.json`'s SHA-256. Four copies end to end make the timed file
  (`wikidata-x4`).
- **The cells:** `ab run --arms mibz2@build-base,mibz2` runs production's decoder at each build,
  unpinned on every CPU (no `--spread`, no spinner), at 6, 8, 10, 12, 16 and 20 threads.
- **Rounds:** 12. Each round runs both arms at each count, interleaved, with the arms' order
  rotated (`arm_order`) and a 2 s settle gate before each run.
- **No diagnostic pass.** bench-open-r5's lazy diagnostic (`mibz2-workers-lazy`) measured the Open
  kernel's route with the plan fixed; this stack is the handoff around every kernel, and production
  already runs lazy workers at every count here.
- **Cost:** 144 runs of about 1.5 s with 2 s settles, about 9 minutes, plus the two builds:
  about 15 minutes in all.

## The rule

A count's **step** is the 12 rounds' paired wall ratios, head over base: their geometric mean and
its 95% interval (Student's t on the logs, 11 degrees of freedom). A step is FASTER if the
interval lies below 1, SLOWER if it lies above 1, and INCONCLUSIVE otherwise. A count's **memory
step** is the same of the paired peak-RSS ratios.

The routes are tried in this order, and the verdict names the first that holds:

1. **ADOPTS (wall):** all three hold:
   - the step at 10 threads is FASTER;
   - none of 6, 8, 12, 16 and 20 threads is SLOWER;
   - at every count, the median peak RSS over the rounds, head over base, is at most 1.10.
2. **ADOPTS (memory)** (PERFORMANCE-POLICY.md §2, "a memory gain is a gain"): all three hold:
   - the memory step at 10 threads lies wholly below 0.90 (peak RSS at least 10% lower beyond
     noise);
   - no count, 10 included, is SLOWER in wall time;
   - at every count, the median peak RSS, head over base, is at most 1.10.
3. **HELD ON MEMORY:** route 1's speed holds (10 FASTER, no other count SLOWER) but the 1.10 gate
   fails at some count. The Coordinator weighs the growth under PERFORMANCE-POLICY rule 4.
4. **NOT ADOPTED:** any other outcome. The verdict says which conditions failed.

- **Holm is not needed:** there is one candidate and one judged count (10 threads), for each
  route; the other counts can only count against adoption (no count's gain is claimed), so there
  is no family of claims to correct.
- **VOID**, with no verdict, if any of these holds:
  - the machine is not the i9's profile;
  - any run is unsuccessful, or has a non-finite or non-positive wall, CPU or peak RSS;
  - any run is pinned or beside a spinner;
  - any record is duplicated, unexpected or missing;
  - the corpus is not `wikidata-x4`;
  - any output is not 838,800,784 bytes, or the outputs' hashes are not all one.

  A VOID run may be repeated once, with the reason logged. A verdict is never re-run to change it.
- **Reported beside each step, deciding nothing:** CPU time, the memory step at every count, and
  the median peak RSS at the base and the head in MiB, with their difference.

**Limits:**
- **The corpus:** Wikidata alone. `text` is not in this run; CI's bench covers it at 1 to 4
  threads.
- **What a result separates:** an ADOPTS shows the whole stack's effect, not any one change's
  share. A NOT ADOPTED does not say which change failed to carry. The source's own ablation
  (cycles 34, 38, 40) is the only per-change evidence, and it is from another machine.
- **The head's branch:** the recipe fetches HEAD from `perf/wormhole-plateau` if the workstation
  lacks it. A later push to #739 is not what this run measures, and needs its own pin.
