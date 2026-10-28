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

The latest record: [`0a032fa`](<https://github.com/lmmx/mibz2/commit/0a032fa4c942415355d260b3c084bada11cb8a94>) (2026-10-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2 and libbz2-rs**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40606972399>)

INTEL(R) XEON(R) PLATINUM 8573C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 2.1.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.829 s | 253 | 11.5 | 1.82× (1.80–1.84) | 4.04× (3.92–4.18) | 4.14× (4.09–4.18) | 4.93× (4.84–5.03) | 4.32× (4.24–4.41) | 4.48× (4.41–4.55) | 4.40× (4.31–4.50) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.388 s | 541 | 18.2 | 1.91× (1.87–1.95) | 3.17× (3.11–3.24) | 3.79× (3.70–3.87) | 3.70× (3.61–3.78) | 3.41× (3.34–3.49) | 9.69× (9.48–9.90) | 9.53× (9.30–9.77) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.016 s | 1,254 | 2.6 | 0.70× (0.68–0.72) | 6.41× (6.27–6.56) | 5.81× (5.67–5.95) | 4.65× (4.55–4.75) | 5.19× (5.08–5.31) | 3.60× (3.51–3.69) | 3.59× (3.51–3.68) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.016 s | 1,282 | 2.5 | 0.72× (0.71–0.73) | 6.45× (6.35–6.55) | 9.02× (8.88–9.17) | 4.74× (4.67–4.80) | 5.38× (5.30–5.45) | 3.66× (3.60–3.72) | 3.65× (3.60–3.71) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.043 s | 461 | 6.6 | 2.40× (2.38–2.43) | 2.38× (2.37–2.40) | 3.34× (3.32–3.37) | 3.91× (3.88–3.94) | 3.34× (3.32–3.37) | 2.85× (2.83–2.88) | 2.60× (2.58–2.62) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 s | 776 | 13.3 | 1.89× (1.82–1.95) | 1.97× (1.90–2.05) | 3.57× (3.43–3.72) | 2.46× (2.28–2.65) | 2.45× (2.35–2.56) | 4.74× (4.56–4.94) | 4.32× (4.16–4.50) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.147 s | 54 | 54.4 | 1.30× (1.29–1.30) | 2.73× (2.71–2.75) | 2.32× (2.31–2.34) | 2.34× (2.32–2.35) | 2.40× (2.38–2.41) | 3.06× (3.04–3.09) | 3.17× (3.15–3.19) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.070 s | 114 | 78.0 | 1.37× (1.33–1.41) | 2.85× (2.78–2.91) | 2.71× (2.64–2.78) | 2.97× (2.87–3.07) | 2.28× (2.22–2.34) | 6.37× (6.19–6.55) | 6.60× (6.44–6.76) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.186 s | 52 | 57.2 | 1.07× (0.96–1.21) | 2.46× (2.19–2.76) | 1.92× (1.71–2.16) | 2.04× (1.81–2.29) | 2.14× (1.90–2.40) | 2.98× (2.66–3.35) | 3.18× (2.83–3.57) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.082 s | 117 | 81.0 | 1.36× (1.32–1.40) | 3.07× (2.99–3.15) | 2.44× (2.25–2.65) | 3.36× (3.18–3.54) | 2.30× (2.24–2.37) | 7.40× (7.22–7.59) | 7.89× (7.69–8.10) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 862 | 3.9 | 0.69× (0.66–0.71) | 4.41× (4.29–4.54) | 8.00× (7.77–8.23) | 3.71× (3.60–3.82) | 4.56× (4.42–4.71) | 2.85× (2.76–2.95) | 2.86× (2.77–2.95) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.012 s | 806 | 4.1 | 0.64× (0.62–0.65) | 4.08× (4.00–4.17) | 7.36× (7.20–7.52) | 3.53× (3.45–3.61) | 4.30× (4.20–4.40) | 2.66× (2.59–2.72) | 2.66× (2.60–2.72) | yes |
| `text` (1.23 MB → 12.2 MB, 9.9×) | 1 | 0.078 s | 156 | 19.1 | 1.34× (1.32–1.35) | 3.85× (3.81–3.90) | 3.73× (3.69–3.77) | 3.60× (3.52–3.67) | 3.52× (3.48–3.56) | 3.80× (3.76–3.84) | 3.83× (3.79–3.87) | yes |
| `text` (1.23 MB → 12.2 MB, 9.9×) | 4 | 0.037 s | 327 | 30.9 | 1.40× (1.37–1.42) | 3.29× (2.82–3.83) | 3.77× (3.71–3.84) | 3.88× (3.78–3.98) | 2.98× (2.93–3.03) | 7.92× (7.78–8.06) | 7.99× (7.86–8.13) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 1.82× (1.80–1.84) | 4.00× (3.88–4.13) | 4.10× (4.06–4.15) | 4.92× (4.83–5.02) | 4.32× (4.24–4.40) | 4.47× (4.40–4.54) | 4.40× (4.30–4.50) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.10× (2.07–2.13) | 3.64× (3.57–3.70) | 3.98× (3.90–4.06) | 4.29× (4.21–4.38) | 3.73× (3.65–3.82) | 2.86× (2.79–2.92) | 2.81× (2.72–2.90) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.73× (0.72–0.75) | 3.56× (3.48–3.64) | 4.56× (4.46–4.65) | 4.26× (4.17–4.36) | 4.74× (4.64–4.85) | 3.33× (3.25–3.41) | 3.32× (3.25–3.39) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.76× (0.75–0.76) | 4.22× (4.17–4.27) | 5.58× (5.52–5.64) | 4.40× (4.35–4.45) | 4.90× (4.85–4.96) | 3.38× (3.33–3.43) | 3.37× (3.32–3.42) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.35× (2.32–2.37) | 2.15× (2.13–2.17) | 2.55× (2.53–2.58) | 3.79× (3.76–3.82) | 3.25× (3.22–3.27) | 2.78× (2.75–2.80) | 2.53× (2.52–2.56) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.03× (1.96–2.10) | 1.62× (1.55–1.70) | 1.93× (1.85–2.00) | 2.23× (2.17–2.31) | 1.78× (1.72–1.84) | 1.37× (1.32–1.42) | 1.25× (1.21–1.29) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.29× (1.29–1.30) | 2.60× (2.58–2.61) | 2.11× (2.09–2.13) | 2.32× (2.31–2.34) | 2.38× (2.36–2.39) | 3.04× (3.01–3.07) | 3.15× (3.13–3.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.55× (1.53–1.57) | 2.65× (2.58–2.71) | 2.10× (2.04–2.16) | 2.86× (2.77–2.95) | 2.32× (2.30–2.33) | 2.12× (2.09–2.15) | 2.20× (2.18–2.22) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.07× (0.96–1.21) | 2.40× (2.14–2.70) | 1.83× (1.63–2.05) | 2.03× (1.80–2.28) | 2.13× (1.90–2.39) | 2.97× (2.64–3.33) | 3.16× (2.81–3.55) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.48× (1.46–1.50) | 2.81× (2.79–2.84) | 2.03× (2.01–2.06) | 2.95× (2.85–3.05) | 2.29× (2.27–2.31) | 2.29× (2.27–2.31) | 2.44× (2.42–2.46) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.73× (0.70–0.75) | 2.77× (2.69–2.86) | 3.60× (3.51–3.70) | 3.34× (3.26–3.42) | 4.09× (3.98–4.20) | 2.60× (2.53–2.67) | 2.61× (2.54–2.68) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.68× (0.67–0.69) | 2.93× (2.88–2.98) | 4.08× (3.99–4.17) | 3.28× (3.22–3.34) | 3.89× (3.82–3.97) | 2.44× (2.39–2.49) | 2.45× (2.40–2.49) |
| `text` (1.23 MB → 12.2 MB, 9.9×) | 1 | 1.33× (1.32–1.34) | 3.24× (3.20–3.28) | 3.44× (3.40–3.47) | 3.54× (3.46–3.61) | 3.46× (3.43–3.50) | 3.73× (3.69–3.77) | 3.77× (3.72–3.81) |
| `text` (1.23 MB → 12.2 MB, 9.9×) | 4 | 1.52× (1.47–1.57) | 2.71× (2.64–2.78) | 3.11× (3.05–3.16) | 3.21× (3.17–3.25) | 2.80× (2.77–2.84) | 2.31× (2.29–2.33) | 2.33× (2.31–2.35) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.37× (0.37–0.38) | 0.27× (0.27–0.27) | 1.11× (1.11–1.12) | 0.85× (0.84–0.86) | 11.41× (11.33–11.49) | 0.31× (0.31–0.31) | 0.33× (0.32–0.33) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.40× (0.39–0.41) | 0.49× (0.47–0.51) | 0.74× (0.73–0.75) | 0.63× (0.62–0.64) | 4.12× (4.07–4.18) | 0.11× (0.11–0.11) | 0.12× (0.11–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.30× (0.30–0.30) | 0.17× (0.16–0.18) | 0.80× (0.79–0.80) | 0.81× (0.80–0.82) | 4.26× (4.24–4.27) | 0.26× (0.24–0.27) | 0.26× (0.26–0.26) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 34 MB | 0.20× (0.20–0.20) | 0.15× (0.14–0.17) | 1.71× (1.69–1.74) | 0.81× (0.80–0.81) | 4.21× (4.20–4.23) | 0.25× (0.24–0.27) | 0.26× (0.26–0.26) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 18 MB | 0.57× (0.57–0.57) | 0.39× (0.37–0.40) | 1.53× (1.53–1.54) | 0.66× (0.65–0.66) | 7.40× (7.37–7.43) | 0.59× (0.59–0.59) | 0.61× (0.61–0.62) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 43 MB | 0.61× (0.60–0.62) | 0.57× (0.54–0.61) | 1.07× (1.05–1.08) | 0.86× (0.85–0.87) | 3.14× (3.11–3.18) | 0.25× (0.24–0.25) | 0.26× (0.25–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 25 MB | 0.45× (0.44–0.45) | 0.29× (0.29–0.29) | 1.09× (1.09–1.10) | 0.72× (0.72–0.72) | 4.88× (4.86–4.91) | 0.35× (0.35–0.35) | 0.36× (0.36–0.37) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 53 MB | 0.60× (0.58–0.62) | 0.42× (0.42–0.43) | 0.80× (0.79–0.82) | 0.76× (0.74–0.78) | 2.35× (2.30–2.40) | 0.16× (0.16–0.17) | 0.17× (0.16–0.17) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 25 MB | 0.45× (0.44–0.45) | 0.30× (0.29–0.30) | 1.10× (1.10–1.10) | 0.81× (0.81–0.81) | 4.93× (4.92–4.95) | 0.35× (0.34–0.35) | 0.36× (0.36–0.36) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 58 MB | 0.57× (0.56–0.58) | 0.41× (0.39–0.42) | 0.74× (0.73–0.74) | 0.76× (0.73–0.79) | 2.14× (2.12–2.15) | 0.15× (0.15–0.15) | 0.15× (0.15–0.16) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 24 MB | 0.43× (0.41–0.44) | 0.20× (0.19–0.20) | 1.07× (1.04–1.11) | 0.74× (0.72–0.76) | 5.17× (5.05–5.28) | 0.33× (0.32–0.34) | 0.35× (0.34–0.35) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 26 MB | 0.26× (0.25–0.27) | 0.18× (0.18–0.18) | 1.51× (1.47–1.55) | 0.72× (0.70–0.74) | 4.82× (4.73–4.92) | 0.31× (0.30–0.31) | 0.32× (0.32–0.33) |
| `text` (1.23 MB → 12.2 MB, 9.9×) | 1 | 20 MB | 0.53× (0.52–0.53) | 0.37× (0.37–0.37) | 1.38× (1.38–1.39) | 0.59× (0.57–0.60) | 6.33× (6.32–6.34) | 0.43× (0.43–0.44) | 0.45× (0.45–0.46) |
| `text` (1.23 MB → 12.2 MB, 9.9×) | 4 | 56 MB | 0.50× (0.49–0.51) | 0.42× (0.39–0.46) | 0.91× (0.88–0.95) | 0.67× (0.63–0.71) | 2.32× (2.26–2.39) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | wall |
| --- | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.99 | 0.02 | 0.388 |

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2915 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m08s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.14 | 10.2% |
| MTF + RLE2 | 2.06 | 18.4% |
| T-vector build | 2.22 | 19.8% |
| walk (the one each build uses) | 5.44 | 48.5% |
| of which: segment assembly | 0.26 | 2.3% |
| RLE1 | 0.24 | 2.2% |
| CRC | 0.12 | 1.1% |
| **total** | 11.23 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 2h00m ± 14m51s |
| 4 threads | 55m50s ± 7m55s |
| 4 threads, the sample's overlap gap as a fixed cost | 36m38s ± 5m14s |
| the scanner alone (its serial part) | 15m08s ± 32.2s |
| the workers alone, over 4 | 36m38s ± 5m14s |

At 4 threads the workers bound it. Per byte: the scanner 8.8 ns a byte of bz2, the workers 4.80 ns a byte of output at 4 threads (CPU, summed over them), 3.46 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,232,504 | 12,212,095 | \`eec9039e0c9d71dd…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: INTEL(R) XEON(R) PLATINUM 8573C, 4 cores, reported clock 2914.6 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 6.17.0-1022-azure; image ubuntu24 20261023.334.3 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source) - bench 2.1.1 (bench/report.py), 10 rounds, threads 1, 4
