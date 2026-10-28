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

The latest record: [`cc92820`](<https://github.com/lmmx/mibz2/commit/cc928203e9afe3d094b4ad0326105c503861feb7>) (2026-10-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2 and libbz2-rs**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40626500960>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 2.1.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.415 s | 506 | 8.5 | 2.01× (1.91–2.11) | 5.17× (4.97–5.38) | 5.36× (5.09–5.64) | 6.11× (5.83–6.41) | 5.40× (5.11–5.70) | 5.66× (5.40–5.94) | 5.26× (5.00–5.54) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.272 s | 770 | 18.1 | 1.56× (1.53–1.60) | 2.63× (2.56–2.71) | 3.32× (3.15–3.49) | 3.06× (2.94–3.18) | 3.22× (3.11–3.34) | 8.73× (8.39–9.08) | 8.12× (7.79–8.46) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.016 s | 1,270 | 3.8 | 0.54× (0.52–0.55) | 3.67× (2.98–4.51) | 5.89× (5.72–6.06) | 3.50× (3.38–3.63) | 6.69× (6.49–6.90) | 2.60× (2.54–2.66) | 2.57× (2.52–2.63) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.016 s | 1,254 | 3.8 | 0.49× (0.48–0.50) | 3.18× (3.09–3.26) | 5.85× (5.69–6.01) | 3.44× (3.37–3.51) | 6.47× (6.31–6.63) | 2.58× (2.48–2.69) | 2.56× (2.46–2.65) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.024 s | 802 | 5.7 | 2.27× (2.06–2.50) | 4.11× (4.01–4.22) | 3.76× (3.65–3.87) | 3.56× (3.49–3.63) | 4.55× (4.44–4.66) | 2.18× (2.09–2.27) | 1.98× (1.93–2.03) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.020 s | 968 | 15.1 | 1.35× (1.27–1.44) | 2.43× (2.27–2.61) | 4.13× (3.51–4.85) | 1.81× (1.68–1.95) | 3.88× (3.62–4.15) | 2.60× (2.44–2.78) | 2.36× (2.20–2.54) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.094 s | 85 | 51.4 | 1.18× (1.07–1.30) | 2.84× (2.51–3.23) | 2.56× (2.29–2.85) | 2.33× (2.10–2.58) | 2.75× (2.48–3.04) | 3.10× (2.81–3.43) | 2.98× (2.68–3.30) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.047 s | 172 | 76.8 | 1.16× (1.12–1.20) | 2.65× (2.31–3.05) | 3.07× (2.99–3.15) | 2.68× (2.59–2.78) | 3.08× (2.94–3.24) | 6.45× (6.29–6.62) | 6.19× (6.02–6.36) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.118 s | 81 | 53.6 | 1.08× (1.06–1.10) | 2.97× (2.89–3.06) | 2.43× (2.34–2.52) | 2.24× (2.20–2.28) | 2.67× (2.57–2.77) | 3.45× (3.35–3.55) | 3.44× (3.38–3.51) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.055 s | 174 | 79.5 | 1.12× (1.09–1.16) | 2.71× (2.58–2.84) | 2.56× (2.44–2.69) | 2.87× (2.73–3.02) | 2.79× (2.67–2.93) | 7.29× (6.95–7.64) | 7.28× (6.96–7.60) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.011 s | 952 | 5.2 | 0.59× (0.57–0.60) | 4.78× (4.67–4.89) | 4.04× (3.94–4.14) | 3.00× (2.85–3.16) | 7.49× (6.76–8.30) | 2.18× (2.14–2.21) | 2.18× (2.12–2.23) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.011 s | 909 | 5.5 | 0.48× (0.47–0.50) | 4.61× (4.50–4.73) | 8.48× (8.28–8.67) | 2.86× (2.79–2.92) | 6.99× (6.87–7.12) | 2.10× (2.02–2.18) | 2.10× (2.03–2.17) | yes |
| `text` (1.36 MB → 12 MB, 8.8×) | 1 | 0.048 s | 250 | 17.8 | 1.31× (1.28–1.33) | 3.35× (3.06–3.66) | 4.01× (3.89–4.14) | 3.53× (3.47–3.60) | 4.19× (3.93–4.47) | 3.70× (3.63–3.76) | 3.43× (3.26–3.60) | yes |
| `text` (1.36 MB → 12 MB, 8.8×) | 4 | 0.030 s | 394 | 33.4 | 0.92× (0.78–1.09) | 2.85× (2.33–3.49) | 2.84× (2.40–3.35) | 2.55× (2.15–3.02) | 3.15× (2.66–3.72) | 5.38× (4.57–6.35) | 4.99× (4.22–5.91) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.00× (1.91–2.10) | 5.10× (4.91–5.31) | 5.30× (5.05–5.57) | 6.09× (5.81–6.39) | 5.38× (5.09–5.68) | 5.64× (5.38–5.92) | 5.25× (4.99–5.52) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 1.66× (1.61–1.71) | 3.11× (2.99–3.23) | 3.38× (3.18–3.59) | 3.72× (3.57–3.87) | 3.35× (3.20–3.51) | 2.69× (2.56–2.83) | 2.51× (2.38–2.63) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.58× (0.56–0.60) | 2.84× (2.75–2.93) | 3.75× (3.63–3.87) | 3.27× (3.15–3.39) | 6.15× (5.96–6.34) | 2.46× (2.39–2.54) | 2.42× (2.37–2.48) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.54× (0.53–0.55) | 2.89× (2.82–2.95) | 4.90× (4.74–5.07) | 3.23× (3.15–3.30) | 5.92× (5.76–6.09) | 2.44× (2.31–2.57) | 2.40× (2.31–2.49) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.20× (1.98–2.45) | 2.52× (2.47–2.57) | 3.06× (2.98–3.14) | 3.40× (3.31–3.48) | 4.33× (4.22–4.45) | 2.11× (2.02–2.20) | 1.93× (1.87–1.98) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.56× (1.49–1.63) | 1.47× (1.41–1.54) | 1.76× (1.68–1.84) | 1.72× (1.63–1.81) | 2.02× (1.93–2.12) | 0.83× (0.79–0.88) | 0.76× (0.72–0.79) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.17× (1.07–1.29) | 2.57× (2.34–2.83) | 2.29× (2.06–2.55) | 2.31× (2.08–2.55) | 2.71× (2.45–3.01) | 3.07× (2.78–3.38) | 2.94× (2.65–3.25) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.26× (1.22–1.30) | 2.33× (2.27–2.39) | 1.95× (1.91–1.99) | 2.48× (2.39–2.59) | 2.42× (2.35–2.50) | 2.10× (2.05–2.15) | 2.01× (1.97–2.05) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.08× (1.05–1.10) | 2.67× (2.60–2.75) | 2.13× (2.09–2.17) | 2.22× (2.18–2.26) | 2.65× (2.55–2.75) | 3.41× (3.32–3.51) | 3.41× (3.34–3.47) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.21× (1.18–1.24) | 2.37× (2.30–2.43) | 1.80× (1.74–1.87) | 2.52× (2.43–2.62) | 2.33× (2.23–2.42) | 2.28× (2.20–2.36) | 2.27× (2.20–2.35) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.64× (0.63–0.66) | 2.41× (2.35–2.46) | 3.21× (3.12–3.30) | 2.74× (2.59–2.89) | 6.64× (5.99–7.37) | 2.02× (1.98–2.06) | 2.02× (1.97–2.08) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.55× (0.54–0.57) | 2.36× (2.28–2.44) | 3.92× (3.78–4.06) | 2.67× (2.61–2.73) | 6.22× (6.11–6.32) | 1.95× (1.88–2.02) | 1.95× (1.88–2.01) |
| `text` (1.36 MB → 12 MB, 8.8×) | 1 | 1.30× (1.27–1.32) | 3.03× (2.95–3.12) | 3.29× (3.21–3.37) | 3.44× (3.38–3.52) | 4.08× (3.83–4.35) | 3.60× (3.53–3.67) | 3.34× (3.18–3.51) |
| `text` (1.36 MB → 12 MB, 8.8×) | 4 | 1.13× (0.93–1.36) | 1.92× (1.58–2.32) | 2.13× (1.77–2.58) | 2.28× (1.89–2.76) | 2.39× (1.97–2.90) | 1.75× (1.44–2.12) | 1.63× (1.33–1.99) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 27 MB | 0.32× (0.32–0.32) | 0.35× (0.34–0.36) | 1.15× (1.15–1.16) | 0.83× (0.81–0.85) | 11.72× (11.71–11.73) | 0.28× (0.28–0.29) | 0.30× (0.30–0.30) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 74 MB | 0.41× (0.40–0.41) | 0.83× (0.69–1.01) | 0.78× (0.76–0.79) | 0.65× (0.64–0.65) | 4.28× (4.26–4.30) | 0.10× (0.10–0.10) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.80) | 4.28× (4.27–4.28) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.78× (1.77–1.78) | 0.83× (0.83–0.83) | 4.39× (4.38–4.40) | 0.27× (0.26–0.27) | 0.28× (0.28–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 17 MB | 0.50× (0.50–0.50) | 0.43× (0.43–0.43) | 1.62× (1.62–1.63) | 0.64× (0.64–0.64) | 7.74× (7.73–7.76) | 0.64× (0.64–0.64) | 0.66× (0.66–0.67) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 43 MB | 0.58× (0.57–0.60) | 0.61× (0.58–0.65) | 1.10× (1.08–1.13) | 0.83× (0.81–0.84) | 3.16× (3.13–3.19) | 0.25× (0.25–0.25) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 24 MB | 0.39× (0.39–0.40) | 0.31× (0.31–0.31) | 1.14× (1.14–1.14) | 0.71× (0.71–0.71) | 5.04× (5.04–5.05) | 0.32× (0.32–0.32) | 0.34× (0.33–0.34) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 51 MB | 0.61× (0.59–0.62) | 0.44× (0.43–0.45) | 0.83× (0.82–0.84) | 0.77× (0.73–0.81) | 2.43× (2.39–2.47) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 24 MB | 0.39× (0.39–0.39) | 0.31× (0.31–0.31) | 1.15× (1.14–1.15) | 0.81× (0.80–0.81) | 5.08× (5.07–5.09) | 0.32× (0.32–0.32) | 0.34× (0.34–0.34) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 60 MB | 0.54× (0.53–0.55) | 0.45× (0.42–0.48) | 0.72× (0.70–0.73) | 0.71× (0.69–0.74) | 2.09× (2.05–2.13) | 0.13× (0.12–0.13) | 0.13× (0.13–0.14) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.24× (0.22–0.25) | 1.08× (1.07–1.08) | 0.77× (0.77–0.77) | 5.41× (5.40–5.42) | 0.36× (0.36–0.36) | 0.38× (0.37–0.38) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.22× (0.22–0.23) | 0.20× (0.20–0.20) | 1.62× (1.62–1.63) | 0.76× (0.76–0.77) | 5.38× (5.37–5.40) | 0.36× (0.35–0.36) | 0.37× (0.37–0.38) |
| `text` (1.36 MB → 12 MB, 8.8×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.44× (1.44–1.45) | 0.61× (0.61–0.61) | 6.53× (6.52–6.54) | 0.41× (0.40–0.41) | 0.43× (0.43–0.43) |
| `text` (1.36 MB → 12 MB, 8.8×) | 4 | 49 MB | 0.55× (0.55–0.56) | 0.50× (0.46–0.55) | 1.03× (1.02–1.04) | 0.80× (0.78–0.82) | 2.65× (2.62–2.68) | 0.16× (0.16–0.16) | 0.17× (0.16–0.17) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.68 | 0.01 | – | 0.272 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4296 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 41.2s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.04 | 12.3% |
| MTF + RLE2 | 1.88 | 22.3% |
| T-vector build | 1.44 | 17.0% |
| walk (the one each build uses) | 3.67 | 43.5% |
| of which: segment assembly | 0.12 | 1.5% |
| RLE1 | 0.15 | 1.7% |
| CRC | 0.25 | 3.0% |
| **total** | 8.44 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h02m ± 8m14s |
| 4 threads | 40m29s ± 5m52s |
| 4 threads, the sample's overlap gap as a fixed cost | 25m06s ± 3m33s |
| the scanner alone (its serial part) | 8m35s ± 35.4s |
| the workers alone, over 4 | 25m06s ± 3m33s |

At 4 threads the workers bound it. Per byte: the scanner 5.0 ns a byte of bz2, the workers 3.29 ns a byte of output at 4 threads (CPU, summed over them), 1.77 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 12 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,360,807 | 12,001,447 | \`3a004df8c0dfe259…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4296.5 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1017-azure; image ubuntu24 20261026.354.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source) - bench 2.1.1 (bench/report.py), 10 rounds, threads 1, 4
