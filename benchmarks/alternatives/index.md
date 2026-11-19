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

The latest record: [`f8061b3`](<https://github.com/lmmx/mibz2/commit/f8061b3c6e13047a787bc2169277389ed7b1b897>) (2026-11-19).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44435769100>)

INTEL(R) XEON(R) PLATINUM 8573C, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.438 s | 478 | 7.5 | 3.27× (3.25–3.29) | 7.23× (7.19–7.28) | 7.66× (7.61–7.72) | 8.59× (8.54–8.65) | 7.76× (7.70–7.82) | 8.26× (8.19–8.33) | 7.92× (7.87–7.96) | 2.54× (2.53–2.56) | 2.31× (2.29–2.34) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.261 s | 804 | 8.2 | 3.12× (2.98–3.26) | 6.60× (6.39–6.81) | 7.84× (7.61–8.07) | 7.78× (7.45–8.13) | 7.00× (6.70–7.31) | 14.03× (13.46–14.62) | 13.44× (12.94–13.96) | 4.32× (4.14–4.50) | 3.92× (3.80–4.05) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.205 s | 1,022 | 12.8 | 3.36× (3.30–3.42) | 5.94× (5.85–6.03) | 6.73× (6.59–6.87) | 6.65× (6.57–6.73) | 6.56× (6.47–6.65) | 17.61× (17.37–17.86) | 16.88× (16.60–17.17) | 5.42× (5.34–5.51) | 4.93× (4.83–5.03) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.019 s | 1,047 | 3.7 | 0.55× (0.54–0.56) | 3.19× (3.13–3.25) | 4.30× (4.18–4.43) | 3.59× (3.53–3.66) | 6.03× (5.92–6.14) | 2.62× (2.57–2.67) | 2.61× (2.56–2.67) | 1.16× (1.05–1.28) | 1.08× (1.06–1.10) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.019 s | 1,044 | 3.8 | 0.52× (0.51–0.52) | 2.67× (2.64–2.70) | 5.70× (5.56–5.85) | 3.59× (3.55–3.63) | 5.96× (5.87–6.06) | 2.62× (2.59–2.65) | 2.62× (2.58–2.65) | 1.16× (1.06–1.27) | 1.08× (1.06–1.09) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,036 | 3.8 | 0.52× (0.51–0.53) | 2.80× (2.65–2.97) | 5.82× (5.72–5.91) | 3.61× (3.53–3.68) | 6.03× (5.78–6.29) | 2.60× (2.56–2.64) | 2.59× (2.55–2.64) | 1.15× (1.05–1.25) | 1.07× (1.05–1.08) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.034 s | 577 | 6.5 | 2.80× (2.77–2.82) | 3.30× (3.28–3.32) | 3.91× (3.88–3.93) | 4.29× (4.23–4.35) | 4.76× (4.71–4.81) | 3.55× (3.51–3.59) | 2.85× (2.83–2.87) | 2.11× (2.10–2.13) | 2.10× (2.09–2.11) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.021 s | 933 | 7.6 | 2.41× (2.28–2.55) | 3.03× (2.78–3.31) | 4.36× (4.09–4.65) | 3.99× (3.68–4.33) | 5.34× (5.10–5.59) | 5.61× (5.37–5.86) | 4.51× (4.32–4.70) | 3.34× (3.22–3.47) | 3.32× (3.18–3.46) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.021 s | 912 | 13.9 | 2.03× (1.93–2.13) | 1.99× (1.86–2.13) | 2.98× (2.80–3.18) | 2.45× (2.27–2.63) | 4.25× (4.10–4.41) | 5.45× (5.23–5.68) | 4.38× (4.19–4.57) | 3.25× (3.12–3.38) | 3.22× (3.09–3.36) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.129 s | 62 | 59.0 | 1.33× (1.31–1.35) | 2.74× (2.72–2.76) | 2.48× (2.42–2.53) | 2.53× (2.50–2.57) | 2.83× (2.80–2.86) | 3.13× (3.09–3.17) | 3.24× (3.21–3.27) | 1.28× (1.24–1.32) | 1.25× (1.23–1.26) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.077 s | 104 | 64.7 | 1.34× (1.19–1.49) | 2.65× (2.55–2.75) | 2.52× (2.48–2.56) | 3.24× (3.19–3.29) | 2.98× (2.93–3.03) | 5.29× (5.21–5.37) | 5.48× (5.39–5.57) | 2.17× (2.10–2.24) | 2.11× (2.07–2.14) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.066 s | 122 | 87.0 | 1.36× (1.31–1.41) | 2.39× (2.31–2.47) | 2.42× (2.31–2.54) | 2.92× (2.84–3.00) | 2.92× (2.84–3.01) | 6.26× (6.10–6.43) | 6.48× (6.29–6.68) | 2.57× (2.42–2.72) | 2.49× (2.43–2.56) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.143 s | 67 | 54.4 | 1.38× (1.37–1.39) | 3.13× (3.11–3.16) | 2.64× (2.61–2.67) | 2.74× (2.72–2.77) | 3.10× (3.08–3.12) | 3.84× (3.81–3.86) | 4.11× (4.04–4.17) | 1.48× (1.47–1.49) | 1.49× (1.47–1.50) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.085 s | 113 | 59.7 | 1.27× (1.23–1.31) | 3.08× (2.75–3.45) | 2.60× (2.54–2.65) | 3.57× (3.47–3.67) | 3.07× (2.99–3.15) | 6.40× (6.24–6.57) | 6.85× (6.72–6.99) | 2.47× (2.40–2.53) | 2.48× (2.40–2.56) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.071 s | 135 | 84.7 | 1.43× (1.40–1.47) | 2.71× (2.61–2.82) | 2.47× (2.37–2.57) | 3.37× (3.29–3.45) | 3.03× (2.95–3.11) | 7.71× (7.54–7.88) | 8.25× (8.04–8.47) | 2.97× (2.91–3.03) | 2.98× (2.91–3.06) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 790 | 5.1 | 0.59× (0.58–0.60) | 3.22× (3.14–3.29) | 4.04× (3.94–4.14) | 3.07× (3.00–3.15) | 6.55× (6.39–6.72) | 2.27× (2.21–2.32) | 2.29× (2.24–2.35) | 1.03× (1.01–1.05) | 0.84× (0.81–0.86) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 775 | 5.3 | 0.52× (0.51–0.52) | 2.47× (2.33–2.62) | 4.86× (4.72–5.01) | 3.09× (2.99–3.20) | 6.45× (6.36–6.54) | 2.24× (2.21–2.27) | 2.26× (2.22–2.30) | 1.02× (0.99–1.04) | 0.82× (0.81–0.84) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 769 | 5.2 | 0.52× (0.51–0.53) | 2.46× (2.30–2.63) | 4.77× (4.72–4.82) | 3.02× (2.99–3.05) | 6.35× (6.23–6.47) | 2.22× (2.19–2.24) | 2.24× (2.22–2.27) | 1.01× (0.99–1.03) | 0.82× (0.81–0.83) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 0.065 s | 200 | 18.4 | 1.87× (1.82–1.91) | 4.09× (4.00–4.19) | 4.51× (4.47–4.55) | 4.72× (4.63–4.80) | 4.82× (4.78–4.86) | 5.03× (4.99–5.06) | 4.93× (4.90–4.96) | 1.53× (1.52–1.54) | 1.55× (1.55–1.56) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 0.042 s | 310 | 21.7 | 1.58× (1.52–1.65) | 3.54× (3.32–3.78) | 4.48× (4.13–4.86) | 4.56× (4.35–4.77) | 4.57× (4.37–4.78) | 7.67× (7.36–8.00) | 7.52× (7.22–7.84) | 2.34× (2.24–2.44) | 2.37× (2.27–2.47) | yes |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 0.037 s | 356 | 32.8 | 1.65× (1.61–1.70) | 3.00× (2.88–3.12) | 3.81× (3.69–3.94) | 3.94× (3.83–4.06) | 4.34× (4.27–4.42) | 8.93× (8.79–9.07) | 8.75× (8.64–8.87) | 2.72× (2.68–2.76) | 2.76× (2.72–2.80) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.26× (3.24–3.28) | 7.20× (7.15–7.24) | 7.63× (7.57–7.68) | 8.57× (8.51–8.63) | 7.74× (7.67–7.80) | 8.23× (8.16–8.31) | 7.89× (7.84–7.94) | 2.54× (2.52–2.55) | 2.29× (2.27–2.32) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 3.04× (3.02–3.07) | 6.90× (6.83–6.98) | 7.50× (7.43–7.58) | 8.33× (8.26–8.40) | 7.16× (7.11–7.21) | 7.57× (7.50–7.64) | 7.25× (7.22–7.29) | 2.33× (2.32–2.35) | 2.11× (2.09–2.13) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.29× (3.23–3.37) | 6.41× (6.29–6.52) | 6.56× (6.45–6.67) | 7.21× (7.09–7.34) | 6.47× (6.37–6.57) | 4.89× (4.80–4.98) | 4.68× (4.61–4.76) | 1.51× (1.48–1.53) | 1.36× (1.34–1.39) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.59× (0.57–0.60) | 2.70× (2.65–2.75) | 3.76× (3.66–3.86) | 3.40× (3.35–3.45) | 5.66× (5.55–5.76) | 2.50× (2.45–2.54) | 2.49× (2.44–2.54) | 1.15× (1.05–1.26) | 0.86× (0.84–0.88) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.56× (0.55–0.57) | 2.68× (2.66–2.70) | 4.95× (4.91–5.00) | 3.37× (3.34–3.40) | 5.52× (5.45–5.60) | 2.46× (2.44–2.49) | 2.46× (2.43–2.48) | 1.14× (1.04–1.24) | 0.85× (0.84–0.86) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.55× (0.54–0.56) | 3.14× (3.08–3.21) | 5.02× (4.92–5.13) | 3.44× (3.37–3.50) | 5.65× (5.42–5.88) | 2.48× (2.43–2.52) | 2.47× (2.43–2.51) | 1.14× (1.05–1.24) | 0.86× (0.84–0.87) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.72× (2.70–2.74) | 3.04× (3.01–3.07) | 3.56× (3.53–3.58) | 4.15× (4.09–4.21) | 4.59× (4.54–4.64) | 3.44× (3.40–3.48) | 2.77× (2.75–2.79) | 2.06× (2.05–2.08) | 1.93× (1.92–1.95) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.46× (2.39–2.54) | 2.74× (2.68–2.80) | 3.30× (3.21–3.39) | 3.62× (3.51–3.72) | 4.00× (3.90–4.10) | 2.90× (2.84–2.96) | 2.34× (2.29–2.39) | 1.74× (1.71–1.77) | 1.63× (1.60–1.67) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.21× (2.12–2.31) | 1.91× (1.79–2.04) | 2.32× (2.22–2.42) | 2.26× (2.17–2.36) | 2.40× (2.31–2.50) | 1.61× (1.55–1.68) | 1.30× (1.25–1.35) | 0.97× (0.93–1.01) | 0.91× (0.87–0.94) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.33× (1.31–1.35) | 2.69× (2.66–2.71) | 2.41× (2.36–2.46) | 2.52× (2.48–2.55) | 2.81× (2.78–2.84) | 3.10× (3.07–3.14) | 3.21× (3.18–3.25) | 1.28× (1.24–1.32) | 1.23× (1.21–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.26× (1.24–1.27) | 2.58× (2.55–2.62) | 2.28× (2.25–2.31) | 3.03× (3.01–3.06) | 2.62× (2.59–2.65) | 2.83× (2.80–2.86) | 2.93× (2.90–2.96) | 1.17× (1.13–1.21) | 1.12× (1.11–1.13) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.52× (1.48–1.57) | 2.65× (2.59–2.71) | 2.26× (2.20–2.33) | 2.94× (2.89–2.98) | 2.60× (2.54–2.65) | 2.09× (2.05–2.13) | 2.16× (2.12–2.21) | 0.86× (0.82–0.90) | 0.82× (0.81–0.84) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.38× (1.36–1.39) | 3.09× (3.07–3.11) | 2.58× (2.56–2.60) | 2.72× (2.70–2.75) | 3.08× (3.06–3.10) | 3.80× (3.78–3.83) | 4.07× (4.00–4.14) | 1.47× (1.46–1.48) | 1.46× (1.45–1.47) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.30× (1.26–1.34) | 2.92× (2.85–2.98) | 2.44× (2.39–2.49) | 3.38× (3.29–3.46) | 2.84× (2.77–2.91) | 3.43× (3.35–3.50) | 3.67× (3.61–3.72) | 1.33× (1.29–1.36) | 1.32× (1.29–1.35) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.59× (1.56–1.62) | 3.01× (2.96–3.06) | 2.22× (2.19–2.26) | 3.16× (3.08–3.23) | 2.75× (2.69–2.81) | 2.45× (2.41–2.50) | 2.63× (2.58–2.67) | 0.95× (0.93–0.97) | 0.94× (0.93–0.96) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.64× (0.62–0.65) | 2.33× (2.28–2.38) | 3.30× (3.20–3.40) | 2.84× (2.77–2.91) | 5.93× (5.79–6.08) | 2.12× (2.07–2.17) | 2.15× (2.10–2.20) | 1.02× (1.00–1.05) | 0.80× (0.78–0.82) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.58× (0.57–0.59) | 2.24× (2.18–2.30) | 4.15× (4.07–4.24) | 2.87× (2.78–2.97) | 5.76× (5.68–5.85) | 2.05× (2.03–2.08) | 2.08× (2.05–2.11) | 0.99× (0.97–1.02) | 0.77× (0.75–0.80) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.57× (0.56–0.58) | 2.61× (2.55–2.66) | 4.15× (4.05–4.24) | 2.88× (2.84–2.91) | 5.80× (5.69–5.91) | 2.08× (2.06–2.10) | 2.11× (2.07–2.14) | 1.00× (0.98–1.03) | 0.78× (0.76–0.81) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 1.85× (1.80–1.89) | 3.95× (3.86–4.04) | 4.35× (4.32–4.39) | 4.63× (4.54–4.71) | 4.73× (4.69–4.77) | 4.93× (4.90–4.96) | 4.83× (4.80–4.87) | 1.52× (1.51–1.53) | 1.43× (1.41–1.45) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 1.61× (1.59–1.63) | 3.44× (3.39–3.50) | 4.09× (3.84–4.35) | 4.39× (4.34–4.44) | 4.06× (4.01–4.11) | 4.15× (4.11–4.20) | 4.07× (4.03–4.12) | 1.28× (1.26–1.30) | 1.20× (1.19–1.22) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 1.83× (1.80–1.87) | 3.14× (3.08–3.19) | 3.43× (3.37–3.48) | 3.82× (3.78–3.86) | 3.50× (3.45–3.56) | 2.76× (2.71–2.80) | 2.70× (2.66–2.75) | 0.85× (0.84–0.86) | 0.80× (0.78–0.81) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.30) | 0.26× (0.26–0.26) | 0.97× (0.97–0.97) | 0.78× (0.77–0.78) | 10.99× (10.97–11.01) | 0.26× (0.26–0.26) | 0.28× (0.28–0.29) | 0.42× (0.42–0.43) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 55 MB | 0.34× (0.33–0.36) | 0.52× (0.46–0.59) | 0.70× (0.68–0.73) | 0.60× (0.58–0.62) | 5.65× (5.44–5.86) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 85 MB | 0.36× (0.34–0.37) | 0.56× (0.50–0.64) | 0.63× (0.62–0.65) | 0.57× (0.55–0.59) | 3.71× (3.64–3.79) | 0.09× (0.09–0.09) | 0.09× (0.09–0.10) | 0.14× (0.14–0.14) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.81× (0.81–0.82) | 0.81× (0.81–0.81) | 4.29× (4.28–4.30) | 0.26× (0.26–0.26) | 0.27× (0.26–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.21) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.39× (4.38–4.39) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.71× (1.70–1.71) | 0.66× (0.66–0.67) | 8.05× (8.03–8.06) | 0.65× (0.65–0.66) | 0.68× (0.68–0.69) | 0.74× (0.73–0.74) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.68× (0.68–0.69) | 0.64× (0.59–0.70) | 1.62× (1.61–1.64) | 0.82× (0.81–0.83) | 5.92× (5.84–5.99) | 0.48× (0.47–0.49) | 0.50× (0.49–0.51) | 0.54× (0.53–0.55) | 0.27× (0.27–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 39 MB | 0.63× (0.61–0.66) | 0.68× (0.63–0.73) | 1.13× (1.08–1.17) | 0.70× (0.68–0.73) | 3.39× (3.26–3.53) | 0.26× (0.25–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.17× (1.17–1.17) | 0.73× (0.73–0.73) | 5.17× (5.16–5.18) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.27× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.52) | 0.40× (0.37–0.43) | 0.91× (0.89–0.93) | 0.70× (0.67–0.74) | 3.15× (3.07–3.24) | 0.20× (0.19–0.20) | 0.21× (0.20–0.22) | 0.31× (0.30–0.32) | 0.16× (0.16–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.62× (0.60–0.64) | 0.45× (0.44–0.46) | 0.85× (0.83–0.87) | 0.80× (0.77–0.82) | 2.48× (2.42–2.55) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.32) | 1.17× (1.17–1.18) | 0.82× (0.82–0.83) | 5.19× (5.16–5.21) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.51× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.49× (0.47–0.51) | 0.34× (0.31–0.37) | 0.87× (0.84–0.90) | 0.71× (0.68–0.74) | 2.98× (2.89–3.07) | 0.18× (0.18–0.19) | 0.20× (0.19–0.20) | 0.29× (0.29–0.30) | 0.15× (0.15–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.62) | 0.46× (0.44–0.47) | 0.81× (0.79–0.82) | 0.82× (0.79–0.85) | 2.33× (2.28–2.37) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.22× (0.21–0.24) | 1.07× (1.07–1.07) | 0.76× (0.76–0.77) | 5.37× (5.36–5.38) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.59× (1.59–1.60) | 0.77× (0.76–0.77) | 5.29× (5.27–5.32) | 0.34× (0.34–0.34) | 0.36× (0.36–0.37) | 0.89× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.25) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.77–0.78) | 5.31× (5.30–5.32) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.38–0.39) | 1.42× (1.42–1.43) | 0.67× (0.66–0.67) | 6.45× (6.44–6.46) | 0.40× (0.39–0.40) | 0.42× (0.42–0.43) | 0.63× (0.62–0.64) | 0.31× (0.31–0.31) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 2 | 37 MB | 0.45× (0.44–0.46) | 0.39× (0.35–0.43) | 1.05× (1.02–1.07) | 0.69× (0.67–0.71) | 3.35× (3.27–3.43) | 0.20× (0.20–0.21) | 0.22× (0.21–0.23) | 0.33× (0.32–0.34) | 0.16× (0.16–0.17) |
| `text` (2.57 MB → 13.1 MB, 5.1×) | 4 | 57 MB | 0.49× (0.47–0.50) | 0.44× (0.39–0.49) | 0.90× (0.87–0.93) | 0.72× (0.69–0.75) | 2.29× (2.24–2.34) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 478 | 2.10 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 804 | 2.28 | 1.09× (1.08–1.09) | 85% | 1.04× (1.03–1.06) | 1.02× (1.01–1.02) |
| 4: all 4 CPUs, sharing SMT siblings | 1,022 | 3.56 | 1.68× (1.65–1.72) | 53% | 1.50× (1.48–1.52) | 1.70× (1.68–1.73) |

**Best:** mibz2 at 4 threads (1,022 MB/s); lbzip2 at 4 threads (172 MB/s); pbz2 at 4 threads (304 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.89, mibz2 4t 1.21, lbzip2 1t 0.04, lbzip2 2t 0.28, lbzip2 4t 0.39, pbz2 1t 0.00, pbz2 2t 0.37, pbz2 4t 0.40. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 10%, mibz2 4t 10%, lbzip2 2t 3%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.09 | 0.56 | 0.02 | 0.07 | 0.205 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3600 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 59.4s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.16 | 11.0% |
| MTF + RLE2 | 1.68 | 16.0% |
| T-vector build | 1.59 | 15.2% |
| walk (the one each build uses) | 5.64 | 53.8% |
| of which: segment assembly | 0.29 | 2.7% |
| RLE1 | 0.26 | 2.5% |
| CRC | 0.14 | 1.3% |
| **total** | 10.47 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.439 | 1.141 | (in its total) | 2.58× (2.54–2.63) |  |
| walk | 1.495 | 2.958 | (in its total) | 1.96× (1.91–2.00) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.081 | 0.074 | (in its total) | 0.90× (0.88–0.92) |  |
| inverse BWT, total | 2.015 | 4.170 | 2.149 | 2.05× (2.01–2.09) | 1.06× (1.03–1.09) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h03m ± 7m13s |
| 4 threads | 29m57s ± 4m13s |
| 4 threads, the sample's overlap gap as a fixed cost | 20m24s ± 2m54s |
| the scanner alone (its serial part) | 12m22s ± 32.9s |
| the workers alone, over 4 | 20m24s ± 2m54s |

At 4 threads the workers bound it. Per byte: the scanner 7.2 ns a byte of bz2, the workers 2.67 ns a byte of output at 4 threads (CPU, summed over them), 1.69 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,568,340 | 13,095,663 | \`3c06b7d4c7dc0233…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: INTEL(R) XEON(R) PLATINUM 8573C, 4 cores, reported clock 3600.0 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
