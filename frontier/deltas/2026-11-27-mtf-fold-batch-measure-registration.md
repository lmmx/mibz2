claim: mtf-fold-batch-measure
by: antigravity
outcome: partial
evidence: frontier/claims/mtf-fold-batch-measure.toml; docs/journal/2026-11-24-algorithm-level/mtf-fold/registration.md

Admitting `mtf-fold-batch-measure` (`idea` -> `open`) and freezing its C1a `stop_test` strictly BEFORE any new build or measurement data is collected (never reusing PR #1132's gate data and never relaxing `mtf-fold/registration.md` §3 K1–K4):

- **(a) Codegen & soundness:** `objdump -d` confirms `0` `vzeroupper` in `QueuesOf<1008, 1024>::push_slow` and `fill_batch_avx2` under baseline x86-64, `-C target-cpu=native`, and JCC-32B-aligned native (`-C target-cpu=native -C llvm-args=-x86-branches-within-32B-boundaries`); `#![forbid(unsafe_code)]` preserved in `src/open/outq.rs` (zero new `unsafe`); 100% bit- and CRC-exact parity across all 233 Wikidata Open blocks and guard corpora; and Miri PASS (`0` UB).
- **(b) Powered paired worker measurement:** `n >= 31` interleaved alternating ON/OFF repeats of steal-free worker CPU (`CLOCK_THREAD_CPUTIME_ID` / `/proc/thread-self/schedstat` on pinned CPU 0) on `corpora/wikidata.bz2` (all 233 Open blocks) under baseline x86-64, `-C target-cpu=native`, and JCC-32B-aligned native.
- **Frozen PASS / STOP verdict rule (C2):** **PASS** iff (a) holds AND the paired geometric-mean 95% CI upper bound for `wkr_cpu_ms` (and joint `A5+A6a+A6c`) is strictly `< 0.960` (`>= 4%` worker speedup) on JCC-32B native AND `< 0.980` on native and baseline, with zero guard regressions (`text`, `runs`, `onebyte`, `random`, `records` all `<= 1.02x`). Otherwise **STOP** with a scoped `frontier/dead-ends.tsv` row.
