# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.13](https://github.com/lmmx/mibz2/compare/v0.5.12...v0.5.13) - 2026-11-22

### <!-- 3 -->Performance

- *(crc)* portable CRC sliced by 16, its lookups XORed as a balanced tree ([#763](https://github.com/lmmx/mibz2/pull/763))
- *(block)* the default path's block reader from Carbon's cycle 37: an L1-resident Huffman decoder and leaner block headers ([#745](https://github.com/lmmx/mibz2/pull/745))
- *(parallel)* Wormhole's plateau stack: owned run-route output, in-place RLE1, tiled CRC, a 12 MiB ready cap and a ready ring ([#739](https://github.com/lmmx/mibz2/pull/739))

### <!-- 4 -->Documentation

- *(blog)* correctness is uniform: a dek that passes, and lighter sentences ([#759](https://github.com/lmmx/mibz2/pull/759))
- *(blog)* correctness is uniform, cost is not ([#755](https://github.com/lmmx/mibz2/pull/755))
- *(skills)* a post from a discussion is briefed in two transmissions, in files ([#754](https://github.com/lmmx/mibz2/pull/754))
- *(blog)* correctness and speed in a decoder that adapts to its input, version 1 ([#752](https://github.com/lmmx/mibz2/pull/752))
- *(skills)* the coordinator merges a post's version 1 within minutes, without waiting for CI ([#737](https://github.com/lmmx/mibz2/pull/737))
- *(skills)* iterate, ship, re-ship: a post's first version goes live in minutes ([#736](https://github.com/lmmx/mibz2/pull/736))
- *(blog)* two centuries of tools, a claimed Riemann proof's toolkit and our search ([#732](https://github.com/lmmx/mibz2/pull/732))
- *(blog)* admitting dense blocks to the Open kernel, 29% less peak memory ([#733](https://github.com/lmmx/mibz2/pull/733))
- *(skills)* results posts are about the result, titled technically, without process or anecdote ([#731](https://github.com/lmmx/mibz2/pull/731))
- *(changelog)* 0.5.12, its release scaling and crossover on the i9, and the attribution row ([#730](https://github.com/lmmx/mibz2/pull/730))

### <!-- 9 -->Other

- Decompression is the harder problem, a conversation with Thomas Murphy ([#761](https://github.com/lmmx/mibz2/pull/761))
- *(runner)* one workstation runner for sibling repos: a machine-wide lock, round-robin across repos, per-repo state ([#760](https://github.com/lmmx/mibz2/pull/760))

- **37% less peak memory at 10 threads, and 4–6% faster from 6 to 20 threads, on the i9
  (#739).** The parallel decoder's output path was reworked, from Wormhole's research line:
  - each run's output is owned and written straight into place;
  - RLE1 is undone in place;
  - the CRC is computed in tiles;
  - finished blocks wait in a ready ring, capped at 12 MiB.

  The adoption run (`bench-wormhole-plateau`, evidence a811364d, sha256 9faac67b…c92f) measured
  head over base, unpinned, Wikidata ×4, 12 rounds:

  | threads | wall | CPU | peak RSS |
  |--:|---|--:|---|
  | 6 | 0.944 [0.935, 0.952] | 0.942 | 0.677 |
  | 10 | **0.963** [0.943, 0.983] (judged) | 0.937 | **0.633** (87.0 → 54.6 MiB) |
  | 16 | 0.962 [0.956, 0.968] | 0.926 | 0.637 |

- **A faster block reader: 2% faster on one thread, and 11–14% faster at 8–10 threads (#745).**
  - **What changed:** the reader that finds and decodes each block's Huffman codes now keeps
    its decoding tables in L1 cache and reads leaner block headers. The design is from Carbon's
    research line.
  - **Why it pays at high thread counts:** that reader is serial, and at 8–10 threads it was
    the cap on the whole decoder, so a faster one raises the plateau.
  - **The run:** re-qualified on master with #739 (`bench-block-reader-on-wormhole`, evidence
    026062a9, sha256 8629d6b9…7108), head over base, Wikidata ×4, 16 rounds: 1 thread
    **0.979** [0.969, 0.989] (judged), 8 threads **0.858** [0.836, 0.880], 10 threads
    **0.895** [0.875, 0.916], RSS at most 1.057.
- **A portable CRC sliced by 16 (#763).**
  - **What changed:** the CRC used where the hardware instruction (PCLMULQDQ) is unavailable,
    and for pieces under 128 bytes, now reads 16 bytes a step. It combines its 16 table
    lookups as a balanced tree of XORs.
  - **CI's bench:** Wikidata on 1 thread −5.5% [−6.4, −4.5], text on 1 thread −8.6%
    [−9.9, −7.3], with no loss.
  - **Caveat:** CI's runners have the hardware CRC, so part of that 1-thread gain may be
    layout. The kernel's own gain is on machines without it, such as aarch64.

**The worker counts are unchanged.** The recalibration (`bench-recalibrate-plateau`, evidence
472ce552, sha256 9e87c11d…dc50b; RELEASING §5c) ran on master at #739 and #745.
- **Its registration:** corrected before the data, with a 2 s settle and a 9/27 arm (#145
  REVIEW 7084586885, LOG 7084590503).
- **The verdict:** it kept every shipped trim, pinned and unpinned.
- **What more workers trade:** they help Wikidata and cost text by about as much. 8/32 over
  7/28 at pinned 10 is 0.935 [0.919, 0.951] on Wikidata, where Holm rejects, but 1.109 on text.
  The same holds at 12–20 threads unpinned. The loss guard holds them all.
- **A refuted prediction:** 9 workers with 27 slots, predicted at −14.7%, came out 1.083
  [1.024, 1.145] at unpinned 10.
- **A lead:** a worker count chosen from the input's own behaviour could take the Wikidata gain
  without the text loss. It would need a registration of its own.

**The paired run on CI** compared v0.5.12 at 788ebe3e with master at cb7121e8 (Benchmark run
45068896168, AMD EPYC 9V45, 4 CPUs, 10 rounds):
- **Outputs:** parity ok on all 420 runs.
- **Wall:** the best of each build −1.3% [−2.4, −0.0], faster.
- **Stages:** the serial scan fell from 0.30 to 0.25 ns/B, and the pipeline from 2.74 to
  2.71 ns/B.
- **CPU:** amplification on the runner's 2 physical cores rose 6.5–6.9%. On the i9, #739
  lowered CPU (0.93–0.94 at 6–20 threads), so this is the runner's shape, not a cost on a
  many-core machine.

**Features reviewed (RELEASING §3.3):** none was removed in this release.
- **To check:** `memory-lever` and `phase-counters`, against their removal conditions.
- **To fix:** `owned-writer` has no retention note.
- Both go in a separate PR.

**Release scaling on the i9** (`release-scaling/run-20261122T212737Z`, evidence fb315cff, sha256
5c336484…7457; recorded by #774): 0.5.12 over 0.5.13 on Wikidata.
- **Wall:** 1.085 [1.058, 1.112] at 2 threads, 1.080 at 8, 1.115 [1.083, 1.148] at 10 and 1.130
  [1.115, 1.145] at 16. At 1 thread it is 1.023 [0.981, 1.066], inconclusive.
- **Peak memory:** from 62–66 MB down to 42–43 MB at 8–16 threads.
- **Output:** the same bytes on every arm.

**The team threshold stays** (`crossover/run-20261122T213836Z`, evidence a40670c5, sha256
ad658393…3e66; rule of LOG 6896455773).
- **No forced arm is faster:** neither one block a thread nor teams of 2 or 3 beats the shipped
  decoder at any count.
- **One block a thread:** it spans 1 where it runs the shipped plan (5, 7 and 9 threads), and is
  slower at 4, 6, 8 and 10 (1.027–1.272).
- **The teams:** they lose everywhere, 1.14–9.9×.

## [0.5.12](https://github.com/lmmx/mibz2/compare/v0.5.11...v0.5.12) - 2026-11-22

### <!-- 3 -->Performance

- *(open)* construction stack, GradedHandle/Prefix15 ([#714](https://github.com/lmmx/mibz2/pull/714)) + Kitchen Sink, Wormhole L4/L5, Mechanisms splice ([#719](https://github.com/lmmx/mibz2/pull/719))

### <!-- 4 -->Documentation

- *(changelog)* 0.5.11's crossover, the team threshold stays ([#713](https://github.com/lmmx/mibz2/pull/713))
- *(blog)* a memory gain is a gain, the change our speed-only rule nearly shelved ([#706](https://github.com/lmmx/mibz2/pull/706))

### <!-- 9 -->Other

- *(release-scaling)* 0.5.11 against 0.5.10 on the i9; attribution row; CHANGELOG ([#712](https://github.com/lmmx/mibz2/pull/712))

- **The Open kernel's construction stack (#719, on #714): 2–5% faster at 8, 10, 16 and 20
  threads on the i9, with about 18% less retained scratch per worker.** It builds the Open
  kernel's structures with less work and in less memory, from several research lines:
  - Wormhole: GradedHandle, Prefix15 and BidiBottom (#714, then the L4 bottom coordinates);
  - Kitchen Sink (Carbon, then Neon): strip and band packing, an interned dictionary and
    pass-one row totals;
  - Mechanisms: the elimination splice;
  - Codex: the native port.

  Measured natively (AMD EPYC 9V45, 4-CPU quota, 12 paired rounds), the six-block one-thread
  kernel sum took 0.872 [0.869, 0.875] of master's time, and the four-worker pass 0.880
  [0.877, 0.883]. Retained Open scratch fell from 1,377,699 to 1,128,960 bytes per worker after
  all 431 blocks (−18.1%).
  - **The adoption run on the i9** (`bench-open-stack`, open-stack/run-20261122T022924Z,
    evidence 91f58375, sha256 50c832fd…1ba) used R4's rule, registered before the data. Wall,
    head/base, unpinned, Wikidata ×4, 16 rounds:

    | threads | wall | label |
    |---|---|---|
    | 6 | 0.952 [0.944, 0.961] | faster |
    | 8 | 0.987 [0.960, 1.015] | inconclusive |
    | 10 | 0.975 [0.967, 0.984] | faster |

    CPU was 0.949–0.962, and median peak RSS 0.97–0.99. The diagnostic pass without two-ended
    emission (head/nobi 1.001 / 0.991 / 0.989) puts the whole gain in the construction.
  - **The output is byte-identical** on all 431 blocks. `examples/bzcat` at 1, 4, 8 and 10
    threads matches `bzip2 -dc` on the whole Wikidata stream.

**The paired run on CI: within noise on every corpus.** It ran v0.5.11 at 839631a1 against
master at 69961392 (Benchmark run 44980197725, Intel Xeon Platinum 8573C, 4 CPUs, 10 rounds). At
1–4 threads the Open kernel is not on the route there. Wikidata's wall, master over v0.5.11:

| threads | 1 | 2 | 4 |
|---|--:|--:|--:|
| wall | 1.005 [0.997, 1.013] | 1.011 [0.997, 1.026] | 0.998 [0.975, 1.022] |

Wikidata's peak RSS is 28 → 28, 56 → 57 and 86 → 84 MB.

**The worker-count recalibration (RELEASING §5c): no trim moves.** `bench-recalibrate-stack`
(recalibrate-stack/run-20261122T030603Z, evidence 9f203342, sha256 88d062e0…b2) measured every
cell at master 53f7ecf9, pinned and unpinned. Every count keeps its shipped trim.
- **The one near-miss:** pinned at 4 threads, the lazy 3/12 over the eager 3/12, wall 0.982
  [0.966, 0.998] at 0.620 of the peak memory. Holm does not reject it among 12 cells, so it
  isn't adopted. A separately registered memory-route confirmation (PERFORMANCE-POLICY §2) will
  test it.
- **MONODROMY's record 89,** frozen before the data, predicted that every trim but pinned 4
  would stay. That held. The pinned-4 move went the predicted way but missed the bar.

**Release scaling** (§5; release-scaling/run-20261122T035641Z, evidence 81341c9c, 10 rounds)
against 0.5.11 on the i9, the modelled step:
- 8 threads: +4.3% (+2.5, +6.2)
- 10 threads: +2.7% (+1.7, +3.8)
- 16 threads: +2.3% (+0.4, +4.1)
- 20 threads: +1.9% (+0.0, +3.9)
- 1, 2, 4 and 12 threads: within noise.
- CPU: 0.5.11 takes 1.043–1.057× 0.5.12's from 8 to 20 threads.
- Median peak RSS: unchanged, 62.3 → 61.5 MiB at 8 threads, 66.3 → 65.7 MiB at 10.
- The run's own paired wall ratios, 0.5.11 over 0.5.12: 1.041 [1.025, 1.058] at 8 threads,
  1.027 [1.023, 1.032] at 10, 1.022 [1.014, 1.030] at 16 and 1.048 [1.004, 1.093] at 20. At 1,
  2, 4 and 12 threads they span 1: 0.995 [0.966, 1.024], 1.014 [0.992, 1.036], 0.998 [0.969,
  1.027] and 1.002 [0.969, 1.035].
- lbzip2 takes 5.02–7.55× 0.5.12's wall time.

The step has its row in `docs/journal/release-attribution.md` (§5b).

**The crossover** (§5a; `bench-crossover`, crossover/run-20261122T040736Z, evidence ab503838, 10 rounds on Wikidata): the team threshold stays. No forced arm beats the shipped decoder at any count.
- **Teams:** `mibz2-team2` and `mibz2-team3` take 1.22–3.60× the shipped wall time; from 8 to 20 threads, 1.82–3.60× (at 10 threads, 2.271 and 1.827).
- **One worker per thread:** `mibz2-workers-w` is never faster. Its interval spans 1 at 4, 5, 7 and 9 threads; elsewhere it takes 1.042–1.239×.

**Caveats:**
- **All the i9 figures are from one machine.** The trims apply only to that measured machine.
- **At 8 threads the adoption run was inconclusive,** while the release scaling's paired 8-thread
  ratio is 1.041 [1.025, 1.058]. The two runs differ in corpus (Wikidata ×4 against Wikidata),
  placement (unpinned against one CPU a core beside a held spinner) and rounds (16 against 10).
- **The attribution row uses the ledger's modelled step,** the lineage fit through lbzip2: 8
  threads +4.3% (+2.5, +6.2) clears the bar; 20 threads' +1.9% does not.
- **The recalibration's unpinned-10 DRAM diagnostic is void,** from a recipe bug; it decides
  nothing.
- **CI's 4 CPUs can't show the gain:** its 1–4 thread arms don't run Open.

## [0.5.11](https://github.com/lmmx/mibz2/compare/v0.5.10...v0.5.11) - 2026-11-21

### <!-- 1 -->Features

- *(site)* why each release happened: changelog notes and linked posts on the History page ([#702](https://github.com/lmmx/mibz2/pull/702))

### <!-- 3 -->Performance

- *(open)* admit dense blocks (width certificate) with producer buffers in edges ([#703](https://github.com/lmmx/mibz2/pull/703))

### <!-- 4 -->Documentation

- *(changelog)* 0.5.10's crossover, the team threshold stays ([#700](https://github.com/lmmx/mibz2/pull/700))
- *(changelog)* 0.5.10, the memory diet's i9 result and the recalibration ([#698](https://github.com/lmmx/mibz2/pull/698))

### <!-- 9 -->Other

- *(release-scaling)* 0.5.10 against 0.5.9 on the i9 (release-scaling/run-20261121T130559Z); its attribution row ([#699](https://github.com/lmmx/mibz2/pull/699))

- **Dense-block admission (#703): peak memory about 28% lower from 8 to 20 threads on the i9, and faster too.**
  - **Admission:** the five densest full Wikidata blocks (r/n 0.15–0.21), which fell back to
    Segmented at about 5.6 MiB per worker, now take the Open kernel under a cap. All 233 full
    run-route blocks are admitted, where 228 were before.
  - **Accepted as a memory gain** (PERFORMANCE-POLICY §2, #704), on a native paired run: peak
    RSS 0.741 at 8 threads and 0.783 at 10, with wall within noise.
  - **Byte-identical output** on all 431 blocks.

**Release scaling** (§5; release-scaling/run-20261121T230111Z, evidence 4127c26f) against 0.5.10
on the i9:
- 8 threads: +2.1% (+1.2, +3.0)
- 10 threads: +4.2% (+3.3, +5.1)
- 12 to 20 threads: +3.6% to +4.8%
- Median peak RSS: 87.0 → 63.4 MiB at 8 threads, 93.9 → 66.5 MiB at 10.

The memory gain also gave wall time on the i9's shared L3, which the native run's 2 MiB-L2 box
couldn't show. The step has its row in `docs/journal/release-attribution.md` (§5b).

**The crossover** (§5a; `bench-crossover`, crossover/run-20261121T231207Z, evidence b91a7024, 10 rounds on Wikidata): the team threshold stays. No forced arm beats the shipped decoder at any count.
- **Teams:** `mibz2-team2` and `mibz2-team3` take 1.08–8.21× the shipped wall time.
- **One worker per thread:** `mibz2-workers-w` is never faster. Its interval spans 1 at 5, 7 and 9 threads; elsewhere it takes 1.037–1.383×.

## [0.5.10](https://github.com/lmmx/mibz2/compare/v0.5.9...v0.5.10) - 2026-11-21

### <!-- 3 -->Performance

- *(open)* memory diet, Endpoint in edges and queues in out ([#692](https://github.com/lmmx/mibz2/pull/692))

- **The Open kernel's memory diet (#692): 5–8% faster from 6 to 10 threads on the i9, at about
  0.89 of the peak memory.** Two research lines' results, combined and ported natively:
  - Wormhole's W1 (cycle 5) keeps the endpoint elimination's state inside `edges`' buffer, which
    is dead until `edges` fills.
  - Mechanisms' OutQ16 (cycle 16) stores the thin-run queues as 16-bit ranks in 1 KiB chunks of
    the output buffer, which is dead until emit.

  Measured natively, per-worker scratch falls from 4,236 to 1,200 KiB (0.283×), so 7–10 workers
  fit the i9's 20 MiB L3. The adoption run (`bench-open-diet`, open-diet/run-20261121T121714Z,
  evidence c56a1371, sha256 a0675fb1…96d2) used R4's rule, registered before the data. Wall,
  head/base, unpinned:

  | threads | wall | label |
  |---|---|---|
  | 6 | 0.952 [0.950, 0.955] | faster |
  | 8 | 0.917 [0.894, 0.940] | faster |
  | 10 | 0.931 [0.916, 0.946] | faster, the judged count |

  CPU was 0.950 / 0.870 / 0.922, and median peak RSS 0.898 / 0.887 / 0.890.
  - On CI's 4 CPUs the gain doesn't show: its 1–4 thread arms don't run Open. The paired bench
    of v0.5.9 against master (run 44903833563, AMD EPYC 9V74) is within noise everywhere except
    wikidata at 1 thread, at +0.8%.
  - The output is byte-identical, and new tests cover the overflow chunks and the 2^16 epochs.

- **The worker-count recalibration (RELEASING §5c): no trim moves.** `bench-recalibrate-diet`
  (recalibrate-diet/run-20261121T124719Z, evidence 9326d856, sha256 643e6a32…3caa) measured
  every cell at master 23e40f77:
  - pinned at 4, 6, 8 and 10 threads;
  - unpinned at 4, 6, 8, 10, 12, 16 and 20.

  Holm rejected no alternative, so every reference stays. Wider arms are held back on memory, as
  in 0.5.8's run. MONODROMY's record 88, frozen before the data, predicted that no trim would move.

**Release scaling** (§5; release-scaling/run-20261121T130559Z, evidence 479eeb0e): against 0.5.9 the
release is faster at 8 threads and above.
- 8 threads: +6.5% (+5.9, +7.2)
- 10 threads: +9.0% (+7.2, +10.8)
- 12 to 20 threads: +5.2% to +7.4%

The step has its row in `docs/journal/release-attribution.md` (§5b).

**The crossover** (§5a; `bench-crossover`, crossover/run-20261121T131655Z, evidence 2e7e6526, 10 rounds on Wikidata): the team threshold stays. No forced arm beats the shipped decoder at any count.
- **Teams:** `mibz2-team2` and `mibz2-team3` take 1.09–7.90× the shipped decoder's wall time.
- **One worker per thread:** `mibz2-workers-w` is never faster. Its interval spans 1 at 5, 7 and 9 threads; at the other counts it takes 1.041–1.409×.

### <!-- 4 -->Documentation

- *(skills)* blog lessons from #689, the material's centre, our experiment, real figures ([#691](https://github.com/lmmx/mibz2/pull/691))
- *(blog)* besieging hard problems, cheap checks first, after the Riemann proof ([#689](https://github.com/lmmx/mibz2/pull/689))
- *(changelog)* 0.5.9's release scaling and crossover, the team threshold stays ([#688](https://github.com/lmmx/mibz2/pull/688))
- *(blog)* fast alone, slow in a crowd, which speed-ups survive many cores ([#682](https://github.com/lmmx/mibz2/pull/682))

## [0.5.9](https://github.com/lmmx/mibz2/compare/v0.5.8...v0.5.9) - 2026-11-21

### <!-- 3 -->Performance

- *(parallel)* the lazy kernel at 4 threads on the measured i9, unpinned ([#678](https://github.com/lmmx/mibz2/pull/678))

### <!-- 4 -->Documentation

- *(blog)* sixteen at a stroke, the fixed-width writes in mibz2 0.5.8's Open kernel ([#674](https://github.com/lmmx/mibz2/pull/674))

- **0.5.8's worker-count recalibration (RELEASING §5c): one change, unpinned at 4 threads on the
  measured i9.** #668 changes the open kernel's per-block work, so every trim was measured again
  on master at 70b18ca4 (`just bench-recalibrate-058`, registered in
  `docs/journal/2026-11-21-recalibration-058-registration.md`; recalibrate-058/run-20261121T020723Z,
  evidence d353b473, sha256 bed46d52…8d79). Unpinned, at exactly 4 threads, the default decoder
  now inverts with the lazy kernel (`OpenCells`) in place of eager `Cells`, at the plan's four
  workers and window: lazy 4/16 over eager 4/16, 0.9452 [0.9225, 0.9685] of the wall on
  Wikidata ×4 (Holm p 1.71e-04 among 14 cells), at 0.927 of the peak memory (guards
  inconclusive: text 1.0666 [0.9778, 1.1634], random 0.9921 [0.8780, 1.1210]).

  Every other trim is unchanged: pinned at 4 (eager 3/12), 6, 8 and 10 threads, where the pinned
  family adopted nothing, and unpinned at 6, 8, 10, 12, 16 and 20. So are the plan,
  `LAZY_FROM_THREADS` and every other machine. The output is byte-identical.

**Release scaling** (§5; release-scaling/run-20261121T094127Z, evidence 221dadcb, #685): no
step from 0.5.8 reaches 3% at any count, so the release adds no attribution row. The run pins
its threads, so #678's unpinned 4-thread cell isn't on its route.

**The crossover** (§5a; `bench-crossover`, crossover/run-20261121T095223Z, evidence 9f0bfba2,
10 rounds on Wikidata): the team threshold stays. No forced arm beats the shipped decoder at any
count.
- **Teams:** `mibz2-team2` and `mibz2-team3` take 1.07–7.34× the shipped wall time.
- **One worker per thread:** `mibz2-workers-w` is never faster. Its interval spans 1 at 5, 7 and 9
  threads; elsewhere it takes 1.064–1.621× the shipped wall time.

## [0.5.8](https://github.com/lmmx/mibz2/compare/v0.5.7...v0.5.8) - 2026-11-21

### <!-- 3 -->Performance

- *(open)* fixed-width queue fill, Vec16 queues and the 16-byte emit ([#668](https://github.com/lmmx/mibz2/pull/668))

### <!-- 4 -->Documentation

- *(blog)* research by obligation, modelling the work rather than the algorithms ([#663](https://github.com/lmmx/mibz2/pull/663))
- *(blog)* where the decoding work goes, random IDs in repetitive JSON ([#662](https://github.com/lmmx/mibz2/pull/662))
- *(blog)* train tracks, carrying a tangle of paths on a few weighted branches ([#660](https://github.com/lmmx/mibz2/pull/660))
- *(blog)* rewrite Open Returns ([#651](https://github.com/lmmx/mibz2/pull/651))
- *(blog)* Open Returns, picking the decoder block by block ([#650](https://github.com/lmmx/mibz2/pull/650))
- *(changelog)* 0.5.7's crossover: the team threshold stays ([#649](https://github.com/lmmx/mibz2/pull/649))
- *(changelog)* 0.5.7's notes: the open kernel, its evidence and caveats ([#643](https://github.com/lmmx/mibz2/pull/643))

### <!-- 9 -->Other

- *(release-scaling)* 0.5.7 against 0.5.6 on the i9; its figures and attribution row ([#648](https://github.com/lmmx/mibz2/pull/648))

## [0.5.7](https://github.com/lmmx/mibz2/compare/v0.5.6...v0.5.7) - 2026-11-19

A kernel release. On LazyCells' admitted blocks (r/n from 1/64 to 0.15: Wikidata and other
repetitive data), the workers now invert with the open kernel. Everywhere else they run
LazyCells as shipped. The output is byte-identical.

**On the i9-10900K** (open-port-3/run-20261119T171047Z, the one registered attempt, verdict ADOPT):

| | 8 threads | 10 threads | 20 threads | peak RSS |
|---|--:|--:|--:|--:|
| Wikidata ×4, wall | 0.954 | 0.934 | 0.940 | 0.76–0.78 |
| held-out Wikidata, wall | 0.944 | 0.933 | 0.927 | 0.78–0.81 |

- **Sustained** (10 threads back to back): 0.954, at 0.74 of the peak memory.
- **Corpora with no admitted block** (27 cells): pooled 1.0003 [0.9954, 1.0051], with no cell
  flagged.
- **Guard corpora with admitted blocks** (12 cells): no slowdown under Holm; points 0.982–1.006.

**The worker trims** were recalibrated for the new kernel (RELEASING §5c). One changes, at
6 threads (below).

**The paired run on CI** (the last tag against master, Benchmark run 44588824608, AMD EPYC 7763,
4 cores) is neutral. At 1–4 threads the open kernel is not on the route there: Wikidata
−0.7%, −0.2% and +0.9%, every interval spanning 0.

**The release scaling** (0.5.7 against 0.5.6 on the i9-10900K, `bench-release-scaling`,
release-scaling/run-20261119T203443Z, 10 rounds): 0.5.6's wall time over 0.5.7's on Wikidata, with
95% intervals. Above 1 means 0.5.7 is faster.

| threads | 1 | 2 | 4 | 8 | 10 | 12 | 16 | 20 |
|---|--:|--:|--:|--:|--:|--:|--:|--:|
| wall | 0.988 [0.953, 1.025] | 0.991 [0.960, 1.022] | 1.011 [0.980, 1.043] | **1.108** [1.088, 1.127] | **1.073** [1.061, 1.085] | **1.081** [1.045, 1.119] | **1.074** [1.039, 1.110] | **1.091** [1.061, 1.123] |

0.5.7 is 7–11% faster from 8 threads up. At 8 threads its peak memory is 96 MB, against 125 MB
for 0.5.6. Below 8 threads the open kernel is not on the route, and nothing changes. The step
has its row in `docs/journal/release-attribution.md` (§5b).

**The crossover** (§5a; `bench-crossover`, crossover/run-20261119T205115Z, evidence 4f895f5d,
10 rounds on Wikidata): the team threshold stays. No forced arm beats the shipped decoder at any
count.
- **Teams:** `mibz2-team2` and `mibz2-team3` are 1.22–3.04× the shipped wall time.
- **One worker per thread:** `mibz2-workers-w` is never faster. Its interval spans 1 at 4, 5, 7
  and 9 threads, and elsewhere it is 1.045–1.136×.
- **At 6 threads:** the shipped decoder, now the lazy kernel at 5 workers (#642), is 1.135
  [1.118, 1.152] faster than one worker per thread. This is independent confirmation of the §5c
  trim.

**Caveats:**
- **The first registered attempt was NOT ADOPTED, and that verdict stands**
  (open-port/run-20261119T125538Z). Four guard point estimates exceeded 1.02, all on corpora
  that route no block to the new kernel, under a rule whose false-fail rate was about 0.97.
- **0.5.7 rests on a second, separately registered evaluation** with fresh data. It used:
  - a deterministic cachegrind equivalence check on those corpora;
  - a pooled wall bound;
  - G per cell, Holm-adjusted, wherever the kernel runs.
- **Its estimated false NOT-ADOPTED rate** was 0.085–0.12 under the planning models, and 0.26
  under the pessimistic model.
- **All figures are from one i9.** The trims apply only to that measured machine.
- **The new policy** is PERFORMANCE-POLICY §4a (#637).

### <!-- 3 -->Performance

- *(parallel)* the lazy kernel at 6 threads on the measured i9, pinned and unpinned ([#642](https://github.com/lmmx/mibz2/pull/642))
- *(open)* the open kernel as the workers' kernel on LazyCells' admitted blocks ([#630](https://github.com/lmmx/mibz2/pull/630))

### <!-- 4 -->Documentation

- *(changelog)* 0.5.6's crossover: the team threshold stays ([#596](https://github.com/lmmx/mibz2/pull/596))
- *(protocol)* a ready signal is the tag on origin, checked ([#594](https://github.com/lmmx/mibz2/pull/594))
- *(changelog)* 0.5.6's notes ([#593](https://github.com/lmmx/mibz2/pull/593))

### <!-- 9 -->Other

- *(moonshot)* a self-checking scorer for candidate kernels against 0.5.6 ([#606](https://github.com/lmmx/mibz2/pull/606))
- stop and walk, the i9 verdict (not adopted) fills the result placeholders ([#499](https://github.com/lmmx/mibz2/pull/499))
- Release scaling: 0.5.6 against 0.5.5 on the i9; its figures in 0.5.6's notes ([#595](https://github.com/lmmx/mibz2/pull/595))

- **0.5.7's worker-count recalibration (RELEASING §5c): one change, at 6 threads on the measured
  i9.** The open kernel (#630) changes the lazy workers' per-block work and scratch, so every
  trim was measured again on master at 17405826 (`just bench-recalibrate-057`, the reviewed
  2026-11-18 design re-pinned; recalibrate-057/run-20261119T185157Z, evidence f6c32d79, sha256
  98200b84…3805). At exactly 6 threads the default decoder now inverts with the lazy kernel
  (`OpenCells`) in place of eager `Cells`, with the shipped worker count and window:
  - pinned one CPU a core: lazy 5/20 over eager 5/20, 0.9340 [0.9264, 0.9417] of the wall on
    Wikidata ×4, at 0.844 of the peak memory (guards: text 0.9972, random 0.9989);
  - unpinned: the plan's six workers, lazy 6/24 over eager 6/24, 0.8531 [0.8024, 0.9071], at
    0.869 of the memory (guards inconclusive: text 1.0846 [0.8799, 1.3370], random 0.9835).

  Every other trim is unchanged: pinned at 4, 8 and 10 threads, and unpinned at 4, 8, 10, 12,
  16 and 20. So are the plan, `LAZY_FROM_THREADS` and every other machine. The output is
  byte-identical.

## [0.5.6](https://github.com/lmmx/mibz2/compare/v0.5.5...v0.5.6) - 2026-11-18

A kernel release: Cells' writer reads its rules once. Every plan, thread count and worker
trim is unchanged.

### <!-- 3 -->Performance

- *(cells)* write each full-period block in one scan of its rules (study 5, #579) ([#581](https://github.com/lmmx/mibz2/pull/581))

### <!-- 4 -->Documentation

- *(rules)* don't wait on a CI job whose inputs did not change ([#584](https://github.com/lmmx/mibz2/pull/584))
- *(blog)* What decoding pays for, review round 1 corrections ([#580](https://github.com/lmmx/mibz2/pull/580))
- *(blog)* What decoding pays for: how long run boundaries persist ([#574](https://github.com/lmmx/mibz2/pull/574))
- *(blog)* Cutting the loop open: decoding a block along shared paths ([#566](https://github.com/lmmx/mibz2/pull/566))

### <!-- 9 -->Other

- where to read the team's results and each role's status ([#572](https://github.com/lmmx/mibz2/pull/572))
- GitHub's macOS runners never hold a merge ([#571](https://github.com/lmmx/mibz2/pull/571))
- a new blog post always goes through a PR; direct commits only for amendments ([#567](https://github.com/lmmx/mibz2/pull/567))
- Release scaling: 0.5.5 against 0.5.4 on the i9; 0.5.5's notes and attribution row ([#563](https://github.com/lmmx/mibz2/pull/563))

- **Cells writes each full-period block in one scan of its rules** (study 5, #579; #581). Eager
  `Cells` and `LazyCells` share the writer. The shipped writer found each long rule's leftmost
  output occurrence, then read the rules again to expand and copy them. The one-scan writer
  instead places each rule child at its *owner*, the parent at its last use. Two consecutive
  stored rules certify the owner, so the induction is unchanged. Each surviving child becomes
  a copy command, stored over the rule records already read and run afterwards in increasing
  parent order. A block whose cycle is shorter than the block is written by the shipped
  writer as before. The output is byte-identical.
  - **The whole decoder on Wikidata, CI's paired bench** (10 rounds; wall time, head over
    base, 95% interval):

    | run | CPU | 1 thread | 2 threads | 4 threads |
    |---|---|--:|--:|--:|
    | #581 at cd7e5de2 | Xeon 6975P-C | −4.8% (−7.8, −1.7) | −3.5% (−5.0, −1.9) | −1.3% (−2.7, +0.2) |
    | #581 at 270a29ba | EPYC 7763 | −3.9% (−5.5, −2.4) | −3.9% (−6.2, −1.5) | −1.6% (−2.9, −0.4) |
    | 0.5.6 against 0.5.5 | Xeon Platinum 8370C | −1.7% (−4.0, +0.6) | −2.4% (−4.2, −0.5) | −1.3% (−2.0, −0.6) |

    CPU time fell 2.0–2.6% at 4 threads in all three runs. No other corpus is slower beyond
    noise in any run, and peak RSS shows no consistent change. The gain at the i9's thread
    counts (7–10) is under evaluation: the release scaling, below, measures it.
  - **The writer alone,** on identical grammars (`examples/writer_owned.rs`, research feature
    `owned-writer`; `docs/journal/2026-11-18-one-scan-writer.md`): 0.81 of the shipped
    writer's time on Wikidata eager and 0.79 lazy, the frozen block 0.80, and other corpora
    under forced Cells 0.55 to 0.72. Its data reads fall to 0.62 and its branch
    mispredictions to 0.70 (simulated). Its D1 write misses rise to 1.19, from the scattered
    order of its stores and copies.
  - **Its checks:** two independent verifications of the mathematics (the Researcher's REVIEW
    6986491194; MONODROMY's record 67). The Guardian's review lifted its evidence HOLD
    (D113, D114) on a completed Miri run and runtime mutation receipts, 8 of 8 killed.
  - **An alternative not taken:** study 6's first-occurrence writer (#582) took 1.79× the
    one-scan writer's time on identical grammars (`docs/journal/2026-11-18-first-occurrence-writer.md`).
- **The worker trims are unchanged, without a recalibration** (RELEASING §3.4). The writer
  cuts per-block CPU by about 2–4%, with no change in peak memory or DRAM bytes per output
  byte. The 10-thread trim is held by its memory cost, not its speed. The release scaling and
  the crossover check it (§5a, §5c).
- **The release scaling** (0.5.6 against 0.5.5 on the i9-10900K, `bench-release-scaling`,
  release-scaling/run-20261118T201717Z, 10 rounds; 0.5.5's wall over 0.5.6's on Wikidata, 95%
  interval; above 1 means 0.5.6 is faster):

  | threads | 1 | 2 | 4 | 8 | 10 | 12 | 16 | 20 |
  |---|--:|--:|--:|--:|--:|--:|--:|--:|
  | wall | 1.012 [0.976, 1.050] | 1.017 [1.000, 1.034] | 1.021 [0.994, 1.048] | **1.027** [1.015, 1.040] | 1.001 [0.957, 1.046] | 1.012 [0.989, 1.035] | **1.018** [1.002, 1.035] | **1.019** [1.001, 1.038] |

  0.5.6 is about 2–3% faster at 8, 16 and 20 threads, and its CPU time is 1–3% lower from 2
  threads up. At 10 threads wall time is unchanged within a wide interval, and CPU time is
  1.2% lower [1.000, 1.024]. No step reaches the attribution ledger's 3% bar (§5b).
- **The crossover** (§5a; `bench-crossover`, crossover/run-20261118T202811Z, evidence 6d06a0dd,
  10 rounds on Wikidata): the team threshold stays. No forced arm is faster than the shipped
  decoder at any count from 4 to 20 threads: no interval lies wholly below 1. The teams of 2
  and 3 are 1.21–2.79× its wall time. One block a thread with the default window
  (`mibz2-workers-w`) comes nearest at 5 threads, 0.987 [0.971, 1.004], and at 9, 0.972
  [0.933, 1.012]; it is 1.07–1.19× from 10 threads up, where the trims run 7 lazy workers.

## [0.5.5](https://github.com/lmmx/mibz2/compare/v0.5.4...v0.5.5) - 2026-11-18

A kernel release: the lazy kernel admits more blocks, so fewer of them take the T-vector walk.
Every plan and thread count is unchanged.

- **Wider admission for the lazy kernel** (#559). `LazyCells`, the workers' kernel from
  `LAZY_FROM_THREADS` threads and on the trimmed i9 plans, admits blocks up to r/n 0.15 (3 run
  records in 20 rows), not 0.10, with a budget of 0.32 n cells, not 0.16 n; the sparse gate is
  Cells' own. The blocks between the lines leave the T-vector walk, whose random reads carried
  about a third of the decoder's modelled DRAM traffic on Wikidata. Eager `Cells` (below
  `LAZY_FROM_THREADS`), the teams and the service core keep Cells' admission. The output is
  byte-identical.
  - On the measured i9 (`bench-admit`, admit/run-20261118T100717Z, #145 LOG 6975512468),
    unpinned, on Wikidata ×4: 0.838 [0.825, 0.851] of master's wall at 8 threads (eight lazy
    workers) and 0.878 [0.866, 0.890] at 10 (seven). DRAM fell from 10.70 to 8.05 B a byte of
    output at seven workers (reads 6.76 to 4.73, writes 3.93 to 3.33). Text and random keep
    their route and their wall (0.993 to 1.007).
  - Peak memory on Wikidata rose 25 to 26 MB (1.14 to 1.16 of master's). The frozen rule held
    it on memory, and it was accepted under PERFORMANCE-POLICY rule 4, as amended by #558, for
    12 to 16% less wall and 25% fewer DRAM bytes a byte. Text and random are unchanged.
- **The release check (RELEASING §3)**, run 44290437026 on an AMD EPYC 9V74 (#145 LOG
  6976975591): one primary row flagged, Wikidata at 1 thread, +1.6% (+0.0, +3.1), at the edge
  of noise; #559's only footprint at 1 thread is one branch a block. The release scaling
  doesn't reproduce it (1.007, spanning 1).
- **Release scaling, 0.5.5 against 0.5.4 on the i9** (release-scaling/run-20261118T112057Z,
  #145 LOG 6977328773): 0.5.4 takes 1.054 [1.031, 1.078] of 0.5.5's wall time at 8 threads,
  1.135 [1.119, 1.151] at 10 and 1.128, 1.132 and 1.121 at 12, 16 and 20, with 1.07–1.14 of
  its CPU. Peak RSS at 8–20 threads is about level: 118–135 MB for 0.5.4, 126–131 MB for 0.5.5
  (+8 MB at 8 threads, on this single copy of Wikidata). At 1–4 threads the two
  are level (0.997–1.011, every interval spanning 1). At 10 threads 0.5.5's median is
  0.089 s, the fastest point a release sweep has recorded.
- **Team threshold (RELEASING §5a), unchanged** (crossover/run-20261118T113151Z, #145 LOG
  6977656986). No team arm beat the shipped plan at any count from 4 to 20. At 8 threads,
  pinned, one block a thread over the shipped 6/24 trim is 0.983 [0.955, 1.011], where 0.5.4's
  run read 1.050: under wide admission the trim's gain there is gone. That is left to the
  worker-count recalibration (RELEASING §5c). At 10–20 threads the shipped plan still beats one
  block a thread by 5–18%.

### <!-- 3 -->Performance

- *(cells)* wider admission for the lazy kernel, to r/n 0.15 with a budget of 0.32 n ([#559](https://github.com/lmmx/mibz2/pull/559))

### <!-- 4 -->Documentation

- *(blog)* Raising the knee, at /blog/2026-11-17-raising-the-knee/
- *(blog)* retitle the knee post "Straightening the knee: scaling throughput beyond physical cores", with a new dek
- *(blog)* retitle the knee post "Straightening the knee: gaining speed with more threads and hyperthreading"

### <!-- 9 -->Other

- the knee is gone, more threads without the fall ([#550](https://github.com/lmmx/mibz2/pull/550))
- Blog (worker count): round-2 revision, a shorter post and technical notes ([#538](https://github.com/lmmx/mibz2/pull/538))
- Release scaling: 0.5.4 against 0.5.3 on the i9; 0.5.4's notes and attribution row ([#548](https://github.com/lmmx/mibz2/pull/548))

## [0.5.4](https://github.com/lmmx/mibz2/compare/v0.5.3...v0.5.4) - 2026-11-17

A plan release: on the measured i9, the trimmed workers now fire by how the process is placed.
Pinned one CPU a core, the decoder runs fewer workers than threads at 4 to 10 threads; on the
whole machine, from 10 threads, as in 0.5.3. Every other machine and mask is unchanged.

- **The trimmed workers by mode** (#537). **Pinned** (one CPU on each of exactly the threads
  asked, no two on a core, as every sweep measured): lazy 7/28 at 10 threads (0.881 to 0.889
  of lazy 10/40), lazy 6/24 at 8 (0.925 of lazy 8/32), eager 5/20 at 6 (0.938 of eager 6/24)
  and eager 3/12 at 4 (0.959 of eager 4/16). The 10-thread cell now fires pinned to CPUs 0–9,
  where 0.5.3 declined it. **Unpinned** (the whole machine): lazy 7/28 from 10 threads, as in
  0.5.3, now measured unpinned at 0.873 [0.866, 0.880] at 10, and nothing below 10. The
  low-count gains are a pinned scheduling effect (a worker fewer frees a core for the front
  end): unpinned, the 8-, 6- and 4-thread trims took 1.00, 1.01 and 1.21 of master's wall.
  **Anything else** (another mask, such as two CPUs of a core or more cores than threads; any
  cpu.max quota; another machine) is unchanged. The machine is recognised from every CPU
  present and online, not from this process's affinity; the service core's predicate still
  reads the affinity. Verdicts: #145 LOGs 6945639925, 6947274621 and 6949008682, admitted by
  the Researcher's REVIEWs 6946555118, 6947387421 and 6948628221.
- **The machine read once a process** (#545). The machine-wide facts (the L3, cpuinfo's CPU
  identities and the topology of every CPU present and online) are cached on first use. The
  affinity and the cgroup quota are read only where those facts admit the measured i9. Every
  plan and cell is unchanged.
- **The release check (RELEASING §3).** Its first run, 44095874788 on an Intel Xeon 8370C,
  caught #537's per-decoder sysfs and cpuinfo reads at 4 to 8 threads (0.5 to 1 ms a default
  decoder on a 4-CPU runner): 4-thread small inputs were 3.8–5.8% slower. #545 fixed it, and
  the re-check, 44108684132 on an AMD EPYC 9V45, had no slower row (#145 LOG 6952096740).
- **Release scaling, 0.5.4 against 0.5.3 on the i9** (release-scaling/run-20261117T151421Z,
  #145 LOG 6952656453). The harness pins runs of 10 threads or fewer one CPU a core, and the
  thread names show the pinned cells: 3 workers at 4 threads, 6 at 8 and 7 at 10. 0.5.3 takes
  1.123 [1.094, 1.152] of 0.5.4's wall time at 10 threads and 1.045 [1.038, 1.053] at 8, with
  1.241 and 1.132 of its CPU and a peak RSS of 184 and 154 MB against 0.5.4's 134 and 121 MB.
  At 4 threads 0.5.3 takes 1.074 of the CPU, and its 1.020 [0.994, 1.048] of the wall is not
  resolved. At 1, 2 and 12–20 threads the two are level (0.980–0.996, every interval
  spanning 1).
- **Team threshold (RELEASING §5a), unchanged** (crossover/run-20261117T152519Z, #145 LOG
  6952857309). No team arm beat the shipped plan at any count from 4 to 20. One block a thread
  is 2–11% slower than the trimmed plan at 4, 6, 8 and 10 threads. In the sanity check at 5, 7
  and 9 threads, where the two arms build the same plan, 5 and 9 span 1 and 7 is a near miss
  at 0.979 [0.969, 0.990]. #537 doesn't change the plan at 7, and the same cell was 1.000 at
  0.5.3, so it is noted, not acted on.

### <!-- 3 -->Performance

- *(plan)* read the machine once a process ([#545](https://github.com/lmmx/mibz2/pull/545))
- *(plan)* on the measured i9, six lazy workers at 8 threads asked and five at 6 ([#537](https://github.com/lmmx/mibz2/pull/537))

### <!-- 9 -->Other

- Release scaling: 0.5.3 against 0.5.2 on the i9; 0.5.3's notes and attribution row ([#544](https://github.com/lmmx/mibz2/pull/544))

## [0.5.3](https://github.com/lmmx/mibz2/compare/v0.5.2...v0.5.3) - 2026-11-17

A plan release: on the measured i9, from 10 threads asked, the decoder runs seven lazy workers
with 28 blocks in flight. Every other machine and thread count is unchanged.

- **Seven lazy workers from 10 threads asked** (#524). It applies only on the measured profile
  (the i9-10900K: 10 cores, 20 CPUs, a 20 MiB L3, no cpu.max limit), in place of 9 workers
  with 36 blocks past the cores and 10 with 40 at exactly 10. On Wikidata ×4, its held-out
  confirmation run (workers-confirm/run-20261117T092105Z) had wall time 0.921, 0.930 and 0.923
  of 0.5.2's plan at 12, 16 and 20 threads and 0.881 at 10, with peak memory 0.73–0.81. The
  text and records guards showed no loss.
- **Release scaling, 0.5.3 against 0.5.2 on the i9** (release-scaling/run-20261117T111525Z):
  0.5.2 takes 1.069 [1.044, 1.095], 1.067 [1.028, 1.107] and 1.075 [1.046, 1.104] of 0.5.3's
  wall time at 12, 16 and 20 threads, with 29–33% more CPU and a peak RSS of 166–170 MB
  against 0.5.3's 129–133 MB. At 1–8 threads the two are level (0.988–1.002, every interval
  spanning 1).
- **That run's 10-thread row doesn't measure the trim.** It pinned the 10-thread cells to
  CPUs 0–9, and 0.5.3 recognised the machine through the process's affinity, so pinned it
  declined the trim and built 10 workers (0.5.2 over 0.5.3: 0.984 [0.957, 1.011]). Unpinned,
  the 10-thread trim was later measured at 0.873 [0.866, 0.880] of 10 workers
  (unpinned/run-20261117T130353Z, #145 LOG 6949008682). The pinned case is fixed under
  [Unreleased] (#537).
- **Team threshold (RELEASING §5a), unchanged** (crossover/run-20261117T112620Z, #145 LOG
  6947191226). Neither teams nor one block a thread beat the shipped plan at any count from 4
  to 20. Teams are slower at every count; one block a thread is level at 4–10 threads and
  20–43% slower at 12–20.

### <!-- 3 -->Performance

- *(plan)* on the measured i9, seven lazy workers from 10 threads asked, with the window sized to them ([#524](https://github.com/lmmx/mibz2/pull/524))
- *(cells)* a 12-byte rules arm (W12LazyCells), the lazy induction storing each rule's children packed ([#506](https://github.com/lmmx/mibz2/pull/506))

### <!-- 4 -->Documentation

- queue-only PRs and already-tested content merge without waiting for CI ([#518](https://github.com/lmmx/mibz2/pull/518))

### <!-- 9 -->Other

- modelling the cache to rightsize workers ([#527](https://github.com/lmmx/mibz2/pull/527))
- Release scaling: 0.5.2 against 0.5.1 on the i9; 0.5.2's notes and attribution row ([#511](https://github.com/lmmx/mibz2/pull/511))

## [0.5.2](https://github.com/lmmx/mibz2/compare/v0.5.1...v0.5.2) - 2026-11-16

A plan release: past the physical cores, the decoder runs one lazy worker fewer than the
cores, and sizes its in-flight window by the workers. That leaves one core for the scanner.
Below the core count nothing changes.

- **One lazy worker a core but one, past the cores** (#481). It applies only on the measured
  profile (10 cores, as on the i9-10900K). On Wikidata ×4, its registered run
  (cores-less-one/run-20261116T203759Z) had wall time 0.956, 0.960 and 0.959 of master's at
  12, 16 and 20 threads, CPU 0.85–0.89 and peak memory 0.82–0.88. The text guard and the
  9- and 10-thread controls showed no change.
- **Release scaling, 0.5.2 against 0.5.1 on the i9** (release-scaling/run-20261116T221259Z):
  0.5.1 takes 1.047, 1.047 and 1.050 of 0.5.2's wall time at 12, 16 and 20 threads, and
  13–16% more CPU. At 1–10 threads the two are level (0.984–1.008, every interval spanning 1).
- **Team threshold (RELEASING §5a), unchanged** (crossover/run-20261116T215235Z). No forced
  arm beat the shipped decoder at any count it would change. Teams are slower at every count
  from 4 to 20.
- **The toolchain is pinned** to rustc 1.99.1 (#504) for CI, the bench, the wheels and
  workstation runs. The published crate doesn't carry the pin, and the MSRV stays 1.80.

### <!-- 1 -->Features

- *(research)* the lazy kernel's induction, recorded block by block (induction-trace) ([#486](https://github.com/lmmx/mibz2/pull/486))

### <!-- 3 -->Performance

- *(plan)* past the cores, one lazy worker a core but one, with the window sized to the workers ([#481](https://github.com/lmmx/mibz2/pull/481))

### <!-- 9 -->Other

- Grammar replay: W10 writer, and W12 as #506's layout, research only ([#508](https://github.com/lmmx/mibz2/pull/508))
- Grammar replay harness: W16/W12/W8 writers on the same rules, research feature only ([#503](https://github.com/lmmx/mibz2/pull/503))
- Pin the Rust toolchain to 1.99.1 for CI, bench, wheels and workstation runs ([#504](https://github.com/lmmx/mibz2/pull/504))
- Towers, words and loops, a mathematical dictionary for BWT inversion ([#502](https://github.com/lmmx/mibz2/pull/502))
- stop and walk, cutting a text rebuild short and walking the rest (result pending #493) ([#495](https://github.com/lmmx/mibz2/pull/495))
- wake Claude Code roles with a direct session message, never through Louis (§1a) ([#484](https://github.com/lmmx/mibz2/pull/484))
- Release scaling: 0.5.1 against 0.5.0 on the i9 ([#475](https://github.com/lmmx/mibz2/pull/475))
- 0.5.0's and 0.5.1's notes ([#474](https://github.com/lmmx/mibz2/pull/474))

## [0.5.1](https://github.com/lmmx/mibz2/compare/v0.5.0...v0.5.1) - 2026-11-16

A plan release: the decoder now uses one block a thread (workers, with lazy letters) instead of
teams at 10 threads, and past the physical cores on machines with 8 to 10 of them. The kernels,
the output and the speed below 10 threads are unchanged.

- **Workers at 10 threads** (#462). `WORKERS_BELOW_THREADS` goes from 10 to 11. On the
  i9-10900K, lazy workers took 0.931× and then 0.917× [0.898, 0.936] the teams' wall at 10
  threads in two lazy-crossover runs (#145 LOGs 6905312941, 6905778670). The service core
  stands down whenever the plan is workers.
- **One lazy worker a core past the cores** (#468). With more threads than physical cores, on a
  machine with 8 to 10 of them, the plan is `Workers(physical cores)`: the hyperthreads stay
  idle, because two workers on one core share its L1 and L2. On the i9 against 0.5.0's plan
  (teams): 0.926 / 0.934 / 0.945 / 0.950 at 12 / 14 / 16 / 20 threads (#465, LOG 6906090872),
  and 0.942 (wikidata) and 0.935 (the 8-fold file) at 20 (occupancy run, LOG 6906620039).
  Uncapped lazy workers took 1.21–1.22× there.
- **So on the i9,** one block a thread now runs at every thread count from 6 to 20: eager Cells
  at 6–7, lazy letters from 8. Teams remain the plan below 6 threads and from 11 physical cores.
- **The paired runbook bench, CI** (4 CPUs, Xeon Platinum 8370C; run 43836782266, v0.5.0 →
  master 3dff14b): level, as expected for changes that only act at 10 threads or more.
  Two of 21 cells' intervals exclude zero either way (`onebyte` at 4 threads +3.5%, `records`
  at 4 threads −3.3%), within the spread of a 21-cell table.
- **Caveats:** one machine. Eleven threads was not measured directly (it gets the same plan as
  12). Machines with 11 or more physical cores keep the teams, untested.
- **Release scaling, the i9-10900K** (`release-scaling/run-20261116T092431Z`, #475): 0.5.0's
  wall over 0.5.1's is level from 1 to 8 threads (0.991–1.006), then 1.079 [1.068, 1.090] at
  10, 1.066 at 12 and 1.084 at 16 and 20: 0.5.1 is 6–8% faster from 10 threads up. Its cost:
  CPU 0.906 at 10 and 0.763–0.776 at 12–20 (10% to 31% more CPU), and peak RSS 72 → 187–193 MB
  at 10–20 threads. Wall is fastest at 8 threads (0.109 s) on this file.
- **The team threshold** (RELEASING.md §5a; `lazy-crossover/run-20261116T093528Z`, #145 LOG
  6913175299): sanity passes at every count, and no forced arm beats the shipped plan anywhere,
  so nothing moves. Against forced three teams, shipped takes 0.902× the wall at 10 threads and
  0.638× at 20. The memory dose is 2.580 eager and 1.985 lazy.
- **Research features reviewed** (RELEASING §3.3): `memory-lever`'s run is read and its lazy arm
  promoted, so it is to go in its own PR; `trace`, `wait-stats`, `walk-stats`, `dose` and
  `phase-counters` stay.

### <!-- 9 -->Other

- Past the physical cores, on 8 to 10 of them: lazy workers, one a core ([#468](https://github.com/lmmx/mibz2/pull/468))
- a deliberate repeat run is fine; a near-miss sanity check is weighed, not a veto ([#463](https://github.com/lmmx/mibz2/pull/463))
- Workers with lazy letters at 10 threads; teams only past the physical cores ([#462](https://github.com/lmmx/mibz2/pull/462))

## [0.5.0](https://github.com/lmmx/mibz2/compare/v0.4.0...v0.5.0) - 2026-11-16

**Lazy letters** (#452): from 8 threads, Cells' induction keeps a slot for a letter only while
the letter is live, so the pool peaks at about 0.32 of the letters (right-end order) instead of
all of them. Each worker touches less memory: on the i9, CPU at 10 threads over CPU at 1 is
1.92 for lazy against 2.58 for eager (the memory dose, LOG 6905312941). Lazy against eager is
0.965 at 8 threads, so `LAZY_FROM_THREADS` is 8. Release scaling on the i9 against 0.4.0: about
4% faster at 8 threads, level elsewhere. The post: "Lazy letters: just-in-time memory for
induction" (#451).

### <!-- 9 -->Other

- Workers invert with lazy letters from 8 threads ([#452](https://github.com/lmmx/mibz2/pull/452))
- Lazy letters: just-in-time memory for induction ([#451](https://github.com/lmmx/mibz2/pull/451))
- memory-lever experiment — lazy letters and rules past the cache as opt-in arms ([#447](https://github.com/lmmx/mibz2/pull/447))
- the free closure of an induction: where free steps come to rest ([#442](https://github.com/lmmx/mibz2/pull/442))
- 0.4.0's notes ([#436](https://github.com/lmmx/mibz2/pull/436))
- the width of an induction: how much it must hold at once ([#439](https://github.com/lmmx/mibz2/pull/439))

## [0.4.0](https://github.com/lmmx/mibz2/compare/v0.3.4...v0.4.0) - 2026-11-15

A soundness and memory release.
- **The breaking change:** `Kernels` can no longer claim a CPU feature from safe code (#431,
  below).
- **Less memory per block:** Cells holds 22% less memory per block than 0.3.4 (#433), with
  output, plan and speed unchanged.
- **Research features trimmed:** #432 removes `slot24` and trims `phase-counters`.

- **Cells memory** (#433). Two pieces of Cells' short-lived state now live where memory is dead
  while they are live:
  - the 8-byte bottom slots in the block's output buffer, during the induction;
  - the writer's position table in the top slots, after the induction.

  Per block, the memory Cells touches besides the output falls from 2.07 / 2.39 / 3.37 MiB to
  1.60 / 1.86 / 2.61 MiB (min / median / max over the 196 Wikidata blocks), 0.776× on every
  block. CI's bench (run 43747945487) shows `wikidata` at 1 thread 0.5% faster (−0.8%,
  −0.2%), and peak RSS 5.1% lower at 2 threads and 7.5% lower at 4.
  - **The contract change:** a Cells budget refusal now leaves the output buffer holding
    scratch, not untouched. Every fallback clears it before writing.
- **Release scaling, the i9-10900K:** the run recorded under 0.3.4 below
  (`release-scaling/run-20261115T193005Z`) built its head at master `06aa5858`, which is 0.4.0's
  code. Against 0.3.3 it is 7.2% faster at 8 threads and level elsewhere.
- **The team threshold** (RELEASING.md §5a; `crossover/run-20261115T191448Z`, #145 LOG
  6900029992) stays at 10.
  - At 10 threads, workers now take 1.040× the teams' wall (1.019–1.061), against 1.153×
    before #433.
- **The paired runbook bench, CI:** v0.3.4 against v0.4.0 (run 43754741304), to follow.

### <!-- 9 -->Other

- Release scaling: 0.3.4 against 0.3.3 on the i9 ([#434](https://github.com/lmmx/mibz2/pull/434))
- bottom slots in the dead output buffer, rule positions in the dead top slots ([#433](https://github.com/lmmx/mibz2/pull/433))
- Research features: remove slot24, trim phase-counters, retention notes ([#432](https://github.com/lmmx/mibz2/pull/432))
- private fields, so safe code can't claim a CPU feature ([#340](https://github.com/lmmx/mibz2/pull/340)) ([#431](https://github.com/lmmx/mibz2/pull/431))
- Removing duplicated interval lengths and text references ([#427](https://github.com/lmmx/mibz2/pull/427))
- 0.3.4's notes; queue: bench-release-scaling for 0.3.4 ([#429](https://github.com/lmmx/mibz2/pull/429))

### Breaking

- **`Kernels` can no longer claim a feature the CPU lacks from safe code** (#340). Its fields
  `avx2` and `clmul` were public, so safe code could build `Kernels { avx2: true, .. }` on a
  CPU without AVX2 and reach `#[target_feature]` code through the public decoders: undefined
  behaviour. The fields are now private. `Kernels::detect()` and `Kernels::scalar()` are
  unchanged; new are `restrict(avx2, clmul)` (safe: it only turns kernels off),
  `unsafe new_unchecked(avx2, clmul)` (the caller vouches for the CPU) and the read-only
  accessors `avx2()` and `clmul()`. Migrate `k.avx2` → `k.avx2()`, `k.clmul` → `k.clmul()`,
  and a struct literal → `Kernels::detect().restrict(..)` or `unsafe { Kernels::new_unchecked(..) }`.
  Breaking while the crate is 0.x, so the next release is 0.4.0. The decoder's own paths,
  and its speed, are unchanged.

## [0.3.4](https://github.com/lmmx/mibz2/compare/v0.3.3...v0.3.4) - 2026-11-15

A plan release: the **team threshold** moves (#425, docs/RELEASING.md §5a, #426). The default
decoder now gives each thread a block of its own (workers, Cells inverting) whenever fewer
than 10 threads each have a physical core of their own (`WORKERS_BELOW_THREADS = 10` in
`src/parallel.rs`). On the i9-10900K that is 6 to 9 threads, which ran teams of 2 until now.
From 10 threads the teams stay (the i9's three teams of three), and so do more threads than
physical cores: on the i9, one block a thread on SMT siblings was 28–49% slower than the teams
(12–20 threads). Below 6 cores nothing changes. The output, the API and the supported inputs
are unchanged.

- **The crossover, the i9-10900K** (`bench-crossover` run 2, `crossover/run-20261115T151452Z`
  on the `evidence` branch at c7db45c2, sha256 15a2b7b7…51e5, master 1c961aa2, 10 paired
  rounds, every arm the same bytes; judged under the rule posted before the data, #145 LOG
  6896455773, result LOG 6896798131), `wikidata`, workers ÷ teams:

  | threads | 6 | 7 | 8 | 9 | 10 |
  |---|--:|--:|--:|--:|--:|
  | wall | **0.861** | **0.889** | **0.923** | **0.964** | **1.153** |
  | 95% interval | 0.852–0.871 | 0.871–0.908 | 0.915–0.930 | 0.956–0.972 | 1.131–1.174 |
  | CPU | **0.884** | 0.997 | **0.967** | **1.124** | **1.341** |
  | peak RSS, MB, teams → workers | 67 → 133 | 65 → 149 | 81 → 168 | 80 → 188 | 73 → 205 |

  That is 13.9%, 11.1%, 7.7% and 3.6% less wall time at 6, 7, 8 and 9 threads, beyond noise,
  and at 10 threads 15.3% more: the teams keep 10. CPU is 11.6% and 3.3% less at 6 and 8
  threads, level at 7, and 12.4% more at 9 (1.112–1.135). The sanity check passed: at 4 and 5
  threads, where both arms run the same decoder, 0.993 and 0.979, each interval spanning 1.
  Run 1 of the same recipe was invalid: its forced arm kept a smaller window than the default
  decoder's and failed that sanity check (fixed by #424).
- **The decision.** The 9-thread CPU cost was over the rule's 10%, so the move went to Louis,
  who chose teams from 10. Peak RSS roughly doubles at 6–9 threads, under 200 MB, weighed for
  a 16 GB laptop as acceptable.
- **The PR's own CI bench** (run 43732867824, AMD EPYC 9V45, 4 vCPU on 2 cores, 10 paired
  rounds), #425's head against its base: CI's runners have 2 physical cores, so both arms run
  the same plan and the bench cannot see the change. It was flat: `wikidata` wall +0.0%
  (−7.7, +8.5), −0.1% (−2.3, +2.3) and −2.5% (−7.6, +2.8) at 1, 2 and 4 threads, CPU −0.0%,
  +0.4% and −2.8%, each interval including no change. The assessment: no change detected.
- **The release's CI bench** (run 43737282330, Intel Xeon Platinum 8573C, 4 vCPU on 2 cores,
  10 paired rounds), v0.3.3 (1244d1a9) → master e18ccafc, the v0.3.4 tag commit. It finished
  after the release, so it is a check after the fact:

  | `wikidata` | 1 thread | 2 threads | 4 threads |
  |---|--:|--:|--:|
  | wall | −0.3% (−1.9, +1.4) | −1.3% (−4.7, +2.2) | +0.3% (−1.8, +2.4) |
  | CPU | −0.3% (−1.9, +1.3) | −1.5% (−4.0, +1.1) | −0.0% (−1.8, +1.8) |
  | peak RSS, MB | 28 → 28 (−0.0%) | 59 → 58 (−1.1%) | 90 → 93 (+0.5%) |

  Level, as expected on a 2-core runner, which runs the same plan in both builds: every
  interval includes no change. Best wall against best wall +0.3% (−1.8, +2.4). Every corpus
  gave the same bytes in every run. The assessment: no change detected.
- **Release scaling, the i9-10900K** (`release-scaling/run-20261115T193005Z`, sha256 f07387d2,
  10 paired rounds, every arm the same bytes), `wikidata`, 0.3.3 ÷ 0.3.4 wall:

  | threads | 1 | 2 | 4 | 8 | 10 | 12 | 16 | 20 |
  |---|--:|--:|--:|--:|--:|--:|--:|--:|
  | 0.3.3 ÷ 0.3.4 | 1.014 | 1.016 | 1.013 | **1.078** | 0.993 | 1.010 | 1.004 | 0.993 |
  | 95% interval | 0.980–1.048 | 0.986–1.048 | 0.993–1.034 | 1.057–1.100 | 0.976–1.011 | 0.986–1.035 | 0.980–1.028 | 0.964–1.023 |
  | peak RSS, MB | 28 → 28 | 58 → 58 | 99 → 93 | 80 → 173 | 73 → 72 | 71 → 72 | 71 → 72 | 72 → 72 |

  At 8 threads, where the plan changed from teams to workers, 0.3.4 takes 7.2% less wall time
  and 3.4% less CPU (1.035, 1.015–1.054), for about twice the peak RSS. Elsewhere it is level,
  as expected: the plan is unchanged at 1–4 threads and from 10.
  - The head was built at master `06aa5858`: 0.3.4 plus the then-unreleased #431–#433. #433's
    Cells memory change shows no wall change beyond noise at 1–4 threads here.
  - The same commit's `bench-crossover` (`crossover/run-20261115T191448Z`, #145 LOG
    6900029992) leaves the threshold at 10. Workers at 10 threads now take 1.040× the teams'
    wall, down from 1.153× before #433.
- **Caveats.**
  - One machine: the i9-10900K (10 cores, 20 threads) is the only evidence. Other 6–9-core
    machines take the same plan without having been measured.
  - Peak RSS roughly doubles at 6–9 threads (65–81 → 133–188 MB).

### <!-- 9 -->Other

- Parallel plan: workers below 10 threads on their own cores; teams from 10 ([#425](https://github.com/lmmx/mibz2/pull/425))
- Bench, tooling and runbook changes: the crossover recipe, queued ([#422](https://github.com/lmmx/mibz2/pull/422)), and its forced arm with the default window ([#424](https://github.com/lmmx/mibz2/pull/424)); the workstation waiter posts when a run starts and when one fails ([#423](https://github.com/lmmx/mibz2/pull/423)), and the workstation queue keeps its logs and markers in the XDG state directory ([#428](https://github.com/lmmx/mibz2/pull/428)); the release runbook re-checks the team threshold when a release could move it ([#426](https://github.com/lmmx/mibz2/pull/426)); 0.3.3's notes ([#421](https://github.com/lmmx/mibz2/pull/421))

## [0.3.3](https://github.com/lmmx/mibz2/compare/v0.3.2...v0.3.3) - 2026-11-15

A memory-traffic release for Cells. Bottom slots shrink from 20 to 8 bytes (#414): a letter's
length and word are now read through its top slot, and the bottom slot keeps only the link and
the partner. A letter's fields are read once, on the side the fronts reach first, and the side
reached second is overwritten unread (rule R4 in MONODROMY's elision bounds, #410), so the
bottom copy was never needed when the top front got there first. The output, the API, the
supported inputs and the plans are unchanged.

- **The census** (#414, the bench corpus `wikidata`, 431 blocks, 196 admitted by Cells, counts
  and no timing): slot bytes in both orders 443.6 → 310.5 MB (0.700), 0.63 B less per output
  byte; the induction's buffer lines (slots, arena words and rules used) 9.64 M → 7.56 M
  (0.784). The price is one dependent load, from the bottom slot to its top slot, on 53.7% of
  slot reads. The load is off the link chain, unlike #407's pointer chase.
- **The PR's own CI bench** (run 43719302600, AMD EPYC 9V45, 4 vCPU on 2 cores, 10 paired
  rounds), #414's head against its base: `wikidata` CPU −3.3% (−5.8, −0.8) at 4 threads, and
  amplification at 4 threads −3.2% (−5.6, −0.8); wall −0.1%, −0.2% and −1.8% at 1, 2 and 4
  threads, each interval including no change. The assessment: better (CPU cost, contention
  sensitivity), and no measure worse.
- **The release's CI bench** (run 43723600944, AMD EPYC 9V45, 4 vCPU on 2 cores, 10 paired
  rounds), v0.3.2 (42c2aa6e) → master 898535524, which differs from the v0.3.3 tag only in the
  version bump. It finished after the release, so it is a check after the fact:

  | `wikidata` | 1 thread | 2 threads | 4 threads |
  |---|--:|--:|--:|
  | wall | −1.4% (−5.0, +2.4) | −1.2% (−4.5, +2.1) | **−4.2% (−4.9, −3.4)** |
  | CPU | −1.4% (−5.0, +2.4) | −0.7% (−2.9, +1.5) | **−4.0% (−5.3, −2.8)** |
  | peak RSS, MB | 30 → 28 (−4.8%) | 65 → 62 (−4.0%) | 102 → 98 (−5.2%) |

  At 4 threads, 4.2% less wall time and 4.0% less CPU, beyond noise; at 1 and 2 threads each
  interval includes no change. Best wall against best wall −4.2% (−4.9, −3.4). Every corpus
  gave the same bytes in every run. The assessment: better (speed, CPU cost), and no measure
  worse.
- **Release scaling, the i9-10900K** (`release-scaling/run-20261115T143048Z`, sha256 86b3e01a,
  10 paired rounds, every arm the same bytes, #420), `wikidata`, 0.3.2 ÷ 0.3.3 wall:

  | threads | 1 | 2 | 4 | 8 | 10 | 12 | 16 | 20 |
  |---|--:|--:|--:|--:|--:|--:|--:|--:|
  | 0.3.2 ÷ 0.3.3 | **1.043** | **1.040** | **1.070** | 1.000 | 0.997 | 1.005 | 0.996 | 1.000 |
  | 95% interval | 1.013–1.074 | 1.017–1.064 | 1.050–1.090 | 0.990–1.009 | 0.989–1.004 | 0.965–1.046 | 0.977–1.015 | 0.970–1.030 |
  | peak RSS, MB | 29 → 28 | 64 → 58 | 106 → 97 | 80 → 80 | 73 → 73 | 72 → 73 | 72 → 72 | 72 → 71 |

  That is 4.1%, 3.9% and 6.5% less wall time at 1, 2 and 4 threads, beyond noise, and 7.3% less
  CPU at 4 (1.078, 1.067–1.090). From 8 threads it is level: the plan there is the teams path,
  which 0.3.3 does not change. That the teams path is level here also suggests that 0.3.2's
  slowdowns at 16 and 20 threads were drift within that run.
- **Caveats.**
  - On CI's 2-core runner the gain is beyond noise only at 4 threads, with co-running workers;
    at 1–2 threads neither CI run shows a change beyond noise. The i9 shows gains at 1 and 2
    threads too (4.1% and 3.9%), with intervals that reach down to 1.3% and 1.7%.
  - #414's bench had two watch items, neither beyond noise: `text` at 2 threads +5.7% wall
    (−2.0, +14.1) and `pattern` at 4 threads +3.9% (−4.3, +12.8). Neither recurs in the
    release's bench: `text` at 2 threads −0.3% (−7.7, +7.6), `pattern` at 4 threads
    +0.3% (−5.0, +5.9).
  - The Cells induction campaign stops here for now. The other candidate built alongside it,
    equal one-row cell elision (#418), was 8–12% slower in wall time and closed unmerged
    (`docs/journal/2026-11-15-equal-cell-elision.md`).

### <!-- 9 -->Other

- Cells induction: 8-byte bottom slots, a letter's length and word read through its top slot ([#414](https://github.com/lmmx/mibz2/pull/414))
- Queue, record and docs changes: 0.3.2's release-scaling queue entry and record ([#415](https://github.com/lmmx/mibz2/pull/415), [#417](https://github.com/lmmx/mibz2/pull/417)), 0.3.2's notes ([#416](https://github.com/lmmx/mibz2/pull/416)) and, after the tag, 0.3.3's release-scaling queue entry and record ([#419](https://github.com/lmmx/mibz2/pull/419), [#420](https://github.com/lmmx/mibz2/pull/420))

## [0.3.2](https://github.com/lmmx/mibz2/compare/v0.3.1...v0.3.2) - 2026-11-15

A memory release for Cells. Slot recycling (#408): when Cells' induction places a letter, it
takes over the dead slot at the front of its order instead of a fresh one, so every slot it
touches is one of the letters' own and each order's region is sized to exactly `letters × SLOT`.
The output, the API, the supported inputs and the plans are unchanged.

- **The census** (#408, the bench corpus `wikidata`, 431 blocks, 196 admitted by Cells): slots
  touched 31.5 M → 22.2 M (−29.7%). The lines the fresh slots used to touch first are never
  touched, and the reused slot was read a step earlier, so it is in cache.
- **The paired runbook bench, CI** (run 43717843002, Intel Xeon Platinum 8573C, 4 vCPU on 2
  cores, 10 paired rounds), v0.3.1 → master: `wikidata` peak RSS 33 → 30 MB, 75 → 62 MB and
  118 → 97 MB at 1, 2 and 4 threads (−9.6%, −14.6% and −14.5%); wall −0.2%, +0.5% and −0.4%,
  each interval including no change. Best wall against best wall −0.4% (−1.5, +0.7). Every
  corpus gave the same bytes in every run, and no corpus is slower in wall time. The
  assessment: better (memory), and no measure worse.
- **The i9-10900K** (`release-scaling/run-20261115T123513Z`, sha256 ea04423b, #408's head
  against its base, 10 paired rounds, every block CRC identical, #145 LOG 6894519106), base ÷
  head, so above 1 is faster:

  | threads | 1 | 2 | 3 | 4 | 5 | 6 | 8 |
  |---|--:|--:|--:|--:|--:|--:|--:|
  | wall | 0.975 | 0.987 | 1.019 | 1.015 | **1.054** | 0.992 | 1.005 |
  | 95% interval | 0.936–1.015 | 0.952–1.022 | 0.994–1.045 | 0.993–1.037 | 1.047–1.061 | 0.976–1.007 | 0.994–1.017 |
  | peak RSS, MB | 32 → 29 | 75 → 63 | 93 → 84 | 120 → 99 | 149 → 117 | 66 → 66 | 81 → 80 |

  The gain grows with the number of co-running Cells workers: none at 1–2 threads, 3.8% less
  CPU at 4 (1.040, 1.025–1.055) and 5.1% less wall at 5. At 6 and 8 threads, where the plan is
  the teams path and Cells isn't used, nothing changes.
- **Release scaling, the i9-10900K** (`release-scaling/run-20261115T134325Z`, sha256 3eafdc28,
  10 paired rounds, every arm the same bytes, #417), `wikidata`, 0.3.1 ÷ 0.3.2 wall:

  | threads | 1 | 2 | 4 | 8 | 10 | 12 | 16 | 20 |
  |---|--:|--:|--:|--:|--:|--:|--:|--:|
  | 0.3.1 ÷ 0.3.2 | **0.955** | 1.015 | **1.027** | 0.998 | 0.995 | 0.994 | **0.960** | **0.958** |
  | 95% interval | 0.923–0.987 | 0.987–1.044 | 1.009–1.046 | 0.988–1.007 | 0.981–1.010 | 0.977–1.012 | 0.929–0.992 | 0.933–0.984 |

  At 4 threads, 2.6% less wall time and 3.2% less CPU (1.033, 1.018–1.048), beyond noise. At
  1 thread this run is 4.7% slower, beyond noise, while the earlier i9 run of #408 (0.975,
  0.936–1.015) and CI (−0.2%, −2.6 to +2.3) were within noise. 16 and 20 threads are also slower,
  but they use the teams path, which #408 does not change, so their slowdown points to drift
  within the run; read the 1-thread figure with that in mind.
- **Caveats.**
  - The wall gain is modest: beyond noise only at 4 and 5 threads on the i9, and not on CI's
    2-core runner. The memory reduction is the clear effect.
  - The release-scaling run's 1-thread slowdown is not seen elsewhere, and is not hidden here.

### <!-- 9 -->Other

- Cells induction: recycle the dead front slot instead of taking a fresh one ([#408](https://github.com/lmmx/mibz2/pull/408))
- Research, record, docs and blog changes: the phase attribution of Cells' memory traffic ([#399](https://github.com/lmmx/mibz2/pull/399)), 0.3.1's notes and release-scaling record ([#401](https://github.com/lmmx/mibz2/pull/401), [#402](https://github.com/lmmx/mibz2/pull/402)), a blog post on slot recycling ([#412](https://github.com/lmmx/mibz2/pull/412)) and the blog skills ([#413](https://github.com/lmmx/mibz2/pull/413))

## [0.3.1](https://github.com/lmmx/mibz2/compare/v0.3.0...v0.3.1) - 2026-11-14

A memory-traffic release for Cells. Each letter's slot in Cells' two orders shrinks from 24 to
20 bytes (#391): the link to the letter's slot in the other order is now a 4-byte offset, since a
block's buffer is far below 4 GiB. Fewer slot bytes means less DRAM traffic once several blocks
decode at once. The output, the API, the supported inputs and the plans are unchanged.

- **The paired runbook bench, CI** (run 43646276187, AMD EPYC 9V45, 4 vCPU), v0.3.0 → master:
  `wikidata` −4.3% (−7.5, −1.0), −3.1% (−5.4, −0.6) and −4.0% (−6.0, −2.0) at 1, 2 and 4 threads;
  peak RSS −4.5%, −3.0% and −12.7%. No corpus slower in wall time. `text` at 1 thread is +2.6%
  in CPU, at the edge of its interval. The assessment: better, and no measure worse.
- **The i9-10900K, forced Workers + Cells** (`cells-diet/run-20261114T195323Z`, sha256 52275b76,
  10 paired rounds, #145 LOG 6883380160): 20 B ÷ 24 B wall 0.980 at 1 thread, **0.957** at 4,
  0.926 at 6 and **0.911** at 10. The gain grows with the thread count, as a bandwidth lever
  should. CPU amplification at 10 threads against 1 fell by about 10% (Q = 0.904, 0.851–0.962).
- **Why:** the memory-dose and working-set tests (#387, #390) showed that Workers + Cells is
  bound by DRAM traffic at 4 threads and up: 1.25 DRAM bytes per output byte with one block in
  flight, 15.5 at 4, 24.2 at 10 (#145 LOG 6882123681).
- **Caveats.**
  - The knee does not move: the best wall is still at 6 threads, and only the plateau is lower
    and flatter.
  - The next step of the diet, a 16-byte slot (#398), was built and rejected: it showed no
    primary gain on CI and +4.4% CPU at 4 threads, from one more dependent load per letter.
- **Release scaling, the i9-10900K** (`release-scaling/run-20261114T231702Z`, sha256 914653a3,
  10 paired rounds, every arm the same bytes), `wikidata`, 0.3.0 ÷ 0.3.1 wall:

  | threads | 1 | 2 | 4 | 8 | 10 | 12 | 16 | 20 |
  |---|--:|--:|--:|--:|--:|--:|--:|--:|
  | 0.3.0 ÷ 0.3.1 | 1.051 | **1.034** | **1.042** | 1.006 | 1.000 | 1.024 | 1.017 | 0.987 |
  | 95% interval | 0.990–1.116 | 1.010–1.059 | 1.022–1.062 | 0.997–1.016 | 0.986–1.013 | 0.996–1.053 | 0.995–1.039 | 0.945–1.031 |

  That is 3.3% and 4.0% less wall time at 2 and 4 threads, beyond noise. 1 thread is 4.9% in
  point estimate, but its interval includes no change. From 8 threads it is level, where the plan
  is the unchanged teams.

### <!-- 4 -->Documentation

- *(changelog)* 0.3.0's notes from the i9 release scaling, the paired run against 0.2.9 and wikidata-pq#225 ([#384](https://github.com/lmmx/mibz2/pull/384))

### <!-- 9 -->Other

- the 20-byte slot (buffer diet step 1), and bench-cells-diet ([#391](https://github.com/lmmx/mibz2/pull/391))
- Cells memory-dose experiment: dose feature, recipe and analysis ([#387](https://github.com/lmmx/mibz2/pull/387))

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
