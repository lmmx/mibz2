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

The latest record: [`5d1bec7`](<https://github.com/lmmx/mibz2/commit/5d1bec79f8f78ee1693db17e89b6545f636ee577>) (2026-11-22).

**mibz2 against pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs, libbzip2, libbz2-rs, bz2ngrs and bz2ng**: 10 rounds on one runner · [run](<https://github.com/lmmx/mibz2/actions/runs/45073872726>)

AMD EPYC 9V74 80-Core Processor, 4 cores · rustc 1.99.1 (58cf8f9c3 2026-10-13) · bench 3.8.2

Each round decodes every corpus once with each decoder at each thread count (`ab run`), their order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is faster (or leaner). 1-thread runs are pinned to one CPU.

Block-parallel: pbz2, lbzip2, indexed\_bzip2, fbz, lbzip2-rs. Single-threaded (¹): libbzip2, libbz2-rs, bz2ngrs, bz2ng, run at 1 thread only; at 4 threads their figures are their 1-thread runs, against mibz2's at 4 threads.

## Wall time against mibz2

| corpus | threads | mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ | same bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 0.393 s | 534 | 7.0 | 3.11× (3.08–3.14) | 7.11× (7.05–7.18) | 7.35× (7.31–7.40) | 8.40× (8.33–8.47) | 7.34× (7.29–7.39) | 7.66× (7.63–7.70) | 7.23× (7.21–7.26) | 2.32× (2.30–2.34) | 2.57× (2.56–2.58) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 0.227 s | 924 | 7.5 | 3.12× (3.04–3.21) | 6.49× (6.24–6.75) | 7.52× (7.23–7.84) | 7.83× (7.56–8.11) | 6.88× (6.67–7.09) | 13.22× (12.83–13.62) | 12.48× (12.13–12.84) | 4.00× (3.88–4.13) | 4.43× (4.29–4.57) | yes |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 0.182 s | 1,149 | 11.8 | 3.28× (3.24–3.33) | 4.79× (4.67–4.91) | 6.18× (5.92–6.44) | 5.90× (5.79–6.02) | 5.74× (5.64–5.85) | 16.40× (16.06–16.74) | 15.48× (15.18–15.79) | 4.96× (4.85–5.08) | 5.49× (5.38–5.61) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.018 s | 1,114 | 3.6 | 0.57× (0.56–0.58) | 3.38× (3.33–3.44) | 4.52× (4.43–4.60) | 3.54× (3.48–3.61) | 6.11× (6.03–6.20) | 2.63× (2.58–2.67) | 2.63× (2.58–2.67) | 0.97× (0.94–1.01) | 2.07× (1.89–2.27) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.019 s | 1,070 | 3.9 | 0.51× (0.50–0.52) | 2.72× (2.68–2.77) | 5.44× (5.35–5.53) | 3.41× (3.35–3.46) | 5.82× (5.68–5.96) | 2.52× (2.48–2.57) | 2.53× (2.48–2.57) | 0.94× (0.90–0.97) | 1.99× (1.80–2.20) | yes |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.019 s | 1,042 | 3.9 | 0.51× (0.50–0.52) | 2.68× (2.63–2.73) | 5.35× (5.25–5.44) | 3.35× (3.29–3.41) | 5.70× (5.57–5.83) | 2.48× (2.43–2.52) | 2.48× (2.43–2.52) | 0.92× (0.88–0.96) | 1.95× (1.75–2.17) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 0.026 s | 745 | 5.3 | 2.72× (2.70–2.73) | 3.10× (3.08–3.11) | 3.83× (3.75–3.92) | 4.13× (4.12–4.15) | 4.53× (4.50–4.56) | 2.79× (2.77–2.80) | 2.29× (2.28–2.30) | 2.11× (2.09–2.12) | 3.09× (3.07–3.10) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 0.017 s | 1,130 | 6.4 | 2.29× (2.17–2.42) | 2.87× (2.70–3.04) | 4.38× (4.04–4.75) | 3.59× (3.42–3.77) | 5.15× (4.86–5.47) | 4.10× (3.86–4.35) | 3.37× (3.17–3.57) | 3.10× (2.92–3.28) | 4.54× (4.28–4.81) | yes |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 0.018 s | 1,071 | 12.2 | 2.10× (2.00–2.22) | 2.09× (1.93–2.26) | 3.41× (3.25–3.58) | 2.34× (2.20–2.49) | 4.45× (4.22–4.70) | 3.92× (3.74–4.12) | 3.22× (3.07–3.38) | 2.96× (2.82–3.11) | 4.34× (4.14–4.55) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 0.107 s | 74 | 50.4 | 1.43× (1.41–1.44) | 2.74× (2.70–2.77) | 2.49× (2.44–2.55) | 2.55× (2.53–2.58) | 2.91× (2.85–2.97) | 3.09× (3.07–3.12) | 3.09× (3.04–3.14) | 1.29× (1.28–1.30) | 1.46× (1.43–1.50) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 0.065 s | 122 | 56.2 | 1.37× (1.30–1.45) | 2.61× (2.55–2.67) | 2.52× (2.46–2.59) | 3.20× (3.13–3.26) | 3.05× (3.01–3.09) | 5.09× (5.00–5.19) | 5.08× (4.98–5.19) | 2.13× (2.09–2.17) | 2.41× (2.35–2.47) | yes |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 0.052 s | 155 | 75.8 | 1.44× (1.39–1.48) | 2.33× (2.23–2.44) | 2.34× (2.24–2.44) | 2.85× (2.78–2.92) | 3.16× (3.08–3.24) | 6.40× (6.24–6.55) | 6.38× (6.20–6.57) | 2.67× (2.60–2.74) | 3.03× (2.91–3.15) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 0.116 s | 83 | 45.3 | 1.55× (1.50–1.61) | 3.09× (2.97–3.22) | 2.69× (2.57–2.81) | 2.79× (2.69–2.90) | 3.15× (3.03–3.28) | 3.73× (3.58–3.88) | 3.94× (3.77–4.11) | 1.57× (1.53–1.62) | 1.82× (1.76–1.88) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 0.070 s | 137 | 50.8 | 1.43× (1.39–1.47) | 2.93× (2.83–3.04) | 2.64× (2.58–2.69) | 3.66× (3.56–3.77) | 3.23× (3.14–3.33) | 6.25× (6.08–6.43) | 6.60× (6.42–6.79) | 2.64× (2.55–2.72) | 3.05× (2.97–3.14) | yes |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 0.059 s | 162 | 73.6 | 1.51× (1.48–1.55) | 2.43× (2.34–2.53) | 2.48× (2.33–2.64) | 3.28× (3.25–3.32) | 3.12× (3.05–3.20) | 7.43× (7.34–7.52) | 7.84× (7.74–7.95) | 3.13× (3.07–3.20) | 3.62× (3.56–3.69) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.012 s | 840 | 5.0 | 0.62× (0.62–0.63) | 3.45× (3.41–3.50) | 4.34× (4.28–4.40) | 3.09× (3.06–3.13) | 6.76× (6.64–6.88) | 2.28× (2.25–2.31) | 2.32× (2.29–2.35) | 0.95× (0.94–0.96) | 1.73× (1.71–1.76) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.013 s | 769 | 5.8 | 0.50× (0.49–0.51) | 2.38× (2.35–2.42) | 4.79× (4.72–4.86) | 2.87× (2.84–2.91) | 6.14× (6.06–6.23) | 2.09× (2.06–2.12) | 2.12× (2.09–2.15) | 0.87× (0.85–0.88) | 1.59× (1.57–1.61) | yes |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.013 s | 760 | 5.6 | 0.50× (0.49–0.51) | 2.35× (2.31–2.39) | 4.71× (4.62–4.79) | 2.83× (2.78–2.88) | 6.06× (5.94–6.18) | 2.06× (2.02–2.10) | 2.09× (2.06–2.12) | 0.85× (0.84–0.87) | 1.56× (1.53–1.59) | yes |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 1 | 0.058 s | 238 | 15.9 | 1.90× (1.88–1.92) | 4.02× (3.94–4.10) | 4.51× (4.43–4.60) | 4.75× (4.66–4.83) | 4.90× (4.79–5.01) | 4.90× (4.84–4.96) | 4.77× (4.69–4.86) | 1.58× (1.55–1.60) | 1.77× (1.74–1.80) | yes |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 2 | 0.042 s | 333 | 20.3 | 1.43× (1.30–1.57) | 3.14× (2.87–3.44) | 3.78× (3.44–4.16) | 4.34× (3.96–4.75) | 4.10× (3.76–4.46) | 6.69× (6.13–7.31) | 6.52× (5.98–7.11) | 2.15× (1.97–2.34) | 2.41× (2.19–2.66) | yes |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 4 | 0.037 s | 370 | 30.5 | 1.52× (1.47–1.57) | 2.43× (2.27–2.61) | 3.41× (3.27–3.56) | 3.48× (3.40–3.57) | 3.82× (3.72–3.93) | 7.66× (7.46–7.86) | 7.46× (7.30–7.62) | 2.46× (2.40–2.53) | 2.76× (2.67–2.86) | yes |

## CPU time against mibz2

| corpus | threads | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 3.10× (3.07–3.14) | 7.07× (7.02–7.13) | 7.32× (7.27–7.36) | 8.37× (8.29–8.44) | 7.31× (7.26–7.36) | 7.63× (7.60–7.67) | 7.21× (7.18–7.23) | 2.31× (2.29–2.34) | 2.55× (2.54–2.56) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 2.98× (2.93–3.02) | 6.75× (6.65–6.85) | 7.11× (6.97–7.25) | 8.20× (8.10–8.31) | 6.91× (6.80–7.03) | 7.05× (6.98–7.13) | 6.66× (6.60–6.72) | 2.14× (2.11–2.17) | 2.35× (2.33–2.38) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 3.15× (3.08–3.23) | 5.14× (5.05–5.24) | 5.75× (5.60–5.91) | 6.32× (6.17–6.48) | 5.39× (5.27–5.52) | 4.51× (4.45–4.57) | 4.26× (4.19–4.32) | 1.37× (1.34–1.39) | 1.50× (1.48–1.52) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 0.61× (0.60–0.61) | 2.89× (2.86–2.93) | 3.89× (3.80–3.99) | 3.32× (3.26–3.37) | 5.66× (5.59–5.73) | 2.48× (2.44–2.52) | 2.48× (2.45–2.52) | 0.98× (0.95–1.01) | 1.64× (1.61–1.66) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 0.55× (0.54–0.55) | 2.58× (2.54–2.62) | 4.67× (4.60–4.75) | 3.11× (3.06–3.16) | 5.22× (5.10–5.34) | 2.31× (2.27–2.34) | 2.31× (2.27–2.34) | 0.91× (0.88–0.94) | 1.52× (1.50–1.54) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 0.54× (0.52–0.55) | 2.85× (2.80–2.89) | 4.66× (4.57–4.74) | 3.11× (3.05–3.17) | 5.19× (5.07–5.31) | 2.31× (2.26–2.35) | 2.31× (2.27–2.35) | 0.91× (0.87–0.95) | 1.52× (1.49–1.55) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 2.61× (2.59–2.63) | 2.86× (2.84–2.87) | 3.42× (3.40–3.45) | 3.94× (3.92–3.96) | 4.31× (4.28–4.34) | 2.68× (2.66–2.69) | 2.21× (2.20–2.22) | 2.04× (2.02–2.06) | 2.67× (2.64–2.69) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 2.34× (2.32–2.36) | 2.62× (2.59–2.65) | 3.46× (3.40–3.53) | 3.43× (3.39–3.46) | 3.73× (3.67–3.79) | 2.20× (2.17–2.23) | 1.82× (1.79–1.84) | 1.68× (1.66–1.70) | 2.19× (2.16–2.22) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 2.19× (2.12–2.25) | 1.78× (1.73–1.82) | 2.37× (2.31–2.42) | 2.16× (2.10–2.23) | 2.25× (2.20–2.31) | 1.14× (1.12–1.17) | 0.94× (0.92–0.97) | 0.87× (0.85–0.89) | 1.14× (1.12–1.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 1.42× (1.41–1.44) | 2.66× (2.63–2.70) | 2.45× (2.39–2.51) | 2.53× (2.50–2.55) | 2.88× (2.82–2.94) | 3.06× (3.03–3.09) | 3.05× (3.00–3.10) | 1.29× (1.28–1.30) | 1.40× (1.39–1.42) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 1.34× (1.31–1.37) | 2.53× (2.48–2.58) | 2.25× (2.20–2.31) | 3.02× (2.97–3.08) | 2.65× (2.60–2.71) | 2.75× (2.69–2.80) | 2.74× (2.69–2.79) | 1.16× (1.13–1.18) | 1.26× (1.24–1.28) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 1.49× (1.44–1.54) | 2.37× (2.32–2.41) | 2.01× (1.97–2.06) | 2.71× (2.67–2.76) | 2.60× (2.54–2.66) | 2.05× (2.03–2.06) | 2.04× (2.01–2.08) | 0.86× (0.85–0.87) | 0.94× (0.92–0.96) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 1.54× (1.49–1.60) | 3.01× (2.90–3.13) | 2.63× (2.53–2.75) | 2.77× (2.66–2.87) | 3.12× (3.00–3.24) | 3.69× (3.54–3.83) | 3.89× (3.73–4.06) | 1.56× (1.52–1.61) | 1.78× (1.73–1.84) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 1.45× (1.43–1.47) | 2.89× (2.85–2.94) | 2.45× (2.42–2.48) | 3.46× (3.40–3.52) | 2.89× (2.85–2.94) | 3.31× (3.26–3.37) | 3.50× (3.43–3.57) | 1.41× (1.38–1.44) | 1.60× (1.58–1.63) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 1.61× (1.55–1.67) | 2.59× (2.50–2.69) | 2.10× (1.96–2.24) | 2.92× (2.86–2.98) | 2.71× (2.62–2.80) | 2.32× (2.28–2.35) | 2.45× (2.41–2.48) | 0.98× (0.96–1.01) | 1.12× (1.09–1.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 0.67× (0.67–0.68) | 2.47× (2.44–2.49) | 3.41× (3.37–3.45) | 2.83× (2.80–2.86) | 6.04× (5.94–6.14) | 2.12× (2.10–2.15) | 2.15× (2.13–2.18) | 0.96× (0.95–0.97) | 1.37× (1.36–1.39) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 0.55× (0.54–0.56) | 2.13× (2.08–2.18) | 3.80× (3.71–3.88) | 2.59× (2.56–2.62) | 5.28× (5.21–5.36) | 1.86× (1.83–1.89) | 1.89× (1.86–1.92) | 0.84× (0.82–0.86) | 1.20× (1.18–1.22) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 0.54× (0.52–0.56) | 2.34× (2.29–2.38) | 3.81× (3.72–3.91) | 2.61× (2.57–2.65) | 5.36× (5.25–5.46) | 1.88× (1.85–1.92) | 1.91× (1.89–1.94) | 0.85× (0.83–0.87) | 1.22× (1.20–1.24) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 1 | 1.88× (1.86–1.89) | 3.89× (3.82–3.96) | 4.32× (4.25–4.38) | 4.63× (4.55–4.72) | 4.78× (4.68–4.89) | 4.79× (4.73–4.84) | 4.66× (4.58–4.75) | 1.56× (1.54–1.58) | 1.69× (1.67–1.71) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 2 | 1.56× (1.50–1.62) | 3.24× (3.11–3.38) | 3.53× (3.39–3.68) | 4.18× (3.99–4.37) | 3.80× (3.64–3.98) | 3.74× (3.57–3.92) | 3.65× (3.48–3.82) | 1.22× (1.16–1.28) | 1.32× (1.26–1.38) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 4 | 1.69× (1.65–1.73) | 2.66× (2.58–2.73) | 3.01× (2.91–3.12) | 3.44× (3.38–3.50) | 3.18× (3.11–3.24) | 2.49× (2.46–2.53) | 2.43× (2.39–2.47) | 0.81× (0.80–0.82) | 0.88× (0.86–0.90) |

## Peak RSS against mibz2

| corpus | threads | mibz2 peak RSS | pbz2 | lbzip2 | indexed\_bzip2 | fbz | lbzip2-rs | libbzip2 ¹ | libbz2-rs ¹ | bz2ngrs ¹ | bz2ng ¹ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 1 | 28 MB | 0.31× (0.30–0.31) | 0.33× (0.32–0.34) | 0.97× (0.97–0.97) | 0.78× (0.78–0.78) | 10.99× (10.97–11.01) | 0.26× (0.26–0.27) | 0.28× (0.28–0.28) | 0.43× (0.42–0.43) | 0.21× (0.21–0.21) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 2 | 54 MB | 0.35× (0.34–0.36) | 0.45× (0.37–0.56) | 0.72× (0.69–0.74) | 0.61× (0.58–0.63) | 5.73× (5.52–5.94) | 0.14× (0.13–0.14) | 0.15× (0.14–0.15) | 0.22× (0.21–0.23) | 0.11× (0.11–0.11) |
| `wikidata` (11.9 MB → 210 MB, 17.7×) | 4 | 82 MB | 0.37× (0.36–0.38) | 0.53× (0.46–0.62) | 0.66× (0.64–0.68) | 0.58× (0.56–0.60) | 3.85× (3.72–3.98) | 0.09× (0.09–0.09) | 0.10× (0.09–0.10) | 0.15× (0.14–0.15) | 0.07× (0.07–0.08) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 1 | 34 MB | 0.27× (0.27–0.27) | 0.18× (0.17–0.18) | 0.81× (0.81–0.81) | 0.81× (0.80–0.81) | 4.27× (4.26–4.28) | 0.26× (0.25–0.26) | 0.27× (0.27–0.27) | 0.91× (0.90–0.91) | 0.11× (0.11–0.11) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 2 | 30 MB | 0.23× (0.22–0.23) | 0.18× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.92–0.93) | 4.85× (4.84–4.86) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `onebyte` (51 B → 20 MB, 392156.9×) | 4 | 30 MB | 0.22× (0.22–0.23) | 0.18× (0.17–0.18) | 1.96× (1.95–1.96) | 0.93× (0.92–0.93) | 4.85× (4.84–4.86) | 0.29× (0.29–0.29) | 0.31× (0.30–0.31) | 1.03× (1.02–1.03) | 0.12× (0.12–0.12) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 1 | 16 MB | 0.51× (0.51–0.52) | 0.44× (0.44–0.44) | 1.69× (1.68–1.70) | 0.66× (0.65–0.66) | 7.96× (7.92–8.01) | 0.65× (0.64–0.65) | 0.68× (0.67–0.69) | 0.74× (0.73–0.74) | 0.37× (0.36–0.37) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 2 | 22 MB | 0.71× (0.69–0.73) | 0.73× (0.70–0.76) | 1.67× (1.64–1.71) | 0.84× (0.83–0.86) | 6.09× (5.97–6.21) | 0.49× (0.48–0.50) | 0.52× (0.51–0.53) | 0.56× (0.55–0.58) | 0.28× (0.27–0.29) |
| `pattern` (1.12 kB → 19.6 MB, 17562.7×) | 4 | 36 MB | 0.70× (0.67–0.74) | 0.74× (0.69–0.80) | 1.24× (1.20–1.29) | 0.76× (0.73–0.79) | 3.65× (3.51–3.80) | 0.29× (0.27–0.30) | 0.30× (0.29–0.31) | 0.33× (0.31–0.34) | 0.16× (0.16–0.17) |
| `random` (7.1 MB → 8 MB, 1.1×) | 1 | 23 MB | 0.40× (0.40–0.40) | 0.31× (0.31–0.31) | 1.16× (1.16–1.16) | 0.72× (0.72–0.73) | 5.13× (5.12–5.14) | 0.32× (0.32–0.32) | 0.34× (0.34–0.35) | 0.52× (0.52–0.52) | 0.26× (0.26–0.26) |
| `random` (7.1 MB → 8 MB, 1.1×) | 2 | 39 MB | 0.49× (0.47–0.50) | 0.40× (0.37–0.42) | 0.89× (0.86–0.91) | 0.71× (0.69–0.72) | 3.07× (2.99–3.15) | 0.19× (0.19–0.20) | 0.20× (0.20–0.21) | 0.31× (0.30–0.32) | 0.16× (0.15–0.16) |
| `random` (7.1 MB → 8 MB, 1.1×) | 4 | 50 MB | 0.64× (0.63–0.66) | 0.45× (0.44–0.47) | 0.87× (0.84–0.89) | 0.80× (0.76–0.84) | 2.52× (2.46–2.59) | 0.15× (0.15–0.16) | 0.16× (0.16–0.17) | 0.24× (0.24–0.25) | 0.12× (0.12–0.13) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 1 | 23 MB | 0.40× (0.40–0.41) | 0.31× (0.31–0.31) | 1.17× (1.17–1.17) | 0.83× (0.82–0.83) | 5.18× (5.17–5.18) | 0.32× (0.32–0.32) | 0.34× (0.33–0.34) | 0.52× (0.51–0.52) | 0.26× (0.26–0.26) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 2 | 41 MB | 0.47× (0.45–0.50) | 0.38× (0.35–0.41) | 0.84× (0.80–0.89) | 0.72× (0.69–0.76) | 2.90× (2.76–3.03) | 0.18× (0.17–0.19) | 0.19× (0.18–0.20) | 0.29× (0.28–0.30) | 0.15× (0.14–0.15) |
| `records` (9.59 MB → 9.6 MB, 1.0×) | 4 | 54 MB | 0.60× (0.59–0.61) | 0.51× (0.48–0.54) | 0.81× (0.80–0.83) | 0.82× (0.79–0.84) | 2.34× (2.30–2.37) | 0.14× (0.14–0.14) | 0.15× (0.14–0.15) | 0.22× (0.22–0.23) | 0.11× (0.11–0.12) |
| `runs` (38 kB → 10 MB, 263.2×) | 1 | 23 MB | 0.37× (0.37–0.37) | 0.23× (0.22–0.24) | 1.06× (1.06–1.07) | 0.76× (0.76–0.76) | 5.32× (5.29–5.34) | 0.35× (0.34–0.35) | 0.37× (0.36–0.37) | 0.91× (0.90–0.91) | 0.13× (0.13–0.13) |
| `runs` (38 kB → 10 MB, 263.2×) | 2 | 20 MB | 0.29× (0.28–0.29) | 0.24× (0.24–0.24) | 1.87× (1.86–1.87) | 0.90× (0.89–0.90) | 6.20× (6.19–6.21) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.06× (1.05–1.06) | 0.15× (0.15–0.15) |
| `runs` (38 kB → 10 MB, 263.2×) | 4 | 20 MB | 0.28× (0.27–0.28) | 0.23× (0.23–0.24) | 1.87× (1.86–1.87) | 0.90× (0.90–0.91) | 6.20× (6.18–6.21) | 0.40× (0.40–0.41) | 0.43× (0.42–0.43) | 1.05× (1.05–1.06) | 0.15× (0.15–0.15) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 1 | 20 MB | 0.46× (0.46–0.46) | 0.38× (0.38–0.38) | 1.40× (1.40–1.41) | 0.66× (0.66–0.67) | 6.40× (6.37–6.42) | 0.39× (0.39–0.39) | 0.41× (0.41–0.42) | 0.62× (0.62–0.63) | 0.31× (0.30–0.31) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 2 | 33 MB | 0.50× (0.49–0.52) | 0.44× (0.39–0.49) | 1.15× (1.12–1.18) | 0.76× (0.74–0.78) | 3.73× (3.66–3.81) | 0.23× (0.23–0.23) | 0.24× (0.24–0.25) | 0.37× (0.36–0.37) | 0.18× (0.18–0.18) |
| `text` (2.74 MB → 13.9 MB, 5.1×) | 4 | 51 MB | 0.55× (0.53–0.56) | 0.49× (0.46–0.53) | 0.99× (0.96–1.01) | 0.82× (0.80–0.84) | 2.51× (2.46–2.56) | 0.15× (0.14–0.15) | 0.16× (0.15–0.16) | 0.24× (0.23–0.24) | 0.12× (0.11–0.12) |

**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error.

## Resources: CPU per byte and scaling, the `wikidata` corpus

**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the threads' mean count runnable but not running.

| threads: placement | mibz2 MB/s | mibz2 CPU ns/B | mibz2 amplification | mibz2 efficiency | lbzip2 amplification | pbz2 amplification |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1: pinned to CPU 1 | 534 | 1.88 | 1 | – | 1 | 1 |
| 2: CPUs 0,2, one a core | 924 | 2.05 | 1.08× (1.07–1.09) | 86% | 1.03× (1.02–1.04) | 1.04× (1.02–1.06) |
| 4: all 4 CPUs, sharing SMT siblings | 1,149 | 3.18 | 1.69× (1.67–1.72) | 54% | 1.23× (1.21–1.25) | 1.72× (1.68–1.76) |

**Best:** mibz2 at 4 threads (1,149 MB/s); lbzip2 at 4 threads (237 MB/s); pbz2 at 4 threads (346 MB/s).

Conditions: 4 CPUs on 2 physical cores (0,1 | 2,3); no CPU quota; the run-queue counter checked and counting; 2-thread runs on CPUs 0,2 (one CPU a core); 4-thread runs share cores' SMT siblings.

**How to read it, and what it cannot show**

\- \*\*The four comparisons the assessment reads,\*\* declared in bench/report.py (\`assessment\`): best against best (wall), CPU per byte at the most threads, amplification at each thread count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no change. With no real change, about one run in five flags one by chance: a flag that matters reproduces on a re-run. - \*\*A trade-off is not a failure.\*\* Faster for more CPU or memory, or the reverse, is for the reviewer to weigh against the deployment; the report names it, and nothing here fails CI but the outputs and the accounting. - \*\*What it can't show:\*\* which shared resource a rise in amplification comes from (cache, memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 workers, \#39). The cores' L3 is shared with other tenants and its size unknown. - \*\*Run-queue wait\*\* (threads' mean count runnable, not running; schedstat): mibz2 1t 0.00, mibz2 2t 0.80, mibz2 4t 1.04, lbzip2 1t 0.07, lbzip2 2t 0.19, lbzip2 4t 0.39, pbz2 1t 0.00, pbz2 2t 0.40, pbz2 4t 0.47. A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two CPUs, so some wait is its shape, not interference. - \*\*The per-thread split's unattributed CPU\*\* (the process's total that no sampled thread holds, sampled every 10 ms): mibz2 2t 11%, mibz2 4t 12%, lbzip2 2t 3%, lbzip2 4t 3%, pbz2 2t 4%, pbz2 4t 4%.

## Where mibz2's CPU goes: the Wikidata blocks at 4 threads (median CPU seconds)

| build | scanner (serial) | workers (summed) | the reading thread and the rest | unattributed | wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| mibz2 | 0.06 | 0.52 | 0.01 | 0.08 | 0.182 |

Unattributed: the process's CPU that no sampled thread holds (threads ending between samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run here; a base built before 3.0.0 still does, so its split and the model below undercount its workers against head's.

## Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median c/B)

Each of 5 rounds runs the build's `examples/phases.rs` once (bench/stages.py), pinned to one CPU: the file decoded in decode order, block by block, each stage timed as it runs with the library's own functions (each stage's fastest of 2 passes), summed over the blocks, in cycles a byte of output at the reported clock (3692 MHz). Diagnostic only: the decoder's own figures are the tables above. 5 runs, 51.8s of the run.

| stage | mibz2 c/B (median) | share |
| --- | ---: | ---: |
| scanner (headers, Huffman) | 0.89 | 10.4% |
| MTF + RLE2 | 1.55 | 18.1% |
| T-vector build | 1.50 | 17.6% |
| walk (the one each build uses) | 4.16 | 48.8% |
| of which: segment assembly | 0.16 | 1.8% |
| RLE1 | 0.19 | 2.3% |
| CRC | 0.26 | 3.0% |
| **total** | 8.54 |  |

The walk row is the stage, the same in every build; its implementation is each build's own, as its `phases` names it: mibz2, “walk, room (used)”.

## The inverse BWT alone: mibz2's kernel against bz2ngrs's and libsais's, phase by phase (median ns/B)

The kernels on the same Wikidata blocks' last columns, on one pinned CPU, 10 passes of the file in decode order, the kernels' order rotating (`kernels`, bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart; libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, built single-threaded from its pinned source, timed as one call a block, its build and walk inside it). Every block's bytes are checked the same from every kernel, and its CRC, before the timed passes. mibz2's build and libsais take the byte counts from MTF; bz2ngrs's counts them itself, so its build row holds that count. A ratio above 1× is the other kernel taking longer than mibz2's. libsais is an inverse BWT, not a bzip2 decoder: it isn't in the decoders' figures above. Diagnostic only: the decoders' own figures are the tables above.

| phase | mibz2 | bz2ngrs | libsais | bz2ngrs / mibz2 | libsais / mibz2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| T-vector build | 0.390 | 1.062 | (in its total) | 2.72× (2.72–2.72) |  |
| walk | 1.051 | 2.032 | (in its total) | 1.93× (1.92–1.95) |  |
| segment assembly (mibz2) / gather (bz2ngrs) | 0.039 | 0.041 | (in its total) | 1.04× (1.02–1.07) |  |
| inverse BWT, total | 1.479 | 3.134 | 1.880 | 2.12× (2.10–2.13) | 1.27× (1.27–1.28) |

## Modelled, not measured: a whole dump

The decode of `wikidata-20261019-all.json.bz2` (103 GB of bz2; 1.83 TB of JSON at the sample's 17.69×), scaled from the Wikidata blocks' runs above, round by round (mean ± its 95% interval, the sample ratio's own uncertainty added in):

|  | mibz2 |
| --- | ---: |
| 1 thread | 57m12s ± 6m48s |
| 4 threads | 26m44s ± 3m47s |
| 4 threads, the sample's overlap gap as a fixed cost | 18m46s ± 2m39s |
| the scanner alone (its serial part) | 8m44s ± 0.00s |
| the workers alone, over 4 | 18m46s ± 2m39s |

At 4 threads the workers bound it. Per byte: the scanner 5.1 ns a byte of bz2, the workers 2.46 ns a byte of output at 4 threads (CPU, summed over them), 1.59 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (mibz2).

**What the model assumes**

\- \*\*Linear in bytes:\*\* the scanner's time in the dump's bz2 bytes, the workers' in its output bytes. - \*\*The sample stands for the dump:\*\* 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×. - \*\*The wall at N threads\*\* is the larger of the scanner's time and the workers' time over N (the scanner is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap gives the third row. - \*\*The machine is this runner,\*\* reading the dump from page cache: no download, no disk, no other work. The output is hashed by the reading thread and discarded. - The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that scales with the output; not the runner's: another CPU model gives other figures.

**The corpora, the machine and the toolchain**

| corpus | bz2 bytes | output bytes | SHA-256 | source | |---|--:|--:|---|---| | \`wikidata\` | 11,854,933 | 209,700,196 | \`82f65ace0731962d…\` | wikidata-20261019-all.json.bz2: 12 windows' whole blocks (bench/wikidata.json) | | \`onebyte\` | 51 | 20,000,000 | \`22924f5b3187645f…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`pattern\` | 1,116 | 19,600,000 | \`0f07e2a274b91896…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`random\` | 7,097,921 | 8,000,000 | \`10d444e6204289af…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`records\` | 9,587,840 | 9,600,000 | \`ff8b5f6ed3dd4445…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`runs\` | 37,995 | 10,000,235 | \`af3be4d130faf312…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | | \`text\` | 2,736,457 | 13,860,455 | \`4685e5a04aaa92a2…\` | ab gen (tests/common/mod.rs corpora), libbzip2 -9 | - CPU: AMD EPYC 9V74 80-Core Processor, 4 cores, reported clock 3691.7 MHz (cycles a byte use it), flags avx2, avx512f, bmi1, bmi2, pclmulqdq, sse4\_1, ssse3; kernel 7.0.0-1020-azure; image ubuntu26 20261116.202.1 - rustc 1.99.1 (58cf8f9c3 2026-10-13); pbz2 0.2.0; lbzip2 version 2.5; indexed\_bzip2 1.7.0; fbz 0.1.13; lbzip2-rs 0.5.13 (the lbzip2 crate); libbzip2 1.0.8 (the bzip2 crate's bundled source); libbz2-rs 0.2.5 (libbz2-rs-sys, through the bzip2 crate); bz2ngrs 0.1.0 at 6a92590 (git); bz2ng bz2-ng 0.1.0 (C, its \`bzip2 -dc\`); libbzip2 (the corpora's encoder) 1.0.8 (the bzip2 crate's bundled source); libsais (the kernels' only) 2.12.1 at c284ba8 (C: -O3 -march=native, single-threaded; bench/libsais.sh) - bench 3.8.2 (bench/report.py), 10 rounds, threads 1, 2, 4
