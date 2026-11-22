# Wormhole Cycle 34 (Neon) — Wall W5: O(1) Ring Ready Slots, Split Directed Condvars, & Bounded Deque Handoff (`-87%`–`-97%` Wakeups, `10W sys_ms -40.6%`)

**Baseline:** `origin/master` `v0.5.12` (`6a36b0e10`), `BzReader::parallel_workers` (`OpenCells`, `slots = 4*W`), `AbHash` consumer, with A/A control (`MasterA` vs `MasterB`) in the same binary.
**Candidate harness:** [`c34_handoff_ab.rs`](2026-11-21-wormhole-cycle34/c34_handoff_ab.rs) (`safe_c34` module `#![forbid(unsafe_code)]`), raw log [`c34_results.txt`](2026-11-21-wormhole-cycle34/c34_results.txt), manifest [`c34_handoff.toml`](2026-11-21-wormhole-cycle34/c34_handoff.toml).

---

## 1. Synchronization Bottlenecks Attacked (`Wall W5`)

In `v0.5.12` (`src/parallel.rs`):
1. `State::ready` is a `BTreeMap<u64, Result<Vec<u8>, Error>>` doing tree traversal + heap node allocation/free inside the global critical section on every block.
2. One `Condvar` calls `notify_all()` on EVERY worker publish (`431` broadcasts!) and EVERY reader take (`431` broadcasts!), creating `862` thundering-herd `FUTEX_WAKE` broadcasts even when the reader isn't waiting on that `place` or the scanner isn't full.
3. Workers serialize on `Mutex<mpsc::Receiver<Job>>` held *across blocking `recv()`*, plus `mpsc::channel` allocates a heap node per block.

**Candidates tested on top of Cycle 32 `A12_FeedPool` (`#![forbid(unsafe_code)]`):**
- **`H12_RingSplitCV`:** Power-of-2 ring slot array `Vec<Option<Result<Vec<u8>, Error>>>` indexed by `(place as usize) & mask` (`0` heap nodes; collision-free by slot invariant `issued - taken <= slots <= cap`) + split `cv_reader` / `cv_scanner` with explicit `reader_waiting` / `scanner_waiting` flags (`cv_reader.notify_one()` ONLY if `reader_waiting && place == taken`; `cv_scanner.notify_one()` ONLY if `scanner_waiting`).
- **`H123_DequeQ`:** `H12` + bounded `VecDeque<Job>` + `cv_worker` (`notify_one()` only when `waiting > 0`), eliminating `Mutex`-across-`recv()` convoy and `mpsc` heap nodes.
- **`H1234_FullCopy`:** `H123` + splitting `free_out` / `free_syms` into separate mutexes + `AtomicU64 taken` scanner fast-path.
- **`H1234_ZeroCopy`:** `H1234` composed with Cycle 33 `C33b` `BufRead` zero-copy output assembly.

---

## 2. Native Safe-Rust A/B + A/A Results (`wikidata_431b_210m.bz2`, `211.0 MB`, `431` blocks, exact `AbHash`)

### 2a. `12`-Round Rotated Fresh-Process `4W` A/B

| Arm | `4W` med (`ms`) | Ratio vs `MasterA` `[95% CI]` | `sys_ms` | `minflt` | `vcsw` | `rss_MB` | Directed wakes (`r`/`s`/`w`) | Verdict |
|---|---:|---|---:|---:|---:|---:|---:|---|
| `MasterA` | `368.22` | `1.0000 [1.0000, 1.0000]` | `37.02` | `15,817` | `710` | `60.29` | `431/431` bcast | Control |
| `MasterB` (`A/A`) | `368.10` | `1.0111 [0.9860, 1.0368]` | `35.91` | `15,802` | `713` | `59.64` | `431/431` bcast | `A/A` clean |
| **`H12_RingSplitCV`** | **`362.30`** | **`0.9836 [0.9727, 0.9946]`** | **`26.34` (`-28.8%`)** | `12,066` | **`446` (`-37.2%`)** | `56.77` | **`114/218` (`notify_one`)** | **PROMOTE** |
| **`H123_DequeQ`** | **`362.64`** | `0.9954 [0.9811, 1.0099]` | **`25.50` (`-31.1%`)** | `12,069` | **`389` (`-45.2%`)** | `57.52` | **`97/163/251`** | **KEEP (10W win)** |
| `H1234_FullCopy` | `364.16` | `0.9988 [0.9740, 1.0242]` | `32.30` (`+6.8` vs `H123`) | `12,276` | `389` | `57.29` | `113/132/285` | **KILL split-pool mutexes** |
| **`H1234_ZeroCopy`** | **`353.65`** | **`0.9647 [0.9535, 0.9761]`** | **`30.83`** | **`11,870`** | **`396`** | **`55.87`** | **`106/203/252`** | **PROMOTE (`H123+C33b`)** |

### 2b. Cold-Start `W = 10` & `W = 16` Contention Check (`3`-round median)

| `W` | Arm | `wall_ms` (`Δ`) | `user_ms` | `sys_ms` (`Δ`) | `minflt` (`Δ`) | `vcsw` | `rss_MB` (`Δ`) | Wakes (`r`/`s`/`w`) |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| `10` | `MasterA` / `MasterB` | `384.99 / 382.78` | `687.9 / 695.9` | `60.69 / 52.70` | `28,919 / 27,266` | `321 / 371` | `106.7 / 95.5` | `431/431` bcast |
| `10` | `H12_RingSplitCV` | `387.88` | `715.9` | **`37.27` (`-38.6%`)** | `21,203` | **`252`** | `94.55` | **`56/12`** (`-87%`/`-97%`) |
| `10` | `H123_DequeQ` | **`367.78` (`-4.47%`)** | `680.1` | `41.40` (`-31.8%`) | `20,060` | `452` | **`85.05` (`-20.3%`)** | `47/44/425` |
| `10` | **`H1234_ZeroCopy`** | **`362.13` (`-5.94%`)** | **`662.0`** | **`36.06` (`-40.6%`)** | **`19,586` (`-32.3%`)** | `449` | **`87.96` (`-17.6%`)** | **`52/65/431`** |
| `16` | `MasterA` / `MasterB` | `397.86 / 401.86` | `707.3 / 718.2` | `70.40 / 67.08` | `36,409 / 36,788` | `352 / 330` | `135.6 / 137.2` | `431/431` bcast |
| `16` | **`H1234_ZeroCopy`** | **`384.24` (`-3.42%`)** | **`689.4`** | **`52.94` (`-24.8%`)** | **`27,433` (`-24.7%`)** | `526` | **`104.77` (`-22.7%`)** | **`56/0/431`** |

---

## 3. Diagnostic Insight & Updated `10T` Critical-Path Model

- **Why `H123` beats `H1234`'s split pool mutexes at `4W` (`sys_ms 25.50 vs 32.30 ms`):** In `H123`, `free_syms.push()` piggybacks on `ready.lock()` at publish and `free_out.push()` piggybacks on `ready.lock()` at take for **zero extra lock acquisitions**. Splitting `free_out` and `free_syms` into two separate mutexes doubles per-block lock ops (`2 → 4` on `ready`/pools), costing `+6.8 ms` `sys_ms` at `4W` while adding no benefit until `16W` (`16W` benefits from the `AtomicU64 taken` check alone!). Therefore in Cycle 35 (Interim Handoff #1) we promote **`H123` (`RingSlot + SplitCV + BoundedDeque`)** piggybacked pools — **killing** the extra split pool mutexes!
- **Updated `10T` i9-10900K Critical Path (Regime 2 post-Mechanisms `C38b`):** `ΔT_handoff_locks` drops **`2.0 ms → ~0.4 ms` (`-1.6 ms`)** as `862` `notify_all()` broadcasts fall to `~68` directed `notify_one()` wakes (`-92%`), bringing modelled `10T` Regime-2 critical path from **`~46.5 ms → ~44.9 ms`**!

---

## 4. Kill Decisions (One Line Each)

- **PROMOTE `H12: Ring-Indexed Ready Slots + Split Directed Condvars`:** Eliminates `BTreeMap` allocs and cuts wakeups `-87%`–`-97%`, `vcsw` `-37.2%`, `sys_ms` `-28.8%` (`4W`) / `-38.6%` (`10W`), and `4W` wall to `0.9836 [0.9727, 0.9946]`.
- **PROMOTE `H123: Bounded VecDeque Job Queue`:** Eliminates `Mutex`-across-`recv()` convoy and `mpsc` node allocs, cutting `4W` `vcsw` `-45.2%` / `sys_ms` `-31.1%` and `10W` wall `-4.47%` / RSS `-20.3%`.
- **KILL `H4: Separate free_out / free_syms Mutexes`:** Breaking piggybacked pool push/pop out of `ready.lock()` doubles per-block lock ops and adds `+6.8 ms` `sys_ms` at `4W` (`0.9988×` vs `0.9836×` `H12`).
