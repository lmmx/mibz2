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

The latest record: [`6e3797a`](<https://github.com/lmmx/mibz2/commit/6e3797a0d4ccc68c647fdd1e91fc891b737d0ced>) (2026-10-27).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2 and libbz2-rs**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40595524863>)

Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 2.1.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.842 s | 249 | 11.2 | 1.80× (1.78–1.83) | 4.03× (3.99–4.07) | 4.31× (4.27–4.35) | 4.71× (4.67–4.75) | 4.15× (4.06–4.25) | 4.63× (4.55–4.71) | 4.19× (4.12–4.27) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.415 s | 506 | 18.9 | 1.91× (1.87–1.94) | 2.69× (2.63–2.74) | 3.51× (3.46–3.55) | 3.12× (3.04–3.20) | 2.98× (2.92–3.04) | 9.38× (9.10–9.67) | 8.49× (8.24–8.74) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.015 s | 1,375 | 2.3 | 0.82× (0.81–0.83) | 6.95× (6.87–7.02) | 9.67× (9.54–9.81) | 4.56× (4.49–4.64) | 5.40× (5.32–5.48) | 3.65× (3.60–3.71) | 3.60× (3.55–3.65) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.015 s | 1,303 | 2.4 | 0.84× (0.81–0.86) | 6.60× (6.44–6.77) | 9.20× (8.98–9.43) | 4.40× (4.29–4.51) | 5.24× (5.11–5.37) | 3.50× (3.42–3.60) | 3.46× (3.38–3.54) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.043 s | 461 | 6.3 | 2.49× (2.47–2.52) | 3.59× (3.56–3.62) | 4.51× (4.49–4.54) | 3.53× (3.51–3.55) | 3.09× (3.01–3.17) | 3.34× (3.33–3.36) | 2.51× (2.50–2.52) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.028 s | 690 | 14.8 | 2.02× (1.97–2.08) | 1.79× (1.75–1.83) | 3.24× (3.15–3.33) | 2.15× (2.10–2.20) | 2.28× (2.22–2.34) | 5.03× (4.91–5.15) | 3.78× (3.68–3.87) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.143 s | 56 | 50.6 | 1.33× (1.31–1.34) | 2.80× (2.77–2.83) | 2.38× (2.36–2.41) | 2.33× (2.31–2.36) | 2.42× (2.38–2.46) | 2.99× (2.95–3.02) | 3.05× (3.02–3.08) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.072 s | 111 | 74.3 | 1.45× (1.43–1.48) | 2.37× (2.11–2.65) | 2.59× (2.40–2.78) | 2.74× (2.67–2.81) | 2.29× (2.25–2.33) | 5.97× (5.89–6.06) | 6.10× (6.01–6.20) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.171 s | 56 | 50.2 | 1.29× (1.27–1.32) | 2.87× (2.80–2.95) | 2.26× (2.23–2.30) | 2.37× (2.33–2.42) | 2.48× (2.44–2.52) | 3.33× (3.28–3.38) | 3.59× (3.52–3.65) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.085 s | 114 | 76.9 | 1.57× (1.53–1.62) | 2.49× (2.29–2.69) | 2.26× (2.19–2.32) | 3.06× (2.96–3.17) | 2.23× (2.17–2.30) | 6.79× (6.61–6.98) | 7.32× (7.13–7.51) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.011 s | 905 | 3.6 | 0.79× (0.77–0.81) | 4.60× (4.53–4.67) | 8.30× (8.18–8.43) | 3.62× (3.57–3.68) | 4.73× (4.65–4.82) | 2.78× (2.73–2.83) | 2.77× (2.74–2.81) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.012 s | 810 | 4.0 | 0.76× (0.74–0.77) | 4.08× (4.01–4.15) | 7.31× (7.18–7.44) | 3.28× (3.21–3.35) | 4.31× (4.26–4.37) | 2.48× (2.43–2.53) | 2.48× (2.43–2.53) | yes |
| `text` (1.23 MB → 12.2 MB, 9.9×) | 1 | 0.077 s | 159 | 18.0 | 1.38× (1.37–1.39) | 3.27× (3.25–3.30) | 3.79× (3.76–3.82) | 3.53× (3.47–3.59) | 3.48× (3.44–3.51) | 3.71× (3.66–3.75) | 3.56× (3.54–3.59) | yes |
| `text` (1.23 MB → 12.2 MB, 9.9×) | 4 | 0.040 s | 307 | 30.2 | 1.42× (1.38–1.47) | 2.88× (2.48–3.34) | 3.57× (3.45–3.69) | 3.33× (3.22–3.44) | 2.71× (2.60–2.82) | 7.21× (6.98–7.44) | 6.92× (6.68–7.18) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 1.80× (1.78–1.83) | 3.99× (3.96–4.03) | 4.27× (4.24–4.31) | 4.70× (4.66–4.74) | 4.14× (4.05–4.24) | 4.62× (4.54–4.71) | 4.18× (4.11–4.26) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.11× (2.09–2.13) | 3.06× (3.02–3.11) | 3.66× (3.62–3.70) | 3.59× (3.55–3.64) | 3.23× (3.18–3.28) | 2.76× (2.70–2.82) | 2.49× (2.44–2.55) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.84× (0.82–0.85) | 3.66× (3.60–3.71) | 6.14× (6.05–6.23) | 4.12× (4.06–4.19) | 4.86× (4.80–4.92) | 3.33× (3.28–3.37) | 3.28× (3.24–3.33) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.86× (0.84–0.88) | 3.94× (3.86–4.02) | 7.03× (6.91–7.15) | 4.06× (3.97–4.15) | 4.74× (4.64–4.85) | 3.22× (3.15–3.29) | 3.18× (3.12–3.25) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.43× (2.41–2.45) | 2.71× (2.70–2.72) | 3.41× (3.39–3.43) | 3.42× (3.40–3.44) | 3.00× (2.92–3.09) | 3.25× (3.23–3.26) | 2.45× (2.44–2.46) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.11× (2.01–2.22) | 1.61× (1.53–1.69) | 2.24× (2.14–2.34) | 1.97× (1.89–2.06) | 1.64× (1.56–1.71) | 1.43× (1.37–1.50) | 1.08× (1.03–1.13) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.32× (1.31–1.34) | 2.47× (2.44–2.49) | 2.11× (2.08–2.13) | 2.32× (2.29–2.34) | 2.40× (2.36–2.44) | 2.96× (2.93–2.99) | 3.02× (3.00–3.05) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.63× (1.55–1.71) | 2.35× (2.30–2.40) | 1.98× (1.92–2.04) | 2.76× (2.70–2.82) | 2.33× (2.29–2.36) | 2.01× (1.97–2.05) | 2.06× (2.02–2.10) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.29× (1.27–1.31) | 2.62× (2.58–2.66) | 2.14× (2.10–2.18) | 2.36× (2.31–2.40) | 2.47× (2.43–2.50) | 3.30× (3.25–3.36) | 3.56× (3.50–3.62) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.73× (1.64–1.81) | 2.41× (2.35–2.48) | 1.91× (1.87–1.96) | 2.96× (2.85–3.07) | 2.30× (2.27–2.34) | 2.18× (2.14–2.21) | 2.35× (2.31–2.38) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.82× (0.80–0.85) | 2.82× (2.77–2.87) | 4.47× (4.39–4.54) | 3.24× (3.18–3.29) | 4.18× (4.11–4.26) | 2.52× (2.47–2.57) | 2.52× (2.48–2.56) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.78× (0.77–0.80) | 2.86× (2.80–2.92) | 4.76× (4.71–4.81) | 3.05× (2.98–3.11) | 3.90× (3.86–3.94) | 2.28× (2.24–2.32) | 2.28× (2.24–2.32) |
| `text` (1.23 MB → 12.2 MB, 9.9×) | 1 | 1.37× (1.36–1.38) | 3.14× (3.11–3.16) | 3.36× (3.32–3.40) | 3.47× (3.41–3.53) | 3.42× (3.38–3.45) | 3.64× (3.60–3.69) | 3.50× (3.48–3.53) |
| `text` (1.23 MB → 12.2 MB, 9.9×) | 4 | 1.61× (1.53–1.69) | 2.45× (2.39–2.50) | 2.85× (2.79–2.91) | 2.86× (2.80–2.91) | 2.60× (2.54–2.67) | 2.19× (2.13–2.24) | 2.10× (2.06–2.15) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.37× (0.36–0.37) | 0.27× (0.26–0.27) | 1.11× (1.11–1.11) | 0.85× (0.85–0.85) | 11.35× (11.33–11.37) | 0.31× (0.31–0.31) | 0.33× (0.32–0.33) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.41× (0.40–0.41) | 0.49× (0.47–0.50) | 0.74× (0.72–0.76) | 0.63× (0.62–0.64) | 4.11× (4.06–4.16) | 0.11× (0.11–0.11) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.30× (0.29–0.31) | 0.17× (0.16–0.18) | 0.80× (0.79–0.80) | 0.81× (0.81–0.82) | 4.26× (4.25–4.27) | 0.25× (0.25–0.25) | 0.26× (0.26–0.26) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 35 MB | 0.20× (0.20–0.20) | 0.15× (0.15–0.15) | 1.72× (1.70–1.75) | 0.81× (0.80–0.81) | 4.19× (4.18–4.20) | 0.25× (0.25–0.25) | 0.26× (0.26–0.26) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 18 MB | 0.57× (0.56–0.58) | 0.39× (0.38–0.41) | 1.53× (1.53–1.54) | 0.66× (0.65–0.66) | 7.40× (7.37–7.43) | 0.59× (0.59–0.59) | 0.61× (0.61–0.62) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 43 MB | 0.61× (0.60–0.63) | 0.59× (0.54–0.64) | 1.07× (1.06–1.08) | 0.85× (0.84–0.87) | 3.14× (3.10–3.19) | 0.25× (0.24–0.25) | 0.25× (0.25–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 24 MB | 0.45× (0.44–0.45) | 0.29× (0.29–0.29) | 1.09× (1.09–1.10) | 0.72× (0.72–0.72) | 4.87× (4.85–4.89) | 0.35× (0.35–0.35) | 0.36× (0.36–0.36) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 54 MB | 0.60× (0.59–0.60) | 0.42× (0.42–0.43) | 0.79× (0.79–0.80) | 0.76× (0.75–0.78) | 2.32× (2.30–2.34) | 0.16× (0.16–0.16) | 0.17× (0.16–0.17) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 25 MB | 0.44× (0.44–0.45) | 0.30× (0.29–0.30) | 1.10× (1.10–1.10) | 0.81× (0.81–0.81) | 4.95× (4.92–4.97) | 0.35× (0.34–0.35) | 0.36× (0.36–0.36) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 58 MB | 0.58× (0.56–0.59) | 0.41× (0.40–0.42) | 0.75× (0.74–0.76) | 0.75× (0.72–0.79) | 2.15× (2.12–2.19) | 0.15× (0.15–0.15) | 0.15× (0.15–0.16) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 24 MB | 0.43× (0.42–0.44) | 0.19× (0.18–0.20) | 1.08× (1.05–1.11) | 0.76× (0.72–0.79) | 5.22× (5.11–5.33) | 0.33× (0.32–0.34) | 0.35× (0.34–0.36) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 26 MB | 0.26× (0.25–0.28) | 0.18× (0.18–0.18) | 1.50× (1.45–1.56) | 0.74× (0.71–0.77) | 4.86× (4.74–4.97) | 0.31× (0.30–0.32) | 0.32× (0.32–0.33) |
| `text` (1.23 MB → 12.2 MB, 9.9×) | 1 | 20 MB | 0.52× (0.52–0.53) | 0.37× (0.37–0.37) | 1.38× (1.38–1.39) | 0.60× (0.58–0.62) | 6.32× (6.29–6.35) | 0.43× (0.43–0.44) | 0.45× (0.45–0.46) |
| `text` (1.23 MB → 12.2 MB, 9.9×) | 4 | 55 MB | 0.51× (0.50–0.53) | 0.47× (0.43–0.52) | 0.94× (0.91–0.96) | 0.64× (0.61–0.67) | 2.37× (2.32–2.43) | 0.16× (0.15–0.16) | 0.16× (0.16–0.17) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | wall |
| --- | ---: | ---: | ---: | ---: |
| mibz2 | 0.12 | 1.15 | 0.03 | 0.415 |

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2790 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m09s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.13 | 10.3% |
| MTF + RLE2 | 1.75 | 16.0% |
| T-vector build | 2.16 | 19.8% |
| walk (the one each build uses) | 5.56 | 50.8% |
| of which: segment assembly | 0.23 | 2.1% |
| RLE1 | 0.21 | 1.9% |
| CRC | 0.12 | 1.1% |
| **total** | 10.94 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 2h03m ± 15m02s |
| 4 threads | 1h00m ± 8m41s |
| 4 threads, the sample's overlap gap as a fixed cost | 41m19s ± 5m58s |
| the scanner alone (its serial part) | 16m26s ± 51.4s |
| the workers alone, over 4 | 41m19s ± 5m58s |

At 4 threads the workers bound it. Per byte: the scanner 9.5 ns a byte of bz2, the workers 5.42 ns a byte of output at 4 threads (CPU, summed over them), 3.50 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 10 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,232,504 | 12,212,095 | \`eec9039e0c9d71dd…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 4 cores, reported clock 2789.5 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 6.17.0-1022-azure; image ubuntu24 20261023.334.3 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source) - bench 2.1.1 (bench/report.py), 10 rounds, threads 1, 4
