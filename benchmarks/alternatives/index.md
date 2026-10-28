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

The latest record: [`40ba07a`](<https://github.com/lmmx/mibz2/commit/40ba07a18629e30693c7a421bc2ae73440671d60>) (2026-10-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2 and libbz2-rs**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40611620287>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 2.1.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.779 s | 269 | 12.1 | 1.89× (1.87–1.90) | 4.42× (4.38–4.47) | 4.31× (4.25–4.37) | 4.89× (4.79–4.98) | 4.07× (3.96–4.19) | 4.53× (4.44–4.62) | 4.28× (4.17–4.39) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.383 s | 548 | 20.2 | 1.75× (1.72–1.79) | 2.65× (2.63–2.67) | 3.41× (3.34–3.49) | 2.95× (2.90–3.00) | 2.72× (2.65–2.78) | 9.19× (8.91–9.49) | 8.69× (8.44–8.96) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.014 s | 1,413 | 2.6 | 0.78× (0.71–0.85) | 6.94× (6.39–7.53) | 6.20× (5.72–6.72) | 4.63× (4.25–5.04) | 4.97× (4.56–5.42) | 3.93× (3.62–4.26) | 3.70× (3.40–4.01) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.014 s | 1,413 | 2.7 | 0.84× (0.83–0.86) | 7.02× (6.83–7.22) | 9.79× (9.50–10.08) | 4.71× (4.58–4.83) | 5.07× (4.95–5.20) | 4.00× (3.89–4.10) | 3.76× (3.67–3.86) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.038 s | 512 | 6.7 | 2.18× (2.11–2.24) | 2.73× (2.48–3.02) | 3.65× (3.53–3.77) | 3.31× (3.21–3.42) | 2.44× (2.35–2.54) | 2.16× (2.10–2.23) | 1.83× (1.78–1.88) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.028 s | 694 | 15.4 | 1.60× (1.56–1.63) | 1.77× (1.73–1.82) | 3.19× (3.11–3.27) | 2.03× (1.96–2.10) | 1.78× (1.72–1.85) | 2.94× (2.87–3.01) | 2.49× (2.43–2.56) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.151 s | 53 | 61.8 | 1.18× (1.17–1.20) | 2.57× (2.46–2.68) | 2.27× (2.25–2.28) | 2.15× (2.12–2.18) | 2.16× (2.15–2.18) | 2.70× (2.67–2.72) | 2.76× (2.74–2.79) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.072 s | 112 | 87.1 | 1.22× (1.16–1.28) | 2.17× (2.01–2.33) | 2.15× (1.93–2.39) | 2.41× (2.33–2.50) | 2.02× (1.96–2.09) | 5.62× (5.44–5.80) | 5.76× (5.58–5.94) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.199 s | 48 | 67.8 | 1.08× (1.07–1.08) | 2.40× (2.31–2.50) | 1.97× (1.96–1.98) | 1.89× (1.87–1.90) | 1.98× (1.96–2.00) | 2.74× (2.72–2.75) | 2.87× (2.86–2.89) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.088 s | 109 | 97.3 | 1.21× (1.18–1.25) | 2.29× (2.26–2.33) | 2.17× (2.14–2.20) | 2.60× (2.54–2.66) | 1.86× (1.82–1.90) | 6.16× (6.07–6.26) | 6.47× (6.37–6.58) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.011 s | 870 | 4.4 | 0.72× (0.72–0.73) | 4.43× (4.34–4.52) | 7.94× (7.79–8.10) | 3.50× (3.45–3.56) | 4.01× (3.93–4.10) | 2.86× (2.81–2.91) | 2.70× (2.66–2.74) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.012 s | 823 | 4.7 | 0.67× (0.65–0.69) | 4.16× (4.03–4.28) | 7.42× (7.20–7.65) | 3.35× (3.27–3.44) | 3.83× (3.69–3.98) | 2.69× (2.61–2.77) | 2.55× (2.47–2.63) | yes |
| `text` (1.24 MB → 12.2 MB, 9.9×) | 1 | 0.076 s | 161 | 20.7 | 1.45× (1.43–1.48) | 3.30× (3.25–3.35) | 3.67× (3.44–3.92) | 3.23× (3.16–3.30) | 2.98× (2.89–3.07) | 3.61× (3.55–3.67) | 3.55× (3.49–3.62) | yes |
| `text` (1.24 MB → 12.2 MB, 9.9×) | 4 | 0.042 s | 289 | 36.2 | 1.27× (1.21–1.32) | 2.42× (2.33–2.52) | 3.36× (3.24–3.48) | 2.91× (2.76–3.07) | 2.11× (2.02–2.20) | 6.56× (6.34–6.79) | 6.46× (6.23–6.69) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 1.89× (1.87–1.90) | 4.38× (4.34–4.42) | 4.27× (4.22–4.33) | 4.87× (4.78–4.97) | 4.07× (3.95–4.18) | 4.52× (4.43–4.61) | 4.27× (4.17–4.38) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 1.93× (1.91–1.96) | 2.99× (2.95–3.03) | 3.41× (3.35–3.47) | 3.42× (3.38–3.47) | 2.98× (2.93–3.02) | 2.71× (2.65–2.78) | 2.57× (2.49–2.64) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.80× (0.74–0.87) | 4.37× (4.06–4.70) | 5.24× (4.86–5.64) | 4.15× (3.85–4.47) | 4.44× (4.11–4.80) | 3.53× (3.29–3.79) | 3.33× (3.10–3.59) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.86× (0.85–0.87) | 4.83× (4.72–4.94) | 6.14× (6.00–6.29) | 4.23× (4.13–4.34) | 4.47× (4.36–4.59) | 3.55× (3.46–3.65) | 3.35× (3.27–3.44) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.12× (2.06–2.18) | 2.51× (2.43–2.58) | 2.89× (2.73–3.05) | 3.19× (3.09–3.29) | 2.37× (2.28–2.46) | 2.10× (2.04–2.17) | 1.79× (1.75–1.84) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.81× (1.73–1.90) | 1.67× (1.59–1.75) | 1.98× (1.88–2.07) | 1.99× (1.89–2.08) | 1.40× (1.34–1.46) | 0.91× (0.86–0.96) | 0.77× (0.74–0.81) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.18× (1.17–1.20) | 2.34× (2.31–2.37) | 1.99× (1.98–2.01) | 2.13× (2.11–2.16) | 2.14× (2.13–2.16) | 2.67× (2.65–2.70) | 2.74× (2.71–2.77) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.36× (1.31–1.40) | 2.13× (2.09–2.16) | 1.72× (1.68–1.76) | 2.45× (2.42–2.47) | 2.03× (1.99–2.07) | 1.89× (1.87–1.92) | 1.94× (1.91–1.97) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.08× (1.07–1.08) | 2.27× (2.26–2.29) | 1.77× (1.76–1.78) | 1.88× (1.86–1.89) | 1.97× (1.95–1.99) | 2.72× (2.70–2.73) | 2.85× (2.84–2.87) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.30× (1.28–1.32) | 2.11× (2.08–2.13) | 1.56× (1.53–1.59) | 2.34× (2.31–2.38) | 1.87× (1.85–1.89) | 1.89× (1.87–1.91) | 1.99× (1.97–2.01) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.77× (0.76–0.78) | 3.17× (3.11–3.24) | 3.86× (3.81–3.91) | 3.10× (3.06–3.14) | 3.54× (3.48–3.60) | 2.56× (2.52–2.60) | 2.43× (2.40–2.46) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.72× (0.71–0.74) | 3.23× (3.13–3.33) | 4.22× (4.10–4.34) | 3.05× (2.99–3.12) | 3.38× (3.27–3.49) | 2.39× (2.33–2.45) | 2.27× (2.21–2.33) |
| `text` (1.24 MB → 12.2 MB, 9.9×) | 1 | 1.44× (1.41–1.46) | 3.07× (3.02–3.12) | 3.20× (3.11–3.29) | 3.16× (3.10–3.23) | 2.92× (2.84–3.01) | 3.54× (3.48–3.60) | 3.48× (3.42–3.54) |
| `text` (1.24 MB → 12.2 MB, 9.9×) | 4 | 1.38× (1.31–1.46) | 2.10× (2.05–2.15) | 2.47× (2.39–2.54) | 2.47× (2.42–2.52) | 2.09× (2.03–2.16) | 2.03× (1.98–2.09) | 2.00× (1.94–2.06) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 27 MB | 0.37× (0.36–0.37) | 0.28× (0.27–0.30) | 1.12× (1.12–1.12) | 0.85× (0.84–0.86) | 11.40× (11.38–11.43) | 0.31× (0.31–0.31) | 0.32× (0.32–0.33) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.40× (0.40–0.40) | 0.50× (0.49–0.51) | 0.75× (0.75–0.76) | 0.62× (0.61–0.63) | 4.09× (4.07–4.12) | 0.11× (0.11–0.11) | 0.11× (0.11–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.30× (0.29–0.30) | 0.17× (0.15–0.18) | 0.80× (0.79–0.80) | 0.81× (0.80–0.82) | 4.25× (4.23–4.27) | 0.25× (0.25–0.25) | 0.26× (0.26–0.26) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 35 MB | 0.20× (0.19–0.21) | 0.15× (0.14–0.15) | 1.72× (1.70–1.74) | 0.80× (0.79–0.81) | 4.21× (4.19–4.22) | 0.24× (0.24–0.25) | 0.26× (0.25–0.26) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 18 MB | 0.57× (0.56–0.57) | 0.40× (0.40–0.40) | 1.54× (1.54–1.55) | 0.64× (0.62–0.66) | 7.37× (7.34–7.40) | 0.59× (0.59–0.59) | 0.61× (0.60–0.61) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 43 MB | 0.59× (0.58–0.60) | 0.65× (0.63–0.67) | 1.06× (1.04–1.08) | 0.84× (0.83–0.85) | 3.10× (3.05–3.14) | 0.24× (0.24–0.24) | 0.25× (0.24–0.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 24 MB | 0.44× (0.44–0.45) | 0.29× (0.29–0.29) | 1.10× (1.10–1.10) | 0.72× (0.72–0.72) | 4.89× (4.85–4.92) | 0.34× (0.34–0.35) | 0.36× (0.36–0.36) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 54 MB | 0.59× (0.57–0.60) | 0.42× (0.40–0.43) | 0.78× (0.77–0.80) | 0.74× (0.71–0.76) | 2.28× (2.25–2.31) | 0.15× (0.15–0.16) | 0.16× (0.16–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 25 MB | 0.44× (0.44–0.45) | 0.30× (0.30–0.30) | 1.11× (1.10–1.11) | 0.81× (0.81–0.81) | 4.93× (4.90–4.95) | 0.34× (0.34–0.34) | 0.36× (0.35–0.36) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 59 MB | 0.56× (0.55–0.57) | 0.45× (0.43–0.47) | 0.73× (0.72–0.74) | 0.75× (0.73–0.78) | 2.11× (2.09–2.14) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 24 MB | 0.41× (0.41–0.42) | 0.19× (0.19–0.19) | 1.06× (1.04–1.07) | 0.76× (0.74–0.79) | 5.14× (5.06–5.21) | 0.32× (0.32–0.33) | 0.34× (0.33–0.34) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 26 MB | 0.26× (0.24–0.27) | 0.17× (0.17–0.18) | 1.50× (1.47–1.53) | 0.72× (0.69–0.75) | 4.85× (4.77–4.93) | 0.30× (0.30–0.31) | 0.32× (0.31–0.32) |
| `text` (1.24 MB → 12.2 MB, 9.9×) | 1 | 20 MB | 0.52× (0.52–0.53) | 0.37× (0.37–0.37) | 1.39× (1.38–1.39) | 0.58× (0.57–0.60) | 6.32× (6.29–6.35) | 0.43× (0.43–0.43) | 0.45× (0.44–0.45) |
| `text` (1.24 MB → 12.2 MB, 9.9×) | 4 | 48 MB | 0.57× (0.57–0.58) | 0.53× (0.47–0.59) | 1.06× (1.04–1.08) | 0.76× (0.73–0.79) | 2.68× (2.65–2.71) | 0.17× (0.17–0.18) | 0.18× (0.18–0.18) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | wall |
| --- | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 1.02 | 0.02 | 0.383 |

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3235 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m06s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.21 | 10.8% |
| MTF + RLE2 | 2.23 | 20.0% |
| T-vector build | 2.24 | 20.0% |
| walk (the one each build uses) | 4.97 | 44.4% |
| of which: segment assembly | 0.17 | 1.5% |
| RLE1 | 0.28 | 2.5% |
| CRC | 0.26 | 2.3% |
| **total** | 11.20 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h53m ± 14m01s |
| 4 threads | 56m08s ± 7m55s |
| 4 threads, the sample's overlap gap as a fixed cost | 37m21s ± 5m18s |
| the scanner alone (its serial part) | 14m07s ± 30.2s |
| the workers alone, over 4 | 37m21s ± 5m18s |

At 4 threads the workers bound it. Per byte: the scanner 8.2 ns a byte of bz2, the workers 4.90 ns a byte of output at 4 threads (CPU, summed over them), 3.27 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 11 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,236,217 | 12,227,335 | \`294c8c24e2835ff3…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3234.8 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 6.17.0-1022-azure; image ubuntu24 20261023.334.3 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source) - bench 2.1.1 (bench/report.py), 10 rounds, threads 1, 4
