# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-69/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`a0f9b19`](<https://github.com/lmmx/mibz2/commit/a0f9b192e6188f3fdcaaaf888707ff519ae3d861>) (2026-10-28).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2 and libbz2-rs**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/40617823177>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 2.1.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.606 s | 346 | 10.7 | 2.04× (2.02–2.05) | 4.65× (4.64–4.66) | 4.87× (4.85–4.89) | 5.45× (5.44–5.46) | 4.87× (4.86–4.88) | 5.39× (5.38–5.41) | 4.85× (4.83–4.86) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.333 s | 629 | 19.6 | 1.69× (1.67–1.72) | 2.72× (2.70–2.75) | 3.23× (3.18–3.29) | 3.10× (3.08–3.13) | 3.18× (3.15–3.20) | 9.81× (9.73–9.89) | 8.81× (8.73–8.89) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.019 s | 1,055 | 3.8 | 0.55× (0.54–0.56) | 5.31× (5.21–5.42) | 4.79× (4.71–4.88) | 3.36× (3.30–3.42) | 6.28× (6.16–6.41) | 2.52× (2.48–2.56) | 2.50× (2.45–2.54) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,064 | 3.8 | 0.52× (0.52–0.53) | 2.68× (2.65–2.70) | 7.53× (7.46–7.59) | 3.42× (3.39–3.45) | 6.36× (6.26–6.47) | 2.55× (2.53–2.57) | 2.53× (2.51–2.55) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.033 s | 588 | 6.6 | 2.17× (2.16–2.18) | 3.06× (3.04–3.08) | 4.28× (4.25–4.30) | 3.29× (3.27–3.30) | 3.96× (3.94–3.97) | 2.74× (2.72–2.76) | 1.82× (1.81–1.83) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 s | 776 | 16.5 | 1.49× (1.46–1.52) | 1.99× (1.94–2.04) | 3.64× (3.54–3.73) | 1.80× (1.75–1.85) | 3.74× (3.64–3.85) | 3.60× (3.51–3.69) | 2.39× (2.33–2.45) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.126 s | 64 | 58.8 | 1.21× (1.21–1.22) | 2.40× (2.39–2.41) | 2.33× (2.31–2.34) | 2.22× (2.21–2.23) | 2.58× (2.56–2.60) | 2.79× (2.77–2.80) | 2.75× (2.73–2.77) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.059 s | 135 | 80.4 | 1.24× (1.19–1.28) | 2.53× (2.44–2.64) | 2.38× (2.29–2.47) | 2.45× (2.36–2.55) | 2.89× (2.78–3.00) | 5.86× (5.65–6.09) | 5.78× (5.57–6.00) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.160 s | 60 | 62.2 | 1.13× (1.13–1.13) | 2.52× (2.51–2.52) | 2.14× (2.13–2.15) | 2.08× (2.06–2.11) | 2.46× (2.37–2.55) | 2.98× (2.96–2.99) | 3.02× (3.01–3.03) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.071 s | 135 | 87.3 | 1.23× (1.21–1.26) | 2.13× (2.08–2.17) | 2.06× (1.92–2.21) | 2.74× (2.67–2.80) | 2.71× (2.65–2.77) | 6.66× (6.53–6.80) | 6.75× (6.63–6.88) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 790 | 5.3 | 0.60× (0.59–0.61) | 3.94× (3.88–4.01) | 7.15× (7.04–7.27) | 2.88× (2.83–2.93) | 7.04× (6.93–7.15) | 2.18× (2.15–2.22) | 2.17× (2.13–2.20) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 775 | 5.5 | 0.51× (0.50–0.52) | 3.90× (3.84–3.96) | 7.08× (6.95–7.21) | 2.87× (2.82–2.93) | 6.97× (6.82–7.12) | 2.16× (2.13–2.20) | 2.14× (2.11–2.18) | yes |
| `text` (1.24 MB → 12.2 MB, 9.9×) | 1 | 0.065 s | 188 | 20.1 | 1.31× (1.30–1.33) | 3.12× (3.10–3.15) | 3.76× (3.73–3.79) | 3.37× (3.34–3.40) | 3.87× (3.80–3.95) | 3.39× (3.36–3.42) | 3.33× (3.30–3.36) | yes |
| `text` (1.24 MB → 12.2 MB, 9.9×) | 4 | 0.034 s | 355 | 33.6 | 1.17× (1.15–1.19) | 2.93× (2.88–2.98) | 2.67× (2.62–2.72) | 3.01× (2.93–3.08) | 3.66× (3.59–3.74) | 6.32× (6.22–6.44) | 6.22× (6.11–6.33) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.03× (2.02–2.04) | 4.63× (4.62–4.64) | 4.82× (4.80–4.84) | 5.44× (5.43–5.45) | 4.86× (4.85–4.87) | 5.38× (5.37–5.39) | 4.83× (4.82–4.85) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 1.81× (1.79–1.83) | 3.08× (3.05–3.12) | 3.26× (3.22–3.31) | 3.67× (3.65–3.70) | 3.23× (3.20–3.25) | 2.94× (2.93–2.96) | 2.64× (2.63–2.66) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.58× (0.58–0.59) | 2.74× (2.69–2.79) | 3.65× (3.56–3.74) | 3.15× (3.10–3.20) | 5.82× (5.71–5.93) | 2.38× (2.34–2.42) | 2.36× (2.32–2.40) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.56–0.57) | 2.91× (2.87–2.95) | 5.37× (5.15–5.60) | 3.22× (3.19–3.25) | 5.87× (5.77–5.97) | 2.40× (2.38–2.42) | 2.38× (2.36–2.40) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.11× (2.10–2.12) | 2.34× (2.33–2.35) | 2.74× (2.73–2.75) | 3.16× (3.15–3.18) | 3.80× (3.79–3.82) | 2.65× (2.63–2.67) | 1.77× (1.76–1.78) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.59× (1.55–1.65) | 1.46× (1.39–1.53) | 1.80× (1.72–1.87) | 1.70× (1.62–1.78) | 1.84× (1.75–1.92) | 1.07× (1.02–1.13) | 0.72× (0.69–0.75) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.21× (1.20–1.22) | 2.29× (2.28–2.30) | 2.10× (2.09–2.11) | 2.20× (2.19–2.21) | 2.56× (2.54–2.57) | 2.76× (2.75–2.78) | 2.73× (2.71–2.74) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.41× (1.36–1.46) | 2.15× (2.10–2.20) | 1.89× (1.87–1.91) | 2.52× (2.49–2.56) | 2.46× (2.44–2.48) | 2.02× (2.00–2.04) | 2.00× (1.98–2.02) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.13× (1.13–1.13) | 2.28× (2.27–2.29) | 1.96× (1.95–1.97) | 2.07× (2.05–2.09) | 2.44× (2.36–2.53) | 2.96× (2.94–2.97) | 3.00× (2.99–3.00) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.38× (1.36–1.40) | 2.19× (2.16–2.22) | 1.73× (1.71–1.75) | 2.46× (2.42–2.51) | 2.30× (2.27–2.33) | 2.12× (2.09–2.15) | 2.15× (2.12–2.18) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.65× (0.64–0.66) | 2.30× (2.26–2.33) | 3.21× (3.14–3.27) | 2.65× (2.61–2.70) | 6.31× (6.21–6.40) | 2.04× (2.01–2.07) | 2.03× (2.00–2.06) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.56–0.58) | 2.39× (2.35–2.43) | 4.11× (4.03–4.19) | 2.69× (2.64–2.73) | 6.21× (6.09–6.32) | 1.99× (1.97–2.02) | 1.98× (1.95–2.01) |
| `text` (1.24 MB → 12.2 MB, 9.9×) | 1 | 1.31× (1.29–1.32) | 2.83× (2.80–2.85) | 3.22× (3.19–3.25) | 3.31× (3.28–3.34) | 3.80× (3.72–3.87) | 3.32× (3.29–3.35) | 3.27× (3.24–3.30) |
| `text` (1.24 MB → 12.2 MB, 9.9×) | 4 | 1.39× (1.37–1.41) | 2.15× (2.12–2.18) | 2.50× (2.47–2.52) | 2.61× (2.56–2.65) | 2.73× (2.70–2.76) | 1.98× (1.96–1.99) | 1.95× (1.93–1.96) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 26 MB | 0.32× (0.32–0.32) | 0.33× (0.32–0.34) | 1.15× (1.15–1.16) | 0.84× (0.83–0.84) | 11.75× (11.73–11.76) | 0.29× (0.29–0.29) | 0.30× (0.30–0.31) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 74 MB | 0.40× (0.40–0.41) | 0.62× (0.53–0.72) | 0.77× (0.75–0.78) | 0.64× (0.63–0.64) | 4.27× (4.24–4.31) | 0.10× (0.10–0.10) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.81× (0.81–0.81) | 0.80× (0.80–0.80) | 4.28× (4.27–4.28) | 0.26× (0.26–0.26) | 0.27× (0.27–0.27) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.78× (1.77–1.78) | 0.83× (0.83–0.83) | 4.40× (4.39–4.41) | 0.27× (0.26–0.27) | 0.28× (0.28–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 17 MB | 0.50× (0.50–0.50) | 0.43× (0.43–0.43) | 1.62× (1.61–1.62) | 0.64× (0.63–0.64) | 7.74× (7.72–7.76) | 0.63× (0.63–0.64) | 0.66× (0.66–0.66) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 42 MB | 0.59× (0.58–0.60) | 0.62× (0.58–0.65) | 1.10× (1.08–1.13) | 0.85× (0.83–0.87) | 3.21× (3.15–3.27) | 0.25× (0.25–0.26) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 24 MB | 0.39× (0.39–0.40) | 0.31× (0.31–0.31) | 1.14× (1.14–1.14) | 0.71× (0.71–0.72) | 5.05× (5.04–5.07) | 0.32× (0.32–0.32) | 0.34× (0.34–0.34) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 53 MB | 0.58× (0.57–0.60) | 0.42× (0.41–0.43) | 0.79× (0.78–0.81) | 0.74× (0.71–0.76) | 2.33× (2.29–2.37) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 24 MB | 0.39× (0.39–0.39) | 0.31× (0.31–0.31) | 1.14× (1.14–1.14) | 0.81× (0.80–0.81) | 5.07× (5.06–5.07) | 0.32× (0.32–0.32) | 0.34× (0.33–0.34) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 58 MB | 0.56× (0.55–0.57) | 0.47× (0.44–0.50) | 0.74× (0.73–0.76) | 0.75× (0.71–0.78) | 2.17× (2.13–2.20) | 0.13× (0.13–0.13) | 0.14× (0.14–0.14) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.22× (0.21–0.23) | 1.07× (1.07–1.08) | 0.77× (0.76–0.77) | 5.40× (5.40–5.41) | 0.36× (0.36–0.36) | 0.37× (0.37–0.38) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.22× (0.22–0.23) | 0.20× (0.20–0.20) | 1.62× (1.62–1.62) | 0.76× (0.76–0.76) | 5.38× (5.37–5.39) | 0.36× (0.35–0.36) | 0.37× (0.37–0.38) |
| `text` (1.24 MB → 12.2 MB, 9.9×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.44× (1.44–1.45) | 0.61× (0.61–0.62) | 6.54× (6.53–6.55) | 0.41× (0.41–0.41) | 0.43× (0.43–0.43) |
| `text` (1.24 MB → 12.2 MB, 9.9×) | 4 | 47 MB | 0.56× (0.56–0.57) | 0.52× (0.48–0.57) | 1.07× (1.06–1.07) | 0.78× (0.74–0.81) | 2.73× (2.69–2.76) | 0.16× (0.16–0.16) | 0.17× (0.17–0.17) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.08 | 0.86 | 0.02 | – | 0.333 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3694 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 56.0s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.16 | 11.4% |
| MTF + RLE2 | 2.09 | 20.7% |
| T-vector build | 1.72 | 17.0% |
| walk (the one each build uses) | 4.69 | 46.4% |
| of which: segment assembly | 0.15 | 1.5% |
| RLE1 | 0.19 | 1.9% |
| CRC | 0.26 | 2.5% |
| **total** | 10.10 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h28m ± 10m45s |
| 4 threads | 48m29s ± 6m48s |
| 4 threads, the sample's overlap gap as a fixed cost | 31m06s ± 4m23s |
| the scanner alone (its serial part) | 11m21s ± 26.3s |
| the workers alone, over 4 | 31m06s ± 4m23s |

At 4 threads the workers bound it. Per byte: the scanner 6.6 ns a byte of bz2, the workers 4.08 ns a byte of output at 4 threads (CPU, summed over them), 2.52 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 11 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,236,217 | 12,227,335 | \`294c8c24e2835ff3…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3694.5 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1017-azure; image ubuntu24 20261026.354.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source) - bench 2.1.1 (bench/report.py), 10 rounds, threads 1, 4
