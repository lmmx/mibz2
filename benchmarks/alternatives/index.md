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

The latest record: [`c4b5f13`](<https://github.com/lmmx/mibz2/commit/c4b5f1384d048000f63749528d7d3f9094d8a474>) (2026-10-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2 and libbz2-rs**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40602314678>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 2.1.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.766 s | 274 | 10.5 | 2.03× (2.02–2.04) | 4.66× (4.63–4.68) | 4.83× (4.80–4.86) | 5.47× (5.44–5.50) | 4.87× (4.84–4.90) | 5.41× (5.38–5.44) | 4.86× (4.84–4.89) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.401 s | 523 | 18.6 | 1.74× (1.72–1.76) | 2.76× (2.73–2.79) | 3.27× (3.21–3.33) | 3.30× (3.26–3.34) | 3.31× (3.28–3.34) | 10.32× (10.22–10.42) | 9.28× (9.18–9.38) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.023 s | 866 | 3.6 | 0.58× (0.57–0.59) | 4.36× (4.30–4.43) | 6.02× (5.93–6.11) | 3.53× (3.49–3.59) | 6.46× (6.38–6.53) | 2.67× (2.63–2.71) | 2.65× (2.61–2.69) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.023 s | 858 | 3.7 | 0.55× (0.54–0.56) | 4.35× (4.30–4.40) | 6.02× (5.95–6.09) | 3.58× (3.48–3.69) | 6.49× (6.41–6.57) | 2.67× (2.64–2.70) | 2.65× (2.62–2.68) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.042 s | 467 | 6.5 | 2.20× (2.19–2.21) | 3.61× (3.59–3.63) | 3.33× (3.32–3.34) | 3.35× (3.34–3.36) | 3.93× (3.91–3.95) | 2.80× (2.79–2.82) | 1.85× (1.84–1.86) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.031 s | 642 | 16.3 | 1.55× (1.53–1.57) | 1.65× (1.63–1.66) | 2.96× (2.92–2.99) | 1.90× (1.86–1.95) | 3.82× (3.70–3.96) | 3.84× (3.82–3.87) | 2.54× (2.52–2.56) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.153 s | 52 | 55.6 | 1.27× (1.27–1.28) | 2.63× (2.62–2.64) | 2.22× (2.21–2.23) | 2.24× (2.23–2.25) | 2.62× (2.61–2.64) | 2.93× (2.91–2.96) | 2.87× (2.85–2.88) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.074 s | 108 | 79.0 | 1.28× (1.25–1.30) | 2.06× (2.02–2.10) | 2.51× (2.34–2.69) | 2.52× (2.45–2.59) | 2.93× (2.86–3.00) | 6.09× (5.96–6.21) | 5.95× (5.80–6.10) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.200 s | 48 | 60.3 | 1.14× (1.11–1.18) | 2.48× (2.40–2.57) | 2.18× (2.10–2.25) | 2.09× (2.01–2.17) | 2.42× (2.34–2.50) | 3.00× (2.90–3.11) | 3.04× (2.94–3.14) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.088 s | 109 | 84.2 | 1.27× (1.23–1.30) | 2.28× (2.21–2.35) | 2.15× (2.09–2.22) | 2.78× (2.70–2.87) | 2.71× (2.62–2.80) | 6.86× (6.65–7.07) | 6.94× (6.71–7.16) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.016 s | 641 | 5.1 | 0.62× (0.61–0.63) | 3.23× (3.18–3.28) | 5.72× (5.63–5.80) | 3.01× (2.96–3.06) | 7.20× (7.12–7.29) | 2.29× (2.26–2.33) | 2.27× (2.24–2.31) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.016 s | 633 | 5.2 | 0.54× (0.53–0.54) | 3.20× (3.16–3.23) | 5.66× (5.60–5.71) | 2.99× (2.95–3.02) | 7.06× (6.98–7.14) | 2.27× (2.24–2.30) | 2.25× (2.22–2.27) | yes |
| `text` (1.23 MB → 12.2 MB, 9.9×) | 1 | 0.081 s | 152 | 19.4 | 1.35× (1.34–1.36) | 3.14× (3.11–3.16) | 3.61× (3.59–3.64) | 3.45× (3.42–3.48) | 3.88× (3.85–3.91) | 3.48× (3.45–3.51) | 3.42× (3.39–3.45) | yes |
| `text` (1.23 MB → 12.2 MB, 9.9×) | 4 | 0.041 s | 300 | 33.2 | 1.25× (1.22–1.29) | 2.46× (2.40–2.51) | 3.41× (3.33–3.48) | 3.19× (3.12–3.26) | 3.92× (3.75–4.09) | 6.78× (6.63–6.92) | 6.66× (6.53–6.79) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.03× (2.01–2.04) | 4.63× (4.61–4.66) | 4.80× (4.77–4.83) | 5.46× (5.43–5.49) | 4.86× (4.83–4.89) | 5.40× (5.37–5.43) | 4.85× (4.83–4.88) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 1.87× (1.86–1.89) | 3.19× (3.16–3.21) | 3.36× (3.35–3.37) | 3.84× (3.81–3.87) | 3.34× (3.32–3.35) | 3.06× (3.05–3.07) | 2.75× (2.74–2.76) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.62) | 2.93× (2.89–2.96) | 3.83× (3.75–3.91) | 3.31× (3.27–3.36) | 5.99× (5.92–6.06) | 2.53× (2.49–2.56) | 2.51× (2.47–2.55) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.59× (0.58–0.60) | 3.03× (2.99–3.07) | 5.40× (5.33–5.48) | 3.38× (3.28–3.48) | 5.99× (5.92–6.06) | 2.51× (2.48–2.54) | 2.49× (2.46–2.52) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.14× (2.14–2.15) | 2.40× (2.38–2.41) | 2.77× (2.76–2.78) | 3.23× (3.22–3.24) | 3.79× (3.77–3.80) | 2.71× (2.70–2.72) | 1.81× (1.80–1.82) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.62× (1.58–1.66) | 1.46× (1.42–1.50) | 1.81× (1.76–1.86) | 1.74× (1.68–1.80) | 1.82× (1.75–1.89) | 1.09× (1.07–1.12) | 0.73× (0.71–0.75) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.27× (1.26–1.28) | 2.38× (2.37–2.40) | 2.12× (2.11–2.13) | 2.22× (2.21–2.23) | 2.60× (2.59–2.61) | 2.91× (2.89–2.93) | 2.84× (2.82–2.86) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.43× (1.40–1.46) | 2.17× (2.15–2.20) | 1.87× (1.85–1.89) | 2.51× (2.48–2.54) | 2.44× (2.41–2.46) | 2.05× (2.02–2.07) | 2.00× (1.98–2.02) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.14× (1.10–1.18) | 2.29× (2.21–2.37) | 1.93× (1.87–2.00) | 2.08× (2.00–2.15) | 2.41× (2.33–2.49) | 2.98× (2.88–3.08) | 3.01× (2.92–3.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.38× (1.32–1.44) | 2.18× (2.08–2.28) | 1.73× (1.65–1.82) | 2.52× (2.41–2.64) | 2.29× (2.18–2.40) | 2.15× (2.05–2.25) | 2.17× (2.07–2.27) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.66× (0.65–0.67) | 2.39× (2.36–2.42) | 3.35× (3.25–3.45) | 2.76× (2.72–2.80) | 6.45× (6.38–6.51) | 2.13× (2.11–2.16) | 2.12× (2.09–2.14) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.59× (0.59–0.60) | 2.48× (2.45–2.51) | 4.23× (4.17–4.30) | 2.79× (2.76–2.82) | 6.30× (6.23–6.38) | 2.09× (2.07–2.11) | 2.07× (2.05–2.10) |
| `text` (1.23 MB → 12.2 MB, 9.9×) | 1 | 1.34× (1.33–1.35) | 2.90× (2.88–2.93) | 3.26× (3.23–3.28) | 3.39× (3.36–3.41) | 3.81× (3.78–3.84) | 3.41× (3.39–3.44) | 3.36× (3.33–3.38) |
| `text` (1.23 MB → 12.2 MB, 9.9×) | 4 | 1.37× (1.31–1.43) | 2.18× (2.13–2.22) | 2.53× (2.49–2.57) | 2.64× (2.59–2.68) | 2.77× (2.68–2.85) | 2.01× (1.98–2.04) | 1.97× (1.94–2.01) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.32× (0.32–0.32) | 0.30× (0.29–0.31) | 1.16× (1.16–1.16) | 0.83× (0.83–0.83) | 11.79× (11.77–11.81) | 0.29× (0.28–0.29) | 0.30× (0.29–0.30) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 74 MB | 0.40× (0.40–0.41) | 0.72× (0.59–0.88) | 0.77× (0.76–0.78) | 0.64× (0.63–0.65) | 4.29× (4.27–4.31) | 0.10× (0.10–0.10) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.15× (0.15–0.15) | 0.82× (0.81–0.82) | 0.80× (0.80–0.80) | 4.29× (4.29–4.30) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.18× (0.18–0.19) | 0.15× (0.15–0.15) | 1.78× (1.78–1.78) | 0.82× (0.82–0.83) | 4.40× (4.40–4.41) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 17 MB | 0.50× (0.49–0.50) | 0.42× (0.42–0.42) | 1.64× (1.63–1.64) | 0.64× (0.63–0.64) | 7.79× (7.77–7.81) | 0.63× (0.63–0.64) | 0.66× (0.65–0.66) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 42 MB | 0.60× (0.59–0.60) | 0.65× (0.61–0.69) | 1.11× (1.09–1.13) | 0.85× (0.84–0.86) | 3.23× (3.21–3.25) | 0.25× (0.25–0.26) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.39× (0.39–0.39) | 0.31× (0.31–0.31) | 1.15× (1.14–1.15) | 0.71× (0.71–0.71) | 5.07× (5.06–5.08) | 0.32× (0.31–0.32) | 0.34× (0.33–0.34) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 53 MB | 0.59× (0.57–0.60) | 0.43× (0.42–0.43) | 0.81× (0.79–0.82) | 0.75× (0.74–0.77) | 2.36× (2.32–2.40) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 24 MB | 0.39× (0.39–0.39) | 0.31× (0.31–0.31) | 1.15× (1.15–1.15) | 0.80× (0.80–0.81) | 5.08× (5.08–5.09) | 0.31× (0.31–0.32) | 0.33× (0.33–0.33) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 58 MB | 0.56× (0.55–0.58) | 0.44× (0.42–0.46) | 0.75× (0.74–0.77) | 0.77× (0.75–0.79) | 2.18× (2.14–2.22) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.20) | 1.09× (1.08–1.09) | 0.77× (0.76–0.77) | 5.43× (5.42–5.44) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.22× (0.22–0.22) | 0.20× (0.20–0.20) | 1.63× (1.63–1.63) | 0.76× (0.76–0.76) | 5.40× (5.39–5.40) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) |
| `text` (1.23 MB → 12.2 MB, 9.9×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.38–0.39) | 1.45× (1.45–1.46) | 0.61× (0.61–0.61) | 6.58× (6.56–6.59) | 0.41× (0.40–0.41) | 0.43× (0.43–0.43) |
| `text` (1.23 MB → 12.2 MB, 9.9×) | 4 | 52 MB | 0.52× (0.50–0.54) | 0.48× (0.44–0.54) | 0.98× (0.95–1.00) | 0.75× (0.72–0.78) | 2.51× (2.45–2.57) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | wall |
| --- | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 1.03 | 0.02 | 0.401 |

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2871 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m11s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.15 | 11.6% |
| MTF + RLE2 | 2.19 | 22.0% |
| T-vector build | 1.71 | 17.2% |
| walk (the one each build uses) | 4.48 | 44.8% |
| of which: segment assembly | 0.14 | 1.4% |
| RLE1 | 0.19 | 1.9% |
| CRC | 0.26 | 2.6% |
| **total** | 9.98 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h51m ± 13m31s |
| 4 threads | 58m30s ± 8m13s |
| 4 threads, the sample's overlap gap as a fixed cost | 37m59s ± 5m27s |
| the scanner alone (its serial part) | 15m08s ± 43.7s |
| the workers alone, over 4 | 37m58s ± 5m27s |

At 4 threads the workers bound it. Per byte: the scanner 8.8 ns a byte of bz2, the workers 4.98 ns a byte of output at 4 threads (CPU, summed over them), 3.16 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,232,504 | 12,212,095 | \`eec9039e0c9d71dd…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 2870.8 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1017-azure; image ubuntu24 20261026.354.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source) - bench 2.1.1 (bench/report.py), 10 rounds, threads 1, 4
