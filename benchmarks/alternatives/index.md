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

The latest record: [`b4b6198`](<https://github.com/lmmx/mibz2/commit/b4b619867e50aa9a2811757f2147c0b20284036f>) (2026-10-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2 and libbz2-rs**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40604114961>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 2.1.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.384 s | 546 | 8.3 | 2.05× (2.00–2.10) | 5.26× (5.11–5.42) | 5.05× (4.94–5.17) | 6.28× (6.14–6.42) | 5.41× (5.27–5.55) | 5.71× (5.56–5.86) | 5.17× (4.99–5.35) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.247 s | 848 | 16.9 | 1.47× (1.42–1.53) | 2.44× (2.37–2.51) | 3.12× (3.01–3.25) | 2.90× (2.83–2.98) | 2.70× (2.60–2.81) | 8.93× (8.57–9.31) | 8.09× (7.79–8.40) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.009 s | 2,235 | 2.4 | 0.76× (0.72–0.80) | 5.66× (5.44–5.90) | 10.59× (10.16–11.04) | 5.20× (4.99–5.42) | 5.57× (5.37–5.78) | 4.23× (4.07–4.40) | 4.22× (4.05–4.39) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.009 s | 2,222 | 2.4 | 0.78× (0.75–0.82) | 5.56× (5.44–5.68) | 10.31× (10.08–10.54) | 5.21× (5.11–5.31) | 5.56× (5.45–5.67) | 4.18× (4.09–4.28) | 4.16× (4.08–4.26) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.020 s | 987 | 4.9 | 2.42× (2.39–2.45) | 5.18× (5.13–5.24) | 4.80× (4.73–4.86) | 4.15× (4.10–4.20) | 3.24× (3.20–3.28) | 2.52× (2.47–2.58) | 2.30× (2.28–2.33) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.017 s | 1,167 | 11.9 | 1.42× (1.34–1.51) | 2.95× (2.79–3.12) | 2.53× (2.39–2.67) | 2.07× (1.99–2.16) | 1.86× (1.76–1.97) | 2.93× (2.76–3.11) | 2.67× (2.54–2.81) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.083 s | 97 | 47.7 | 1.27× (1.25–1.29) | 3.05× (3.00–3.10) | 2.83× (2.65–3.02) | 2.59× (2.55–2.63) | 2.57× (2.52–2.61) | 3.43× (3.39–3.48) | 3.31× (3.27–3.36) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.041 s | 197 | 69.2 | 1.20× (1.14–1.26) | 2.44× (2.34–2.55) | 2.47× (2.19–2.78) | 2.87× (2.67–3.08) | 2.20× (2.11–2.30) | 6.87× (6.62–7.13) | 6.63× (6.37–6.90) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.109 s | 88 | 52.1 | 1.10× (1.09–1.11) | 2.93× (2.78–3.09) | 2.31× (2.22–2.40) | 2.34× (2.32–2.37) | 2.37× (2.34–2.40) | 3.61× (3.55–3.67) | 3.60× (3.56–3.65) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.051 s | 187 | 74.9 | 1.13× (1.08–1.17) | 3.01× (2.91–3.12) | 2.87× (2.77–2.97) | 3.02× (2.86–3.20) | 2.04× (1.97–2.11) | 7.76× (7.46–8.07) | 7.76× (7.48–8.05) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.007 s | 1,379 | 3.9 | 0.68× (0.67–0.69) | 7.00× (6.71–7.30) | 6.16× (5.90–6.42) | 3.91× (3.62–4.23) | 4.25× (4.09–4.42) | 2.92× (2.80–3.04) | 2.91× (2.79–3.03) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.008 s | 1,299 | 4.1 | 0.63× (0.60–0.65) | 6.59× (6.35–6.84) | 5.64× (5.44–5.86) | 3.61× (3.48–3.75) | 4.11× (3.95–4.28) | 2.76× (2.66–2.87) | 2.75× (2.65–2.86) | yes |
| `text` (1.23 MB → 12.2 MB, 9.9×) | 1 | 0.042 s | 294 | 15.9 | 1.38× (1.36–1.40) | 3.61× (3.55–3.67) | 3.45× (3.40–3.51) | 3.75× (3.67–3.82) | 3.52× (3.47–3.58) | 3.94× (3.88–4.00) | 3.66× (3.60–3.72) | yes |
| `text` (1.23 MB → 12.2 MB, 9.9×) | 4 | 0.023 s | 528 | 29.5 | 1.14× (1.07–1.22) | 2.27× (2.00–2.59) | 3.94× (3.77–4.13) | 3.25× (3.04–3.46) | 2.35× (2.22–2.48) | 6.95× (6.63–7.27) | 6.45× (6.16–6.76) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.04× (2.00–2.09) | 5.17× (5.02–5.34) | 4.98× (4.86–5.10) | 6.26× (6.12–6.40) | 5.39× (5.25–5.53) | 5.69× (5.54–5.84) | 5.15× (4.97–5.34) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 1.67× (1.61–1.74) | 2.96× (2.86–3.06) | 3.25× (3.13–3.37) | 3.55× (3.42–3.69) | 3.10× (2.98–3.23) | 2.80× (2.70–2.90) | 2.53× (2.42–2.65) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.79× (0.76–0.83) | 4.43× (4.25–4.63) | 5.24× (5.01–5.47) | 4.61× (4.40–4.82) | 4.92× (4.72–5.12) | 3.77× (3.61–3.94) | 3.76× (3.60–3.92) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.81× (0.78–0.85) | 4.61× (4.52–4.71) | 5.94× (5.82–6.06) | 4.68× (4.58–4.77) | 4.93× (4.84–5.03) | 3.75× (3.67–3.84) | 3.74× (3.66–3.82) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.31× (2.28–2.35) | 2.92× (2.88–2.95) | 3.20× (3.16–3.24) | 3.93× (3.87–3.99) | 3.08× (3.04–3.12) | 2.42× (2.37–2.48) | 2.21× (2.18–2.24) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.72× (1.65–1.79) | 1.70× (1.60–1.79) | 1.95× (1.87–2.04) | 2.06× (1.96–2.17) | 1.54× (1.46–1.62) | 1.01× (0.96–1.05) | 0.92× (0.87–0.96) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.26× (1.24–1.28) | 2.88× (2.83–2.93) | 2.38× (2.34–2.42) | 2.57× (2.52–2.61) | 2.54× (2.49–2.59) | 3.39× (3.34–3.44) | 3.27× (3.22–3.32) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.36× (1.33–1.39) | 2.52× (2.47–2.58) | 2.07× (2.02–2.12) | 2.74× (2.67–2.81) | 2.27× (2.23–2.32) | 2.35× (2.29–2.41) | 2.27× (2.22–2.31) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.10× (1.09–1.11) | 2.78× (2.74–2.82) | 2.15× (2.04–2.26) | 2.33× (2.30–2.35) | 2.35× (2.32–2.38) | 3.57× (3.51–3.64) | 3.57× (3.52–3.61) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.26× (1.23–1.28) | 2.42× (2.38–2.47) | 1.84× (1.80–1.89) | 2.65× (2.59–2.70) | 2.13× (2.09–2.17) | 2.46× (2.40–2.51) | 2.45× (2.41–2.49) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.72× (0.71–0.74) | 3.19× (2.98–3.40) | 3.72× (3.54–3.91) | 3.47× (3.21–3.74) | 3.73× (3.58–3.88) | 2.62× (2.50–2.74) | 2.60× (2.49–2.72) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.68× (0.66–0.71) | 3.12× (3.02–3.22) | 4.12× (3.99–4.27) | 3.31× (3.21–3.41) | 3.67× (3.54–3.80) | 2.51× (2.44–2.59) | 2.50× (2.43–2.57) |
| `text` (1.23 MB → 12.2 MB, 9.9×) | 1 | 1.37× (1.34–1.39) | 3.28× (3.23–3.33) | 3.23× (3.17–3.29) | 3.66× (3.59–3.73) | 3.44× (3.38–3.50) | 3.85× (3.78–3.91) | 3.57× (3.51–3.64) |
| `text` (1.23 MB → 12.2 MB, 9.9×) | 4 | 1.22× (1.16–1.28) | 2.16× (2.10–2.21) | 2.47× (2.42–2.51) | 2.68× (2.62–2.73) | 2.27× (2.21–2.32) | 2.10× (2.04–2.16) | 1.95× (1.90–2.00) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.37× (0.36–0.37) | 0.34× (0.33–0.35) | 1.11× (1.11–1.11) | 0.85× (0.84–0.86) | 11.35× (11.34–11.37) | 0.31× (0.31–0.31) | 0.33× (0.32–0.33) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 78 MB | 0.40× (0.40–0.40) | 0.49× (0.48–0.50) | 0.73× (0.72–0.74) | 0.62× (0.61–0.64) | 4.07× (4.04–4.09) | 0.11× (0.11–0.11) | 0.12× (0.11–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.30× (0.29–0.31) | 0.18× (0.17–0.18) | 0.78× (0.78–0.79) | 0.80× (0.80–0.81) | 4.20× (4.16–4.24) | 0.25× (0.24–0.26) | 0.26× (0.25–0.27) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 35 MB | 0.20× (0.20–0.20) | 0.15× (0.15–0.15) | 1.71× (1.69–1.73) | 0.81× (0.80–0.82) | 4.21× (4.19–4.22) | 0.25× (0.24–0.26) | 0.26× (0.25–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 18 MB | 0.57× (0.55–0.58) | 0.41× (0.40–0.43) | 1.52× (1.52–1.53) | 0.65× (0.65–0.66) | 7.39× (7.36–7.42) | 0.58× (0.58–0.59) | 0.61× (0.61–0.61) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 44 MB | 0.61× (0.59–0.62) | 0.61× (0.58–0.64) | 1.05× (1.03–1.06) | 0.83× (0.83–0.84) | 3.08× (3.06–3.11) | 0.24× (0.23–0.24) | 0.25× (0.24–0.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 25 MB | 0.45× (0.44–0.45) | 0.30× (0.30–0.30) | 1.09× (1.09–1.09) | 0.72× (0.72–0.72) | 4.90× (4.88–4.92) | 0.35× (0.35–0.35) | 0.36× (0.36–0.37) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 52 MB | 0.60× (0.58–0.62) | 0.43× (0.42–0.43) | 0.81× (0.79–0.82) | 0.76× (0.73–0.78) | 2.37× (2.32–2.42) | 0.16× (0.16–0.17) | 0.17× (0.17–0.17) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 25 MB | 0.44× (0.44–0.45) | 0.30× (0.30–0.30) | 1.10× (1.09–1.10) | 0.81× (0.81–0.81) | 4.93× (4.92–4.95) | 0.34× (0.34–0.35) | 0.36× (0.36–0.36) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 60 MB | 0.55× (0.54–0.56) | 0.40× (0.38–0.43) | 0.72× (0.70–0.73) | 0.74× (0.71–0.78) | 2.10× (2.06–2.14) | 0.14× (0.14–0.15) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 25 MB | 0.42× (0.41–0.42) | 0.21× (0.20–0.23) | 1.06× (1.03–1.09) | 0.76× (0.73–0.78) | 5.11× (5.02–5.20) | 0.33× (0.32–0.33) | 0.34× (0.34–0.35) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 26 MB | 0.26× (0.24–0.27) | 0.18× (0.18–0.18) | 1.47× (1.44–1.50) | 0.70× (0.68–0.72) | 4.78× (4.76–4.80) | 0.30× (0.30–0.31) | 0.32× (0.32–0.32) |
| `text` (1.23 MB → 12.2 MB, 9.9×) | 1 | 20 MB | 0.51× (0.50–0.53) | 0.37× (0.37–0.38) | 1.38× (1.38–1.38) | 0.61× (0.59–0.63) | 6.34× (6.32–6.37) | 0.43× (0.43–0.44) | 0.46× (0.45–0.46) |
| `text` (1.23 MB → 12.2 MB, 9.9×) | 4 | 54 MB | 0.52× (0.50–0.54) | 0.44× (0.40–0.48) | 0.94× (0.92–0.97) | 0.71× (0.69–0.72) | 2.41× (2.34–2.49) | 0.16× (0.15–0.16) | 0.17× (0.16–0.17) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | wall |
| --- | ---: | ---: | ---: | ---: |
| mibz2 | 0.05 | 0.55 | 0.01 | 0.247 |

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4530 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 35.1s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.04 | 13.5% |
| MTF + RLE2 | 1.90 | 24.8% |
| T-vector build | 1.44 | 18.8% |
| walk (the one each build uses) | 2.89 | 37.7% |
| of which: segment assembly | 0.10 | 1.3% |
| RLE1 | 0.15 | 1.9% |
| CRC | 0.25 | 3.3% |
| **total** | 7.67 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 56m33s ± 7m01s |
| 4 threads | 36m09s ± 5m11s |
| 4 threads, the sample's overlap gap as a fixed cost | 20m31s ± 3m19s |
| the scanner alone (its serial part) | 7m08s ± 54.7s |
| the workers alone, over 4 | 20m31s ± 3m19s |

At 4 threads the workers bound it. Per byte: the scanner 4.1 ns a byte of bz2, the workers 2.69 ns a byte of output at 4 threads (CPU, summed over them), 1.62 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 12 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,232,504 | 12,212,095 | \`eec9039e0c9d71dd…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4529.9 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 6.17.0-1022-azure; image ubuntu24 20261023.334.3 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source) - bench 2.1.1 (bench/report.py), 10 rounds, threads 1, 4
