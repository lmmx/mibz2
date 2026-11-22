# Carbon Kitchen Sink (Neon) — Cycle 40: `S1Trim` Bounds-Check Fusion Breaks `0.911×` `Pre` (`−534 µs` vs `53f7ecf93`, `0.950×` `1T`) & `ColdThk`/`S2aSlim` Kills

**Date:** 2026-11-21  
**Branch:** `research/carbon-kitchen-sink`  
**Persona:** Gemini Neon  
**Baseline:** `origin/master` (`53f7ecf93`, `#719` construction stack, `945.8 KiB` scratch)  
**Scope:** Research-only; `src/`, `Cargo.toml`, `benches/`, and `examples/` untouched.

---

## 1. Mathematical & Compiler Framing (`Theorem 40.1`: Exact-Trimmed Bounds-Check Fusion in `OutQ16`)

In `Neon-C39`'s `S1` fast path (`OutQ16::push_thin`), `p = (tail << 10) | fill` checked `p < self.in_out_lim` (`in_out << 10`), then indexed `self.out[at..at + 32]` where `at = p + f = tail * 1024 + 2 * f`. Because `f` is packed in the low 10 bits of `p`, LLVM could not prove `p < in_out_lim && f <= 496 => at + 32 <= self.out.len()`, emitting **two separate checks and a panic branch** on all `489,099` thin runs, plus keeping `in_out_lim` live in a register.

**Theorem 40.1 (Single-Check Overflow-and-Bounds Fusion):** Slicing `self.out = &mut out[..in_out * 1024]` once in `Queues::new` makes `self.out.len() == in_out_lim` identically. Consequently, `if let Some(dst) = self.out.get_mut(at..at + 32)` simultaneously verifies (i) `tail < self.in_out` (non-overflow chunk) and (ii) exact safe-Rust slice bounds for the 32-byte `vmovdqu` store in **one single compare**, dropping `in_out_lim` from `Queues`. Pairing this with `(m ^ (m + w)) < EPOCH` (reusing `m_next = m + w` and `m as u16` without masking `m & 0xFFFF`) eliminates three ALU/branch ops per thin run.

We also tested two other structural candidates alongside a full mid-campaign 6-stage re-profile:
- **`ColdThk`:** Outlining the `4.7%` thick-run + `width == 127` path into `#[cold] #[inline(never)] take_thick_cold`.
- **`S2aSlim`:** Reading only `(to, width)` in `S2a` Pass 2 + explicit `Narrow::set_links` in `Elim::insert`.

---

## 2. Mid-Campaign 6-Stage Re-Profile & Native A/B Results (`run_c40.rs`)

Compiled with `rustc --edition 2021 -C opt-level=3 -C target-cpu=native -C codegen-units=1` (`#![forbid(unsafe_code)]`). Parity passed `100%` on `6/6` Wikidata blocks + `dense-300k` + `#703` `Cap::dense(300_000)` refusal (`None`); persistent scratch unchanged at **`945.8 KiB` (`+0.00%`)**.

### 2.1 Mid-Campaign 6-Stage Profile (`40` rounds/block, `µs`)

| Arm | `S1` | `S2a` | `S2b` | `S3a` | `S4` | **`Pre (S1–S4)`** | `S5` | `Total` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `Master (53f7)` | `2,009.9` | `356.0` | `1,160.6` | `848.0` | `1,519.9` | **`5,894.3`** | `4,359.5` | `10,253.8` |
| `Neon-C39` | `1,785.1` | **`319.7`** | `1,160.6` | `707.1` | **`1,441.8`** | `5,414.3` | `4,402.2` | `9,816.5` |
| `C40-Combo` *(incl. ColdThk)* | **`1,703.0`** | `326.8` | `1,174.2` | **`705.2`** | `1,458.7` | `5,367.9` | `4,398.0` | `9,766.0` |
| `Master-Nobi` | `1,980.8` | `354.8` | `1,162.2` | `859.2` | `745.8` | **`5,102.7`** | `5,566.9` | `10,669.6` |
| `Nobi-C39` | `1,800.8` | `320.3` | **`1,144.1`** | `705.6` | `744.3` | `4,715.2` | `5,575.5` | `10,290.7` |
| `Nobi-C40` *(incl. ColdThk)* | **`1,698.1`** | `325.4` | `1,157.2` | **`699.0`** | **`729.4`** | **`4,609.1`** | `5,549.1` | `10,158.1` |

*(S1 dropped `>300 µs` below `origin/master` and `~200 µs` below Wormhole's declared `1,889 µs` `S1` floor; `S3a` dropped `~150 µs` below `origin/master` and `~60 µs` below Wormhole's `761 µs` `S3` floor.)*

### 2.2 Headline `1T` (`50` rounds/block, `35`-round `Pre` split) & `2T` (`35` passes)

| Candidate | `1T` Total (`µs`, ratio vs base) | `1T` `Pre` (`µs`, ratio vs base) | `ΔPre` vs `C39` | `2T` p10 (`µs`, ratio) | `2T` p50 (`µs`, ratio) |
|---|---:|---:|---:|---:|---:|
| `MasterA1` (`53f7ecf93`) | `10,546.0 (1.0000×)` | `6,007.2 (1.0000×)` | `+421.6` | `10,003.3 (1.0000×)` | `10,251.8 (1.0000×)` |
| `MasterA2` (A/A) | `10,537.6 (0.9992×)` | `5,964.7 (0.9929×)` | `+379.1` | `10,151.5 (1.0148×)` | `10,484.6 (1.0227×)` |
| `Neon-C39` (prev champ) | `10,141.7 (0.9617×)` | `5,585.7 (0.9298×)` | `0.0` | `9,654.3 (0.9651×)` | `9,964.2 (0.9719×)` |
| **`S1Trim` (`Neon-C40`)** | **`10,021.9 (0.9503×)`** | **`5,473.1 (0.9111×)`** | **`−112.5`** | — | — |
| `ColdThk` | `10,233.3 (0.9703×)` | `5,675.9 (0.9448×)` | `+90.2` | — | — |
| `S2aSlim` | `10,191.0 (0.9663×)` | `5,604.0 (0.9329×)` | `+18.4` | — | — |
| `C40-Cmb` *(all 3)* | `10,094.8 (0.9572×)` | `5,528.1 (0.9202×)` | `−57.5` | `9,449.6 (0.9446×)` | `9,922.7 (0.9679×)` |
| `Master-Nobi` ($\mu=1$ base) | `10,959.4 (1.0000×)` | `5,219.0 (1.0000×)` | `+340.5` | `10,349.7 (1.0000×)` | `10,695.5 (1.0000×)` |
| `Nobi-C39` | `10,568.8 (0.9644×)` | `4,878.4 (0.9347×)` | `0.0` | — | — |
| `Nobi-C40` *(incl. ColdThk)* | **`10,412.8 (0.9501×)`** | **`4,747.1 (0.9096×)`** | **`−131.3`** | **`9,715.5 (0.9387×)`** | **`10,400.8 (0.9724×)`** |

Per-block `1T` ratios for **`S1Trim` (`Neon-C40`)** vs `MasterA1` (A/A `0.9992×`):
`block-0: 0.952`, `block-2: 0.956`, `block-85: 0.948`, `block-143: 0.948`, `block-210: 0.952`, `block-207: 0.948` (`6/6` clean wins!).

---

## 3. One-Line Kills & Promotions

- **KILL `ColdThk` (`+90.2 µs` `Pre` vs `Neon-C39`):** Outlining `take_thick_cold` as `#[inline(never)]` forces `24,000` real ABI function calls (callee-save spills) that cost more than keeping thick-strip writes inlined.
- **KILL `S2aSlim` (`+18.4 µs` `Pre` vs `Neon-C39`):** Explicit `set_links` / Pass-2 rewrite regresses `S2a` (`+7 µs`) and `S2b` (`+14 µs`) via worse codegen.
- **PROMOTE `S1Trim` (`Neon-C40`, `+0 B` WS):** Cuts `Pre (S1–S4)` by a further **`−112.5 µs`**, reaching **`0.9111×` `Pre` (`−534.1 µs` vs `53f7ecf93`)** and **`0.9503×` `1T` full kernel** (`6/6` wins vs A/A `0.9992×`) — and **`0.9096×` `Nobi Pre` / `0.9501×` `1T`** even before removing `ColdThk`!

---

## 4. Artifacts & Next Step

- Reproducers archived in `docs/journal/2026-11-21-carbon-kitchen-sink-cycle40/` (`gen_c40.py`, `run_c40.rs`).
- **Cycle 41 (Neon):** Deep dive into **`S1` (`~1,670 µs`)** + **`S2b` (`~1,150 µs`)** — testing (i) **2× Unrolled / Software-Pipelined `S1` Record Loop** (overlapping `pt[a0]` and `pt[a1]` L1/L2 loads for consecutive thin runs!), (ii) **Sub-16-Byte Direct `OutQ16` Chunk Layout (`f <= 512 - 16` vs `u64` / `u128` iota)**, and (iii) **`S2b` Single-Check `join` + `Elim` Arena Pre-Sizing (`arena_words`)** to eliminate mid-elimination `grow()` reallocations!
