# Wormhole (Neon) Cycle 21 — Independent Safe-Rust Audit of `Neon-C29` vs `#714` + `L4/L5` Against `mibz2 0.5.11` (`8ff1187af`)

**Session:** `research/carbon-wormhole` (Round 4 — Gemini Neon, Cycle 21 of 30; 1st of 10 in Round 4)  
**Baseline `origin/master`:** `8ff1187af6a920f7d57fd338680db3a5633a3ac5` (`v0.5.11`; `src/` byte-identical to `90e671966`)  
**Verification harness:** [`verify_cycle21.rs`](2026-11-21-wormhole-cycle21-neon-c29-independent-audit/verify_cycle21.rs) (`#![forbid(unsafe_code)]`, `0` warnings, `6/6` `manifest.json` SHA-256 auth + `5,229` Gate G1/spiral/multi-epoch cases byte-exact + `9`-arm `15`-round 5-stage attribution + `25`-round `p50`/`p10` `InverseBwt::invert_runs` confrontation), [`neon_c29_audit.toml`](2026-11-21-wormhole-cycle21-neon-c29-independent-audit/neon_c29_audit.toml) (`PASS`), [`cycle21_output.txt`](2026-11-21-wormhole-cycle21-neon-c29-independent-audit/cycle21_output.txt), [`cycle21_results.json`](2026-11-21-wormhole-cycle21-neon-c29-independent-audit/cycle21_results.json)

---

## 1. One-Line Audit & Keep/Kill Decisions (Cycle 21)

- **AUDIT VERDICT ON `Neon-C29` (`0.855×` 1T claim reproduced at `0.850×` `p50` vs `M0` / `0.829×` vs `A1`):** **`60.6%` (`-1,113 µs` of `-1,843 µs` vs `M0`) of `Neon-C29`'s claimed `0.855×` 1T win comes from `Fused-XOR` ($\mu=2$ two-ended walk), which already failed natively at `1.021 [1.014, 1.028]` at 4 workers in `#718`; without `Fused-XOR`, `Neon-C29`'s $\mu=1$ construction stack (`N3`) achieves `0.941×` vs `M0` (`0.917×` vs `A1`) and `-19.0%` worker scratch (`950.0 KiB`), matching our Round 3 `W1: #714+L4+L5` (`0.940×` vs `M0`, `0.917×` vs `A1`) in speed while beating it by `-98.1 KiB/worker` in scratch.**
- **KEEP `MuxStrips + Band64 + 48 KiB spelling` (`N2` vs `N1`):** Cuts persistent worker scratch from `1,172.7 KiB` (`M0`) to **`950.0 KiB` (`-19.0%`, `-98.1 KiB` below `W1`'s `StripBandMux`)** with zero runtime regression (`S2+S3` `-18 µs`).
- **KEEP `Pass-1 All+Pos` (`all = thin + thick_sum`; `pos = (tail << 10) | fill`, `N3` vs `N2`):** Delivers a clean **`-65.1 µs` (`-3.2%` on `S1`)** across 6 blocks at `-1 KiB` stack and `0 B` heap (`smaller than Kitchen Sink's -198 µs claim, yet consistently positive`).
- **KEEP `L4: BidiBottom` + `pop_tail` (`#714`):** Both transfer cleanly on top of `Neon-C29`'s construction stack at `0 B` scratch.
- **KILL `Fused-XOR` (`N4` vs `N3`, confirming `#718`):** Adds `+879 µs` (`2.18×`) to `S4` random RMW traffic (`1,622 µs` vs `743 µs`) to win `1T` `S5` (`-2,026 µs`), which reverses to `1.021×` slower at 4 workers natively (`#718`).
- **FLAGGED FOR CYCLE 22 SYNERGY (`Operad-MN16` vs `#714 GradedHandle` & `emit_no_off` `+245 µs` anomaly):** `Operad-MN16` wins `S3` by **`-466 µs`** (`806` vs `1,272 µs`) over `#714`, but **loses `S2` by `+348 µs`** (`1,824` vs `1,476 µs`) because it lacks `#714`'s `GradedHandle` length tags — AND `N1/N2/N3/W2` suffer a mysterious **`+245 µs` `S5 Emit` regression (`6,040` vs `5,795 µs`)** relative to `M0`/`W1` despite shrinking `inline16` from `60.8` to `26.8 KiB`!

---

## 2. Piece-by-Piece Attribution of `Neon-C29` (`measured` in [`verify_cycle21.rs`](2026-11-21-wormhole-cycle21-neon-c29-independent-audit/verify_cycle21.rs))

### 2.1 Dictionary Compaction & Persistent Worker Scratch (6-Block Peak, Excluding `879 KiB` `out`)

`Neon-C29`'s zero-alloc `pair_ht/str_ht` (`24 KiB` inside dead `edges[0..6144]`) deduplicates codes across all 6 Wikidata blocks, bringing `inline16` strictly under `32 KiB` L1D on every block (worst-case `block-143.L`: `60.8 -> 26.8 KiB`):

| Block | Unique Codes (`M0 -> N3`) | `inline16` (`M0 -> N3`) | `spelling` (`M0 -> N3`) |
|---|---:|---:|---:|
| `block-0.L` | `1,582 -> 995` (`-37.1%`) | `24.7 -> 15.5 KiB` | `50.8 -> 46.1 KiB` |
| `block-2.L` | `2,178 -> 1,281` (`-41.2%`) | `34.0 -> 20.0 KiB` | `48.2 -> 40.8 KiB` |
| `block-85.L` | `1,476 -> 811` (`-45.1%`) | `23.1 -> 12.7 KiB` | `30.9 -> 26.3 KiB` |
| `block-143.L` | `3,891 -> 1,712` (`-56.0%`) | `60.8 -> 26.8 KiB` | `56.8 -> 41.5 KiB` |
| `block-210.L` | `2,484 -> 1,219` (`-50.9%`) | `38.8 -> 19.0 KiB` | `37.8 -> 28.4 KiB` |
| `block-207.L` | `2,515 -> 1,205` (`-52.1%`) | `39.3 -> 18.8 KiB` | `54.6 -> 44.9 KiB` |

- **Per-worker scratch (excl. `879 KiB` `out`) / full WS (incl. `out`):**
  - `M0: Clone (0.5.11)`: `1,172.7 KiB` scratch (`2,052 KiB = 2.00 MiB` full WS)
  - `W1: #714+L4+L5`: `1,048.1 KiB` (`-10.6%` scratch; `1,927 KiB = 1.88 MiB` full WS)
  - `N1: OperadDedup`: `1,146.0 KiB` (`-2.3%` scratch)
  - `N2: +MuxBand64+L4` / **`N3: +All+Pos (S*)`** / `W2: Graded+AllPos` / `N4: Full Neon-C29`: **`950.0 KiB` (`-19.0%` scratch; `1,829 KiB = 1.79 MiB` full WS -> `10` i9 workers = `17.9 MiB <= 20 MiB` L3!)**
- **DRAM traffic (`S*`, `N3`):** `0.034–0.059 B/B` (`0` extra `2048K` L3 misses vs `origin/master`).
- **Acquisition cost:** ~180 lines of safe Rust across `src/open/{mod.rs,endpoint.rs,outq.rs}` (already being ported by Codex).

### 2.2 5-Stage Attribution (`6`-Block Sum in `µs`, `15` Rotating Rounds Median)

| Arm | `S1` | `S2` | `S3` | `S4` | **`Pre (S1–S4)`** | **`S5 Emit`** | **6-Blk Sum** | **Scratch** |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `M0: Clone (0.5.11)` | `2,044.4` | `1,873.7` | `1,472.6` | `745.5` | `6,136.1` | **`5,794.6`** | `11,930.6` | `1,172.7 KiB` |
| `W1: #714+L4+L5` | `1,996.8` | **`1,475.8`** | `1,271.8` | `748.9` | `5,493.4` (`-643`) | `5,812.5` | **`11,305.9`** | `1,048.1 KiB` |
| `N1: OperadDedup` | `2,008.1` | `1,798.2` | `852.4` | **`726.5`** | `5,385.3` (`-751`) | `6,069.4` (`+275`!) | `11,454.7` | `1,146.0 KiB` |
| `N2: +MuxBand64+L4` | `2,020.3` | `1,826.8` | **`805.9`** | `743.1` | `5,396.1` (`-740`) | `6,017.9` (`+223`!) | `11,414.0` | **`950.0 KiB`** |
| **`N3: +All+Pos (S*)`** | **`1,955.2`** | `1,824.2` | **`805.8`** | `743.2` | **`5,328.4` (`-808`)** | `6,039.6` (`+245`!) | `11,368.0` | **`950.0 KiB`** |
| `W2: Graded+AllPos` | **`1,936.1`** | **`1,496.1`** | `1,401.2` | `749.1` | `5,582.5` (`-554`) | `6,030.9` (`+236`!) | `11,613.4` | **`950.1 KiB`** |
| `N4: Full Neon-C29` | `1,966.7` | `1,841.5` | `810.7` | `1,622.4` (`+879`!) | `6,241.4` (`+105`) | **`4,013.2`** (`-1,781`) | **`10,254.6`** | **`950.0 KiB`** |

### 2.3 Headline 1-Thread `InverseBwt::invert_runs` Confrontation (`25` Rotating Rounds, `p50` & `p10`)

| Block | `A1:Master` (`p50`) | `A2/A1` | `M0/A1` | `W1/A1` | `N1/A1` | `N2/A1` | **`N3(S*)/A1`** | `W2/A1` | `N4(C29)/A1` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `1,362.7 µs` | `0.988x` | `0.987x` | **`0.909x`** | `0.924x` | `0.919x` | `0.910x` | `0.943x` | `0.873x` |
| `block-2.L` | `1,663.5 µs` | `0.988x` | `0.991x` | **`0.906x`** | `0.922x` | `0.917x` | `0.915x` | `0.943x` | `0.866x` |
| `block-85.L` | `1,706.6 µs` | `0.983x` | `0.954x` | **`0.917x`** | `0.942x` | `0.942x` | `0.923x` | `0.946x` | `0.853x` |
| `block-143.L` | `3,038.2 µs` | `0.995x` | `0.972x` | `0.913x` | `0.906x` | **`0.902x`** | **`0.902x`** | `0.930x` | `0.795x` |
| `block-210.L` | `2,638.0 µs` | `0.998x` | `0.971x` | **`0.930x`** | `0.945x` | `0.946x` | `0.936x` | `0.953x` | `0.803x` |
| `block-207.L` | `2,002.7 µs` | `0.995x` | `0.979x` | `0.920x` | `0.926x` | `0.926x` | **`0.917x`** | `0.946x` | `0.832x` |
| `dense-703.syn` | `3,142.6 µs` | `0.991x` | `0.958x` | `0.916x` | `0.946x` | `0.911x` | **`0.893x`** | `0.946x` | `0.714x` |
| **`6-BLK p50 / A1`** | `12,411.7 µs` | **`0.992x`** | `0.975x` | **`0.917x`** | `0.927x` | `0.925x` | **`0.917x`** | `0.943x` | **`0.829x`** |
| **`6-BLK p10 / A1`** | `12,055.7 µs` | **`0.986x`** | `0.971x` | **`0.910x`** | `0.928x` | `0.927x` | **`0.912x`** | `0.936x` | **`0.829x`** |
| **`6-BLK p50 / M0`** | `12,098.2 µs` | `---` | `1.000x` | **`0.940x`** | `0.951x` | `0.948x` | **`0.941x`** | `0.967x` | **`0.850x`** |

---

## 3. Two Huge Findings Setting Up Cycle 22 (`BMCP_CML` Re-Derivation on `S*`)

1. **Exact Decomposition of `Neon-C29`'s `0.850×` (`-15.0%` vs `M0`):**
   - `Operad-MN16 + Dedup + pop_tail` (`N1`): `-4.9%` (`-751 µs` `Pre`, **offset by `+275 µs` `S5` regression!**)
   - `MuxStrips + Band64 + L4` (`N2`): `-0.3%` time, **`-196 KiB` scratch (`-19.0%` total)**
   - `Pass-1 All+Pos` (`N3`): `-0.7%` (`-65 µs` `S1`, `-1 KiB` stack)
   - `Fused-XOR` (`N4`, killed at 4W in `#718`): **`-9.1%` at 1T (`60.6%` of the entire headline!)**
2. **Two Untapped Goldmines Exposed Directly by Cycle 21's Table:**
   - **Goldmine A (Why did `N1..N3/W2` lose `+245 µs` in `S5 Emit`?):** Every arm with `emit_no_off` + `WordDedup` regressed `S5 Emit` from `5,795 µs` (`M0`) to `6,040 µs`! Cycle 22 will isolate whether that `+245 µs` comes from `emit_no_off`'s `meta` codegen (`rustc --emit=asm`!) or `str_ht` code scatter!
   - **Goldmine B (`GradedOperad` = `#714` `GradedHandle` + `Operad-MN16` `RULE=4 u128`):** Combining `#714`'s `1,476 µs` `S2` with `Operad-MN16`'s `806 µs` `S3` promises another **`-250 to -330 µs`** in `S2` on top of `S*` (`N3`) at `0 B` extra memory!
