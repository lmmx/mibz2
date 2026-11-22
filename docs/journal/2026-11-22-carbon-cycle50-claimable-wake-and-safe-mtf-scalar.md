# Carbon Cycle 50 (`12/20`): Work-Counted `ParScanHub` Helper Wakeups, Lazy `ScanScratch`, & Safe `mtf_scalar`

**Date:** 2026-11-22  
**Branch:** `research/carbon-word-transport`  
**Base `origin/master`:** `dd6450e6ab7dc80064f86ce2a737eb4b998cedf1` (`#739` + `#745`)  
**Safety:** `#![forbid(unsafe_code)]` on `src/par_scan.rs`; `100%` safe Rust in `mtf_scalar` (`src/mtf.rs`) and `front_scalar` (`src/runs.rs`).

---

## 1. Hypotheses & Keep / Kill Ledger (One-Line Kills)

1. **KEEP (`c50_claimable_wake` + lazy `Option<Box<ScanScratch>>` in `src/par_scan.rs`):**
   - Previously, `ParJobs::recv` called `hub.helper_cv.notify_one()` on every `rem <= 1` pop even after `EOF` when `0` unclaimed candidates remained, `par_scanner_lead` called `notify_all()` even when `<= 1` candidate was claimable (`lead` itself claims `1`), and `par_scanner_helper` allocated `54 KiB` `ScanScratch` eagerly on thread spawn.
   - Added `claimable: AtomicUsize` synchronized inside `hub.mu` (`sync_claimable`), `wake_helpers(jobs, 1)` (`avail == 0` -> `0` wakes, `avail == 1` -> `notify_one()`, `avail >= 2` -> `notify_all()`), race-free double-checked sleep in `par_scanner_helper`, and lazy `Option<Box<ScanScratch>>`.
   - Cut `records-l9` (`11` blocks) CPU ratio from `1.0400x–1.0814x` down to **`1.0092x` at `8T`**, **`1.0218x` at `10T`** (`1.0199x` wall vs `A/A = 1.0180x`, `0.9927x` vs `3296195e`), and **`1.0128x` at `12T`** (`-6.86%` CPU), with `0.4639x` modeled 10T critical path (`wikidata-l9` serial share cut to **`0.57%`**).
2. **KEEP (`c50_mtf_scalar_b1` in `src/mtf.rs` & `src/runs.rs`):**
   - Initialized `mtf_scalar` and `runs::front_scalar` MTF lists directly from `b.seq_to_unseq` (eliminating `b.seq_to_unseq[v]` dependent L1d lookup on every symbol and run) and replaced per-symbol `ll.push(c)` in `mtf_scalar` with `16 KiB` L1d-tiled slice writes (`ll.resize((n + 16384).min(max), 0)`).
   - Standalone `mtf_scalar` (`15` interleaved rounds): **`0.9323x`** (`48.53 -> 45.25 ms`, `A/A = 0.9934x`) on `wikidata-l9` and **`0.9349x`** (`111.95 -> 104.66 ms`, `A/A = 1.0011x`) on `records-l9`.
3. **KILL (`c50_mtf_u64x32` — `[u64; 32]` word-carry MTF list):** Killed because although `nn < 8` won on `wikidata-l9` (`0.8317x`), the `for slot in &mut list[..q]` serial `carry` chain regressed high-rank `records-l9` to `1.8308x` vs vectorized `libc::memmove`.
4. **KILL (`c50_mtf_slack32` — `[u8; 288]` safe 32-byte loop blend for `nn < 32`):** Killed (`0.9821x` / `0.9638x`) because scalar byte-select codegen for `out32` lost to `c50_mtf_scalar_b1` (`0.9323x` / `0.9349x`).

---

## 2. Verification & Peak RSS (`VmHWM`)

- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`: **PASS**
- `mtf_checkpoints` (`3/3`), `mtf_kernels` (`4/4`), `par_scan` (`6/6`), `trace::tests` (`5/5`), `1,500`-case streaming fuzz (`PASS 1..5`): **PASS**
- `VmHWM` Peak RSS (`wikidata_x3.bz2`):
  - `i9`: `2T: 0.9614x`, `4T: 0.9167x`, `6T: 0.9106x`, `8T: 0.7718x`, `10T: 0.8456x`, `12T: 0.8079x`, `16T: 0.8340x`, `20T: 0.8132x` (**all `6–20T <= 0.911x`**).
  - `native`: `2T: 0.9202x`, `4T: 0.9899x`, `6T: 1.0124x`, `8T: 0.7879x`, `10T: 0.7851x`, `12T: 0.7817x`, `16T: 0.7583x`, `20T: 0.8689x`.

---

## 3. Same-Binary Push-Streaming A/B (`7` Interleaved Rounds, `-C opt-level=3 -C codegen-units=1`)

- **`wikidata-l9`** (Serial share: `14.14%` -> **`0.57%`**; Modeled 10T crit = `0.7276x`):
  `1T: 1.0096x` (`0.9817x` vs `3296195e`), `2T: 0.9909x`, `4T: 0.9955x`, `6T: 1.0059x` (`A/A 1.0111x`), `8T: 0.9916x`, `10T: 1.0000x` (`A/A 1.0084x`, `0.9792x` vs `3296195e`), `12T: 0.9561x`, `16T: 0.9573x`, `20T: 0.9593x`.
- **`wikidata-l1`** (Serial share: `18.33%` -> `0.69%`; Modeled 10T crit = `0.6332x`):
  `1T: 1.0065x` (`A/A 1.0052x`), `2T: 0.9637x` (CPU `0.9784x`), `4T: 1.0036x` (`A/A 1.0335x`), `20T: 0.9966x`.
- **`records-l9`** (Serial share: `21.68%` -> **`1.40%`**; Modeled 10T crit = **`0.4639x`**):
  `1T: 0.9977x`, `2T: 0.9935x`, `4T: 1.0128x` (CPU `0.9882x`), `8T: 1.0220x` (CPU **`1.0092x`**), `10T: 1.0199x` (`A/A 1.0180x`, `0.9927x` vs `3296195e`, CPU **`1.0218x`**), `12T: 1.0216x` (CPU **`1.0128x`**).
