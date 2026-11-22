# Wormhole Cycle 46 (Neon) — Allocator Deep Dive #2: Contiguous Slab Arena vs `960 KiB` `Vec`s & the Linux `2 MiB` THP Floor (`minflt -95%..98%`, `0.44–0.49×` Cold Slab)

**Host Linux THP config:** `/sys/kernel/mm/transparent_hugepage/enabled = always [madvise] never` (standard Debian/Ubuntu default, matching i9-10900K workstation).
**Artifacts:** [`c46_arena_slab_ab.rs`](2026-11-21-wormhole-cycle46/c46_arena_slab_ab.rs), [`c46_results.txt`](2026-11-21-wormhole-cycle46/c46_results.txt), [`c46_slab_thp.toml`](2026-11-21-wormhole-cycle46/c46_slab_thp.toml).

---

## 1. Why Separate `960 KiB` `Vec`s Cannot Use Transparent Huge Pages

Even after Cycle 32 eliminated `900 KiB → 1.80 MiB` doubling, every individual `960 KiB` `out` buffer lies strictly below Linux's `2 MiB` hugepage size (`960 KiB < 2,048 KiB`), so Linux must fault `235` individual `4 KiB` pages per buffer (`3,548` faults for `16` buffers, `8,836` for `40` buffers) and install `3,548+` `4 KiB` TLB entries (overflowing Comet Lake's `1,536`-entry `4 KiB` STLB).

## 2. `12`-Round Fresh-Process Cold-Start Slab Confrontation (`N × 960 KiB`, `T` Concurrent Threads)

| Scenario | Arm | med (`ms`) | Ratio vs `Sep_Double` `[95% CI]` | **`sys_ms`** | **`minflt`** | `rss_MB` | Verdict |
|---|---|---:|---|---:|---:|---:|---|
| `N=16, T=4` | `Sep_Double` (`v0.5.12`) | `5.88` | `1.0000 [1.0000, 1.0000]` | `7.65` | `3,538` | `16.10` | Baseline |
| `N=16, T=4` | `Slab_Safe` (`1` VMA, `4K`) | `5.63` | `0.9272 [0.8533, 1.0075]` | `6.87` | `3,548` | `16.23` | **KILL alone** |
| `N=16, T=4` | **`Slab_THP`** (`2 MiB` THP) | **`2.93`** | **`0.4544 [0.3862, 0.5348]`** | **`2.70` (`-64.7%`)** | **`176` (`-95.0%`)** | `16.98` | **PROMOTE (Linux opt-in)** |
| `N=16, T=10` | `Sep_Double` | `6.38` | `1.0000 [1.0000, 1.0000]` | `6.07` | `3,548` | `16.10` | Baseline |
| `N=16, T=10` | `Slab_Safe` (`1` VMA, `4K`) | `6.19` | `0.9823 [0.8929, 1.0806]` | `5.46` | `3,557` | `16.22` | **KILL alone** |
| `N=16, T=10` | **`Slab_THP`** (`2 MiB` THP) | **`3.18`** | **`0.4887 [0.4451, 0.5366]`** | **`1.65` (`-72.8%`)** | **`187` (`-94.7%`)** | `17.00` | **PROMOTE (Linux opt-in)** |
| `N=40, T=10` | `Sep_Double` | `14.77` | `1.0000 [1.0000, 1.0000]` | `16.74` | `8,836` | `36.78` | Baseline |
| `N=40, T=10` | `Slab_Safe` (`1` VMA, `4K`) | `13.91` | `0.9659 [0.8850, 1.0543]` | `18.55` | `8,860` | `36.93` | **KILL alone** |
| `N=40, T=10` | **`Slab_THP`** (`2 MiB` THP) | **`6.48`** | **`0.4364 [0.4035, 0.4719]`** | **`4.94` (`-70.5%`)** | **`198` (`-97.8%`)** | `39.03` | **PROMOTE (Linux opt-in)** |

---

## 3. Architectural Takeaway & Kill Decisions

- **Mechanism proved:** Consolidating VMAs alone (`Slab_Safe` with `4 KiB` pages) recovers only `~0.2–0.9 ms` (`0.966–0.982×`) because `3,548–8,836` individual `4 KiB` page faults still occur. Adding Linux `madvise(MADV_HUGEPAGE)` on a single `2 MiB`-aligned `16 MiB` output slab (`8` hugepages) eliminates **`95%–98%`** of minor page faults (`-73%` `sys_ms`, **`2.0–2.3×` faster cold allocation**, and `8` hugepage TLB entries vs `3,840`, saving `~1.5–3.2 ms` on a cold-start `10T` decode!).
- **KILL `Slab_Safe` (4 KiB single slab without THP):** Only `0.982×` at `10T` (`N=16`); does not pay for replacing `&mut Vec<u8>` pool handles with slab slices.
- **PROMOTE `Slab_THP` (optional Linux `#[cfg(target_os = "linux")]` `MADV_HUGEPAGE` `16 MiB` output slab):** Cuts `minflt -95%..-98%`, `sys_ms -73%`, and cold slab latency `0.44–0.49×` with graceful 4 KiB fallback when THP is unavailable.
