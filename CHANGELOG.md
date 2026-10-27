# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
