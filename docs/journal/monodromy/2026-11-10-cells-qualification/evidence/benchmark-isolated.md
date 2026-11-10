<!-- mibz2-bench -->
## Benchmark: mibz2 decoder

**head `989f782` against base `edaa984`** (the merge base with master): 10 paired rounds on one runner · [run](https://github.com/lmmx/mibz2/actions/runs/42899341988)

Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.1

Each round decodes every corpus once with each build at each thread count (`ab run`: the base's harness binary as the arm `mibz2@base/ab`, so both builds run in the same process shape), the builds' order alternating. **Change** is head over base: the geometric mean of the rounds' paired ratios and its 95% interval; − is faster, and **faster** or **slower** where the interval excludes no change. 1-thread runs are pinned to one CPU.

| corpus | threads | wall, base → head (median) | wall change | CPU change | peak RSS, base → head | head MB/s | same bytes |
|---|--:|--:|--:|--:|--:|--:|---|
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.816 → 0.811 s | −0.5% (−2.5%, +1.6%) | −0.5% (−2.5%, +1.6%) | 26 → 26 MB (−0.1%) | 258 | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.437 → 0.429 s | −0.8% (−4.0%, +2.5%) | −0.6% (−3.5%, +2.4%) | 52 → 51 MB (−1.0%) | 488 | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.394 → 0.403 s | +0.0% (−2.6%, +2.7%) | −0.2% (−3.1%, +2.9%) | 78 → 77 MB (+0.2%) | 520 | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.022 → 0.022 s | −4.3% (−13.0%, +5.2%) | −3.9% (−12.2%, +5.2%) | 34 → 34 MB (+0.0%) | 911 | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.023 → 0.023 s | −1.3% (−4.6%, +2.0%) | −1.1% (−4.2%, +2.0%) | 33 → 33 MB (+0.1%) | 862 | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.023 → 0.023 s | +2.0% (−1.1%, +5.3%) | +2.3% (−0.7%, +5.4%) | 33 → 33 MB (+0.1%) | 870 | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.045 → 0.045 s | +2.9% (−5.4%, +11.8%) | +2.8% (−5.1%, +11.5%) | 16 → 16 MB (−0.2%) | 432 | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.033 → 0.033 s | −2.7% (−10.6%, +5.9%) | −0.3% (−5.8%, +5.5%) | 22 → 22 MB (−0.1%) | 600 | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.031 → 0.031 s | +2.2% (−2.9%, +7.6%) | +2.4% (−1.4%, +6.3%) | 39 → 39 MB (−0.1%) | 623 | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.174 → 0.173 s | −0.5% (−2.0%, +1.0%) | −0.5% (−1.9%, +0.9%) | 23 → 23 MB (+0.0%) | 46 | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.102 → 0.102 s | +0.5% (−1.7%, +2.7%) | +0.9% (+0.2%, +1.6%) **slower** | 38 → 40 MB (+3.0%) | 78 | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.084 → 0.082 s | −1.3% (−6.0%, +3.6%) | −1.9% (−4.9%, +1.2%) | 50 → 49 MB (−0.7%) | 97 | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.199 → 0.198 s | −0.4% (−0.9%, +0.1%) | −0.4% (−0.9%, +0.1%) | 23 → 23 MB (+0.0%) | 48 | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.113 → 0.114 s | −0.2% (−1.6%, +1.3%) | −1.1% (−2.3%, +0.0%) | 43 → 43 MB (+2.0%) | 84 | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.095 → 0.097 s | +0.5% (−4.2%, +5.5%) | −1.5% (−4.3%, +1.4%) | 54 → 52 MB (−1.6%) | 100 | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.015 → 0.015 s | +0.8% (−1.3%, +2.9%) | +0.2% (−2.2%, +2.6%) | 23 → 23 MB (−0.2%) | 662 | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.016 → 0.016 s | −3.9% (−5.9%, −1.9%) **faster** | −3.0% (−5.2%, −0.8%) **faster** | 24 → 24 MB (+0.3% **more**) | 641 | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.016 → 0.016 s | −1.9% (−4.6%, +0.8%) | −3.7% (−7.2%, −0.0%) **faster** | 23 → 23 MB (+0.1%) | 619 | yes |
| `text` (2.63 MB → 13.2 MB, 5.0×) | 1 | 0.091 → 0.094 s | +2.4% (+0.2%, +4.6%) **slower** | +2.3% (+0.2%, +4.4%) **slower** | 19 → 19 MB (+0.2%) | 141 | yes |
| `text` (2.63 MB → 13.2 MB, 5.0×) | 2 | 0.056 → 0.057 s | +2.0% (−1.1%, +5.3%) | +1.0% (−1.8%, +3.8%) | 37 → 37 MB (+3.1%) | 231 | yes |
| `text` (2.63 MB → 13.2 MB, 5.0×) | 4 | 0.052 → 0.051 s | −0.8% (−4.7%, +3.3%) | −1.0% (−3.4%, +1.4%) | 56 → 54 MB (−0.7%) | 257 | yes |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

### Resources: CPU per byte and scaling, the `wikidata` corpus

**Assessment:** No change detected on the four measures at this run's precision (every interval holds no change; not a proof that nothing changed).

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | base MB/s | base CPU ns/B | base amplification | base efficiency | head MB/s | head CPU ns/B | head amplification | head efficiency | amplification, head over base |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| 1: pinned to CPU 1 | 257 | 3.90 | 1 | – | 258 | 3.88 | 1 | – |  |
| 2: CPUs 0,2, one a core | 480 | 4.11 | 1.06× (1.03–1.09) | 93% | 488 | 4.07 | 1.06× (1.02–1.10) | 94% | −0.1% (−3.9%, +3.7%) |
| 4: all 4 CPUs, sharing SMT siblings | 532 | 7.16 | 1.87× (1.81–1.94) | 51% | 520 | 7.36 | 1.88× (1.80–1.96) | 50% | +0.3% (−3.3%, +4.1%) |

**Best:** base at 4 threads (532 MB/s); head at 4 threads (520 MB/s); head's best against base's, wall: +0.0% (−2.6%, +2.7%).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

<details><summary>How to read it, and what it cannot show</summary>

- **The four comparisons the assessment reads,** declared in bench/report.py (`assessment`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run.
- **A trade-off is not a failure.** Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting.
- **What it can't show:** which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, #39). The cores' L3 is shared with other tenants and its size unknown.
- **Run-queue wait** (threads' mean count runnable, not running; schedstat): base 1t 0.00, base 2t 0.63, base 4t 0.91, head 1t 0.00, head 2t 0.62, head 4t 0.92. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference.
- **The per-thread split's unattributed CPU** (the process's total that no sampled thread holds, sampled every 10 ms): base 2t 6%, base 4t 5%, head 2t 6%, head 4t 5%.

</details>

### Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
|---|--:|--:|--:|--:|--:|
| head | 0.12 | 1.29 | 0.04 | 0.09 | 0.403 |
| base | 0.12 | 1.26 | 0.04 | 0.07 | 0.394 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

### Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs each build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU, the builds' order alternating: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (800 MHz). **Change** is head over base per stage, paired by round as above; with seven intervals a run, a mark on one stage alone can be chance (about one run in three). Diagnostic only: the decoder's own figures are the tables above. 10 runs, 2m54s of the run.

| stage | base → head c/B (median) | head share | change |
|---|--:|--:|--:|
| scanner (headers, Huffman) | 0.30 → 0.30 | 9.4% | +0.1% (−0.6%, +0.8%) |
| MTF + RLE2 | 0.42 → 0.42 | 13.2% | +0.2% (−0.7%, +1.2%) |
| T-vector build | 0.44 → 0.44 | 14.0% | +0.4% (−2.1%, +2.9%) |
| walk (the one each build uses) | 1.88 → 1.87 | 59.3% | +0.1% (−1.8%, +2.0%) |
| of which: segment assembly | 0.12 → 0.12 | 3.9% | +0.5% (−4.8%, +6.2%) |
| RLE1 | 0.09 → 0.09 | 2.9% | −0.4% (−5.1%, +4.5%) |
| CRC | 0.04 → 0.04 | 1.2% | −0.1% (−1.0%, +0.8%) |
| **total** | 3.16 → 3.15 |  | +0.1% (−1.6%, +1.9%) |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: base, “walk, room (used)”; head, “walk, room (used)”.

### Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

| | base | head | head over base, paired |
|---|--:|--:|--:|
| 1 thread | 1h59m ± 14m34s | 1h58m ± 14m28s | −0.5% (−2.5%, +1.6%) |
| 4 threads | 58m47s ± 8m31s | 58m45s ± 8m18s | +0.0% (−2.6%, +2.7%) |
| 4 threads, the sample's overlap gap as a fixed cost | 47m28s ± 6m55s | 47m00s ± 6m47s |  |
| the scanner alone (its serial part) | 17m01s ± 30.2s | 16m53s ± 32.2s |  |
| the workers alone, over 4 | 47m28s ± 6m54s | 47m00s ± 6m47s |  |

At 4 threads the workers bound it. Per byte: the scanner 9.8 ns a byte of bz2, the workers 6.16 ns a byte of output at 4 threads (CPU, summed over them), 3.33 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 11 worker threads (head).

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
| `text` | 2,625,903 | 13,238,014 | `f003180996938247…` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 |

- CPU: Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 4 cores, reported clock 800.0 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1
- rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source)
- bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4

</details>
