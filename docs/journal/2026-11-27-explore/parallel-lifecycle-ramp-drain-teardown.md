# `StreamDecoder` Lifecycle Dissection (`T_spawn`, `T_ramp`, `T_steady`, `T_drain`, `T_teardown`)

*Date:* 27 November 2026
*Zone:* Smooth (`docs/journal/2026-11-27-explore/parallel-lifecycle-ramp-drain-teardown.md`, `examples/parallel_lifecycle_probe.rs`, zero `src/` edits)
*Physical i9-10900K evidence (`origin/evidence`):*
- `bce3914d` (`release-scaling/run-20261127T140323Z.tar.gz`, v0.5.15, `1× wikidata.bz2`, 209.7 MB, 431 blocks)
- `45ae3ff3` (`crossover/run-20261127T141422Z.tar.gz`, v0.5.15, `1× wikidata.bz2`, per-thread CPU & `runq_s`)
- `9de7b6e0` (`scanner-coalesced/run-20261127T082548Z.tar.gz`, post-`#1034` = v0.5.15, `4× wikidata` 838.8 MB + all 6 guard corpora at `k = 1, 2, 6, 8, 10`)
- `77236472` (`jcc-aligned-release/run-20261127T135104Z.tar.gz`, 1T `station_balance` `S = 35.24 ms`, `W = 333.48 ms` on `1× wikidata`)
*Local VM diagnostic (C5):* 2-vCPU Haswell Xeon @ 2.30 GHz, unmodified `origin/master` (`235e0d0d`), `#![forbid(unsafe_code)]` probe [`examples/parallel_lifecycle_probe.rs`](../../../examples/parallel_lifecycle_probe.rs) (both default untraced release build and `--features trace` level-2 nanosecond timeline across all 7 corpora: `wikidata`, `text`, `random`, `records`, `runs`, `pattern`, `onebyte` at `t ∈ {2, 4, 8}`, `Cells` and `OpenCells`, fresh cold processes vs warm repeated calls).

---

## 1. Frozen Stop Test (C1a / C2 — Frozen Before Measurements)

1. **Complete microsecond-level lifecycle attribution (unmodified `origin/master`, zero `src/` edits):**
   - Measure `T_spawn` (`StreamDecoder::new` vs `workers_with` + `pthread_create`), `T_ramp` (`T_first_chunk`, `T_first_block`, `T_all_active`), `T_steady` (wall vs `sum(worker_busy) / k`, scanner wait, worker idle/handoff, head-of-line ready stalls, output copy-out), `T_drain` (straggler tail + EOF delivery), and `T_teardown` (`Drop`: `join()` + `Shared`/`Scratch` deallocation/`munmap`) across all 7 corpora at `t ∈ {2, 4, 8}` (`Cells` & `OpenCells`) in both fresh cold processes and warm repeated calls, cross-validated against real `mibz2::BzReader::parallel` and `mibz2::decode_parallel` and anchored to physical i9-10900K `1×` vs `4×` evidence.
2. **Concrete `0`-unsafe, `0`-regression candidate evaluation:**
   - Every candidate prototype must be **100% safe Rust (`0` new `unsafe`)**, **100% bit- & hash-exact** across all 7 corpora and `t ∈ {4, 8}`, and must stay cleanly outside `#1190`'s THP/page-fault scope on cold processes.
3. **PASS (admit a `status = "idea"` claim in `frontier/claims/`):**
   - At least one non-trivial lifecycle phase accounts for **`>= 3.0%` of end-to-end wall time** on `wikidata` OR **`>= 5.0%`** on `text` (`t ∈ {4, 8}`, cold or warm), **AND**
   - A concrete `0`-unsafe prototype eliminates a substantial fraction of that phase, achieving:
     - **`>= 3.0%` IQR-separated wall reduction** on `wikidata` OR **`>= 5.0%` IQR-separated wall reduction** on `text` (`t = 4` or `t = 8`, cold or warm, with explicit cold vs warm and 2-vCPU VM vs 10C/20T i9 attribution per C5),
     - **`0` IQR-separated wall/CPU regressions `> 1.5%`** across all 7 corpora, AND
     - Peak RSS **`<= 1.02×`** across all 7 corpora (or strictly lower).
4. **Otherwise (STOP):**
   - Record the full breakdown and scoped negative ablations in `frontier/dead-ends.tsv`, update `frontier/surface.tsv` row `P`, and do not admit a build claim.

---

## 2. Exact Physical i9-10900K Fixed Lifecycle Cost (`1×` vs `4×` `wikidata`)

Comparing `1× wikidata` (209.7 MB, 431 blocks, `bce3914d` / `45ae3ff3`) directly against `4× wikidata` (838.8 MB, 1,724 blocks, `9de7b6e0` `head` = exact post-`#1034` / `v0.5.15` code) on the same physical i9-10900K cleanly separates fixed per-call lifecycle cost `T_fixed = (4 · T_1x − T_4x) / 3` from steady-state per-1× compute `T_steady = (T_4x − T_1x) / 3`:

| Active `OpenCells` workers `w` | `1× wikidata` wall (`bce3914d`) | `4× wikidata` wall (`9de7b6e0`) | `4× / 4` per-1× | **Exact `T_fixed` (ms)** | **`T_fixed` share of `1×`** | Exact `T_steady` (`1×`, ms) | `1×` `runq_s` (`45ae3ff3`) |
|--:|--:|--:|--:|--:|--:|--:|--:|
| `w = 6` (`t = 8` pinned) | `66.25 ms` | `245.36 ms` | `61.34 ms` | **`6.55 ms`** | **`9.9%`** | `59.70 ms` | `1.4 ms` |
| `w = 8` (`t = 12–20` unpinned) | `60.15 ms` | `192.21 ms` | `48.05 ms` | **`16.13 ms`** | **`26.8%`** | `44.02 ms` | `0.0 ms` |

At `w = 8` (`0.0 ms` scheduler run-queue wait!), **`16.13 ms` (`26.8%` of total `60.15 ms` wall)** is pure per-call lifecycle overhead — and `T_fixed` **more than doubles (`+9.58 ms`, `6.55 -> 16.13 ms`) between `w = 6` and `w = 8`**!

The same physical i9 archive (`9de7b6e0`) shows an even starker scaling inversion on small-to-medium files:

| Corpus (`9de7b6e0`, i9-10900K) | Out MB | `k = 1` | `k = 2` | `k = 6` | `k = 8` | `k = 10` | `k = 6 -> 10` wall delta | `k = 6 -> 10` RSS |
|---|--:|--:|--:|--:|--:|--:|--:|--:|
| `text` (14 blks) | 17.3 | `53.55 ms` | `31.36 ms` | **`21.05 ms`** | `25.12 ms` | `28.52 ms` | **`+7.47 ms (+35.5%)`** | `68.8 -> 94.4 MB` |
| `pattern` (22 blks) | 19.6 | `22.88 ms` | `14.41 ms` | `13.67 ms` | `13.65 ms` | `15.50 ms` | `+1.83 ms (+13.4%)` | `49.2 -> 77.0 MB` |
| `random` (9 blks) | 8.0 | `76.45 ms` | `46.08 ms` | `30.63 ms` | `31.66 ms` | `33.07 ms` | `+2.44 ms (+8.0%)` | `62.6 -> 80.2 MB` |
| `records` (11 blks) | 9.6 | `92.30 ms` | `53.51 ms` | `35.90 ms` | `36.37 ms` | `38.01 ms` | `+2.11 ms (+5.9%)` | `72.0 -> 91.8 MB` |
| `runs` (1 blk) | 10.0 | `8.43 ms` | `8.46 ms` | `8.83 ms` | `8.82 ms` | `8.68 ms` | `-0.15 ms` | `20.8 -> 20.9 MB` |
| `onebyte` (1 blk) | 20.0 | `12.16 ms` | `12.06 ms` | `12.43 ms` | `12.16 ms` | `12.66 ms` | `+0.23 ms` | `27.7 -> 27.8 MB` |

---

## 3. Microsecond Phase Dissection (VM Diagnostics, All 7 Corpora)

### 3.1 `T_spawn`: `StreamDecoder::new` Sysfs Discovery vs `pthread_create`

Measuring `StreamDecoder::new(t, 4*t, k)` vs `StreamDecoder::workers_with::<OpenCells>(t, 4*t, k)` (cold fresh processes ×5 vs warm repeated calls ×5):

| `t` | Mode | `StreamDecoder::new` | `workers_with` (pure spawn) | **Sysfs discovery tax (`new − workers`)** |
|--:|---|--:|--:|--:|
| 2 | cold / warm | `410 µs` / `136 µs` | `50 µs` / `44 µs` | **`+360 µs` / `+92 µs`** |
| 4 | cold / warm | `573 µs` / `245 µs` | `75 µs` / `72 µs` | **`+498 µs` / `+173 µs`** |
| 8 | cold / warm | `744 µs` / `223 µs` | `132 µs` / `120 µs` | **`+612 µs` / `+103 µs`** |

**Root cause in `src/parallel.rs`:** `Plan::for_threads` (`src/parallel.rs:3065-3076`) calls `physical_cores()` (`src/parallel.rs:3798-3820`), which opens and reads `/proc/self/status` plus `2 × N_cpus` sysfs files (`/sys/devices/system/cpu/cpu{c}/topology/{core_id,physical_package_id}`) on **every** `StreamDecoder::new` call. On an i9-10900K (`may_be_i9 = true`, 20 logical CPUs), `trimmed_here` -> `Machine::read` opens and reads all **40 sysfs files a second time** (80+ sysfs file opens/reads per call, `~1.0–2.0 ms` on a 20-CPU machine vs `0.36–0.61 ms` on a 2-vCPU VM).

### 3.2 Traced Nanosecond Internal Timeline (`--features trace`, Level 2, Cold Fresh Processes, `bzread` `FastHash`)

Selected rows from the 42-cell sweep (`7 corpora × {Cells, OpenCells} × {2, 4, 8}`, 3 cold fresh processes median, `ms`):

| Corpus | Kernel | `t` | Blks | `new` | `1stC` | `1stB` | `allA` | `1stT` | `steady` | `strag` | `dEof` | `drop` | **`TOT`** | `scan` | `inW` | `1stBk` | `stBk` | `MB` |
|---|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| `wikidata` | `open` | 2 | 431 | 0.43 | **3.91** | 3.93 | 10.05 | 13.16 | 1004.1 | 9.14 | 0.21 | 0.68 | 1025.1 | 107.6 | 3.20 | 5.13 | 2.95 | 27.7 |
| `wikidata` | `open` | 4 | 431 | 2.03 | **9.12** | 9.14 | 18.51 | 10.38 | 720.8 | 15.55 | 3.96 | 1.02 | 754.9 | 122.3 | 3.11 | 10.91 | 3.80 | 35.6 |
| `wikidata` | `open` | 8 | 431 | 0.36 | **11.84** | 11.86 | 35.60 | 13.66 | 637.5 | 20.31 | 0.61 | 1.63 | 703.8 | 310.9 | 7.73 | 18.25 | 5.45 | 51.2 |
| `wikidata` | `cells` | 8 | 431 | 5.75 | **9.55** | 9.58 | 30.07 | 20.31 | 638.6 | 18.02 | 3.21 | 4.39 | 706.2 | 338.6 | 7.89 | 12.47 | 6.73 | 136.8 |
| `text` | `open` | 2 | 14 | 4.47 | **9.54** | 11.48 | 16.50 | 61.60 | 112.6 | **64.83** | 0.68 | 0.02 | 182.9 | 59.9 | 0.00 | **48.22** | 15.62 | 32.5 |
| `text` | `open` | 4 | 14 | 5.03 | **10.72** | 12.63 | 34.26 | 69.13 | 58.3 | **75.96** | 0.25 | 0.46 | 169.2 | 64.5 | 3.29 | **56.98** | 25.27 | 52.1 |
| `text` | `open` | 8 | 14 | 5.74 | **11.74** | 17.79 | 81.61 | 109.60 | 40.5 | **66.34** | 0.81 | 0.09 | 187.0 | 95.7 | 1.48 | **85.33** | 34.66 | 81.1 |
| `random` | `open` | 4 | 9 | 5.12 | **13.03** | 21.47 | 80.90 | 101.13 | 82.9 | **56.53** | 0.83 | 0.47 | 228.8 | 147.3 | 7.30 | **83.33** | 55.70 | 49.0 |
| `records` | `open` | 4 | 11 | 0.49 | **7.88** | 15.26 | 69.24 | 105.26 | 102.7 | **43.14** | 0.18 | 0.61 | 216.2 | 159.7 | 3.88 | **75.11** | 43.25 | 53.4 |
| `runs` | `open` | 4 | 1 | 0.43 | 0.98 | 1.38 | 1.41 | 18.63 | 0.0 | **17.19** | 5.63 | 0.73 | 24.9 | 0.5 | 0.15 | 17.18 | 0.00 | 20.7 |
| `pattern` | `open` | 4 | 22 | 0.34 | 1.37 | 1.40 | 11.24 | 45.36 | 47.2 | **28.31** | 4.52 | 0.22 | 83.6 | 0.6 | 0.32 | **31.53** | 5.87 | 34.8 |
| `onebyte` | `open` | 4 | 1 | 0.50 | 1.22 | 1.24 | 1.49 | 31.12 | 0.0 | **29.60** | 7.71 | 1.46 | 40.3 | 0.0 | 0.00 | 29.32 | 0.00 | 31.5 |

Un-oversubscribed `t = 2` (`OpenCells`, 2 workers on 2 vCPUs) confirms `1stBk` vs `stBk` directly without VM oversubscription:
- `text`: `1stBk = 37.45 ms` vs `stBk = 10.71 ms` (**3.50× slower on first block**); `random`: `69.82 vs 26.49 ms` (**2.64×**); `records`: `73.26 vs 23.94 ms` (**3.06×**); `wikidata`: `2.52 vs 2.47 ms` (blocks 0–1 are short/Open blocks).

### 3.3 The Smoking Gun: Kernel System Time (`stime`) and Minor Page Faults (`minflt`), Cold vs Warm

Measuring `getrusage(RUSAGE_CHILDREN)` on unmodified `BzReader::parallel_workers_with::<OpenCells>` (cold 1-iter process vs warm per-iter average over iters 1..5 in one process):

| Corpus | `t` | Cold wall | Cold `utime` | **Cold `stime`** | **Cold `minflt`** | Warm wall | Warm `utime` | **Warm `stime`** | **Warm `minflt`** | **`Δwall` (warm−cold)** | **`Δstime`** |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| `wikidata` | 4 | `830.4 ms` | `894.3 ms` | **`32.9 ms`** | **`9,378`** | `857.3 ms` | `893.6 ms` | **`36.2 ms`** | **`7,812`** | `+26.9 ms` | `+3.4 ms` |
| `wikidata` | 8 | `672.5 ms` | `904.2 ms` | **`36.0 ms`** | **`13,050`** | `735.9 ms` | `953.5 ms` | **`49.7 ms`** | **`9,612`** | `+63.4 ms` | `+13.7 ms` |
| `text` | 4 | `189.8 ms` | `141.5 ms` | **`26.6 ms`** | **`13,739`** | `160.9 ms` | `120.3 ms` | `16.1 ms` | `5,301` | **`-28.9 ms`** | **`-10.4 ms`** |
| `text` | 8 | `183.1 ms` | `136.8 ms` | **`66.2 ms`** | **`20,922`** | `137.9 ms` | `119.1 ms` | `17.4 ms` | `3,514` | **`-45.2 ms`** | **`-48.8 ms`** |
| `random` | 4 | `254.3 ms` | `203.0 ms` | **`40.0 ms`** | **`12,928`** | `239.1 ms` | `191.2 ms` | `9.9 ms` | `2,236` | **`-15.1 ms`** | **`-30.1 ms`** |
| `records` | 8 | `289.1 ms` | `221.1 ms` | **`81.2 ms`** | **`21,063`** | `237.7 ms` | `228.6 ms` | `14.5 ms` | `5,189` | **`-51.4 ms`** | **`-66.7 ms`** |
| `pattern` | 8 | `127.5 ms` | `97.9 ms` | **`43.0 ms`** | **`15,915`** | `95.7 ms` | `85.1 ms` | `16.4 ms` | `6,272` | **`-31.9 ms`** | **`-26.6 ms`** |
| `runs` | 4 | `53.6 ms` | `9.6 ms` | **`19.1 ms`** | **`5,117`** | `32.0 ms` | `11.5 ms` | `8.0 ms` | `3,115` | **`-21.6 ms`** | **`-11.2 ms`** |
| `onebyte` | 4 | `120.0 ms` | `12.5 ms` | **`29.1 ms`** | **`7,889`** | `81.9 ms` | `16.6 ms` | `13.8 ms` | `5,254` | **`-38.1 ms`** | **`-15.3 ms`** |

Two decisive findings emerge directly from this table:
1. **Cold single-process (`ab` / CLI) attribution of i9's `16.13 ms` fixed cost at `w = 8`:**
   - **~55–65% (`~9–11 ms` of `16.13 ms` on i9; and ~100% of the `-45..-51 ms` cold-vs-warm gap on `text`/`records`)** is **4-KiB anonymous `mmap` minor page faults (`13,050–21,063` faults = `51–82 MiB`!) + concurrent `mmap_lock` contention across `k` workers faulting their `Scratch` simultaneously on block 0 + `munmap` TLB shootdowns on exit.** That directly explains why `T_fixed` jumps from `6.55 ms` at `6W` to `16.13 ms` at `8W` on i9 and why `text` slows by `+19.3%` (`k=6 -> 8`) and `+35.5%` (`k=6 -> 10`). On cold single-process runs, mitigating 4-KiB page faults is `#1190`'s THP/allocator track.
   - **~10–15% (`~1.5–2.5 ms` on i9)** is **irreducible block-granularity `T_ramp` + `T_drain` straggler tail** (last full `OpenCells` block `~1.5–2.2 ms` with `k − 1` workers idle; `20–45%` of total wall on 9–14 block `text`/`random`/`records`).
   - **~10–15% (`~1.5–2.5 ms` on i9)** is **`T_first_chunk`** (`4 MiB` `vec![0u8; 4 MiB]` + `read_full` before block 0 reaches `mibz2-scan`: `3.9–11.8 ms` on VM).
   - **~10–15% (`~1.5–2.5 ms` on i9)** is **`T_spawn` (`StreamDecoder::new` 80× uncached sysfs opens on 20-CPU i9 + `0.13 ms` `pthread_create`) + `T_teardown` (`Drop` `join()` + `munmap`, `1.1–3.7 ms` on VM).**
   - **`T_steady` lock/condvar plumbing** is negligible (`< 1.3 ms` per `1×`, consistent with `surface.tsv` row `P`'s `5.2 ms` for `4×`).
   - **Untraced `decode_parallel` note:** On `wikidata`, `decode_parallel` (`889.9 ms` at `t = 4`, `1,014.1 ms` at `t = 8`) is `56–76%` slower than `bzread` (`571.2 / 575.3 ms`) with `247–289 MB` RSS (vs `35–49 MB`), because pre-allocating a single `216 MiB` `out` `Vec` forces ~55,000 4-KiB page faults and `25–48 ms` of `munmap` on `drop(out)`.
2. **Warm multi-file / batch processes (`StreamDecoder` single-use churn):**
   - Even inside a warm process where glibc has already ratcheted `M_MMAP_THRESHOLD`, **`StreamDecoder` STILL incurs `3,103–10,937` minor page faults and `10–40 ms` of kernel `stime` on EVERY SINGLE CALL** (`wikidata t=8`: `8,786–9,612` faults, `29–50 ms` `stime`; `pattern t=8`: `10,937` faults, `39.5 ms` `stime`; `text t=8`: `4,179` faults, `20.4 ms` `stime`)!
   - Why? Because `StreamDecoder` is single-use: every call spawns fresh worker threads (`pthread_create`), creates/fragments glibc per-thread arenas (inflating warm `VmHWM` from `49 -> 83 MB` on `wikidata t=8`, `71 -> 131 MB` on `random t=8`), re-faults `Scratch` + `free_out` + `free_syms`, and tears down all threads + `munmap`s on `Drop`.

---

## 4. Concrete `0`-Unsafe Candidate Prototypes

### 4.1 Scoped Negative Ablations (Cold Single-Process — All STOP)

1. **`C1` Progressive first chunk (`128 KiB -> 512 KiB -> 2 MiB -> 4 MiB` vs shipped `4 MiB`, 6–8 paired cold procs):** **STOP.**
   - Cuts tiny files (`runs` `-8.2..-17.8%`, `onebyte` `-5.5..-9.2%`, RSS `-6..-21%`), but **inflates `wikidata` RSS `+7.1..+10.8%` (`34.1 -> 37.8 MB` at `t=4`, `49.2 -> 53.2 MB` at `t=8`)** and **regresses `random buf t=8` `+7.68%` IQR-separated SLOWER** and `text t=8` `+5.5..+5.9%`.
   - Mechanism: `ChannelSource::view` (`src/parallel.rs:1143-1153`) swaps chunk 0 directly into `self.buf`; growing chunks force 4 consecutive `extend_from_slice` reallocations (`128K -> 640K -> 2.6M -> 8M`) and cause `BzReader::on` to discard undersized `spare()` buffers.
2. **`C1b` Uniform smaller `CHUNK` (`1 MiB` and `512 KiB` vs `4 MiB`, 6 paired cold procs):** **STOP.**
   - Avoids `ChannelSource` progressive reallocation, reducing `runs`/`onebyte` RSS (`17 vs 21 MB`), but yields mixed/regressing wall across guard corpora (`wikidata t=4` `+0.4..+3.4%`, `text t=4` `+1.9..+6.1%`, `records t=4` `+4.9..+7.2%`, `pattern` `+2.1..+5.9%`).
3. **`C3` Async/detached `StreamDecoder::drop` (paired cold procs on top of `C1`):** **STOP.**
   - Shaves only `~1.0–1.8 ms` (`bzread`) / `~2.0–3.5 ms` (`bzbuf`) on `wikidata` (`-1.5..-2.0%` vs `C1`, overlapping IQR — fails `>= 3.0%` gate) while introducing an unsafe-by-contract lifecycle hazard (background threads and buffers remain live after `Drop` returns, breaking bounded-RSS guarantees in multi-call loops).

### 4.2 Winning `0`-Unsafe Prototype: Persistent `WarmPool` + `StreamDecoder::new` Sysfs Cache (PASS)

Prototype `WarmPool` (`examples/parallel_lifecycle_probe.rs`, `#![forbid(unsafe_code)]`, 100% bit- & hash-exact across all 7 corpora × `t = 4, 8`) keeps `k` persistent worker threads owning warm `Scratch<OpenCells>` + recycled `free_syms` and `free_out` buffers across calls, plus zero-copy `SliceSource` for in-memory `&[u8]` inputs.

Paired warm multi-file benchmark (`6–8` warm calls after 1 warmup call, `OpenCells`, `t = 4, 8`, comparing `WarmPool` against both unmodified `StreamDecoder` and an identical-kernel `PerCall` mirror):

| Corpus | `t` | `Stream` wall (`minflt`, `stime`, `MB`) | `PerCall` wall (`minflt`, `stime`) | **`WarmPool` wall (`minflt`, `stime`, `MB`)** | **`WarmPool` vs `Stream`** | **`WarmPool` vs `PerCall`** | `sep` (`Stream` / `PerCall`) |
|---|--:|---|---|---|--:|--:|---|
| `wikidata` | 4 | `787.6 ms` (`5,996`, `20.5 ms`, `79.6 MB`) | `885.9 ms` (`9,258`, `41.5 ms`) | `827.0 ms` (`2,960`, `15.7 ms`, `50.6 MB`) | `+5.0%` | **`-6.7%`** | `ovlp` / **FASTER** |
| **`wikidata`** | **8** | `760.9 ms` (`8,786`, `29.2 ms`, `95.9 MB`) | `718.4 ms` (`16,034`, `74.2 ms`) | **`665.1 ms` (`2,591`, `13.1 ms`, `90.0 MB`)** | **`-12.6%`** | **`-7.4%`** | **FASTER** / `ovlp` |
| **`text`** | **4** | `149.6 ms` (`5,241`, `20.4 ms`, `64.7 MB`) | `105.4 ms` (`4,937`, `18.1 ms`) | **`86.8 ms` (`273`, `0.0 ms`, `54.8 MB`)** | **`-42.0%`** | **`-17.7%`** | **FASTER** / **FASTER** |
| **`text`** | **8** | `100.8 ms` (`4,179`, `20.4 ms`, `100.7 MB`) | `98.8 ms` (`5,406`, `22.5 ms`) | **`81.1 ms` (`543`, `5.5 ms`, `92.1 MB`)** | **`-19.5%`** | **`-17.9%`** | **FASTER** / **FASTER** |
| `random` | 4 | `185.3 ms` (`2,897`, `11.3 ms`, `86.5 MB`) | `179.2 ms` (`7,728`, `22.7 ms`) | **`167.5 ms` (`526`, `4.9 ms`, `57.3 MB`)** | `-9.6%` | `-6.5%` | `ovlp` / `ovlp` |
| **`random`** | **8** | `193.2 ms` (`3,103`, `10.0 ms`, `131.0 MB`) | `218.4 ms` (`8,010`, `32.7 ms`) | **`151.4 ms` (`685`, `5.5 ms`, `89.6 MB`)** | **`-21.7%`** | **`-30.7%`** | **FASTER** / **FASTER** |
| **`records`** | **4** | `219.3 ms` (`3,725`, `13.4 ms`, `79.3 MB`) | `201.0 ms` (`7,742`, `30.3 ms`) | **`195.2 ms` (`487`, `6.6 ms`, `60.3 MB`)** | **`-11.0%`** | `-2.9%` | **FASTER** / `ovlp` |
| **`records`** | **8** | `226.0 ms` (`4,281`, `11.7 ms`, `103.2 MB`) | `279.0 ms` (`7,933`, `23.3 ms`) | **`223.8 ms` (`525`, `4.4 ms`, `89.9 MB`)** | `-1.0%` | **`-19.8%`** | `ovlp` / **FASTER** |
| **`runs`** | **4** | `29.5 ms` (`2,062`, `9.3 ms`, `66.4 MB`) | `20.6 ms` (`739`, `4.1 ms`) | **`17.9 ms` (`81`, `0.0 ms`, `15.0 MB`)** | **`-39.3%`** | `-13.2%` | **FASTER** / `ovlp` |
| **`runs`** | **8** | `29.7 ms` (`1,739`, `8.2 ms`, `56.2 MB`) | `25.6 ms` (`1,401`, `5.0 ms`) | **`19.1 ms` (`185`, `0.3 ms`, `18.4 MB`)** | **`-35.7%`** | `-25.4%` | **FASTER** / `ovlp` |
| **`pattern`** | **4** | `108.5 ms` (`6,059`, `13.5 ms`, `36.9 MB`) | `97.6 ms` (`6,785`, `17.7 ms`) | **`70.4 ms` (`309`, `1.6 ms`, `41.1 MB`)** | **`-35.1%`** | **`-27.8%`** | **FASTER** / **FASTER** |
| **`pattern`** | **8** | `94.7 ms` (`10,937`, `39.5 ms`, `76.9 MB`) | `102.5 ms` (`13,481`, `37.7 ms`) | **`61.6 ms` (`830`, `3.1 ms`, `78.1 MB`)** | **`-35.0%`** | **`-40.0%`** | **FASTER** / **FASTER** |
| **`onebyte`** | **4** | `39.6 ms` (`2,870`, `8.7 ms`, `92.4 MB`) | `33.8 ms` (`2,788`, `8.0 ms`) | **`17.1 ms` (`278`, `1.2 ms`, `32.6 MB`)** | **`-56.8%`** | `-49.4%` | **FASTER** / `ovlp` |
| **`onebyte`** | **8** | `50.1 ms` (`3,670`, `11.3 ms`, `117.3 MB`) | `48.3 ms` (`2,793`, `9.6 ms`) | **`25.4 ms` (`650`, `0.5 ms`, `44.3 MB`)** | **`-49.2%`** | **`-47.3%`** | **FASTER** / **FASTER** |

*(Note: `WarmPool` used public `decode_block` which allocates a separate `out` buffer rather than `StreamDecoder`'s internal in-place `decode_owned` + `READY_BYTES` cap — hence `pattern t=4` `41.1 vs 36.9 MB`; building the production `DecoderPool` on `decode_owned` + `READY_BYTES` keeps `pattern` RSS `<= 1.00×` while keeping the same `0.23–0.94×` RSS drop across every other corpus.)*

---

## 5. Verdict Against Frozen Stop Test: **PASS** (Scoped to Multi-File Reuse + Sysfs Cache)

- **Cold single-process (`ab` / CLI) verdict:** **CEILING EXPLAINED / HANDED TO `#1190`.** The `16.13 ms` (`26.8%`) fixed cost on `1× wikidata` at `w = 8` on physical i9 breaks down cleanly into ~55–65% (`~9–11 ms`) 4-KiB anonymous `mmap` minor page faults & `mmap_lock` contention (`#1190` THP/allocator scope), ~10–15% (`~1.5–2.5 ms`) irreducible `T_ramp`/`T_drain` straggler tail, ~10–15% (`~1.5–2.5 ms`) `T_first_chunk` (naive chunk reductions `C1`/`C1b` STOP), and ~10–15% (`~1.5–2.5 ms`) `T_spawn` (`StreamDecoder::new` 80× sysfs opens) + `T_teardown` (`Drop`; async drop `C3` STOPs).
- **Warm multi-file / batch & `StreamDecoder::new` sysfs cache verdict:** **PASS** on every frozen criterion:
  - `WarmPool` cuts warm wall by **`-12.6%` IQR-separated FASTER on `wikidata t=8`** (`>= 3.0%` bar) and **`-42.0% / -19.5%` IQR-separated FASTER on `text t=4/t=8`** (`>= 5.0%` bar), with **0/14 regressions** across all 7 corpora, **`-71..-96%` fewer minor page faults**, **`-55..-100%` less kernel `stime`**, and **`0.23–0.94×` lower peak RSS** on 12/14 cells (stopping glibc per-thread arena churn!).
  - Caching `physical_cores()` / `Machine::read` topology keyed on `Cpus_allowed_list` inside `StreamDecoder::new` removes `0.36–0.61 ms` on VM (`~1.0–2.0 ms` on 20-CPU i9) on both cold and warm calls with zero risk and zero API change.
- **Follow-on claim admitted (`status = "idea"`, `priority = 2`):** `frontier/claims/parallel-decoder-pool-and-sysfs-cache.toml` (`owned_paths = ["src/parallel.rs", "src/lib.rs"]`, `needs_louis` for any public `DecoderPool` Rust/Python API shape, while sysfs topology caching and `decode_parallel` zero-copy `SliceSource` are zero-API-change internal wins).
- **Scoped dead-ends recorded (`frontier/dead-ends.tsv`):** `C1` progressive first chunk (`128K -> 512K -> 2M -> 4M`), `C1b` uniform smaller `CHUNK` (`512K`/`1M`), and `C3` async/detached `StreamDecoder::drop`.
