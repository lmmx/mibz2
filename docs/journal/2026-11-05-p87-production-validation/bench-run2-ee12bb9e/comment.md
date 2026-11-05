<!-- mibz2-bench -->
## Benchmark: mibz2 decoder

**head `ee12bb9` against base `08e7fc3`** (the merge base with master): 10 paired rounds on one runner · [run](https://github.com/lmmx/mibz2/actions/runs/41941330350)

Intel(R) Xeon(R) 6975P-C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.0

Each round decodes every corpus once with each build at each thread count (`ab run`: the base's harness binary as the arm `mibz2@base/ab`, so both builds run in the same process shape), the builds' order alternating. **Change** is head over base: the geometric mean of the rounds' paired ratios and its 95% interval; − is faster, and **faster** or **slower** where the interval excludes no change. 1-thread runs are pinned to one CPU.

| corpus | threads | wall, base → head (median) | wall change | CPU change | peak RSS, base → head | head MB/s | same bytes |
|---|--:|--:|--:|--:|--:|--:|---|
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.654 → 0.655 s | +0.0% (−0.2%, +0.2%) | +0.0% (−0.2%, +0.3%) | 26 → 26 MB (−3.2% **less**) | 320 | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.384 → 0.377 s | −1.1% (−4.8%, +2.8%) | −0.2% (−1.4%, +1.0%) | 49 → 45 MB (−6.3% **less**) | 557 | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.304 → 0.303 s | −1.1% (−2.3%, +0.2%) | −0.7% (−1.9%, +0.5%) | 71 → 66 MB (−6.8% **less**) | 691 | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.016 → 0.016 s | −1.2% (−8.3%, +6.5%) | −0.8% (−7.7%, +6.7%) | 33 → 33 MB (−1.2% **less**) | 1,270 | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.016 → 0.016 s | +0.7% (−7.1%, +9.2%) | +1.1% (−5.9%, +8.7%) | 33 → 32 MB (−1.3% **less**) | 1,270 | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.016 → 0.016 s | +0.7% (−4.3%, +6.0%) | +0.7% (−4.1%, +5.8%) | 33 → 32 MB (−1.1% **less**) | 1,242 | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 → 0.034 s | −0.2% (−2.9%, +2.6%) | −0.5% (−3.3%, +2.3%) | 17 → 16 MB (−5.2% **less**) | 582 | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.021 → 0.019 s | −3.4% (−13.7%, +8.0%) | −1.1% (−3.3%, +1.2%) | 23 → 22 MB (−8.1% **less**) | 1,013 | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.021 → 0.021 s | −4.2% (−10.3%, +2.4%) | +2.7% (−4.3%, +10.2%) | 42 → 39 MB (−7.7% **less**) | 958 | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.118 → 0.118 s | +0.8% (−1.7%, +3.4%) | +0.8% (−1.8%, +3.4%) | 23 → 22 MB (−3.7% **less**) | 68 | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.068 → 0.069 s | +1.3% (−2.0%, +4.8%) | −0.0% (−1.2%, +1.2%) | 37 → 35 MB (−5.5% **less**) | 116 | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.055 → 0.055 s | −0.8% (−4.0%, +2.5%) | −0.1% (−3.2%, +3.1%) | 53 → 49 MB (−7.3% **less**) | 146 | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.148 → 0.147 s | −2.7% (−9.1%, +4.1%) | −2.6% (−8.9%, +4.0%) | 24 → 23 MB (−3.7% **less**) | 65 | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.084 → 0.084 s | −0.8% (−3.5%, +2.0%) | −0.1% (−1.0%, +0.7%) | 39 → 37 MB (−4.8% **less**) | 115 | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.065 → 0.065 s | −1.1% (−4.0%, +2.0%) | −2.0% (−3.3%, −0.7%) **faster** | 56 → 51 MB (−8.3% **less**) | 148 | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.011 → 0.010 s | −1.4% (−7.1%, +4.6%) | −1.5% (−7.2%, +4.4%) | 23 → 22 MB (−1.1% **less**) | 990 | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.011 → 0.011 s | +1.2% (−3.8%, +6.4%) | +0.2% (−4.6%, +5.3%) | 23 → 23 MB (−1.0% **less**) | 935 | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 → 0.011 s | +0.8% (−1.8%, +3.4%) | +0.9% (−2.1%, +4.0%) | 23 → 23 MB (−1.0% **less**) | 885 | yes |
| `text` (1.69 MB → 12.4 MB, 7.3×) | 1 | 0.069 → 0.069 s | +0.1% (−0.9%, +1.2%) | +0.2% (−0.9%, +1.2%) | 19 → 18 MB (−4.7% **less**) | 181 | yes |
| `text` (1.69 MB → 12.4 MB, 7.3×) | 2 | 0.040 → 0.039 s | −0.8% (−3.6%, +2.0%) | −0.8% (−2.2%, +0.7%) | 32 → 30 MB (−7.1% **less**) | 316 | yes |
| `text` (1.69 MB → 12.4 MB, 7.3×) | 4 | 0.032 → 0.032 s | −2.3% (−5.8%, +1.4%) | +0.0% (−2.8%, +2.9%) | 52 → 48 MB (−7.5% **less**) | 387 | yes |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

### Resources: CPU per byte and scaling, the `wikidata` corpus

**Assessment:** Better (memory), and no measure worse.

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | base MB/s | base CPU ns/B | base amplification | base efficiency | head MB/s | head CPU ns/B | head amplification | head efficiency | amplification, head over base |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| 1: pinned to CPU 1 | 320 | 3.13 | 1 | – | 320 | 3.13 | 1 | – |  |
| 2: CPUs 0,1, one a core | 546 | 3.19 | 1.02× (1.01–1.03) | 85% | 557 | 3.19 | 1.02× (1.02–1.02) | 86% | −0.3% (−1.3%, +0.8%) |
| 4: all 4 CPUs, sharing SMT siblings | 690 | 4.74 | 1.51× (1.50–1.53) | 53% | 691 | 4.70 | 1.50× (1.49–1.52) | 54% | −0.8% (−2.0%, +0.5%) |

**Best:** base at 4 threads (690 MB/s); head at 4 threads (691 MB/s); head's best against base's, wall: −1.1% (−2.3%, +0.2%).

Conditions: 4 CPUs on 2 physical cores (0,2 | 1,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,1 (one CPU a core); 4-thread runs share cores' SMT siblings.

<details><summary>How to read it, and what it cannot show</summary>

- **The four comparisons the assessment reads,** declared in bench/report.py (`assessment`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run.
- **A trade-off is not a failure.** Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting.
- **What it can't show:** which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, #39). The cores' L3 is shared with other tenants and its size unknown.
- **Run-queue wait** (threads' mean count runnable, not running; schedstat): base 1t 0.00, base 2t 0.73, base 4t 1.05, head 1t 0.00, head 2t 0.74, head 4t 0.99. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference.
- **The per-thread split's unattributed CPU** (the process's total that no sampled thread holds, sampled every 10 ms): base 2t 7%, base 4t 9%, head 2t 8%, head 4t 8%.

</details>

### Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
|---|--:|--:|--:|--:|--:|
| head | 0.08 | 0.81 | 0.02 | 0.08 | 0.303 |
| base | 0.08 | 0.81 | 0.02 | 0.09 | 0.304 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

### Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs each build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU, the builds' order alternating: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3896 MHz). **Change** is head over base per stage, paired by round as above; with seven intervals a run, a mark on one stage alone can be chance (about one run in three). Diagnostic only: the decoder's own figures are the tables above. 10 runs, 2m04s of the run.

| stage | base → head c/B (median) | head share | change |
|---|--:|--:|--:|
| scanner (headers, Huffman) | 1.24 → 1.24 | 10.6% | −0.4% (−1.1%, +0.2%) |
| MTF + RLE2 | 2.19 → 2.19 | 18.7% | −0.1% (−0.8%, +0.6%) |
| T-vector build | 1.71 → 1.70 | 14.6% | −0.1% (−0.5%, +0.2%) |
| walk (the one each build uses) | 6.15 → 6.15 | 52.7% | −0.1% (−0.5%, +0.3%) |
| of which: segment assembly | 0.32 → 0.32 | 2.7% | −0.3% (−1.0%, +0.5%) |
| RLE1 | 0.25 → 0.26 | 2.2% | +3.9% (+3.1%, +4.8%) **slower** |
| CRC | 0.14 → 0.14 | 1.2% | −1.2% (−3.1%, +0.8%) |
| **total** | 11.68 → 11.67 |  | −0.1% (−0.3%, +0.2%) |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: base, “walk, room (used)”; head, “walk, room (used)”.

### Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

| | base | head | head over base, paired |
|---|--:|--:|--:|
| 1 thread | 1h35m ± 11m40s | 1h35m ± 11m42s | +0.0% (−0.2%, +0.2%) |
| 4 threads | 44m34s ± 6m16s | 44m05s ± 6m11s | −1.1% (−2.3%, +0.2%) |
| 4 threads, the sample's overlap gap as a fixed cost | 29m19s ± 4m08s | 29m06s ± 4m08s |  |
| the scanner alone (its serial part) | 11m56s ± 26.3s | 11m38s ± 0.00s |  |
| the workers alone, over 4 | 29m19s ± 4m08s | 29m06s ± 4m08s |  |

At 4 threads the workers bound it. Per byte: the scanner 6.7 ns a byte of bz2, the workers 3.81 ns a byte of output at 4 threads (CPU, summed over them), 2.74 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (head).

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
| `text` | 1,690,460 | 12,385,319 | `42f88d7c5a17e825…` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 |

- CPU: Intel(R) Xeon(R) 6975P-C, 4 cores, reported clock 3896.3 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4_1, ssse3; kernel 7.0.0-1016-aws; image ubuntu26 20261101.64.1
- rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source)
- bench 3.2.0 (bench/report.py), 10 rounds, threads 1, 2, 4

</details>
