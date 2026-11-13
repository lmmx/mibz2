<!-- mibz2-bench -->
## Benchmark: mibz2 decoder

**head `cb73bf9` against base `e674070`** (the merge base with master): 10 paired rounds on one runner · [run](https://github.com/lmmx/mibz2/actions/runs/43404843443)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each build at each thread count (`ab run`: the base's harness binary as the arm `mibz2@base/ab`, so both builds run in the same process shape), the builds' order alternating. **Change** is head over base: the geometric mean of the rounds' paired ratios and its 95% interval; − is faster, and **faster** or **slower** where the interval excludes no change. 1-thread runs are pinned to one CPU.

| corpus | threads | wall, base → head (median) | wall change | CPU change | peak RSS, base → head | head MB/s | same bytes |
|---|--:|--:|--:|--:|--:|--:|---|
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.473 → 0.473 s | −0.1% (−2.4%, +2.3%) | −0.1% (−2.4%, +2.2%) | 26 → 26 MB (+0.0%) | 443 | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.280 → 0.291 s | +4.6% (−2.7%, +12.4%) | +4.3% (−2.9%, +12.0%) | 52 → 52 MB (+2.0%) | 720 | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.278 → 0.274 s | +1.0% (−4.1%, +6.5%) | −0.5% (−4.4%, +3.5%) | 79 → 81 MB (+2.1% **more**) | 764 | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 → 0.019 s | −3.8% (−20.4%, +16.2%) | −2.6% (−19.1%, +17.3%) | 33 → 33 MB (−0.1%) | 1,075 | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.019 → 0.019 s | −0.0% (−1.8%, +1.8%) | +0.0% (−1.7%, +1.8%) | 33 → 33 MB (+0.1%) | 1,039 | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 → 0.019 s | −1.0% (−3.6%, +1.7%) | −1.0% (−3.4%, +1.6%) | 33 → 33 MB (−0.1%) | 1,047 | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.028 → 0.028 s | −0.1% (−2.5%, +2.4%) | −0.1% (−2.4%, +2.3%) | 16 → 16 MB (+0.3%) | 710 | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.020 → 0.019 s | −4.1% (−12.9%, +5.7%) | −1.7% (−4.4%, +1.2%) | 23 → 22 MB (−3.1%) | 1,005 | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.022 → 0.022 s | +1.4% (−2.4%, +5.4%) | +0.7% (−2.8%, +4.3%) | 39 → 39 MB (+1.4%) | 883 | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.117 → 0.116 s | −1.1% (−2.9%, +0.7%) | −1.2% (−3.0%, +0.7%) | 23 → 23 MB (−0.2%) | 69 | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.074 → 0.073 s | −0.7% (−4.5%, +3.2%) | −0.8% (−3.6%, +2.0%) | 39 → 38 MB (−3.4%) | 109 | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.059 → 0.058 s | −0.6% (−3.0%, +1.9%) | +0.4% (−1.2%, +2.0%) | 50 → 50 MB (+0.8%) | 137 | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.126 → 0.125 s | −1.3% (−4.2%, +1.6%) | −1.5% (−4.6%, +1.7%) | 23 → 23 MB (−0.1%) | 77 | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.079 → 0.077 s | −0.8% (−3.4%, +1.8%) | −1.0% (−3.9%, +1.9%) | 41 → 43 MB (+3.3%) | 125 | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.066 → 0.066 s | +1.3% (−3.4%, +6.3%) | +3.4% (−1.3%, +8.3%) | 54 → 54 MB (+0.0%) | 145 | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 → 0.012 s | −0.8% (−3.1%, +1.6%) | −1.7% (−5.6%, +2.2%) | 23 → 23 MB (−0.1%) | 820 | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 → 0.013 s | +1.4% (−1.2%, +4.2%) | +0.8% (−1.7%, +3.3%) | 24 → 24 MB (+0.0%) | 775 | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 → 0.013 s | −0.6% (−2.3%, +1.2%) | −0.5% (−2.1%, +1.2%) | 23 → 23 MB (+0.4%) | 800 | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 1 | 0.062 → 0.063 s | +0.8% (−0.6%, +2.2%) | +0.7% (−0.6%, +2.1%) | 19 → 19 MB (−0.2%) | 207 | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 2 | 0.044 → 0.044 s | −0.2% (−3.9%, +3.7%) | +1.6% (−2.6%, +5.9%) | 34 → 35 MB (+1.3%) | 296 | yes |
| `text` (2.6 MB → 13 MB, 5.0×) | 4 | 0.040 → 0.041 s | +0.6% (−2.1%, +3.4%) | −0.0% (−2.9%, +2.9%) | 54 → 54 MB (+0.4%) | 320 | yes |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

### Resources: CPU per byte and scaling, the `wikidata` corpus

**Assessment:** Worse (memory), and no measure better.
**Needs a machine with more cores before a release:** peak RSS grew. This runner has 2 physical cores; a change in CPU amplification or in memory can grow with the cores (the 10-core workstation's knee: #39, #65).

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | base MB/s | base CPU ns/B | base amplification | base efficiency | head MB/s | head CPU ns/B | head amplification | head efficiency | amplification, head over base |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| 1: pinned to CPU 1 | 444 | 2.26 | 1 | – | 443 | 2.26 | 1 | – |  |
| 2: CPUs 0,2, one a core | 748 | 2.62 | 1.16× (1.13–1.20) | 84% | 720 | 2.73 | 1.21× (1.14–1.30) | 80% | +4.4% (−3.6%, +13.0%) |
| 4: all 4 CPUs, sharing SMT siblings | 755 | 5.00 | 2.18× (2.10–2.26) | 43% | 764 | 4.79 | 2.17× (2.09–2.25) | 43% | −0.4% (−5.8%, +5.2%) |

**Best:** base at 4 threads (755 MB/s); head at 4 threads (764 MB/s); head's best against base's, wall: +1.0% (−4.1%, +6.5%).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

<details><summary>How to read it, and what it cannot show</summary>

- **The four comparisons the assessment reads,** declared in bench/report.py (`assessment`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run.
- **A trade-off is not a failure.** Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting.
- **What it can't show:** which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, #39). The cores' L3 is shared with other tenants and its size unknown.
- **Run-queue wait** (threads' mean count runnable, not running; schedstat): base 1t 0.00, base 2t 0.73, base 4t 1.03, head 1t 0.00, head 2t 0.71, head 4t 1.07. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference.
- **The per-thread split's unattributed CPU** (the process's total that no sampled thread holds, sampled every 10 ms): base 2t 9%, base 4t 8%, head 2t 10%, head 4t 8%.

</details>

### Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
|---|--:|--:|--:|--:|--:|
| head | 0.08 | 0.82 | 0.02 | 0.09 | 0.274 |
| base | 0.08 | 0.85 | 0.02 | 0.08 | 0.278 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

### Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs each build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU, the builds' order alternating: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3696 MHz). **Change** is head over base per stage, paired by round as above; with seven intervals a run, a mark on one stage alone can be chance (about one run in three). Diagnostic only: the decoder's own figures are the tables above. 10 runs, 2m00s of the run.

| stage | base → head c/B (median) | head share | change |
|---|--:|--:|--:|
| scanner (headers, Huffman) | 1.12 → 1.12 | 12.1% | −0.1% (−1.2%, +0.9%) |
| MTF + RLE2 | 1.60 → 1.60 | 17.3% | −2.5% (−11.1%, +7.0%) |
| T-vector build | 1.61 → 1.57 | 16.9% | −1.6% (−3.9%, +0.7%) |
| walk (the one each build uses) | 4.56 → 4.47 | 48.3% | −1.8% (−4.8%, +1.2%) |
| of which: segment assembly | 0.24 → 0.20 | 2.2% | −6.9% (−16.7%, +4.0%) |
| RLE1 | 0.20 → 0.20 | 2.1% | +0.8% (−2.0%, +3.6%) |
| CRC | 0.26 → 0.26 | 2.8% | −0.2% (−1.2%, +0.8%) |
| **total** | 9.35 → 9.27 |  | −1.6% (−4.9%, +1.8%) |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: base, “walk, room (used)”; head, “walk, room (used)”.

### Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

| | base | head | head over base, paired |
|---|--:|--:|--:|
| 1 thread | 1h09m ± 8m07s | 1h09m ± 8m02s | −0.1% (−2.4%, +2.3%) |
| 4 threads | 40m07s ± 5m46s | 40m34s ± 6m00s | +1.0% (−4.1%, +6.5%) |
| 4 threads, the sample's overlap gap as a fixed cost | 30m51s ± 4m30s | 30m33s ± 4m37s |  |
| the scanner alone (its serial part) | 12m13s ± 32.2s | 12m05s ± 30.2s |  |
| the workers alone, over 4 | 30m51s ± 4m30s | 30m33s ± 4m37s |  |

At 4 threads the workers bound it. Per byte: the scanner 7.0 ns a byte of bz2, the workers 4.01 ns a byte of output at 4 threads (CPU, summed over them), 1.87 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (head).

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

- CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3696.0 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1
- rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source)
- bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4

</details>

