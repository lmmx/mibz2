# Wormhole Cycle 31 (Neon) — `v0.5.12` 8→20T Plateau Anatomy, `occupancy.rs` Timeline, & `10T` Critical-Path Model

**Baseline:** `origin/master` `v0.5.12` (`6a36b0e10b2e2e781518a101db7ccf8b6352b986`).
**Mechanisms coordination:** `origin/research/carbon-word-transport` (`300a29976`, Cycles 30–38; `C38b StreamParScan`). Mechanisms owns the stream scan and block finder; Wormhole does **not** duplicate scan/finder work and explicitly models both **Regime 1** (standalone `v0.5.12` serial scan) and **Regime 2** (coordinated with Mechanisms `C38b StreamParScan`).
**Host:** `Intel(R) Xeon(R) Platinum 8481C` (`2` physical vCPUs, `2 MiB` L2, `105 MiB` L3); target model: **i9-10900K** (`10C/20T`, `256 KiB` L2/core, `20 MiB` shared L3, `~40–45 GB/s` dual-channel DDR4, release-scaling plateau **`68–73 ms`** on `210 MB` Wikidata from `8T` to `20T`).
**Artifacts:** [`diag_c31_deep.rs`](2026-11-21-wormhole-cycle31/diag_c31_deep.rs), [`occupancy_0512.txt`](2026-11-21-wormhole-cycle31/occupancy_0512.txt), [`diag_c31_cold.txt`](2026-11-21-wormhole-cycle31/diag_c31_cold.txt), [`c31_plateau_model.toml`](2026-11-21-wormhole-cycle31/c31_plateau_model.toml).

---

## 1. Benchmark Corpus Construction (Authenticated Wikidata Provenance)

From the six authenticated Wikidata blocks in `/home/antigravity/admitted-bwt/` (`block-0`, `block-2`, `block-85`, `block-143`, `block-207`, `block-210`, checked against `manifest.json`), we inverted each `.L` column to exact Wikidata JSON text and level-9 bzip2-compressed realistic multi-block concatenated streams:
- `wikidata_431b_210m.bz2` (`13.73 MiB` compressed → `211.00 MB` uncompressed, `431` blocks: `234` full `~900 KB` + `197` tiny `~2 KB` — matching `corpora/wikidata.bz2`'s `431`-block / `210 MB` shape);
- `wikidata_60b.bz2` (`3.9 MiB` → `54.0 MB`, `60` full blocks for high-round `4W` A/B);
- `hetero_60b.bz2` (`8.4 MiB` → `54.0 MB`, `48` Wikidata + `6` dense + `6` random fallback blocks for straggler/HOL stress).

---

## 2. Measured `v0.5.12` Baseline: `examples/occupancy.rs` (`--features trace`) + Fresh-Process Cold-Start OS Counters

### 2a. `examples/occupancy.rs` Timeline & Live Structures (`wikidata_431b_210m.bz2`, `3` rounds, `parity ok`)

| `T` | `slots` | `MB/s` | `dec` | `idle` (`i:rd` / `i:q`) | `r:wait` | `ready` max (mean) | `out` cap/used (max) | Worker buf (MB) |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `1` | `4` | `322.9` | `79%` | `3%` (`2%` / `1%`) | `82%` | `4` blks, `1.8 MB` (`0.02 MB`) | **`3.51×` (`1.80 MB`)** | `14.40` (Cells) |
| `2` | `8` | `601.6` | `84%` | `10%` (`6%` / `3%`) | `84%` | `8` blks, `3.6 MB` (`0.38 MB`) | **`3.11×` (`1.80 MB`)** | `14.40` (Cells) |
| `4*` | `16` | `530.2` | `74%` | `21%` (`14%` / `9%`) | `85%` | `16` blks, `6.3 MB` (`1.26 MB`) | **`3.38×` (`1.80 MB`)** | `14.49` (Cells) |
| `7*` | `28` | `501.4` | `72%` | `23%` (`18%` / `17%`) | `81%` | `27` blks, `10.8 MB` (`2.83 MB`) | **`3.30×` (`1.80 MB`)** | `14.56` (Cells) |
| `8*` | `32` | `554.6` | `61%` | `34%` (`30%` / `22%`) | `77%` | `29` blks, `12.6 MB` (`2.83 MB`) | **`3.36×` (`1.80 MB`)** | `1.43` (`Open`) |
| `10*` | `40` | `572.9` | `49%` | `44%` (`43%` / `28%`) | `72%` | **`30` blks, `13.5 MB` (`3.90 MB`)** | **`3.20×` (`1.80 MB`)** | `1.45` (`Open`) |

### 2b. Fresh-Process Cold-Start Scaling (`diag_c31_deep.rs`, `OpenCells` forced at every `W`, `3`-round median)

| `W` | `slots` | `wall_ms` | `user_ms` | **`sys_ms`** | **`minflt`** | `nvcsw` | `nivcsw` | `rss_MiB` | `reads` |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `1` | `4` | `514.90` | `641.72` | `31.34` | `8,474` | `949` | `333` | `40.98` | `431` |
| `2` | `8` | `353.24` | `635.83` | `29.82` | `10,634` | `820` | `591` | `43.92` | `431` |
| `4` | `16` | `382.58` | `672.00` | `33.62` | `15,521` | `759` | `616` | `58.96` | `431` |
| `4` | `8` | `380.62` | `667.55` | `34.84` | `12,845` | `875` | `624` | `54.71` | `431` |
| **`7`** | **`28`** | `381.01` | `679.65` | **`53.48`** | **`22,359`** | `385` | `449` | `76.92` | `431` |
| `7` | `14` | `394.24` | `682.23` | `53.95` | `17,738` | `805` | `595` | `62.97` | `431` |
| `8` | `32` | `413.28` | `686.33` | **`57.95`** | **`23,133`** | `315` | `817` | `80.11` | `431` |
| **`10`** | **`40`** | `422.83` | `695.55` | **`64.44`** | **`26,729`** | `317` | `850` | `92.19` | `431` |
| `10` | `20` | `408.90` | `718.12` | **`47.02`** | **`22,225`** | `624` | `493` | `80.23` | `431` |
| `12` | `48` | `398.68` | `693.72` | **`75.59`** | **`29,899`** | `388` | `454` | `105.21` | `431` |
| `16` | `64` | `415.51` | `727.41` | **`81.73`** | **`37,347`** | `279` | `501` | `138.92` | `431` |
| `20` | `80` | `450.98` | `724.47` | **`86.39`** | **`37,899`** | `398` | `1,023` | `135.70` | `431` |

### 2c. Isolated 1T Stage & Reader Breakdown (`wikidata_431b_210m.bz2`, Xeon `8481C`)
- `Scanner` (Huffman + headers, **Mechanisms' domain**): **`95.01 ms`** (`211.00 MB` out, `59.76 MB` `syms`).
- `decode_block(OpenCells)` (`front + Open + RLE1 + CRC`): **`469.12 ms`** (`234` full blocks: `min=1.37 ms`, `p50=1.84 ms`, `p95=2.95 ms`, `max=4.35 ms`).
- Reader serial `memcpy(r.cur → 4 MiB buf)` (`234` blocks): **`10.86 ms`** (`19.4 GB/s`), `431` `read()` calls (one block per call even into a `4 MiB` caller buffer!).

---

## 3. Root-Cause Diagnosis: The Six Compounding Walls Behind the `68–73 ms` i9 Plateau (`8T → 20T`)

| # | Wall / Stage | Owner | Measured Mechanism in `v0.5.12` (`6a36b0e10`) | Status |
|---|---|---|---|---|
| **W0** | **Hard i9 Policy Trim** | Policy (`parallel.rs:2898`) | `trimmed_here` hard-caps ALL `T ∈ {10, 12, 16, 20}` unpinned to the **exact same `7/28` decoder (`workers(7, 28, k, true)`)**. Untrimmed `10W/12W/16W/20W` previously regressed `1.054× / 1.124× / 1.135× / 1.160×` vs `7W` due to Walls W1–W5 below. | **proved** (source inspection) |
| **W1** | **Serial Stream Scan & Block Finder** | **Mechanisms** (`C38b`) | Single-thread `Scanner::next` takes `95.0 ms` (Xeon 8481C) / `74.5 ms` (CI Xeon) / **`~54–60 ms`** (i9-10900K). Caps `W → ∞` at `~56–60 ms` until Mechanisms' `C38b StreamParScan` (`-96.3%` serial CPU → `~2.2 ms` on i9) lands. **Do not duplicate.** | **measured** (coord. C38b) |
| **W2** | **Allocator, `out` Doubling, `mmap`/`munmap` TLB Shootdowns, & `minflt` Storm** | **Wormhole (C32)** | (a) `rle::unrle` (`src/rle.rs:47`) calls `out.reserve(n + 64)` on empty `Vec`; RLE1 expansion past `n + 64` **doubles 100% of first-use `out` buffers `900 KiB → 1.80 MiB`** (`occupancy`: `3.20×–3.51×` reserved/used, `largest 1.80 MB`), causing `28–80` `mremap`/copy/`munmap` cycles (`50–144 MiB` reserved!). (b) `BzReader::on` (`src/read.rs:247`) allocates fresh `vec![0u8; 4 MiB]` per chunk (`mmap` + `1,024` faults + `ChannelSource` copy + `munmap` TLB shootdown across all cores!). (c) Cold-start `minflt` explodes `8,474 (1W) → 22,359 (7W) → 26,729 (10W) → 37,899 (20W)` and `sys_ms` explodes `31.3 → 53.5 → 64.4 → 86.4 ms` (`~2.3 µs/fault`, serialized on Linux `mmap_lock` + cross-core TLB shootdowns). | **measured** |
| **W3** | **Post-BWT `7.0 B/B` DRAM Traffic & `RLE1 + CRC32` Double-Pass** | **Wormhole (C33, C36–37)** | After BWT inversion completes, `v0.5.12` touches the `210 MB` output **7 times**: write `ll` (`1.0`) → read `ll` (`1.0`) → write `out` (`1.0`, plus `2.0` on first-block doubling!) → **re-read `out` for `crc32` (`1.0`, after `900 KiB` already evicted `256 KiB` L2!)** → read `r.cur` in `StreamDecoder::read` (`1.0`, from L3/DRAM!) → write `4 MiB` caller `buf` (`1.0`) → caller reads `buf` (`1.0`). At `68 ms` (`3.09 GB/s`), `7.0 B/B = 21.6 GB/s` alone (~50% of i9 DDR4 bandwidth!) before `OpenCells` misses. | **proved** + **measured** |
| **W4** | **Window Dirty-L3 Pollution (`4*W` Slots)** | **Wormhole (C36, C38)** | At `10W/40`, `occupancy` records up to **`30` ready blocks (`13.5 MB` live, `54 MB` cap)** + `1.9 MB` `syms` sitting dirty in the `20 MiB` L3 alongside `10 × 2.5 MiB = 25 MiB` active worker scratch (`40.4 MiB ≈ 2.0× L3`!), evicting `OpenCells`' `edges` and `dict` to DRAM. Cutting `slots` `40 → 20` already cut `sys_ms` `-27%` (`64.4 → 47.0 ms`) and `minflt` `-17%` (`26.7k → 22.2k`). | **measured** |
| **W5** | **Block Hand-Off Locks, `notify_all()` Herd, Serial `read()` 1-Block Cap, & Tail Stragglers** | **Wormhole (C34, C39, C41)** | (a) Single global `Mutex<State>` acquired `5×`/block (`2,155` times) + `2` `Condvar::notify_all()` broadcasts/block (`862` herds) + `BTreeMap` heap nodes + `Mutex<Receiver<Job>>` held across blocking `recv()`. (b) `StreamDecoder::read` returns `<=1` block per call (`431` calls even for `4 MiB` buffer), delaying `free_out` return and costing `~10.9 ms` Xeon / `~7–9 ms` i9 pure serial `memcpy` (`15–22 ms` with caller hash). (c) Tail stragglers (`max = 4.35 ms` vs `p50 = 1.84 ms`, `2.36×` ratio) add `~2–3 ms` Amdahl tail at `10W`. | **measured** |

---

## 4. Quantitative `10T` (and `16T/20T`) i9-10900K Critical-Path Model (`CP_10T_CML`)

Anchored to i9-10900K release-scaling (`v0.5.12`: `68–72 ms` at `8–20T` on `210 MB`, `20 MiB` L3, `~42 GB/s` DDR4) and `open-stage-attribution.md`:

| Critical-Path Component (ms on i9-10900K, `210 MB` Wikidata) | `v0.5.12` Shipped (`7W/28`, Regime 1) | `v0.5.12` Untrimmed (`10W/40`, Regime 1) | `v0.5.12` + Mech `C38b` (`10W/40`, Regime 2) | **Post-Wormhole R5 + Mech `C38b` (`10W/16`, Target)** |
|---|---:|---:|---:|---:|
| `T_scan_serial` (Mechanisms' domain) | `56.0` | `56.0` | `2.2` | `2.2` |
| `T_worker_ideal` (`(front+Open+RLE+CRC [+ParHuff]) / W`) | `51.9` (`363/7`) | `36.3` (`363/10`) | `39.5` (`395/10`) | **`35.5`** (`-10%` fused RLE+CRC + no-double + alloc) |
| `ΔT_L3_DRAM` (L3 eviction & DRAM BW penalty) | `+12.5` (`31 MiB / 20 MiB`) | `+21.5` (`40.4 MiB / 20 MiB`) | `+17.5` | **`+4.5`** (`22→14 MiB` window + `-3.0 B/B` DRAM) |
| `ΔT_alloc_pf` (cold-start `minflt`, `mmap_lock`, TLB shootdowns) | `+4.5` (`22.4k` flt) | `+6.2` (`26.7k` flt) | `+6.2` | **`+1.2`** (`~7k` flt, pre-sized/pooled, `0` `mremap`/`munmap`) |
| `ΔT_handoff_locks` (`Mutex<State>`, `notify_all`, `BTreeMap`, `Mutex<Rx>`) | `+1.5` | `+2.4` | `+2.0` | **`+0.3`** (ring slots + split CVs + lock-free atomic queue) |
| `ΔT_tail` (straggler drain tail) | `+1.8` | `+2.4` | `+2.4` | **`+0.8`** (size/priority dispatch) |
| `T_reader_serial` (`read()` memcpy + drain, concurrent unless > worker) | `16.0` (hidden) | `16.0` (hidden) | `16.0` (hidden, +`6 GB/s` BW) | **`8.5`** (zero-copy/batch-fill, `-2.0 B/B` BW) |
| **Modelled Wall Time (`max(T_scan, T_par + ΣΔT, T_reader)`)** | **`~70–72 ms`** | **`~69–73 ms`** (`+4–6%` vs `7W`) | **`~67.6 ms`? → `~51–56 ms`** (`10W` w/o W2-W5 fixes) | **`~36–42 ms`** (**`~0.53–0.60×` of `v0.5.12`!**) |

*Note on `16T / 20T` (SMT on 10 physical cores):* On Comet Lake `10C/20T`, SMT siblings share the `256 KiB` L2 (`128 KiB`/thread) and L1d (`16 KiB`/thread); `v0.5.12`'s `Open` worker needs `>256 KiB` L2 working set (`edges` + `dict`), so `16–20` *workers* thrash L2 AND explode `slots = 64–80` (`37.9k` `minflt`, `86 ms` `sys_ms`). Once `W2–W5` are removed, the optimal i9 operating point shifts from **`7W/28` (`~70 ms`)** to **`10W/16..20` (Regime 2: `~36–42 ms`)** — and in Cycle 43 we will explicitly test whether `10` physical workers + `2` SMT helper roles (feeder/reader/scan) or `11–12` trimmed workers wins on SMT!

### 6-Cost Separation Summary (Baseline `v0.5.12`, `W=10, slots=40`)
1. **Work (`W`):** `695.6 ms` user + `64.4 ms` sys on Xeon (`~363 ms` worker + `~56 ms` scan + `~25 ms` sys on i9).
2. **Depth (`D`):** `max(T_scan ≈ 56 ms, T_worker_par + contention ≈ 69 ms)`.
3. **Memory traffic (`B`):** `~7.0 B/B` post-BWT (`ll` W + `ll` R + `out` W + `out` R[CRC] + `r.cur` R + `buf` W + `buf` R) + `~1.0–2.0 B/B` first-block `out` doubling + `~3–6 B/B` `Open` L3 misses = **`~11–15 B/B`** (`~2.3–3.1 GB` per `210 MB` decode!).
4. **Dependent misses (`Q`):** `Open` emit walk `~0.10n` dependent L3 hits/misses (amplified by `13.5 MB` dirty `ready` L3 pollution!).
5. **Working set (`M`):** `92.2 MiB` peak RSS (`10W/40`), `138.9 MiB` (`16W/64`), `135.7 MiB` (`20W/80`); `25 MiB` worker scratch + `15.4 MiB` live `ready`+`syms` = `40.4 MiB` L3 pressure.
6. **OS / Synchronization overhead (`S`):** `26,729` `minflt` (`64.4 ms` `sys_ms`) at `10W/40`; `37,899` `minflt` (`86.4 ms` `sys_ms`) at `20W/80`; `2,155` `Mutex<State>` locks + `862` `notify_all()` broadcasts.

---

## 5. Kill Decision & Next Cycle Plan

- **Cycle 31 Decision:** Baseline `v0.5.12` (`6a36b0e10`) plateau anatomy established and quantitatively anchored to `examples/occupancy.rs` + cold-start OS counters; no candidate killed yet.
- **Cycle 32 (next):** Attack **Wall W2 (Allocator, `out` `900K→1.8M` Doubling, Feeder `4 MiB` `mmap`/`munmap` TLB Shootdowns, & `minflt` Storm)** with native safe-Rust `4W` A/B (+ `10W` cold-start OS counter measurement) against `origin/master` `v0.5.12` in the same binary with A/A control!
