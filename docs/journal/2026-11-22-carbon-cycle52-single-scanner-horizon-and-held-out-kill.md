# Carbon Cycle 52 (`14/20`): `single_slab_scanner` `max_front_ahead` Horizon Cap & `held_out` Kill

**Date:** 2026-11-22  
**Branch:** `research/carbon-word-transport`  
**Base `origin/master`:** `dd6450e6ab7dc80064f86ce2a737eb4b998cedf1` (`#739` + `#745`)  
**Safety:** `#![forbid(unsafe_code)]` on `src/par_scan.rs` (`0` new `unsafe` anywhere).

---

## 1. Hypotheses & Keep / Kill Ledger (One-Line Kills)

1. **KEEP (`c52a` — `single_slab_scanner` `max_front_ahead` horizon cap in `src/par_scan.rs`):**
   - Previously, `single_slab_scanner` (`workers <= 6`) bounded in-flight blocks only by `shared.full(&st)` (`shared.slots = 4 * workers` on untrimmed `native`, i.e., `16` blocks at `4T` and `24` blocks at `6T`), while `ParScanHub` bounded `(issued - taken) < max_front_ahead = (workers + 4).min(slots).max(2)`.
   - Enforcing `(st.issued - st.taken) < hub.max_front_ahead as u64` + `max_syms_pool = max_ahead.min(6)` inside `single_slab_scanner` (deadlock-free by induction since every `st.taken` increment in `StreamDecoder` notifies `scan_cv` while `st.issued > st.taken`) cut **`4T` `i9` RSS from `45.22 -> 38.36 MiB` (`0.7940x`, `-9.95 MiB`)**, **`6T` `i9` RSS from `51.52 -> 43.15 MiB` (`0.7570x`, `-13.85 MiB`)**, **`4T` `native` RSS to `0.8499x` (`-16.06 MiB`)**, and **`6T` `native` RSS to `0.8966x` (`-12.94 MiB`)**.
2. **KILL (`c52b` — `held_out` thread-local `free_out` hoarding in `parscan_hub_worker`):** Killed because parked workers retaining a `900 KiB` `free_out` buffer across `jobs.recv()` starved active workers and added `+4.2 MiB` (`i9` `8–20T`) to `+11.2 MiB` (`native` `20T`) peak RSS without lowering wall time.

---

## 2. Subprocess `VmHWM` Peak RSS (`wikidata_x3.bz2`, `629.10 MB` decoded)

Every multi-threaded row (`2–20T`) on both `i9` and `native` is now **`0.757x–0.945x`** of `master`:

| Threads | `i9` Master MiB | `i9` `c52a` MiB | `i9` Ratio | `native` Master MiB | `native` `c52a` MiB | `native` Ratio |
|---:|---:|---:|---:|---:|---:|---:|
| `1` | `28.46` | `28.29` | **`0.9940x`** | `28.35` | `28.44` | `1.0032x` |
| `2` | `68.38` | `61.32` | **`0.8968x`** | `64.93` | `61.37` | **`0.9451x`** |
| `4` | `48.30` | `38.36` | **`0.7940x`** | `107.00` | `90.94` | **`0.8499x`** |
| `6` | `57.00` | `43.15` | **`0.7570x`** | `125.13` | `112.19` | **`0.8966x`** |
| `8` | `61.93` | `48.84` | **`0.7886x`** | `62.41` | `48.32` | **`0.7741x`** |
| `10` | `61.40` | `49.46` | **`0.8055x`** | `67.95` | `52.36` | **`0.7705x`** |
| `12` | `61.06` | `48.45` | **`0.7934x`** | `71.39` | `56.45` | **`0.7906x`** |
| `16` | `60.47` | `49.39` | **`0.8167x`** | `77.88` | `63.92` | **`0.8207x`** |
| `20` | `58.77` | `48.47` | **`0.8247x`** | `88.46` | `69.91` | **`0.7903x`** |
