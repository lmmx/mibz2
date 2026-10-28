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

The latest record: [`97d4ba2`](<https://github.com/lmmx/mibz2/commit/97d4ba2f721aab99b2d3b52ba4c8f963a6253541>) (2026-10-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2 and libbz2-rs**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40624674995>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 2.1.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.823 s | 255 | 12.8 | 1.86× (1.83–1.90) | 4.21× (4.12–4.30) | 4.45× (4.35–4.56) | 4.71× (4.63–4.80) | 4.31× (4.23–4.39) | 4.54× (4.45–4.63) | 4.32× (4.24–4.40) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.425 s | 493 | 22.1 | 1.74× (1.72–1.76) | 2.79× (2.72–2.86) | 3.45× (3.35–3.56) | 3.01× (2.95–3.07) | 3.20× (3.12–3.29) | 8.93× (8.78–9.08) | 8.48× (8.35–8.62) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.022 s | 897 | 4.0 | 0.57× (0.57–0.58) | 4.59× (4.53–4.66) | 6.31× (6.22–6.40) | 3.36× (3.32–3.40) | 6.39× (6.29–6.49) | 2.65× (2.62–2.69) | 2.49× (2.45–2.52) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.023 s | 879 | 4.1 | 0.56× (0.55–0.57) | 4.50× (4.42–4.58) | 6.24× (6.13–6.36) | 3.32× (3.27–3.38) | 6.26× (6.14–6.38) | 2.62× (2.56–2.68) | 2.45× (2.41–2.50) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.046 s | 429 | 7.9 | 2.03× (1.98–2.08) | 2.39× (2.13–2.68) | 3.14× (2.91–3.39) | 2.82× (2.77–2.87) | 3.38× (3.31–3.44) | 1.85× (1.81–1.89) | 1.56× (1.53–1.60) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.032 s | 608 | 18.4 | 1.50× (1.46–1.54) | 1.65× (1.41–1.95) | 2.77× (2.69–2.85) | 1.81× (1.76–1.85) | 3.50× (3.41–3.58) | 2.60× (2.54–2.66) | 2.20× (2.15–2.25) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.160 s | 50 | 65.7 | 1.16× (1.14–1.17) | 2.52× (2.50–2.54) | 2.13× (2.11–2.15) | 2.04× (2.02–2.05) | 2.51× (2.42–2.61) | 2.57× (2.55–2.58) | 2.64× (2.62–2.66) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.077 s | 103 | 94.4 | 1.21× (1.18–1.24) | 2.00× (1.86–2.15) | 2.16× (1.94–2.42) | 2.28× (2.22–2.34) | 2.73× (2.66–2.80) | 5.25× (5.15–5.34) | 5.39× (5.28–5.50) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.210 s | 46 | 71.6 | 1.03× (0.97–1.10) | 2.33× (2.18–2.50) | 1.81× (1.69–1.94) | 1.77× (1.65–1.90) | 2.18× (2.04–2.33) | 2.54× (2.38–2.72) | 2.72× (2.55–2.90) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.096 s | 100 | 102.7 | 1.16× (1.13–1.19) | 2.15× (2.02–2.28) | 1.98× (1.94–2.01) | 2.43× (2.34–2.52) | 2.46× (2.40–2.53) | 5.67× (5.54–5.80) | 6.07× (5.92–6.22) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.015 s | 654 | 5.7 | 0.61× (0.60–0.63) | 3.28× (3.22–3.34) | 5.86× (5.75–5.97) | 2.82× (2.76–2.87) | 7.02× (6.90–7.14) | 2.22× (2.18–2.26) | 2.09× (2.05–2.13) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.016 s | 623 | 6.0 | 0.54× (0.52–0.55) | 3.18× (3.13–3.23) | 5.64× (5.55–5.74) | 2.75× (2.71–2.80) | 6.74× (6.60–6.87) | 2.14× (2.11–2.18) | 2.02× (1.99–2.06) | yes |
| `text` (1.32 MB → 12 MB, 9.1×) | 1 | 0.086 s | 140 | 23.6 | 1.36× (1.34–1.39) | 2.94× (2.88–3.01) | 3.39× (3.32–3.46) | 3.02× (2.96–3.08) | 3.44× (3.38–3.50) | 3.23× (3.18–3.28) | 3.12× (3.08–3.17) | yes |
| `text` (1.32 MB → 12 MB, 9.1×) | 4 | 0.043 s | 277 | 40.3 | 1.30× (1.25–1.35) | 2.33× (2.29–2.36) | 3.33× (3.12–3.56) | 2.98× (2.91–3.06) | 3.59× (3.53–3.66) | 6.36× (6.26–6.46) | 6.15× (6.05–6.24) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 1.86× (1.82–1.90) | 4.17× (4.08–4.26) | 4.42× (4.32–4.52) | 4.70× (4.62–4.79) | 4.30× (4.22–4.38) | 4.53× (4.44–4.62) | 4.31× (4.23–4.38) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 1.87× (1.85–1.89) | 3.19× (3.13–3.25) | 3.43× (3.36–3.51) | 3.52× (3.46–3.58) | 3.28× (3.20–3.36) | 2.65× (2.62–2.68) | 2.52× (2.49–2.55) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.61–0.62) | 3.07× (3.03–3.11) | 3.99× (3.94–4.05) | 3.14× (3.10–3.18) | 5.89× (5.80–5.99) | 2.50× (2.46–2.54) | 2.35× (2.31–2.38) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.60× (0.59–0.61) | 3.23× (3.14–3.33) | 5.16× (5.09–5.23) | 3.12× (3.07–3.17) | 5.71× (5.60–5.81) | 2.44× (2.39–2.49) | 2.29× (2.25–2.33) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 1.98× (1.93–2.03) | 2.14× (2.10–2.19) | 2.64× (2.53–2.77) | 2.74× (2.69–2.78) | 3.27× (3.21–3.33) | 1.81× (1.78–1.85) | 1.54× (1.51–1.57) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.64× (1.58–1.71) | 1.50× (1.42–1.57) | 1.79× (1.71–1.87) | 1.70× (1.63–1.77) | 1.81× (1.74–1.88) | 0.79× (0.76–0.81) | 0.67× (0.64–0.69) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.15× (1.14–1.16) | 2.24× (2.21–2.26) | 2.00× (1.98–2.02) | 2.02× (2.01–2.04) | 2.49× (2.41–2.59) | 2.55× (2.53–2.56) | 2.61× (2.60–2.63) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.35× (1.29–1.40) | 2.13× (2.08–2.18) | 1.76× (1.72–1.80) | 2.32× (2.28–2.36) | 2.30× (2.25–2.34) | 1.77× (1.75–1.79) | 1.82× (1.81–1.83) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.03× (0.97–1.10) | 2.14× (2.02–2.27) | 1.73× (1.63–1.85) | 1.76× (1.64–1.89) | 2.17× (2.03–2.32) | 2.52× (2.36–2.70) | 2.70× (2.53–2.88) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.31× (1.28–1.34) | 2.15× (2.02–2.28) | 1.60× (1.57–1.63) | 2.26× (2.21–2.32) | 2.15× (2.09–2.21) | 1.82× (1.79–1.86) | 1.95× (1.91–1.99) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.66× (0.65–0.67) | 2.48× (2.43–2.55) | 3.38× (3.32–3.45) | 2.58× (2.54–2.62) | 6.26× (6.16–6.35) | 2.06× (2.03–2.09) | 1.95× (1.92–1.98) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.60× (0.58–0.61) | 2.53× (2.48–2.58) | 4.12× (4.06–4.17) | 2.57× (2.52–2.62) | 5.96× (5.83–6.09) | 1.96× (1.92–2.00) | 1.85× (1.82–1.89) |
| `text` (1.32 MB → 12 MB, 9.1×) | 1 | 1.36× (1.33–1.38) | 2.74× (2.69–2.80) | 3.06× (3.01–3.12) | 2.97× (2.91–3.03) | 3.38× (3.32–3.44) | 3.18× (3.12–3.23) | 3.07× (3.03–3.12) |
| `text` (1.32 MB → 12 MB, 9.1×) | 4 | 1.40× (1.35–1.46) | 2.25× (2.15–2.35) | 2.49× (2.41–2.57) | 2.45× (2.37–2.54) | 2.59× (2.54–2.65) | 1.86× (1.84–1.89) | 1.80× (1.78–1.83) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.32× (0.32–0.32) | 0.30× (0.28–0.31) | 1.16× (1.16–1.16) | 0.83× (0.83–0.84) | 11.79× (11.77–11.82) | 0.28× (0.28–0.29) | 0.30× (0.30–0.30) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 74 MB | 0.40× (0.40–0.41) | 0.78× (0.68–0.90) | 0.78× (0.77–0.79) | 0.64× (0.62–0.65) | 4.28× (4.26–4.30) | 0.10× (0.10–0.10) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.81× (0.81–0.82) | 0.80× (0.80–0.80) | 4.29× (4.28–4.29) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.18× (0.18–0.19) | 0.15× (0.15–0.15) | 1.78× (1.78–1.79) | 0.83× (0.82–0.83) | 4.41× (4.40–4.42) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 17 MB | 0.50× (0.49–0.50) | 0.42× (0.42–0.42) | 1.63× (1.63–1.64) | 0.64× (0.63–0.64) | 7.78× (7.76–7.80) | 0.63× (0.63–0.64) | 0.66× (0.65–0.66) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 42 MB | 0.60× (0.58–0.62) | 0.65× (0.61–0.70) | 1.12× (1.09–1.14) | 0.85× (0.84–0.87) | 3.23× (3.17–3.29) | 0.25× (0.25–0.26) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.39× (0.39–0.39) | 0.31× (0.31–0.31) | 1.15× (1.14–1.15) | 0.71× (0.71–0.71) | 5.07× (5.07–5.08) | 0.32× (0.32–0.32) | 0.33× (0.33–0.34) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 54 MB | 0.58× (0.57–0.59) | 0.41× (0.41–0.42) | 0.79× (0.77–0.80) | 0.72× (0.70–0.75) | 2.31× (2.26–2.35) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 24 MB | 0.39× (0.39–0.39) | 0.31× (0.31–0.31) | 1.15× (1.15–1.15) | 0.80× (0.80–0.81) | 5.09× (5.08–5.09) | 0.32× (0.31–0.32) | 0.33× (0.33–0.33) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 57 MB | 0.56× (0.56–0.57) | 0.46× (0.44–0.49) | 0.75× (0.74–0.76) | 0.74× (0.71–0.77) | 2.19× (2.17–2.21) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.20–0.22) | 1.09× (1.08–1.09) | 0.76× (0.76–0.77) | 5.43× (5.42–5.44) | 0.35× (0.35–0.36) | 0.37× (0.37–0.37) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.22× (0.22–0.22) | 0.20× (0.20–0.20) | 1.63× (1.63–1.63) | 0.76× (0.76–0.76) | 5.40× (5.39–5.41) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) |
| `text` (1.32 MB → 12 MB, 9.1×) | 1 | 19 MB | 0.46× (0.45–0.46) | 0.38× (0.38–0.38) | 1.45× (1.44–1.45) | 0.61× (0.61–0.62) | 6.53× (6.52–6.54) | 0.40× (0.40–0.41) | 0.42× (0.42–0.43) |
| `text` (1.32 MB → 12 MB, 9.1×) | 4 | 51 MB | 0.53× (0.52–0.54) | 0.49× (0.45–0.54) | 1.00× (0.99–1.01) | 0.76× (0.74–0.78) | 2.54× (2.51–2.57) | 0.15× (0.15–0.15) | 0.16× (0.15–0.16) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.11 | 1.17 | 0.02 | – | 0.425 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3241 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m15s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.23 | 9.6% |
| MTF + RLE2 | 2.26 | 17.8% |
| T-vector build | 2.31 | 18.1% |
| walk (the one each build uses) | 6.39 | 50.2% |
| of which: segment assembly | 0.21 | 1.7% |
| RLE1 | 0.28 | 2.2% |
| CRC | 0.26 | 2.0% |
| **total** | 12.73 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 2h01m ± 15m02s |
| 4 threads | 1h01m ± 8m42s |
| 4 threads, the sample's overlap gap as a fixed cost | 42m12s ± 6m02s |
| the scanner alone (its serial part) | 15m43s ± 49.3s |
| the workers alone, over 4 | 42m12s ± 6m02s |

At 4 threads the workers bound it. Per byte: the scanner 9.1 ns a byte of bz2, the workers 5.53 ns a byte of output at 4 threads (CPU, summed over them), 3.46 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 11 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,321,515 | 12,045,998 | \`b5d7ffd459153f16…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3241.3 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1017-azure; image ubuntu24 20261026.354.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source) - bench 2.1.1 (bench/report.py), 10 rounds, threads 1, 4
