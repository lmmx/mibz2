# Alternatives

Loading the chart…

mibz2 beside other bzip2 decoders on master's latest recorded commit, decoded in the same rounds on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-480/benchmarks/index.md>)). A ratio above 1× is the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

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

The latest record: [`137c542`](<https://github.com/lmmx/mibz2/commit/137c542e86483d649a57a6837c3f1b84b449d1cf>) (2026-11-16).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/43838874668>)

INTEL(R) XEON(R) PLATINUM 8573C, 4 cores · rustc 1.100.0 (0ef1bc386 2026-11-09) · bench 3.8.1

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.523 s | 401 | 7.8 | 2.99× (2.98–3.01) | 6.72× (6.68–6.77) | 7.19× (7.16–7.22) | 8.08× (8.02–8.13) | 7.22× (7.16–7.27) | 7.75× (7.69–7.80) | 7.42× (7.37–7.47) | 2.31× (2.30–2.32) | 2.16× (2.15–2.17) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.303 s | 692 | 8.4 | 2.95× (2.80–3.11) | 6.16× (5.98–6.35) | 7.24× (6.92–7.57) | 7.44× (7.09–7.81) | 6.61× (6.29–6.94) | 13.10× (12.45–13.79) | 12.55× (11.97–13.16) | 3.90× (3.72–4.10) | 3.66× (3.49–3.84) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.244 s | 860 | 13.4 | 3.19× (3.16–3.23) | 5.56× (5.48–5.64) | 6.40× (6.24–6.56) | 6.33× (6.28–6.39) | 6.16× (6.07–6.25) | 16.51× (16.34–16.69) | 15.81× (15.63–16.01) | 4.92× (4.87–4.96) | 4.61× (4.57–4.65) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.021 s | 943 | 3.5 | 0.58× (0.57–0.59) | 3.38× (3.31–3.45) | 4.31× (4.22–4.40) | 3.81× (3.73–3.90) | 6.27× (6.15–6.39) | 2.81× (2.77–2.86) | 2.82× (2.76–2.87) | 1.14× (1.12–1.16) | 0.98× (0.96–1.00) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.021 s | 937 | 3.6 | 0.55× (0.54–0.56) | 2.89× (2.85–2.93) | 5.72× (5.64–5.79) | 3.77× (3.72–3.83) | 6.17× (6.05–6.28) | 2.79× (2.76–2.83) | 2.79× (2.76–2.83) | 1.13× (1.11–1.16) | 0.97× (0.96–0.99) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.021 s | 935 | 3.6 | 0.54× (0.52–0.56) | 2.87× (2.72–3.02) | 5.61× (5.39–5.85) | 3.69× (3.55–3.82) | 6.09× (5.86–6.32) | 2.72× (2.63–2.82) | 2.73× (2.63–2.82) | 1.10× (1.06–1.14) | 0.95× (0.92–0.98) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.038 s | 513 | 6.3 | 2.81× (2.76–2.86) | 3.44× (3.37–3.51) | 3.93× (3.86–4.01) | 4.45× (4.37–4.53) | 4.84× (4.74–4.93) | 3.66× (3.58–3.74) | 2.98× (2.93–3.03) | 2.10× (2.05–2.14) | 2.11× (2.07–2.16) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.024 s | 820 | 7.5 | 2.38× (2.05–2.77) | 3.06× (2.91–3.22) | 4.15× (3.64–4.74) | 3.86× (3.31–4.49) | 5.08× (4.41–5.86) | 5.54× (4.80–6.38) | 4.51× (3.89–5.21) | 3.17× (2.74–3.68) | 3.20× (2.77–3.70) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.025 s | 790 | 13.4 | 2.08× (1.96–2.21) | 2.11× (1.99–2.23) | 2.96× (2.76–3.18) | 2.71× (2.44–3.01) | 4.21× (3.96–4.48) | 5.58× (5.29–5.89) | 4.54× (4.28–4.82) | 3.20× (3.03–3.38) | 3.23× (3.04–3.43) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.153 s | 52 | 59.8 | 1.30× (1.29–1.31) | 2.58× (2.56–2.60) | 2.25× (2.21–2.29) | 2.31× (2.29–2.32) | 2.62× (2.60–2.64) | 2.98× (2.95–3.01) | 3.14× (3.09–3.19) | 1.26× (1.24–1.27) | 1.27× (1.25–1.28) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.095 s | 85 | 65.9 | 1.22× (1.14–1.31) | 2.44× (2.27–2.63) | 2.25× (2.11–2.40) | 2.86× (2.66–3.08) | 2.67× (2.51–2.85) | 4.81× (4.50–5.14) | 5.06× (4.72–5.42) | 2.03× (1.91–2.15) | 2.04× (1.91–2.17) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.075 s | 107 | 88.0 | 1.38× (1.34–1.42) | 2.34× (2.28–2.41) | 2.23× (2.20–2.25) | 2.74× (2.68–2.80) | 2.80× (2.74–2.87) | 6.07× (5.93–6.22) | 6.38× (6.24–6.52) | 2.56× (2.49–2.62) | 2.57× (2.52–2.63) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.169 s | 57 | 55.1 | 1.35× (1.33–1.37) | 2.95× (2.89–3.00) | 2.41× (2.36–2.45) | 2.51× (2.48–2.55) | 2.84× (2.80–2.89) | 3.61× (3.56–3.67) | 4.11× (4.04–4.17) | 1.48× (1.45–1.51) | 1.53× (1.49–1.57) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.101 s | 95 | 61.3 | 1.30× (1.24–1.37) | 2.74× (2.68–2.80) | 2.43× (2.38–2.50) | 3.44× (3.27–3.61) | 2.91× (2.88–2.94) | 6.10× (6.00–6.19) | 6.93× (6.82–7.04) | 2.50× (2.43–2.56) | 2.58× (2.53–2.63) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.086 s | 112 | 86.3 | 1.39× (1.35–1.44) | 2.61× (2.51–2.72) | 2.28× (2.22–2.35) | 3.07× (2.95–3.19) | 2.78× (2.70–2.87) | 7.12× (6.89–7.36) | 8.09× (7.84–8.35) | 2.91× (2.81–3.02) | 3.01× (2.91–3.11) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.014 s | 719 | 4.8 | 0.63× (0.62–0.65) | 2.97× (2.91–3.03) | 4.35× (4.30–4.39) | 3.27× (3.23–3.30) | 6.80× (6.72–6.88) | 2.44× (2.40–2.48) | 2.46× (2.43–2.49) | 1.05× (1.03–1.07) | 1.49× (1.47–1.51) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.015 s | 683 | 5.2 | 0.55× (0.52–0.58) | 2.84× (2.78–2.90) | 4.89× (4.79–5.00) | 3.16× (3.09–3.23) | 6.53× (6.42–6.65) | 2.34× (2.30–2.39) | 2.36× (2.31–2.42) | 1.01× (0.97–1.04) | 1.43× (1.40–1.46) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.015 s | 676 | 5.1 | 0.53× (0.52–0.55) | 2.79× (2.71–2.86) | 4.78× (4.64–4.92) | 3.11× (3.03–3.19) | 6.36× (6.19–6.54) | 2.29× (2.23–2.35) | 2.31× (2.25–2.37) | 0.98× (0.95–1.02) | 1.40× (1.36–1.44) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 0.078 s | 172 | 18.4 | 1.78× (1.76–1.80) | 3.87× (3.84–3.90) | 4.33× (4.28–4.38) | 4.53× (4.45–4.61) | 4.55× (4.52–4.59) | 4.93× (4.85–5.01) | 4.85× (4.81–4.89) | 1.46× (1.45–1.48) | 1.44× (1.43–1.45) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 0.050 s | 270 | 21.8 | 1.62× (1.51–1.74) | 3.56× (3.25–3.91) | 4.24× (4.01–4.49) | 4.47× (4.18–4.77) | 4.43× (4.19–4.67) | 7.54× (7.13–7.97) | 7.41× (7.03–7.81) | 2.24× (2.11–2.37) | 2.20× (2.09–2.32) | yes |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 0.042 s | 315 | 32.6 | 1.71× (1.66–1.77) | 3.11× (2.94–3.29) | 3.85× (3.71–3.99) | 3.99× (3.90–4.09) | 4.37× (4.28–4.47) | 9.01× (8.77–9.26) | 8.86× (8.66–9.06) | 2.67× (2.61–2.74) | 2.63× (2.57–2.69) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 2.99× (2.97–3.00) | 6.69× (6.65–6.74) | 7.16× (7.13–7.19) | 8.05× (8.00–8.11) | 7.20× (7.14–7.25) | 7.72× (7.67–7.78) | 7.40× (7.35–7.45) | 2.30× (2.29–2.31) | 2.15× (2.14–2.16) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.86× (2.83–2.90) | 6.45× (6.37–6.54) | 7.05× (6.99–7.12) | 7.91× (7.81–8.01) | 6.73× (6.66–6.81) | 7.03× (6.94–7.12) | 6.74× (6.63–6.84) | 2.10× (2.07–2.12) | 1.96× (1.93–1.98) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.11× (3.06–3.15) | 5.91× (5.84–5.98) | 6.15× (6.07–6.23) | 6.75× (6.70–6.81) | 5.94× (5.89–6.00) | 4.48× (4.46–4.51) | 4.30× (4.26–4.33) | 1.34× (1.33–1.34) | 1.25× (1.24–1.25) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.63) | 2.89× (2.83–2.94) | 3.97× (3.88–4.06) | 3.60× (3.52–3.69) | 5.88× (5.76–6.00) | 2.68× (2.63–2.72) | 2.68× (2.63–2.74) | 1.13× (1.11–1.15) | 0.91× (0.89–0.93) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.59× (0.58–0.61) | 2.84× (2.80–2.89) | 5.15× (5.10–5.20) | 3.54× (3.49–3.59) | 5.71× (5.61–5.82) | 2.62× (2.59–2.65) | 2.63× (2.60–2.66) | 1.11× (1.08–1.13) | 0.89× (0.88–0.90) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.57× (0.56–0.59) | 3.21× (3.11–3.32) | 5.13× (4.96–5.30) | 3.53× (3.42–3.64) | 5.73× (5.55–5.92) | 2.60× (2.53–2.68) | 2.61× (2.54–2.69) | 1.10× (1.07–1.13) | 0.89× (0.86–0.92) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.73× (2.68–2.78) | 3.16× (3.11–3.21) | 3.69× (3.64–3.75) | 4.30× (4.23–4.37) | 4.67× (4.58–4.76) | 3.54× (3.47–3.62) | 2.89× (2.84–2.94) | 2.05× (2.00–2.09) | 1.95× (1.91–1.99) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.45× (2.40–2.51) | 2.85× (2.74–2.97) | 3.34× (3.28–3.40) | 3.69× (3.61–3.77) | 3.99× (3.92–4.06) | 2.96× (2.91–3.02) | 2.42× (2.37–2.47) | 1.71× (1.67–1.76) | 1.63× (1.60–1.67) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.26× (2.17–2.35) | 1.92× (1.84–2.01) | 2.35× (2.27–2.43) | 2.31× (2.23–2.40) | 2.43× (2.35–2.51) | 1.65× (1.59–1.71) | 1.35× (1.30–1.40) | 0.95× (0.92–0.99) | 0.91× (0.88–0.94) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.30× (1.28–1.31) | 2.52× (2.50–2.55) | 2.20× (2.17–2.22) | 2.29× (2.28–2.31) | 2.60× (2.58–2.62) | 2.96× (2.93–2.99) | 3.11× (3.06–3.16) | 1.25× (1.24–1.27) | 1.24× (1.23–1.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.25× (1.22–1.27) | 2.46× (2.42–2.49) | 2.13× (2.10–2.17) | 2.77× (2.72–2.82) | 2.45× (2.40–2.49) | 2.69× (2.65–2.73) | 2.82× (2.77–2.88) | 1.14× (1.12–1.15) | 1.13× (1.12–1.14) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.47× (1.40–1.55) | 2.53× (2.48–2.58) | 2.08× (2.03–2.12) | 2.74× (2.67–2.81) | 2.45× (2.39–2.51) | 1.99× (1.95–2.03) | 2.09× (2.04–2.14) | 0.84× (0.82–0.86) | 0.84× (0.82–0.86) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.34× (1.32–1.37) | 2.91× (2.86–2.97) | 2.37× (2.33–2.40) | 2.50× (2.46–2.54) | 2.83× (2.78–2.87) | 3.59× (3.53–3.65) | 4.08× (4.01–4.15) | 1.47× (1.45–1.50) | 1.50× (1.46–1.53) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.29× (1.26–1.31) | 2.78× (2.72–2.84) | 2.28× (2.23–2.34) | 3.13× (3.06–3.21) | 2.64× (2.59–2.70) | 3.23× (3.16–3.30) | 3.67× (3.59–3.75) | 1.33× (1.29–1.36) | 1.35× (1.31–1.38) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.58× (1.54–1.62) | 2.91× (2.81–3.01) | 2.14× (2.10–2.19) | 2.98× (2.90–3.06) | 2.59× (2.53–2.66) | 2.32× (2.26–2.38) | 2.63× (2.56–2.71) | 0.95× (0.92–0.98) | 0.97× (0.94–0.99) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.66–0.69) | 2.42× (2.37–2.46) | 3.48× (3.43–3.52) | 3.01× (2.98–3.04) | 6.16× (6.07–6.24) | 2.28× (2.25–2.32) | 2.30× (2.27–2.33) | 1.04× (1.02–1.06) | 0.83× (0.82–0.84) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.61× (0.58–0.64) | 2.29× (2.24–2.35) | 4.23× (4.15–4.31) | 2.93× (2.87–3.00) | 5.86× (5.75–5.96) | 2.16× (2.12–2.20) | 2.18× (2.13–2.22) | 0.98× (0.96–1.01) | 0.79× (0.78–0.80) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.58× (0.57–0.59) | 2.63× (2.57–2.69) | 4.24× (4.15–4.33) | 2.99× (2.93–3.04) | 5.88× (5.77–6.00) | 2.18× (2.14–2.22) | 2.19× (2.14–2.25) | 0.99× (0.96–1.02) | 0.79× (0.78–0.81) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 1.76× (1.74–1.78) | 3.75× (3.73–3.77) | 4.19× (4.14–4.24) | 4.45× (4.38–4.53) | 4.48× (4.44–4.51) | 4.84× (4.77–4.92) | 4.76× (4.72–4.80) | 1.45× (1.44–1.47) | 1.38× (1.37–1.40) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 1.62× (1.60–1.64) | 3.49× (3.27–3.73) | 3.93× (3.89–3.97) | 4.31× (4.26–4.36) | 3.92× (3.89–3.96) | 4.09× (4.02–4.16) | 4.02× (3.98–4.06) | 1.23× (1.21–1.24) | 1.17× (1.15–1.18) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 1.81× (1.72–1.90) | 3.10× (3.03–3.16) | 3.48× (3.40–3.55) | 3.80× (3.74–3.86) | 3.47× (3.41–3.52) | 2.74× (2.69–2.79) | 2.69× (2.66–2.73) | 0.82× (0.81–0.84) | 0.78× (0.77–0.79) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.31–0.31) | 0.26× (0.26–0.26) | 0.98× (0.98–0.98) | 0.78× (0.78–0.78) | 11.06× (11.03–11.09) | 0.26× (0.26–0.26) | 0.29× (0.28–0.29) | 0.43× (0.43–0.43) | 0.22× (0.21–0.22) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 58 MB | 0.32× (0.31–0.34) | 0.41× (0.34–0.50) | 0.66× (0.63–0.70) | 0.57× (0.54–0.59) | 5.34× (5.08–5.61) | 0.13× (0.12–0.13) | 0.14× (0.13–0.14) | 0.21× (0.20–0.22) | 0.10× (0.10–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 91 MB | 0.33× (0.32–0.35) | 0.54× (0.45–0.65) | 0.60× (0.57–0.62) | 0.52× (0.50–0.55) | 3.50× (3.36–3.64) | 0.08× (0.08–0.08) | 0.09× (0.08–0.09) | 0.13× (0.13–0.14) | 0.07× (0.06–0.07) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.17× (0.16–0.18) | 0.81× (0.81–0.82) | 0.80× (0.80–0.81) | 4.29× (4.28–4.29) | 0.26× (0.26–0.26) | 0.27× (0.27–0.28) | 0.91× (0.91–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.17) | 1.77× (1.77–1.77) | 0.84× (0.84–0.84) | 4.39× (4.38–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.92–0.93) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 33 MB | 0.20× (0.20–0.20) | 0.16× (0.16–0.16) | 1.77× (1.77–1.78) | 0.84× (0.84–0.84) | 4.39× (4.39–4.40) | 0.26× (0.26–0.26) | 0.28× (0.28–0.28) | 0.93× (0.93–0.93) | 0.11× (0.11–0.11) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.52× (0.51–0.52) | 0.43× (0.41–0.45) | 1.70× (1.70–1.71) | 0.66× (0.65–0.66) | 8.02× (7.99–8.06) | 0.65× (0.65–0.66) | 0.69× (0.68–0.69) | 0.75× (0.74–0.75) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.66× (0.64–0.68) | 0.61× (0.57–0.65) | 1.60× (1.56–1.64) | 0.80× (0.79–0.82) | 5.83× (5.70–5.95) | 0.47× (0.46–0.48) | 0.50× (0.49–0.51) | 0.54× (0.53–0.55) | 0.27× (0.26–0.27) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 38 MB | 0.65× (0.62–0.67) | 0.70× (0.65–0.75) | 1.15× (1.10–1.20) | 0.70× (0.68–0.73) | 3.44× (3.33–3.56) | 0.27× (0.26–0.28) | 0.28× (0.27–0.29) | 0.31× (0.30–0.32) | 0.15× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.73× (0.73–0.74) | 5.23× (5.21–5.24) | 0.33× (0.32–0.33) | 0.35× (0.34–0.36) | 0.54× (0.53–0.54) | 0.27× (0.27–0.27) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 38 MB | 0.50× (0.48–0.52) | 0.39× (0.37–0.42) | 0.90× (0.87–0.93) | 0.68× (0.65–0.71) | 3.13× (3.02–3.24) | 0.19× (0.19–0.20) | 0.21× (0.20–0.21) | 0.32× (0.31–0.33) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.62–0.66) | 0.45× (0.44–0.46) | 0.86× (0.84–0.88) | 0.81× (0.79–0.82) | 2.50× (2.44–2.55) | 0.15× (0.15–0.15) | 0.16× (0.16–0.16) | 0.24× (0.24–0.25) | 0.12× (0.12–0.12) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.41× (0.40–0.41) | 0.32× (0.32–0.32) | 1.18× (1.18–1.18) | 0.83× (0.82–0.83) | 5.22× (5.21–5.24) | 0.32× (0.32–0.33) | 0.35× (0.34–0.35) | 0.53× (0.52–0.53) | 0.26× (0.26–0.27) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 42 MB | 0.48× (0.46–0.49) | 0.37× (0.34–0.40) | 0.84× (0.82–0.86) | 0.69× (0.67–0.71) | 2.88× (2.83–2.94) | 0.18× (0.17–0.18) | 0.19× (0.19–0.19) | 0.29× (0.28–0.29) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 52 MB | 0.63× (0.61–0.64) | 0.50× (0.47–0.53) | 0.82× (0.81–0.83) | 0.82× (0.78–0.86) | 2.37× (2.34–2.41) | 0.14× (0.14–0.14) | 0.15× (0.15–0.15) | 0.23× (0.23–0.23) | 0.12× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.38) | 0.20× (0.20–0.20) | 1.07× (1.07–1.08) | 0.76× (0.76–0.76) | 5.37× (5.36–5.39) | 0.35× (0.35–0.35) | 0.37× (0.37–0.38) | 0.92× (0.92–0.92) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 24 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.59–1.60) | 0.77× (0.77–0.77) | 5.30× (5.29–5.31) | 0.34× (0.34–0.35) | 0.37× (0.36–0.37) | 0.90× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 23 MB | 0.24× (0.24–0.24) | 0.20× (0.20–0.20) | 1.60× (1.60–1.61) | 0.77× (0.77–0.78) | 5.33× (5.30–5.35) | 0.35× (0.34–0.35) | 0.37× (0.37–0.37) | 0.91× (0.90–0.92) | 0.13× (0.13–0.13) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 1 | 19 MB | 0.46× (0.46–0.46) | 0.39× (0.39–0.39) | 1.42× (1.41–1.42) | 0.67× (0.67–0.68) | 6.43× (6.41–6.46) | 0.39× (0.39–0.40) | 0.42× (0.42–0.43) | 0.64× (0.63–0.64) | 0.31× (0.31–0.31) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 2 | 37 MB | 0.45× (0.44–0.46) | 0.40× (0.37–0.43) | 1.05× (1.01–1.08) | 0.68× (0.66–0.70) | 3.34× (3.28–3.39) | 0.20× (0.20–0.21) | 0.22× (0.21–0.22) | 0.33× (0.32–0.34) | 0.16× (0.16–0.16) |
| `text` (2.63 MB → 13.4 MB, 5.1×) | 4 | 58 MB | 0.49× (0.48–0.50) | 0.48× (0.44–0.53) | 0.89× (0.87–0.91) | 0.74× (0.71–0.76) | 2.28× (2.22–2.33) | 0.13× (0.13–0.14) | 0.14× (0.14–0.15) | 0.21× (0.21–0.22) | 0.11× (0.10–0.11) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 401 | 2.50 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 692 | 2.73 | 1.10× (1.08–1.11) | 85% | 1.06× (1.04–1.07) | 1.05× (1.04–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 860 | 4.31 | 1.72× (1.71–1.73) | 53% | 1.52× (1.50–1.54) | 1.79× (1.77–1.82) |

**Best:** mibz2 at 4 threads (860 MB/s); lbzip2 at 4 threads (154 MB/s); pbz2 at 4 threads (268 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.83, mibz2 4t 1.17, lbzip2 1t 0.05, lbzip2 2t 0.24, lbzip2 4t 0.36, pbz2 1t 0.00, pbz2 2t 0.34, pbz2 4t 0.39. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 8%, mibz2 4t 9%, lbzip2 2t 2%, lbzip2 4t 2%, pbz2 2t 3%, pbz2 4t 2%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.10 | 0.70 | 0.02 | 0.08 | 0.244 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3098 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 1m06s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 1.19 | 11.5% |
| MTF + RLE2 | 1.72 | 16.6% |
| T-vector build | 1.61 | 15.6% |
| walk (the one each build uses) | 5.38 | 52.2% |
| of which: segment assembly | 0.29 | 2.8% |
| RLE1 | 0.27 | 2.6% |
| CRC | 0.14 | 1.3% |
| **total** | 10.32 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.508 | 1.324 | (in its total) | 2.61× (2.60–2.62) |  |
| walk | 1.630 | 2.999 | (in its total) | 1.84× (1.83–1.85) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.092 | 0.081 | (in its total) | 0.89× (0.87–0.92) |  |
| inverse BWT, total | 2.228 | 4.404 | 2.367 | 1.98× (1.97–1.99) | 1.06× (1.06–1.06) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 1h16m ± 8m32s |
| 4 threads | 35m43s ± 5m01s |
| 4 threads, the sample's overlap gap as a fixed cost | 25m23s ± 3m35s |
| the scanner alone (its serial part) | 15m17s ± 32.9s |
| the workers alone, over 4 | 25m23s ± 3m35s |

At 4 threads the workers bound it. Per byte: the scanner 8.9 ns a byte of bz2, the workers 3.33 ns a byte of output at 4 threads (CPU, summed over them), 2.00 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 7 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,629,453 | 13,353,938 | \`724a67aca091a7ae…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: INTEL(R) XEON(R) PLATINUM 8573C, 4 cores, reported clock 3097.6 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1019-azure; image ubuntu26 20261111.196.1 - rustc 1.100.0 (0ef1bc386 2026-11-09); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.1 (bench/report.py), 10 rounds, threads 1, 2, 4
