<!-- mibz2-bench -->
## Benchmark: mibz2 decoder

**head `ef28ad3` against base `08e7fc3`** (the merge base with master): 10 paired rounds on one runner · [run](https://github.com/lmmx/mibz2/actions/runs/41940471497)

INTEL(R) XEON(R) PLATINUM 8573C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.0

Each round decodes every corpus once with each build at each thread count (`ab run`: the base's harness binary as the arm `mibz2@base/ab`, so both builds run in the same process shape), the builds' order alternating. **Change** is head over base: the geometric mean of the rounds' paired ratios and its 95% interval; − is faster, and **faster** or **slower** where the interval excludes no change. 1-thread runs are pinned to one CPU.

| corpus | threads | wall, base → head (median) | wall change | CPU change | peak RSS, base → head | head MB/s | same bytes |
|---|--:|--:|--:|--:|--:|--:|---|
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.858 → 0.858 s | +0.0% (−0.6%, +0.7%) | +0.0% (−0.6%, +0.7%) | 26 → 25 MB (−3.4% **less**) | 244 | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.507 → 0.504 s | −0.2% (−1.8%, +1.4%) | −0.2% (−1.1%, +0.6%) | 48 → 45 MB (−7.8% **less**) | 416 | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.402 → 0.394 s | −2.2% (−4.0%, −0.5%) **faster** | −1.3% (−2.9%, +0.3%) | 71 → 66 MB (−6.8% **less**) | 532 | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.022 → 0.022 s | −0.5% (−1.5%, +0.6%) | +0.2% (−1.0%, +1.4%) | 33 → 33 MB (−1.2% **less**) | 926 | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.022 → 0.022 s | +0.0% (−1.9%, +2.0%) | +0.0% (−1.9%, +2.0%) | 33 → 32 MB (−1.2% **less**) | 920 | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.022 → 0.022 s | −0.9% (−2.2%, +0.4%) | −0.8% (−2.1%, +0.6%) | 33 → 32 MB (−1.2% **less**) | 922 | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.047 → 0.047 s | −0.2% (−1.0%, +0.6%) | −0.1% (−1.0%, +0.7%) | 17 → 16 MB (−5.1% **less**) | 421 | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.028 → 0.028 s | −3.0% (−7.5%, +1.7%) | −1.6% (−3.2%, +0.1%) | 23 → 22 MB (−7.8% **less**) | 694 | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.029 → 0.028 s | −3.0% (−10.5%, +5.1%) | +1.4% (−2.7%, +5.8%) | 42 → 38 MB (−7.6% **less**) | 706 | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.167 → 0.167 s | +6.4% (−3.8%, +17.7%) | +6.3% (−3.8%, +17.5%) | 23 → 22 MB (−3.6% **less**) | 48 | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.098 → 0.096 s | −2.2% (−4.2%, −0.2%) **faster** | −1.8% (−2.9%, −0.7%) **faster** | 36 → 35 MB (−4.6% **less**) | 84 | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.077 → 0.077 s | −1.4% (−5.9%, +3.4%) | −2.0% (−3.6%, −0.3%) **faster** | 53 → 49 MB (−6.2% **less**) | 104 | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.205 → 0.205 s | +2.6% (−3.6%, +9.3%) | +2.6% (−3.6%, +9.2%) | 24 → 23 MB (−3.5% **less**) | 47 | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.117 → 0.116 s | −0.7% (−2.6%, +1.3%) | −0.6% (−1.6%, +0.5%) | 39 → 37 MB (−3.8% **less**) | 83 | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.093 → 0.093 s | −0.8% (−3.4%, +1.9%) | −2.6% (−3.9%, −1.3%) **faster** | 56 → 52 MB (−6.7% **less**) | 103 | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.015 → 0.015 s | −0.8% (−1.6%, +0.1%) | −0.9% (−1.8%, −0.0%) **faster** | 23 → 23 MB (−1.1% **less**) | 690 | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 → 0.015 s | +1.6% (−2.9%, +6.2%) | −0.3% (−1.4%, +0.8%) | 23 → 23 MB (−1.4% **less**) | 664 | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 → 0.015 s | +0.3% (−1.1%, +1.8%) | +0.4% (−0.9%, +1.7%) | 23 → 23 MB (−1.1% **less**) | 664 | yes |
| `text` (1.69 MB → 12.4 MB, 7.3×) | 1 | 0.090 → 0.091 s | +0.2% (−0.8%, +1.2%) | +0.2% (−0.8%, +1.2%) | 19 → 18 MB (−4.6% **less**) | 137 | yes |
| `text` (1.69 MB → 12.4 MB, 7.3×) | 2 | 0.054 → 0.053 s | +0.0% (−3.9%, +4.0%) | −0.1% (−0.8%, +0.6%) | 33 → 31 MB (−6.8% **less**) | 232 | yes |
| `text` (1.69 MB → 12.4 MB, 7.3×) | 4 | 0.046 → 0.045 s | −1.8% (−4.1%, +0.4%) | −1.4% (−3.2%, +0.4%) | 54 → 50 MB (−7.3% **less**) | 275 | yes |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

### Resources: CPU per byte and scaling, the `wikidata` corpus

**Assessment:** Better (speed, memory), and no measure worse.

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | base MB/s | base CPU ns/B | base amplification | base efficiency | head MB/s | head CPU ns/B | head amplification | head efficiency | amplification, head over base |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| 1: pinned to CPU 1 | 244 | 4.10 | 1 | – | 244 | 4.10 | 1 | – |  |
| 2: CPUs 0,2, one a core | 414 | 4.37 | 1.06× (1.04–1.07) | 85% | 416 | 4.34 | 1.05× (1.04–1.06) | 85% | −0.3% (−1.4%, +0.8%) |
| 4: all 4 CPUs, sharing SMT siblings | 522 | 6.60 | 1.61× (1.58–1.64) | 53% | 532 | 6.49 | 1.59× (1.56–1.61) | 54% | −1.4% (−3.0%, +0.3%) |

**Best:** base at 4 threads (522 MB/s); head at 4 threads (532 MB/s); head's best against base's, wall: −2.2% (−4.0%, −0.5%) **faster**.

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

<details><summary>How to read it, and what it cannot show</summary>

- **The four comparisons the assessment reads,** declared in bench/report.py (`assessment`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run.
- **A trade-off is not a failure.** Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting.
- **What it can't show:** which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, #39). The cores' L3 is shared with other tenants and its size unknown.
- **Run-queue wait** (threads' mean count runnable, not running; schedstat): base 1t 0.00, base 2t 0.67, base 4t 0.92, head 1t 0.00, head 2t 0.68, head 4t 0.94. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference.
- **The per-thread split's unattributed CPU** (the process's total that no sampled thread holds, sampled every 10 ms): base 2t 6%, base 4t 6%, head 2t 6%, head 4t 7%.

</details>

### Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
|---|--:|--:|--:|--:|--:|
| head | 0.13 | 1.11 | 0.03 | 0.09 | 0.394 |
| base | 0.13 | 1.15 | 0.03 | 0.09 | 0.402 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

### Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs each build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU, the builds' order alternating: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3000 MHz). **Change** is head over base per stage, paired by round as above; with seven intervals a run, a mark on one stage alone can be chance (about one run in three). Diagnostic only: the decoder's own figures are the tables above. 10 runs, 2m26s of the run.

| stage | base → head c/B (median) | head share | change |
|---|--:|--:|--:|
| scanner (headers, Huffman) | 1.27 → 1.27 | 11.0% | −0.2% (−0.6%, +0.3%) |
| MTF + RLE2 | 2.24 → 2.24 | 19.4% | +0.4% (−0.5%, +1.2%) |
| T-vector build | 1.74 → 1.74 | 15.1% | +0.2% (−0.3%, +0.7%) |
| walk (the one each build uses) | 5.89 → 5.89 | 51.0% | +2.2% (−0.3%, +4.7%) |
| of which: segment assembly | 0.29 → 0.28 | 2.5% | +0.6% (−0.7%, +1.9%) |
| RLE1 | 0.27 → 0.28 | 2.4% | +2.6% (+2.0%, +3.2%) **slower** |
| CRC | 0.14 → 0.14 | 1.2% | −0.1% (−1.7%, +1.5%) |
| **total** | 11.54 → 11.55 |  | +1.2% (+0.1%, +2.4%) **slower** |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: base, “walk, room (used)”; head, “walk, room (used)”.

### Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

| | base | head | head over base, paired |
|---|--:|--:|--:|
| 1 thread | 2h05m ± 15m00s | 2h05m ± 15m00s | +0.0% (−0.6%, +0.7%) |
| 4 threads | 58m54s ± 8m21s | 57m34s ± 8m08s | −2.2% (−4.0%, −0.5%) **faster** |
| 4 threads, the sample's overlap gap as a fixed cost | 41m52s ± 5m56s | 41m00s ± 5m51s |  |
| the scanner alone (its serial part) | 18m29s ± 42.1s | 18m29s ± 42.1s |  |
| the workers alone, over 4 | 41m52s ± 5m56s | 41m00s ± 5m51s |  |

At 4 threads the workers bound it. Per byte: the scanner 10.7 ns a byte of bz2, the workers 5.37 ns a byte of output at 4 threads (CPU, summed over them), 3.50 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (head).

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
| `text` | 1,687,702 | 12,373,991 | `81b3b27d361a5cf4…` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 |

- CPU: INTEL(R) XEON(R) PLATINUM 8573C, 4 cores, reported clock 3000.0 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1
- rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source)
- bench 3.2.0 (bench/report.py), 10 rounds, threads 1, 2, 4

</details>
