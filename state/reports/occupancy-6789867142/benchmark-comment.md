<!-- mibz2-bench -->
## Benchmark: mibz2 decoder

**head `bfd5d32` against base `2635f04`** (the merge base with master): 10 paired rounds on one runner · [run](https://github.com/lmmx/mibz2/actions/runs/42984639455)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.1

Each round decodes every corpus once with each build at each thread count (`ab run`: the base's harness binary as the arm `mibz2@base/ab`, so both builds run in the same process shape), the builds' order alternating. **Change** is head over base: the geometric mean of the rounds' paired ratios and its 95% interval; − is faster, and **faster** or **slower** where the interval excludes no change. 1-thread runs are pinned to one CPU.

| corpus | threads | wall, base → head (median) | wall change | CPU change | peak RSS, base → head | head MB/s | same bytes |
|---|--:|--:|--:|--:|--:|--:|---|
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.429 → 0.430 s | +0.1% (−0.2%, +0.5%) | +0.1% (−0.2%, +0.4%) | 26 → 26 MB (+0.0%) | 488 | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.241 → 0.242 s | −0.2% (−1.7%, +1.4%) | −0.3% (−1.8%, +1.2%) | 52 → 52 MB (+0.3%) | 867 | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.220 → 0.220 s | −0.5% (−1.8%, +0.9%) | −0.5% (−1.9%, +1.0%) | 81 → 81 MB (−0.7%) | 955 | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 → 0.018 s | −0.6% (−2.0%, +0.8%) | −0.6% (−1.8%, +0.7%) | 33 → 34 MB (−0.0%) | 1,133 | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 → 0.018 s | +0.1% (−1.6%, +1.8%) | +0.1% (−1.5%, +1.7%) | 33 → 33 MB (−0.2%) | 1,099 | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 → 0.018 s | +0.2% (−4.4%, +5.1%) | +0.0% (−4.2%, +4.5%) | 33 → 33 MB (+0.1%) | 1,096 | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 → 0.026 s | +0.2% (−2.1%, +2.4%) | +0.1% (−2.1%, +2.3%) | 16 → 16 MB (−0.1%) | 748 | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.018 → 0.017 s | −10.5% (−19.9%, +0.1%) | −1.1% (−4.0%, +1.9%) | 23 → 22 MB (−0.9%) | 1,130 | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.020 → 0.020 s | +0.8% (−2.5%, +4.2%) | +0.3% (−3.7%, +4.5%) | 39 → 39 MB (−0.7%) | 966 | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.110 → 0.109 s | −1.3% (−4.5%, +2.0%) | −1.3% (−4.4%, +1.9%) | 23 → 23 MB (+0.0%) | 73 | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.067 → 0.068 s | −0.4% (−3.6%, +2.8%) | −0.8% (−3.5%, +1.9%) | 38 → 38 MB (+0.7%) | 119 | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.054 → 0.056 s | +1.8% (−0.9%, +4.4%) | −0.3% (−1.6%, +0.9%) | 50 → 50 MB (−0.4%) | 144 | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.118 → 0.118 s | +0.6% (−1.0%, +2.3%) | +0.6% (−1.0%, +2.3%) | 23 → 23 MB (−0.1%) | 82 | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.071 → 0.073 s | +3.8% (+0.2%, +7.5%) **slower** | +2.2% (−0.5%, +5.0%) | 41 → 41 MB (−1.7%) | 131 | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.062 → 0.063 s | +0.3% (−1.9%, +2.5%) | +0.2% (−2.5%, +2.8%) | 54 → 54 MB (−0.2%) | 153 | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 → 0.012 s | −0.1% (−2.2%, +2.1%) | +0.0% (−1.9%, +2.0%) | 23 → 23 MB (−0.1%) | 833 | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.012 → 0.012 s | +1.0% (−2.1%, +4.3%) | +0.7% (−1.9%, +3.3%) | 24 → 24 MB (+0.2%) | 823 | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.012 → 0.012 s | −2.0% (−5.0%, +1.0%) | −1.9% (−4.5%, +0.7%) | 23 → 23 MB (−0.1%) | 826 | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 1 | 0.057 → 0.057 s | +2.5% (−1.7%, +6.9%) | +2.4% (−1.7%, +6.6%) | 19 → 19 MB (−0.0%) | 228 | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 2 | 0.042 → 0.040 s | −1.9% (−6.7%, +3.1%) | −1.2% (−4.8%, +2.6%) | 40 → 41 MB (+2.8%) | 328 | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 4 | 0.037 → 0.036 s | −1.4% (−5.1%, +2.5%) | +0.1% (−3.0%, +3.4%) | 58 → 59 MB (+0.4%) | 364 | yes |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

### Resources: CPU per byte and scaling, the `wikidata` corpus

**Assessment:** No change detected on the four measures at this run's precision (every interval holds no change; not a proof that nothing changed).

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | base MB/s | base CPU ns/B | base amplification | base efficiency | head MB/s | head CPU ns/B | head amplification | head efficiency | amplification, head over base |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| 1: pinned to CPU 1 | 489 | 2.05 | 1 | – | 488 | 2.06 | 1 | – |  |
| 2: CPUs 0,2, one a core | 869 | 2.25 | 1.10× (1.08–1.11) | 89% | 867 | 2.25 | 1.09× (1.09–1.10) | 89% | −0.4% (−1.8%, +1.0%) |
| 4: all 4 CPUs, sharing SMT siblings | 952 | 3.83 | 1.87× (1.85–1.88) | 49% | 955 | 3.80 | 1.85× (1.83–1.88) | 49% | −0.6% (−2.1%, +0.9%) |

**Best:** base at 4 threads (952 MB/s); head at 4 threads (955 MB/s); head's best against base's, wall: −0.5% (−1.8%, +0.9%).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

<details><summary>How to read it, and what it cannot show</summary>

- **The four comparisons the assessment reads,** declared in bench/report.py (`assessment`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run.
- **A trade-off is not a failure.** Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting.
- **What it can't show:** which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, #39). The cores' L3 is shared with other tenants and its size unknown.
- **Run-queue wait** (threads' mean count runnable, not running; schedstat): base 1t 0.00, base 2t 0.80, base 4t 1.19, head 1t 0.00, head 2t 0.80, head 4t 1.20. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference.
- **The per-thread split's unattributed CPU** (the process's total that no sampled thread holds, sampled every 10 ms): base 2t 11%, base 4t 9%, head 2t 11%, head 4t 11%.

</details>

### Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
|---|--:|--:|--:|--:|--:|
| head | 0.08 | 0.61 | 0.02 | 0.08 | 0.220 |
| base | 0.09 | 0.62 | 0.02 | 0.08 | 0.220 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

### Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs each build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU, the builds' order alternating: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3696 MHz). **Change** is head over base per stage, paired by round as above; with seven intervals a run, a mark on one stage alone can be chance (about one run in three). Diagnostic only: the decoder's own figures are the tables above. 10 runs, 1m43s of the run.

| stage | base → head c/B (median) | head share | change |
|---|--:|--:|--:|
| scanner (headers, Huffman) | 1.11 → 1.11 | 12.8% | +0.1% (−0.3%, +0.4%) |
| MTF + RLE2 | 1.52 → 1.52 | 17.6% | −0.3% (−0.7%, +0.1%) |
| T-vector build | 1.49 → 1.48 | 17.2% | −0.1% (−0.4%, +0.3%) |
| walk (the one each build uses) | 4.03 → 4.08 | 47.3% | +0.8% (−0.8%, +2.4%) |
| of which: segment assembly | 0.15 → 0.15 | 1.7% | +4.2% (−7.1%, +16.9%) |
| RLE1 | 0.19 → 0.19 | 2.2% | +0.1% (−1.7%, +1.9%) |
| CRC | 0.26 → 0.26 | 3.0% | +0.2% (−0.2%, +0.6%) |
| **total** | 8.60 → 8.63 |  | +0.3% (−0.4%, +1.1%) |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: base, “walk, room (used)”; head, “walk, room (used)”.

### Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

| | base | head | head over base, paired |
|---|--:|--:|--:|
| 1 thread | 1h02m ± 7m01s | 1h02m ± 7m07s | +0.1% (−0.2%, +0.5%) |
| 4 threads | 32m08s ± 4m31s | 31m59s ± 4m29s | −0.5% (−1.8%, +0.9%) |
| 4 threads, the sample's overlap gap as a fixed cost | 22m40s ± 3m11s | 22m16s ± 3m08s |  |
| the scanner alone (its serial part) | 12m22s ± 32.9s | 11m47s ± 19.7s |  |
| the workers alone, over 4 | 22m40s ± 3m11s | 22m16s ± 3m08s |  |

At 4 threads the workers bound it. Per byte: the scanner 6.8 ns a byte of bz2, the workers 2.92 ns a byte of output at 4 threads (CPU, summed over them), 1.66 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (head).

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
| `text` | 2,597,062 | 13,035,595 | `b6341d3af4a2a231…` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 |

- CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3695.5 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1
- rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source)
- bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4

</details>
