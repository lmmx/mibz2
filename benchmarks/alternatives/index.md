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
| bz2ngrs | Softleif's safe-Rust port of bz2-ng, at a pinned commit: `DecompressReader` | single-threaded (one thread a stream) |
| bz2ng | bz2-ng's C, at a pinned commit: its `bzip2 -dc`, read through a pipe (the reference bz2ngrs ports) | single-threaded |

**Single-threaded decoders run at 1 thread only.** libbzip2, libbz2-rs, bz2ngrs and bz2ng have no thread count, so they never decode on 4 threads. At 4 threads, their figures are their 1-thread runs against mibz2's at 4 threads: in the chart, hollow points with dashed whiskers, under "single-threaded (1 thread)"; in the tables, marked ¹.

**mibz2's inverse BWT against bz2ngrs's and libsais's.** Below the decoders' tables, the kernels run on the same Wikidata blocks, with the build, the walk and the output's assembly timed apart where a kernel allows it. libsais 2.12.1 (Ilya Grebnov) is the fastest inverse BWT measured here. It is a library of BWT algorithms, not a bzip2 decoder, so it appears in that table only, never in the chart. Its build and walk happen inside one call, so it shows a total. bz2ngrs and mibz2 store the same successor link in each 32-bit entry, but a different byte beside it: bz2ngrs the row's first-column byte, mibz2 its last-column byte (lbzip2's packing). The table says which phase a whole-decoder difference comes from ([the study](<https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-29-bz2ngrs.md>)).

Not included: `bzip2 -dc` and Python's `bz2` module, which are libbzip2 itself (its arm stands for them); and bz2k, mibz2's own prototype, which it replaced.

The latest record: [`16646a9`](<https://github.com/lmmx/mibz2/commit/16646a9bb6de6c9af34689bc78788b1e79883864>) (2026-11-06).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/42309987502>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.2.3

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.646 s | 325 | 10.0 | 2.33× (2.32–2.34) | 5.34× (5.31–5.37) | 5.45× (5.42–5.48) | 5.95× (5.92–5.99) | 5.31× (5.27–5.36) | 5.74× (5.70–5.77) | 5.47× (5.43–5.50) | 1.73× (1.72–1.74) | 2.02× (2.01–2.03) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.360 s | 582 | 10.8 | 2.36× (2.29–2.43) | 5.12× (4.90–5.34) | 5.71× (5.55–5.86) | 5.70× (5.51–5.89) | 5.10× (4.99–5.21) | 10.17× (9.95–10.41) | 9.70× (9.47–9.92) | 3.07× (3.00–3.14) | 3.58× (3.50–3.66) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.300 s | 698 | 16.9 | 2.31× (2.23–2.39) | 3.49× (3.38–3.60) | 4.36× (4.20–4.52) | 3.97× (3.84–4.11) | 3.99× (3.86–4.13) | 12.19× (11.77–12.62) | 11.61× (11.23–12.01) | 3.67× (3.56–3.80) | 4.29× (4.15–4.43) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 978 | 3.6 | 0.59× (0.57–0.62) | 3.69× (3.51–3.89) | 4.37× (4.20–4.55) | 3.52× (3.39–3.66) | 6.08× (5.84–6.32) | 2.80× (2.70–2.91) | 2.63× (2.53–2.73) | 0.99× (0.96–1.03) | 1.49× (1.43–1.55) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 964 | 3.7 | 0.57× (0.56–0.58) | 2.97× (2.93–3.02) | 5.91× (5.79–6.04) | 3.54× (3.49–3.59) | 6.03× (5.93–6.12) | 2.80× (2.76–2.84) | 2.63× (2.59–2.67) | 0.99× (0.97–1.01) | 1.49× (1.46–1.51) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 946 | 3.8 | 0.57× (0.56–0.58) | 2.93× (2.88–2.97) | 5.85× (5.70–6.00) | 3.47× (3.42–3.53) | 5.96× (5.88–6.05) | 2.74× (2.70–2.79) | 2.58× (2.54–2.61) | 0.97× (0.96–0.99) | 1.46× (1.44–1.48) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.036 s | 548 | 6.2 | 2.47× (2.46–2.49) | 2.85× (2.83–2.86) | 3.66× (3.64–3.68) | 3.60× (3.58–3.61) | 3.89× (3.87–3.91) | 2.34× (2.33–2.35) | 1.98× (1.97–1.98) | 1.89× (1.87–1.90) | 2.83× (2.82–2.85) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.025 s | 800 | 7.8 | 1.98× (1.83–2.14) | 2.44× (2.25–2.64) | 3.64× (3.38–3.93) | 3.06× (2.86–3.27) | 4.25× (3.97–4.56) | 3.31× (3.06–3.59) | 2.80× (2.58–3.03) | 2.67× (2.46–2.90) | 4.01× (3.69–4.34) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.026 s | 752 | 15.0 | 1.72× (1.61–1.84) | 1.84× (1.69–2.01) | 2.77× (2.58–2.98) | 2.04× (1.91–2.18) | 3.71× (3.48–3.96) | 3.16× (2.97–3.36) | 2.66× (2.50–2.83) | 2.54× (2.38–2.72) | 3.82× (3.58–4.07) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.127 s | 63 | 52.4 | 1.37× (1.36–1.38) | 2.76× (2.73–2.79) | 2.47× (2.45–2.50) | 2.56× (2.55–2.57) | 2.92× (2.91–2.93) | 3.12× (3.11–3.14) | 3.28× (3.25–3.30) | 1.27× (1.27–1.28) | 1.43× (1.43–1.44) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.079 s | 101 | 59.7 | 1.27× (1.20–1.35) | 2.55× (2.41–2.69) | 2.39× (2.26–2.53) | 3.10× (2.96–3.24) | 2.94× (2.78–3.10) | 4.93× (4.68–5.19) | 5.17× (4.90–5.46) | 2.01× (1.91–2.12) | 2.26× (2.14–2.38) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.067 s | 119 | 82.6 | 1.32× (1.29–1.35) | 2.14× (2.08–2.19) | 2.06× (2.00–2.11) | 2.56× (2.53–2.60) | 2.84× (2.80–2.89) | 5.87× (5.77–5.96) | 6.16× (6.05–6.27) | 2.39× (2.36–2.43) | 2.69× (2.65–2.73) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.142 s | 68 | 48.6 | 1.45× (1.44–1.46) | 3.12× (3.08–3.16) | 2.61× (2.49–2.74) | 2.70× (2.68–2.72) | 3.14× (3.04–3.25) | 3.76× (3.74–3.78) | 4.01× (3.98–4.04) | 1.44× (1.42–1.46) | 1.71× (1.70–1.72) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.087 s | 110 | 55.3 | 1.34× (1.31–1.37) | 2.80× (2.73–2.87) | 2.50× (2.44–2.57) | 3.49× (3.43–3.56) | 3.10× (3.03–3.17) | 6.12× (6.02–6.23) | 6.53× (6.42–6.65) | 2.34× (2.29–2.40) | 2.78× (2.73–2.84) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.078 s | 124 | 80.9 | 1.38× (1.34–1.41) | 2.24× (2.17–2.31) | 2.20× (2.15–2.25) | 2.91× (2.85–2.97) | 2.77× (2.71–2.82) | 6.93× (6.80–7.07) | 7.40× (7.27–7.53) | 2.65× (2.60–2.71) | 3.15× (3.09–3.21) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 738 | 5.0 | 0.67× (0.66–0.67) | 3.10× (2.95–3.25) | 4.44× (4.40–4.48) | 3.12× (3.09–3.15) | 6.81× (6.77–6.85) | 2.47× (2.45–2.49) | 2.34× (2.32–2.36) | 0.98× (0.97–0.99) | 1.53× (1.51–1.54) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 676 | 5.5 | 0.55× (0.54–0.55) | 2.80× (2.76–2.85) | 4.82× (4.74–4.89) | 2.92× (2.88–2.96) | 6.29× (6.18–6.41) | 2.28× (2.25–2.31) | 2.16× (2.13–2.19) | 0.90× (0.89–0.92) | 1.41× (1.39–1.43) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 687 | 5.4 | 0.57× (0.56–0.58) | 2.85× (2.81–2.89) | 4.88× (4.82–4.95) | 2.96× (2.92–3.00) | 6.33× (6.28–6.39) | 2.31× (2.28–2.34) | 2.19× (2.16–2.22) | 0.92× (0.90–0.93) | 1.43× (1.41–1.45) | yes |
| `text` (1.76 MB → 12 MB, 6.8×) | 1 | 0.081 s | 149 | 22.3 | 1.49× (1.48–1.50) | 3.15× (3.12–3.17) | 3.37× (3.34–3.39) | 3.35× (3.33–3.38) | 3.57× (3.53–3.61) | 3.80× (3.77–3.82) | 3.73× (3.71–3.76) | 1.15× (1.14–1.15) | 1.38× (1.37–1.39) | yes |
| `text` (1.76 MB → 12 MB, 6.8×) | 2 | 0.053 s | 227 | 26.2 | 1.27× (1.17–1.37) | 2.60× (2.42–2.79) | 3.32× (3.17–3.48) | 3.42× (3.17–3.69) | 3.34× (3.16–3.53) | 5.70× (5.39–6.02) | 5.60× (5.30–5.92) | 1.72× (1.63–1.82) | 2.07× (1.96–2.19) | yes |
| `text` (1.76 MB → 12 MB, 6.8×) | 4 | 0.044 s | 270 | 37.9 | 1.29× (1.25–1.33) | 2.10× (1.96–2.26) | 2.90× (2.77–3.04) | 2.95× (2.83–3.07) | 3.10× (3.01–3.19) | 6.87× (6.69–7.06) | 6.76× (6.57–6.95) | 2.08× (2.02–2.14) | 2.50× (2.43–2.57) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.33× (2.31–2.34) | 5.32× (5.29–5.35) | 5.43× (5.40–5.46) | 5.94× (5.90–5.97) | 5.30× (5.25–5.35) | 5.72× (5.69–5.76) | 5.45× (5.42–5.49) | 1.73× (1.72–1.74) | 2.01× (1.99–2.02) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.25× (2.24–2.26) | 5.03× (5.00–5.06) | 5.27× (5.24–5.30) | 5.79× (5.75–5.82) | 4.98× (4.95–5.01) | 5.27× (5.23–5.32) | 5.03× (4.99–5.06) | 1.59× (1.58–1.60) | 1.85× (1.83–1.86) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.27× (2.21–2.33) | 3.80× (3.70–3.89) | 4.10× (4.02–4.18) | 4.32× (4.24–4.40) | 3.82× (3.74–3.90) | 3.41× (3.34–3.48) | 3.25× (3.18–3.31) | 1.03× (1.01–1.05) | 1.19× (1.17–1.22) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.63× (0.60–0.65) | 3.23× (3.12–3.35) | 4.00× (3.85–4.16) | 3.30× (3.17–3.43) | 5.63× (5.41–5.86) | 2.64× (2.54–2.75) | 2.49× (2.39–2.59) | 0.99× (0.96–1.03) | 1.09× (1.02–1.16) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.62× (0.61–0.63) | 3.07× (3.04–3.10) | 5.34× (5.27–5.42) | 3.30× (3.26–3.34) | 5.52× (5.44–5.60) | 2.61× (2.57–2.64) | 2.45× (2.42–2.48) | 0.98× (0.96–1.00) | 1.07× (1.02–1.13) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.61× (0.60–0.61) | 3.36× (3.27–3.45) | 5.35× (5.27–5.43) | 3.28× (3.23–3.33) | 5.52× (5.45–5.60) | 2.60× (2.56–2.64) | 2.44× (2.41–2.48) | 0.98× (0.96–0.99) | 1.07× (1.01–1.13) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.40× (2.38–2.41) | 2.62× (2.61–2.64) | 3.29× (3.26–3.33) | 3.46× (3.44–3.48) | 3.74× (3.72–3.76) | 2.27× (2.26–2.28) | 1.93× (1.92–1.93) | 1.84× (1.83–1.85) | 2.55× (2.53–2.58) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.08× (2.04–2.12) | 2.32× (2.27–2.38) | 2.81× (2.77–2.86) | 2.93× (2.88–2.98) | 3.13× (3.07–3.18) | 1.82× (1.79–1.84) | 1.54× (1.52–1.57) | 1.47× (1.44–1.50) | 2.04× (2.00–2.08) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.87× (1.81–1.94) | 1.61× (1.54–1.68) | 2.03× (1.97–2.10) | 1.96× (1.88–2.04) | 1.96× (1.90–2.03) | 0.94× (0.91–0.97) | 0.79× (0.77–0.82) | 0.76× (0.73–0.78) | 1.05× (1.02–1.08) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.36× (1.36–1.37) | 2.70× (2.68–2.71) | 2.42× (2.40–2.44) | 2.54× (2.53–2.55) | 2.89× (2.88–2.90) | 3.09× (3.08–3.10) | 3.24× (3.22–3.27) | 1.27× (1.26–1.27) | 1.40× (1.39–1.40) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.27× (1.26–1.28) | 2.51× (2.48–2.54) | 2.22× (2.18–2.26) | 2.96× (2.93–2.99) | 2.61× (2.57–2.65) | 2.71× (2.69–2.74) | 2.84× (2.80–2.88) | 1.11× (1.10–1.12) | 1.22× (1.21–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.45× (1.43–1.47) | 2.27× (2.22–2.31) | 1.85× (1.81–1.89) | 2.55× (2.50–2.59) | 2.41× (2.37–2.44) | 1.96× (1.92–1.99) | 2.05× (2.01–2.09) | 0.80× (0.79–0.82) | 0.88× (0.87–0.90) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.45× (1.44–1.45) | 3.06× (3.02–3.10) | 2.57× (2.46–2.69) | 2.68× (2.66–2.70) | 3.12× (3.01–3.22) | 3.72× (3.71–3.74) | 3.97× (3.94–4.00) | 1.43× (1.42–1.45) | 1.67× (1.66–1.67) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.34× (1.34–1.35) | 2.80× (2.78–2.81) | 2.32× (2.31–2.34) | 3.27× (3.23–3.31) | 2.79× (2.72–2.86) | 3.26× (3.23–3.30) | 3.48× (3.45–3.51) | 1.26× (1.24–1.28) | 1.46× (1.45–1.48) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.54× (1.52–1.57) | 2.48× (2.44–2.53) | 1.90× (1.86–1.94) | 2.72× (2.68–2.75) | 2.47× (2.44–2.50) | 2.24× (2.22–2.27) | 2.39× (2.36–2.43) | 0.86× (0.85–0.88) | 1.01× (0.99–1.02) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.71× (0.70–0.72) | 2.72× (2.68–2.76) | 3.56× (3.52–3.60) | 2.85× (2.82–2.88) | 6.08× (6.05–6.11) | 2.29× (2.26–2.31) | 2.17× (2.14–2.20) | 0.99× (0.98–1.00) | 0.98× (0.95–1.02) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.61× (0.60–0.62) | 2.45× (2.41–2.49) | 4.17× (4.11–4.22) | 2.70× (2.66–2.74) | 5.57× (5.47–5.68) | 2.09× (2.05–2.12) | 1.98× (1.95–2.02) | 0.90× (0.89–0.91) | 0.90× (0.86–0.93) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.61× (0.60–0.62) | 2.73× (2.69–2.78) | 4.26× (4.18–4.35) | 2.80× (2.77–2.84) | 5.71× (5.66–5.76) | 2.15× (2.12–2.18) | 2.04× (2.02–2.07) | 0.93× (0.91–0.95) | 0.93× (0.90–0.96) |
| `text` (1.76 MB → 12 MB, 6.8×) | 1 | 1.48× (1.47–1.49) | 2.99× (2.97–3.02) | 3.26× (3.24–3.28) | 3.30× (3.27–3.32) | 3.51× (3.47–3.54) | 3.73× (3.71–3.75) | 3.67× (3.64–3.69) | 1.14× (1.14–1.15) | 1.35× (1.33–1.36) |
| `text` (1.76 MB → 12 MB, 6.8×) | 2 | 1.35× (1.32–1.37) | 2.65× (2.61–2.69) | 2.92× (2.88–2.96) | 3.14× (3.10–3.19) | 3.04× (2.99–3.09) | 3.15× (3.11–3.19) | 3.10× (3.06–3.14) | 0.97× (0.95–0.98) | 1.14× (1.12–1.15) |
| `text` (1.76 MB → 12 MB, 6.8×) | 4 | 1.55× (1.51–1.60) | 2.26× (2.20–2.31) | 2.48× (2.42–2.54) | 2.62× (2.56–2.69) | 2.55× (2.50–2.61) | 2.21× (2.16–2.25) | 2.17× (2.13–2.22) | 0.68× (0.66–0.69) | 0.80× (0.78–0.81) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 25 MB | 0.33× (0.33–0.33) | 0.33× (0.32–0.34) | 1.08× (1.08–1.08) | 0.86× (0.86–0.86) | 12.23× (12.22–12.23) | 0.28× (0.28–0.29) | 0.30× (0.30–0.31) | 0.46× (0.46–0.47) | 0.24× (0.24–0.24) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 48 MB | 0.38× (0.37–0.38) | 0.58× (0.51–0.65) | 0.80× (0.79–0.82) | 0.67× (0.66–0.67) | 6.41× (6.37–6.45) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 77 MB | 0.39× (0.38–0.39) | 0.72× (0.63–0.82) | 0.70× (0.70–0.71) | 0.61× (0.59–0.62) | 4.10× (4.07–4.13) | 0.09× (0.09–0.09) | 0.10× (0.10–0.10) | 0.15× (0.15–0.15) | 0.08× (0.08–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.19) | 0.83× (0.82–0.83) | 0.81× (0.81–0.81) | 4.33× (4.32–4.34) | 0.25× (0.25–0.26) | 0.27× (0.26–0.27) | 0.91× (0.91–0.92) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.17) | 1.80× (1.80–1.80) | 0.84× (0.83–0.84) | 4.44× (4.43–4.45) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.94× (0.93–0.94) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 32 MB | 0.19× (0.19–0.19) | 0.16× (0.16–0.16) | 1.80× (1.80–1.80) | 0.84× (0.83–0.84) | 4.44× (4.43–4.45) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.94× (0.93–0.94) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.51) | 0.45× (0.45–0.46) | 1.74× (1.74–1.75) | 0.66× (0.66–0.67) | 8.21× (8.18–8.23) | 0.66× (0.65–0.66) | 0.69× (0.68–0.69) | 0.75× (0.75–0.76) | 0.38× (0.38–0.38) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.65× (0.64–0.67) | 0.74× (0.69–0.80) | 1.63× (1.60–1.66) | 0.80× (0.78–0.81) | 5.88× (5.78–5.98) | 0.47× (0.46–0.48) | 0.49× (0.48–0.50) | 0.54× (0.53–0.55) | 0.27× (0.27–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.64× (0.61–0.67) | 0.70× (0.67–0.74) | 1.17× (1.13–1.21) | 0.76× (0.74–0.79) | 3.44× (3.33–3.55) | 0.26× (0.26–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 22 MB | 0.40× (0.40–0.40) | 0.33× (0.32–0.33) | 1.20× (1.20–1.20) | 0.74× (0.74–0.74) | 5.29× (5.28–5.30) | 0.33× (0.32–0.33) | 0.34× (0.34–0.35) | 0.53× (0.53–0.53) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.50× (0.48–0.52) | 0.41× (0.38–0.45) | 0.93× (0.89–0.96) | 0.69× (0.66–0.72) | 3.19× (3.08–3.30) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.32× (0.31–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.62–0.65) | 0.45× (0.44–0.46) | 0.86× (0.84–0.89) | 0.80× (0.78–0.83) | 2.51× (2.45–2.57) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.20× (1.20–1.20) | 0.83× (0.83–0.84) | 5.29× (5.28–5.29) | 0.32× (0.32–0.32) | 0.34× (0.33–0.34) | 0.52× (0.52–0.52) | 0.27× (0.27–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.46–0.49) | 0.39× (0.37–0.42) | 0.87× (0.85–0.89) | 0.71× (0.67–0.75) | 2.96× (2.89–3.04) | 0.18× (0.18–0.18) | 0.19× (0.18–0.19) | 0.29× (0.28–0.30) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.59× (0.58–0.60) | 0.49× (0.46–0.53) | 0.81× (0.79–0.82) | 0.79× (0.76–0.81) | 2.31× (2.28–2.35) | 0.14× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.21× (0.21–0.21) | 1.10× (1.10–1.10) | 0.77× (0.77–0.77) | 5.47× (5.46–5.47) | 0.35× (0.35–0.35) | 0.37× (0.37–0.37) | 0.93× (0.93–0.93) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.21–0.21) | 1.64× (1.64–1.65) | 0.77× (0.76–0.77) | 5.41× (5.39–5.43) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.92× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.23× (0.23–0.23) | 0.21× (0.21–0.21) | 1.65× (1.64–1.65) | 0.77× (0.77–0.77) | 5.44× (5.42–5.45) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `text` (1.76 MB → 12 MB, 6.8×) | 1 | 18 MB | 0.47× (0.47–0.47) | 0.41× (0.41–0.41) | 1.51× (1.51–1.51) | 0.65× (0.65–0.66) | 6.78× (6.76–6.79) | 0.41× (0.40–0.41) | 0.43× (0.42–0.43) | 0.66× (0.66–0.67) | 0.33× (0.33–0.33) |
| `text` (1.76 MB → 12 MB, 6.8×) | 2 | 32 MB | 0.50× (0.49–0.51) | 0.43× (0.39–0.48) | 1.23× (1.20–1.26) | 0.76× (0.75–0.77) | 3.87× (3.82–3.93) | 0.23× (0.23–0.23) | 0.24× (0.24–0.25) | 0.38× (0.38–0.38) | 0.19× (0.19–0.19) |
| `text` (1.76 MB → 12 MB, 6.8×) | 4 | 49 MB | 0.56× (0.55–0.57) | 0.55× (0.52–0.59) | 1.04× (1.02–1.07) | 0.81× (0.80–0.83) | 2.67× (2.60–2.73) | 0.15× (0.15–0.16) | 0.16× (0.15–0.16) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 325 | 3.09 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 582 | 3.34 | 1.08× (1.08–1.09) | 89% | 1.03× (1.02–1.03) | 1.05× (1.04–1.05) |
| 4: all 4 CPUs, sharing SMT siblings | 698 | 5.22 | 1.68× (1.64–1.72) | 53% | 1.20× (1.19–1.21) | 1.64× (1.61–1.67) |

**Best:** mibz2 at 4 threads (698 MB/s); lbzip2 at 4 threads (198 MB/s); pbz2 at 4 threads (299 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.68, mibz2 4t 1.15, lbzip2 1t 0.09, lbzip2 2t 0.35, lbzip2 4t 0.50, pbz2 1t 0.00, pbz2 2t 0.30, pbz2 4t 0.42. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 8%, mibz2 4t 8%, lbzip2 2t 4%, lbzip2 4t 5%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.89 | 0.02 | 0.09 | 0.300 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3240 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m04s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.14 | 12.1% |
| MTF + RLE2 | 1.51 | 16.1% |
| T-vector build | 2.13 | 22.7% |
| walk (the one each build uses) | 4.05 | 43.3% |
| of which: segment assembly | 0.15 | 1.6% |
| RLE1 | 0.29 | 3.1% |
| CRC | 0.26 | 2.7% |
| **total** | 9.37 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.672 | 1.301 | (in its total) | 1.94× (1.93–1.94) |  |
| walk | 1.213 | 2.614 | (in its total) | 2.16× (2.15–2.17) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.045 | 0.055 | (in its total) | 1.24× (1.21–1.26) |  |
| inverse BWT, total | 1.930 | 3.970 | 3.271 | 2.06× (2.05–2.07) | 1.70× (1.69–1.71) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h33m ± 11m02s |
| 4 threads | 44m17s ± 6m23s |
| 4 threads, the sample's overlap gap as a fixed cost | 31m58s ± 4m34s |
| the scanner alone (its serial part) | 15m17s ± 32.9s |
| the workers alone, over 4 | 31m58s ± 4m34s |

At 4 threads the workers bound it. Per byte: the scanner 8.9 ns a byte of bz2, the workers 4.19 ns a byte of output at 4 threads (CPU, summed over them), 2.58 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 8 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 1,758,792 | 12,027,975 | \`3c64b8b3acbf9c14…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 3239.9 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261101.186.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.2.3 (bench/report.py), 10 rounds, threads 1, 2, 4
