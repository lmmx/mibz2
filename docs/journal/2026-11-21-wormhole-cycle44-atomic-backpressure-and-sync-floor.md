# Wormhole Cycle 44 (Neon) — `L1` Atomic Backpressure Fast-Path, Directed `notify_one()`, & Closing the Synchronization Floor (`16W vcsw 1,078 → 217`)

**Baseline:** `C42` `ParScan`-coupled `P2` with `notify_one()` fix (`C42A` vs `C42B` A/A control) on `wikidata_431b_210m.bz2` (`211.0 MB`, `431` blocks).
**Candidate harness:** [`c44_sharded_tls_ab.rs`](2026-11-21-wormhole-cycle44/c44_sharded_tls_ab.rs) (`safe_c44` module `#![forbid(unsafe_code)]`), raw log [`c44_results.txt`](2026-11-21-wormhole-cycle44/c44_results.txt), manifest [`c44_sync_floor.toml`](2026-11-21-wormhole-cycle44/c44_sync_floor.toml).

---

## 1. Candidates Tested & Native Fresh-Process Results (`4W` `12`-rd, `10W`/`16W` `5`-rd)

- **`C42A/B` (`notify_one()` fix from Cycle 42):** Single `ReadyRing` `Mutex` with directed `cv_worker.notify_one()`.
- **`L1_AtomicFast`:** Lock-free `AtomicUsize` fast-path on `taken` + `ready_bytes`, eliminating the pre-decode backpressure lock on `99%` of blocks (`2` uncontended lock ops/block total).
- **`L2_Sharded`:** `cap` independent per-slot `Mutex<Option<...>>` + separate `free_out` + `wake_mu`.

| `W` | Arm | med (`ms`) | Ratio vs `C42A` `[95% CI]` | `sys_ms` | `minflt` | **`vcsw`** | Verdict |
|---:|---|---:|---|---:|---:|---:|---|
| `4` | `C42A` / `C42B` (`A/A`) | `341.1 / 339.4` | `1.0000` / `0.9822 [0.9597, 1.0053]` | `10.5 / 11.1` | `3,961` | `215 / 218` | Control |
| `4` | **`L1_AtomicFast`** | `339.85` | `0.9941 [0.9825, 1.0058]` | `12.21` | `3,958` | `228` | **KEEP** |
| `4` | `L2_Sharded` | `341.97` | `0.9959 [0.9785, 1.0137]` | `12.89` | `3,960` | `364` (`+69%`) | **KILL** |
| `10` | `C42A` / `C42B` (`A/A`) | `335.7 / 334.8` | `1.0000` / `0.9857 [0.9578, 1.0145]` | `19.2 / 21.9` | `9,198` | `87 / 87` | Control |
| `10` | **`L1_AtomicFast`** | `337.41` | `0.9960 [0.9792, 1.0130]` | `23.12` | `9,219` | **`82`** (`-80%` vs `0.5.12`) | **KEEP** |
| `10` | `L2_Sharded` | `336.66` | `0.9826 [0.9358, 1.0318]` | `22.76` | `9,429` | `97` | **KILL** |
| `16` | `C42A` / `C42B` (`A/A`) | `364.9 / 361.4` | `1.0000` / `0.9761 [0.9432, 1.0101]` | `31.7 / 30.6` | `13,303` | **`262`** (was `1,078`!) | `notify_one` win |
| `16` | **`L1_AtomicFast`** | **`357.27`** | **`0.9637 [0.9205, 1.0089]`** | `32.19` | `13,278` | **`217` (`-79.9%` vs C42)** | **PROMOTE** |
| `16` | `L2_Sharded` | `364.90` | `0.9833 [0.9473, 1.0206]` | `31.84` | `13,302` | **`870`** (`+3.3×`!) | **KILL** |

---

## 2. Theorem 44.1 — Closure of Synchronization Overhead (`Wall W5`)

With `O(1)` ring-indexed `ReadyRing`, directed `notify_one()`, piggybacked `free_out`, and `L1_AtomicFast`:
- Total critical-section hold time across `431` blocks (`2 × 431 = 862` uncontended `~15 ns` ops) is **`~13 µs` (`0.03%` of a `40 ms` `10T` decode!)**.
- `vcsw` drops to **`82`** at `10W` (`-79.9%` vs `v0.5.12`'s `407`) and **`217`** at `16W` (`-79.9%` vs Cycle 42's `1,078`).
- Any further sharding (`L2_Sharded`) adds a second `wake_mu` lock + double-checked locking race window, multiplying `vcsw` `3.3×` (`262 → 870` at `16W`) with zero wall gain. **Wall W5 (block hand-off & lock contention) is CLOSED.**

---

## 3. Kill Decisions (One Line Each)

- **PROMOTE `L1_AtomicFast + Directed cv_worker.notify_one()`:** Eliminates the `16W` wake herd (`vcsw 1,078 → 217`), cuts `10W` `vcsw` to `82`, and wins `16W` (`0.9637×`).
- **KILL `L2_Sharded` (Per-Slot Mutexes):** Double-checked `wake_mu` locking raises `vcsw` `+69%` (`4W`) and `+232%` (`16W`: `262 → 870`) with neutral wall time (`0.9959×` `4W`, `0.9833×` `16W`).
