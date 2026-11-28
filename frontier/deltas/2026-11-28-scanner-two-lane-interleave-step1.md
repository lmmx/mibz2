claim: none
by: antigravity
outcome: partial
evidence: docs/notebook/2026-11-28-scanner-two-lane-interleave-step1.md; local prototype commit 82ddf946 (off dcda47f6, unpushed); Haswell-EP VM (Model 63, 32 KiB L1d, 2 load ports), 17 interleaved rotated repeats default & JCC-32B, 7/7 symbol- & bit-exact parity; objdump -d of decode_pair_step_bmi2 (0 critical spills, 5x unrolled); Miri 3/3 0 UB (`cargo +nightly miri test --lib -- stream::tests`); `cargo clippy --all-targets -- -D warnings` clean

**What was done:** executed Step 1 (paid A2+A3 discriminating probe, inline vs helper-thread finder, default & JCC-32B, plus rescue optimisation and a bounded `B = 9` 32 KiB L1d experiment) and Step 2 correctness verification for `scanner-two-lane-interleave` locally without taking a hold or editing `src/` on master.

**What we found (32 KiB L1d Haswell-EP VM, NOT i9 evidence):**
- **Codegen:** v4 (`82ddf946`) eliminated all critical-chain spills in `decode_pair_step_bmi2` (`abuf=%r9, alive=%edi, luta=%rsi, ba=%r11, bbuf=%r15, blive=%r14d, lutb=%rbp`, 5x unrolled, 0 stack stores, 1 off-path stack read `bb`) in both default and JCC-32B builds, with a 100%-safe 2 KiB branchless `M0 & M1` 4-byte-stride `BLOCK_MAGIC` finder (3.2–5.1 GB/s).
- **Inline finder ruled out:** adds `+3.0%` to `+6.6%` paid A2+A3 CPU (`wikidata 0.949x -> 0.995x`, `records 1.027x -> 1.077x`). A helper/feeder-thread finder is strictly required.
- **`B = 10` (`48 KiB > 32 KiB L1d`, `HelperThread` thread CPU):** `wikidata` `0.9492x` (default) / `0.9504x` (JCC-32B), `text` `1.0127x` / `1.0252x`, `random` `0.9291x` / `0.9386x`, `records` `1.0270x` / `0.9888x`, `onebyte` `0.9973x` / `0.9917x`. Misses frozen Step 1 (`wikidata <= 0.92x`, `text <= 0.94x`) on this 32 KiB L1d core.
- **`B = 9` (`24 KiB + 4 KiB chunks = 28 KiB < 32 KiB L1d`, `HelperThread` thread CPU):** decisively confirms Notebook Risk #1 (32 KiB L1d thrashing) on uniform-length corpora where `B = 9` loses zero `K = 2` pairs (`records` `-4.7` to `-8.4 pp` to `0.9434x` / `0.9416x`, `random` `-4.4` to `-5.9 pp` to `0.8847x` / `0.8801x`), but hits a Pareto trap on skewed corpora (`wikidata` `+7.8%` lookups & `1.24% -> 1.89%` slow -> `0.9880x` / `0.9903x`; `text` `+4.6%` lookups & `1.83% -> 3.30%` slow -> `1.0632x` / `1.0706x`; `runs` `24x` slow `0.32% -> 7.67%` -> `1.4820x` / `1.4709x`).
- **Step 2 correctness:** 7/7 corpora exact parity, synthetic planted false-positive `0x314159265359` discarded cleanly, `libbzip2`-parity error attribution with lane B discard, zero new `unsafe` outside BMI2 dispatch, Miri 3/3 with 0 UB, `fmt` and `clippy --all-targets -- -D warnings` clean.

**What changed:** added `docs/notebook/2026-11-28-scanner-two-lane-interleave-step1.md` and this delta only. `scanner-two-lane-interleave` remains `status = "idea"` with no hold taken.

**What's next:** before spending i9 queue time on `scanner-two-lane-interleave`, either close it under Step 1's 32 KiB L1d trap or test the remaining unpriced lever (rebase `82ddf946` onto post-`#1117` master + inline slow-path without 5-step batch abort + per-block `B = 9` only when `min_len >= 6` where `K = 2` pairs are impossible anyway).
