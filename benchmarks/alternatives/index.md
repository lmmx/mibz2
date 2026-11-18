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

The latest record: [`7a852b3`](<https://github.com/lmmx/mibz2/commit/7a852b3d9e60c3e35b3f9912c1464e6f45b56771>) (2026-11-18).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44391826784>)

AMD EPYC 9V45 96-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.296 s | 707 | 6.1 | 2.85× (2.80–2.90) | 6.94× (6.83–7.06) | 7.28× (7.10–7.47) | 8.32× (8.08–8.58) | 7.42× (7.23–7.62) | 7.62× (7.42–7.83) | 7.29× (7.14–7.44) | 2.21× (2.15–2.27) | 2.53× (2.50–2.57) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.181 s | 1,159 | 7.0 | 2.92× (2.75–3.10) | 6.17× (5.84–6.52) | 7.60× (7.15–8.08) | 7.45× (7.00–7.93) | 6.61× (6.30–6.93) | 12.48× (11.93–13.05) | 11.93× (11.39–12.50) | 3.62× (3.44–3.80) | 4.15× (3.92–4.38) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.156 s | 1,342 | 10.8 | 2.82× (2.72–2.94) | 4.42× (4.28–4.56) | 5.87× (5.62–6.14) | 5.36× (5.22–5.50) | 5.44× (5.27–5.62) | 14.85× (14.09–15.65) | 14.20× (13.48–14.96) | 4.30× (4.06–4.56) | 4.94× (4.69–5.20) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.014 s | 1,394 | 3.5 | 0.57× (0.56–0.58) | 3.58× (3.48–3.68) | 4.73× (4.48–5.00) | 3.72× (3.63–3.80) | 6.66× (6.41–6.91) | 2.77× (2.71–2.82) | 2.79× (2.70–2.87) | 1.01× (0.99–1.03) | 2.15× (2.09–2.21) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.015 s | 1,324 | 3.6 | 0.51× (0.50–0.52) | 3.03× (2.59–3.53) | 5.82× (5.65–6.00) | 3.60× (3.52–3.67) | 6.32× (6.19–6.44) | 2.65× (2.59–2.72) | 2.67× (2.61–2.73) | 0.97× (0.94–1.01) | 2.06× (2.01–2.12) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.015 s | 1,351 | 3.5 | 0.52× (0.51–0.54) | 3.02× (2.77–3.29) | 6.02× (5.54–6.54) | 3.66× (3.52–3.80) | 6.31× (6.11–6.52) | 2.65× (2.57–2.74) | 2.67× (2.58–2.77) | 0.97× (0.93–1.02) | 2.06× (2.01–2.12) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.018 s | 1,071 | 4.4 | 3.00× (2.94–3.07) | 3.69× (3.55–3.84) | 4.37× (4.06–4.72) | 4.63× (4.58–4.69) | 5.48× (5.43–5.53) | 3.08× (3.00–3.16) | 2.63× (2.55–2.70) | 2.19× (2.13–2.25) | 3.30× (3.22–3.39) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.013 s | 1,508 | 5.7 | 2.18× (1.84–2.58) | 2.94× (2.45–3.53) | 4.32× (3.60–5.19) | 3.92× (3.13–4.91) | 5.56× (4.64–6.67) | 4.07× (3.33–4.98) | 3.47× (2.86–4.21) | 2.90× (2.41–3.49) | 4.37× (3.64–5.25) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.017 s | 1,136 | 12.0 | 1.63× (1.53–1.75) | 1.87× (1.74–2.00) | 2.81× (2.55–3.10) | 1.98× (1.85–2.11) | 4.11× (3.85–4.39) | 3.27× (3.04–3.52) | 2.79× (2.59–3.01) | 2.33× (2.18–2.48) | 3.51× (3.31–3.73) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.083 s | 96 | 45.8 | 1.30× (1.26–1.34) | 2.91× (2.84–2.99) | 2.65× (2.56–2.74) | 2.57× (2.49–2.64) | 2.93× (2.85–3.01) | 3.29× (3.21–3.37) | 3.28× (3.20–3.37) | 1.26× (1.23–1.28) | 1.47× (1.42–1.52) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.055 s | 146 | 54.4 | 1.17× (1.10–1.25) | 2.67× (2.56–2.78) | 2.47× (2.33–2.61) | 2.94× (2.74–3.16) | 2.85× (2.66–3.04) | 4.96× (4.66–5.28) | 4.95× (4.63–5.30) | 1.90× (1.77–2.03) | 2.22× (2.07–2.38) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.045 s | 179 | 71.7 | 1.25× (1.22–1.28) | 2.34× (2.26–2.42) | 2.32× (2.25–2.38) | 2.73× (2.64–2.83) | 3.00× (2.90–3.11) | 6.23× (6.00–6.46) | 6.22× (5.98–6.47) | 2.38× (2.30–2.47) | 2.78× (2.66–2.91) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.087 s | 110 | 39.9 | 1.46× (1.44–1.49) | 3.60× (3.50–3.70) | 2.98× (2.90–3.06) | 3.01× (2.93–3.10) | 3.43× (3.35–3.50) | 4.31× (4.19–4.43) | 4.65× (4.57–4.74) | 1.56× (1.53–1.59) | 1.79× (1.74–1.84) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.057 s | 168 | 48.1 | 1.25× (1.15–1.35) | 3.25× (3.09–3.43) | 2.60× (2.40–2.83) | 3.50× (3.13–3.90) | 3.09× (2.82–3.39) | 6.38× (5.79–7.03) | 6.89× (6.22–7.63) | 2.31× (2.09–2.55) | 2.65× (2.40–2.93) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.050 s | 192 | 68.0 | 1.35× (1.32–1.38) | 2.51× (2.44–2.59) | 2.44× (2.29–2.59) | 3.25× (3.12–3.39) | 3.05× (2.93–3.18) | 7.66× (7.34–8.00) | 8.28× (7.88–8.69) | 2.77× (2.65–2.91) | 3.19× (3.06–3.31) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.010 s | 1,031 | 4.9 | 0.62× (0.61–0.63) | 3.17× (3.09–3.26) | 4.40× (4.26–4.53) | 3.13× (3.03–3.22) | 7.10× (6.80–7.41) | 2.31× (2.23–2.39) | 2.32× (2.24–2.40) | 0.98× (0.95–1.02) | 2.12× (2.06–2.18) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.010 s | 1,000 | 5.2 | 0.49× (0.47–0.52) | 3.01× (2.81–3.22) | 5.18× (4.84–5.55) | 2.97× (2.81–3.14) | 6.67× (6.27–7.09) | 2.18× (2.06–2.30) | 2.19× (2.08–2.31) | 0.93× (0.87–0.99) | 2.00× (1.88–2.14) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.010 s | 985 | 5.0 | 0.51× (0.50–0.53) | 3.06× (2.97–3.15) | 5.25× (5.08–5.42) | 3.04× (2.95–3.14) | 6.77× (6.55–6.99) | 2.21× (2.15–2.27) | 2.22× (2.17–2.27) | 0.94× (0.90–0.98) | 2.03× (1.97–2.09) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 0.044 s | 295 | 15.2 | 1.75× (1.71–1.79) | 4.25× (4.15–4.36) | 4.60× (4.42–4.79) | 4.77× (4.52–5.04) | 4.95× (4.80–5.11) | 5.03× (4.91–5.15) | 4.90× (4.64–5.17) | 1.51× (1.48–1.54) | 1.76× (1.67–1.85) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 0.032 s | 404 | 19.3 | 1.24× (1.09–1.40) | 2.89× (2.50–3.33) | 3.54× (3.09–4.07) | 3.85× (3.20–4.63) | 3.88× (3.35–4.50) | 6.09× (5.30–7.00) | 5.93× (5.04–6.98) | 1.83× (1.60–2.09) | 2.13× (1.87–2.43) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 0.031 s | 422 | 29.5 | 1.25× (1.20–1.30) | 2.39× (2.32–2.45) | 3.11× (2.97–3.25) | 3.20× (3.06–3.34) | 3.73× (3.64–3.81) | 7.05× (6.83–7.27) | 6.86× (6.48–7.27) | 2.11× (2.03–2.19) | 2.47× (2.34–2.60) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.84× (2.79–2.89) | 6.89× (6.78–7.00) | 7.23× (7.05–7.41) | 8.28× (8.04–8.54) | 7.38× (7.20–7.58) | 7.58× (7.38–7.79) | 7.25× (7.10–7.41) | 2.20× (2.14–2.27) | 2.50× (2.46–2.54) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.70× (2.61–2.80) | 6.41× (6.26–6.57) | 6.92× (6.73–7.11) | 7.93× (7.69–8.16) | 6.68× (6.47–6.90) | 6.73× (6.54–6.93) | 6.44× (6.25–6.63) | 1.96× (1.88–2.03) | 2.22× (2.14–2.30) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.78× (2.68–2.88) | 5.02× (4.84–5.22) | 5.68× (5.41–5.96) | 6.12× (5.99–6.26) | 5.38× (5.25–5.51) | 4.35× (4.11–4.60) | 4.16× (3.95–4.38) | 1.26× (1.19–1.34) | 1.43× (1.36–1.51) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.62) | 3.06× (2.97–3.15) | 3.98× (3.87–4.09) | 3.42× (3.34–3.51) | 6.07× (5.80–6.36) | 2.57× (2.52–2.63) | 2.59× (2.52–2.67) | 1.01× (0.99–1.04) | 1.82× (1.77–1.87) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.57× (0.55–0.58) | 2.94× (2.53–3.43) | 4.96× (4.87–5.05) | 3.31× (3.25–3.37) | 5.70× (5.60–5.80) | 2.45× (2.39–2.51) | 2.46× (2.41–2.52) | 0.96× (0.93–1.00) | 1.73× (1.70–1.77) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.56–0.58) | 3.03× (2.94–3.12) | 5.21× (4.81–5.63) | 3.45× (3.32–3.57) | 5.81× (5.63–6.00) | 2.51× (2.43–2.59) | 2.53× (2.44–2.62) | 0.99× (0.95–1.03) | 1.78× (1.73–1.82) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.85× (2.79–2.90) | 3.19× (3.15–3.24) | 3.90× (3.63–4.19) | 4.34× (4.29–4.40) | 5.13× (5.07–5.19) | 2.91× (2.84–2.99) | 2.50× (2.43–2.57) | 2.10× (2.05–2.15) | 2.78× (2.76–2.81) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.28× (2.17–2.40) | 2.62× (2.51–2.74) | 3.13× (2.98–3.28) | 3.46× (3.28–3.65) | 3.95× (3.73–4.19) | 2.19× (2.02–2.37) | 1.88× (1.74–2.02) | 1.58× (1.47–1.69) | 2.09× (1.95–2.24) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 1.86× (1.79–1.94) | 1.67× (1.61–1.72) | 2.06× (1.99–2.13) | 2.00× (1.92–2.07) | 2.24× (2.16–2.33) | 1.07× (1.02–1.13) | 0.92× (0.87–0.97) | 0.77× (0.74–0.81) | 1.02× (0.98–1.07) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.30× (1.26–1.34) | 2.81× (2.75–2.88) | 2.56× (2.48–2.65) | 2.54× (2.47–2.61) | 2.90× (2.82–2.98) | 3.24× (3.17–3.32) | 3.24× (3.16–3.32) | 1.25× (1.23–1.28) | 1.38× (1.34–1.43) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.21× (1.18–1.25) | 2.61× (2.54–2.68) | 2.27× (2.21–2.32) | 2.92× (2.82–3.02) | 2.58× (2.49–2.67) | 2.80× (2.72–2.88) | 2.79× (2.70–2.89) | 1.08× (1.05–1.11) | 1.19× (1.15–1.23) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.40× (1.36–1.44) | 2.53× (2.43–2.64) | 2.10× (2.04–2.16) | 2.78× (2.71–2.86) | 2.56× (2.50–2.61) | 2.13× (2.07–2.20) | 2.13× (2.07–2.20) | 0.82× (0.80–0.85) | 0.91× (0.88–0.94) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.46× (1.43–1.48) | 3.52× (3.42–3.61) | 2.88× (2.81–2.96) | 2.97× (2.89–3.06) | 3.38× (3.30–3.46) | 4.25× (4.13–4.37) | 4.58× (4.50–4.67) | 1.55× (1.52–1.58) | 1.73× (1.69–1.77) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.32× (1.30–1.35) | 3.16× (3.07–3.26) | 2.50× (2.44–2.56) | 3.45× (3.34–3.57) | 2.90× (2.82–2.98) | 3.58× (3.45–3.70) | 3.86× (3.71–4.02) | 1.30× (1.26–1.35) | 1.46× (1.40–1.52) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.47× (1.40–1.53) | 2.80× (2.72–2.88) | 2.14× (2.05–2.23) | 3.03× (2.91–3.17) | 2.73× (2.63–2.83) | 2.50× (2.38–2.62) | 2.70× (2.56–2.84) | 0.91× (0.87–0.96) | 1.02× (0.97–1.07) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.68) | 2.52× (2.44–2.59) | 3.34× (3.27–3.40) | 2.82× (2.73–2.90) | 6.21× (5.95–6.47) | 2.12× (2.04–2.20) | 2.13× (2.06–2.20) | 0.99× (0.95–1.02) | 1.47× (1.45–1.50) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.55–0.61) | 2.26× (2.10–2.42) | 3.92× (3.67–4.18) | 2.70× (2.57–2.84) | 5.77× (5.47–6.08) | 1.97× (1.88–2.07) | 1.98× (1.89–2.07) | 0.92× (0.86–0.97) | 1.37× (1.29–1.45) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.56–0.59) | 2.50× (2.42–2.59) | 4.04× (3.92–4.17) | 2.87× (2.78–2.96) | 6.06× (5.87–6.26) | 2.07× (2.02–2.13) | 2.08× (2.03–2.13) | 0.96× (0.93–1.00) | 1.44× (1.40–1.48) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 1.73× (1.69–1.77) | 3.99× (3.90–4.08) | 4.39× (4.24–4.54) | 4.64× (4.39–4.89) | 4.81× (4.66–4.97) | 4.89× (4.77–5.01) | 4.76× (4.51–5.02) | 1.49× (1.46–1.52) | 1.62× (1.58–1.66) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 1.42× (1.38–1.47) | 3.24× (3.14–3.34) | 3.58× (3.45–3.71) | 4.08× (3.91–4.26) | 3.82× (3.66–3.98) | 3.77× (3.59–3.95) | 3.67× (3.42–3.94) | 1.15× (1.11–1.19) | 1.25× (1.20–1.30) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 1.50× (1.45–1.55) | 2.70× (2.61–2.78) | 2.98× (2.86–3.10) | 3.31× (3.18–3.44) | 3.14× (3.05–3.24) | 2.43× (2.33–2.54) | 2.37× (2.20–2.55) | 0.74× (0.71–0.77) | 0.81× (0.77–0.84) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.30) | 0.34× (0.33–0.35) | 0.97× (0.97–0.97) | 0.79× (0.77–0.80) | 10.99× (10.98–11.01) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.42× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 59 MB | 0.32× (0.31–0.34) | 0.46× (0.40–0.53) | 0.66× (0.64–0.69) | 0.57× (0.54–0.59) | 5.32× (5.12–5.52) | 0.13× (0.12–0.13) | 0.13× (0.13–0.14) | 0.20× (0.20–0.21) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 89 MB | 0.34× (0.33–0.36) | 0.51× (0.43–0.61) | 0.61× (0.59–0.64) | 0.53× (0.51–0.56) | 3.54× (3.43–3.66) | 0.08× (0.08–0.09) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 33 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.81–0.81) | 4.29× (4.29–4.29) | 0.26× (0.25–0.26) | 0.27× (0.27–0.28) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.21× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.84–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.76–1.77) | 0.84× (0.83–0.84) | 4.38× (4.37–4.39) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.92) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.51) | 0.44× (0.44–0.44) | 1.70× (1.69–1.70) | 0.66× (0.65–0.66) | 8.00× (7.98–8.02) | 0.65× (0.64–0.65) | 0.68× (0.67–0.69) | 0.73× (0.73–0.74) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.70× (0.65–0.74) | 0.67× (0.61–0.74) | 1.60× (1.54–1.66) | 0.81× (0.77–0.84) | 5.80× (5.57–6.03) | 0.47× (0.45–0.48) | 0.49× (0.47–0.51) | 0.53× (0.51–0.55) | 0.27× (0.26–0.28) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 42 MB | 0.60× (0.57–0.63) | 0.64× (0.59–0.70) | 1.10× (1.04–1.16) | 0.68× (0.65–0.71) | 3.19× (3.03–3.36) | 0.25× (0.23–0.26) | 0.26× (0.25–0.27) | 0.28× (0.27–0.30) | 0.14× (0.13–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.17× (1.17–1.17) | 0.73× (0.73–0.73) | 5.17× (5.16–5.18) | 0.32× (0.32–0.33) | 0.35× (0.35–0.35) | 0.52× (0.52–0.53) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 40 MB | 0.49× (0.48–0.51) | 0.40× (0.37–0.43) | 0.88× (0.86–0.91) | 0.69× (0.65–0.73) | 3.08× (2.99–3.17) | 0.19× (0.18–0.20) | 0.21× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.63× (0.61–0.65) | 0.45× (0.44–0.46) | 0.85× (0.84–0.87) | 0.81× (0.80–0.83) | 2.48× (2.44–2.52) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.82× (0.82–0.83) | 5.18× (5.17–5.19) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.51× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.48× (0.47–0.50) | 0.38× (0.35–0.41) | 0.86× (0.83–0.89) | 0.72× (0.68–0.77) | 2.96× (2.86–3.06) | 0.18× (0.18–0.19) | 0.19× (0.19–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.61× (0.59–0.63) | 0.47× (0.44–0.49) | 0.80× (0.79–0.81) | 0.81× (0.79–0.84) | 2.31× (2.27–2.34) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.22) | 0.11× (0.11–0.11) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.21–0.24) | 1.07× (1.07–1.07) | 0.76× (0.76–0.76) | 5.37× (5.36–5.38) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.58–1.60) | 0.77× (0.76–0.77) | 5.29× (5.26–5.31) | 0.35× (0.34–0.35) | 0.36× (0.36–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.77–0.78) | 5.32× (5.30–5.34) | 0.35× (0.35–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 19 MB | 0.45× (0.45–0.46) | 0.38× (0.38–0.39) | 1.41× (1.40–1.41) | 0.66× (0.65–0.66) | 6.38× (6.36–6.39) | 0.39× (0.39–0.40) | 0.41× (0.41–0.42) | 0.62× (0.62–0.62) | 0.31× (0.31–0.31) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 34 MB | 0.49× (0.48–0.50) | 0.41× (0.37–0.45) | 1.15× (1.12–1.18) | 0.74× (0.71–0.77) | 3.61× (3.54–3.69) | 0.22× (0.22–0.23) | 0.23× (0.23–0.24) | 0.35× (0.34–0.36) | 0.18× (0.17–0.18) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 56 MB | 0.50× (0.49–0.52) | 0.48× (0.45–0.52) | 0.91× (0.89–0.93) | 0.75× (0.73–0.78) | 2.35× (2.30–2.39) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.22) | 0.11× (0.11–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 707 | 1.42 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 1,159 | 1.64 | 1.13× (1.09–1.16) | 82% | 1.05× (1.02–1.08) | 1.07× (1.03–1.11) |
| 4: all 4 CPUs, sharing SMT siblings | 1,342 | 2.51 | 1.74× (1.66–1.83) | 49% | 1.27× (1.20–1.34) | 1.71× (1.60–1.82) |

**Best:** mibz2 at 4 threads (1,342 MB/s); lbzip2 at 4 threads (311 MB/s); pbz2 at 4 threads (478 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.89, mibz2 4t 1.25, lbzip2 1t 0.07, lbzip2 2t 0.30, lbzip2 4t 0.39, pbz2 1t 0.00, pbz2 2t 0.49, pbz2 4t 0.52. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 13%, mibz2 4t 14%, lbzip2 2t 4%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 5%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.40 | 0.01 | 0.07 | 0.156 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (4311 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 36.4s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.98 | 14.5% |
| MTF + RLE2 | 1.43 | 21.1% |
| T-vector build | 1.25 | 18.3% |
| walk (the one each build uses) | 2.72 | 40.1% |
| of which: segment assembly | 0.11 | 1.6% |
| RLE1 | 0.15 | 2.2% |
| CRC | 0.24 | 3.6% |
| **total** | 6.79 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.288 | 0.691 | (in its total) | 2.40× (2.35–2.46) |  |
| walk | 0.607 | 1.538 | (in its total) | 2.56× (2.49–2.64) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.026 | 0.029 | (in its total) | 1.13× (1.08–1.18) |  |
| inverse BWT, total | 0.922 | 2.259 | 1.274 | 2.47× (2.41–2.54) | 1.38× (1.36–1.41) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 44m00s ± 5m05s |
| 4 threads | 22m36s ± 3m20s |
| 4 threads, the sample's overlap gap as a fixed cost | 14m09s ± 2m09s |
| the scanner alone (its serial part) | 8m26s ± 26.3s |
| the workers alone, over 4 | 14m09s ± 2m09s |

At 4 threads the workers bound it. Per byte: the scanner 4.9 ns a byte of bz2, the workers 1.85 ns a byte of output at 4 threads (CPU, summed over them), 1.17 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,568,747 | 13,092,730 | \`9b1907f0616f895e…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V45 96-Core Processor, 4 cores, reported clock 4310.7 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
