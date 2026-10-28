# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

| decoder | what runs | threads |
| --- | --- | --- |
| pbz2 | Soof Golan's Rust decoder: `DecoderReader`, or `ParallelDecoder::with_threads` | block-parallel |
| lbzip2 | the C program: `lbzip2 -dc -n T`, read through a pipe | block-parallel |
| indexed\_bzip2 | C++, from Python: `IndexedBzip2File(path, parallelization=T)` (`bench/ib.py`; the interpreter's start left out) | block-parallel |
| fbz | Answer.AI's Rust decoder: `fbz::Reader`, `threads: T` | block-parallel |
| lbzip2-rs | the `lbzip2` crate (pure Rust): `StreamingBz2Read`, on the CPUs it may run on (it takes no thread count) | block-parallel |
| libbzip2 | the reference C library, 1.0.8: the `bzip2` crate's bundled source, `MultiBzDecoder` (as `bzip2 -dc` and Python's `bz2` decode) | single-threaded |
| libbz2-rs | libbz2-rs-sys, the pure-Rust port of libbzip2, through the `bzip2` crate 0.6 | single-threaded |

**Single-threaded decoders run at 1 thread only.** libbzip2 and libbz2-rs have no thread count, so they never decode on 4 threads. At 4 threads, their figures are their 1-thread runs against mibz2's at 4 threads: in the chart, hollow points with dashed whiskers, under "single-threaded (1 thread)"; in the tables, marked ¹.

Not included: `bzip2 -dc` and Python's `bz2` module, which are libbzip2 itself (its arm stands for them); and bz2k, mibz2's own prototype, which it replaced.

The latest record: [`115f26f`](<https://github.com/lmmx/mibz2/commit/115f26fd05a25fc331a9521a7e7e2a865c5a0b34>) (2026-10-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2 and libbz2-rs**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40615185535>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 2.1.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.591 s | 355 | 10.1 | 2.00× (1.98–2.02) | 4.70× (4.65–4.75) | 4.50× (4.42–4.57) | 5.47× (5.35–5.59) | 4.64× (4.54–4.74) | 5.27× (5.11–5.43) | 4.51× (4.48–4.54) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.311 s | 675 | 17.8 | 1.72× (1.70–1.75) | 2.57× (2.52–2.62) | 3.22× (3.16–3.29) | 3.04× (2.99–3.08) | 2.74× (2.69–2.78) | 9.99× (9.68–10.32) | 8.56× (8.43–8.69) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.012 s | 1,702 | 2.4 | 0.77× (0.74–0.79) | 8.75× (8.49–9.03) | 7.94× (7.72–8.18) | 4.95× (4.81–5.09) | 5.09× (4.93–5.25) | 4.02× (3.91–4.14) | 3.98× (3.86–4.10) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.012 s | 1,695 | 2.5 | 0.78× (0.77–0.80) | 4.26× (4.17–4.34) | 7.75× (7.59–7.91) | 4.95× (4.84–5.06) | 5.16× (5.01–5.32) | 3.98× (3.89–4.06) | 3.93× (3.85–4.01) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.030 s | 661 | 5.8 | 2.33× (2.31–2.36) | 3.42× (3.39–3.46) | 3.15× (3.11–3.18) | 3.66× (3.62–3.70) | 2.66× (2.63–2.69) | 3.05× (3.02–3.08) | 2.00× (1.98–2.02) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.021 s | 918 | 14.2 | 1.67× (1.64–1.70) | 2.34× (2.30–2.39) | 4.27× (4.18–4.36) | 2.07× (2.01–2.14) | 1.79× (1.75–1.84) | 4.20× (4.10–4.30) | 2.76× (2.70–2.81) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.120 s | 66 | 54.7 | 1.18× (1.06–1.32) | 2.40× (2.15–2.67) | 2.28× (2.04–2.56) | 2.21× (1.99–2.45) | 2.14× (1.92–2.39) | 2.76× (2.47–3.08) | 2.72× (2.44–3.04) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.056 s | 144 | 72.6 | 1.33× (1.20–1.48) | 2.70× (2.61–2.78) | 2.54× (2.45–2.62) | 2.55× (2.47–2.64) | 2.06× (2.00–2.13) | 6.20× (6.00–6.41) | 6.12× (5.93–6.32) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.154 s | 62 | 58.0 | 1.15× (1.14–1.16) | 2.62× (2.60–2.63) | 2.13× (2.01–2.25) | 2.13× (2.10–2.15) | 2.19× (2.10–2.29) | 3.06× (3.04–3.09) | 3.12× (3.10–3.13) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.066 s | 145 | 80.2 | 1.29× (1.22–1.37) | 2.34× (2.16–2.53) | 2.13× (2.05–2.21) | 2.85× (2.75–2.95) | 2.00× (1.93–2.07) | 7.03× (6.77–7.30) | 7.15× (6.90–7.42) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 1,026 | 4.2 | 0.67× (0.66–0.68) | 5.27× (5.15–5.38) | 4.79× (3.96–5.79) | 3.57× (3.48–3.67) | 3.84× (3.75–3.92) | 2.79× (2.73–2.86) | 2.75× (2.68–2.82) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 990 | 4.3 | 0.63× (0.61–0.66) | 5.02× (4.86–5.19) | 9.15× (8.86–9.45) | 3.48× (3.38–3.60) | 3.74× (3.63–3.85) | 2.69× (2.60–2.78) | 2.65× (2.55–2.74) | yes |
| `text` (1.24 MB → 12.2 MB, 9.9×) | 1 | 0.060 s | 203 | 18.2 | 1.36× (1.34–1.37) | 3.36× (3.32–3.39) | 3.60× (3.30–3.93) | 3.46× (3.35–3.57) | 3.25× (3.22–3.29) | 3.60× (3.57–3.64) | 3.52× (3.45–3.58) | yes |
| `text` (1.24 MB → 12.2 MB, 9.9×) | 4 | 0.031 s | 389 | 29.6 | 1.23× (1.15–1.32) | 3.19× (3.07–3.31) | 3.04× (2.73–3.39) | 3.20× (3.01–3.39) | 2.25× (2.17–2.34) | 6.83× (6.57–7.09) | 6.66× (6.40–6.93) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.00× (1.98–2.02) | 4.64× (4.59–4.69) | 4.44× (4.38–4.50) | 5.46× (5.34–5.58) | 4.63× (4.53–4.73) | 5.25× (5.10–5.42) | 4.50× (4.47–4.53) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 1.90× (1.86–1.94) | 2.95× (2.92–2.99) | 3.32× (3.26–3.39) | 3.61× (3.57–3.65) | 3.09× (3.05–3.13) | 3.02× (2.92–3.12) | 2.58× (2.55–2.61) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.80× (0.78–0.82) | 4.21× (4.10–4.31) | 5.13× (4.85–5.42) | 4.40× (4.29–4.51) | 4.52× (4.40–4.64) | 3.60× (3.51–3.69) | 3.56× (3.48–3.65) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.81× (0.79–0.82) | 4.40× (4.32–4.48) | 6.16× (6.06–6.25) | 4.44× (4.36–4.52) | 4.56× (4.45–4.68) | 3.56× (3.50–3.62) | 3.52× (3.47–3.57) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.25× (2.23–2.28) | 2.57× (2.55–2.60) | 2.80× (2.77–2.84) | 3.50× (3.46–3.54) | 2.56× (2.54–2.59) | 2.93× (2.90–2.96) | 1.94× (1.92–1.96) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.72× (1.61–1.83) | 1.59× (1.53–1.65) | 1.92× (1.85–1.99) | 1.94× (1.88–2.00) | 1.33× (1.28–1.38) | 1.22× (1.17–1.26) | 0.81× (0.78–0.84) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.18× (1.06–1.31) | 2.25× (2.02–2.51) | 1.96× (1.76–2.18) | 2.19× (1.98–2.43) | 2.13× (1.91–2.37) | 2.73× (2.45–3.05) | 2.70× (2.42–3.01) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.43× (1.36–1.49) | 2.24× (2.18–2.30) | 1.94× (1.92–1.95) | 2.67× (2.63–2.71) | 2.21× (2.19–2.24) | 2.15× (2.13–2.17) | 2.12× (2.10–2.14) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.15× (1.14–1.16) | 2.34× (2.33–2.35) | 1.91× (1.89–1.93) | 2.11× (2.09–2.14) | 2.18× (2.09–2.27) | 3.04× (3.02–3.06) | 3.09× (3.08–3.10) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.36× (1.30–1.42) | 2.23× (2.17–2.28) | 1.74× (1.71–1.76) | 2.61× (2.56–2.66) | 2.09× (2.07–2.12) | 2.21× (2.19–2.24) | 2.25× (2.23–2.28) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.73× (0.72–0.74) | 2.85× (2.79–2.90) | 3.62× (3.45–3.81) | 3.15× (3.08–3.22) | 3.38× (3.31–3.44) | 2.50× (2.45–2.55) | 2.47× (2.42–2.52) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.69× (0.67–0.71) | 2.97× (2.89–3.05) | 4.16× (4.04–4.27) | 3.17× (3.09–3.25) | 3.33× (3.25–3.41) | 2.42× (2.35–2.49) | 2.39× (2.32–2.46) |
| `text` (1.24 MB → 12.2 MB, 9.9×) | 1 | 1.34× (1.33–1.36) | 2.99× (2.96–3.02) | 3.13× (3.08–3.18) | 3.38× (3.28–3.49) | 3.19× (3.16–3.22) | 3.53× (3.50–3.55) | 3.44× (3.38–3.50) |
| `text` (1.24 MB → 12.2 MB, 9.9×) | 4 | 1.39× (1.34–1.45) | 2.20× (2.14–2.26) | 2.58× (2.53–2.63) | 2.76× (2.71–2.81) | 2.29× (2.25–2.34) | 2.14× (2.10–2.18) | 2.09× (2.05–2.13) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.37× (0.37–0.38) | 0.33× (0.32–0.35) | 1.11× (1.11–1.12) | 0.85× (0.85–0.86) | 11.38× (11.35–11.40) | 0.31× (0.31–0.32) | 0.33× (0.33–0.33) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 78 MB | 0.40× (0.39–0.40) | 0.49× (0.48–0.50) | 0.73× (0.73–0.74) | 0.63× (0.62–0.63) | 4.08× (4.06–4.10) | 0.11× (0.11–0.11) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.30× (0.30–0.30) | 0.18× (0.18–0.18) | 0.79× (0.78–0.80) | 0.80× (0.79–0.82) | 4.23× (4.18–4.28) | 0.25× (0.24–0.25) | 0.26× (0.25–0.26) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 35 MB | 0.20× (0.19–0.21) | 0.16× (0.14–0.17) | 1.71× (1.69–1.74) | 0.81× (0.80–0.81) | 4.22× (4.20–4.24) | 0.25× (0.24–0.25) | 0.26× (0.25–0.26) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 18 MB | 0.57× (0.57–0.57) | 0.40× (0.40–0.41) | 1.53× (1.52–1.53) | 0.66× (0.65–0.66) | 7.41× (7.38–7.43) | 0.59× (0.59–0.59) | 0.61× (0.61–0.62) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 44 MB | 0.61× (0.60–0.63) | 0.63× (0.60–0.65) | 1.06× (1.03–1.08) | 0.84× (0.82–0.86) | 3.13× (3.07–3.18) | 0.24× (0.24–0.25) | 0.25× (0.25–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 25 MB | 0.45× (0.44–0.45) | 0.30× (0.30–0.30) | 1.09× (1.09–1.09) | 0.72× (0.72–0.72) | 4.90× (4.87–4.92) | 0.35× (0.35–0.35) | 0.36× (0.36–0.37) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 53 MB | 0.61× (0.58–0.64) | 0.42× (0.41–0.43) | 0.79× (0.77–0.81) | 0.74× (0.71–0.78) | 2.32× (2.26–2.38) | 0.16× (0.16–0.16) | 0.17× (0.16–0.17) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 25 MB | 0.44× (0.44–0.45) | 0.30× (0.30–0.30) | 1.10× (1.10–1.10) | 0.81× (0.81–0.81) | 4.93× (4.91–4.95) | 0.34× (0.34–0.35) | 0.36× (0.36–0.36) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 60 MB | 0.55× (0.54–0.56) | 0.41× (0.39–0.44) | 0.71× (0.70–0.72) | 0.69× (0.67–0.71) | 2.08× (2.05–2.12) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 25 MB | 0.42× (0.41–0.44) | 0.20× (0.19–0.20) | 1.06× (1.03–1.10) | 0.75× (0.73–0.78) | 5.10× (5.00–5.21) | 0.33× (0.32–0.33) | 0.34× (0.33–0.35) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 26 MB | 0.25× (0.25–0.25) | 0.18× (0.18–0.18) | 1.50× (1.48–1.53) | 0.70× (0.68–0.72) | 4.76× (4.73–4.79) | 0.30× (0.30–0.31) | 0.32× (0.32–0.32) |
| `text` (1.24 MB → 12.2 MB, 9.9×) | 1 | 20 MB | 0.52× (0.50–0.53) | 0.38× (0.37–0.38) | 1.38× (1.38–1.39) | 0.60× (0.58–0.62) | 6.35× (6.34–6.36) | 0.43× (0.43–0.44) | 0.45× (0.45–0.46) |
| `text` (1.24 MB → 12.2 MB, 9.9×) | 4 | 48 MB | 0.59× (0.57–0.61) | 0.49× (0.45–0.54) | 1.06× (1.05–1.08) | 0.77× (0.72–0.82) | 2.70× (2.67–2.73) | 0.18× (0.18–0.18) | 0.19× (0.18–0.19) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | wall |
| --- | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.84 | 0.02 | 0.311 |

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3582 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 51.4s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.12 | 11.7% |
| MTF + RLE2 | 2.03 | 21.2% |
| T-vector build | 1.66 | 17.3% |
| walk (the one each build uses) | 4.36 | 45.5% |
| of which: segment assembly | 0.13 | 1.4% |
| RLE1 | 0.18 | 1.9% |
| CRC | 0.25 | 2.6% |
| **total** | 9.59 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h26m ± 10m37s |
| 4 threads | 45m29s ± 6m25s |
| 4 threads, the sample's overlap gap as a fixed cost | 30m07s ± 4m20s |
| the scanner alone (its serial part) | 10m37s ± 59.2s |
| the workers alone, over 4 | 30m07s ± 4m20s |

At 4 threads the workers bound it. Per byte: the scanner 6.2 ns a byte of bz2, the workers 3.95 ns a byte of output at 4 threads (CPU, summed over them), 2.48 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 11 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,236,217 | 12,227,335 | \`294c8c24e2835ff3…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3581.6 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 6.17.0-1022-azure; image ubuntu24 20261023.334.3 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source) - bench 2.1.1 (bench/report.py), 10 rounds, threads 1, 4
