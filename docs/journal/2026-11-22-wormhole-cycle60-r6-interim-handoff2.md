# Wormhole Cycle 60 (Neon): Round 6 Interim Handoff #2 (`Cycles 56–60`) — i9 `workers × slots` Grid & Live-Verified PR 4 `BlockSpan` Spec

**Date:** 2026-11-22  
**Branch:** `research/carbon-wormhole`  
**Baseline:** `origin/master` `3296195e3` (`#739` merged; `origin/master` HEAD `10c7673bd` has identical `src/`, with `#748` queuing Mechanisms' block reader `#745` on top of `#739`/`#746`)  
**Mechanisms Coordination:** `origin/research/carbon-word-transport` `300a29976` (`C38a`/`C38b`)  
**Manifest:** [`c60_interim_handoff2.toml`](2026-11-22-wormhole-cycle60/c60_interim_handoff2.toml) (`check_manifest.py`: `PASS`)

---

## Executive Summary (All Four Round 6 Charges Addressed at Cycle 60)

With Interim Handoff #1 ([Cycle 55](2026-11-22-wormhole-cycle55-r6-interim-handoff1.md)) closing **Charge 1** (re-diagnosis + `10W` DRAM `10.82 → ~4.6 B/B`) and **Charge 3** (PR 2 residual closure: `0` hot `notify_all()`, `L1_AtomicFast` killed — port nothing more from PR 2), Cycles 56–59 now complete **Charge 2** and **Charge 4**:

1. **Charge 2 (Cycle 56 — Ready-to-Queue i9 `workers × slots` Grid):** Delivered a **`28`-arm experimental grid** (`16` cells at `10T` + `12` unique SMT cells for `12, 16, 20T`, `< 4 min` on the i9 — **grid only, no hardcoded table**). Anchored to all four real i9 measurements (`#739 7W = 66.9 ms`, `+MechBR 7W = 62.4 ms`, `+MechBR 8W = 59.1 ms`, max error `2.6 ms`). Predicts **`9W / 18..36`** wins on `#739 + MechBR` (**`52.9–53.9 ms`**, `-14%` vs `7W/28`, putting `9` workers + `1` dedicated serial scanner on `10/10` physical cores with `0%` SMT clash) while **`10W / 20..30`** wins once PR 4 removes the serial scanner (**`55.5 ms`** on current Open; **`~44–46 ms`** with Round 4 Open). Also explains `#741`'s `1.11×` `6T` regression (`StreamDecoder(6)` unpinned uses `6 + 1 = 7` active threads vs `parscan(6)`'s `6`!).
2. **Charge 4 (Cycles 57–59 — Finalized & Live-Verified PR 4 `BlockSpan` Streaming Interface Spec):** Agreed interface contract (`SpanCand`, `SpanFinder`, `WindowBuf`, `SpecOutcome`, `ChainValidator`) so Mechanisms' Handoff 1 (`C38b` anchor finder + `C38a` slice block reader) drops straight into `StreamDecoder` with **zero rework**, keeping `#739`'s `Ready` ring, `READY_BYTES = 12 MiB`, `POOLED_FROM = 64 KiB`, `spare()`, `read_block()`, and `cancel()`. Live same-binary multi-regime A/B (`0` precomputed `.spans`, `5.7 GiB/s` safe-Rust anchor scan) hits **`882/882` (`100.0%`, `fb = 0`)** fast hits across all 4 corpora, beats/matches `#739` across all 5 regimes (**`10W 0.9521× [0.9391, 0.9654]`**, **`wikidata-l1 0.9410× [0.9313, 0.9509]`**, **`4W 0.9782×`**), and holds peak RSS **strictly below `#739` everywhere (`0.776×–0.992×` vs `#741`'s `1.28×`!)**.

---

## 1. Charge 2 Deliverable: Ready-to-Queue i9-10900K `workers × slots` Grid (`Cycle 56`)

**Do not hardcode a new `trimmed_here` table before the i9 runs this grid.** Queue these `28` arms (`16` at `10T` + `12` unique SMT arms at `12/16/20T`) on `corpora/wikidata.bz2` (unpinned `whole_machine` + pinned `0..9`), plus a `hetero`/`level-1` guard on the top-3 `(W, slots)` finalists so `slots = 2W` is never selected if it regresses heterogeneous blocks (our Xeon guard in [Cycle 56 §3](2026-11-22-wormhole-cycle56-i9-worker-slots-grid.md) showed `slots = 3W/4W` consistently beats `2W` on `hetero-l9`).

### 1a. `10`-Thread Experimental Grid (`16` cells)
| Cell (`W / slots`) | `slots/W` | Pred `10T` (`3296195e3`) | **Pred `10T` (`+MechBR`)** | **Pred `10T` (`+PR4`)** | Pred `L3` | Pred `DRAM` | Pred `RSS` | Role |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| `7 / 14`, `7 / 21`, **`7 / 28`** | `2×, 3×, 4×` | `64.5 / 64.3 / 64.3 ms` | `61.4 / 61.7 / 62.4 ms` | `68.1 / 68.3 / 68.9 ms` | `20.3–22.3 MiB` | `2.60–3.05 B/B` | `47.0–54.3 MiB` | Current baseline (`7/28`) & controls |
| `8 / 16`, `8 / 24`, `8 / 32` | `2×, 3×, 4×` | `64.6 / 64.4 / 64.5 ms` | `56.5 / 56.7 / 57.4 ms` | `62.5 / 62.7 / 63.3 ms` | `22.3–24.5 MiB` | `3.06–3.48 B/B` | `51.1–59.4 MiB` | Matches real i9 `8T = 59.1 ms` |
| **`9 / 18`, `9 / 27`, `9 / 36`** | `2×, 3×, 4×` | `64.7 / 64.6 / 64.6 ms` | **`52.9 / 53.2 / 53.9 ms`** | `58.4 / 58.6 / 59.3 ms` | `24.3–26.8 MiB` | `3.45–3.85 B/B` | `55.3–64.6 MiB` | **Predicted `+MechBR` #1** (`9W + 1` scan = `10` phys cores) |
| **`10 / 20`, `10 / 30`**, `10 / 40` | `2×, 3×, 4×` | `74.9 / 74.7 / 74.8 ms` | `56.3 / 56.2 / 56.2 ms` | **`55.5 / 55.7 / 56.4 ms`** | `26.3–29.1 MiB` | `3.77–4.16 B/B` | `59.4–69.8 MiB` | **Predicted `+PR4` #1** (`10W` = `10` phys cores) |
| `11 / 22`, `11 / 33` | `2×, 3×` | `75.0 / 74.9 ms` | `56.4 / 56.3 ms` | `57.2 / 57.5 ms` | `28.3–29.8 MiB` | `4.06–4.25 B/B` | `63.5–69.3 MiB` | SMT onset (`+1` SMT) |
| `12 / 24`, `12 / 36` | `2×, 3×` | `75.1 / 75.0 ms` | `56.6 / 56.5 ms` | `59.0 / 59.2 ms` | `30.3–32.0 MiB` | `4.31–4.49 B/B` | `67.7–73.9 MiB` | SMT cliff (`+2` SMT) |

### 1b. `12, 16, and 20`-Thread SMT Extension (`12` unique cells)
- **`T = 12`:** `[winner_10T, 10/30, 11/33, 12/24, 12/36, 12/48]`
- **`T = 16`:** `[winner_10T, 10/30, 12/36, 14/28, 16/32, 16/64]` (predicted `16/32..64` = `60.0–65.9 ms`, `+12%..+18%` slower from `40.5 MiB` L3 spill & SMT L2 halving)
- **`T = 20`:** `[winner_10T, 10/30, 12/36, 16/32, 20/40, 20/80]` (resolves `#739`'s inconclusive `20T` cell by testing clamp to `winner_10T`)

---

## 2. Charge 4 Deliverable: Hardened PR 4 `BlockSpan` Streaming Interface Spec (`Cycles 57–59`)

Full formal specification lives in [Cycle 58](2026-11-22-wormhole-cycle58-pr4-blockspan-streaming-interface-spec.md) with two mandatory production invariants added in [Cycle 59](2026-11-22-wormhole-cycle59-pr4-live-multiregime-ab.md):

### 2a. Exact Ownership Split (`Mechanisms` ↔ `StreamDecoder`)
- **Mechanisms (`src/scan_span.rs` + `src/block.rs`):**
  1. `pub struct SpanCand { pub rel_bit: u64, pub guessed_level: u8 }`
  2. `pub trait SpanFinder { fn find_cands(&self, win: &[u8], rel_start_bit: u64, init_level: u8, out: &mut Vec<SpanCand>); }` — implemented by Mechanisms' `C38b` disjoint 4-byte anchor scanner (`find_magics_anchor_range`).
  3. Slice-based `read_block_from_slice(&[u8], rel_bit, level, syms)` — Mechanisms' `C38a` block reader (no `Source` refill path on the fast path).
- **`StreamDecoder` (PR 4 Bounded Ping-Pong Transport):**
  1. **Constants (Theorem 59.1):** `DEFAULT_WIN_CAP = 4 << 20` (`4 MiB`), `MAX_LIVE_WINS = 2` (`<= 8 MiB` total input RAM — strictly **not** `2 MiB`, which starves `4W` pools on `0.9 MiB` incompressible `BZh9` blocks in `hetero-l9` at `1.0563×`).
  2. **Early `Arc<WindowBuf>` Reclamation:** Workers call `release_win(win)` immediately after `read_block_from_slice` **before** BWT inversion, and the consumer releases `cur_win` upon window rollover, returning the `Vec<u8>` buffer to `free_win` with `0` epoch drain barriers (`-45%..-47%` `vcsw` vs single-window barrier).
  3. **Zero-Seam Deduplication:** Carry from `cands.last().rel_bit / 8` on non-EOF windows, scan from `0` in the next window, and filter `abs_base * 8 + c.rel_bit >= min_next_abs_bit` -> `0` seam fallbacks even at adversarial `256 KiB` (`65` seams).
  4. **Mandatory Cycle 59 Invariant #1 (`published_seq` EOF Guard):** Store `published_seq: usize` in `ReadyRing` alongside `win_closed = true`. `take_next_spec` returns `None` **only** when `st.win_closed && st.taken_seq >= st.published_seq` (never when `tq.q.is_empty()` alone, which races with an in-flight last worker and causes double-decode `fb = 1`).
  5. **Mandatory Cycle 59 Invariant #2 (Zero-`memset` Window Refill):** Keep recycled window buffers at `win_cap` length and read directly into `&mut buf[filled..win_cap]`, truncating only after the read loop (`0` per-window `memset`, $O(\text{win\_cap})$ on arbitrary pipe chunks).
  6. **Mandatory Activation Gate (Fixing `#741`'s `1.11×` `6T` Regression):** Engage PR 4 streaming only when `workers >= phys_cores` (or explicitly requested); below `phys_cores` (e.g., `6T` on a `10C` i9), serial `StreamDecoder` gets a free dedicated scanner core (`W + 1 <= phys_cores`) and avoids redundant anchor-scan work.

### 2b. Live Multi-Regime Verification Summary (`Cycle 59`, `0` Precomputed `.spans`, `A/A` Controlled)
| Regime | `M739_A` | `A/A` (`M739_B`) | **`PR4_Live_4M` `[95% CI]`** | `vcsw` Δ | `minflt` Δ | **Peak RSS (`PR4_Live_4M` vs `#739`)** | Fast hits |
|---|---:|---:|---|---:|---:|---:|---:|
| `wikidata-l9 4W` | `344.75 ms` | `0.9917×` | **`0.9782 [0.9535, 1.0034]`** | `-33.6%` | **`-35.6%`** | **`43.72 vs 56.32 MiB` (`0.776×`)** | `431/431` (`fb=0`) |
| `wikidata-l9 10W` | `383.06 ms` | `1.0016×` | **`0.9521 [0.9391, 0.9654]`** | **`-72.8%`** | **`-16.3%`** | **`69.65 vs 76.82 MiB` (`0.907×`)** | `431/431` (`fb=0`) |
| `wikidata-l1 4W` | `96.01 ms` | `0.9994×` | **`0.9410 [0.9313, 0.9509]`** | **`-63.0%`** | `-1.0%` | **`17.92 vs 18.07 MiB` (`0.992×`)** | `379/379` (`fb=0`) |
| `hetero-l9 4W` | `155.76 ms` | `0.9948×` | **`0.9827 [0.9566, 1.0095]`** | `-18.6%` | `-10.0%` | **`70.86 vs 77.87 MiB` (`0.910×`)** | `60/60` (`fb=0`) |
| `random-l9 4W` | `113.77 ms` | `1.0194×` | **`0.9887 [0.9518, 1.0271]`** | `+6.5%` | `-9.0%` | **`60.92 vs 62.63 MiB` (`0.973×`)** | `12/12` (`fb=0`) |

---

## 3. Cumulative Round 6 Keep / Kill Ledger (`Cycles 51–60`)

| Cycle | Candidate / Mechanism | Verdict | One-Line Evidence |
|---:|---|---|---|
| C52 | **`ZeroShift52` (Thm 52.1)** | **PROMOTED** | Cuts `src/rle.rs:138` `copy_within` **`-46.6%` (`350.8 → 187.4 MB`)** and RLE1+CRC **`0.8944× [0.8850, 0.9039]`** (`0` pre-shift, `0` `s.ll` fallback, `5,000/5,000` + `431/431` exact). |
| C52 | `bzcat_8M_noz` | **KILLED** | `0.9985× [0.9744, 1.0233]` — noise on 2 chunks; `1 MiB` chunk kept strictly for `-9.9%` RSS / `-11.2%` `minflt`. |
| C53 | `L1_AtomicFast` & `Str_TrimLock` | **KILLED** | `0.990–1.006×` across `4W L9`, `4W L1`, `10W`, `16W` & `ParScan` (`0` hot `notify_all()` remain in `#739`; saves `~8.6 µs`). **Port nothing more from PR 2.** |
| C54 | `D1_SliceSrc` alone | **KILLED** | `0.9948× [0.9758, 1.0142]` neutral wall without adaptive capacity. |
| C54 | **`D2_AdaptCap` & `D3_Chunks`** | **PROMOTED** | `D2_AdaptCap`: **`0.9786× [0.9633, 0.9942]`** (`sys -12.9%`, `grow 2→1`) on `13.5×` Wikidata AND `-13.1%` `minflt` / `-9.3%` RSS on `1.0×` `random-l9`; `D3_Chunks`: **`0.9540× [0.9464, 0.9617]`**. |
| C56 | Hardcoded `trimmed_here` table | **KILLED** | Replaced by `28`-arm experimental i9 grid (the i9 decides). |
| C57 | `P0_WholeIn` (`#741` model) | **KILLED** | Reproduces `#741`'s `1.32×` RSS at `10W` (`1.72×` at `16W`) and neutral wall (`1.0011×`). |
| C57–59 | **`PR4_Live_4M` (`4 MiB × 2` Ping-Pong)** | **PROMOTED** | `882/882` (`100.0%`, `fb=0`), beats/matches `#739` across all 5 regimes (`10W 0.9521×`, `l1 0.9410×`, `4W 0.9782×`) at **`0.776×–0.992×` RSS** (`-45%..-47%` `vcsw` vs single-window barrier). |
| C59 | `PR4_Live_2M` (`2 MiB × 2`) | **KILLED** | Regresses `hetero-l9 4W` to **`1.0563× [1.0105, 1.1041]`** (`pub=2` incompressible `0.9 MiB` blocks/window starves `4W` pool). |

---

## 4. Roadmap for Final Half (`Cycles 61–70`)

- **C61:** Adversarial PR 4 stress & edge-case suite (multi-stream concat `BZh1`..`BZh9`, oversized `> win_cap` block, injected false-positive `0x314159265359` & `BZh5` inside compressed payloads, mid-window `cancel()` deadlock/leak check, and `StreamDecoder::write()` push-API adapter).
- **C62:** Window-size × `MAX_LIVE_WINS` Pareto sweep (`{1, 2, 4, 8 MiB} × {1, 2, 3}`) across `4W/10W/16W` on `wikidata-l9`, `wikidata-l1`, and `hetero-l9`.
- **C63:** Composed `PR4_Live_4M` + **Theorem 52.1 `ZeroShift52`** in-place RLE1 end-to-end A/B vs `3296195e3` across all 4 corpora.
- **C64:** Post-`#739 + MechBR + PR 4` residual bottleneck profile (`1T` & `10T` i9 roofline) + cold-start `Slab_THP` check on `3296195e3`.
- **C65:** Full push/pull `StreamDecoder` (`write`/`spare`/`read_block`/`BufRead::fill_buf`) PR 4 adapter prototype & A/B.
- **C66:** Unified `decode_parallel(&[u8])` via PR 4 `SliceWindow` (`0` input copy, `0` feeder thread) + `D2_AdaptCap` A/B.
- **C67:** Within-window LPT straggler scheduling (`cands[i+1].rel_bit - cands[i].rel_bit` compressed-span proxy) vs FIFO at `4W/10W` on `wikidata-l9` & `hetero-l9` (keep or kill honestly).
- **C68:** Final post-Round-6 six-cost (`W, D, B, Q, M, S`) & `10W` DRAM `B/B` trajectory + upstream `origin/master` / Mechanisms sync check.
- **C69:** Final `15`-round rotated fresh-process `4W/6W/10W/16W` cumulative ablation vs `3296195e3` across all 4 regimes.
- **C70:** **Final Round 6 Handoff (`Cycles 51–70`) & clean stop.**
