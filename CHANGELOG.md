# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0](https://github.com/lmmx/mibz2/compare/v0.2.9...v0.3.0) - 2026-11-14

A speed release for Wikidata-like data on up to 4 decoding threads. When the plan is plain workers
(fewer than 6 threads), each block is inverted by **Cells**, a Rauzy–Veech induction over the
block's run records that builds a small grammar and expands it, instead of walking the 3.6 MB
T-vector (#383). Blocks whose runs are too dense (10·r ≥ n) or too sparse (64·r < n) are refused
before any allocation and take 0.2.9's path. The team plans (6 threads and up) are unchanged.
The output, the API and the supported inputs are unchanged.

- **Release scaling, the i9-10900K** (`release-scaling/run-20261114T143213Z`, sha256 802fc690,
  10 paired rounds, every arm the same bytes), `wikidata`, 0.2.9 ÷ 0.3.0 wall:

  | threads | 1 | 2 | 4 | 8 | 10 | 12 | 16 | 20 |
  |---|--:|--:|--:|--:|--:|--:|--:|--:|
  | 0.2.9 ÷ 0.3.0 | **1.161** | **1.034** | **1.092** | 0.998 | 0.996 | 0.983 | 0.998 | 0.984 |
  | 95% interval | 1.120–1.204 | 1.022–1.046 | 1.076–1.107 | 0.988–1.009 | 0.979–1.014 | 0.966–1.000 | 0.974–1.022 | 0.958–1.011 |

  That is 13.9%, 3.3% and 8.4% less wall time at 1, 2 and 4 threads, and level from 8 threads,
  where the plan is the unchanged teams. 0.3.0 is 3.8–6.2× faster than lbzip2 2.5 at every
  thread count.
- **The paired runbook bench, CI** (run 43604267716, Intel Xeon Platinum 8573C, 4 vCPU), v0.2.9 →
  master: `wikidata` −11.9% (−13.6, −10.2), −10.1% (−11.1, −9.0) and −10.3% (−11.9, −8.8) at 1, 2
  and 4 threads; no other corpus slower in any cell.
- **The gate it merged under** (#145 LOGs 6877415527, 6877775115, 6878643235): Wikidata decided.
  It passed I1 at 1, 2 and 4 threads (i9 run 2: 0.858, 0.977, 0.927 new ÷ old). It failed I2, the
  no-loss bound of 1.02, only on interval width at 10 and 20 threads (point estimates 0.999, on
  the unchanged team kernel), and the owner excepted those two cells (6878656114).
- **The cost: memory.** Cells' buffer is about 12.8n + 48r bytes, roughly 15 MB per decoding
  thread: CI's peak RSS 26 → 34 MB at 1 thread, 51 → 76 MB at 2, 78 → 126 MB at 4 (over C3's 50%
  ceiling, excepted by the owner, 6877546534).
- **Downstream:** wikidata-pq#225 (`BZ2_THREADS_CAP` 4, so the split's decoder runs Cells): the
  split's release ETA 1h20m56s → 1h16m46s (−5.3% ±1.9%), parity passed.
- **Caveats.** The gain depends on the CPU: on CI's AMD EPYC runners #383's bench was level at 1
  and 2 threads and −2.9% at 4. Cells does not yet reach the team plans; a phase-split census and
  then a cooperative Cells are next (#145 LOG 6878326831).

### <!-- 2 -->Bug Fixes

- *(examples)* initialise every MTF base with Rust 1.100 Clippy
- *(examples)* iterate checked packet path for Rust 1.100 Clippy
- *(trace)* account for AVX2 run fill slack in retained tables
- *(trace)* validate retained allocations by worker route history

### <!-- 4 -->Documentation

- focus companion on reconstruction and align interval arrowheads
- *(blog)* add detailed Rauzy–Veech technical companion
- *(team)* register Codex delegate and acting Coordinator failover
- *(changelog)* 0.2.9's notes from #359's workstation run and the paired run against 0.2.8 ([#360](https://github.com/lmmx/mibz2/pull/360))

### <!-- 9 -->Other

- Cells (Rauzy–Veech induction) as the Workers plan's inverse, with a sparse admission gate ([#383](https://github.com/lmmx/mibz2/pull/383))
- Release scaling: 0.2.9 against 0.2.8 on the i9 (release-scaling/run-20261110T170952Z, sha256 33469444): wikidata 10-15% faster at 8-20 threads, 0.9% slower at 1; CHANGELOG figures

## [0.2.9](https://github.com/lmmx/mibz2/compare/v0.2.8...v0.2.9) - 2026-11-10

A speed release for many-core machines. On the i9-10900K the decoder's team path, which it
plans from 6 threads up, decodes the Wikidata sample 11–15% faster, because #324's run-record
front now reaches the teams (#359). CI's 4-CPU runners never plan teams and show no change.
The output, the API and the supported inputs are unchanged.

- **Team paths read run records (#359).** Since 0.2.8 the one-thread `Segmented` kernel has
  built its backward table from run records, without writing the last column. The plain
  cooperative teams now do the same: the team's pool fills the table from the records and
  walks it, and a block below the cooperative size takes the one-core fallback. Its
  workstation A/B (`release-scaling/run-20261110T160612Z`, the i9-10900K, 10 paired rounds,
  every arm the same bytes) under a rule written before the data, as base ÷ head wall:

  | corpus | 6 | 8 | 10 | 12 | 16 | 20 threads |
  |---|--:|--:|--:|--:|--:|--:|
  | `wikidata` | 1.163 | 1.151 | **1.138** (1.123–1.154) | 1.117 | 1.108 | 1.150 |
  | `text` | 1.277 | 1.096 | **1.024** (1.010–1.038) | 1.015 | 1.017 | 1.028 |
  | `records` | 1.000 | 1.000 | 1.001 | 1.007 | 0.989 | 1.004 |
  | `random` | 1.002 | 0.998 | 0.998 | 1.017 | 0.986 | 1.002 |

  Every `wikidata` interval excludes no change; `records` and `random` are level (random at 12
  threads, 1.017, is faster). A new fuzz mode forces the plain teams (`examples/fuzz.rs`,
  `teams`, in CI's fuzz job), and the Guardian's review cleared its coverage (D097, D098).
- **Admission by the bound and the share of run symbols (#353).** A short block of
  high-entropy data keeps the column instead of taking the run records, which cost it 10–11%
  more instructions.
- **The move-to-front list in one page (#358).** `mtf_checkpoints_avx2`'s list is aligned to
  512 bytes, so it no longer straddles a page in 4 of its 64 possible layouts.
- **Harness only (#356).** `ab summary` reads a record's top-level fields only (bench 3.8.1).

**The paired run against 0.2.8** (CI, Intel Xeon Platinum 8370C, 4 CPUs on 2 cores, run
[42858124938](https://github.com/lmmx/mibz2/actions/runs/42858124938), 10 paired rounds):
no change detected at 1, 2 and 4 threads on any corpus. Head over base, wall:

| corpus | 1 | 2 | 4 threads |
|---|--:|--:|--:|
| `wikidata` | −0.1% (−4.4, +4.3) | +2.3% (−0.5, +5.3) | +0.6% (−1.7, +2.9) |
| `text` | −1.0% (−2.9, +1.0) | +0.7% (−13.4, +17.1) | −1.1% (−7.2, +5.3) |
| `records` | −0.2% (−2.7, +2.4) | −1.9% (−6.6, +3.0) | −0.7% (−6.3, +5.3) |
| `random` | −1.3% (−7.8, +5.7) | −2.1% (−7.1, +3.2) | −2.4% (−6.7, +2.2) |

That is expected: #359's gain is on the team path, which these runners don't plan.

**The cost.** The MTF and RLE2 stage, timed alone, is +3.4% (+2.0%, +4.8%) slower; the wall is
unchanged. Peak memory on `text` at 4 threads is 53 → 56 MB (+3.6%); every other corpus and
thread count is within 1%.

**Release scaling on the i9-10900K** (0.2.9 against 0.2.8, `release-scaling/run-20261110T170952Z`,
SHA-256 33469444…; 0.2.8 ÷ 0.2.9 wall on `wikidata`, paired, with 95% intervals):

| threads | 1 | 2 | 4 | 8 | 10 | 12 | 16 | 20 |
|---|--:|--:|--:|--:|--:|--:|--:|--:|
| `wikidata` | 0.991 (0.983–0.998) | 0.998 | 1.001 | **1.147** | **1.145** | **1.111** | **1.109** | **1.102** |

From 8 threads, where the decoder plans teams, 0.2.9 is 10–15% faster. At 1 thread it is
0.9% slower beyond noise, the size of #359's first CI flag. The 1-thread code path changed by
one inlining decision (the Pathfinder's codegen comparison on #145), which a follow-up patch
restores.

**Caveats.** The gain is measured on one machine, the i9-10900K, in #359's registered run and
in this release-scaling run; CI can't measure the team path. #351 (Rauzy–Veech induction) merged as a research example and journal only; it
is not in the library.

## [0.2.8](https://github.com/lmmx/mibz2/compare/v0.2.7...v0.2.8) - 2026-11-10

A speed release. On the Wikidata sample a decode is 12–19% faster on CI's runner, from a new
path for blocks of few symbols (#324) and a faster table build (#312). On the i9-10900K at 10
threads, a guarded service core becomes the default (#316). The output, the API and the
supported inputs are unchanged.

- **Blocks of few symbols (#324).** On a block with at most one symbol per two rows of its
  bound, the default `Segmented` kernel, which decodes a block on one thread, no longer
  writes the last column. It turns the
  move-to-front symbols into run records, writes the backward (LF) table from them in arrival
  order, with no sort and no scatter, and walks it backward with the room walk's own code.
  That covers every Wikidata and `text` block; `records` and `random` keep the column. Every
  other kernel (the cooperative teams, `Plain`, the pipelined teams) is unchanged. Louis
  adopted it as the default although its registered no-harm rule failed on `random` and
  `records` at one thread (see the cost and the caveats).
- **The table build (#312).** The T-vector build cuts its 16-row windows into runs, with a
  cheap classifier for high-entropy windows. It reaches the plain-worker path, which CI
  measures, and a team's `solo` fallback; the shared-team path (`FusedBuild::share`) doesn't
  carry it.
- **The i9-10900K (#316).** At 10 threads on exactly that machine (its CPU model and
  topology, AVX2 and CLMUL, Linux x86-64, an unlimited quota), the default is now the guarded
  service core. Its qualification run on the i9 (`service-guarded/20261109T195345Z`, 420
  runs, none in the slow mode) gave throughput +6.8% and CPU −14.7%. It is inactive on every
  other machine, which keeps the plan it had, as tests check.
- **Harness only (#350).** The bench harness can hold a spinner on a spare CPU for a
  workstation run (`ab run --hold-cpu`, bench 3.8.0). It is off by default, and the library
  is untouched by it. #311's wait counts are behind the `wait-stats` feature, off by default.

### Measured

One paired run of this release's source (master at `47149bff`, whose library is 0.2.8's)
against `v0.2.7` (`539523b1`), both builds on one 4-CPU CI runner (AMD EPYC 9V74, 2 physical
cores), 10 rounds, the builds' order alternating, the bench's Wikidata sample (12 slices of
the dump, 210 MB decoded) and the crate's corpora
([run 42811647197](https://github.com/lmmx/mibz2/actions/runs/42811647197), bench 3.8.0):

| threads | wall, 0.2.7 → 0.2.8 (median) | wall change (95%) | CPU change (95%) | peak RSS |
|--:|--:|--:|--:|--:|
| 1 | 0.534 → 0.433 s | **−19.0% (−19.3, −18.6)** | **−18.9% (−19.2, −18.6)** | 26 → 26 MB (+1.9%) |
| 2 | 0.290 → 0.242 s | **−16.6% (−17.6, −15.6)** | **−17.3% (−17.5, −17.2)** | 49 → 52 MB (+6.5%) |
| 4 | 0.247 → 0.217 s | **−12.1% (−13.7, −10.5)** | **−14.1% (−15.1, −13.2)** | 77 → 81 MB (+4.5%) |

- **Result:** faster in wall and CPU at every thread count, beyond noise.
  - Best wall time against best: −12.1% (−13.7, −10.5), both at 4 threads (848 → 967
    MB/s).
  - CPU per byte at 4 threads: 4.45 → 3.81 ns/B. Amplification over one thread rose: +1.9%
    (+1.4, +2.4) at 2 threads and +5.8% (+4.6, +7.1) at 4: the CPU at one thread fell
    more (−18.9%) than at four (−14.1%).
  - The output was the same bytes as the base's on every corpus.
- **The other corpora:**
  - `text` is faster at every thread count: −25.9% (−26.9, −24.8) at 1 thread, −17.4% at
    2, −10.2% at 4.
  - `pattern` is faster at every thread count: −8.6% at 1 thread, −11.9% at 2, −5.4% at 4.
  - `runs` is −2.3% (−4.2, −0.3) at 2 threads, and unchanged at 1 and 4.
  - `random` is +1.2% (+0.7, +1.7) slower at 1 thread (below), and unchanged at 2 and 4.
  - `records` and `onebyte` are unchanged.
- **By stage,** at one thread: a total of 9.21 → 8.94 cycles a byte, −3.0% (−3.3, −2.7).
  The stage table times the library's `mtf`, `bwt::build` and forward walk directly, so it
  can't see #324's path; its rows show #312.
  - The T-vector build: −16.0% (−16.3, −15.8), #312's windows.
  - RLE1: −5.7% (−6.4, −4.9).
  - The walk −0.2% and CRC +0.0%, neither a change.
  - The scanner +1.0% (+0.8, +1.1), MTF + RLE2 +1.7% (+1.0, +2.4) and the walk's segment
    assembly +3.8% (+0.6, +7.1) are flagged slower. With seven intervals a run, a small flag
    on one stage can be chance or code layout.

### The cost

- **`random` at one thread:** +1.2% (+0.7, +1.7) wall and CPU, beyond noise. Its blocks run
  none of #324's new code. #324's registration measured +5.3% on `random` and +2.1% on
  `records` at one thread; Louis accepted that trade-off for the gain on Wikidata and text.
- **Memory:** peak RSS is higher on the corpora that take #324's path: `text` +5.0% to
  +7.6% (35 → 38 MB at 2 threads), `wikidata` +1.9% to +6.5% (49 → 52 MB at 2 threads). The
  other corpora move by at most +1.4% (`runs`); `random`, `pattern`, and `records` at 2 and
  4 threads, not beyond noise.
- **CPU amplification** at 4 threads rose by +5.8% (above). It is a 2-core runner's figure;
  how it grows with more cores needs a larger machine.

### Caveats

- **Short high-entropy blocks.** On blocks of 200–440 kB that are mostly high-entropy, #324's
  admission test takes the new path, and their decode costs about 10–11% more instructions. No
  CI corpus holds such blocks, so the run above can't show it. 0.2.8 carries this cost. It
  is fixed on master by #353 (admission by the bound and the share of run symbols, so such a
  block keeps the column), which ships in the next release.
- **One run on one CPU model,** a 2-core runner. It can't show the ten-core plans, which
  CI's runners never take, nor #316's default, which is inactive there.
- **The release-scaling figures follow.** 0.2.8 against 0.2.7 on the i9-10900K is pending.
  Its figures will be added here once they are in.
- **0.2.6's known issue isn't re-measured.** On the ten-core topology, 0.2.6's default was
  slower than 0.2.4's on files compressed at bzip2 levels 1 and 2 (see 0.2.6's notes).
  #316 changes the i9's default at 10 threads; whether those files still decode slower is
  not yet measured.

### <!-- 3 -->Performance

- run-record front and LF table on blocks of few symbols, walked backward ([#324](https://github.com/lmmx/mibz2/pull/324))
- guarded service core as the default on the i9-10900K, target-gated ([#316](https://github.com/lmmx/mibz2/pull/316))
- *(bwt)* the build's windows cut into runs, with a cheap classifier for high-entropy windows ([#312](https://github.com/lmmx/mibz2/pull/312))
- count the cooperative waits (wait-stats feature, off by default); the spin hypothesis for the i9 slow mode not confirmed ([#311](https://github.com/lmmx/mibz2/pull/311))

### <!-- 4 -->Documentation

- *(team)* ownership: the team owns mibz2 and ships ([#345](https://github.com/lmmx/mibz2/pull/345))
- *(changelog)* 0.2.7's notes from its paired run against 0.2.6 ([#307](https://github.com/lmmx/mibz2/pull/307))

### <!-- 9 -->Other

- optional held spinner for workstation runs (W1) ([#350](https://github.com/lmmx/mibz2/pull/350))
- MONODROMY joins as mathematical inventor (Louis, 10 Nov 2026): exploratory lane exempt from the 48h expiry, peer of the Pathfinder, inbox #346, routing and cursor ([#347](https://github.com/lmmx/mibz2/pull/347))
- *(coop)* the walk-stats instrument, feature-gated (off by default) ([#338](https://github.com/lmmx/mibz2/pull/338))
- *(ab)* the child's record, schema ab-child/2 (bench 3.6.0) ([#336](https://github.com/lmmx/mibz2/pull/336))
- share-nothing pricing, a diagnostic C5 recipe (registration, not frozen) ([#326](https://github.com/lmmx/mibz2/pull/326))
- bench-par-scan, the parallel scan's knee sweep, its run's result ([#317](https://github.com/lmmx/mibz2/pull/317))
- RULES C6: delegate bench design, merge acceptance and adoption policy as a class (Louis, 2026-11-09) ([#306](https://github.com/lmmx/mibz2/pull/306))

## [0.2.7](https://github.com/lmmx/mibz2/compare/v0.2.6...v0.2.7) - 2026-11-09

A safety release. A parallel `BzReader` whose source panics now fails its next read, instead
of waiting for ever (#278). The decoder's plans, the output, the API and the supported
inputs are unchanged.

- **The fix (#278).** A parallel reader reads its source on a thread of its own. If the
  source's `read` panicked, that thread used to unwind without cancelling the decode, and the
  reader's next read waited for input that would never come. Every parallel reader was
  affected: the workers, the teams and the pipelined teams. Now a guard on that thread
  cancels the decode as it unwinds, and the next read returns an `io::Error` saying the
  input's reader panicked. `BzReader`'s docs state the contract and what stays outside it:
  an abort (an out-of-memory one among them) still ends the process, and a `read` that
  blocks for ever is still never interrupted.
- **Tests only (#275, Guardian D047; and #273, D042).** A deterministic test unwinds a
  pipelined team's worker at three points and checks what the reader gets, with a mutant
  that the test kills, a Miri filter, and the provenance of the pipeline's fuzz entry. #273
  adds pipeline fuzz candidates and invariant-mutation evidence for 0.2.6's default. Both
  are `cfg(test)`: the shipped decoder is unchanged by them.

### Measured

One paired run of this release's source (master at `5c98d423`, whose library is 0.2.7's)
against `v0.2.6` (`a49038b0`), both builds on one 4-CPU CI runner (AMD EPYC 9V45, 2 physical
cores), 10 rounds, the builds' order alternating, the bench's Wikidata sample (12 slices of
the dump, 210 MB decoded) and the crate's corpora
([run 42643237164](https://github.com/lmmx/mibz2/actions/runs/42643237164), bench 3.4.0):

| threads | wall, 0.2.6 → 0.2.7 (median) | wall change (95%) | CPU change (95%) | peak RSS |
|--:|--:|--:|--:|--:|
| 1 | 0.348 → 0.349 s | +0.6% (−0.7, +2.0) | +0.6% (−0.7, +2.0) | 26 → 26 MB (+0.6%) |
| 2 | 0.194 → 0.201 s | +1.6% (−1.0, +4.3) | +1.7% (−0.7, +4.2) | 49 → 49 MB (+0.2%) |
| 4 | 0.178 → 0.182 s | +2.9% (−0.2, +6.0) | +2.7% (−1.1, +6.6) | 78 → 78 MB (+0.2%) |

- **Result:** no change detected on the four measures at this run's precision. Every
  interval holds no change, which is not a proof that nothing changed.
  - Best wall time against best: +2.9% (−0.2, +6.0), both at 4 threads.
  - CPU per byte at 4 threads: 3.10 → 3.19 ns/B. Amplification over one thread: +1.0%
    (−2.0, +4.2) at 2 threads and +2.1% (−1.6, +5.9) at 4, neither a change.
  - The output was the same bytes as the base's on every corpus.
- **The other corpora:** no change at 2 or 4 threads. One flag of 21 rows: `text` at one
  thread, +2.1% (+0.4, +3.8) slower, about what chance gives across that many intervals.
- **By stage,** at one thread: a total of 7.14 → 7.04 cycles a byte, −1.6% (−3.4, +0.4).
  The T-vector build reads −5.2% (−6.5, −3.8), though its code is the same in both builds:
  with seven intervals a run, one stage's flag alone can be chance or code layout. The
  scanner, MTF, the walk, RLE1 and CRC are each within ±1.2%, none a change.

### The cost

- **The guard:** one check, as the reader's thread ends, and nothing a chunk. The run above
  shows no wall or CPU cost at any thread count.
- **Memory:** peak RSS is 0.1–0.3 MB higher (+0.5% to +0.9%) at one thread on every
  corpus, and at every thread count on `onebyte` and `runs`, each flagged. On the Wikidata
  sample at 2 and 4 threads it isn't a change (+0.2%). Its cause isn't identified. Being
  about the same in megabytes on every corpus, it looks like a fixed cost of the process,
  not a cost a byte. One larger flag, `random` at 4 threads, +2.0 MB, +3.1% (+0.1, +6.3),
  sits beside `pattern`'s −2.3 MB at 4 threads (not a change): the multi-threaded rows'
  memory is noisier than this.

### Caveats

- **One run on one CPU model,** a 2-core runner. It is a no-regression check: it can't show
  the ten-core plans, which CI's runners never take.
- **The release-scaling figures follow.** `bench-release-scaling`, 0.2.7 against 0.2.6 on
  the i9-10900K, is queued on the workstation at `539523b1`. Its figures will be added
  here once they are in.
- **0.2.6's known issue stands.** On the ten-core topology, the default is slower than
  0.2.4's on files compressed at bzip2 levels 1 and 2 (see 0.2.6's notes). 0.2.7 doesn't
  change the plans; `Plan::Teams { teams: 4, k: 2 }` still avoids it.

### <!-- 9 -->Other

- Bench 3.4.0: the recorder check's failed reads and controls are INCOMPLETE (Guardian D066)
- Bench 3.4.0: the recorder check revised for Guardian D065
- Bench 3.4.0: the machine-state recorder's validity fixes, raw timestamped readings, and its untimed check
- publish first, review in flight (Louis's instruction) ([#294](https://github.com/lmmx/mibz2/pull/294))
- point every reference at the pit-crew post's new URL and card
- "mibz2 gets a pit crew"; check_prose.py; title and dek rules ([#295](https://github.com/lmmx/mibz2/pull/295))
- Service threads, moving mibz2's serial work out of its teams ([#292](https://github.com/lmmx/mibz2/pull/292))
- per-run cycles, clock and thermal state, recorded and never judged ([#290](https://github.com/lmmx/mibz2/pull/290))
- coordinate-blog-post; takeaways first and visuals over dense prose in write-blog-post ([#291](https://github.com/lmmx/mibz2/pull/291))
- ask the Guardian, not Louis, when genuinely unsure ([#287](https://github.com/lmmx/mibz2/pull/287))
- Workstation queue: #271 and #270; Louis's standing authorisation for run-signal ([#285](https://github.com/lmmx/mibz2/pull/285))
- certified-packet census on the inverse BWT; packets cover almost nothing on Wikidata ([#280](https://github.com/lmmx/mibz2/pull/280))
- a parallel BzReader's source panicking cancels the decode instead of stranding the reader ([#278](https://github.com/lmmx/mibz2/pull/278))
- deterministic worker-unwind test for M12, Miri filter, provenance (Guardian D047) ([#275](https://github.com/lmmx/mibz2/pull/275))
- pipeline fuzz candidates and invariant-mutation evidence for 0.2.6's default (Guardian D042) ([#273](https://github.com/lmmx/mibz2/pull/273))
- the MTF-runs post is its own post, not a sub-post of the inverse-BWT navigation post
- a run-fed T-vector and a run-compressed Ψ², checked and censused; neither proposed for timing ([#266](https://github.com/lmmx/mibz2/pull/266))
- reading the BWT's runs off bzip2's move-to-front codes ([#262](https://github.com/lmmx/mibz2/pull/262))
- inverse-BWT navigation restructured, with a technical appendix ([#259](https://github.com/lmmx/mibz2/pull/259))
- the mathematical structure of inverse-BWT navigation ([#257](https://github.com/lmmx/mibz2/pull/257))
- README, CHANGELOG: 0.2.6's measured slowdown at bzip2 levels 1-2 ([#256](https://github.com/lmmx/mibz2/pull/256))
- relay an attached file's complete text in a collapsed details block within the one #145 comment
- CI incidents runbook: classify a red check from its log, re-run only the failed job paced to the outage, record what did not run; the green-head gate holds, an infrastructure exception is proposed for Louis
- a general plan policy (rule A, the evidence envelope), reviewed; README: unmeasured topologies ([#253](https://github.com/lmmx/mibz2/pull/253))
- pipelined teams, how mibz2 0.2.6 decodes 7% faster on ten cores ([#252](https://github.com/lmmx/mibz2/pull/252))
- The workstation queue: prepared runs listed in bench/workstation-queue.txt, run with just workstation-queue (PROTOCOL 9); await-submit checks out the tagged commit

## [0.2.6](https://github.com/lmmx/mibz2/compare/v0.2.5...v0.2.6) - 2026-11-07

### <!-- 4 -->Documentation

- *(changelog)* 0.2.5's notes from its i9 runs and its paired run against 0.2.4 ([#245](https://github.com/lmmx/mibz2/pull/245))

### <!-- 9 -->Other

- The default pipelines three teams of three on the ten-core topology, with park and small ([#250](https://github.com/lmmx/mibz2/pull/250))
- just submit-when-ready: the unattended wait-run-suspend as a recipe; CLAUDE.md documents it and the run-signal workflow
- stand-in reviews when a role's Work task is unavailable (ROLES, PROTOCOL 8, CHATGPT.md)
- why mibz2 stops scaling at eight threads, and why three teams help ([#248](https://github.com/lmmx/mibz2/pull/248))
- Release runbook: docs/RELEASING.md, referenced from CLAUDE.md

On a machine with 10 physical cores and a 20 MiB L3, such as the i9-10900K, the decoder now
pipelines its three teams of three at 10 or more threads: each team's leader undoes one
block's RLE1 and checks its CRC, and runs the next block's move-to-front, while its helpers
build and walk the block between (#247). Two levers are on: waiting helpers park rather than
yield (#249's park), and a leader keeps staging after a small block (#249's small). Only that
plan changes: every other plan, and every other machine shape, keeps the 0.2.5 path. So do
the output, the errors, the API and the supported inputs.

### Measured

**On the target machine:** three registered runs on an Intel i9-10900K (10 cores, 20 CPUs), 20
rounds each, on the bench's Wikidata sample. Louis ran all three with `just submit`. Their bundles are
on the `evidence` branch, as `team-pipeline/20261107T042622Z`, `pipeline-levers/20261107T091926Z`
and `pipelined-default/20261107T105448Z`.

- **The confirmation (`pipelined-default/20261107T105448Z`, VALID PASS):** this release's
  default against 0.2.5's, paired by round, admitted on the terms in `docs/journal/decisions.md`
  (2026-11-07):

  | threads | plan, 0.2.5 → 0.2.6 | speed (95%) | CPU | peak RSS |
  |--:|---|--:|--:|--:|
  | 8 | 4 × 2 → 4 × 2 (unchanged) | 1.002 (0.991–1.013) | 0.99 | 1.00 |
  | 9 | 4 × 2 → 4 × 2 (unchanged) | 1.001 (0.991–1.012) | 1.00 | 1.00 |
  | 10 | 3 × 3 → 3 × 3 pipelined | **1.072 (1.054–1.089)** | 1.04 | 1.10 |
  | 11 | 3 × 3 → 3 × 3 pipelined | 1.070 (1.055–1.085) | 1.05 | 1.12 |
  | 12 | 3 × 3 → 3 × 3 pipelined | 1.069 (1.054–1.085) | 1.05 | 1.10 |
  | 16 | 3 × 3 → 3 × 3 pipelined | 1.065 (1.045–1.085) | 1.05 | 1.13 |
  | 20 | 3 × 3 → 3 × 3 pipelined | 1.070 (1.048–1.092) | 1.04 | 1.12 |

  CPU and peak RSS are the new default's medians over 0.2.5's. Both are reported, not judged.
  The figures below are from the earlier, exploratory runs.

- **Speed:** 7.2% faster, 95% CI +5.3% to +9.1%. This is a paired ratio of 1.072
  [1.053, 1.091]: the pipelined teams with both levers against the plain three teams of three
  (0.2.5's plan), both on 10 threads.
- **CPU:** 5.2% more, from the paired ratio 0.951 (1/0.951) alongside that speedup.
  - **The medians tell a slightly different story.** Against the shipped default, the medians
    are 1.030 s against 0.973 s of CPU, which is 5.9% more.
  - **Both figures stand.** They use different summaries (a paired ratio, and a ratio of
    medians), and neither corrects the other.
- **Wall, by the medians, against the shipped default (mibz2@10):** 0.142 s against 0.154 s.
- **Memory:** peak RSS 75 MB against 67 MB (+8 MB) on the measured workload. The extra is a
  second set of move-to-front buffers and a second assembled buffer, in each team.
- **The levers, separately (each against the plain pipeline, at 3 × 3 on 10 threads):**
  - park cuts CPU by about 5% (a ratio of 1.055, about 5.2% less) at an unchanged wall: 0.999 [0.981, 1.017] speed, 1.055
    [1.039, 1.071] CPU;
  - small is 4.3% faster: 1.043 [1.020, 1.066];
  - both: 1.038 [1.021, 1.055] faster, 1.032 [1.018, 1.046] less CPU.
- **The pipeline alone (`team-pipeline/20261107T042622Z`):** 1.042 [1.016, 1.068] faster
  than the plain teams at 3 × 3, at a CPU ratio of 0.918 [0.895, 0.942]. That ratio is the
  plain teams' CPU over the pipeline's, so the pipeline spends about 8.9% more CPU (6.2–11.7%).
  - The levers' run reproduced it at 1.033 [1.017, 1.049].

**On CI:** CI's 4-CPU runners have 2 physical cores, so neither build takes the 3 × 3 plan
there. CI's paired bench is therefore only the no-regression check of every other path.

**Caveats:**
- **One machine, one corpus.** The evidence is the i9-10900K decoding Wikidata's bzip2 blocks.
  Any CPU with 10 physical cores and a 20 MiB L3 gets the change, measured or not.
- **The 3 × 3 plan only.** The pipelined teams are the default at that plan alone (`PIPELINED_PLAN`). Their gain at
  other plans, 3 × 4 at 12 threads among them, isn't shipped.
- **The gain is not split into causes.** The runs can't separate the overlap's gain from the
  leader's extra walking: a leader that has finished its serial work joins its helpers' walk.
- **A frozen confirmation run** against 0.2.5 is registered before release
  (`docs/journal/2026-11-07-pipelined-default-registration.md`). Its result is recorded there.

**Known issue, measured after release (8 November 2026, `plan-levels/20261108T094232Z`):** on
the ten-core topology, at 10 and 20 threads, 0.2.6's default is slower than 0.2.4's on files
compressed at bzip2 levels 1 and 2:
- 0.763 (0.752–0.775) at level 1, and 0.776 (0.759–0.794) at level 2, both at 10 threads.
  These are speed ratios (0.2.4's wall over 0.2.6's): about 23% lower throughput, so such a
  file takes about 31% (level 1) and 29% (level 2) longer to decode;
- forced to 4 × 2, 0.2.6 matches 0.2.4 (1.007–1.009), so all of the loss is the plan;
- at level 3, the default is 1.047 (1.015–1.081) against 0.2.4.

Passing `Plan::Teams { teams: 4, k: 2 }` avoids it. A fix is being registered.

**The lower-CPU path stays available.** The plain three teams of three are
`StreamDecoder::planned(Plan::Teams { teams: 3, k: 3 }, slots, kernels)`, or
`BzReader::parallel_planned` with the same plan. The default's window is 12 blocks.

## [0.2.5](https://github.com/lmmx/mibz2/compare/v0.2.4...v0.2.5) - 2026-11-06

On a machine with 10 physical cores and a 20 MiB L3, such as the i9-10900K, the decoder now
splits 10 or more threads into three teams of three, where it used to split them into four
teams of two (#234). Every other machine shape keeps the 0.2.4 plan unchanged. So do the
output, the API and the supported inputs.

### Measured

**On the target machine:** two registered runs on an Intel i9-10900K (10 cores, 20 CPUs).
- **Method:** 0.2.5's default against the 0.2.4 base, 20 rounds, on the bench's Wikidata sample.
- **Admission rule:** fixed before the first run (`docs/journal/2026-11-06-three-team-policy.md`). Both runs had to pass, and both did.
- **Reading the table:** speed is the base's wall time over the head's, so above 1 is faster. CPU and RSS are the head's median over the base's.

| threads | run 1 speed (95%) | confirmation speed (95%) | CPU | peak RSS |
|--:|--:|--:|--:|--:|
| 8 (control, same plan) | 0.986 (0.977–0.995) | 1.005 (0.992–1.019) | 1.00 | 1.00 |
| 9 (control, same plan) | 1.000 (0.991–1.009) | 1.007 (0.997–1.017) | 0.99 | 0.99 |
| **10** | **1.089 (1.075–1.103)** | **1.088 (1.076–1.101)** | 0.89 | 0.81 |
| 11 | 1.081 | 1.072 | 0.89 | 0.81 |
| 12 | 1.065 | 1.085 | 0.88 | 0.82 |
| 16 | 1.076 | 1.082 | 0.90 | 0.82 |
| 20 | 1.092 | 1.081 | 0.89 | 0.82 |

The CPU and RSS columns are the confirmation's. Run 1's are within 0.01 of them.

**On CI:** one paired run of this release against `v0.2.4`, both builds on one 4-CPU GitHub runner, 10 rounds,
on the Wikidata sample and the crate's corpora ([run 42310037840](https://github.com/lmmx/mibz2/actions/runs/42310037840)).
- **Result:** no change on any of the four measures.
  - Best wall time: −0.1% (−1.1, +0.9).
  - CPU per byte and amplification at 2 and 4 threads: no change.
  - The output was the same bytes as the base's.
- **Stages:** within ±0.03 ns/B, at a total of about 2.4 ns/B.
- **Why no gain shows:** this runner has 2 physical cores. The policy doesn't apply to it, so this run only checks that nothing else moved.

**Caveats:**
- **One machine shape, one workload.** The evidence is a 10-core Intel part with 20 MiB of L3, decoding Wikidata's bzip2 blocks. The policy applies only to that shape.
- **The cause of run 1's control offset is unknown.** In run 1, the 8-thread control, which runs the same plan in both builds, was 1.4% slower for the head. That is inside its registered ±2% band, and it didn't recur in the confirmation.
- **The gain is not split into causes.** The lower CPU and RSS are consistent with fewer, larger teams sharing L3 better, but that is not measured.
- **Evidence:** both runs' bundles are on the `evidence` branch (`820f4a87`, `0b340a53`), each with its SHA-256.

### <!-- 3 -->Performance

- three teams of three at 10 threads on a 10-core, 20 MiB-L3 machine ([#234](https://github.com/lmmx/mibz2/pull/234))

### <!-- 4 -->Documentation

- *(team)* WORK-TASKS.md, the only ChatGPT Work task prompts ([#226](https://github.com/lmmx/mibz2/pull/226))
- *(team)* the agent team, independent of compact30 ([#225](https://github.com/lmmx/mibz2/pull/225))
- *(team)* add the Oracle, the team's record of what it is doing and why ([#223](https://github.com/lmmx/mibz2/pull/223))
- keep the agent team's protocol on master (MODE, PROTOCOL, roles, specs, log) ([#221](https://github.com/lmmx/mibz2/pull/221))
- *(changelog)* 0.2.4's notes from its paired run against 0.2.3 ([#216](https://github.com/lmmx/mibz2/pull/216))

### <!-- 9 -->Other

- Workstation evidence by command: just submit RECIPE (RULES C5) ([#242](https://github.com/lmmx/mibz2/pull/242))
- The relay's return direction: no repeated re-pin, and requests the Coordinator closes ([#241](https://github.com/lmmx/mibz2/pull/241))
- The relay's return direction: the Coordinator's requests to ChatGPT ([#240](https://github.com/lmmx/mibz2/pull/240))
- The Oracle is renamed the Observer ([#239](https://github.com/lmmx/mibz2/pull/239))
- how a new ChatGPT conversation sends Louis's requests to the team ([#238](https://github.com/lmmx/mibz2/pull/238))
- a footer link to "Behind the scenes" on every page ([#237](https://github.com/lmmx/mibz2/pull/237))
- Louis's requests relayed from ChatGPT on #145 ([#235](https://github.com/lmmx/mibz2/pull/235))
- The Oracle explains: a plain-prose account first, its hierarchy behind it ([#231](https://github.com/lmmx/mibz2/pull/231))
- Behind the scenes, the Oracle's account read live from observatory/intention ([#229](https://github.com/lmmx/mibz2/pull/229))
- team size against live tables on the i9, registered, run and recorded; workstation recipes end with one tarball ([#219](https://github.com/lmmx/mibz2/pull/219))
- release scaling past 4 threads, recorded locally and shown on the History chart ([#217](https://github.com/lmmx/mibz2/pull/217))

## [0.2.4](https://github.com/lmmx/mibz2/compare/v0.2.3...v0.2.4) - 2026-11-05

The move-to-front decode keeps the first 32 bytes of its list in two registers, so a move
whose index is below 32 no longer writes the list to memory and reads it back. About 98%
of the Wikidata sample's moves take that path. The output, the API and the supported inputs
are unchanged.

### Measured

One paired run of this release against `v0.2.3`, both builds on one CI runner (AMD EPYC
9V45, 4 vCPUs), 10 rounds, the builds' order alternating, the bench's Wikidata sample (12
slices of the dump, 210 MB decoded):

| threads | `wikidata` wall | `records` wall |
|--:|--:|--:|
| 1 | **−4.8% (−8.8, −0.7)** | **−23.5%** |
| 2 | −0.4% (−6.6, +6.3) | **−17.5%** |
| 4 | −2.1% (−11.1, +7.7) | **−17.7%** |

- **By stage,** at one thread: MTF + RLE2 −17.1%. The table build was flagged +6.7%
  slower, though its code is unchanged: with seven stages a run, one flagged by chance is
  expected about one run in three.
- **#207's own runs:**
  - three on an AMD EPYC 7763 (Zen 3), with `wikidata` at 1 thread about −5% and the MTF
    stage about −23%;
  - a cross-model check on an EPYC 9V45 (Zen 5), with −2.1% (−3.8, −0.4) and −18.5%.
- **No run drew an Intel CPU.** At 2 and 4 threads this run shows no change beyond noise.
- **The output** was the same bytes on every corpus.

### <!-- 3 -->Performance

- *(mtf)* libbzip2's two-level list, its first 32 bytes in two registers ([#207](https://github.com/lmmx/mibz2/pull/207))

### <!-- 4 -->Documentation

- *(claude)* the coordinator drives releases itself ([#214](https://github.com/lmmx/mibz2/pull/214))
- *(blog)* the bit-buffer figure narrates itself; section 2 builds the table on the toy code first ([#212](https://github.com/lmmx/mibz2/pull/212))
- *(blog)* the Huffman post opens with what a bit buffer is ([#211](https://github.com/lmmx/mibz2/pull/211))

## [0.2.3](https://github.com/lmmx/mibz2/compare/v0.2.2...v0.2.3) - 2026-11-05

### <!-- 4 -->Documentation

- *(changelog)* 0.2.2's notes from its paired run against 0.2.1 ([#210](https://github.com/lmmx/mibz2/pull/210))
- *(blog)* the Huffman post builds the reader's model first; the writing skill learns why ([#208](https://github.com/lmmx/mibz2/pull/208))

## [0.2.2](https://github.com/lmmx/mibz2/compare/v0.2.1...v0.2.2) - 2026-11-05

The inverse BWT's walk, the move-to-front decode and the Huffman scanner are each faster,
and a decode keeps more blocks in flight per thread. The output, the API and the
supported inputs are unchanged.

### Measured

One paired run of this release against `v0.2.1`, both builds on one CI runner (AMD EPYC
7763, 4 vCPUs), 10 rounds, the builds' order alternating, the bench's Wikidata sample
(12 slices of the dump, 210 MB decoded):

| threads | wall | CPU |
|--:|--:|--:|
| 1 | **−15.4% (−17.5, −13.3)** | **−15.3%** |
| 2 | **−25.2% (−26.5, −23.8)** | **−17.8%** |
| 4 | **−25.8% (−27.3, −24.2)** | **−15.5%** |

- **By stage,** at one thread: the walk −24.9%, MTF −12.5%, the Huffman scanner −7.6%.
  RLE1 was +2.6% slower and the table build unchanged.
- **The other corpora** were faster or unchanged at every thread count. The output was
  the same bytes on all of them.
- **The cost:** peak memory at 4 threads +4.3%, from the window of 4 blocks a thread
  (#180).
- **One run on one CPU model,** so the intervals understate the run-to-run spread, which
  merges that left the decoder alone put at about 0.7% at one thread.

### Packages

0.2.2 is on crates.io only. Its PyPI wheels were not built: the macOS wheel jobs ran on
`macos-14` runners, which GitHub no longer serves, and the PyPI step needs every wheel.
0.2.3 publishes the wheels, built from the same source with the macOS jobs on
`macos-15` (#209).

### <!-- 2 -->Bug Fixes

- *(sweep)* walk blocks under MIN_SEGMENTED plainly at every setting, as the decoder does ([#200](https://github.com/lmmx/mibz2/pull/200))
- *(occupancy)* complete traces under a team plan; --teams K, checked in CI ([#116](https://github.com/lmmx/mibz2/pull/116))

### <!-- 3 -->Performance

- *(block)* the symbol loop keeps the bit reader's state in registers ([#204](https://github.com/lmmx/mibz2/pull/204))
- *(bwt)* the staged walk tests each lane's start flag, without the deferred flags ([#181](https://github.com/lmmx/mibz2/pull/181))
- *(parallel)* the default window is 4 blocks a thread, not 2 ([#180](https://github.com/lmmx/mibz2/pull/180))
- *(mtf)* the move-to-front list holds the bytes, not their indices ([#172](https://github.com/lmmx/mibz2/pull/172))
- *(bwt)* the room walk tests its flags once an offset and stages its bytes in one tile ([#171](https://github.com/lmmx/mibz2/pull/171))
- *(mtf)* a run of at most 64 bytes is two vector stores, not a memset call ([#170](https://github.com/lmmx/mibz2/pull/170))
- *(bwt)* Segmented inverts in place over ll; rle is never allocated ([#164](https://github.com/lmmx/mibz2/pull/164))
- *(bwt)* the room walk's pass steps eight lanes a turn, handovers after ([#157](https://github.com/lmmx/mibz2/pull/157))

### <!-- 4 -->Documentation

- *(blog)* Huffman decoding in mibz2: keeping the bit buffer in a register ([#206](https://github.com/lmmx/mibz2/pull/206))
- *(coop)* the pool's preamble said nothing uses it; the cooperative kernel's teams do ([#199](https://github.com/lmmx/mibz2/pull/199))
- *(claude)* audit runs every time the coordinator wakes, not only on a recurring timer ([#192](https://github.com/lmmx/mibz2/pull/192))
- *(benchmarks)* the history as one chained trajectory, and absolute figures one runner at a time ([#176](https://github.com/lmmx/mibz2/pull/176))
- look at rendered pages before publishing; fix the renders it found ([#167](https://github.com/lmmx/mibz2/pull/167))
- *(blog)* name the table-builder post's sidebar group after its title
- *(blog)* plainer title and a shorter dek for the table-builder post, and say what compact30 means
- *(blog)* how compact30's table builder went from 49 instructions to 12 ([#144](https://github.com/lmmx/mibz2/pull/144))
- *(blog)* restore the space–time post's original dek; fix the card check on previews ([#143](https://github.com/lmmx/mibz2/pull/143))
- check that each post's description, index card and social card match its text ([#142](https://github.com/lmmx/mibz2/pull/142))
- *(blog)* revisiting the space–time tradeoff in BWT inversion ([#140](https://github.com/lmmx/mibz2/pull/140))
- *(blog)* a smaller inverse-BWT table wins with four workers ([#139](https://github.com/lmmx/mibz2/pull/139))
- *(blog)* the MLP post states its conclusion up front ([#138](https://github.com/lmmx/mibz2/pull/138))
- *(blog)* MLP post, links at first mention and an intuition for the chain and IPC ([#136](https://github.com/lmmx/mibz2/pull/136))
- *(blog)* memory-level parallelism in mibz2, with measurement notes ([#135](https://github.com/lmmx/mibz2/pull/135))
- *(blog)* restore the P87 post's title ([#127](https://github.com/lmmx/mibz2/pull/127))
- *(blog)* rewrite the P87 post around the problem: the extra CPU cost of four-worker decoding ([#126](https://github.com/lmmx/mibz2/pull/126))
- *(site)* render maths with KaTeX; the blog posts drop process detail ([#124](https://github.com/lmmx/mibz2/pull/124))
- *(blog)* Investigating inverse-BWT memory interference in mibz2 ([#123](https://github.com/lmmx/mibz2/pull/123))
- *(blog)* Block dispatch in mibz2: mutexes, futexes and the scheduler ([#121](https://github.com/lmmx/mibz2/pull/121))
- *(blog)* 32 lanes, 8.6 chains: where the inverse BWT walk spends its cycles ([#119](https://github.com/lmmx/mibz2/pull/119))
- 0.2.1's measured notes (the one-pass build: who gets it, the i9 and CI runs); the compact-Ψ feasibility review after #112 (the 32-row descriptor is what #112 measured; on the i9 a smaller table can add a fifth team at most, about 1.25x the inversion; wikidata-pq's 0.1.4 pin and 4-thread cap found)

### <!-- 6 -->Testing

- *(bwt)* the staged walk's chunk edges, each required; blocks over 2^23 rows refused ([#174](https://github.com/lmmx/mibz2/pull/174))
- witness the walk's deferred handovers ([#159](https://github.com/lmmx/mibz2/pull/159))

### <!-- 7 -->Build System and CI

- *(channel)* head-sha reads the merge back; the expected head goes in expectedHeadSha ([#190](https://github.com/lmmx/mibz2/pull/190))
- *(channel)* the tripwire is a record, not a wake-up; audit runs on every recurring wake ([#189](https://github.com/lmmx/mibz2/pull/189))
- *(channel)* the tripwire flags a misroute as a failing check run on #145 ([#187](https://github.com/lmmx/mibz2/pull/187))
- *(channel)* route the coordinator's messages deterministically, and a tripwire for misroutes ([#186](https://github.com/lmmx/mibz2/pull/186))

### <!-- 9 -->Other

- Call on mibz2, cutting instructions from bzip2 decoding with an autonomous swarm ([#152](https://github.com/lmmx/mibz2/pull/152))

## [0.2.1](https://github.com/lmmx/mibz2/compare/v0.2.0...v0.2.1) - 2026-10-30

The one-core inverse-BWT table build now writes each entry in one pass. The table, the
output and the API are unchanged. Where it applies, the decoder is a few percent faster
at slightly less CPU on the one workstation measured; on CI's 2-core runners only the
build stage is measurably faster.

### Who gets what

- **Every decode that builds a block's table on one core:**
  - the serial decoder;
  - `Plan::Workers`, the default below 6 physical cores (4-core machines, GitHub's
    runners) and at any thread count below 6;
  - the cooperative kernel's fallback for blocks below `MIN_COOP`.
- **Unchanged:** the cooperative teams that 0.2.0 plans on 6 or more physical cores build
  with their own fused build, as before.

### Measured

Both measurements were registered before they ran
([journal](https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-29-one-pass-build.md)).

**The i9-10900K** (10 cores, 20 MiB L3, Linux): this build against `v0.2.0`, whose source
differs by this change alone, 10 paired rounds, the bench corpus's Wikidata blocks.

| threads | speed | CPU-seconds | the plan |
|--:|--:|--:|---|
| 1 | **1.048× (1.026–1.070)** | **0.954× (0.934–0.975)** | serial |
| 2 | 1.025× (0.994–1.056) | 0.980× (0.953–1.006) | workers |
| 4 | **1.035× (1.010–1.060)** | **0.971× (0.945–0.998)** | workers |
| 6 | 0.999× (0.973–1.026) | 1.002× | teams: unchanged, the control |
| 8 | 1.007× (0.986–1.028) | 0.994× | the control |
| 10 | 0.998× (0.961–1.036) | 1.001× | the control |

**CI** (a 2-core runner with SMT, 10 paired rounds):
- the build stage −6.0% (−8.5%, −3.5%);
- no thread count slower in wall time, across 21 corpus and thread-count rows;
- no whole-decoder gain beyond noise: the Wikidata blocks +0.8% (−1.2%, +2.8%) at
  1 thread;
- CPU-seconds rose 1–2% in three rows whose 4 threads share SMT siblings, each interval
  excluding no change. The two machines disagree here, and both results are kept.

Every run on both machines gave the same bytes.

### <!-- 3 -->Performance

- *(bwt)* the one-core build in one pass ([#114](https://github.com/lmmx/mibz2/pull/114)).
  Each entry is written whole, the byte read from the last column at its slot, with no
  init pass and no read-modify-write. Each write is checked against its bucket's end, and
  an end check proves every slot was written once. Counts that aren't the column's own
  fall back to the two-pass build, unchanged. Its unchecked writes are tested under Miri.

### <!-- 4 -->Documentation

- *(changelog)* 0.2.0's notes from its own tag-to-tag paired run and the i9 runs ([#109](https://github.com/lmmx/mibz2/pull/109))
- *(frontier)* the two charts, drawn from frontier.json ([#108](https://github.com/lmmx/mibz2/pull/108))

## [0.2.0](https://github.com/lmmx/mibz2/compare/v0.1.6...v0.2.0) - 2026-10-29

The parallel decoder now shares each block between two threads on machines that have the
cores for it. It does this automatically: `BzReader::parallel`, `decode_parallel`,
`StreamDecoder::new`, and Python's `mibz2.decompress` and `mibz2.open` all choose the
cooperative path, with no change to their signatures.

On the one machine where it has been measured, it decodes the benchmark's Wikidata corpus
1.257× as fast as the best previous configuration (95% interval 1.197–1.320), with the
same bytes, 1.015× the peak memory and 1.214× the CPU time. That machine is an Intel
i9-10900K workstation (10 cores, 20 MiB L3, Linux).

The policy engages only on Linux, and only when 6 or more physical cores serve the
threads. Everywhere else the decoder runs as in 0.1.6: GitHub's runners, 4-core machines,
macOS and Windows.

### Who gets what

- **Linux with 6 or more physical cores,** at the default thread count (all CPUs) or any
  count of 6 or more: teams of 2 threads, each team decoding one block, with as many teams
  as keep their tables within about 0.7 of the L3 cache (4 teams on a 20 MiB L3).
- **Everything else:** a block a worker, exactly as 0.1.6, including the
  same bytes, the same memory and the same speed. That covers fewer than 6 physical cores
  (counted from the CPUs the process may use), macOS, Windows, and any machine whose
  topology can't be read.
- **The gain is largest where 0.1.6 was weakest.** 0.1.6 slowed down past 4 workers on the
  i9: at 20 threads, Python's default there, it ran at 471 MB/s, against 1,007 at 4. The
  0.2.0 default at 20 threads runs at 1,256 MB/s, 2.62× (2.54–2.71) the 0.1.6 default
  there.

### Measured: the cooperative decoder on an i9-10900K (workstation, full decoder)

The pre-registered A/B ([journal](https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-29-cooperative-decoder.md),
run `coop-decoder-20261029T174657Z`, its raw files beside the journal):
- **Machine:** Intel i9-10900K, 10 cores and 20 CPUs (SMT), 20 MiB L3, 125 GB RAM, Linux.
- **Workload:** the bench corpus `wikidata.bz2`: 12 windows of the 2026-10-19 Wikidata
  dump's own bzip2 blocks (level 9), 11.9 MB compressed, 209.7 MB decoded.
- **Level:** the whole decoder, from file to hashed output, every block and stream CRC
  checked.
- **Method:** 20 interleaved rounds, rotated and reversed, back to back and unpinned.
- **Reference:** 4 workers, the previous decoder's best thread count on this machine.

| gate (registered before the run) | result | pass |
|---|--:|---|
| throughput ≥ 1.15× | 1.257× | yes |
| its 95% lower bound > 1.05 | 1.197 | yes |
| same bytes | one output hash in 420 runs, no errors | yes |
| peak RSS ≤ 1.15× | 1.015× | yes |
| CPU-seconds ≤ 1.5× | 1.214× | yes |
| 4 physical cores (CPUs 0–3, one SMT sibling each): no slower | 0.997× (0.980–1.014) | yes |

| decoder | threads | MB/s | against 0.1.6 at 4 (paired, 95%) | peak RSS |
|---|--:|--:|--:|--:|
| 0.1.6 (workers) | 4 | 1,007 | 1 | 76 MB |
| 0.1.6 (workers) | 20 | 471 | 0.479 (0.462–0.497) | 267 MB |
| 0.2.0 (default) | 6 | 1,155 | 1.166 (1.123–1.210) | 64 MB |
| 0.2.0 (default) | 20 | 1,256 | 1.257 (1.197–1.320) | 77 MB |

**Confirmed at the release's own code,** through the default entry point users call
(`BzReader::parallel`), by a second run on the same machine
(`coop-decoder-20261029T181511Z`, 10 rounds, #106's head):
- all six gates passed again: 1.223× (1.163–1.286) at 20 CPUs, one output hash, RSS 1.016×,
  CPU 1.254×, and 1.013× (0.999–1.027) on 4 physical cores;
- at 6 threads, 1.217× (1.178–1.258) at 0.83× the peak RSS;
- both arms ran slower in absolute terms than in the first run, but the ratios reproduced.

These are one machine and one workload. Another CPU, cache size or block mix may gain more,
less or nothing. The figure is the whole decoder's. Earlier inversion-only figures from
the research harness (1.38–1.44×) measured a part of the decoder, and are not this
release's.

### Measured: v0.2.0 against v0.1.6 on CI (tag to tag)

One paired run of the two tags ([run 40930714619](https://github.com/lmmx/mibz2/actions/runs/40930714619)).
- **Method:** 10 rounds on one runner. Each round decodes every corpus once with each build
  at each thread count, the builds' order alternating; v0.1.6 runs as its own harness binary
  (`mibz2@base/ab`).
- **The runner is below the policy's threshold:** it has 2 physical cores, so v0.2.0 runs
  workers there exactly as v0.1.6 did. This run is the check that nothing changed where the
  policy doesn't engage.
- **Change** is v0.2.0 over v0.1.6: the geometric mean of the paired wall ratios, with its
  95% interval. A minus sign means faster.

| corpus | threads | wall, v0.1.6 → v0.2.0 (median) | wall change (95% interval) | peak RSS | same bytes |
|---|--:|--:|--:|--:|---|
| `wikidata` | 1 | 0.864 → 0.862 s | −0.7% (−1.8%, +0.3%) | 26 → 26 MB | yes |
| `wikidata` | 2 | 0.553 → 0.540 s | −2.2% (−4.5%, +0.2%) | 49 → 49 MB | yes |
| `wikidata` | 4 | 0.436 → 0.437 s | −0.4% (−2.4%, +1.6%) | 74 → 74 MB | yes |
| `onebyte` | 1 | 0.022 → 0.022 s | +0.4% (−1.2%, +2.1%) | 33 → 33 MB | yes |
| `onebyte` | 2 | 0.022 → 0.022 s | +1.4% (−1.3%, +4.0%) | 33 → 33 MB | yes |
| `onebyte` | 4 | 0.023 → 0.023 s | +1.1% (−0.7%, +3.0%) | 32 → 33 MB | yes |
| `pattern` | 1 | 0.048 → 0.047 s | −0.6% (−3.6%, +2.5%) | 17 → 17 MB | yes |
| `pattern` | 2 | 0.030 → 0.030 s | +2.0% (−2.3%, +6.5%) | 23 → 23 MB | yes |
| `pattern` | 4 | 0.033 → 0.033 s | −0.4% (−3.9%, +3.2%) | 42 → 42 MB | yes |
| `random` | 1 | 0.160 → 0.160 s | −2.0% (−9.4%, +6.0%) | 23 → 23 MB | yes |
| `random` | 2 | 0.097 → 0.098 s | −1.3% (−5.2%, +2.6%) | 36 → 37 MB | yes |
| `random` | 4 | 0.078 → 0.078 s | +1.0% (−1.6%, +3.7%) | 53 → 51 MB | yes |
| `records` | 1 | 0.209 → 0.210 s | +1.4% (−1.3%, +4.1%) | 24 → 24 MB | yes |
| `records` | 2 | 0.120 → 0.120 s | −0.3% (−2.3%, +1.8%) | 40 → 40 MB | yes |
| `records` | 4 | 0.096 → 0.096 s | −0.3% (−1.0%, +0.5%) | 58 → 59 MB | yes |
| `runs` | 1 | 0.015 → 0.015 s | −0.5% (−1.4%, +0.5%) | 23 → 23 MB | yes |
| `runs` | 2 | 0.015 → 0.016 s | +0.2% (−2.5%, +2.9%) | 23 → 23 MB | yes |
| `runs` | 4 | 0.016 → 0.016 s | +2.2% (−0.7%, +5.1%) | 23 → 23 MB | yes |
| `text` | 1 | 0.100 → 0.100 s | −1.0% (−1.9%, −0.1%) **faster** | 19 → 19 MB | yes |
| `text` | 2 | 0.059 → 0.059 s | −0.3% (−2.9%, +2.4%) | 32 → 32 MB | yes |
| `text` | 4 | 0.048 → 0.048 s | −1.6% (−4.8%, +1.6%) | 52 → 52 MB | yes |

- **Same bytes:** every run of every arm gave the same output hash on every corpus, with no
  error.
- **The resource assessment** found no change at this run's precision: best against best,
  CPU per byte, amplification at each thread count, peak RSS at the most threads.
- **The stage table** (one thread, diagnostic only) marks the build −2.5% and the scanner
  −1.2%, with the total −1.2% (−3.4%, +1.1%). With seven intervals a run, a lone mark can be
  chance, so neither is claimed.

<details><summary>Reproducing this run</summary>

- `gh workflow run bench.yml --ref v0.2.0 -f base=v0.1.6`; artifact
  `bench-pr-11407937dadd94174d789269fca82294f4741713`.
- **Tags:** v0.2.0 at 11407937dadd94174d789269fca82294f4741713; v0.1.6 at
  4fc6af0759da5d600059847f0c450ee77ee2e18c.
- **Runner:** AMD EPYC 7763, 4 cores (2 physical), reported clock 3250.8 MHz; kernel
  7.0.0-1017-azure; image ubuntu24 20261026.354.1.
- **Toolchain and bench:** rustc 1.99.1 (58cf8f9c3 2026-10-13); bench 3.2.0, 10 rounds,
  threads 1, 2 and 4.
- **Corpora:** `wikidata` 11,854,933 bytes, decoded 209,700,196 bytes, SHA-256
  82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915; the others as in
  result.json's `corpora`.

</details>

### <!-- 3 -->Performance

- **Cooperative inverse BWT, chosen automatically** ([#101](https://github.com/lmmx/mibz2/pull/101),
  [#103](https://github.com/lmmx/mibz2/pull/103), [#104](https://github.com/lmmx/mibz2/pull/104),
  [#105](https://github.com/lmmx/mibz2/pull/105), [#106](https://github.com/lmmx/mibz2/pull/106))
  - *Mechanism:* the step bounding the parallel decoder past 4 workers was the shared L3
    cache, not compute. Each block's 3.6 MB table is walked at random, so more blocks at
    once evicted each other's tables. A team of threads now shares one block's table.
    The move-to-front stage records symbol counts every 16 KiB, so the team can build the
    table in parallel (`coop::build_fused`) and walk it in parallel (`coop::walk_room`).
    `Plan::for_threads` chooses teams of 2, capped by the L3 size.
  - *Soundness:*
    - The parallel build checks every window of its counts, and an end check proves the
      table equals the one-thread build's. A refused build falls back to that build.
    - The thread pool waits for every helper on every exit, and resumes a helper's panic
      in the caller.
    - Its tests run under Miri.
    - Every sample shape at levels 1, 5 and 9 decodes to libbzip2's bytes at teams of 2,
      3 and 4.
    - Corrupt inputs fail with the same error, at the same bit, as the one-thread kernel.
  - *Measured:* above (the i9). On CI each pull request's bench showed no change, as
    expected below 6 physical cores.
  - *Trade-offs:* about 1.2× the CPU-seconds for 1.26× the throughput on the i9. Building
    the table in parallel costs about 7–8% of the build on the development VM for its window checks.
    These were kept for soundness.
  - *Evidence:* [the decoder journal](https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-29-cooperative-decoder.md);
    the research record in the cooperative-inversion journal.

### <!-- 0 -->Features

- **`parallel::Plan`, `StreamDecoder::planned` and `BzReader::parallel_planned`** (#105):
  choose the plan by hand. `Plan::Workers(n)` is the 0.1.6 decoder, and
  `Plan::Teams { teams, k }` uses teams of 2, 3 or 4. `bwt::Cooperative<K>` implements
  `InverseBwt`, which gains two provided methods (`wants_checkpoints`,
  `invert_checkpointed`), so existing implementations are unchanged.
- **`mtf::mtf_checkpoints` and `mtf::CHECKPOINT`** (#101).

### Behaviour changes

- `StreamDecoder::new(threads, ..)` may run fewer threads than `threads` on a machine where
  it plans teams. On the i9 it runs 8 threads (4 teams of 2) for any `threads` from 8 to
  20. `StreamDecoder::threads()` reports the threads it runs. The blocks in flight scale to
  the teams: `slots` × teams / `threads`, with at least one block per team.

### Benchmarks and docs

- `ab` gains the arms `mibz2-workers` (the 0.1.6 decoder), `mibz2-coop2/3/4` and
  `mibz2-auto`. `just bench-coop-decoder` runs the release-gate A/B on a workstation.
- The throughput–memory frontier page now has the i9's full-decoder series with measured
  peak RSS. Every worker count from 4 to 20 is dominated, on throughput and RSS, by a team
  configuration. The low-memory end (1–3 workers) is not yet measured.
- The literature study, the marker audit and the ψ census are in the journal.
- **The inverse BWT alone against libsais, measured after the release**
  ([#107](https://github.com/lmmx/mibz2/pull/107)). This is the shipped
  `bwt::Cooperative<K>` against libsais 2.12.1's multi-threaded inversion on the i9, in the
  research harness that measured libsais at 2,356 MB/s. It is a component figure, not the
  decoder's:
  - **Build + walk** (the MTF outside the timing): the library at 3 × 3 reaches 2,430 MB/s
    against libsais's 2,260 in the same run. Paired over 20 rounds that is 1.114×
    (1.027–1.208), median 1.081, at 9 cores, and 1.343× (1.303–1.385) at 8.
  - **With the MTF stage timed:** a tie at 9 cores, 1.022× (0.986–1.059); 1.218×
    (1.182–1.255) at 8.

## [0.1.6](https://github.com/lmmx/mibz2/compare/v0.1.5...v0.1.6) - 2026-10-29

An instrumentation release. Decoding is unchanged from 0.1.5: in one paired run of the two
tags on one runner, every corpus decodes to the same bytes, no change in wall time was
detected at any thread count, and the resource assessment found no change in CPU or peak
memory. 0.1.6 adds an opt-in `trace` feature (the
parallel decoder's own timeline and a footprint ledger) and the public helpers the bench's
walk sweeps use. The Rust API is compatible with 0.1.5, and the Python package is unchanged.

### Measured: v0.1.6 against v0.1.5

One paired run of the two tags ([run 40902365737](https://github.com/lmmx/mibz2/actions/runs/40902365737)).
It ran 10 rounds on one runner. Each round decodes every corpus once with each build at each
thread count, and the builds' order alternates from round to round. v0.1.5 runs as its own
harness binary (`mibz2@base/ab`), so both builds run in the same process shape.

**Change** is v0.1.6 over v0.1.5: the geometric mean of the rounds' paired wall ratios, with
its 95% interval. A minus sign means faster. A figure is marked **faster** or **slower** only
where its interval excludes zero.

| corpus | threads | wall, v0.1.5 → v0.1.6 (median) | wall change (95% interval) | peak RSS | same bytes |
|---|--:|--:|--:|--:|---|
| `wikidata` | 1 | 0.463 → 0.470 s | +0.1% (−3.4%, +3.8%) | 26 → 26 MB | yes |
| `wikidata` | 2 | 0.348 → 0.350 s | −2.6% (−9.2%, +4.6%) | 50 → 49 MB | yes |
| `wikidata` | 4 | 0.311 → 0.307 s | −0.8% (−4.8%, +3.4%) | 74 → 74 MB | yes |
| `pattern` | 1 | 0.025 → 0.025 s | +3.8% (−6.0%, +14.5%) | 17 → 17 MB | yes |
| `pattern` | 2 | 0.021 → 0.018 s | −19.8% (−36.1%, +0.5%) | 24 → 24 MB | yes |
| `pattern` | 4 | 0.024 → 0.022 s | −3.7% (−8.3%, +1.0%) | 43 → 43 MB | yes |
| `text` | 1 | 0.054 → 0.054 s | −0.4% (−2.1%, +1.3%) | 19 → 19 MB | yes |
| `text` | 2 | 0.037 → 0.036 s | +1.6% (−4.9%, +8.7%) | 32 → 32 MB | yes |
| `text` | 4 | 0.032 → 0.032 s | −0.9% (−3.7%, +2.0%) | 50 → 52 MB | yes |
| `random` | 1 | 0.098 → 0.097 s | +0.7% (−2.1%, +3.6%) | 24 → 24 MB | yes |
| `random` | 2 | 0.061 → 0.060 s | +4.8% (−3.7%, +14.1%) | 37 → 37 MB | yes |
| `random` | 4 | 0.051 → 0.049 s | −3.6% (−9.1%, +2.3%) | 52 → 53 MB | yes |
| `records` | 1 | 0.129 → 0.124 s | −3.1% (−8.9%, +3.1%) | 24 → 24 MB | yes |
| `records` | 2 | 0.074 → 0.075 s | +1.6% (−5.8%, +9.6%) | 40 → 40 MB | yes |
| `records` | 4 | 0.060 → 0.060 s | +0.1% (−3.5%, +3.8%) | 60 → 60 MB | yes |
| `runs` | 1 | 0.011 → 0.011 s | −4.8% (−14.8%, +6.3%) | 23 → 23 MB | yes |
| `runs` | 2 | 0.011 → 0.011 s | −0.0% (−4.3%, +4.4%) | 23 → 23 MB | yes |
| `runs` | 4 | 0.011 → 0.011 s | −4.2% (−10.0%, +2.0%) | 23 → 23 MB | yes |
| `onebyte` | 1 | 0.017 → 0.016 s | −0.8% (−2.4%, +0.8%) | 34 → 34 MB | yes |
| `onebyte` | 2 | 0.016 → 0.016 s | +4.5% (−8.9%, +19.8%) | 33 → 33 MB | yes |
| `onebyte` | 4 | 0.017 → 0.016 s | −0.9% (−3.9%, +2.2%) | 33 → 33 MB | yes |

- **Same bytes:** every run of every arm gave the same output hash on every corpus, with no error.
- **The resource assessment** (best against best, CPU per byte, amplification at each thread count, peak RSS at the most threads): no change detected.
- **The stage table does not compare across these tags, so it is not quoted.**
  - Each build runs its own `examples/phases.rs`.
  - v0.1.5's times each stage as its best repeat on the same block, cache-warm.
  - 0.1.6's times the stages once each, in decode order (bench 2.0.0, [#47](https://github.com/lmmx/mibz2/pull/47)).
  - So the table's differences here (+6.6% in total) are the change of method, mostly in the stages that move the most data (build +8.1%, walk +10.8%). They are not the decoder.
  - The decoder's own figures above use one harness for both builds. Two earlier pre-release runs of master against v0.1.5, on another CPU model, agreed with them (runs [40895174632](https://github.com/lmmx/mibz2/actions/runs/40895174632) and [40896779028](https://github.com/lmmx/mibz2/actions/runs/40896779028)).

<details><summary>Reproducing this run</summary>

`gh workflow run bench.yml --ref py-0.1.6 -f base=v0.1.5`. Artifact `bench-pr-4fc6af0759da5d600059847f0c450ee77ee2e18c`.

- **Tags:** v0.1.6 is 4fc6af0759da5d600059847f0c450ee77ee2e18c (tagged `py-0.1.6`), and v0.1.5 is 7ff58cee77250c46c1317830ca8038b331380c05.
- **Runner:** AMD EPYC 9V45 96-Core Processor, 4 cores (2 physical), 4229.4 MHz reported; kernel 7.0.0-1017-azure; image ubuntu24 20261026.354.1.
- **Toolchain and bench:** rustc 1.99.1 (58cf8f9c3 2026-10-13); bench 3.2.0; 10 rounds; threads 1, 2 and 4.
- **Corpora:**
  - `wikidata`: 11,854,933 → 209,700,196 bytes, SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`
  - `pattern`: 1,116 → 19,600,000 bytes, SHA-256 `0f07e2a274b9189677c0f22fd3fc8d15f17324b1f49e651a321239321d4e1407`
  - `text`: 1,502,031 → 12,368,727 bytes, SHA-256 `b92cb4c4cbeb8dfcd163a4060a506a6c3a8a9cee10d3b998ff7e94c7dcac3525`
  - `random`: 7,097,921 → 8,000,000 bytes, SHA-256 `10d444e6204289af739688e54a348940d7915957172ea65035b08b72dfdc3f18`
  - `records`: 9,587,840 → 9,600,000 bytes, SHA-256 `ff8b5f6ed3dd44459908a01a588f6979b2544c6a444c9520741a2aa96c391f8c`
  - `runs`: 37,995 → 10,000,235 bytes, SHA-256 `af3be4d130faf312fe087baee037ca0d802dcf9cd6aefeefe9d9373b25a92fb8`
  - `onebyte`: 51 → 20,000,000 bytes, SHA-256 `22924f5b3187645f97653ae4923df14d2597816c2e57f2c7228bc844119c4a9e`

</details>

### <!-- 1 -->Features

- **`trace`: the parallel decoder's own timeline** ([#53](https://github.com/lmmx/mibz2/pull/53), [#70](https://github.com/lmmx/mibz2/pull/70))
  - *Mechanism:*
    - marks at the decoder's waits and handoffs: input, the scanner's slots, blocks taken, and each block's inversion;
    - a footprint ledger of each worker's buffers (`trace::Buffer`, `Scratch::buffers`).

    It is compiled in only with the opt-in `trace` feature, so the default build carries none of it. `examples/occupancy.rs` reads it (`--features trace`).
  - *Measured:* the default build decodes as 0.1.5 does (the table above).
- **The walk sweep's controls** ([#41](https://github.com/lmmx/mibz2/pull/41), [#49](https://github.com/lmmx/mibz2/pull/49))
  - `walk_segments_flagged_with_prefetch` and `walk_segments_room_with_prefetch` take the prefetch level as `PF_OFF`, `PF_L1`, `PF_L2` or `PF_NTA`.
  - `walk_segments_room_parts` and `SegScratch::assemble` make the room walk's segment assembly a step of its own, which the bench times.
  - The decoder uses the same walk and prefetch as 0.1.5 (`PF_L1`).

### <!-- 2 -->Bug Fixes

- **0.1.5's walk signatures kept** ([#97](https://github.com/lmmx/mibz2/pull/97))
  - `walk_segments_flagged_with` and `walk_segments_room_with` take 0.1.5's `bool` prefetch switch again (`true` is `PF_L1`).
  - The sweep's change of their parameter to a `u8` level ([#41](https://github.com/lmmx/mibz2/pull/41)) would otherwise have broken 0.1.5's callers.

### Measurement and documentation (the decoder's figures did not change)

- **The bench (2.0.0 → 3.2.0):**
  - the stage table in decode order ([#47](https://github.com/lmmx/mibz2/pull/47)), and the segment assembly timed on its own ([#49](https://github.com/lmmx/mibz2/pull/49));
  - the walk sweep timed in decode order ([#45](https://github.com/lmmx/mibz2/pull/45));
  - CPU per byte, amplification and a resource assessment on every pull request ([#69](https://github.com/lmmx/mibz2/pull/69));
  - the contention fingerprint ([#71](https://github.com/lmmx/mibz2/pull/71));
  - bz2ngrs, bz2-ng and libsais as alternatives ([#74](https://github.com/lmmx/mibz2/pull/74), [#75](https://github.com/lmmx/mibz2/pull/75));
  - a PMU probe of what a hosted runner's counters can measure ([#87](https://github.com/lmmx/mibz2/pull/87)).
- **The workstation instruments:** occupancy, scaling, block size, thermal, regimes, the memory-controller counters and the stall profile ([#54](https://github.com/lmmx/mibz2/pull/54), [#56](https://github.com/lmmx/mibz2/pull/56), [#58](https://github.com/lmmx/mibz2/pull/58), [#59](https://github.com/lmmx/mibz2/pull/59), [#61](https://github.com/lmmx/mibz2/pull/61), [#64](https://github.com/lmmx/mibz2/pull/64), [#67](https://github.com/lmmx/mibz2/pull/67)).
- **The documentation site:**
  - Why 32 couriers beat one ([#38](https://github.com/lmmx/mibz2/pull/38));
  - a Status page ([#86](https://github.com/lmmx/mibz2/pull/86));
  - a research blog with link-preview cards ([#96](https://github.com/lmmx/mibz2/pull/96));
  - 0.1.5's notes held to their own tag-to-tag run ([#44](https://github.com/lmmx/mibz2/pull/44)).

## [0.1.5](https://github.com/lmmx/mibz2/compare/v0.1.4...v0.1.5) - 2026-10-27

The inverse BWT's walk now runs in passes at a shared output offset, after libsais's chain
decode. Against 0.1.4, in one paired run on one runner, the Wikidata blocks decode
**9.5% faster at 1 thread** and **7.9% faster at 4 threads**. The output is the same bytes
and peak memory is unchanged.

### Measured: v0.1.5 against v0.1.4

One paired run of the two tags ([run 40564384819](https://github.com/lmmx/mibz2/actions/runs/40564384819)).
It ran 10 rounds on one runner. Each round decodes every corpus once with each build at each
thread count, and the builds' order alternates from round to round. v0.1.4 runs as its own
harness binary (`mibz2@base/ab`), so both builds run in the same process shape.

**Change** is v0.1.5 over v0.1.4: the geometric mean of the rounds' paired wall ratios, with
its 95% interval. A minus sign means faster. A figure is marked **faster** or **slower** only
where its interval excludes zero.

| corpus | threads | wall, v0.1.4 → v0.1.5 (median) | wall change (95% interval) | peak RSS | same bytes |
|---|--:|--:|--:|--:|---|
| `wikidata` | 1 | 0.712 → 0.645 s | −9.5% (−11.1%, −7.9%) **faster** | 26 → 26 MB | yes |
| `wikidata` | 4 | 0.382 → 0.348 s | −7.9% (−10.3%, −5.4%) **faster** | 74 → 74 MB | yes |
| `pattern` | 1 | 0.040 → 0.034 s | −15.5% (−16.7%, −14.3%) **faster** | 17 → 17 MB | yes |
| `pattern` | 4 | 0.029 → 0.026 s | −10.4% (−12.6%, −8.2%) **faster** | 42 → 42 MB | yes |
| `text` | 1 | 0.062 → 0.059 s | −5.2% (−7.3%, −3.1%) **faster** | 19 → 19 MB | yes |
| `text` | 4 | 0.034 → 0.033 s | −4.3% (−8.3%, −0.3%) **faster** | 49 → 49 MB | yes |
| `random` | 1 | 0.131 → 0.129 s | −2.2% (−3.1%, −1.3%) **faster** | 24 → 24 MB | yes |
| `random` | 4 | 0.060 → 0.060 s | −1.3% (−3.6%, +1.1%) | 52 → 52 MB | yes |
| `records` | 1 | 0.167 → 0.163 s | −2.2% (−2.9%, −1.6%) **faster** | 24 → 24 MB | yes |
| `records` | 4 | 0.073 → 0.073 s | −0.5% (−2.6%, +1.6%) | 57 → 57 MB | yes |
| `runs` | 1 | 0.013 → 0.013 s | +1.3% (+0.6%, +2.0%) **slower** | 23 → 23 MB | yes |
| `runs` | 4 | 0.013 → 0.013 s | −0.4% (−3.8%, +3.1%) | 23 → 23 MB | yes |
| `onebyte` | 1 | 0.019 → 0.019 s | +0.8% (−0.1%, +1.8%) | 34 → 34 MB | yes |
| `onebyte` | 4 | 0.019 → 0.019 s | −1.6% (−3.0%, −0.3%) **faster** | 33 → 33 MB | yes |

- **Peak RSS:** every change is within ±1%.
- **Same bytes:** every run of both builds gave the same output hash on each corpus, with no
  errors.
- **CPU at 4 threads on Wikidata:** the workers' summed CPU went from 1.06 to 0.90 s
  (medians). The serial scanner stayed at 0.08 s.
- **Stages, on the Wikidata blocks at one thread** (diagnostic, not the decoder's figures):
  - Method: 5 rounds of `examples/phases.rs`, each stage the best of 2 runs. Figures are
    cycles a byte at the reported 3695 MHz.
  - The walk went from 5.51 to 4.27 c/B: −25.3% (−32.8%, −17.1%).
  - The stage total went from 10.87 to 9.68 c/B: −13.0% (−18.0%, −7.8%).
  - The scanner, MTF + RLE2, the T-vector build and RLE1 show no change.
  - CRC is marked +0.3% (+0.1%, +0.5%), on unchanged code. With seven intervals a run, a
    single mark is chance about one run in three.
- **One runner model:** the walk's gain depends on the CPU. #37's own runs measured −10% at
  1 thread on this model (EPYC 9V74), but −5.8% on a Xeon 6973P-C (below).

<details><summary>Reproducing this run</summary>

- **Rerun:** `gh workflow run bench.yml --ref v0.1.5 -f base=v0.1.4`. The figures are the
  run's summary and its artifact, `bench-pr-7ff58ce…` (`result.json`, `runs.jsonl`,
  `stages.jsonl`).
- **Builds:**
  - v0.1.5 is `7ff58cee77250c46c1317830ca8038b331380c05`, and v0.1.4 is
    `37ca5da87a127f043c92ca3c7a1ae5e3c23871f3`.
  - Both use the release profile: fat LTO, one codegen unit.
- **Runner:** AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3694.6 MHz. Its flags
  include AVX2, AVX-512F, BMI2 and PCLMULQDQ. Kernel 7.0.0-1017-azure, image ubuntu24
  20261026.354.1.
- **Toolchain:** rustc 1.99.1 (58cf8f9c3 2026-10-13).
- **Bench:** version 1.2.0, run by v0.1.5's harness (`ab run`, `bench/report.py`,
  `bench/stages.py`) with each tag's own `examples/ab.rs` and `examples/phases.rs`.
  - 10 rounds, at 1 and 4 threads. The 1-thread runs are pinned to one CPU.
  - The stages ran 5 rounds, each the best of 2.
- **Corpora:**
  - `wikidata` is 12 windows' whole blocks of `wikidata-20261019-all.json.bz2`
    (`bench/wikidata.json`).
  - The others come from `ab gen` at scale 4 (`tests/common/mod.rs`), compressed by
    libbzip2 1.0.8 at -9.
  - `text` is built from the head's own sources, so its bytes change with every source
    change. Within one run, both builds decode the same file.

| corpus | bz2 bytes | output bytes | SHA-256 |
|---|--:|--:|---|
| `wikidata` | 11,854,933 | 209,700,196 | `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915` |
| `onebyte` | 51 | 20,000,000 | `22924f5b3187645f97653ae4923df14d2597816c2e57f2c7228bc844119c4a9e` |
| `pattern` | 1,116 | 19,600,000 | `0f07e2a274b9189677c0f22fd3fc8d15f17324b1f49e651a321239321d4e1407` |
| `random` | 7,097,921 | 8,000,000 | `10d444e6204289af739688e54a348940d7915957172ea65035b08b72dfdc3f18` |
| `records` | 9,587,840 | 9,600,000 | `ff8b5f6ed3dd44459908a01a588f6979b2544c6a444c9520741a2aa96c391f8c` |
| `runs` | 37,995 | 10,000,235 | `af3be4d130faf312fe087baee037ca0d802dcf9cd6aefeefe9d9373b25a92fb8` |
| `text` | 968,954 | 12,137,045 | `ba59df4a9563ca82d68a90de0f1a11574e68928eabca702dbd8d03065b12bf0c` |

</details>

### <!-- 3 -->Performance

- **The walk in passes with a shared offset** ([#37](https://github.com/lmmx/mibz2/pull/37))
  - *Mechanism:*
    - The flagged walk (32 lanes, 512 segments, the prefetch) runs in passes of `room`
      rounds, where `room` is the fewest bytes any lane's buffer has spare.
    - Each lane writes at its base plus the pass's shared offset. A step no longer has the
      per-lane pointer store or the buffer-end test (a load, a compare and a branch).
    - The table, the lanes, the prefetch and the segments are unchanged. The same misses are
      in flight, with fewer instructions a step.
    - It follows libsais 2.12.1's `decode_chains*`.
  - *Measured in its pull request* (head over its base, Wikidata blocks):

    | run | runner | 1 thread | 4 threads |
    |---|---|--:|--:|
    | [db3593d](https://github.com/lmmx/mibz2/actions/runs/40556508472) | EPYC 9V74 | −10.3% (−10.6%, −10.0%) | −9.1% (−12.2%, −5.9%) |
    | [a09c91b](https://github.com/lmmx/mibz2/actions/runs/40561710124) | EPYC 9V74 | −10.5% (−12.7%, −8.2%) | −2.8% (−10.2%, +5.1%), no change |
    | [6c645ef](https://github.com/lmmx/mibz2/actions/runs/40562627744) | Xeon 6973P-C | −5.8% (−7.3%, −4.2%) | −2.7% (−4.1%, −1.3%) |

    In the Xeon run, the walk stage went from 4.37 to 3.81 c/B: −14.2% (−17.8%, −10.5%).
    Peak RSS was unchanged in every run.
  - *Trade-offs:*
    - **Small synthetic corpora at 1 thread:** on the Xeon runner, `onebyte` was +3.2%
      (+0.2%, +6.4%) slower and `runs` +4.6% (+0.9%, +8.4%). Both are 10–14 ms decodes of
      single-symbol or run-only blocks. In the release run above, `runs` is +1.3% and
      `onebyte` shows no change. Both are unchanged or faster at 4 threads. This was
      weighed against −6 to −10% on Wikidata, and kept.
    - **More unchecked code:** 541 lines added to `src/bwt.rs` (its tests included), with unchecked writes.
      They are covered by:
      - their invariants, written on `walk_segments_room_with`;
      - a safe segment oracle, checked after every segmented walk in the tests;
      - test-only counters, which assert that the five handoff transitions are reached;
      - a Miri job (`miri.yml`) on every change to `src/bwt.rs`.
    - **An overrun, never released:** a heap overrun (a pass ending with a buffer exactly
      full) was found during development, then fixed. It has a regression test.
  - *Evidence:*
    - [the PR body](https://github.com/lmmx/mibz2/pull/37), with the soundness case;
    - [its bench comment](https://github.com/lmmx/mibz2/pull/37#issuecomment-6449798695),
      the last run's full table;
    - the journal:
      [the pair walk, "What remains open"](https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-27-pair-walk.md#what-remains-open),
      where the mechanism was identified, and
      [the research synthesis, section 1](https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-27-research-synthesis.md).

### <!-- 1 -->Features

- **Bench 1.2.0: a paired per-stage table** on the Wikidata blocks
  ([#35](https://github.com/lmmx/mibz2/pull/35)). Measurement only.
  - `bench/stages.py` runs each build's `examples/phases.rs` pinned to one CPU, 5 rounds
    of the best of 2.
  - It reports each stage's cycles a byte, head over base, with 95% intervals.
  - The decoder did not change: [its run](https://github.com/lmmx/mibz2/pull/35#issuecomment-6448968226)
    measured Wikidata at −1.3% (−5.7%, +3.4%) at 1 thread and +1.0% (−1.7%, +3.8%) at
    4 threads, neither a change.
  - It costs about 60–100 s a bench run.

### <!-- 4 -->Documentation

- **The journal:** the external research reports and a source-grounded synthesis of them
  ([#33](https://github.com/lmmx/mibz2/pull/33)).

### Downstream: wikidata-pq

These are not decoder figures. They come from wikidata-pq's own bench on its pull request
that bumped the decoder from 0.1.4 to 0.1.5,
[lmmx/wikidata-pq#209](https://github.com/lmmx/wikidata-pq/pull/209)
([its bench comment](https://github.com/lmmx/wikidata-pq/pull/209#issuecomment-6450883879)),
from its full tier: 3 paired rounds on a 4-chunk sample. The pull request was closed
unmerged, under wikidata-pq's rule that a change merges only when an end-to-end figure
improves beyond noise; the pin moves with the next release.

- *Measured:* parity passed: the pipeline, the misfit chunk, a new claims field, and the
  split by this decoder against lbzip2's, byte for byte (68 files).
- *Measured:* the split's CPU on the sample went from 17.2 s to 16.5 s.
- *Measured:* the sample end to end (split, pipeline and finalise) shows no change beyond
  noise: CPU 1m36s and wall 38.12 s, before and after.
- *Modelled:* the full Wikidata release ETA went from 4h34m37s ± 13m13s to 4h32m14s ± 9m20s,
  and the split's share from 1h33m22s ± 3m19s to 1h29m44s ± 2m53s (−3.9%): no change beyond
  noise. wikidata-pq's comment states the model's assumptions.
- *Interpretation, not a measurement:* decoding is about a third of the split's CPU, so the
  one-thread change above predicts about −3% on the split; the change seen is that size,
  and below what that bench resolves.

## [0.1.4](https://github.com/lmmx/mibz2/compare/v0.1.3...v0.1.4) - 2026-10-27

### <!-- 3 -->Performance

- *(build)* 16 equal last-column bytes linked with one 16-wide store ([#7](https://github.com/lmmx/mibz2/pull/7)) ([#32](https://github.com/lmmx/mibz2/pull/32))

### Measured: v0.1.4 against v0.1.3

This block was added after the release, from one paired run of the two tags
([run 40564400962](https://github.com/lmmx/mibz2/actions/runs/40564400962)). The method is
the same as 0.1.5's. It ran on an AMD EPYC 9V45 (4 cores, 4272.5 MHz), a different runner
model from 0.1.5's run, so the two releases' figures do not chain.

| corpus | threads | wall, v0.1.3 → v0.1.4 (median) | wall change (95% interval) | peak RSS | same bytes |
|---|--:|--:|--:|--:|---|
| `wikidata` | 1 | 0.622 → 0.501 s | −17.9% (−19.6%, −16.2%) **faster** | 26 → 27 MB | yes |
| `wikidata` | 4 | 0.374 → 0.328 s | −11.1% (−15.0%, −6.9%) **faster** | 74 → 74 MB | yes |
| `pattern` | 1 | 0.038 → 0.029 s | −24.1% (−25.2%, −23.1%) **faster** | 17 → 17 MB | yes |
| `pattern` | 4 | 0.028 → 0.023 s | −15.1% (−18.4%, −11.7%) **faster** | 42 → 42 MB | yes |
| `text` | 1 | 0.048 → 0.045 s | −7.1% (−9.1%, −5.2%) **faster** | 19 → 19 MB | yes |
| `text` | 4 | 0.031 → 0.028 s | −5.4% (−12.3%, +2.0%) | 49 → 49 MB | yes |
| `random` | 1 | 0.099 → 0.098 s | −0.9% (−2.9%, +1.2%) | 24 → 24 MB | yes |
| `random` | 4 | 0.050 → 0.050 s | −1.8% (−4.5%, +1.0%) | 52 → 51 MB | yes |
| `records` | 1 | 0.126 → 0.128 s | +2.8% (−4.1%, +10.2%) | 24 → 24 MB | yes |
| `records` | 4 | 0.060 → 0.059 s | −0.7% (−3.1%, +1.6%) | 60 → 61 MB | yes |
| `runs` | 1 | 0.011 → 0.011 s | +0.8% (−1.6%, +3.2%) | 23 → 23 MB | yes |
| `runs` | 4 | 0.011 → 0.011 s | −0.5% (−1.8%, +0.9%) | 23 → 23 MB | yes |
| `onebyte` | 1 | 0.016 → 0.016 s | +1.2% (−0.2%, +2.6%) | 34 → 34 MB | yes |
| `onebyte` | 4 | 0.016 → 0.016 s | +5.4% (−8.3%, +21.1%) | 32 → 33 MB | yes |

<details><summary>Reproducing this run</summary>

- **Rerun:** `gh workflow run bench.yml --ref v0.1.4 -f base=v0.1.3`. Its artifact is
  `bench-pr-37ca5da…`.
- **Builds:** v0.1.4 is `37ca5da87a127f043c92ca3c7a1ae5e3c23871f3`, and v0.1.3 is
  `0f2d1bcc5ce588a9af271ad1b5a10014c6c4eb86`.
- **Runner:** AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4272.5 MHz. Kernel
  7.0.0-1017-azure, image ubuntu24 20261026.354.1.
- **Toolchain:** rustc 1.99.1 (58cf8f9c3 2026-10-13).
- **Bench:** version 1.1.0, v0.1.4's harness, which has no stage table. 10 rounds, at 1 and 4
  threads.
- **Corpora:** the same SHA-256s as 0.1.5's run, except `text`, which is built from the
  head's sources. Here it is 942,611 bz2 bytes and 12,078,951 output bytes, SHA-256
  `8aea7b3193bf0620d4d23191719a2989c753483da0561647959daa02b715ed7b`.
- **In #32's own run** ([comment](https://github.com/lmmx/mibz2/pull/32#issuecomment-6448157133)),
  on an AMD EPYC 7763, Wikidata measured −16.4% (−17.4%, −15.4%) at 1 thread and −15.5%
  (−18.6%, −12.3%) at 4 threads.

</details>

## [0.1.3](https://github.com/lmmx/mibz2/compare/v0.1.2...v0.1.3) - 2026-10-27

### <!-- 1 -->Features

- *(bench)* every alternative on master's run, single-threaded decoders shown as their 1-thread runs ([#26](https://github.com/lmmx/mibz2/pull/26))

### <!-- 3 -->Performance

- *(huffman)* up to two symbols a table lookup, four lookups a refill ([#8](https://github.com/lmmx/mibz2/pull/8)) ([#24](https://github.com/lmmx/mibz2/pull/24))
- *(walk)* each lane's next T-vector entry prefetched as soon as its row is known, 32 lanes and 512 segments ([#21](https://github.com/lmmx/mibz2/pull/21))

### <!-- 4 -->Documentation

- *(site)* Alternatives, the page and its chart: mibz2 beside each other decoder now ([#20](https://github.com/lmmx/mibz2/pull/20))

## [0.1.2](https://github.com/lmmx/mibz2/compare/v0.1.1...v0.1.2) - 2026-10-27

### <!-- 3 -->Performance

- *(walk)* the segment starts flagged in their own T-vector entries, raw per-lane write pointers, no idle-lane test until the last segments ([#12](https://github.com/lmmx/mibz2/pull/12))

### <!-- 4 -->Documentation

- *(site)* the header logo and favicon, the home page's illustration without its shadows ([#17](https://github.com/lmmx/mibz2/pull/17))
- README badges (PyPI, crates.io, docs.rs, the docs site, the licence) ([#15](https://github.com/lmmx/mibz2/pull/15))
- the documentation site (Zensical, GitHub Pages), its benchmark history and competitors pages from bench-results ([#14](https://github.com/lmmx/mibz2/pull/14))
- CHANGELOG.md for 0.1.0 and 0.1.1, and Release Notes, a version's GitHub release made or updated from it ([#3](https://github.com/lmmx/mibz2/pull/3))

### <!-- 7 -->Build System and CI

- Benchmark, the decoder timed on each pull request against its base, and on master against pbz2 and lbzip2 ([#11](https://github.com/lmmx/mibz2/pull/11))

### <!-- 8 -->Styling

- Cargo.toml in taplo 0.10's sorted key order (pre-commit.ci) ([#13](https://github.com/lmmx/mibz2/pull/13))

### <!-- 7 -->Build System and CI

- Tag Release, a commit tagged at the version it already holds and CI started on the tag ([#2](https://github.com/lmmx/mibz2/pull/2))
- Release Notes, a version's GitHub release made or updated from this changelog, on the commit the version was released from

## [0.1.1](https://github.com/lmmx/mibz2/compare/v0.1.0...v0.1.1) - 2026-10-27

### <!-- 9 -->Other

- the same code as 0.1.0, released at the next version by Deploy Release: the crate and the Python package (wheels for the same platforms) at one version, from one commit

## [0.1.0](https://github.com/lmmx/mibz2/releases/tag/v0.1.0) - 2026-10-27

### <!-- 1 -->Features

- a bzip2 decoder, block-parallel, its inverse BWT walked as 12 interleaved chains within each block, SIMD kernels (AVX2, PCLMULQDQ) chosen at run time with a portable twin each; byte for byte libbzip2's output, every block's and every stream's CRC checked, randomised blocks (bzip2 0.9.0 and earlier) refused
- `decode`, `decode_parallel`, `BzReader` (on the calling thread, or block-parallel), `StreamDecoder` (compressed chunks written from one thread, decoded bytes read from another), `scan` (block boundaries without decoding) and `decode_blocks` (any range of whole blocks), errors with the block and bit where decoding failed
- the Python package: `decompress`, `open` (a buffered binary file over a path or a binary file object) and `BZ2Reader.verified`, true only once a read reaches the end with every CRC matched; `DecodeError`; abi3 wheels for Python 3.10+ on Linux x86-64 and aarch64, macOS x86-64 and arm64, and Windows x64 ([#1](https://github.com/lmmx/mibz2/pull/1))

### <!-- 4 -->Documentation

- the decoder's development journals from lmmx/wikidata-pq (its design and CI figures, the v2 plan, the bake-off against pbz2, lbzip2, fbz, the `lbzip2` crate and indexed_bzip2, the BWT inversion literature inventory, the compute frontier) ([#1](https://github.com/lmmx/mibz2/pull/1))

### <!-- 7 -->Build System and CI

- the decoder's history moved from lmmx/wikidata-pq, and the packaging and release workflows of lmmx/pq-glue: the crate and the Python package at one version, published to crates.io and PyPI by Trusted Publishing ([#1](https://github.com/lmmx/mibz2/pull/1))
