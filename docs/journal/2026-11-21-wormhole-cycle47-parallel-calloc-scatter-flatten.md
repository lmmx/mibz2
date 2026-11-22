# Wormhole Cycle 47 (Neon): Parallel `calloc` + $W$-Way Scatter Flatten vs Incremental `extend_from_slice` and Zero-Copy Chunks

**Date:** 2026-11-21  
**Branch:** `research/carbon-wormhole`  
**Baseline:** `origin/master` `v0.5.12` (`6a36b0e10b2e2e781518a101db7ccf8b6352b986`)  
**Scope:** API-identical `decode_parallel -> Result<Vec<u8>>` flattening strategies vs zero-copy `decode_parallel_chunks -> Result<Vec<Vec<u8>>>`  
**Artifacts:** [c47_parallel_flatten_ab.rs](2026-11-21-wormhole-cycle47/c47_parallel_flatten_ab.rs), [c47_results.txt](2026-11-21-wormhole-cycle47/c47_results.txt), [c47_parallel_flatten.toml](2026-11-21-wormhole-cycle47/c47_parallel_flatten.toml)

---

## 1. Question & Hypothesis

In Cycle 39, `R2_Chunks` (`decode_parallel_chunks -> Result<Vec<Vec<u8>>>`) beat `v0.5.12`'s `decode_parallel` by `-12.5%` (`0.8754×`) at `4W` by eliminating the final `211 MB` flatten copy into a single contiguous `Vec<u8>`. For callers that strictly require a contiguous `Vec<u8>`, an appealing safe-Rust idea (`F1_CallocScat`) is:

1. Decode all $N$ blocks in parallel with zero serial reader bottleneck into `chunks: Vec<Vec<u8>>`;
2. Compute `total = chunks.iter().map(Vec::len).sum()`;
3. Allocate `let mut out = vec![0u8; total]` (which lowers to `calloc` — a single anonymous `mmap` in `~3 µs` with zero serial memset);
4. Split `out` along exact block boundaries via safe `slice::split_at_mut` and scatter-copy across all $W$ workers in parallel.

Does `F1_CallocScat` beat overlapped incremental `Par_SerialExt` (`ParScan`-coupled workers + bounded recycled `free_out` + overlapped reader `out.extend_from_slice(&v)`) — or does holding `chunks` + `out` simultaneously trigger a **double-RSS + concurrent page-fault** cliff?

---

## 2. Experimental Setup

Same-binary safe-Rust (`#![forbid(unsafe_code)]` inside `mod safe_c47`) `12`-round rotated fresh-process `4W` A/B on the `431`-block, `211,000,232`-byte authenticated Wikidata stream (`wikidata_431b_210m.bz2`), verifying exact byte length (`211000232`) on every run:

- **`MasterA` / `MasterB`:** Unmodified `v0.5.12` `decode_parallel` path (`StreamDecoder::workers(4, 16, k, true)` + `4 MiB` feeder chunks + `4 MiB` reader bounce buffer + `out.extend_from_slice`).
- **`Par_SerialExt`:** `ParScan`-coupled atomic-steal workers with recycled `960 KiB` `free_out` (`cap >= 64 KiB` filter) + overlapped reader `out.extend_from_slice(&v)` into pre-sized `out`.
- **`F1_CallocScat`:** `ParScan`-coupled `decode_to_chunks` + `vec![0u8; total]` + $W$-way safe `split_at_mut` parallel scatter-copy.
- **`R2_Chunks`:** `ParScan`-coupled `decode_to_chunks -> Result<Vec<Vec<u8>>>` (zero flatten copy).

---

## 3. Native Safe-Rust Results (`12`-Round Fresh-Process `4W` A/B, `211 MB`)

| Arm | `med_ms` | `vs MasterA [95% CI]` | `user_ms` | `sys_ms` | `minflt` | `rss_MB` | Verdict |
|---|---:|---:|---:|---:|---:|---:|---|
| `MasterA` | `404.44` | `1.0000 [1.0000, 1.0000]` | `612.64` | `137.02` | `67,682` | `272.37` | Baseline |
| `MasterB` (A/A) | `402.12` | `0.9998 [0.9806, 1.0193]` | `607.94` | `140.68` | `67,541` | `270.00` | Clean A/A |
| **`Par_SerialExt`** | **`373.25`** | **`0.9285 [0.9098, 0.9477]`** | `599.66` | **`102.37`** (`-25.3%`) | **`56,353`** (`-16.7%`) | **`234.84`** (`-13.8%`) | **PROMOTED (contiguous `Vec<u8>`)** |
| `F1_CallocScat` | `428.70` | `1.0751 [1.0418, 1.1096]` | `595.13` | `236.52` (`+72.6%`) | `105,688` (`+56.2%`) | `420.20` (`+54.3%`) | **KILLED** |
| **`R2_Chunks`** | **`348.36`** | **`0.8586 [0.8486, 0.8687]`** | **`571.04`** | `110.87` (`-19.1%`) | **`54,177`** (`-20.0%`) | **`225.89`** (`-17.1%`) | **PROMOTED (zero-copy chunks)** |

---

## 4. Mechanism Analysis & Theorem 47.1 (The Double-Live Flatten Law)

**Theorem 47.1 (Why Two-Phase Parallel Scatter Regresses on Contiguous `Vec<u8>`).**  
Any two-phase decode-then-scatter flattener (`chunks: Vec<Vec<u8>>` $\to$ `out = vec![0u8; total]`) pays **three strictly avoidable penalties** relative to overlapped incremental `Par_SerialExt`:

1. **Double Live RSS ($2N_{\text{out}}$):** Because exact scatter offsets are unknown until all blocks finish RLE1 expansion, all `211 MB` of `chunks` must remain resident while the `211 MB` destination `out` is faulted in, driving peak RSS to **`420.20 MiB`** (`+185.36 MiB` over `Par_SerialExt`'s `234.84 MiB` and `+194.31 MiB` over `R2_Chunks`'s `225.89 MiB`).
2. **Concurrent `mmap_lock` Demand-Zero Fault Storm (`+49,335` faults, `+134.15 ms` `sys_ms`):** Although `vec![0u8; total]` returns in `~3 µs`, the subsequent $W$-way scatter touches all `51,514` fresh `4 KiB` pages concurrently on a single `211 MiB` anonymous VMA, driving `minflt` from `56,353` to **`105,688`** and `sys_ms` from `102.37 ms` to **`236.52 ms`** (`+131%`!). Even though `user_ms` drops by `4.5 ms` (`599.66 -> 595.13 ms`), the `+134.15 ms` kernel-fault explosion swamps it by `30×`.
3. **Cold-DRAM Round-Trip on i9-10900K (`+3.0 B/B` DRAM = `+15.1 ms`):** By the time block `430` finishes, blocks `0..400` in `chunks` have been completely evicted from Comet Lake's `20 MiB` L3 to DRAM. Scatter-reading `211 MB` cold from DRAM (`+1.0 B/B`) + write-allocating `211 MB` into `out` (`+2.0 B/B` RFO+WB) adds `+3.0 B/B` (`+15.1 ms` at `42 GB/s`), whereas `Par_SerialExt` copies each `900 KiB` block while it is still **L3-hot** and immediately recycles its buffer back into the `12 MiB` pool.

### Modelled `10T` i9-10900K Critical Path (`decode_parallel`, `211 MB`, post-`ParScan`)

| Variant | Peak RSS | Flatten DRAM (`B/B`) | Modelled `10T` Wall (`211 MB`) |
|---|---:|---:|---:|
| `v0.5.12` `decode_parallel` (w/o `ParScan`) | `272 MiB` | `+4.0 B/B` (`4 MiB` bounce + `Vec` doublings) | `~74–78 ms` |
| `F1_CallocScat` (w/ `ParScan`) | **`420 MiB`** | `+3.0 B/B` cold + `10T` `mmap_lock` storm | `~66–72 ms` (**KILLED**) |
| **`Par_SerialExt`** (API-identical `Vec<u8>`, w/ `ParScan`) | **`235 MiB`** | **`+1.5–2.0 B/B`** (L3-hot src, overlapped) | **`~46–49 ms`** |
| **`R2_Chunks`** (`decode_parallel_chunks`, w/ `ParScan`) | **`226 MiB`** | **`+1.0 B/B`** (write-once, `0` flatten copy) | **`~40–43 ms`** |

*(Note: For streaming `bzcat` / `BufRead` consumers that never materialize the entire `211 MB` in RAM at once, `WormholeScaleDecoder` holds only `~51 MiB` RSS at `10W` and reaches `~39–41 ms`.)*

---

## 5. Six-Cost Separation (`Par_SerialExt` / `R2_Chunks` vs `F1_CallocScat`)

| Cost | `Master` (`v0.5.12`) | `F1_CallocScat` (Killed) | `Par_SerialExt` (Promoted) | `R2_Chunks` (Promoted) |
|---|---|---|---|---|
| **W (work)** | `612.6 ms` user (`4W`) | `595.1 ms` user | `599.7 ms` user | **`571.0 ms`** user (`-6.8%`) |
| **D (crit path)** | Serial scan + `4 MiB` bounce | Serial barrier + `10T` fault storm | Overlapped L3-hot `extend` | **`0 ms`** flatten barrier |
| **B (DRAM)** | Bounce + `Vec` growth copies | `+3.0 B/B` cold scatter | `+1.5–2.0 B/B` L3-hot | **`0 B/B`** flatten copy |
| **Q (rand misses)** | Unchanged | Unchanged | Unchanged | Unchanged |
| **M (memory/RSS)** | `272.4 MiB`, `67.7k` `minflt` | `420.2 MiB`, `105.7k` `minflt` | **`234.8 MiB`**, `56.4k` `minflt` | **`225.9 MiB`**, `54.2k` `minflt` |
| **S (sync/sys)** | `137.0 ms` `sys_ms` | `236.5 ms` `sys_ms` (`mmap_lock`) | **`102.4 ms`** `sys_ms` (`-25.3%`) | `110.9 ms` `sys_ms` |

---

## 6. Kill / Keep Decisions

- **KILLED `F1_CallocScat`** (`1.0751× [1.0418, 1.1096]`, `+134 ms` `sys_ms`, `105.7k` `minflt`, `420.2 MiB` RSS) — two-phase decode-then-scatter doubles live RSS and triggers a `51.5k`-fault concurrent `mmap_lock` storm.
- **PROMOTED `Par_SerialExt`** (`0.9285× [0.9098, 0.9477]`, `sys -25.3%`, `minflt -16.7%`, `RSS -13.8%`) — best safe-Rust implementation for API-identical `decode_parallel -> Result<Vec<u8>>`.
- **PROMOTED `R2_Chunks`** (`0.8586× [0.8486, 0.8687]`, `-41.6 ms` user, `RSS -17.1%`) — best additive API (`decode_parallel_chunks -> Result<Vec<Vec<u8>>>`) for callers that can consume block slices directly.
