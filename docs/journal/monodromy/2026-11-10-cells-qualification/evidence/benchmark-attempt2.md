<!-- mibz2-bench -->
## Benchmark: mibz2 decoder

**head `b24d9ba` against base `edaa984`** (the merge base with master): 10 paired rounds on one runner · [run](https://github.com/lmmx/mibz2/actions/runs/42883082689)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.1

Each round decodes every corpus once with each build at each thread count (`ab run`: the base's harness binary as the arm `mibz2@base/ab`, so both builds run in the same process shape), the builds' order alternating. **Change** is head over base: the geometric mean of the rounds' paired ratios and its 95% interval; − is faster, and **faster** or **slower** where the interval excludes no change. 1-thread runs are pinned to one CPU.

| corpus | threads | wall, base → head (median) | wall change | CPU change | peak RSS, base → head | head MB/s | same bytes |
|---|--:|--:|--:|--:|--:|--:|---|
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.522 → 0.525 s | +0.7% (−0.0%, +1.3%) | +0.7% (−0.0%, +1.3%) | 26 → 26 MB (+0.5% **more**) | 400 | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.296 → 0.297 s | +2.0% (−2.6%, +6.8%) | +1.7% (−3.0%, +6.6%) | 52 → 52 MB (−0.7%) | 707 | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.271 → 0.268 s | −0.6% (−2.2%, +1.0%) | −0.0% (−1.6%, +1.6%) | 81 → 81 MB (+0.4%) | 782 | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 → 0.021 s | +1.1% (−2.2%, +4.6%) | +1.5% (−2.2%, +5.3%) | 34 → 33 MB (−0.1%) | 973 | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 → 0.021 s | +0.4% (−2.1%, +3.1%) | +0.4% (−1.7%, +2.5%) | 33 → 33 MB (+0.2%) | 941 | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 → 0.021 s | −0.9% (−2.7%, +0.9%) | −0.8% (−2.4%, +0.9%) | 33 → 33 MB (−0.2%) | 959 | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 → 0.034 s | +0.6% (−0.2%, +1.3%) | +0.4% (−0.3%, +1.1%) | 16 → 16 MB (−0.2%) | 581 | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.024 → 0.022 s | −3.7% (−13.0%, +6.5%) | +0.0% (−3.0%, +3.2%) | 22 → 22 MB (−1.0%) | 877 | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 → 0.026 s | +4.1% (−2.6%, +11.2%) | +3.7% (−0.9%, +8.5%) | 40 → 39 MB (−1.1%) | 766 | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.132 → 0.131 s | −0.5% (−1.3%, +0.2%) | −0.6% (−1.3%, +0.2%) | 23 → 23 MB (+0.4% **more**) | 61 | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.083 → 0.082 s | −0.6% (−2.8%, +1.7%) | −0.8% (−2.2%, +0.6%) | 37 → 39 MB (+2.2%) | 97 | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.068 → 0.070 s | +2.3% (+0.7%, +3.9%) **slower** | +1.0% (−1.8%, +3.8%) | 50 → 50 MB (−0.9%) | 115 | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.146 → 0.145 s | −0.4% (−1.0%, +0.2%) | −0.4% (−1.0%, +0.2%) | 23 → 23 MB (+0.6% **more**) | 66 | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.087 → 0.089 s | +1.2% (−3.3%, +6.0%) | −0.9% (−5.1%, +3.6%) | 41 → 41 MB (−1.4%) | 108 | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.076 → 0.078 s | +2.1% (−1.1%, +5.4%) | +1.0% (−1.9%, +4.1%) | 54 → 53 MB (−1.7%) | 124 | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 → 0.014 s | −0.3% (−1.7%, +1.2%) | −0.2% (−1.8%, +1.4%) | 23 → 23 MB (−0.1%) | 733 | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 → 0.014 s | +0.8% (−1.0%, +2.5%) | +0.6% (−1.1%, +2.4%) | 24 → 23 MB (−0.3%) | 694 | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 → 0.015 s | +2.6% (+0.5%, +4.6%) **slower** | +2.5% (+0.6%, +4.5%) **slower** | 23 → 23 MB (−0.0%) | 676 | yes |
| `text` (2.62 MB → 13.2 MB, 5.0×) | 1 | 0.072 → 0.072 s | +2.1% (−1.5%, +5.8%) | +2.1% (−1.4%, +5.7%) | 19 → 19 MB (−0.0%) | 183 | yes |
| `text` (2.62 MB → 13.2 MB, 5.0×) | 2 | 0.051 → 0.050 s | −1.4% (−5.0%, +2.3%) | −1.0% (−4.0%, +2.0%) | 35 → 35 MB (−0.3%) | 263 | yes |
| `text` (2.62 MB → 13.2 MB, 5.0×) | 4 | 0.046 → 0.046 s | −0.1% (−2.9%, +2.6%) | −0.6% (−3.5%, +2.4%) | 56 → 54 MB (−3.5%) | 285 | yes |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

### Resources: CPU per byte and scaling, the `wikidata` corpus

**Assessment:** No change detected on the four measures at this run's precision (every interval holds no change; not a proof that nothing changed).

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | base MB/s | base CPU ns/B | base amplification | base efficiency | head MB/s | head CPU ns/B | head amplification | head efficiency | amplification, head over base |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| 1: pinned to CPU 1 | 402 | 2.50 | 1 | – | 400 | 2.51 | 1 | – |  |
| 2: CPUs 0,2, one a core | 708 | 2.76 | 1.11× (1.09–1.13) | 88% | 707 | 2.79 | 1.12× (1.08–1.17) | 87% | +1.0% (−3.8%, +6.0%) |
| 4: all 4 CPUs, sharing SMT siblings | 774 | 4.66 | 1.88× (1.85–1.92) | 48% | 782 | 4.70 | 1.87× (1.84–1.90) | 49% | −0.7% (−2.4%, +1.1%) |

**Best:** base at 4 threads (774 MB/s); head at 4 threads (782 MB/s); head's best against base's, wall: −0.6% (−2.2%, +1.0%).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

<details><summary>How to read it, and what it cannot show</summary>

- **The four comparisons the assessment reads,** declared in bench/report.py (`assessment`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run.
- **A trade-off is not a failure.** Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting.
- **What it can't show:** which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, #39). The cores' L3 is shared with other tenants and its size unknown.
- **Run-queue wait** (threads' mean count runnable, not running; schedstat): base 1t 0.00, base 2t 0.75, base 4t 1.17, head 1t 0.00, head 2t 0.75, head 4t 1.16. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference.
- **The per-thread split's unattributed CPU** (the process's total that no sampled thread holds, sampled every 10 ms): base 2t 9%, base 4t 8%, head 2t 9%, head 4t 9%.

</details>

### Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
|---|--:|--:|--:|--:|--:|
| head | 0.11 | 0.77 | 0.02 | 0.09 | 0.268 |
| base | 0.10 | 0.78 | 0.02 | 0.08 | 0.271 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

### Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs each build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU, the builds' order alternating: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3243 MHz). **Change** is head over base per stage, paired by round as above; with seven intervals a run, a mark on one stage alone can be chance (about one run in three). Diagnostic only: the decoder's own figures are the tables above. 10 runs, 2m06s of the run.

| stage | base → head c/B (median) | head share | change |
|---|--:|--:|--:|
| scanner (headers, Huffman) | 1.14 → 1.14 | 12.7% | +0.1% (−1.2%, +1.4%) |
| MTF + RLE2 | 1.52 → 1.52 | 17.1% | +0.2% (−0.8%, +1.2%) |
| T-vector build | 1.55 → 1.54 | 17.3% | −0.9% (−1.8%, −0.1%) **faster** |
| walk (the one each build uses) | 4.21 → 4.17 | 46.9% | −0.5% (−2.5%, +1.7%) |
| of which: segment assembly | 0.15 → 0.15 | 1.7% | −1.3% (−5.6%, +3.2%) |
| RLE1 | 0.28 → 0.28 | 3.2% | +0.2% (−0.4%, +0.9%) |
| CRC | 0.26 → 0.26 | 2.9% | −0.1% (−0.5%, +0.3%) |
| **total** | 8.95 → 8.91 |  | −0.3% (−1.4%, +0.8%) |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: base, “walk, room (used)”; head, “walk, room (used)”.

### Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

| | base | head | head over base, paired |
|---|--:|--:|--:|
| 1 thread | 1h15m ± 8m32s | 1h16m ± 8m31s | +0.7% (−0.0%, +1.3%) |
| 4 threads | 39m19s ± 5m32s | 39m04s ± 5m30s | −0.6% (−2.2%, +1.0%) |
| 4 threads, the sample's overlap gap as a fixed cost | 28m31s ± 4m03s | 28m09s ± 3m58s |  |
| the scanner alone (its serial part) | 15m08s ± 32.2s | 15m52s ± 19.7s |  |
| the workers alone, over 4 | 28m31s ± 4m03s | 28m09s ± 3m58s |  |

At 4 threads the workers bound it. Per byte: the scanner 9.2 ns a byte of bz2, the workers 3.69 ns a byte of output at 4 threads (CPU, summed over them), 1.99 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (head).

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
| `text` | 2,624,130 | 13,234,387 | `7e341c4d8eb43c51…` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 |

- CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3243.0 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1
- rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source)
- bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4

</details>
