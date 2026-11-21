# Wormhole Cycle 14 — Breaking Kakutani Conjugate Thin-Run Streams (`G = 1` and `G = 16` in Dead `out`) vs `OutQ16` (`G = 256`): The Dead-Tail Overwrite & Band-Coalescing Asymmetry Theorem

**Session:** `research/carbon-wormhole` (Round 3, Cycle 14 of 20; 4th of 10 in Round 3)  
**Baseline `origin/master`:** `90e6719663af0615fb417b038906ad2636193aa3` (`mibz2::bwt::OpenCells`)  
**Verification harness:** [`verify_cycle14.rs`](2026-11-21-wormhole-cycle14-kakutani-run-streams-vs-outq16/verify_cycle14.rs) (`#![forbid(unsafe_code)]`, `0` warnings, `5,228` Gate G1 + `Spill::ovf` + multi-epoch cases byte-exact + `19`-round rotating A/B/A confrontation), [`kakutani_run_stream.toml`](2026-11-21-wormhole-cycle14-kakutani-run-streams-vs-outq16/kakutani_run_stream.toml) (`PASS`), [`cycle14_output.txt`](2026-11-21-wormhole-cycle14-kakutani-run-streams-vs-outq16/cycle14_output.txt), [`cycle14_results.json`](2026-11-21-wormhole-cycle14-kakutani-run-streams-vs-outq16/cycle14_results.json)

---

## 1. One-Line Kill Decisions (Cycle 14)

- **KILL `B1: Unqueued` (`QUEUE = false` re-scan, `open.rs:486-514`):** Killed because although skipping `OutQ16` cuts `S1` from `264.7–604.6 µs` to `158.6–309.4 µs`, re-scanning `records` and scattering `r_thin` short scalar writes into `edges` balloons `S4` from `32.8–183.8 µs` to **`666.1–1,467.8 µs`**, losing **`1.261×–1.429×` (`7/7` blocks)** against `origin/master`.
- **KILL `B2: Kakutani-1Stream` (Kakutani 1943 single sequential `(sym: u8, width: u8)` log aliased in dead `out`, `0 B` scratch):** Killed because despite beating `Unqueued` by `-162` to `-366 µs` and cutting `S1` by `-70` to `-178 µs`, unbucketed per-run `S4` dispatch (`476.4–1,113.5 µs` vs `OutQ16`'s `32.8–183.8 µs`) loses **`1.165×–1.252×` (`7/7` blocks)** against `origin/master` `OutQ16` (note: in Round 1 Cycle 4 `CSW-Open` won `0.973–0.991×` against `0.5.8`'s `2 MiB` heap `Vec<u32>[256]` queues, but loses decisively once `OutQ16` aliases `256` AVX2 queues into dead `out`!).
- **KILL `B3: Radix16-Kakutani` (McIlroy–Bostic–McIlroy 1993 16-nibble `(sym_lo4 << 28) | ((w-1) << 24) | m24` `u32` super-queues in dead `out`, `0` epoch cuts, `16` pages $\le$ `32`-page streamer & `64`-DTLB):** Killed (`1.190×–1.285×` on `7/7` blocks; `S4 = 464.8–1,091.4 µs`) because even when all `16` active symbol cursors fit inside L1d/DTLB/streamer, `G = 16` still leaves `16` symbols interleaved per stream, preventing band-level coalescing in `S4`.

---

## 2. The Chain of Existing Results Tested

Cycle 13 proved that `origin/master`'s `OutQ16` is already AVX2-optimal *within* a 256-queue layout (`1` 256-bit `vpaddw + vmovdqu %ymm0` per thin run). The remaining structural question on `S1 + S4` was whether reducing the radix bucket count $G \in \{1, 16, 256\}$ could beat `G = 256` (`OutQ16`) by eliminating `S1`'s `256`-stream scatter (`107–223` `DTLB64` misses and `1.5–4.4 MiB` of `32 B` over-writes):
1. **Link 1 — Kakutani (1943) Induced-Return Conjugate Thin-Run Log (`B2`, $G = 1$):** Under `#703`'s density gate $r / n < 7/25 = 0.28$, $2 r_{\mathrm{thin}} \le 2(r + 2) < 0.56 n + 4 \le n + 16$, so the entire sequence of retained base runs $(c_t, w_t)_{t=0}^{r_{\mathrm{thin}}-1}$ fits sequentially in `out[0..2*r_thin]` (`95–281 KiB`, `1` active `4 KiB` page, `0 B` scratch).
2. **Link 2 — McIlroy–Bostic–McIlroy (1993) 16-Way Top-Nibble Super-Queues + Exact 32-Bit Token Packing (`B3`, $G = 16$):** Partitioning by $g = c \gg 4 \in \{0..15\}$ leaves 4 bits for $c \mathbin{\&} 15$, 4 bits for $w - 1 \in \{0..15\}$, and 24 bits for exact rank $m < 900{,}000 < 2^{24}$ — packing `((c & 15) << 28) | ((w - 1) << 24) | m` into a single `4 B` `u32` with **zero `Spill::cuts` epoch arrays** and **at most `16` active `4 KiB` pages** in both `S1` and `S4`.

---

## 3. Theorem 14.1 — The Dead-Tail Overwrite & Band-Coalescing Asymmetry Theorem (`proved` + `measured`)

Why does `G = 256` (`OutQ16`) beat **every** $G < 256$ (`G = 1` unqueued, `G = 1` Kakutani, `G = 16` Radix16-Kakutani) by **`2.0×–2.4×` on `S1 + S4` combined** (`353–744 µs` vs `671–1,775 µs`), even when $G = 16$ eliminates all `DTLB64` and `32`-page streamer misses?

**Theorem 14.1 (`proved` + `measured` in [`verify_cycle14.rs`](2026-11-21-wormhole-cycle14-kakutani-run-streams-vs-outq16/verify_cycle14.rs)):** Let $r_{\mathrm{thin}}$ be the number of retained runs (`47,736–140,660`, mean width $\bar{w} = m / r_{\mathrm{thin}} \in [1.00, 2.02]$) and $B_{\mathrm{bands}}$ be the number of monotone return bands (`2,446–6,788` on Wikidata, mean span $m / B_{\mathrm{bands}} \in [28.1, 63.9]$ rows). Between `S1` (queue append) and `S4` (`edges[0..m)` scatter), two strict structural asymmetries hold:
1. **Dead-Tail Overwrite Asymmetry (`32 B` SIMD in `S1` vs Exact-Width Scalar in `S4`):** In `S1`, each queue $q$ appends into its own private unwritten tail `chunk[fill[q]..]`, so `OutQ16` unconditionally writes `16` ranks (`32 B`) in **1 branchless AVX2 `vmovdqu %ymm0` uop** regardless of $w \in [1, 16]$. In `S4`, `edges[0..m)` is partitioned into adjacent live intervals `[dest, dest + span)` of different bands; for any $G < 256$, each thin run writes only $w$ edges (`mean 1.0–2.0`) and **cannot** over-write `edges[dest + w .. dest + 16]` without clobbering a live neighbour band, forcing a variable-trip scalar loop (`for i in 0..w`) with branch mispredictions on $w$!
2. **Band-Coalescing Factor $\frac{r_{\mathrm{thin}}}{B_{\mathrm{bands}}} \in [15.2, 35.4]\times$ (`G = 256` vs $G < 256$):** Only at **exact symbol resolution $G = 256$** are all thin runs of byte $a$ concatenated prior to `S4`, collapsing `S4`'s outer loop from $r_{\mathrm{thin}}$ per-run dispatches (`stream[a]` RMW + `cursors[a]` load + `bands[cur]` load + `end == b.end` check) down to $B_{\mathrm{bands}}$ contiguous `4× vpmovzxwd` vector bursts! At $G = 16$ (`B3`), `16` symbols remain interleaved in each nibble queue, so `S4` still executes **100% of all $r_{\mathrm{thin}}$ per-run scalar dispatches** (`464.8–1,091.4 µs`, nearly identical to $G = 1$'s `476.4–1,113.5 µs`!).

### Stage-Isolated & End-to-End Native Safe-Rust Confrontation (`19` Interleaved Rounds, `0 B` Extra Scratch)

- **Per-Worker Working Set:** `1.885 MiB` across all four arms (`A1/A2`, `B1`, `B2`, `B3` all alias dead `out` with `0 B` extra scratch on admitted blocks).
- **DRAM Traffic:** `0.034–0.059 B/B` (`0` extra `2048K` L3 misses).

| Block | `M0:OutQ16` (`S1 / S4 / Sum µs`) | `B1:Unq` (`S1 / S4 / Sum µs`) | `B2:Kak1S` (`S1 / S4 / Sum µs`) | `B3:Rad16` (`S1 / S4 / Sum µs`) | `A1:Master` (`µs`) | `A2/A1` (Ctrl) | `B1:Unq / A1` | `B2:Kak1S / A1` | `B3:Rad16 / A1` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `264.7 / 88.4 /` **`353.1`** | `158.6 / 666.1 / 824.7` | `194.4 / 476.4 / 670.8` | `240.8 / 464.8 / 705.6` | `1,369.8` | `0.992x` | `1.303x` | `1.183x` | `1.214x` |
| `block-2.L` | `284.1 / 112.1 /` **`396.3`** | `174.4 / 753.5 / 928.0` | `209.8 / 555.4 / 765.3` | `259.0 / 551.1 / 810.1` | `1,684.8` | `0.988x` | `1.261x` | `1.165x` | `1.190x` |
| `block-85.L` | `379.4 / 88.0 /` **`467.4`** | `199.0 / 1013.7 / 1212.7` | `256.8 / 772.4 / 1029.2` | `336.4 / 752.5 / 1088.9` | `1,701.8` | `0.993x` | `1.380x` | `1.252x` | `1.285x` |
| `block-143.L` | `534.3 / 183.8 /` **`718.1`** | `298.0 / 1467.8 / 1765.7` | `390.3 / 1113.5 / 1503.9` | `491.6 / 1091.4 / 1583.1` | `3,000.1` | `1.000x` | `1.305x` | `1.196x` | `1.222x` |
| `block-210.L` | `604.6 / 139.7 /` **`744.4`** | `309.4 / 1465.1 / 1774.5` | `426.3 / 1085.0 / 1511.3` | `540.6 / 1035.3 / 1575.9` | `2,652.6` | `0.992x` | `1.337x` | `1.207x` | `1.243x` |
| `block-207.L` | `425.8 / 127.9 /` **`553.7`** | `231.0 / 1065.1 / 1296.1` | `319.8 / 781.0 / 1100.8` | `388.4 / 746.3 / 1134.7` | `2,008.2` | `0.991x` | `1.318x` | `1.197x` | `1.226x` |
| `dense-703.syn` | `579.3 / 32.8 /` **`612.1`** | `287.5 / 1107.3 / 1394.7` | `404.1 / 760.1 / 1164.2` | `572.2 / 626.5 / 1198.7` | `1,587.2` | `0.993x` | `1.429x` | `1.239x` | `1.252x` |

---

## 4. Closure of `S1 + S4` Transport & Pivot to `Stage S2` (Cycles 15–16)

Cycles 13 and 14 **jointly close `Stage S1 + S4` (`OutQ16`)**:
- `158.6–309.4 µs` of `S1` is irreducible `maximal(records)` + `strips.push` (whose upstream `runs::front` `CAP=127` boundary we audit in Cycle 19 for Obligation #5), and
- `OutQ16`'s remaining `106–295 µs` in `S1` buys **`-388` to `-1,075 µs`** in `S4` via `Theorem 14.1` (`32 B` dead-tail `vmovdqu` + `18–35×` band coalescing + `86%` `4 KiB` streamer hiding), making `G = 256` `OutQ16` strictly optimal across both store width and bucket count!

We now pivot directly to **Pre-`Emit` Gap #2: `Stage S2` (`Thick-Strip Construction + RV-Morse Elimination`, `141–452 µs`, `5.95–8.70×` L2 miss inflation)** in **Cycle 15**!
