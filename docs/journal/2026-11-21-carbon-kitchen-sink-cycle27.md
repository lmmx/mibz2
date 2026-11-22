# Carbon Kitchen Sink (Neon) — Cycle 27: XMM-to-GPR Domain Split Obstruction (`Inline15-ZeroLens`), Exact `48 KiB` `spelling` Fit (`944 KiB = −19.4%` WS), & 6-Phase Profile of `CKS-BiOperad`

**Date:** 2026-11-21  
**Branch:** `research/carbon-kitchen-sink`  
**Theme:** Testing self-describing 128-bit operad slots (`Inline15-ZeroLens`: byte `15 = len in 1..=15, 0 = LONG`, eliminating `lens: Vec<u8>` and `if lbu == 16`), quantifying exact `wlen == 16` incidence, and capturing the full 6-phase microsecond breakdown of `CKS-BiOperad` to steer Neon Cycles 28–34.

---

## 1. Mathematical Construction & Register-Domain Obstruction

In `CKS-BiOperad` (Cycle 25), `inline16: Vec<[u8; 16]>` and `lens: Vec<u8>` store words of length `1..=16`, requiring `4` dictionary loads per dual-cursor step (`lens[cf]`, `inline16[cf]`, `lens[cb]`, `inline16[cb]`) plus a branch `if lbu == 16` before shifting `u128 << ((16 - lbu) * 8)`. Restricting inline words to `1..=15` bytes leaves `slot[15]` free for `wlen` (`1..=15`, `0 = LONG`), which:
- eliminates `Kernel::lens: Vec<u8>` (`−1.8 KiB`),
- unifies `Elim`'s short operad encoding with `inline16` wire format (`inline16[c] == v_bytes` checks bytes and length in one 16-byte compare),
- makes the backward `u128` shift unconditionally branchless (`(16 - lb) * 8 in 8..=120`) while automatically shifting `slot[15]` off the top of the `u128`.

### Theorem 27.1 (XMM-to-GPR Cross-Domain Residency of `lens[c]` vs Self-Describing 128-Bit Slots)
Despite `wlen == 16` occurring on only **`1,322 / 895,391` steps (`0.148%`)** across the 6 Wikidata blocks (and **`0 / 299,999` (`0.000%`)** on `dense-300k`), folding `len` into `slot[15]` of `[u8; 16]` regresses `walk` by **`+465.4 µs` (`0.7234x` vs `0.6475x`)** on `CKS-BiOperad` — and regresses `dense-300k` by **`+6.0%`** (`0.826x` vs `0.766x`) and `CKS-DietOperad` ($\mu = 1$) by **`+3.8%`** (`0.9687x` vs `0.9305x`)!
- **Mechanism:** Keeping `lens: Vec<u8>` (`1.75 KiB`, 28 L1D lines) separate keeps `at_f += lf` and `at_b -= lb` purely inside the **GPR integer domain** (`movzx`), while `inline16[c]` remains purely in the **XMM/vector copy domain** (`movdqu`). Reading `slot[15]` from a 16-byte `[u8; 16]` forces a cross-domain XMM-to-GPR extraction (`pextrb` / stack-store forwarding stall) directly on the critical pointer-advance chain.
- **Orthogonal Memory Win Preserved:** Independently of `Inline15`, pre-reserving `spelling` at `48 KiB` (`49,152 B`, from Cycle 26's Proposition 26.2) cleanly cuts worker scratch from `981.8 KiB` to **`945.8 KiB` (`−19.28%` vs `0.5.11` `master`)** with `lens` intact (`944.0 KiB = −19.43%` without `lens`).

---

## 2. 6-Phase Microsecond Profile of `CKS-BiOperad` (Steers Cycles 28–34)

| Block | `pass1` (µs) | `finish` (µs) | `elim` (µs) | `dedup` (µs) | `wire_xor` (µs) | `walk` (µs) | `wlen == 16` steps |
|---|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `238.8` | `5.1` | `285.5` | `101.8` | `167.9` | `377.3` | `135 / 89,874` (`0.15%`) |
| `block-2.L` | `266.6` | `5.6` | `348.7` | `127.7` | `219.9` | `502.4` | `101 / 112,070` (`0.09%`) |
| `block-85.L` | `343.3` | `2.4` | `149.6` | `67.4` | `238.7` | `642.1` | `101 / 156,404` (`0.06%`) |
| `block-143.L` | `481.5` | `8.9` | `502.7` | `196.2` | `367.8` | `893.4` | `547 / 196,841` (`0.28%`) |
| `block-210.L` | `549.2` | `4.5` | `275.1` | `116.4` | `341.1` | `892.7` | `337 / 204,281` (`0.16%`) |
| `block-207.L` | `391.7` | `5.4` | `336.3` | `128.3` | `259.3` | `593.1` | `101 / 135,921` (`0.07%`) |
| **6-Blk Sum** | **`2,270.9` (`21.8%`)** | **`31.9` (`0.3%`)** | **`1,897.9` (`18.2%`)** | **`737.8` (`7.1%`)** | **`1,594.7` (`15.3%`)** | **`3,901.0` (`37.4%`)** | **`1,322 / 895,391` (`0.148%`)** |

---

## 3. Native Safe-Rust Confrontation Against `0.5.11` `master` (`70` Pairs/Block `1T`, `45` Passes `2T`)

Passed `12,276` exhaustive binary BWTs (`1..=10` $\times$ `LIMIT` $\in \{1, 4, 16\}$) + `dense-300k` + `periodic-150k` + `6 / 6` Wikidata blocks byte-for-byte:

| Block / Suite | `A/A` (`M2/M1`) | `CKS-DietOperad` ($\mu=1$) | `Inline15-Diet` ($\mu=1$) | **`CKS-BiOperad`** ($\mu=2$) | `Inline15-Bi` ($\mu=2$) |
|---|---:|---:|---:|---:|---:|
| `block-0.L` (`1T p10`) | `1.000` | `0.933x` | `0.971x` | **`0.908x`** (`walk 399.6`) | `0.941x` (`walk 442.5`) |
| `block-2.L` (`1T p10`) | `1.005` | `0.932x` | `0.969x` | **`0.899x`** (`walk 500.9`) | `0.937x` (`walk 554.5`) |
| `block-85.L` (`1T p10`) | `0.990` | `0.946x` | `0.980x` | **`0.889x`** (`walk 655.7`) | `0.933x` (`walk 732.5`) |
| `block-143.L` (`1T p10`) | `0.992` | `0.928x` | `0.962x` | **`0.848x`** (`walk 903.2`) | `0.883x` (`walk 1014.5`) |
| `block-210.L` (`1T p10`) | `0.997` | `0.929x` | `0.977x` | **`0.842x`** (`walk 905.4`) | `0.897x` (`walk 1019.0`) |
| `block-207.L` (`1T p10`) | `1.001` | `0.922x` | `0.957x` | **`0.870x`** (`walk 606.7`) | `0.914x` (`walk 674.0`) |
| **`1T` 6-Block Sum** | **`0.9967`** | **`0.9305x`** | `0.9687x` | **`0.8694x`** (`walk 0.6475x`) | `0.9114x` (`walk 0.7234x`) |
| **`dense-300k`** | `1.009` | `0.983x` | `1.008x` | **`0.766x`** | `0.826x` |
| **`periodic-150k`** | `1.005` | `0.873x` | `0.889x` | **`0.869x`** | `0.871x` |
| **`2T` Concurrent (`p10 / p50`)** | **`0.9993 / 0.9991`** | **`0.9358 / 0.9372`** | `0.9626 / 0.9628` | **`0.8683 / 0.8673`** | `0.9037 / 0.9115` |
| **6-Blk Worker Scratch** | `1,171.7 KiB` | `981.8 KiB` (`−16.21%`) | `944.0 KiB` (`−19.43%`) | `981.8 KiB` (`−16.21%`) | `944.0 KiB` (`−19.43%`) |

---

## 4. Cycle 27 Verdicts (One-Line Kill & Promotion)

- **Kill (`Inline15-ZeroLens` Self-Describing Byte-15 Length Packing):** Killed (`0.9114x` vs `CKS-BiOperad`'s `0.8694x`, `walk` `+465.4 µs` slower, and `0.826x` vs `0.766x` on `0`-long `dense-300k`) because reading `slot[15]` from `[u8; 16]` incurs an XMM-to-GPR cross-domain stall vs GPR-domain `lens[c]` (`1.75 KiB`).
- **Promote (`48 KiB` `spelling` Pre-Reserve, Orthogonal to `Inline15`):** Confirmed exact `49,152 B` capacity cuts scratch by `−36.0 KiB` (`981.8 -> 945.8 KiB = −19.28%` vs `0.5.11` `master`) with zero `spelling` reallocations on `6 / 6` blocks.
