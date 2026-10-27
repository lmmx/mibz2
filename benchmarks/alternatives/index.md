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

The latest record: [`0392786`](<https://github.com/lmmx/mibz2/commit/039278649c3e366cc8815794fdd229484e4ddb4d>) (2026-10-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2 and libbz2-rs**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40585051850>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 2.1.0

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.773 s | 271 | 9.0 | 1.89× (1.85–1.94) | 4.47× (4.35–4.59) | 4.29× (4.15–4.44) | 4.92× (4.73–5.12) | 4.13× (3.99–4.28) | 4.52× (4.33–4.72) | 4.29× (4.15–4.44) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.388 s | 541 | 15.4 | 1.74× (1.71–1.76) | 2.63× (2.57–2.69) | 3.41× (3.36–3.46) | 2.93× (2.88–2.98) | 2.75× (2.69–2.81) | 9.15× (8.89–9.42) | 8.68× (8.45–8.91) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.015 s | 1,379 | 2.0 | 0.80× (0.75–0.84) | 7.01× (6.66–7.38) | 6.27× (5.95–6.61) | 4.66× (4.40–4.93) | 5.07× (4.81–5.35) | 4.03× (3.80–4.27) | 3.77× (3.58–3.96) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.014 s | 1,404 | 2.0 | 0.88× (0.86–0.90) | 7.10× (6.93–7.27) | 9.87× (9.62–10.13) | 4.75× (4.63–4.88) | 5.20× (5.09–5.32) | 4.11× (3.97–4.25) | 3.84× (3.75–3.93) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.039 s | 506 | 5.1 | 2.35× (2.28–2.43) | 2.73× (2.51–2.97) | 3.63× (3.57–3.70) | 3.31× (3.27–3.36) | 2.55× (2.50–2.59) | 2.16× (2.13–2.20) | 1.92× (1.87–1.97) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.028 s | 699 | 12.3 | 1.58× (1.45–1.72) | 1.74× (1.60–1.88) | 3.11× (2.87–3.36) | 2.01× (1.84–2.19) | 1.76× (1.64–1.90) | 2.88× (2.67–3.11) | 2.55× (2.35–2.78) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.153 s | 52 | 47.4 | 1.13× (1.05–1.22) | 2.49× (2.25–2.76) | 2.17× (2.01–2.33) | 2.08× (1.93–2.24) | 2.08× (1.92–2.25) | 2.60× (2.41–2.80) | 2.67× (2.48–2.88) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.074 s | 108 | 66.5 | 1.27× (1.18–1.37) | 2.12× (1.98–2.27) | 2.15× (1.92–2.41) | 2.42× (2.24–2.60) | 1.96× (1.93–1.99) | 5.54× (5.46–5.61) | 5.68× (5.60–5.78) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.202 s | 48 | 52.0 | 1.06× (1.03–1.09) | 2.47× (2.40–2.54) | 1.92× (1.86–1.98) | 1.86× (1.81–1.91) | 1.98× (1.89–2.08) | 2.68× (2.61–2.76) | 2.91× (2.75–3.08) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.090 s | 106 | 74.4 | 1.20× (1.18–1.23) | 2.24× (2.20–2.28) | 2.12× (2.08–2.16) | 2.62× (2.51–2.74) | 1.85× (1.82–1.89) | 6.05× (5.96–6.14) | 6.55× (6.24–6.89) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 833 | 3.5 | 0.72× (0.70–0.73) | 4.23× (4.10–4.36) | 7.56× (7.30–7.83) | 3.43× (3.34–3.53) | 3.90× (3.81–3.98) | 2.77× (2.71–2.84) | 2.61× (2.55–2.68) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 794 | 3.7 | 0.70× (0.68–0.73) | 4.08× (3.95–4.21) | 7.28× (7.04–7.53) | 3.37× (3.21–3.55) | 3.80× (3.70–3.91) | 2.69× (2.61–2.77) | 2.53× (2.47–2.60) | yes |
| `text` (982 kB → 12 MB, 12.2×) | 1 | 0.069 s | 173 | 14.6 | 1.56× (1.53–1.59) | 3.59× (3.51–3.68) | 3.71× (3.46–3.98) | 3.45× (3.36–3.55) | 3.19× (3.07–3.33) | 3.63× (3.51–3.75) | 3.63× (3.53–3.73) | yes |
| `text` (982 kB → 12 MB, 12.2×) | 4 | 0.038 s | 317 | 26.5 | 1.41× (1.36–1.47) | 2.79× (2.58–3.01) | 3.73× (3.63–3.83) | 3.15× (3.09–3.22) | 2.34× (2.17–2.51) | 6.74× (6.49–7.00) | 6.74× (6.55–6.94) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 1.89× (1.84–1.94) | 4.43× (4.31–4.56) | 4.26× (4.12–4.41) | 4.91× (4.72–5.11) | 4.12× (3.98–4.27) | 4.51× (4.32–4.71) | 4.28× (4.14–4.42) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 1.91× (1.88–1.93) | 2.96× (2.92–3.01) | 3.39× (3.33–3.45) | 3.39× (3.32–3.45) | 2.99× (2.93–3.04) | 2.69× (2.60–2.77) | 2.55× (2.48–2.62) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.82× (0.79–0.86) | 4.47× (4.27–4.68) | 5.32× (5.07–5.58) | 4.18× (3.98–4.39) | 4.54× (4.34–4.75) | 3.63× (3.45–3.83) | 3.40× (3.26–3.55) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.90× (0.88–0.92) | 4.93× (4.81–5.06) | 6.30× (6.13–6.47) | 4.29× (4.19–4.40) | 4.60× (4.49–4.70) | 3.66× (3.54–3.79) | 3.43× (3.35–3.52) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.28× (2.21–2.35) | 2.51× (2.48–2.54) | 2.91× (2.78–3.04) | 3.18× (3.14–3.23) | 2.46× (2.42–2.50) | 2.10× (2.07–2.13) | 1.87× (1.82–1.92) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.76× (1.68–1.85) | 1.66× (1.60–1.72) | 1.92× (1.84–1.99) | 1.90× (1.81–2.00) | 1.37× (1.31–1.43) | 0.88× (0.85–0.92) | 0.79× (0.75–0.82) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.13× (1.05–1.22) | 2.26× (2.09–2.43) | 1.93× (1.78–2.08) | 2.06× (1.91–2.22) | 2.06× (1.91–2.23) | 2.58× (2.39–2.78) | 2.65× (2.46–2.85) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.40× (1.33–1.47) | 2.14× (2.09–2.20) | 1.73× (1.69–1.77) | 2.41× (2.36–2.47) | 2.02× (1.99–2.04) | 1.89× (1.86–1.92) | 1.94× (1.92–1.97) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.06× (1.03–1.09) | 2.23× (2.17–2.30) | 1.73× (1.68–1.77) | 1.85× (1.80–1.90) | 1.97× (1.88–2.07) | 2.66× (2.59–2.74) | 2.89× (2.73–3.05) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.29× (1.25–1.33) | 2.10× (2.03–2.16) | 1.56× (1.54–1.59) | 2.32× (2.28–2.36) | 1.86× (1.83–1.89) | 1.88× (1.85–1.90) | 2.03× (1.94–2.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.76× (0.75–0.78) | 3.06× (2.97–3.14) | 3.76× (3.68–3.83) | 3.05× (2.98–3.12) | 3.44× (3.38–3.51) | 2.49× (2.44–2.55) | 2.36× (2.30–2.42) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.74× (0.72–0.77) | 3.19× (3.12–3.27) | 4.18× (4.03–4.33) | 3.06× (2.93–3.20) | 3.33× (3.24–3.42) | 2.38× (2.31–2.45) | 2.25× (2.19–2.30) |
| `text` (982 kB → 12 MB, 12.2×) | 1 | 1.54× (1.51–1.57) | 3.28× (3.20–3.37) | 3.35× (3.28–3.42) | 3.38× (3.29–3.47) | 3.13× (3.00–3.26) | 3.55× (3.43–3.67) | 3.55× (3.45–3.65) |
| `text` (982 kB → 12 MB, 12.2×) | 4 | 1.55× (1.46–1.64) | 2.19× (2.14–2.23) | 2.52× (2.47–2.57) | 2.57× (2.53–2.60) | 2.18× (2.03–2.35) | 1.96× (1.89–2.03) | 1.96× (1.90–2.02) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 27 MB | 0.37× (0.37–0.37) | 0.28× (0.27–0.30) | 1.12× (1.12–1.12) | 0.85× (0.85–0.86) | 11.41× (11.39–11.44) | 0.31× (0.31–0.31) | 0.32× (0.32–0.32) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.40× (0.40–0.40) | 0.50× (0.48–0.51) | 0.74× (0.73–0.75) | 0.63× (0.62–0.64) | 4.07× (4.04–4.11) | 0.11× (0.11–0.11) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.30× (0.29–0.30) | 0.18× (0.17–0.19) | 0.80× (0.79–0.81) | 0.82× (0.81–0.83) | 4.27× (4.23–4.31) | 0.25× (0.24–0.27) | 0.26× (0.26–0.26) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 35 MB | 0.20× (0.19–0.22) | 0.15× (0.14–0.15) | 1.70× (1.68–1.73) | 0.79× (0.79–0.80) | 4.20× (4.20–4.21) | 0.25× (0.24–0.26) | 0.25× (0.25–0.26) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 18 MB | 0.56× (0.56–0.57) | 0.40× (0.40–0.40) | 1.54× (1.53–1.54) | 0.65× (0.65–0.66) | 7.40× (7.37–7.44) | 0.59× (0.59–0.59) | 0.61× (0.60–0.61) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 43 MB | 0.61× (0.59–0.62) | 0.66× (0.63–0.70) | 1.05× (1.03–1.07) | 0.84× (0.83–0.85) | 3.11× (3.07–3.16) | 0.24× (0.24–0.24) | 0.25× (0.24–0.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 24 MB | 0.45× (0.44–0.45) | 0.29× (0.29–0.30) | 1.10× (1.09–1.10) | 0.72× (0.71–0.73) | 4.87× (4.84–4.91) | 0.35× (0.34–0.35) | 0.36× (0.35–0.36) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 54 MB | 0.60× (0.58–0.62) | 0.42× (0.41–0.43) | 0.78× (0.76–0.81) | 0.73× (0.70–0.77) | 2.31× (2.27–2.35) | 0.16× (0.15–0.16) | 0.16× (0.16–0.17) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 25 MB | 0.44× (0.44–0.45) | 0.30× (0.30–0.30) | 1.10× (1.10–1.11) | 0.81× (0.79–0.82) | 4.92× (4.90–4.95) | 0.34× (0.34–0.34) | 0.36× (0.35–0.36) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 59 MB | 0.56× (0.54–0.57) | 0.44× (0.41–0.47) | 0.73× (0.72–0.74) | 0.74× (0.71–0.77) | 2.11× (2.08–2.14) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 24 MB | 0.41× (0.41–0.42) | 0.19× (0.19–0.19) | 1.05× (1.03–1.07) | 0.76× (0.73–0.79) | 5.14× (5.04–5.24) | 0.32× (0.32–0.33) | 0.34× (0.33–0.35) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 26 MB | 0.26× (0.24–0.28) | 0.17× (0.17–0.17) | 1.49× (1.47–1.52) | 0.72× (0.70–0.74) | 4.79× (4.77–4.80) | 0.30× (0.30–0.30) | 0.32× (0.32–0.32) |
| `text` (982 kB → 12 MB, 12.2×) | 1 | 20 MB | 0.52× (0.52–0.52) | 0.37× (0.37–0.37) | 1.38× (1.37–1.38) | 0.63× (0.62–0.65) | 6.28× (6.26–6.31) | 0.43× (0.42–0.43) | 0.45× (0.44–0.45) |
| `text` (982 kB → 12 MB, 12.2×) | 4 | 51 MB | 0.53× (0.52–0.54) | 0.48× (0.44–0.52) | 1.00× (0.98–1.02) | 0.78× (0.77–0.80) | 2.51× (2.47–2.55) | 0.17× (0.16–0.17) | 0.17× (0.17–0.18) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | wall |
| --- | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 1.03 | 0.02 | 0.388 |

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2445 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m06s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.92 | 10.8% |
| MTF + RLE2 | 1.69 | 19.8% |
| T-vector build | 1.69 | 19.8% |
| walk (the one each build uses) | 3.82 | 44.8% |
| of which: segment assembly | 0.13 | 1.5% |
| RLE1 | 0.21 | 2.5% |
| CRC | 0.19 | 2.3% |
| **total** | 8.51 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h54m ± 14m41s |
| 4 threads | 56m35s ± 7m59s |
| 4 threads, the sample's overlap gap as a fixed cost | 37m30s ± 5m16s |
| the scanner alone (its serial part) | 14m16s ± 39.5s |
| the workers alone, over 4 | 37m30s ± 5m16s |

At 4 threads the workers bound it. Per byte: the scanner 8.3 ns a byte of bz2, the workers 4.92 ns a byte of output at 4 threads (CPU, summed over them), 3.29 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 11 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 982,167 | 12,010,659 | \`677bf0735616169b…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2445.4 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 6.17.0-1022-azure; image ubuntu24 20261023.334.3 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source) - bench 2.1.0 (bench/report.py), 10 rounds, threads 1, 4
