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

The latest record: [`8e26834`](<https://github.com/lmmx/mibz2/commit/8e268349ca6924be4cdedcf29129804e7675ae3d>) (2026-10-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2 and libbz2-rs**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40590654799>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 2.1.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.413 s | 507 | 8.4 | 2.01× (1.99–2.03) | 5.24× (5.15–5.33) | 5.12× (4.95–5.29) | 6.25× (6.15–6.35) | 5.34× (5.25–5.43) | 5.61× (5.45–5.78) | 5.12× (4.94–5.30) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.275 s | 761 | 18.2 | 1.41× (1.24–1.60) | 2.30× (2.00–2.63) | 2.88× (2.50–3.33) | 2.61× (2.28–2.99) | 2.44× (2.13–2.81) | 7.95× (7.01–9.00) | 7.25× (6.48–8.11) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.009 s | 2,128 | 2.4 | 0.75× (0.72–0.78) | 5.61× (4.75–6.63) | 9.72× (9.34–10.13) | 5.09× (4.92–5.26) | 5.38× (5.21–5.56) | 4.10× (3.96–4.25) | 4.10× (3.96–4.24) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.010 s | 2,030 | 2.5 | 0.74× (0.70–0.77) | 5.40× (4.61–6.31) | 9.72× (8.75–10.79) | 5.27× (4.57–6.08) | 5.73× (4.76–6.90) | 3.98× (3.80–4.16) | 3.97× (3.79–4.16) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.021 s | 924 | 5.0 | 2.15× (1.67–2.79) | 4.35× (3.43–5.53) | 4.03× (3.16–5.14) | 3.71× (2.87–4.78) | 2.90× (2.25–3.73) | 2.26× (1.75–2.91) | 2.03× (1.58–2.61) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.018 s | 1,083 | 13.7 | 1.44× (1.38–1.51) | 2.73× (2.62–2.86) | 2.51× (2.06–3.04) | 1.99× (1.91–2.07) | 1.94× (1.64–2.30) | 2.87× (2.77–2.98) | 2.58× (2.47–2.69) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.088 s | 90 | 48.0 | 1.19× (1.08–1.31) | 3.01× (2.66–3.40) | 2.66× (2.43–2.90) | 2.44× (2.22–2.69) | 2.41× (2.19–2.64) | 3.32× (2.97–3.72) | 3.11× (2.83–3.43) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.043 s | 185 | 71.2 | 1.22× (1.18–1.27) | 2.42× (2.19–2.68) | 3.29× (3.17–3.41) | 2.77× (2.69–2.84) | 2.18× (2.10–2.26) | 7.00× (6.59–7.44) | 6.56× (6.34–6.79) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.116 s | 83 | 52.3 | 1.07× (1.05–1.09) | 3.04× (2.97–3.11) | 2.14× (2.08–2.21) | 2.28× (2.24–2.31) | 2.30× (2.27–2.32) | 3.45× (3.39–3.52) | 3.50× (3.43–3.58) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.053 s | 180 | 76.8 | 1.16× (1.13–1.18) | 2.81× (2.73–2.89) | 2.66× (2.59–2.74) | 2.90× (2.82–2.99) | 2.02× (1.96–2.07) | 7.44× (7.23–7.66) | 7.55× (7.32–7.78) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.007 s | 1,361 | 3.9 | 0.71× (0.68–0.75) | 6.92× (6.60–7.26) | 6.02× (5.73–6.32) | 3.92× (3.73–4.11) | 4.39× (4.20–4.58) | 2.98× (2.82–3.15) | 3.00× (2.87–3.15) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.008 s | 1,235 | 4.2 | 0.61× (0.59–0.63) | 6.13× (5.98–6.28) | 5.21× (5.07–5.34) | 3.54× (3.45–3.64) | 4.21× (3.67–4.84) | 2.66× (2.59–2.72) | 2.68× (2.62–2.75) | yes |
| `text` (982 kB → 12 MB, 12.2×) | 1 | 0.040 s | 301 | 14.8 | 1.50× (1.48–1.52) | 3.83× (3.78–3.88) | 3.65× (3.60–3.69) | 4.00× (3.91–4.10) | 3.72× (3.69–3.75) | 3.87× (3.82–3.92) | 3.64× (3.57–3.73) | yes |
| `text` (982 kB → 12 MB, 12.2×) | 4 | 0.024 s | 510 | 28.8 | 1.19× (1.15–1.22) | 2.24× (1.95–2.57) | 4.01× (3.71–4.34) | 3.55× (3.01–4.18) | 2.30× (2.23–2.38) | 6.35× (6.15–6.56) | 5.99× (5.72–6.27) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.00× (1.98–2.02) | 5.18× (5.10–5.25) | 5.06× (4.90–5.23) | 6.22× (6.12–6.33) | 5.32× (5.23–5.42) | 5.59× (5.43–5.76) | 5.10× (4.92–5.28) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 1.57× (1.36–1.81) | 2.61× (2.23–3.05) | 2.96× (2.53–3.45) | 3.13× (2.69–3.65) | 2.75× (2.35–3.21) | 2.42× (2.10–2.79) | 2.21× (1.94–2.51) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.77× (0.73–0.81) | 4.35× (3.89–4.87) | 5.00× (4.72–5.31) | 4.40× (4.17–4.64) | 4.64× (4.40–4.90) | 3.57× (3.38–3.77) | 3.57× (3.38–3.77) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.77× (0.73–0.80) | 4.58× (3.99–5.25) | 5.83× (5.29–6.42) | 4.71× (4.07–5.45) | 5.05× (4.19–6.09) | 3.55× (3.42–3.69) | 3.55× (3.41–3.69) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.07× (1.62–2.65) | 2.60× (2.04–3.31) | 2.90× (2.29–3.68) | 3.50× (2.74–4.45) | 2.77× (2.17–3.53) | 2.15× (1.69–2.73) | 1.95× (1.54–2.49) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.59× (1.51–1.66) | 1.51× (1.43–1.60) | 1.78× (1.70–1.85) | 1.85× (1.77–1.93) | 1.46× (1.27–1.68) | 0.88× (0.84–0.93) | 0.80× (0.76–0.84) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.19× (1.08–1.31) | 2.74× (2.48–3.03) | 2.22× (2.02–2.43) | 2.42× (2.19–2.66) | 2.38× (2.17–2.61) | 3.28× (2.94–3.66) | 3.07× (2.79–3.38) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.34× (1.31–1.36) | 2.40× (2.32–2.48) | 2.02× (1.98–2.07) | 2.64× (2.59–2.68) | 2.18× (2.13–2.23) | 2.30× (2.20–2.40) | 2.15× (2.10–2.21) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.07× (1.05–1.09) | 2.69× (2.65–2.73) | 2.06× (2.04–2.08) | 2.26× (2.22–2.29) | 2.28× (2.25–2.30) | 3.42× (3.36–3.48) | 3.47× (3.39–3.54) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.25× (1.21–1.28) | 2.40× (2.35–2.45) | 1.78× (1.73–1.83) | 2.62× (2.55–2.68) | 2.08× (2.05–2.10) | 2.32× (2.28–2.36) | 2.35× (2.31–2.40) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.76× (0.72–0.79) | 3.17× (3.03–3.31) | 3.84× (3.67–4.02) | 3.37× (3.24–3.51) | 3.75× (3.62–3.89) | 2.60× (2.48–2.74) | 2.63× (2.53–2.73) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.67× (0.65–0.69) | 3.02× (2.94–3.10) | 4.04× (3.91–4.18) | 3.21× (3.14–3.28) | 3.71× (3.26–4.23) | 2.40× (2.35–2.45) | 2.41× (2.36–2.47) |
| `text` (982 kB → 12 MB, 12.2×) | 1 | 1.48× (1.46–1.50) | 3.47× (3.43–3.52) | 3.43× (3.37–3.48) | 3.88× (3.79–3.97) | 3.61× (3.58–3.64) | 3.75× (3.70–3.80) | 3.54× (3.46–3.61) |
| `text` (982 kB → 12 MB, 12.2×) | 4 | 1.34× (1.26–1.42) | 2.18× (2.12–2.25) | 2.60× (2.34–2.90) | 2.97× (2.50–3.53) | 2.22× (2.15–2.29) | 1.94× (1.87–2.01) | 1.83× (1.76–1.89) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.37× (0.37–0.37) | 0.33× (0.33–0.34) | 1.11× (1.11–1.11) | 0.85× (0.85–0.86) | 11.37× (11.34–11.40) | 0.31× (0.31–0.32) | 0.32× (0.32–0.33) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 78 MB | 0.40× (0.40–0.40) | 0.49× (0.48–0.50) | 0.73× (0.72–0.74) | 0.62× (0.60–0.63) | 4.06× (4.02–4.09) | 0.11× (0.11–0.11) | 0.11× (0.11–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.30× (0.29–0.31) | 0.17× (0.16–0.18) | 0.79× (0.79–0.80) | 0.80× (0.80–0.81) | 4.23× (4.20–4.25) | 0.25× (0.25–0.25) | 0.26× (0.25–0.26) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 35 MB | 0.20× (0.20–0.20) | 0.15× (0.15–0.15) | 1.71× (1.68–1.73) | 0.80× (0.79–0.81) | 4.21× (4.19–4.23) | 0.24× (0.24–0.25) | 0.26× (0.25–0.26) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 18 MB | 0.57× (0.56–0.58) | 0.40× (0.40–0.41) | 1.53× (1.53–1.54) | 0.66× (0.66–0.66) | 7.42× (7.38–7.45) | 0.59× (0.59–0.59) | 0.60× (0.60–0.61) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 44 MB | 0.61× (0.59–0.62) | 0.58× (0.54–0.63) | 1.05× (1.03–1.07) | 0.84× (0.83–0.85) | 3.11× (3.07–3.15) | 0.24× (0.24–0.24) | 0.25× (0.24–0.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 25 MB | 0.45× (0.45–0.45) | 0.30× (0.30–0.30) | 1.09× (1.09–1.10) | 0.72× (0.72–0.73) | 4.89× (4.86–4.91) | 0.35× (0.35–0.35) | 0.36× (0.36–0.37) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 52 MB | 0.61× (0.59–0.63) | 0.43× (0.41–0.44) | 0.81× (0.79–0.83) | 0.77× (0.75–0.80) | 2.37× (2.32–2.43) | 0.16× (0.16–0.17) | 0.17× (0.17–0.17) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 25 MB | 0.44× (0.44–0.45) | 0.30× (0.30–0.30) | 1.10× (1.09–1.10) | 0.81× (0.80–0.82) | 4.94× (4.92–4.96) | 0.34× (0.34–0.35) | 0.36× (0.36–0.36) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 61 MB | 0.55× (0.53–0.57) | 0.41× (0.38–0.43) | 0.71× (0.70–0.73) | 0.74× (0.71–0.77) | 2.09× (2.05–2.13) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 24 MB | 0.42× (0.41–0.43) | 0.21× (0.20–0.22) | 1.07× (1.03–1.10) | 0.76× (0.73–0.79) | 5.18× (5.08–5.28) | 0.33× (0.32–0.34) | 0.35× (0.34–0.35) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 26 MB | 0.25× (0.25–0.25) | 0.18× (0.17–0.18) | 1.47× (1.44–1.50) | 0.71× (0.69–0.73) | 4.77× (4.75–4.80) | 0.30× (0.30–0.31) | 0.32× (0.32–0.32) |
| `text` (982 kB → 12 MB, 12.2×) | 1 | 20 MB | 0.52× (0.51–0.52) | 0.38× (0.37–0.38) | 1.37× (1.37–1.38) | 0.62× (0.60–0.64) | 6.29× (6.26–6.32) | 0.43× (0.43–0.44) | 0.45× (0.45–0.45) |
| `text` (982 kB → 12 MB, 12.2×) | 4 | 52 MB | 0.52× (0.52–0.53) | 0.50× (0.45–0.54) | 0.97× (0.95–0.99) | 0.76× (0.73–0.80) | 2.48× (2.45–2.52) | 0.16× (0.16–0.17) | 0.17× (0.17–0.17) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | wall |
| --- | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.70 | 0.01 | 0.275 |

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4259 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 38.1s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.05 | 13.3% |
| MTF + RLE2 | 1.90 | 24.2% |
| T-vector build | 1.44 | 18.3% |
| walk (the one each build uses) | 3.08 | 39.1% |
| of which: segment assembly | 0.10 | 1.3% |
| RLE1 | 0.15 | 1.9% |
| CRC | 0.26 | 3.2% |
| **total** | 7.87 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 59m50s ± 7m04s |
| 4 threads | 42m58s ± 9m16s |
| 4 threads, the sample's overlap gap as a fixed cost | 28m03s ± 7m09s |
| the scanner alone (its serial part) | 9m45s ± 2m21s |
| the workers alone, over 4 | 28m03s ± 7m09s |

At 4 threads the workers bound it. Per byte: the scanner 5.7 ns a byte of bz2, the workers 3.68 ns a byte of output at 4 threads (CPU, summed over them), 1.64 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 12 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 982,167 | 12,010,659 | \`677bf0735616169b…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4259.3 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 6.17.0-1022-azure; image ubuntu24 20261023.334.3 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source) - bench 2.1.1 (bench/report.py), 10 rounds, threads 1, 4
