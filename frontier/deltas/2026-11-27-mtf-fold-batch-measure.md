claim: mtf-fold-batch-measure
by: antigravity
outcome: done
evidence: src/open/outq.rs; examples/mtf_a6a_dissection.rs; corpora/wikidata.bz2 (SHA-256 82f65ace…, 233/233 Open blocks, crc 0xa78fe5d8); ab gen guards (text, runs, onebyte, random, records); objdump -dC (baseline, -C target-cpu=native, JCC-32B native); n=31 interleaved alternating ON/OFF pinned CPU 0 (CLOCK_THREAD_CPUTIME_ID + schedstat); cargo +nightly miri (miri_open_kernel_small_columns + 3/3 miri_fold)

**What was asked:** Execute the frozen C1a `stop_test` (`frontier/deltas/2026-11-27-mtf-fold-batch-measure-registration.md`, PR #1221): (a) clear the 11 native `vzeroupper` in `QueuesOf<1008, 1024>::push_slow` without `unsafe` (`#![forbid(unsafe_code)]`), verify `0` `vzeroupper` across baseline, `-C target-cpu=native`, and JCC-32B-aligned native, and confirm bit/CRC exactness + Miri 0 UB; (b) run `n >= 31` interleaved alternating ON/OFF steal-free worker-CPU measurements on `corpora/wikidata.bz2` (all 233 Open blocks) and guard corpora (`text`, `runs`, `onebyte`, `random`, `records`) across all three builds; judge honestly under C2.

**What I changed:**
- **`src/open/outq.rs`** (`#![forbid(unsafe_code)]`, zero `unsafe`): capped each `QueuesOf::push_slow` outer step to `take <= 4` ranks (8 bytes) written via a straight-line 4-step unroll over `chunks_exact_mut(2)` (no inner loop), preventing LLVM auto-vectorization to 256-bit YMM and eliminating all `vzeroupper` under `-C target-cpu=native` and JCC-32B native.
- **`examples/mtf_a6a_dissection.rs`**: added steal-free `wkr_cpu_ms` (`CLOCK_THREAD_CPUTIME_ID`), `wkr_sched_ms` (`/proc/thread-self/schedstat`), `OpenPass1End` tracking (`A5+A6a_ms` and `A6bc+A7_ms`), exact Student-t 95% paired geometric-mean confidence intervals, and `--e2e-only` / `--workers W` flags.

**What I found:**
- **Gate (a) Codegen & soundness — PASS:**
  - `objdump -dC` across all 3 builds (`--features mtf-fold,trace`):
    - Baseline x86-64: `QueuesOf<1008, 1024>::push_slow` 314 insns (`0 vzeroupper`, `0 ymm`), `QueuesOf<512, 512>::push_slow` 299 insns (`0 vzeroupper`, `0 ymm`), `fill_batch_avx2` 267 insns (`0 vzeroupper`, `0 ymm`).
    - `-C target-cpu=native`: `QueuesOf<1008, 1024>::push_slow` 299 insns (`0 vzeroupper`, `0 ymm`), `QueuesOf<512, 512>::push_slow` 299 insns (`0 vzeroupper`, `0 ymm`), `fill_batch_avx2` 269 insns (`0 vzeroupper`, `0 ymm`).
    - JCC-32B native (`-C target-cpu=native -C llvm-args=-x86-branches-within-32B-boundaries`): `QueuesOf<1008, 1024>::push_slow` 327 insns (`0 vzeroupper`, `0 ymm`), `QueuesOf<512, 512>::push_slow` 312 insns (`0 vzeroupper`, `0 ymm`), `fill_batch_avx2` 274 insns (`0 vzeroupper`, `0 ymm`).
  - Checks: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` (default, `mtf-fold`, `mtf-fold,trace`, `forward-anchor-lanes`, `trace`), release `open::` + `runs::tests::fold_tests::` (45/45) + `tests/mtf_fold.rs` (1/1), and `cargo +nightly miri test --lib --features mtf-fold` (`open::b5::miri_open_kernel_small_columns` + 3/3 `miri_fold_*`, 0 UB).
  - Exactness: 100% length- and CRC-exact across Wikidata (`209,700,196 B`, `0xa78fe5d8`, `233/233` Open blocks folded, `0` bailed, `0` not_open) and all 5 guard corpora on all 3 builds.
- **Gate (b) Primary Wikidata (`n = 31` interleaved ON/OFF, pinned CPU 0, 1 worker) — PASS on all 3 builds:**
  - **Baseline x86-64:** `wkr_cpu_ms` OFF `771.29` -> ON `747.24 ms`, **`gm = 0.9662x` 95% CI `[0.9612, 0.9713]` (`-3.38%`, hi `< 0.980`)**; `wkr_sched_ms` `0.9663x [0.9613, 0.9713]`; `A5+A6a_ms` `0.8679x [0.8404, 0.8964]` (`-13.21%`); `A5+A6+A7_ms` `0.9088x [0.8736, 0.9455]` (`-9.12%`).
  - **`-C target-cpu=native`:** `wkr_cpu_ms` OFF `764.48` -> ON `703.89 ms`, **`gm = 0.9242x` 95% CI `[0.9183, 0.9302]` (`-7.58%`, hi `< 0.980`)**; `wkr_sched_ms` `0.9242x [0.9183, 0.9302]`; `A5+A6a_ms` `0.7503x [0.7229, 0.7788]` (`-24.97%`); `A5+A6+A7_ms` `0.8904x [0.8387, 0.9454]` (`-10.96%`).
  - **JCC-32B native:** `wkr_cpu_ms` OFF `700.05` -> ON `638.69 ms`, **`gm = 0.9224x` 95% CI `[0.9129, 0.9319]` (`-7.76%`, hi `< 0.960`)**; `wkr_sched_ms` `0.9224x [0.9130, 0.9319]`; `A5+A6a_ms` `0.7186x [0.6688, 0.7722]` (`-28.14%`); `A5+A6+A7_ms` `0.8479x [0.7946, 0.9048]` (`-15.21%`).
- **Gate (b) Guard corpora (pinned CPU 0, 1 worker, `wkr_cpu_ms`, `<= 1.02x` required) — FAIL on `text` and `runs`:**
  - **`text`** (`crc 0x9f1887ed`, `folded=0, bailed=0, not_open=28`): **FAIL** on all 3 builds — baseline (`n=15`) `1.4743x [1.3594, 1.5990]` (`+47.43%`), native (`n=15`) `1.3716x [1.3460, 1.3977]` (`+37.16%`), JCC-32B (`n=31`) `1.3584x [1.3236, 1.3941]` (`+35.84%`).
  - **`runs`** (`crc 0xb079fdaa`, `folded=1, bailed=0, not_open=0`): **FAIL** on `wkr_cpu_ms` — baseline (`n=15`) `1.0814x [0.8267, 1.4144]` (`+8.14%`, `A5+A6a -26.89%`), native (`n=15`) `1.2740x [0.8761, 1.8527]` (`+27.40%`, `A5+A6a -6.72%`), JCC-32B (`n=31`) `1.3193x [1.0107, 1.7222]` (`+31.93%`, `A5+A6a -17.88%`, `A5+A6+A7 -26.77%`, `A9 +51.04%`).
  - `onebyte` (`0x5cb40057`, `0/0`): PASS (`1.0058x` base, `1.0016x` native, `1.0011x` JCC-32B).
  - `random` (`0xf4abcbc0`, `0/0`): PASS (`1.0063x` base, `1.0042x` native, `0.9981x` JCC-32B).
  - `records` (`0xcc7c90f7`, `0/0`): PASS (`0.9942x` base, `0.9961x` native, `0.9993x` JCC-32B).

**Root cause of the guard failures:**
1. **`text` (`+35.8..+47.4%`, 28/28 `not_open`):** `runs::fold::refused(max_len)` (`src/runs/fold.rs:152`) only cuts off before Pass 2 at `lits >= 7/25 * max_len` (`28%`). All 28 full `text` blocks sit below `28%` in literals yet exceed `OpenCells::cap`'s `f.m > cap.m` after Pass 2; consequently `OpenCellsOf::fold` runs a complete Pass 1 + Pass 2 + `OutQ16` write + `edges` build, throws all of it away on `Outcome::NotOpen`, and replays shipped `runs::front` + `LazyCells` from scratch.
2. **`runs` (`+8.1..+31.9%` `wkr_cpu_ms` despite `-17.9..-26.9%` `A5+A6a`):** `runs.bz2` expands 1 block to 20 MiB in `A9` (`rle::unrle_in_place`). `decode_folded` sizes `s.ll` to `bound + bound/16 + 64` and swaps `out` and `s.ll` (`src/lib.rs:576-582`), altering `unrle_in_place` staging and zero-page fault behavior on the 20 MiB expansion (`A9` `+51.04%` on JCC-32B).

**Verdict (C2, nothing relaxed):** **STOP** at the frozen guard-corpus gate (`text` `+35.8..+47.4%`, `runs` `+8.1..+31.9%` > `1.02x`). Scoped row added to `frontier/dead-ends.tsv`. No i9 run (`mtf-fold-run`) is queued from this claim. The safe `src/open/outq.rs` zero-`vzeroupper` fix and the powered `examples/mtf_a6a_dissection.rs` harness are merged so both shipped and experimental paths benefit and future work builds on clean codegen.

**What's next (research persistence):** The Wikidata advantage is now established at high statistical power (`-3.38%` base / `-7.58%` native / `-7.76%` JCC-32B worker CPU, `-13.2..-28.1%` `A5+A6a`, `0` `vzeroupper` across all builds), and both guard regressions have clear, independent fixes: (1) early `not_open` refusal aligned with `post-1080-open-admission`'s `r/n < 0.15` / `m <= 0.22n` two-tier rule (and/or early `m`-cap abort inside `FoldSink::batch` before completing Pass 2) so `text` never pays a wasted Pass 2; (2) avoiding the `s.ll` / `out` staging penalty on 20 MiB RLE1 expansion in `decode_folded`. Once those guard regressions are eliminated in a follow-on claim, `mtf-fold` is ready for the i9 kill test (`mtf-fold-run`).
