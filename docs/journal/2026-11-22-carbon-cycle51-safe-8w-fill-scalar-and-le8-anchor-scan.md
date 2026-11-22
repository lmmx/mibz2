# Carbon Cycle 51 (`13/20`): Safe `8`-Wide Zero-Slack `runs::fill_scalar` & Little-Endian `8`-Way Unrolled Anchor Scanner

**Date:** 2026-11-22  
**Branch:** `research/carbon-word-transport`  
**Base `origin/master`:** `dd6450e6ab7dc80064f86ce2a737eb4b998cedf1` (`#739` + `#745`)  
**Safety:** `#![forbid(unsafe_code)]` on `src/par_scan.rs`; `100%` safe Rust in `runs::fill_scalar` (`src/runs.rs`).

---

## 1. Hypotheses & Keep / Kill Ledger (One-Line Kills)

1. **KEEP (`c51_anchor_le8` in `src/par_scan.rs`):**
   - Re-indexed `ANCHOR_BITMAP` by native little-endian `u16::from_le_bytes` (`movzx` with zero shift/OR/byte-swap) and unrolled `find_magics_anchor_range` + `scan_slab_interior` to `8` anchors (`32` bytes via `4x u64::from_le_bytes` loads) per iteration.
   - Standalone `wikidata.bz2` (`12 MiB`, `25` interleaved rounds): `Master = 2.204 ms` (`A/A = 1.0028x`) -> **`Cand le8 = 1.902 ms` (`0.8631x`, `1.159x` faster; `443 / 443` magics exact)**.
   - Cut remaining serial-stage CPU share to **`0.54%`** on `wikidata-l9` and **`0.59%`** on `wikidata-l1`.
2. **KEEP (`c51_fill_scalar_8w_noslack` in `src/runs.rs`):**
   - Replaced `tt.extend((0..len).map(...))` in `runs::fill_scalar` with `8`-wide safe slice writes (`&mut tt[w..w + 8]` auto-vectorized to AVX2/SSE2/NEON vector add+store) backed by `16 KiB` (`4,096`-`u32`) L1d-tiled `tt.resize` while `end_u + 8 <= n_u`, handling the final `<= 7` rows scalarly so `tt.reserve(n_u)` needs **zero** `+ 8` slack.
   - Standalone `runs::fill_scalar` (`64` `wikidata-l9` blocks, `15` rounds): `Master = 24.02 ms` (`A/A = 0.9862x`) -> **`Cand 8w = 17.52 ms` (`0.7295x`, `1.371x` faster; `100%` LF-table exact)**.
3. **KILL (`c51_fill_scalar_slack8` — `tt.reserve(n + 8)` in `fill_scalar`):** Killed because `+ 8` slack exceeded `trace::tests::ledger_actual_routes_and_retained_history`'s exact-`n` scalar `TVector` capacity invariant on power-of-two boundaries (`n = 100,000` `u32`s = `400,000 B`).

---

## 2. Verification & Peak RSS (`VmHWM`)

- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`: **PASS**
- `trace::tests` (`5/5`), `runs::tests` (`15/15`), `par_scan::tests` (`1/1`), `tests/par_scan` (`6/6`): **PASS**
- `VmHWM` Peak RSS (`wikidata_x3.bz2`):
  - `i9`: `2T: 0.9345x`, `4T: 0.9370x`, `6T: 0.9049x`, `8T: 0.7971x`, `10T: 0.8168x`, `12T: 0.8195x`, `16T: 0.8259x`, `20T: 0.8182x`.
  - `native`: `1T: 0.9956x`, `2T: 0.9124x`, `4T: 0.9932x`, `6T: 0.9126x`, `8T: 0.7920x`, `10T: 0.7836x`, `12T: 0.7980x`, `16T: 0.8275x`, `20T: 0.8650x`.

---

## 3. Same-Binary Push-Streaming A/B (`7` Interleaved Rounds, `-C opt-level=3 -C codegen-units=1`)

- **`wikidata-l9`** (Serial share: `14.08%` -> **`0.54%`**; Modeled 10T crit = `0.7241x`):
  `1T: 0.9910x`, `2T: 0.9835x`, `4T: 0.9876x`, `6T: 0.9888x`, `8T: 0.9776x`, `10T: 0.9941x`, `12T: 0.9857x`, `16T: 0.9613x`, `20T: 0.9605x` (**every row `1..20T` `< 1.00x` against both `dd6450e6` and `3296195e`**).
- **`wikidata-l1`** (Serial share: `18.38%` -> **`0.59%`**; Modeled 10T crit = `0.6167x`):
  `1T: 0.9727x`, `2T: 1.0008x` (`A/A 1.0435x`), `4T: 1.0190x` (CPU `0.9945x`), `6T: 1.0175x` (CPU `0.9966x`), `10T: 1.0423x` (CPU `1.0057x`).
- **`records-l9`** (Serial share: `20.22%` -> `1.55%`; Modeled 10T crit = `0.4991x`):
  `1T: 0.9956x`, `2T: 0.9887x`, `8T: 1.0166x` (`0.9966x` vs `3296195e`), `10T: 1.0176x` (`0.9898x` vs `3296195e`).
