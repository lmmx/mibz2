<!-- mibz2-bench -->
## Benchmark: mibz2 decoder

**head `660a1b7` against base `4df9651`** (the merge base with master): 10 paired rounds on one runner · [run](https://github.com/lmmx/mibz2/actions/runs/43401424749)

Intel(R) Xeon(R) 6975P-C, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each build at each thread count (`ab run`: the base's harness binary as the arm `mibz2@base/ab`, so both builds run in the same process shape), the builds' order alternating. **Change** is head over base: the geometric mean of the rounds' paired ratios and its 95% interval; − is faster, and **faster** or **slower** where the interval excludes no change. 1-thread runs are pinned to one CPU.

| corpus | threads | wall, base → head (median) | wall change | CPU change | peak RSS, base → head | head MB/s | same bytes |
|---|--:|--:|--:|--:|--:|--:|---|
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.530 → 0.530 s | +0.8% (−1.0%, +2.6%) | +0.8% (−1.0%, +2.6%) | 26 → 26 MB (−0.1%) | 396 | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.290 → 0.292 s | +0.5% (−0.9%, +2.0%) | +0.4% (−1.1%, +1.8%) | 52 → 52 MB (−0.2%) | 718 | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.252 → 0.252 s | +0.4% (−0.5%, +1.3%) | +1.1% (−1.9%, +4.3%) | 77 → 78 MB (+1.4%) | 831 | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.016 → 0.016 s | −0.4% (−1.8%, +1.1%) | −0.7% (−2.4%, +1.1%) | 34 → 33 MB (−0.1%) | 1,238 | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.017 → 0.017 s | −0.2% (−1.9%, +1.4%) | −0.5% (−1.9%, +0.9%) | 33 → 33 MB (+0.1%) | 1,212 | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.017 → 0.017 s | −0.7% (−3.3%, +1.9%) | −0.9% (−3.4%, +1.6%) | 33 → 33 MB (+0.0%) | 1,208 | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.031 → 0.031 s | +0.4% (−1.6%, +2.5%) | +0.4% (−1.7%, +2.6%) | 16 → 16 MB (+0.1%) | 635 | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.020 → 0.020 s | −2.6% (−7.4%, +2.4%) | −1.2% (−4.8%, +2.5%) | 22 → 22 MB (−1.7%) | 998 | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.019 → 0.019 s | −3.1% (−7.7%, +1.6%) | +2.0% (−2.7%, +7.0%) | 39 → 39 MB (−0.1%) | 1,026 | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.114 → 0.114 s | −0.1% (−0.4%, +0.3%) | −0.1% (−0.5%, +0.3%) | 23 → 23 MB (+0.1%) | 70 | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.068 → 0.069 s | +0.6% (−1.3%, +2.6%) | +0.3% (−0.9%, +1.6%) | 38 → 38 MB (−0.2%) | 117 | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.056 → 0.055 s | −1.6% (−4.7%, +1.5%) | −0.9% (−3.5%, +1.7%) | 50 → 50 MB (+0.1%) | 146 | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.126 → 0.126 s | −0.2% (−0.6%, +0.2%) | −0.2% (−0.6%, +0.2%) | 23 → 23 MB (+0.1%) | 76 | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.073 → 0.074 s | +0.6% (−1.4%, +2.7%) | −0.2% (−1.0%, +0.6%) | 42 → 43 MB (+1.6%) | 129 | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.062 → 0.063 s | +0.8% (−1.5%, +3.1%) | −0.1% (−2.9%, +2.8%) | 53 → 53 MB (−0.5%) | 153 | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.011 → 0.011 s | +1.1% (−0.3%, +2.6%) | +1.6% (−0.2%, +3.5%) | 23 → 23 MB (−0.1%) | 922 | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 → 0.011 s | +0.2% (−1.9%, +2.3%) | +0.5% (−1.8%, +2.9%) | 23 → 23 MB (−0.2%) | 885 | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 → 0.011 s | −1.0% (−2.6%, +0.6%) | −1.0% (−2.3%, +0.4%) | 23 → 23 MB (−0.0%) | 885 | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 1 | 0.060 → 0.060 s | −0.1% (−0.5%, +0.3%) | +0.0% (−0.4%, +0.4%) | 19 → 19 MB (−0.1%) | 216 | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 2 | 0.040 → 0.039 s | −2.5% (−5.2%, +0.2%) | −1.0% (−2.0%, +0.1%) | 36 → 35 MB (−1.3%) | 336 | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 4 | 0.034 → 0.034 s | +1.0% (−3.7%, +5.8%) | −0.8% (−3.4%, +1.8%) | 54 → 54 MB (−0.3%) | 381 | yes |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

### Resources: CPU per byte and scaling, the `wikidata` corpus

**Assessment:** No change detected on the four measures at this run's precision (every interval holds no change; not a proof that nothing changed).

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | base MB/s | base CPU ns/B | base amplification | base efficiency | head MB/s | head CPU ns/B | head amplification | head efficiency | amplification, head over base |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| 1: pinned to CPU 1 | 396 | 2.54 | 1 | – | 396 | 2.54 | 1 | – |  |
| 2: CPUs 0,1, one a core | 723 | 2.74 | 1.08× (1.07–1.09) | 91% | 718 | 2.73 | 1.08× (1.06–1.09) | 91% | −0.4% (−1.8%, +1.1%) |
| 4: all 4 CPUs, sharing SMT siblings | 832 | 4.59 | 1.79× (1.74–1.83) | 53% | 831 | 4.61 | 1.79× (1.75–1.84) | 53% | +0.4% (−3.7%, +4.6%) |

**Best:** base at 4 threads (832 MB/s); head at 4 threads (831 MB/s); head's best against base's, wall: +0.4% (−0.5%, +1.3%).

Conditions: 4 CPUs on 2 physical cores (0,2 | 1,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,1 (one CPU a core); 4-thread runs share cores' SMT siblings.

<details><summary>How to read it, and what it cannot show</summary>

- **The four comparisons the assessment reads,** declared in bench/report.py (`assessment`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run.
- **A trade-off is not a failure.** Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting.
- **What it can't show:** which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, #39). The cores' L3 is shared with other tenants and its size unknown.
- **Run-queue wait** (threads' mean count runnable, not running; schedstat): base 1t 0.00, base 2t 0.73, base 4t 1.06, head 1t 0.00, head 2t 0.74, head 4t 1.06. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference.
- **The per-thread split's unattributed CPU** (the process's total that no sampled thread holds, sampled every 10 ms): base 2t 10%, base 4t 10%, head 2t 9%, head 4t 9%.

</details>

### Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
|---|--:|--:|--:|--:|--:|
| head | 0.09 | 0.77 | 0.02 | 0.09 | 0.252 |
| base | 0.09 | 0.77 | 0.02 | 0.09 | 0.252 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

### Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs each build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU, the builds' order alternating: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3900 MHz). **Change** is head over base per stage, paired by round as above; with seven intervals a run, a mark on one stage alone can be chance (about one run in three). Diagnostic only: the decoder's own figures are the tables above. 10 runs, 1m54s of the run.

| stage | base → head c/B (median) | head share | change |
|---|--:|--:|--:|
| scanner (headers, Huffman) | 1.20 → 1.19 | 11.0% | −0.2% (−0.7%, +0.2%) |
| MTF + RLE2 | 1.76 → 1.77 | 16.3% | +0.1% (−0.2%, +0.3%) |
| T-vector build | 1.55 → 1.55 | 14.3% | −0.1% (−0.7%, +0.4%) |
| walk (the one each build uses) | 5.93 → 5.92 | 54.7% | +0.0% (−0.5%, +0.6%) |
| of which: segment assembly | 0.31 → 0.31 | 2.8% | +0.9% (−1.8%, +3.7%) |
| RLE1 | 0.26 → 0.26 | 2.4% | +0.1% (−0.7%, +0.9%) |
| CRC | 0.14 → 0.14 | 1.3% | −0.7% (−2.1%, +0.8%) |
| **total** | 10.83 → 10.82 |  | −0.0% (−0.3%, +0.3%) |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: base, “walk, room (used)”; head, “walk, room (used)”.

### Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

| | base | head | head over base, paired |
|---|--:|--:|--:|
| 1 thread | 1h17m ± 9m04s | 1h17m ± 9m15s | +0.8% (−1.0%, +2.6%) |
| 4 threads | 36m39s ± 5m09s | 36m48s ± 5m10s | +0.4% (−0.5%, +1.3%) |
| 4 threads, the sample's overlap gap as a fixed cost | 27m30s ± 3m58s | 27m45s ± 3m58s |  |
| the scanner alone (its serial part) | 12m22s ± 32.9s | 12m31s ± 43.7s |  |
| the workers alone, over 4 | 27m30s ± 3m58s | 27m45s ± 3m58s |  |

At 4 threads the workers bound it. Per byte: the scanner 7.3 ns a byte of bz2, the workers 3.64 ns a byte of output at 4 threads (CPU, summed over them), 2.14 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (head).

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
| `text` | 2,595,980 | 13,035,883 | `ca1f7f044bc5b44a…` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 |

- CPU: Intel(R) Xeon(R) 6975P-C, 4 cores, reported clock 3900.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4_1, ssse3; kernel 7.0.0-1016-aws; image ubuntu26 20261111.82.6
- rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source)
- bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4

</details>

