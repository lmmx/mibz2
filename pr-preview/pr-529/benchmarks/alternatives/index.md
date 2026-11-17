# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-529/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`33e7225`](<https://github.com/lmmx/mibz2/commit/33e7225ba6147fb480168dc54fe30727f0b93aea>) (2026-11-17).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/44055180739>)

AMD EPYC 7763 64-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.519 s | 404 | 6.8 | 2.90× (2.90–2.91) | 6.70× (6.67–6.72) | 6.83× (6.80–6.85) | 7.46× (7.44–7.48) | 6.69× (6.63–6.75) | 7.34× (7.32–7.36) | 6.84× (6.79–6.89) | 2.15× (2.15–2.16) | 2.49× (2.47–2.51) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.297 s | 706 | 7.4 | 2.84× (2.72–2.97) | 6.23× (5.91–6.56) | 6.88× (6.61–7.16) | 6.90× (6.66–7.15) | 6.15× (5.95–6.37) | 12.66× (12.21–13.13) | 11.80× (11.39–12.23) | 3.71× (3.59–3.85) | 4.30× (4.14–4.46) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.239 s | 876 | 11.4 | 2.90× (2.87–2.93) | 4.45× (4.37–4.54) | 5.53× (5.42–5.63) | 5.07× (4.99–5.15) | 5.03× (4.99–5.07) | 15.93× (15.83–16.03) | 14.85× (14.70–15.00) | 4.67× (4.64–4.70) | 5.40× (5.34–5.46) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.019 s | 1,028 | 2.9 | 0.62× (0.61–0.62) | 3.72× (3.63–3.80) | 4.75× (4.58–4.93) | 3.68× (3.64–3.73) | 6.36× (6.30–6.42) | 3.11× (3.07–3.15) | 2.77× (2.74–2.81) | 1.02× (1.01–1.03) | 1.58× (1.56–1.60) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 968 | 3.1 | 0.57× (0.55–0.58) | 3.01× (2.95–3.08) | 6.00× (5.86–6.15) | 3.57× (3.50–3.64) | 6.07× (5.96–6.17) | 2.97× (2.90–3.03) | 2.64× (2.59–2.70) | 0.97× (0.94–1.00) | 1.51× (1.47–1.54) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 976 | 3.1 | 0.57× (0.56–0.58) | 3.02× (2.98–3.06) | 5.95× (5.87–6.03) | 3.54× (3.50–3.59) | 6.06× (5.97–6.16) | 2.96× (2.93–3.00) | 2.64× (2.61–2.68) | 0.97× (0.95–0.99) | 1.50× (1.48–1.52) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.033 s | 598 | 4.8 | 2.68× (2.65–2.71) | 3.11× (3.09–3.13) | 4.09× (3.90–4.27) | 3.89× (3.88–3.91) | 4.19× (4.17–4.21) | 2.66× (2.57–2.74) | 2.15× (2.14–2.16) | 2.10× (2.09–2.12) | 3.06× (2.98–3.15) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.022 s | 907 | 6.0 | 2.20× (2.05–2.36) | 2.85× (2.63–3.10) | 4.00× (3.67–4.35) | 3.45× (3.22–3.69) | 4.73× (4.47–5.00) | 3.91× (3.72–4.11) | 3.16× (2.98–3.35) | 3.10× (2.92–3.28) | 4.51× (4.25–4.78) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 s | 778 | 11.7 | 1.91× (1.85–1.97) | 1.96× (1.82–2.12) | 2.91× (2.82–3.01) | 2.24× (2.16–2.31) | 3.90× (3.79–4.02) | 3.50× (3.34–3.66) | 2.83× (2.75–2.90) | 2.77× (2.70–2.84) | 4.03× (3.94–4.13) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.129 s | 62 | 44.4 | 1.38× (1.37–1.39) | 2.76× (2.75–2.78) | 2.44× (2.43–2.44) | 2.51× (2.50–2.53) | 2.90× (2.89–2.91) | 3.22× (3.19–3.26) | 3.24× (3.23–3.26) | 1.24× (1.23–1.24) | 1.42× (1.42–1.42) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.078 s | 102 | 49.7 | 1.28× (1.22–1.33) | 2.51× (2.36–2.66) | 2.37× (2.24–2.50) | 3.09× (2.93–3.26) | 2.92× (2.79–3.06) | 5.18× (4.94–5.44) | 5.21× (4.98–5.46) | 1.99× (1.90–2.08) | 2.28× (2.18–2.39) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.067 s | 120 | 69.0 | 1.36× (1.34–1.37) | 2.24× (2.18–2.31) | 2.04× (1.97–2.11) | 2.56× (2.48–2.63) | 2.83× (2.75–2.92) | 6.17× (5.96–6.38) | 6.20× (6.02–6.39) | 2.36× (2.29–2.44) | 2.71× (2.63–2.80) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.142 s | 68 | 40.9 | 1.50× (1.49–1.50) | 3.09× (3.06–3.12) | 2.59× (2.55–2.63) | 2.68× (2.67–2.69) | 3.13× (3.12–3.15) | 3.71× (3.70–3.72) | 3.99× (3.98–4.00) | 1.40× (1.40–1.41) | 1.71× (1.71–1.71) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.087 s | 111 | 46.6 | 1.37× (1.35–1.39) | 2.81× (2.75–2.86) | 2.51× (2.46–2.57) | 3.43× (3.37–3.50) | 3.09× (3.05–3.13) | 6.09× (6.01–6.16) | 6.55× (6.46–6.63) | 2.30× (2.27–2.33) | 2.80× (2.77–2.84) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.075 s | 127 | 64.5 | 1.43× (1.39–1.47) | 2.32× (2.25–2.39) | 2.23× (2.19–2.27) | 2.99× (2.91–3.07) | 2.78× (2.74–2.82) | 7.00× (6.88–7.11) | 7.53× (7.41–7.65) | 2.64× (2.60–2.69) | 3.22× (3.17–3.27) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.013 s | 778 | 4.0 | 0.68× (0.67–0.70) | 3.17× (3.13–3.21) | 4.67× (4.61–4.72) | 3.22× (3.18–3.26) | 6.96× (6.87–7.05) | 2.64× (2.61–2.68) | 2.43× (2.39–2.46) | 1.00× (0.99–1.01) | 1.60× (1.58–1.62) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.014 s | 712 | 4.4 | 0.56× (0.55–0.57) | 2.92× (2.87–2.97) | 5.03× (4.95–5.11) | 2.99× (2.93–3.05) | 6.46× (6.34–6.57) | 2.43× (2.39–2.47) | 2.23× (2.20–2.26) | 0.92× (0.90–0.93) | 1.47× (1.44–1.49) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.014 s | 704 | 4.4 | 0.57× (0.56–0.57) | 2.92× (2.90–2.94) | 5.02× (4.98–5.06) | 3.01× (2.99–3.03) | 6.42× (6.35–6.50) | 2.42× (2.40–2.44) | 2.22× (2.21–2.24) | 0.92× (0.90–0.93) | 1.46× (1.45–1.48) | yes |
| `text` (2.43 MB → 12.4 MB, 5.1×) | 1 | 0.065 s | 189 | 14.8 | 1.98× (1.97–2.00) | 4.20× (4.17–4.23) | 4.47× (4.45–4.50) | 4.54× (4.51–4.57) | 4.77× (4.71–4.82) | 5.33× (5.29–5.37) | 4.99× (4.95–5.03) | 1.51× (1.50–1.52) | 1.86× (1.85–1.87) | yes |
| `text` (2.43 MB → 12.4 MB, 5.1×) | 2 | 0.047 s | 264 | 19.0 | 1.51× (1.47–1.54) | 3.28× (3.11–3.46) | 3.67× (3.52–3.83) | 4.26× (4.03–4.50) | 4.10× (4.00–4.19) | 7.37× (7.09–7.66) | 6.89× (6.63–7.17) | 2.08× (2.00–2.17) | 2.57× (2.47–2.67) | yes |
| `text` (2.43 MB → 12.4 MB, 5.1×) | 4 | 0.041 s | 305 | 29.0 | 1.51× (1.46–1.56) | 2.34× (2.25–2.42) | 2.94× (2.81–3.08) | 3.44× (3.29–3.59) | 3.63× (3.52–3.74) | 8.49× (8.24–8.76) | 7.95× (7.70–8.20) | 2.40× (2.33–2.47) | 2.96× (2.88–3.05) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.89× (2.89–2.90) | 6.66× (6.64–6.69) | 6.80× (6.78–6.82) | 7.44× (7.42–7.46) | 6.67× (6.61–6.73) | 7.32× (7.30–7.33) | 6.82× (6.77–6.87) | 2.15× (2.14–2.15) | 2.47× (2.45–2.49) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.75× (2.74–2.76) | 6.24× (6.19–6.29) | 6.49× (6.43–6.56) | 7.18× (7.09–7.28) | 6.14× (6.12–6.17) | 6.71× (6.68–6.73) | 6.25× (6.20–6.30) | 1.97× (1.96–1.98) | 2.26× (2.24–2.29) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 2.75× (2.70–2.81) | 4.68× (4.64–4.72) | 5.03× (5.00–5.06) | 5.34× (5.32–5.37) | 4.69× (4.66–4.72) | 4.33× (4.31–4.36) | 4.04× (4.00–4.08) | 1.27× (1.26–1.28) | 1.46× (1.45–1.48) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.65× (0.64–0.66) | 3.42× (3.38–3.46) | 4.27× (4.16–4.39) | 3.47× (3.43–3.51) | 5.93× (5.87–5.98) | 2.95× (2.91–2.98) | 2.63× (2.60–2.67) | 1.02× (1.01–1.03) | 1.10× (1.07–1.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.61× (0.60–0.63) | 3.09× (3.03–3.16) | 5.38× (5.28–5.47) | 3.33× (3.26–3.39) | 5.56× (5.47–5.65) | 2.76× (2.70–2.82) | 2.46× (2.41–2.51) | 0.95× (0.92–0.98) | 1.03× (1.00–1.06) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.60× (0.59–0.61) | 3.43× (3.36–3.50) | 5.43× (5.35–5.50) | 3.35× (3.30–3.39) | 5.61× (5.51–5.70) | 2.79× (2.75–2.83) | 2.49× (2.46–2.52) | 0.96× (0.94–0.98) | 1.04× (1.01–1.07) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.60× (2.57–2.63) | 2.86× (2.84–2.88) | 3.78× (3.61–3.97) | 3.75× (3.73–3.77) | 4.02× (4.00–4.05) | 2.57× (2.49–2.66) | 2.09× (2.07–2.10) | 2.05× (2.03–2.06) | 2.76× (2.71–2.81) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.24× (2.21–2.27) | 2.49× (2.46–2.53) | 3.05× (3.01–3.09) | 3.17× (3.14–3.20) | 3.37× (3.34–3.39) | 2.06× (1.99–2.13) | 1.67× (1.65–1.68) | 1.64× (1.62–1.65) | 2.21× (2.17–2.24) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.03× (1.95–2.11) | 1.74× (1.70–1.79) | 2.20× (2.15–2.24) | 2.11× (2.04–2.19) | 2.10× (2.05–2.15) | 1.05× (1.01–1.10) | 0.85× (0.83–0.87) | 0.84× (0.82–0.85) | 1.13× (1.10–1.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.37× (1.37–1.38) | 2.68× (2.68–2.69) | 2.40× (2.39–2.40) | 2.49× (2.48–2.50) | 2.88× (2.87–2.88) | 3.19× (3.16–3.23) | 3.21× (3.20–3.22) | 1.23× (1.23–1.24) | 1.38× (1.37–1.38) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.29× (1.28–1.30) | 2.51× (2.49–2.53) | 2.21× (2.20–2.22) | 2.97× (2.93–3.01) | 2.60× (2.59–2.62) | 2.85× (2.81–2.89) | 2.86× (2.85–2.88) | 1.10× (1.09–1.11) | 1.23× (1.22–1.24) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.46× (1.41–1.52) | 2.24× (2.18–2.31) | 1.85× (1.82–1.88) | 2.55× (2.51–2.59) | 2.41× (2.37–2.44) | 2.06× (2.03–2.09) | 2.07× (2.04–2.10) | 0.79× (0.79–0.80) | 0.89× (0.88–0.90) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.49× (1.49–1.50) | 3.05× (3.01–3.08) | 2.55× (2.52–2.57) | 2.66× (2.65–2.67) | 3.11× (3.09–3.12) | 3.68× (3.67–3.69) | 3.96× (3.95–3.97) | 1.40× (1.39–1.40) | 1.66× (1.65–1.67) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.38× (1.36–1.39) | 2.80× (2.77–2.84) | 2.31× (2.29–2.34) | 3.23× (3.20–3.27) | 2.75× (2.72–2.78) | 3.23× (3.20–3.26) | 3.47× (3.44–3.50) | 1.22× (1.21–1.24) | 1.46× (1.44–1.47) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.66× (1.62–1.70) | 2.61× (2.56–2.67) | 1.98× (1.94–2.02) | 2.80× (2.74–2.87) | 2.57× (2.51–2.64) | 2.33× (2.29–2.38) | 2.51× (2.46–2.57) | 0.89× (0.87–0.90) | 1.05× (1.03–1.07) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.72× (0.71–0.74) | 2.83× (2.79–2.87) | 3.69× (3.63–3.74) | 2.96× (2.92–3.01) | 6.26× (6.17–6.35) | 2.46× (2.42–2.50) | 2.26× (2.23–2.29) | 1.00× (0.98–1.01) | 1.04× (0.99–1.10) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.62× (0.61–0.63) | 2.54× (2.50–2.57) | 4.26× (4.20–4.32) | 2.77× (2.72–2.82) | 5.72× (5.64–5.80) | 2.21× (2.19–2.24) | 2.04× (2.01–2.06) | 0.90× (0.88–0.91) | 0.94× (0.89–0.99) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.61× (0.60–0.62) | 2.77× (2.74–2.81) | 4.33× (4.28–4.38) | 2.84× (2.82–2.85) | 5.79× (5.71–5.87) | 2.25× (2.23–2.26) | 2.06× (2.05–2.08) | 0.91× (0.90–0.92) | 0.95× (0.91–1.00) |
| `text` (2.43 MB → 12.4 MB, 5.1×) | 1 | 1.96× (1.95–1.97) | 4.04× (4.00–4.07) | 4.34× (4.30–4.37) | 4.45× (4.42–4.48) | 4.67× (4.62–4.73) | 5.22× (5.18–5.26) | 4.89× (4.85–4.93) | 1.49× (1.48–1.51) | 1.78× (1.77–1.80) |
| `text` (2.43 MB → 12.4 MB, 5.1×) | 2 | 1.60× (1.58–1.63) | 3.23× (3.18–3.29) | 3.53× (3.47–3.59) | 3.94× (3.87–4.01) | 3.70× (3.64–3.77) | 4.04× (3.96–4.12) | 3.78× (3.71–3.85) | 1.16× (1.14–1.18) | 1.38× (1.36–1.40) |
| `text` (2.43 MB → 12.4 MB, 5.1×) | 4 | 1.72× (1.64–1.80) | 2.62× (2.57–2.67) | 2.87× (2.84–2.91) | 3.08× (3.03–3.14) | 3.01× (2.99–3.04) | 2.67× (2.63–2.71) | 2.50× (2.47–2.53) | 0.77× (0.76–0.77) | 0.91× (0.90–0.92) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.30× (0.30–0.30) | 0.28× (0.27–0.30) | 0.97× (0.97–0.98) | 0.78× (0.77–0.78) | 11.01× (10.98–11.04) | 0.26× (0.26–0.26) | 0.28× (0.27–0.29) | 0.42× (0.42–0.42) | 0.21× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 56 MB | 0.34× (0.33–0.35) | 0.49× (0.41–0.59) | 0.71× (0.68–0.74) | 0.59× (0.57–0.61) | 5.61× (5.41–5.81) | 0.13× (0.13–0.14) | 0.14× (0.13–0.15) | 0.21× (0.21–0.22) | 0.11× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 89 MB | 0.34× (0.33–0.35) | 0.64× (0.59–0.70) | 0.61× (0.59–0.63) | 0.53× (0.52–0.55) | 3.55× (3.44–3.66) | 0.08× (0.08–0.09) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.07–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.82× (0.82–0.82) | 0.81× (0.81–0.81) | 4.29× (4.28–4.29) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.19–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.27–0.28) | 0.92× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.39× (4.39–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.71× (1.70–1.71) | 0.66× (0.66–0.67) | 8.03× (8.01–8.05) | 0.65× (0.64–0.65) | 0.69× (0.68–0.69) | 0.74× (0.74–0.75) | 0.37× (0.37–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.67× (0.66–0.68) | 0.67× (0.63–0.71) | 1.62× (1.60–1.65) | 0.81× (0.80–0.83) | 5.87× (5.78–5.96) | 0.47× (0.46–0.48) | 0.50× (0.49–0.51) | 0.54× (0.53–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 40 MB | 0.65× (0.62–0.68) | 0.68× (0.63–0.75) | 1.15× (1.11–1.19) | 0.71× (0.68–0.73) | 3.37× (3.26–3.48) | 0.26× (0.25–0.27) | 0.28× (0.27–0.29) | 0.30× (0.29–0.31) | 0.15× (0.14–0.15) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.32× (0.32–0.32) | 1.17× (1.17–1.18) | 0.73× (0.73–0.74) | 5.18× (5.16–5.21) | 0.33× (0.33–0.33) | 0.35× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 37 MB | 0.53× (0.51–0.54) | 0.43× (0.41–0.45) | 0.94× (0.92–0.97) | 0.76× (0.74–0.78) | 3.27× (3.18–3.35) | 0.20× (0.20–0.21) | 0.22× (0.21–0.22) | 0.32× (0.32–0.33) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 48 MB | 0.65× (0.63–0.67) | 0.47× (0.45–0.48) | 0.88× (0.87–0.90) | 0.83× (0.81–0.85) | 2.57× (2.53–2.61) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.25× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.17× (1.17–1.18) | 0.82× (0.82–0.82) | 5.17× (5.16–5.18) | 0.32× (0.32–0.32) | 0.34× (0.33–0.35) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.47× (0.45–0.49) | 0.39× (0.37–0.40) | 0.84× (0.81–0.88) | 0.69× (0.67–0.72) | 2.89× (2.79–3.00) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 53 MB | 0.61× (0.60–0.63) | 0.50× (0.48–0.52) | 0.81× (0.79–0.83) | 0.83× (0.80–0.87) | 2.34× (2.29–2.39) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.23× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.20× (0.20–0.21) | 1.08× (1.07–1.08) | 0.76× (0.76–0.77) | 5.37× (5.36–5.38) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.91× (0.91–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.77–0.77) | 5.30× (5.29–5.32) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.89–0.90) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.23–0.24) | 0.20× (0.20–0.20) | 1.61× (1.60–1.61) | 0.77× (0.77–0.78) | 5.32× (5.31–5.32) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.89–0.91) | 0.13× (0.13–0.13) |
| `text` (2.43 MB → 12.4 MB, 5.1×) | 1 | 19 MB | 0.45× (0.45–0.46) | 0.39× (0.39–0.39) | 1.42× (1.41–1.42) | 0.66× (0.65–0.66) | 6.38× (6.37–6.39) | 0.40× (0.39–0.40) | 0.42× (0.41–0.42) | 0.63× (0.62–0.63) | 0.31× (0.31–0.31) |
| `text` (2.43 MB → 12.4 MB, 5.1×) | 2 | 40 MB | 0.42× (0.41–0.42) | 0.37× (0.35–0.40) | 0.99× (0.98–1.00) | 0.64× (0.62–0.65) | 3.08× (3.04–3.12) | 0.19× (0.19–0.19) | 0.20× (0.20–0.20) | 0.30× (0.30–0.31) | 0.15× (0.15–0.15) |
| `text` (2.43 MB → 12.4 MB, 5.1×) | 4 | 58 MB | 0.49× (0.48–0.50) | 0.47× (0.43–0.52) | 0.89× (0.87–0.90) | 0.70× (0.67–0.74) | 2.26× (2.21–2.30) | 0.13× (0.13–0.14) | 0.14× (0.14–0.14) | 0.21× (0.21–0.21) | 0.10× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 404 | 2.49 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 706 | 2.71 | 1.09× (1.09–1.10) | 86% | 1.02× (1.01–1.03) | 1.04× (1.03–1.04) |
| 4: all 4 CPUs, sharing SMT siblings | 876 | 4.18 | 1.69× (1.68–1.70) | 54% | 1.19× (1.18–1.19) | 1.61× (1.57–1.64) |

**Best:** mibz2 at 4 threads (876 MB/s); lbzip2 at 4 threads (199 MB/s); pbz2 at 4 threads (304 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.80, mibz2 4t 1.19, lbzip2 1t 0.09, lbzip2 2t 0.38, lbzip2 4t 0.55, pbz2 1t 0.00, pbz2 2t 0.32, pbz2 4t 0.44. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 9%, mibz2 4t 10%, lbzip2 2t 5%, lbzip2 4t 6%, pbz2 2t 3%, pbz2 4t 3%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.09 | 0.68 | 0.02 | 0.09 | 0.239 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (2723 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m03s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.95 | 11.9% |
| MTF + RLE2 | 1.79 | 22.4% |
| T-vector build | 1.27 | 15.9% |
| walk (the one each build uses) | 3.51 | 44.1% |
| of which: segment assembly | 0.13 | 1.6% |
| RLE1 | 0.23 | 2.9% |
| CRC | 0.21 | 2.7% |
| **total** | 7.96 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.491 | 1.300 | (in its total) | 2.64× (2.64–2.65) |  |
| walk | 1.240 | 2.618 | (in its total) | 2.12× (2.11–2.12) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.047 | 0.054 | (in its total) | 1.15× (1.14–1.15) |  |
| inverse BWT, total | 1.778 | 3.972 | 3.226 | 2.24× (2.23–2.24) | 1.81× (1.81–1.82) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h15m ± 8m42s |
| 4 threads | 34m48s ± 4m53s |
| 4 threads, the sample's overlap gap as a fixed cost | 24m46s ± 3m29s |
| the scanner alone (its serial part) | 13m23s ± 26.3s |
| the workers alone, over 4 | 24m46s ± 3m29s |

At 4 threads the workers bound it. Per byte: the scanner 7.8 ns a byte of bz2, the workers 3.25 ns a byte of output at 4 threads (CPU, summed over them), 2.04 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,434,784 | 12,361,720 | \`c83a65c1574133bd…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 7763 64-Core Processor, 4 cores, reported clock 2723.1 MHz (cycles a byte use it), flags avx2, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
