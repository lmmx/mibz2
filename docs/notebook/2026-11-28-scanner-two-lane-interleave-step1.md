# Two-lane scanner Step 1 probe: 32 KiB L1d pricing (`B = 10` and `B = 9`)

*Lab notebook, 28 Nov 2026. Local prototype branch `build/scanner-two-lane-interleave` at `82ddf946` (based on `dcda47f6`, pre-`#1117` A2 fast build), evaluated before taking a hold on `scanner-two-lane-interleave`. VM numbers only — NOT i9-10900K evidence.*

## Machine & measurement setup

- **CPU:** `Intel(R) Xeon(R) CPU @ 2.30GHz`, Family 6 Model 63 Stepping 0 (Haswell-EP class: 2 vCPUs, **32 KiB L1d**, 256 KiB L2, 45 MiB L3, 192-entry ROB, 4-wide dispatch, 2 load ports, AVX2 + BMI2).
- **Why this CPU is informative vs `#1251`:** the explore probe (`docs/notebook/2026-11-27-explore-two-lane-scan.md`) ran on an Emerald Rapids-class VM (**48 KiB L1d**, 512-entry ROB, 6-wide dispatch, 3 load ports) and explicitly flagged **Risk #1 (32 KiB L1d)** because Comet Lake (`i9-10900K`) has a **32 KiB L1d** and **2 load ports** (224-entry ROB, 4-wide dispatch).
- **Build & method:** `rustc` `opt-level=3`, `codegen-units=16`, `debuginfo=line-tables-only`, no debug assertions, in both **default** and **JCC-32B** (`-C llvm-args=-x86-align-branch-boundary=32 -C llvm-args=-x86-align-branch=jcc+fused+jmp`) builds. 17 interleaved rotated repeats (`[0,1,2]`, `[1,2,0]`, `[2,0,1]`) comparing production single-lane `Scanner` (`B = 10`) against `TwoLaneScanner` with `FinderMode::Inline` and `FinderMode::HelperThread` on paid A2 + A3 across `wikidata.bz2` (SHA-256 verified, 11,854,933 B, 431 blocks) and all six `ab gen` corpora (`text`, `runs`, `random`, `records`, `onebyte`, `pattern`).
- **Preemption control:** concurrent builds on the shared 2-vCPU VM kept load average at `12–18`, inflating wall-clock (`Instant::now()`) variance. Every run therefore recorded **both** wall-clock (`w_*`) and **`CLOCK_THREAD_CPUTIME_ID`** (`c_*`, reproducible within ~0.5% across runs; `c_hlp` isolates scanner-thread CPU excluding the helper thread, matching the target architecture where a feeder thread runs the finder off the scanner's critical path).

## Implementation & `objdump -d` register-residency audit

The `Scanner`-shaped `TwoLaneScanner` (`src/huff.rs`, `src/block.rs`, `src/stream.rs`) ports the rolling two-lane schedule with:

1. **Safe 4-byte-stride `BLOCK_MAGIC` finder (`find_block_magics`):** 2 KiB L1d branchless bitmask tables (`BLOCK_MAGIC_M0_M1`, `M0[b0] & M1[b1]`, 0.041%–0.066% hit rate across corpora, 4×16-byte unrolled loop, zero `unsafe`). Measured throughput: **3.2–5.1 GB/s** (`+3.1–3.7 ms` on `wikidata`, `+2.4–3.6 ms` on `records`).
2. **Speculative verification & error attribution:** lane B's start bit is verified at lane A's EOB (`a_end_bit == b_start_bit` and matching stream `level`); false positives and speculative errors are silently discarded; promotion swaps `scratch_a`/`scratch_b` in `O(1)`; inputs `< 64 KiB` delegate directly to single-lane `Scanner`.
3. **Codegen rescue (v1 → v4) verified by `objdump -d`:**
   - **v1** inlined `finish_slice` and `Vec::extend_from_slice` into `decode_pair_step_bmi2` (472-byte stack frame), spilling `ra.buf`/`ra.live` (`0x50(%rsp)`/`0x58(%rsp)`) onto the critical shift-load-shift chain (`wikidata` `1.046×`, `records` `1.213×`), reproducing the notebook's spill warning.
   - **v4 (`82ddf946`)** hoisted `decode_entry_slow` out of the 5-step fast loop, outlined `finish_slice_bmi2` and `flush_chunk`, and eliminated the per-selector `LockstepRegs` boundary. In **both** default and JCC-32B `objdump -d` of `decode_pair_step_bmi2`:
     - All 5 steps are fully unrolled with zero `and $0xf` masks and **zero stack stores** in the 5-step span.
     - Both critical shift-load-shift chains are 100% register-resident: Lane A (`abuf = %r9`, `alive = %edi`, `luta = %rsi`, `ba = %r11`), Lane B (`bbuf = %r15`, `blive = %r14d`, `lutb = %rbp`). Only 1 off-critical-path stack read (`bb` at `0x70(%rsp)`) remains per paired step.
     - JCC-32B branch alignment confirmed (`xchg %ax,%ax` padding before `je` at 32-byte boundaries).

## Experiment 1: `B = 10` (`4 KiB/table`, `12 × 4 KiB = 48 KiB > 32 KiB L1d`)

Corpora: `wikidata.bz2` (431 blocks) + `ab gen` scale 1 (`text` 2.23 MB / 14 blks, `runs` 38 KB / 1 blk, `random` 7.10 MB / 9 blks, `records` 9.59 MB / 11 blks, `onebyte` 51 B / 1 blk, `pattern` 1.1 KB / 22 blks). 17 interleaved repeats; 7/7 exact parity.

### Default build (`B = 10`, `82ddf946`)

| corpus | blks | `w_in/sgl` | `w_hp/sgl` | `c_sgl_ms` | `c_in_ms` | `c_hlp_ms` | `c_in/sgl` | `c_hlp/sgl` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `wikidata` | 431 | 1.0431 | 0.9803 | 77.357 | 77.594 | 74.236 | 0.9947 | **0.9492** |
| `text` | 14 | 1.0495 | 1.0951 | 15.883 | 16.587 | 16.098 | 1.0460 | 1.0127 |
| `runs` | 1 | 1.1682 | 1.0468 | 0.268 | 0.278 | 0.280 | 1.0418 | 1.0377 |
| `random` | 9 | 0.9869 | 0.9751 | 35.304 | 34.392 | 32.802 | 0.9743 | **0.9291** |
| `records` | 11 | 1.0953 | 1.0690 | 44.274 | 47.885 | 45.505 | 1.0772 | 1.0270 |
| `onebyte` | 1 | 1.0038 | 0.9971 | 0.004 | 0.004 | 0.004 | 1.0020 | 0.9973 |
| `pattern` | 22 | 1.1577 | 0.9928 | 0.024 | 0.024 | 0.024 | 1.0137 | 1.0150 |

*(Repeat run before the small-input fast-path refactor gave tightly matching thread CPU: `wikidata` `0.9460×`, `text` `1.0038×`, `random` `0.9262×`, `records` `1.0131×`.)*

### JCC-32B build (`B = 10`, `82ddf946`)

| corpus | blks | `w_in/sgl` | `w_hp/sgl` | `c_sgl_ms` | `c_in_ms` | `c_hlp_ms` | `c_in/sgl` | `c_hlp/sgl` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `wikidata` | 431 | 0.9238 | 1.0163 | 75.113 | 73.928 | 71.474 | 0.9907 | **0.9504** |
| `text` | 14 | 1.0809 | 1.0968 | 16.546 | 17.249 | 17.537 | 1.0500 | 1.0252 |
| `runs` | 1 | 1.1343 | 0.9967 | 0.244 | 0.254 | 0.264 | 1.0439 | 1.0339 |
| `random` | 9 | 1.0111 | 1.0523 | 35.025 | 35.513 | 35.440 | 0.9772 | **0.9386** |
| `records` | 11 | 1.0631 | 1.0050 | 47.371 | 51.414 | 47.877 | 1.0659 | **0.9888** |
| `onebyte` | 1 | 0.9500 | 0.9540 | 0.004 | 0.004 | 0.004 | 0.9745 | 0.9917 |
| `pattern` | 22 | 0.9827 | 0.9187 | 0.025 | 0.025 | 0.024 | 0.9924 | 0.9853 |

**Readings from `B = 10`:**

- **Inline finder is ruled out:** even at 3.2–5.1 GB/s, inline scanning adds `+3.0%` to `+6.6%` paid CPU, erasing the `wikidata` gain (`0.949× → 0.995×`) and pushing `records` to `1.066–1.077×`. A helper/feeder-thread finder is strictly required.
- **Even with helper-thread finder, `B = 10` misses Step 1 on 32 KiB L1d:** `wikidata` reaches `0.949–0.950×` (short of `<= 0.92×` by ~3.0 pp) and `text` sits at `1.013–1.025×` (short of `<= 0.94×` by ~7–8 pp). Selector inspection explains the corpus split: `text` and `records` rotate all 6 Huffman tables every 64.6–98.2 symbols (`12 × 4 KiB = 48 KiB` of root LUTs + `4 KiB` chunks = `52 KiB > 32 KiB L1d`).

## Experiment 2: `B = 9` (`2 KiB/table`, `12 × 2 KiB + 4 KiB chunks = 28 KiB < 32 KiB L1d`)

To test whether fitting both blocks' root LUTs inside 32 KiB L1d rescues Step 1, we switched ONLY the two-lane path (`build_into_b9`, `decode_tail_b9`, `decode_entry_slow_b9`, keeping `K = 2` pairs inside `B = 9`) while leaving production single-lane `Scanner` at `B = 10`. Corpora: `wikidata.bz2` (431 blks) + `ab gen` default scale 2 (`text` 4.52 MB / 28 blks, `runs` 75 KB / 1 blk, `random` 14.19 MB / 18 blks, `records` 19.18 MB / 22 blks, `onebyte` 51 B / 1 blk, `pattern` 2.2 KB / 44 blks). 17 repeats; 7/7 exact parity.

### Dynamic lookup & slow-path rates (`B = 10` vs `B = 9`, exact counts over all blocks)

| corpus | total syms | `B=10` lookups | `B=10` sym/lk | `B=10` slow% | `B=9` lookups | `B=9` sym/lk | `B=9` slow% | lookup ratio (`B9/B10`) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `wikidata` | 24,979,299 | 13,710,961 | 1.822 | 1.24% | 14,777,447 | 1.690 | 1.89% | **1.0778** (+7.8%) |
| `text` | 9,955,959 | 5,593,533 | 1.780 | 1.83% | 5,852,598 | 1.701 | **3.30%** | **1.0463** (+4.6%) |
| `records` | 19,199,860 | 19,199,860 | 1.000 | 0.88% | 19,199,860 | 1.000 | 1.02% | 1.0000 |
| `random` | 15,999,077 | 15,998,878 | 1.000 | 0.35% | 15,999,077 | 1.000 | 0.60% | 1.0000 |
| `runs` | 124,190 | 91,241 | 1.361 | 0.32% | 91,685 | 1.355 | **7.67%** | 1.0049 (24× slow) |

### Default build (`B = 9` two-lane vs `B = 10` single-lane)

| corpus | blks | `w_in/sgl` | `w_hp/sgl` | `c_sgl_ms` | `c_in_ms` | `c_hlp_ms` | `c_in/sgl` | `c_hlp/sgl` | `hp_wins` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `wikidata` | 431 | 1.0078 | 1.0045 | 76.384 | 79.310 | 75.470 | 1.0386 | 0.9880 | 16/17 |
| `text` | 28 | 1.0889 | 1.0353 | 32.909 | 36.052 | 34.940 | 1.0943 | 1.0632 | 0/17 |
| `runs` | 1 | 1.5539 | 1.6422 | 0.451 | 0.665 | 0.665 | 1.4792 | 1.4820 | 0/17 |
| `random` | 18 | 0.9454 | 0.9085 | 71.270 | 67.894 | 63.335 | 0.9516 | **0.8847** | 17/17 |
| `records` | 22 | 1.0524 | 0.9475 | 89.111 | 89.285 | 84.071 | 1.0018 | **0.9434** | 16/17 |
| `onebyte` | 1 | 0.9884 | 0.9883 | 0.004 | 0.004 | 0.004 | 0.9886 | 0.9903 | 12/17 |
| `pattern` | 44 | 1.0040 | 0.9943 | 0.044 | 0.044 | 0.044 | 1.0041 | 0.9998 | 9/17 |

### JCC-32B build (`B = 9` two-lane vs `B = 10` single-lane)

| corpus | blks | `w_in/sgl` | `w_hp/sgl` | `c_sgl_ms` | `c_in_ms` | `c_hlp_ms` | `c_in/sgl` | `c_hlp/sgl` | `hp_wins` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `wikidata` | 431 | 1.0322 | 0.9741 | 72.804 | 75.465 | 72.367 | 1.0363 | 0.9903 | 13/17 |
| `text` | 28 | 1.1407 | 1.1123 | 30.943 | 34.315 | 33.238 | 1.1060 | 1.0706 | 0/17 |
| `runs` | 1 | 1.5589 | 1.4491 | 0.430 | 0.634 | 0.633 | 1.4680 | 1.4709 | 0/17 |
| `random` | 18 | 0.9694 | 0.9390 | 69.054 | 64.401 | 60.837 | 0.9327 | **0.8801** | 17/17 |
| `records` | 22 | 1.0149 | 0.9366 | 87.227 | 88.537 | 82.228 | 1.0092 | **0.9416** | 15/17 |
| `onebyte` | 1 | 1.0104 | 1.0127 | 0.004 | 0.004 | 0.004 | 1.0104 | 1.0268 | 5/17 |
| `pattern` | 44 | 1.0072 | 0.9593 | 0.045 | 0.045 | 0.046 | 0.9899 | 1.0105 | 8/17 |

### The 32 KiB L1d Pareto trap

`B = 9` cleanly isolates and proves the 32 KiB L1d mechanism:

1. **On uniform-length corpora (`records`, `random`, where `sym/lk = 1.000` so `B = 9` loses zero pairs and slow% stays `<= 1.02%`), fitting `28 KiB < 32 KiB L1d` delivers a huge win:** `records` drops **-4.7 to -8.4 pp** (`1.027× → 0.943×` default, `0.989× → 0.942×` JCC-32B) and `random` drops **-4.4 to -5.9 pp** (`0.929× → 0.885×` default, `0.939× → 0.880×` JCC-32B).
2. **On skewed real-world corpora (`wikidata`, `text`, `runs`), `B = 9` pays more in lost pairs and `> 9`-bit slow exits than L1d recovers:** `wikidata` pays `+7.8%` extra lookups (`1.822 → 1.690 sym/lk`) and `+52%` slow exits (`1.24% → 1.89%`), regressing `0.949× → 0.988×`; `text` pays `+4.6%` lookups and nearly double the slow exits (`1.83% → 3.30%`, aborting `1 - 0.967^10 = 28.5%` of 5-step lockstep batches early), regressing `1.013× → 1.063×`; `runs` explodes slow exits `24×` (`0.32% → 7.67%`), regressing to `1.48×`.

## Step 2 correctness verification (`82ddf946`)

- **7-corpus parity:** 100% symbol- and bit-exact (`Inline` and `HelperThread` vs `Scanner`) on `wikidata`, `text`, `runs`, `random`, `records`, `onebyte`, `pattern` in both default and JCC-32B builds (both `B = 10` and `B = 9`).
- **Unit tests (`block::` 5/5, `stream::` 3/3, `bits::` 2/2):** all pass, including:
  - `finder_matches_naive_at_all_shifts_and_boundaries` (all 8 bit shifts + boundary sizes);
  - `two_lane_matches_single_on_multistream_and_planted_false_magic` (synthetic stream with `0x314159265359` planted inside compressed payload bits — verified and silently discarded);
  - `two_lane_matches_libbzip2_error_attribution_with_speculative_discard` (truncated, bad selector, bad Huffman code attributed to the exact block/bit with lane B discarded).
- **Safety, Miri, lint:** zero new `unsafe` outside the existing `#[target_feature(enable = "bmi2")]` dispatch boundary in `src/block.rs` (`src/huff.rs` and `src/stream.rs` retain `#![forbid(unsafe_code)]`). `cargo +nightly miri test --lib -- stream::tests` passed **3/3 with 0 UB**. `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` both pass cleanly.

## Verdict & what stays open

Under `scanner-two-lane-interleave`'s frozen Step 1 stop test (`wikidata <= 0.92×`, `text <= 0.94×`, no guard `> 1.02×` in both default and JCC-32B builds), **Step 1 MISSES on this 32 KiB L1d Haswell-EP VM** at both `B = 10` (`wikidata 0.949–0.950×`, `text 1.013–1.025×`) and `B = 9` (`wikidata 0.988–0.990×`, `text 1.063–1.071×`, `runs 1.47–1.48×`). The `B = 9` scratch experiment was reverted; `82ddf946` (`B = 10`) is preserved locally, and `scanner-two-lane-interleave` remains at `status = "idea"` with no hold taken and no dead-ends row filed yet.

Three concrete facts remain if the claim is picked up later:

1. **Post-`#1117` A2 share:** `#1117` (`scanner-a2-fast-build`) landed on `master` after `dcda47f6`, shrinking A2 and increasing A3's share of paid S.
2. **Comet Lake vs Haswell-EP:** i9-10900K shares the 32 KiB L1d and 2 load ports (so the L1d Pareto trap directly applies), but has a larger ROB (224 vs 192 entries) and higher IPC, which could shift `B = 10` `wikidata` slightly closer to `0.92×`, though `text` at `1.01–1.025×` leaves a 7–8 pp gap.
3. **Unexplored lever:** keeping `B = 10` while decoding rare `> 10`-bit slow entries inline inside the 5-step batch (instead of aborting the remaining steps of both lanes) or choosing `B = 9` per-block only when `min_len >= 6` (zero `K = 2` pairs possible anyway, like `records` and `random`, where `B = 9` gained `-4.4` to `-8.4 pp` for free).
