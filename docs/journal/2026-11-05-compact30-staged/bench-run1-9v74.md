<!-- mibz2-bench -->
## Benchmark: mibz2 decoder

**head `ba94ff5` against base `448d23d`** (the merge base with master): 10 paired rounds on one runner · [run](https://github.com/lmmx/mibz2/actions/runs/42036285431)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.1

Each round decodes every corpus once with each build at each thread count (`ab run`: the base's harness binary as the arm `mibz2@base/ab`, so both builds run in the same process shape), the builds' order alternating. **Change** is head over base: the geometric mean of the rounds' paired ratios and its 95% interval; − is faster, and **faster** or **slower** where the interval excludes no change. 1-thread runs are pinned to one CPU.

| corpus | threads | wall, base → head (median) | wall change | CPU change | peak RSS, base → head | head MB/s | same bytes |
|---|--:|--:|--:|--:|--:|--:|---|
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.555 → 0.620 s | +11.4% (+10.1%, +12.7%) **slower** | +11.4% (+10.1%, +12.7%) **slower** | 25 → 24 MB (−5.3% **less**) | 338 | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.300 → 0.329 s | +9.1% (+8.1%, +10.2%) **slower** | +9.3% (+8.1%, +10.5%) **slower** | 49 → 45 MB (−7.0% **less**) | 638 | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.246 → 0.260 s | +4.8% (+3.2%, +6.4%) **slower** | +7.0% (+5.9%, +8.2%) **slower** | 77 → 71 MB (−7.7% **less**) | 808 | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 → 0.018 s | −1.7% (−3.8%, +0.4%) | −1.5% (−3.8%, +0.7%) | 33 → 32 MB (−2.3% **less**) | 1,130 | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.018 → 0.018 s | −2.0% (−4.1%, +0.1%) | −1.8% (−3.7%, +0.2%) | 32 → 32 MB (−2.2% **less**) | 1,130 | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.018 → 0.018 s | +3.5% (−7.4%, +15.6%) | +3.4% (−6.9%, +14.8%) | 32 → 32 MB (−2.2% **less**) | 1,111 | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.028 → 0.030 s | +5.2% (+4.6%, +5.8%) **slower** | +4.8% (+4.2%, +5.4%) **slower** | 16 → 14 MB (−9.9% **less**) | 656 | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.018 → 0.018 s | −0.7% (−7.2%, +6.2%) | +1.8% (−0.3%, +4.0%) | 22 → 18 MB (−14.5% **less**) | 1,114 | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.021 → 0.020 s | −3.9% (−8.2%, +0.6%) | −7.4% (−12.6%, −2.0%) **faster** | 38 → 33 MB (−12.0% **less**) | 1,000 | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.120 → 0.125 s | +8.7% (−2.0%, +20.6%) | +8.5% (−2.0%, +20.3%) | 22 → 21 MB (−6.6% **less**) | 64 | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.073 → 0.074 s | +2.9% (+0.4%, +5.5%) **slower** | +4.7% (+1.7%, +7.8%) **slower** | 39 → 34 MB (−11.3% **less**) | 108 | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.058 → 0.058 s | +2.1% (−1.5%, +5.8%) | +2.1% (−1.1%, +5.4%) | 50 → 44 MB (−11.4% **less**) | 139 | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.154 → 0.159 s | +3.8% (+3.1%, +4.6%) **slower** | +3.8% (+3.0%, +4.5%) **slower** | 23 → 21 MB (−6.4% **less**) | 60 | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.089 → 0.092 s | +2.6% (+0.1%, +5.1%) **slower** | +1.8% (−0.4%, +4.2%) | 42 → 39 MB (−5.6% **less**) | 105 | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.071 → 0.071 s | +1.0% (−3.0%, +5.1%) | +0.5% (−1.9%, +3.0%) | 53 → 47 MB (−11.5% **less**) | 135 | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 → 0.012 s | +3.0% (+0.8%, +5.2%) **slower** | +2.7% (+0.6%, +4.8%) **slower** | 22 → 22 MB (−1.0% **less**) | 826 | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 → 0.012 s | +0.2% (−1.8%, +2.4%) | +0.4% (−1.3%, +2.2%) | 23 → 22 MB (−1.4% **less**) | 803 | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 → 0.013 s | +0.9% (−1.5%, +3.5%) | +0.8% (−1.2%, +3.0%) | 23 → 22 MB (−1.2% **less**) | 778 | yes |
| `text` (1.98 MB → 12.5 MB, 6.3×) | 1 | 0.069 → 0.078 s | +13.2% (+12.9%, +13.5%) **slower** | +12.9% (+12.6%, +13.2%) **slower** | 18 → 17 MB (−8.2% **less**) | 160 | yes |
| `text` (1.98 MB → 12.5 MB, 6.3×) | 2 | 0.043 → 0.047 s | +7.9% (+3.4%, +12.6%) **slower** | +8.9% (+6.8%, +11.1%) **slower** | 30 → 27 MB (−9.2% **less**) | 265 | yes |
| `text` (1.98 MB → 12.5 MB, 6.3×) | 4 | 0.037 → 0.039 s | +4.0% (+1.0%, +7.2%) **slower** | +6.7% (+4.8%, +8.7%) **slower** | 49 → 43 MB (−11.3% **less**) | 320 | yes |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

### Resources: CPU per byte and scaling, the `wikidata` corpus

**Assessment:** A trade-off: better contention sensitivity, memory; worse speed, CPU cost. Neither build dominates: weigh it for the deployment (wikidata-pq's release).

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | base MB/s | base CPU ns/B | base amplification | base efficiency | head MB/s | head CPU ns/B | head amplification | head efficiency | amplification, head over base |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| 1: pinned to CPU 1 | 378 | 2.66 | 1 | – | 338 | 2.97 | 1 | – |  |
| 2: CPUs 0,2, one a core | 699 | 2.82 | 1.06× (1.04–1.08) | 93% | 638 | 3.10 | 1.04× (1.03–1.05) | 95% | −1.9% (−3.5%, −0.2%) **less** |
| 4: all 4 CPUs, sharing SMT siblings | 854 | 4.44 | 1.67× (1.64–1.70) | 56% | 808 | 4.78 | 1.60× (1.59–1.62) | 60% | −3.9% (−5.3%, −2.5%) **less** |

**Best:** base at 4 threads (854 MB/s); head at 4 threads (808 MB/s); head's best against base's, wall: +4.8% (+3.2%, +6.4%) **slower**.

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

<details><summary>How to read it, and what it cannot show</summary>

- **The four comparisons the assessment reads,** declared in bench/report.py (`assessment`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run.
- **A trade-off is not a failure.** Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting.
- **What it can't show:** which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, #39). The cores' L3 is shared with other tenants and its size unknown.
- **Run-queue wait** (threads' mean count runnable, not running; schedstat): base 1t 0.00, base 2t 0.71, base 4t 1.10, head 1t 0.00, head 2t 0.66, head 4t 1.03. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference.
- **The per-thread split's unattributed CPU** (the process's total that no sampled thread holds, sampled every 10 ms): base 2t 8%, base 4t 9%, head 2t 9%, head 4t 9%.

</details>

### Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
|---|--:|--:|--:|--:|--:|
| head | 0.09 | 0.80 | 0.01 | 0.09 | 0.260 |
| base | 0.09 | 0.74 | 0.02 | 0.08 | 0.246 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

### Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs each build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU, the builds' order alternating: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2596 MHz). **Change** is head over base per stage, paired by round as above; with seven intervals a run, a mark on one stage alone can be chance (about one run in three). Diagnostic only: the decoder's own figures are the tables above. 10 runs, 1m49s of the run.

| stage | base → head c/B (median) | head share | change |
|---|--:|--:|--:|
| scanner (headers, Huffman) | 0.82 → 0.81 | 12.3% | −0.9% (−1.2%, −0.5%) **faster** |
| MTF + RLE2 | 1.41 → 1.41 | 21.3% | +0.0% (−0.4%, +0.4%) |
| T-vector build | 1.24 → 1.25 | 18.9% | +0.5% (−0.4%, +1.3%) |
| walk (the one each build uses) | 2.87 → 2.83 | 42.8% | −0.7% (−1.8%, +0.4%) |
| of which: segment assembly | 0.10 → 0.10 | 1.5% | −0.3% (−2.0%, +1.5%) |
| RLE1 | 0.14 → 0.14 | 2.0% | −4.9% (−6.7%, −3.0%) **faster** |
| CRC | 0.18 → 0.18 | 2.7% | −0.3% (−1.2%, +0.6%) |
| **total** | 6.66 → 6.62 |  | −0.4% (−1.1%, +0.3%) |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: base, “walk, room (used)”; head, “walk, room (used)”.

### Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

| | base | head | head over base, paired |
|---|--:|--:|--:|
| 1 thread | 1h21m ± 9m38s | 1h30m ± 10m54s | +11.4% (+10.1%, +12.7%) **slower** |
| 4 threads | 36m03s ± 5m06s | 37m46s ± 5m18s | +4.8% (+3.2%, +6.4%) **slower** |
| 4 threads, the sample's overlap gap as a fixed cost | 27m06s ± 3m51s | 29m19s ± 4m07s |  |
| the scanner alone (its serial part) | 12m48s ± 26.3s | 13m06s ± 0.00s |  |
| the workers alone, over 4 | 27m06s ± 3m51s | 29m19s ± 4m07s |  |

At 4 threads the workers bound it. Per byte: the scanner 7.6 ns a byte of bz2, the workers 3.84 ns a byte of output at 4 threads (CPU, summed over them), 2.54 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (head).

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
| `text` | 1,983,409 | 12,512,891 | `2c4ca073b2783816…` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 |

- CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 2596.1 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1
- rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source)
- bench 3.2.1 (bench/report.py), 10 rounds, threads 1, 2, 4

</details>
