<!-- mibz2-bench -->
## Benchmark: mibz2 decoder

**head `93781bd` against base `3e7e294`** (the merge base with master): 10 paired rounds on one runner · [run](https://github.com/lmmx/mibz2/actions/runs/43422422187)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each build at each thread count (`ab run`: the base's harness binary as the arm `mibz2@base/ab`, so both builds run in the same process shape), the builds' order alternating. **Change** is head over base: the geometric mean of the rounds' paired ratios and its 95% interval; − is faster, and **faster** or **slower** where the interval excludes no change. 1-thread runs are pinned to one CPU.

| corpus | threads | wall, base → head (median) | wall change | CPU change | peak RSS, base → head | head MB/s | same bytes |
|---|--:|--:|--:|--:|--:|--:|---|
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.431 → 0.430 s | −0.1% (−0.7%, +0.4%) | −0.2% (−0.7%, +0.4%) | 26 → 26 MB (−0.1%) | 488 | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.242 → 0.241 s | −0.4% (−2.2%, +1.3%) | −0.2% (−0.8%, +0.3%) | 52 → 52 MB (+2.0% **more**) | 870 | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.221 → 0.220 s | −0.7% (−2.0%, +0.5%) | −0.6% (−2.0%, +0.8%) | 81 → 81 MB (+0.1%) | 954 | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 → 0.018 s | −5.1% (−17.5%, +9.2%) | −4.5% (−16.2%, +8.9%) | 33 → 33 MB (+0.1%) | 1,127 | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 → 0.018 s | −1.1% (−4.3%, +2.3%) | −0.9% (−3.9%, +2.2%) | 33 → 33 MB (+0.2%) | 1,114 | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 → 0.018 s | +0.1% (−2.5%, +2.8%) | +0.0% (−2.2%, +2.4%) | 33 → 33 MB (−0.1%) | 1,096 | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 → 0.026 s | −0.3% (−0.9%, +0.3%) | −0.5% (−1.2%, +0.3%) | 16 → 16 MB (−0.0%) | 754 | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.018 → 0.017 s | +1.0% (−7.1%, +9.8%) | −0.1% (−2.9%, +2.7%) | 22 → 22 MB (−0.5%) | 1,146 | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.020 → 0.020 s | +2.8% (−3.4%, +9.4%) | −0.9% (−5.4%, +3.8%) | 40 → 41 MB (+2.8%) | 963 | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.109 → 0.109 s | +0.0% (−0.8%, +0.9%) | +0.0% (−0.8%, +0.9%) | 23 → 23 MB (−0.0%) | 73 | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.068 → 0.067 s | −2.0% (−5.4%, +1.4%) | −0.7% (−3.2%, +2.0%) | 38 → 39 MB (+2.0%) | 119 | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.056 → 0.058 s | +2.0% (−1.0%, +5.0%) | +0.3% (−1.4%, +2.0%) | 50 → 49 MB (−1.4%) | 139 | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.118 → 0.118 s | +0.2% (−1.4%, +1.8%) | +0.2% (−1.4%, +1.8%) | 23 → 23 MB (+0.0%) | 82 | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.073 → 0.072 s | +1.3% (−7.4%, +10.8%) | +0.5% (−7.8%, +9.6%) | 43 → 42 MB (−0.0%) | 133 | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.063 → 0.063 s | −2.6% (−7.2%, +2.1%) | −4.1% (−6.4%, −1.8%) **faster** | 54 → 52 MB (−3.1% **less**) | 153 | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 → 0.011 s | −0.7% (−3.0%, +1.8%) | −0.7% (−2.7%, +1.4%) | 23 → 23 MB (−0.0%) | 866 | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 → 0.012 s | −1.9% (−3.6%, −0.1%) **faster** | −1.7% (−3.0%, −0.3%) **faster** | 24 → 23 MB (−0.3%) | 840 | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.012 → 0.012 s | +1.6% (−0.4%, +3.8%) | +1.5% (−0.3%, +3.3%) | 23 → 23 MB (−0.0%) | 810 | yes |
| `text` (2.39 MB → 12 MB, 5.0×) | 1 | 0.053 → 0.053 s | +0.3% (−0.5%, +1.2%) | +0.3% (−0.5%, +1.1%) | 19 → 19 MB (−0.0%) | 227 | yes |
| `text` (2.39 MB → 12 MB, 5.0×) | 2 | 0.037 → 0.036 s | −2.4% (−5.7%, +1.0%) | −1.6% (−4.1%, +1.0%) | 36 → 36 MB (−0.8%) | 332 | yes |
| `text` (2.39 MB → 12 MB, 5.0×) | 4 | 0.033 → 0.033 s | −0.3% (−2.7%, +2.2%) | +0.4% (−3.3%, +4.1%) | 54 → 55 MB (+0.8%) | 365 | yes |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

### Resources: CPU per byte and scaling, the `wikidata` corpus

**Assessment:** No change detected on the four measures at this run's precision (every interval holds no change; not a proof that nothing changed).

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | base MB/s | base CPU ns/B | base amplification | base efficiency | head MB/s | head CPU ns/B | head amplification | head efficiency | amplification, head over base |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| 1: pinned to CPU 1 | 486 | 2.06 | 1 | – | 488 | 2.06 | 1 | – |  |
| 2: CPUs 0,2, one a core | 866 | 2.25 | 1.09× (1.09–1.09) | 89% | 870 | 2.24 | 1.09× (1.08–1.10) | 89% | −0.1% (−0.9%, +0.7%) |
| 4: all 4 CPUs, sharing SMT siblings | 948 | 3.86 | 1.86× (1.82–1.89) | 49% | 954 | 3.83 | 1.85× (1.82–1.88) | 49% | −0.5% (−2.2%, +1.3%) |

**Best:** base at 4 threads (948 MB/s); head at 4 threads (954 MB/s); head's best against base's, wall: −0.7% (−2.0%, +0.5%).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

<details><summary>How to read it, and what it cannot show</summary>

- **The four comparisons the assessment reads,** declared in bench/report.py (`assessment`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run.
- **A trade-off is not a failure.** Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting.
- **What it can't show:** which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, #39). The cores' L3 is shared with other tenants and its size unknown.
- **Run-queue wait** (threads' mean count runnable, not running; schedstat): base 1t 0.00, base 2t 0.81, base 4t 1.20, head 1t 0.00, head 2t 0.79, head 4t 1.18. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference.
- **The per-thread split's unattributed CPU** (the process's total that no sampled thread holds, sampled every 10 ms): base 2t 11%, base 4t 10%, head 2t 11%, head 4t 12%.

</details>

### Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
|---|--:|--:|--:|--:|--:|
| head | 0.08 | 0.61 | 0.02 | 0.09 | 0.220 |
| base | 0.09 | 0.62 | 0.02 | 0.09 | 0.221 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

### Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs each build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU, the builds' order alternating: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3682 MHz). **Change** is head over base per stage, paired by round as above; with seven intervals a run, a mark on one stage alone can be chance (about one run in three). Diagnostic only: the decoder's own figures are the tables above. 10 runs, 1m43s of the run.

| stage | base → head c/B (median) | head share | change |
|---|--:|--:|--:|
| scanner (headers, Huffman) | 1.10 → 1.10 | 12.9% | −0.2% (−0.7%, +0.4%) |
| MTF + RLE2 | 1.58 → 1.58 | 18.4% | −2.0% (−7.9%, +4.3%) |
| T-vector build | 1.47 → 1.47 | 17.2% | +0.2% (−0.6%, +0.9%) |
| walk (the one each build uses) | 4.01 → 3.98 | 46.4% | −0.8% (−1.7%, +0.1%) |
| of which: segment assembly | 0.14 → 0.14 | 1.6% | −0.6% (−3.7%, +2.5%) |
| RLE1 | 0.19 → 0.19 | 2.2% | −1.3% (−2.4%, −0.1%) **faster** |
| CRC | 0.26 → 0.25 | 3.0% | −0.2% (−0.6%, +0.1%) |
| **total** | 8.63 → 8.57 |  | −0.8% (−1.7%, +0.2%) |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: base, “walk, room (used)”; head, “walk, room (used)”.

### Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

| | base | head | head over base, paired |
|---|--:|--:|--:|
| 1 thread | 1h02m ± 7m03s | 1h02m ± 7m08s | −0.1% (−0.7%, +0.4%) |
| 4 threads | 32m16s ± 4m32s | 32m02s ± 4m30s | −0.7% (−2.0%, +0.5%) |
| 4 threads, the sample's overlap gap as a fixed cost | 22m20s ± 3m11s | 22m11s ± 3m08s |  |
| the scanner alone (its serial part) | 12m22s ± 32.9s | 11m47s ± 19.7s |  |
| the workers alone, over 4 | 22m20s ± 3m11s | 22m11s ± 3m08s |  |

At 4 threads the workers bound it. Per byte: the scanner 6.8 ns a byte of bz2, the workers 2.91 ns a byte of output at 4 threads (CPU, summed over them), 1.67 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (head).

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
| `text` | 2,391,178 | 12,031,602 | `2199959a66dec0a9…` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 |

- CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3682.2 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1
- rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source)
- bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4

</details>
