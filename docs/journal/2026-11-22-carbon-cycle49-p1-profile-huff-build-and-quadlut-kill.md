# Carbon Cycle 49 (`11/20`): `P1` Sub-Phase Profile, Vectorized `Huff::build_into`, `16 KiB` L1d-Tiled `syms.resize`, & `QuadLut10` Kill

**Date:** 2026-11-22  
**Branch:** `research/carbon-word-transport`  
**Base `origin/master`:** `dd6450e6ab7dc80064f86ce2a737eb4b998cedf1` (`#739` + `#745`)  
**Safety:** `#![forbid(unsafe_code)]` on `src/huff.rs` and `src/block.rs` (`0` new `unsafe` anywhere).

---

## 1. Fine-Grained `P1` (`block::read_item`) Sub-Phase Profile

Instrumented `c49_p1_screen` across all three reference corpora (`100%` bit- and error-exact across all blocks + `1,500` mutated/truncated bitstreams):

| Corpus | Blocks | Total `P1` | `hdr + sel` | `lens` (delta) | `Huff::build_into` | `syms.resize` | `decode_group_win` |
|---|---:|---:|---:|---:|---:|---:|---:|
| `wikidata-l9` | `431` | `62.41 ms` | `2.60 ms` (`4.2%`) | `0.67 ms` (`1.1%`) | `2.94 ms` (`4.7%`) | `0.06 ms` (`0.1%`) | **`56.15 ms` (`90.0%`)** |
| `wikidata-l1` | `361` | `20.94 ms` | `0.86 ms` (`4.1%`) | `1.04 ms` (`5.0%`) | **`3.74 ms` (`17.9%`)** | `0.01 ms` (`0.1%`) | **`15.28 ms` (`73.0%`)** |
| `records-l9` | `11` | `33.12 ms` | `1.34 ms` (`4.0%`) | `0.05 ms` (`0.1%`) | `0.13 ms` (`0.4%`) | `0.02 ms` (`0.1%`) | **`31.59 ms` (`95.4%`)** |

---

## 2. Keep / Kill Ledger (One-Line Kills)

1. **KEEP (`c49_huff_build` + `c49_l1_resize` in `src/huff.rs` & `src/block.rs`):**
   - `Huff::build_into`: replaced indexed `self.lut[off_p + u] = self.lut[off0 + u].wrapping_add(delta)` (which carried aliasing + dual bounds checks) with `self.lut.split_at_mut(off_p)` disjoint slice zip (`vpaddd` `8x u32` auto-vectorization), replaced per-code `self.symbol(zn, p)` calls with a direct `perm` cursor, and scoped `d_tail` zeroing to `[..LUT_LEN - filled_to]` (`4..64 B` instead of `1,024 B`).
   - `block::read_item_core`: replaced upfront cold `syms.resize(max_out, 0)` (`up to 1.8 MiB`) with `8,192`-`u16` (`16 KiB` L1d-resident) incremental resizing inside the group loop (`if n + 66 > syms.len() && syms.len() < max_out { syms.resize((n + 8192).min(max_out), 0); }`).
   - Standalone `P1` (`15` interleaved rounds): `wikidata-l9` **`0.9790x`** (`62.20 -> 60.90 ms`, `A/A = 0.9940x`), `wikidata-l1` **`0.9992x`** (`A/A = 1.0100x`), `records-l9` **`0.9882x`** (`33.07 -> 32.68 ms`, `A/A = 0.9839x`).
   - Eliminates cold page touches past `n + 8192`, cutting `native` `6T` peak RSS from `1.0028x` -> **`0.9380x`** (`-8.28 MiB`) so **every `1–20T` row on both `native` and `i9` is now `< 1.00x` (`0.7720x–0.9955x`)**.
2. **KILL (`c49_sel_lens` — SWAR `leading_zeros` zero-run skip in selector MTF & delta code lengths):** Killed (`1.0051x` on `wikidata-l9`, `1.0307x` on `wikidata-l1`) because `hdr+sel` (`4.1%`) + `lens` (`1.1–5.0%`) have short zero runs (`0..2` bits), so extra branch mispredicts outweigh `1`-bit skips.
3. **KILL (`c49_quad_lut10` — `8 KiB` `u64` 4-symbol self-composed canonical Huffman LUT):** Killed (`1.1062x` on `wikidata-l9`, `1.2376x` on `wikidata-l1`, `1.2754x` on `records-l9`) because `c <= 4` shrinks the branchless `5`-step unrolled group prefix from `k <= 40` (`80%+`) to `k <= 30` (`60%`), pushing `40%` of every 50-symbol group onto the branchy tail while doubling active L1d table footprint (`48 KiB` vs `24 KiB`).

---

## 3. Subprocess `VmHWM` Peak RSS (`wikidata_x3.bz2`, `629.10 MB` out)

| Threads | `i9` Master MiB | `i9` `c49` MiB | `i9` Ratio | `native` Master MiB | `native` `c49` MiB | `native` Ratio |
|---:|---:|---:|---:|---:|---:|---:|
| `1` | `28.43` | `28.30` | **`0.9953x`** | `28.48` | `28.35` | **`0.9955x`** |
| `2` | `68.97` | `65.10` | **`0.9438x`** | `65.67` | `64.05` | **`0.9753x`** |
| `4` | `48.26` | `42.46` | **`0.8799x`** | `94.66` | `89.89` | **`0.9496x`** |
| `6` | `56.27` | `50.83` | **`0.9034x`** | `133.66` | `125.38` | **`0.9380x`** |
| `8` | `62.00` | `49.02` | **`0.7907x`** | `62.84` | `48.51` | **`0.7720x`** |
| `10` | `59.57` | `49.38` | **`0.8289x`** | `68.00` | `53.04` | **`0.7799x`** |
| `12` | `59.43` | `48.49` | **`0.8159x`** | `70.00` | `57.24` | **`0.8178x`** |
| `16` | `59.42` | `48.57` | **`0.8174x`** | `75.96` | `62.88` | **`0.8277x`** |
| `20` | `60.48` | `48.98` | **`0.8099x`** | `82.59` | `70.70` | **`0.8560x`** |

---

## 4. Same-Binary Push-Streaming A/B (`7` Interleaved Rounds, `-C opt-level=3 -C codegen-units=1`)

- **`wikidata-l9`** (Serial share: `14.09%` -> `0.66%`; Modeled 10T crit = `0.7259x`):
  `1T: 0.9637x` (`A/A 1.0014x`), `2T: 0.9733x`, `4T: 0.9819x`, `6T: 0.9973x`, `8T: 0.9888x`, `10T: 0.9814x` (`0.9557x` vs `3296195e`), `12T: 0.9830x`, `16T: 0.9402x`, `20T: 0.9274x` (**every row `1..20T` `< 1.00x`**).
- **`wikidata-l1`** (Serial share: `18.37%` -> `0.68%`; Modeled 10T crit = **`0.5985x`**):
  `1T: 0.9642x` (`A/A 0.9979x`), `2T: 0.9624x`, `4T: 1.0093x`, `6T: 0.9993x`, `10T: 1.0322x` (CPU `1.0050x`).
- **`records-l9`** (Serial share: `21.00%` -> `1.61%`; Modeled 10T crit = `0.4855x`):
  `1T: 1.0026x` (`A/A 1.0067x`), `4T: 0.9935x`, `6T: 1.0040x`, `8T: 1.0250x` (CPU `1.0400x`), `10T: 1.0239x` (CPU `1.0276x`).
