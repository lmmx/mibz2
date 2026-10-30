# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.1](https://github.com/lmmx/mibz2/compare/v0.2.0...v0.2.1) - 2026-10-30

### <!-- 3 -->Performance

- *(bwt)* the one-core build in one pass ([#114](https://github.com/lmmx/mibz2/pull/114))

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
