# Wormhole Cycle 52 (Neon) — Theorem 52.1: Skip-Free Zero-Shift In-Place `RLE1` (`-46.6%` `copy_within`, `0.894×` RLE+CRC) & Feeder Chunk Audit

**Date:** 2026-11-22  
**Branch:** `research/carbon-wormhole`  
**Baseline:** `origin/master` `3296195e3` (`#739`, [`src/rle.rs:103-170`](../../src/rle.rs), [`examples/bzcat.rs:27-35`](../../examples/bzcat.rs))  
**Artifacts:** [`c52_rle_zero_shift_ab.rs`](2026-11-22-wormhole-cycle52/c52_rle_zero_shift_ab.rs), [`c52_results.txt`](2026-11-22-wormhole-cycle52/c52_results.txt), [`c52_rle_zero_shift.toml`](2026-11-22-wormhole-cycle52/c52_rle_zero_shift.toml)

---

## 1. Theorem 52.1 — Skip-Free Zero-Shift In-Place `RLE1` Decomposition

In `#739` ([`src/rle.rs:135-139`](../../src/rle.rs)), whenever a block has positive peak growth `shift > 0` (`195/234` full Wikidata blocks = `83.3%`), `unrle_in_place` shifts the entire suffix `buf[first..n]` right by `shift` (`copy_within(first..n, first + shift)`), and then copies every literal back left in a second pass — moving **`350.78 MB`** (`1.67 B/B` = `3.34 B/B` R+W cache traffic!) on the `209.8 MB` Wikidata corpus, and falling back to staging in `s.ll` whenever `shift > n`.

**Theorem 52.1 (Zero-Shift Valley/Hill Decomposition).**  
Let $g(k) = \sum_{i < k} (\mathrm{extra}_i - 1)$ be the cumulative output-minus-input offset before run $k$. Because every bzip2 RLE1 run satisfies $\mathrm{extra}_k \in [0, 255]$, the step $\Delta_k = \mathrm{extra}_k - 1 \ge -1$ is **skip-free downwards**. Consequently:
1. Every maximal **valley** ($g \le 0$) satisfies $\mathrm{dst} \le \mathrm{src}$ and expands **left-to-right** in one pass without overwriting unread input.
2. Every maximal **hill** ($g > 0$) starts at a rising run $k_0$ ($g(k_0) \le 0 < g(k_0+1)$) and either reaches EOF ($n$) or terminates at a falling run $k_1$ where **necessarily $g(k_1) = +1$, $\mathrm{extra}_{k_1} = 0$, and $g(k_1+1) = 0$**. Run $k_1$'s output occupies $[j_{k_1}+1, j_{k_1}+5)$ — ending **exactly** at $j_{k_1}+5$, the start of the next segment, with zero overlap! Because every literal inside the hill has $g \ge +1 > 0$ ($\mathrm{dst} > \mathrm{src}$) and all run headers $(j, c, \mathrm{extra})$ are already in `runs`, walking the hill **right-to-left** expands it in one pass with **zero pre-shift copy** and **zero `s.ll` staging fallback** (even when `shift > n`!).

---

## 2. Native Safe-Rust Verification & A/B Results (`#![forbid(unsafe_code)]`)

### 2a. Exactness & `copy_within` Traffic Census
- **Adversarial Stress Gate:** **`5,000 / 5,000`** randomized blocks (mixing `extra = 0`, `extra = 1`, small hills, and `extra = 255` hills with `shift > n`) match scalar reference **byte-for-byte and CRC-for-CRC**.
- **`431`-Block Authenticated Wikidata Parity:** **`431 / 431`** exact bytes + stored CRC; total `copy_within` bytes drop from **`350.78 MB`** (`#739`) to **`187.35 MB`** (**`-46.6%`**, **`-163.43 MB`** eliminated = **`-1.56 B/B`** L2/L3 R+W traffic!).

### 2b. Part 1 — `15`-Round Rotated `1T` Micro A/B (In-Place RLE1 Expand + `64 KiB` Tiled CLMUL CRC, `210 MB`)

| Arm | `med_ms` | `vs R739_A [95% CI]` | `copy_within` (`MB`) | Verdict |
|---|---:|---:|---:|---|
| `R739_A` (`src/rle.rs:103-170`) | `33.20` | `1.0000 [1.0000, 1.0000]` | `350.78` | Baseline |
| `R739_B` (A/A control) | `33.14` | `1.0025 [0.9921, 1.0130]` | `350.78` | Clean A/A |
| **`ZeroShift52` (Theorem 52.1)** | **`29.62`** | **`0.8944 [0.8850, 0.9039]`** | **`187.35` (`-46.6%`)** | **PROMOTED** |

### 2c. Part 2 — `12`-Round Fresh-Process `4W` Feeder Chunk & Zero-Fill A/B on `3296195e3` (`210 MB`)

| Arm (`examples/bzcat.rs:27-35` model) | `med_ms` | `vs 739_8M [95% CI]` | `sys_ms` | `minflt` | `rss_MB` | Verdict |
|---|---:|---:|---:|---:|---:|---|
| `bzcat_739_8M_z` (`8 MiB` + `resize(8M,0)`) | `333.29` | `1.0000 [1.0000, 1.0000]` | `20.10` | `9,715` | `54.23` | Baseline |
| `bzcat_8M_noz` (`8 MiB` no zero-fill) | `330.58` | `0.9985 [0.9744, 1.0233]` | `23.19` | `9,629` | `53.71` | **KILLED** |
| `bzcat_1M_pool` (`1 MiB` chunk size) | `328.56` | `0.9962 [0.9786, 1.0140]` | **`17.93` (`-10.8%`)** | **`8,629` (`-11.2%`)** | **`48.88` (`-9.9%`)** | **KEEP (RSS hygiene only)** |

---

## 3. Kill / Keep Decisions

- **PROMOTED `ZeroShift52` (Theorem 52.1):** Clean safe-Rust drop-in for [`src/rle.rs:103-170`](../../src/rle.rs): cuts `copy_within` traffic **`-46.6%`** (`-163.4 MB`), speeds up RLE1+CRC **`0.8944× [0.8850, 0.9039]`** (`-3.58 ms` 1T CPU, `-1.56 B/B` cache traffic), and eliminates the `shift > n` `s.ll` staging fallback completely.
- **KILLED `bzcat_8M_noz` (`0.9985×`):** Removing `buf.resize(8<<20, 0)` alone is noise on a `15.6 MiB` `.bz2` stream (only `2` chunks).
- **KEPT `bzcat_1M_pool` strictly as RSS/minflt hygiene (`-9.9%` RSS, `-11.2%` `minflt`, neutral `0.9962×` wall):** Not a wall-time lever on its own, but trims `5.35 MiB` of input RSS cleanly.
