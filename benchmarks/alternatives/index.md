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

The latest record: [`acda268`](<https://github.com/lmmx/mibz2/commit/acda26897c6f8e0788b683ebfce77325f9c5947e>) (2026-10-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2 and libbz2-rs**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40609227203>)

INTEL(R) XEON(R) PLATINUM 8573C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 2.1.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.739 s | 284 | 12.7 | 1.94× (1.94–1.94) | 4.28× (4.28–4.28) | 4.30× (4.26–4.34) | 5.13× (5.12–5.14) | 4.57× (4.56–4.58) | 4.73× (4.62–4.85) | 4.60× (4.49–4.71) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.329 s | 638 | 18.8 | 2.03× (2.00–2.07) | 3.49× (3.45–3.54) | 4.08× (4.02–4.14) | 3.93× (3.89–3.98) | 3.70× (3.66–3.75) | 10.60× (10.34–10.86) | 10.30× (10.05–10.56) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.014 s | 1,424 | 2.8 | 0.70× (0.64–0.77) | 7.68× (6.96–8.48) | 7.09× (6.42–7.83) | 4.72× (4.24–5.25) | 5.45× (4.91–6.05) | 3.69× (3.34–4.08) | 3.69× (3.35–4.08) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.014 s | 1,404 | 2.8 | 0.72× (0.66–0.79) | 3.97× (3.34–4.72) | 6.76× (6.17–7.42) | 4.69× (4.28–5.14) | 5.36× (4.90–5.86) | 3.59× (3.26–3.94) | 3.59× (3.26–3.95) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.038 s | 511 | 7.3 | 2.39× (2.33–2.45) | 2.63× (2.57–2.70) | 3.74× (3.66–3.83) | 3.73× (3.63–3.82) | 3.25× (3.17–3.32) | 2.71× (2.64–2.77) | 2.46× (2.41–2.52) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.022 s | 895 | 14.0 | 1.91× (1.79–2.04) | 2.27× (2.12–2.42) | 4.14× (3.88–4.43) | 2.42× (2.21–2.65) | 2.47× (2.30–2.66) | 4.67× (4.37–4.99) | 4.25× (3.98–4.55) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.126 s | 64 | 57.3 | 1.33× (1.32–1.35) | 3.19× (3.15–3.22) | 2.72× (2.69–2.75) | 2.61× (2.58–2.65) | 2.65× (2.61–2.69) | 3.27× (3.23–3.31) | 3.40× (3.36–3.44) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.059 s | 137 | 81.9 | 1.43× (1.38–1.47) | 2.99× (2.64–3.39) | 2.60× (2.38–2.85) | 3.27× (3.17–3.37) | 2.54× (2.46–2.63) | 7.09× (6.88–7.30) | 7.37× (7.14–7.59) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.160 s | 60 | 60.5 | 1.22× (1.16–1.28) | 3.12× (3.03–3.22) | 2.44× (2.37–2.52) | 2.47× (2.40–2.55) | 2.57× (2.49–2.65) | 3.45× (3.35–3.56) | 3.69× (3.58–3.81) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.070 s | 137 | 83.5 | 1.40× (1.36–1.45) | 2.89× (2.82–2.96) | 2.75× (2.69–2.83) | 3.54× (3.38–3.70) | 2.51× (2.44–2.57) | 7.94× (7.74–8.15) | 8.50× (8.28–8.72) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.009 s | 1,087 | 3.9 | 0.73× (0.69–0.76) | 5.62× (5.24–6.02) | 4.81× (4.47–5.16) | 4.02× (3.76–4.29) | 5.07× (4.72–5.45) | 3.06× (2.84–3.29) | 3.05× (2.85–3.26) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 948 | 4.4 | 0.63× (0.61–0.65) | 4.74× (4.59–4.91) | 8.66× (8.37–8.96) | 3.52× (3.39–3.65) | 4.47× (4.31–4.63) | 2.60× (2.51–2.70) | 2.59× (2.51–2.67) | yes |
| `text` (1.24 MB → 12.2 MB, 9.9×) | 1 | 0.068 s | 179 | 20.5 | 1.41× (1.39–1.44) | 3.71× (3.64–3.77) | 4.33× (4.25–4.40) | 3.99× (3.92–4.06) | 3.89× (3.82–3.96) | 4.13× (4.06–4.20) | 4.15× (4.07–4.24) | yes |
| `text` (1.24 MB → 12.2 MB, 9.9×) | 4 | 0.032 s | 379 | 32.1 | 1.41× (1.36–1.47) | 3.22× (2.99–3.47) | 4.37× (4.20–4.55) | 4.00× (3.85–4.16) | 3.16× (3.04–3.27) | 8.58× (8.25–8.93) | 8.63× (8.28–9.00) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 1.94× (1.93–1.94) | 4.24× (4.23–4.25) | 4.26× (4.22–4.29) | 5.12× (5.11–5.13) | 4.56× (4.55–4.57) | 4.72× (4.61–4.84) | 4.59× (4.49–4.70) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.30× (2.27–2.32) | 4.04× (4.00–4.08) | 4.33× (4.29–4.38) | 4.68× (4.63–4.73) | 4.16× (4.12–4.20) | 3.19× (3.11–3.28) | 3.10× (3.04–3.17) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.73× (0.67–0.80) | 3.70× (3.36–4.07) | 4.68× (4.24–5.16) | 4.35× (3.93–4.81) | 5.00× (4.53–5.52) | 3.42× (3.12–3.76) | 3.43× (3.12–3.76) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.74× (0.69–0.80) | 4.23× (3.90–4.59) | 5.48× (5.08–5.92) | 4.35× (4.01–4.71) | 4.88× (4.51–5.28) | 3.32× (3.04–3.61) | 3.32× (3.04–3.62) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.34× (2.29–2.40) | 2.05× (2.00–2.10) | 2.46× (2.40–2.52) | 3.63× (3.54–3.72) | 3.17× (3.10–3.23) | 2.65× (2.58–2.71) | 2.41× (2.36–2.47) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.10× (2.00–2.19) | 1.64× (1.56–1.72) | 1.94× (1.85–2.03) | 2.22× (2.12–2.32) | 1.81× (1.74–1.89) | 1.38× (1.32–1.44) | 1.26× (1.20–1.31) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.33× (1.31–1.35) | 2.83× (2.80–2.86) | 2.38× (2.34–2.41) | 2.59× (2.56–2.63) | 2.63× (2.59–2.67) | 3.25× (3.21–3.29) | 3.37× (3.34–3.41) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.58× (1.53–1.63) | 2.88× (2.78–2.97) | 2.37× (2.31–2.43) | 3.21× (3.13–3.28) | 2.56× (2.49–2.63) | 2.32× (2.26–2.39) | 2.41× (2.34–2.49) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.22× (1.16–1.27) | 2.87× (2.78–2.97) | 2.24× (2.16–2.31) | 2.46× (2.38–2.54) | 2.56× (2.48–2.64) | 3.43× (3.32–3.53) | 3.67× (3.56–3.78) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.56× (1.51–1.61) | 3.01× (2.93–3.09) | 2.20× (2.11–2.29) | 3.20× (3.06–3.34) | 2.52× (2.46–2.59) | 2.50× (2.43–2.57) | 2.67× (2.60–2.74) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.76× (0.73–0.79) | 3.12× (2.93–3.32) | 3.89× (3.68–4.11) | 3.60× (3.39–3.82) | 4.50× (4.22–4.81) | 2.77× (2.59–2.96) | 2.76× (2.60–2.94) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.67× (0.64–0.69) | 3.10× (2.87–3.34) | 3.99× (3.83–4.16) | 3.27× (3.16–3.39) | 4.03× (3.90–4.17) | 2.39× (2.31–2.47) | 2.38× (2.32–2.45) |
| `text` (1.24 MB → 12.2 MB, 9.9×) | 1 | 1.41× (1.38–1.43) | 3.59× (3.53–3.66) | 3.80× (3.70–3.89) | 3.93× (3.86–4.00) | 3.83× (3.76–3.90) | 4.07× (4.00–4.13) | 4.09× (4.00–4.18) |
| `text` (1.24 MB → 12.2 MB, 9.9×) | 4 | 1.58× (1.50–1.67) | 3.09× (2.96–3.22) | 3.39× (3.29–3.49) | 3.49× (3.38–3.60) | 3.15× (3.06–3.24) | 2.59× (2.51–2.68) | 2.61× (2.53–2.69) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.37× (0.36–0.38) | 0.30× (0.28–0.31) | 1.11× (1.11–1.11) | 0.85× (0.85–0.85) | 11.35× (11.33–11.38) | 0.31× (0.31–0.31) | 0.32× (0.32–0.33) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 78 MB | 0.40× (0.39–0.41) | 0.47× (0.45–0.48) | 0.73× (0.72–0.74) | 0.63× (0.62–0.64) | 4.07× (4.04–4.10) | 0.11× (0.11–0.11) | 0.11× (0.11–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.30× (0.29–0.31) | 0.17× (0.16–0.18) | 0.79× (0.78–0.80) | 0.80× (0.79–0.81) | 4.23× (4.18–4.28) | 0.25× (0.24–0.25) | 0.26× (0.25–0.26) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 35 MB | 0.21× (0.20–0.23) | 0.15× (0.15–0.15) | 1.72× (1.70–1.74) | 0.80× (0.80–0.81) | 4.21× (4.20–4.21) | 0.24× (0.24–0.25) | 0.26× (0.25–0.26) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 18 MB | 0.57× (0.56–0.57) | 0.40× (0.39–0.41) | 1.53× (1.52–1.53) | 0.66× (0.65–0.66) | 7.39× (7.35–7.43) | 0.59× (0.59–0.59) | 0.62× (0.61–0.62) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 43 MB | 0.61× (0.60–0.61) | 0.60× (0.55–0.65) | 1.05× (1.03–1.07) | 0.85× (0.84–0.87) | 3.12× (3.08–3.17) | 0.24× (0.24–0.25) | 0.25× (0.25–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 24 MB | 0.45× (0.45–0.45) | 0.30× (0.30–0.30) | 1.09× (1.09–1.10) | 0.72× (0.72–0.72) | 4.89× (4.88–4.91) | 0.35× (0.35–0.35) | 0.36× (0.36–0.36) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 54 MB | 0.60× (0.58–0.61) | 0.42× (0.42–0.43) | 0.79× (0.78–0.80) | 0.75× (0.73–0.77) | 2.31× (2.28–2.34) | 0.16× (0.16–0.16) | 0.17× (0.16–0.17) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 25 MB | 0.44× (0.43–0.45) | 0.30× (0.30–0.30) | 1.10× (1.10–1.10) | 0.81× (0.81–0.81) | 4.93× (4.92–4.95) | 0.34× (0.34–0.35) | 0.36× (0.36–0.36) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 58 MB | 0.56× (0.55–0.58) | 0.43× (0.41–0.44) | 0.74× (0.73–0.75) | 0.76× (0.74–0.77) | 2.14× (2.10–2.18) | 0.15× (0.14–0.15) | 0.15× (0.15–0.16) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 24 MB | 0.43× (0.42–0.43) | 0.21× (0.20–0.23) | 1.09× (1.06–1.12) | 0.77× (0.75–0.80) | 5.22× (5.14–5.30) | 0.33× (0.33–0.34) | 0.35× (0.34–0.36) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 26 MB | 0.27× (0.25–0.29) | 0.18× (0.18–0.18) | 1.51× (1.49–1.53) | 0.72× (0.70–0.75) | 4.83× (4.75–4.91) | 0.31× (0.30–0.31) | 0.32× (0.32–0.33) |
| `text` (1.24 MB → 12.2 MB, 9.9×) | 1 | 20 MB | 0.52× (0.52–0.53) | 0.38× (0.37–0.38) | 1.38× (1.38–1.39) | 0.59× (0.57–0.61) | 6.33× (6.30–6.36) | 0.43× (0.43–0.44) | 0.45× (0.45–0.46) |
| `text` (1.24 MB → 12.2 MB, 9.9×) | 4 | 49 MB | 0.57× (0.56–0.58) | 0.47× (0.45–0.49) | 1.04× (1.02–1.06) | 0.75× (0.72–0.77) | 2.65× (2.61–2.69) | 0.18× (0.17–0.18) | 0.18× (0.18–0.19) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | wall |
| --- | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.84 | 0.02 | 0.329 |

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3600 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m02s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.21 | 10.0% |
| MTF + RLE2 | 2.21 | 18.2% |
| T-vector build | 2.44 | 20.1% |
| walk (the one each build uses) | 5.86 | 48.4% |
| of which: segment assembly | 0.29 | 2.4% |
| RLE1 | 0.26 | 2.2% |
| CRC | 0.13 | 1.1% |
| **total** | 12.11 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h47m ± 13m22s |
| 4 threads | 48m01s ± 6m45s |
| 4 threads, the sample's overlap gap as a fixed cost | 30m44s ± 4m21s |
| the scanner alone (its serial part) | 12m05s ± 30.2s |
| the workers alone, over 4 | 30m44s ± 4m21s |

At 4 threads the workers bound it. Per byte: the scanner 7.0 ns a byte of bz2, the workers 4.03 ns a byte of output at 4 threads (CPU, summed over them), 3.13 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,236,217 | 12,227,335 | \`294c8c24e2835ff3…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: INTEL(R) XEON(R) PLATINUM 8573C, 4 cores, reported clock 3600.0 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 6.17.0-1022-azure; image ubuntu24 20261023.334.3 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source) - bench 2.1.1 (bench/report.py), 10 rounds, threads 1, 4
