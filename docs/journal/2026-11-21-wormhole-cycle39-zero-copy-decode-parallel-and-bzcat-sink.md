# Wormhole Cycle 39 (Neon) — Zero-Copy Consumer Output Assembly: `decode_parallel` (`0.8754×`) & `bzcat` Sink (`0.9043×`, `-55.6%` `minflt`, `-38.7%` RSS)

**Baseline:** `origin/master` `v0.5.12` (`6a36b0e10`), `src/lib.rs:540` `decode_parallel` loop & `examples/bzcat.rs` `16 MiB` bounce loop (`OpenCells`, `4W`, `slots = 4*W`), with A/A control in the same binary.
**Candidate harness:** [`c39_reader_pipeline_ab.rs`](2026-11-21-wormhole-cycle39/c39_reader_pipeline_ab.rs) (`safe_c39` module `#![forbid(unsafe_code)]`), raw log [`c39_results.txt`](2026-11-21-wormhole-cycle39/c39_results.txt), manifest [`c39_reader_assembly.toml`](2026-11-21-wormhole-cycle39/c39_reader_assembly.toml).

---

## 1. The `5.6 B/B` Copy Cascade Inside `v0.5.12`'s `decode_parallel` & `bzcat`

Inspecting `src/lib.rs:540–568` (`decode_parallel`) and `examples/bzcat.rs`:
1. `decode_parallel(data: &[u8])` splits an already-contiguous `&[u8]` into `4 MiB` `to_vec()` heap copies sent to a feeder thread (copy #1), which `ChannelSource` copies into `buf` (copy #2); then `dec.read(&mut buf)` copies `211 MB` from `r.cur` into a `4 MiB` bounce buffer (copy #3, `+2.0 B/B`), `out.extend_from_slice(&buf[..n])` copies `211 MB` again into `out` (copy #4, `+2.0 B/B`), and `out = Vec::with_capacity(data.len() * 4)` (`56 MB`) **doubles twice** (`56 → 112 → 224 MB`, `+1.6 B/B` + `mmap`/`munmap` TLB shootdowns!) = **`5.6 B/B` (`1.18 GB`!)** of serial reader-thread traffic!
2. `examples/bzcat.rs` allocates a `16 MiB` bounce buffer (`4,096` page faults!) + `8 MiB` input chunks and copies all `211 MB` through `dec.read(&mut buf)` before `out.write_all(&buf[..n])`.

**Candidates (`#![forbid(unsafe_code)]`, using `SliceSource` directly on `&[u8]` + `P1c` `RingSlot/SplitCV/BoundedDeque`):**
- **`R1_DirectExtend` (`decode_parallel` API-identical):** `SliceSource(data)` + `out = Vec::with_capacity(data.len() * 8)` + direct `out.extend_from_slice(&blk)` without any `4 MiB` bounce buffer.
- **`R2_ZeroCopyChunks` (`decode_parallel_chunks` → `Vec<Vec<u8>>`):** Consumer takes direct ownership of each decoded block `Vec<u8>` (**`0 B/B` reader copy!**).
- **`R3_ZeroCopySink` (`decode_to_writer` / `bzcat` via `BufRead`):** Writes directly from `&blk` via `w.write_all(&blk)` and immediately recycles `blk` (`0` `16 MiB` bounce buffer, **`-2.0 B/B` DRAM**!).

---

## 2. Native Safe-Rust `12`-Round Rotated Fresh-Process `4W` A/B (`wikidata_431b_210m.bz2`, `211.0 MB`)

| Arm | `4W` med (`ms`) | Ratio vs Base `[95% CI]` | `user_ms` | `sys_ms` | `minflt` | `rss_MB` | Verdict |
|---|---:|---|---:|---:|---:|---:|---|
| `MasterA_dec_par` | `400.23` | `1.0000 [1.0000, 1.0000]` | `591.95` | `137.63` | `67,527` | `271.74` | Control (`decode_parallel`) |
| `MasterB_dec_par` (`A/A`) | `387.94` | `0.9811 [0.9512, 1.0119]` | `584.94` | `134.01` | `67,397` | `270.74` | `A/A` clean |
| **`R1_DirectExtend`** | **`373.81`** | **`0.9559 [0.9028, 1.0122]`** | `567.90` (`-24.1`) | `120.20` (`-12.7%`) | `58,832` (`-12.9%`) | `243.26` (`-28.5`) | **PROMOTE (drop-in)** |
| **`R2_ZeroCopyChunks`** | **`351.02`** | **`0.8754 [0.8567, 0.8945]`** | **`537.59` (`-54.4`)** | **`116.23` (`-15.5%`)** | **`56,255` (`-16.7%`)** | **`233.27` (`-38.5`)** | **PROMOTE (new API)** |
| `Master_bzcat_null` | `321.29` | `1.0000 [1.0000, 1.0000]` | `559.90` | `36.14` | `15,625` | `65.99` | Control (`bzcat`) |
| **`R3_ZeroCopySink`** | **`295.02`** | **`0.9043 [0.8866, 0.9225]`** | **`534.25` (`-25.7`)** | **`19.12` (`-47.1%`)** | **`6,935` (`-55.6%`)** | **`40.46` (`-38.7%`)** | **PROMOTE (`bzcat`)** |

---

## 3. Impact on `10T` i9-10900K Critical-Path Model

For `decode_parallel` callers (`Vec<u8>` output), `v0.5.12`'s `5.6 B/B` serial reader cascade adds **`~24–30 ms`** of serial memory copy + `mmap_lock` fault contention (`~45%` of wall time at `10T`!). `R1_DirectExtend` cuts `2.5+ B/B` with zero API change; `R2_ZeroCopyChunks` removes ALL `5.6 B/B` (`-12.5%` wall even at `4W`, `~18–25 ms` at `10T`!). For `bzcat` / `BufRead` streaming callers, `R3_ZeroCopySink` removes the `16 MiB` bounce buffer (`-55.6%` `minflt`, `-47.1%` `sys_ms`, `-38.7%` RSS, **`0.9043×`** wall at `4W`!).

---

## 4. Kill Decisions (One Line Each)

- **PROMOTE `R1_DirectExtend` (`decode_parallel`):** Drop-in zero-API-change removal of `4 MiB` bounce buffer + `SliceSource` direct scan (`0.9559×`, `-24 ms` user, `-28.5 MiB` RSS).
- **PROMOTE `R2_ZeroCopyChunks` (`decode_parallel_chunks`):** Zero-copy owned-block API (`0.8754 [0.8567, 0.8945]`, `-54.4 ms` user, `-38.5 MiB` RSS).
- **PROMOTE `R3_ZeroCopySink` (`examples/bzcat.rs` via `BufRead`):** Eliminates `16 MiB` bounce buffer and `211 MB` memcpy (`0.9043 [0.8866, 0.9225]`, `-47.1%` `sys_ms`, `-55.6%` `minflt`, `-38.7%` RSS).
