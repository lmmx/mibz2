<!-- mibz2-bench -->
## Benchmark: mibz2 decoder

**head `a9c7716` against base `6dec95b`** (the merge base with master): 10 paired rounds on one runner · [run](https://github.com/lmmx/mibz2/actions/runs/43534513477)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each build at each thread count (`ab run`: the base's harness binary as the arm `mibz2@base/ab`, so both builds run in the same process shape), the builds' order alternating. **Change** is head over base: the geometric mean of the rounds' paired ratios and its 95% interval; − is faster, and **faster** or **slower** where the interval excludes no change. 1-thread runs are pinned to one CPU.

| corpus | threads | wall, base → head (median) | wall change | CPU change | peak RSS, base → head | head MB/s | same bytes |
|---|--:|--:|--:|--:|--:|--:|---|
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.524 → 0.528 s | −0.8% (−3.1%, +1.7%) | −0.8% (−3.1%, +1.6%) | 26 → 34 MB (+30.9% **more**) | 397 | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.293 → 0.294 s | +1.1% (−1.1%, +3.4%) | +0.9% (−1.0%, +2.7%) | 52 → 52 MB (−0.0%) | 713 | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.263 → 0.264 s | +0.8% (−1.2%, +3.0%) | −0.9% (−2.2%, +0.4%) | 80 → 81 MB (−0.4%) | 793 | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.020 → 0.022 s | +8.0% (+7.2%, +8.9%) **slower** | +7.3% (+6.2%, +8.5%) **slower** | 33 → 36 MB (+7.3% **more**) | 920 | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 → 0.021 s | +0.2% (−1.6%, +2.1%) | +0.1% (−1.7%, +1.8%) | 33 → 33 MB (+0.1%) | 955 | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 → 0.021 s | +1.5% (−0.2%, +3.3%) | +1.4% (−0.1%, +3.0%) | 33 → 33 MB (−0.1%) | 943 | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.033 → 0.054 s | +59.6% (+57.9%, +61.3%) **slower** | +56.0% (+54.4%, +57.6%) **slower** | 16 → 18 MB (+14.1% **more**) | 365 | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 → 0.022 s | +0.6% (−3.4%, +4.9%) | +2.4% (+0.7%, +4.2%) **slower** | 22 → 23 MB (+2.6% **more**) | 873 | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 → 0.025 s | −1.2% (−5.3%, +3.0%) | −0.9% (−6.4%, +5.0%) | 39 → 40 MB (+1.5%) | 786 | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.134 → 0.134 s | +0.1% (−0.4%, +0.6%) | +0.2% (−0.3%, +0.7%) | 23 → 23 MB (−0.0%) | 60 | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.085 → 0.085 s | −0.2% (−2.8%, +2.4%) | −1.0% (−3.3%, +1.4%) | 38 → 38 MB (+0.1%) | 94 | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.070 → 0.070 s | +0.3% (−3.5%, +4.2%) | +0.2% (−2.6%, +3.0%) | 49 → 50 MB (+0.4%) | 114 | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.146 → 0.147 s | −0.1% (−1.3%, +1.1%) | −0.1% (−1.3%, +1.1%) | 23 → 23 MB (−0.1%) | 66 | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.089 → 0.089 s | +0.8% (−1.5%, +3.2%) | +0.5% (−0.8%, +1.8%) | 42 → 42 MB (+1.4%) | 108 | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.078 → 0.079 s | +0.9% (−0.4%, +2.2%) | +0.5% (−3.4%, +4.5%) | 54 → 54 MB (+0.2%) | 122 | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 → 0.013 s | +0.1% (−1.0%, +1.1%) | +0.3% (−1.0%, +1.6%) | 23 → 23 MB (−0.1%) | 746 | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 → 0.014 s | +1.0% (−0.0%, +2.0%) | +0.4% (−0.8%, +1.5%) | 23 → 23 MB (−0.2%) | 697 | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 → 0.014 s | +0.1% (−1.4%, +1.6%) | +0.5% (−1.2%, +2.3%) | 23 → 23 MB (−0.2%) | 694 | yes |
| `text` (2.41 MB → 12.2 MB, 5.0×) | 1 | 0.065 → 0.065 s | −1.0% (−3.3%, +1.4%) | −1.0% (−3.3%, +1.4%) | 19 → 19 MB (+0.0%) | 187 | yes |
| `text` (2.41 MB → 12.2 MB, 5.0×) | 2 | 0.047 → 0.047 s | −0.1% (−2.3%, +2.1%) | −0.8% (−2.5%, +0.8%) | 35 → 36 MB (+1.1%) | 260 | yes |
| `text` (2.41 MB → 12.2 MB, 5.0×) | 4 | 0.043 → 0.041 s | −3.9% (−7.0%, −0.6%) **faster** | +1.5% (−1.9%, +5.0%) | 54 → 55 MB (+2.4%) | 295 | yes |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

### Resources: CPU per byte and scaling, the `wikidata` corpus

**Assessment:** No change detected on the four measures at this run's precision (every interval holds no change; not a proof that nothing changed).

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | base MB/s | base CPU ns/B | base amplification | base efficiency | head MB/s | head CPU ns/B | head amplification | head efficiency | amplification, head over base |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| 1: pinned to CPU 1 | 400 | 2.51 | 1 | – | 397 | 2.53 | 1 | – |  |
| 2: CPUs 0,2, one a core | 715 | 2.75 | 1.08× (1.05–1.11) | 91% | 713 | 2.75 | 1.10× (1.08–1.11) | 89% | +1.6% (−1.3%, +4.6%) |
| 4: all 4 CPUs, sharing SMT siblings | 798 | 4.55 | 1.80× (1.74–1.86) | 50% | 793 | 4.53 | 1.79× (1.78–1.81) | 50% | −0.2% (−2.7%, +2.4%) |

**Best:** base at 4 threads (798 MB/s); head at 4 threads (793 MB/s); head's best against base's, wall: +0.8% (−1.2%, +3.0%).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

<details><summary>How to read it, and what it cannot show</summary>

- **The four comparisons the assessment reads,** declared in bench/report.py (`assessment`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run.
- **A trade-off is not a failure.** Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting.
- **What it can't show:** which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, #39). The cores' L3 is shared with other tenants and its size unknown.
- **Run-queue wait** (threads' mean count runnable, not running; schedstat): base 1t 0.00, base 2t 0.74, base 4t 1.18, head 1t 0.00, head 2t 0.74, head 4t 1.20. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference.
- **The per-thread split's unattributed CPU** (the process's total that no sampled thread holds, sampled every 10 ms): base 2t 11%, base 4t 9%, head 2t 10%, head 4t 8%.

</details>

### Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
|---|--:|--:|--:|--:|--:|
| head | 0.11 | 0.74 | 0.02 | 0.08 | 0.264 |
| base | 0.10 | 0.74 | 0.02 | 0.09 | 0.263 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

### Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs each build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU, the builds' order alternating: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3246 MHz). **Change** is head over base per stage, paired by round as above; with seven intervals a run, a mark on one stage alone can be chance (about one run in three). Diagnostic only: the decoder's own figures are the tables above. 10 runs, 2m04s of the run.

| stage | base → head c/B (median) | head share | change |
|---|--:|--:|--:|
| scanner (headers, Huffman) | 1.13 → 1.13 | 12.8% | −0.1% (−0.7%, +0.4%) |
| MTF + RLE2 | 1.58 → 1.59 | 18.0% | +0.2% (−0.2%, +0.6%) |
| T-vector build | 1.51 → 1.51 | 17.0% | −0.6% (−1.6%, +0.3%) |
| walk (the one each build uses) | 4.09 → 4.09 | 46.3% | +0.0% (−3.0%, +3.2%) |
| of which: segment assembly | 0.15 → 0.14 | 1.6% | −3.0% (−5.0%, −0.9%) **faster** |
| RLE1 | 0.28 → 0.28 | 3.1% | −1.4% (−2.7%, −0.1%) **faster** |
| CRC | 0.26 → 0.26 | 2.9% | +0.1% (−0.5%, +0.7%) |
| **total** | 8.85 → 8.84 |  | −0.1% (−1.7%, +1.5%) |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: base, “walk, room (used)”; head, “walk, room (used)”.

### Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

| | base | head | head over base, paired |
|---|--:|--:|--:|
| 1 thread | 1h17m ± 8m57s | 1h16m ± 8m36s | −0.8% (−3.1%, +1.7%) |
| 4 threads | 38m24s ± 5m25s | 38m43s ± 5m28s | +0.8% (−1.2%, +3.0%) |
| 4 threads, the sample's overlap gap as a fixed cost | 27m08s ± 3m51s | 26m57s ± 3m47s |  |
| the scanner alone (its serial part) | 15m08s ± 32.2s | 15m25s ± 32.2s |  |
| the workers alone, over 4 | 27m08s ± 3m51s | 26m57s ± 3m47s |  |

At 4 threads the workers bound it. Per byte: the scanner 8.9 ns a byte of bz2, the workers 3.53 ns a byte of output at 4 threads (CPU, summed over them), 2.01 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (head).

<details><summary>What the model assumes</summary>

- **Linear in bytes:** the scanner's time in the dump's bz2 bytes, the workers' in its output bytes.
- **The sample stands for the dump:** 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×.
- **The wall at N threads** is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row.
- **The machine is this runner,** reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded.
- The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

</details>

<details><summary>The corpora, the machine and the toolchain</summary>

| corpus | bz2 bytes | output bytes | SHA-256 | source |
|---|--:|--:|---|---|
| `wikidata` | 11,854,933 | 209,700,196 | `82f65ace0731962d…` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) |
| `onebyte` | 51 | 20,000,000 | `22924f5b3187645f…` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 |
| `pattern` | 1,116 | 19,600,000 | `0f07e2a274b91896…` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 |
| `random` | 7,097,921 | 8,000,000 | `10d444e6204289af…` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 |
| `records` | 9,587,840 | 9,600,000 | `ff8b5f6ed3dd4445…` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 |
| `runs` | 37,995 | 10,000,235 | `af3be4d130faf312…` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 |
| `text` | 2,409,171 | 12,164,402 | `acb5f8a27b49b265…` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 |

- CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3245.7 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1
- rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source)
- bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4

</details>
