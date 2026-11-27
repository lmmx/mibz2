# Physical i9-10900K verdicts, `v0.5.15` scaling, and the `5T/7T/9T` crossover cliff

Six physical Intel Core i9-10900K (Comet Lake-S stepping 5, 10C/20T, 20 MiB L3) evidence bundles landed on `origin/evidence` on 27 November 2026: four paired kill tests, the `v0.5.15` release-scaling run, and the `v0.5.15` crossover sweep exposing the `5T/7T/9T` `trim_cell` sawtooth cliff and stale `LAZY_FROM_THREADS = 8`.

---

## 1. Six physical i9-10900K runs (`origin/evidence`, 27 Nov 2026)

Every frozen C2 verdict is preserved verbatim. Hardware runs below are physical i9-10900K measurements; local VM diagnostics are labeled separately.

| # | Subject | `origin/evidence` commit | Bundle SHA-256 prefix | Frozen verdict | Operational disposition & architectural takeaway |
|--:|---|---|---|---|---|
| 1 | `#1031` `scanner-seam-window` | `c0b2341c` | *(verified in `#1118`)* | **`NOT ADOPTED`** | PR `#1031` closed unmerged. `in_maint` fell only `29.9%` (`10.69 → 7.49 ms`). Residual dissected in `#1141`, rescued in PR `#1144` (`scanner-adaptive-seam-window`). |
| 2 | `#1053` `a6b-endpoint-build` | `a6e427b1` | `623a7096f412aabf` | **`STOP/KILL`** | Off by default (`#1151`). Establishes the **`<= 1.01×` 8W L3-footprint law**: `+524–546 KiB/w` (`+4.19 MiB` at 8W) erased `-1.76%` 1W gain on 20 MiB L3. Compact rescues refuted in `#1161` (`STOP`). |
| 3 | `#1034` `scanner-coalesced-u8-decode` | `9de7b6e0` | `0a5e398f7bd4465b` | **`STOP`** *(frozen 10T CI upper `<= 0.970`)* | Serial scan `-10.50%` (`143.5 ms`), `8T` `-1.93%`, `10T` `-4.02% [0.9457, 0.9742]`, `0/30` guard regressions. Kept in default & `v0.5.15` by owner override (`#1149`); `#1084` CI flag resolved (`#1164`). |
| 4 | `#1119` `jcc-aligned-release-build` | `77236472` | `608502c3520fa9e5` | **`STOP`** (`#1172`) | Proved **Comet Lake-S stepping 5 has the JCC silicon fix** (`W +3.05%` slower with alignment). Directly measured physical i9 `k* = W/S`: `wikidata 9.46`, `text 5.09`, `records 3.68`, `random 3.90`. |
| 5 | `v0.5.15` (`aaaee363`) `release-scaling` | `bce3914d` | `f7ef4216ee04adf9` | **Recorded (`#1174`)** | vs `v0.5.14`: `+5.7%` 1T, `+3.4%` 2T, flat 4T/8T/10T (`8T` CPU `+0.9%`), `+9.5%` 12T, `+6.3%` 20T. `5.34–8.54×` faster than `lbzip2` (`7.77–8.54×` at `8–20T` at `1.62–3.62×` lower RSS). |
| 6 | `v0.5.15` (`aaaee363`) `crossover` | `45ae3ff3` | *(recorded in `#1180`/`#1182`)* | **Diagnostic (`#1182`)** | Exposed the **`5T/7T/9T` `trim_cell` sawtooth cliff** (`7T` `0.103 s / 130 MB` vs `6T` `0.081 s / 40 MB` and `8T` `0.066 s / 42 MB`), stale `LAZY_FROM_THREADS = 8`, and `window()` slot-scaling bug. |

### 1.1 `#1031` `scanner-seam-window` (`c0b2341c`) → `NOT ADOPTED`, rescued by `#1141` + `#1144`
- `10T` (8W) cleared `1.000` (`0.9811 [0.9670, 0.9954]`, `-1.89%`), while `12T` (8W) missed the frozen `<= 1.000` upper bound (`0.9898 [0.9726, 1.0073]`); scanner `in_maint` fell only `29.93%` (`10.689 → 7.490 ms`) vs the modeled `80%` (`#1118`, `#1133`).
- `#1141` dissected the remaining `7.49 ms` residual and validated an adaptive per-block lookahead + one-shot seam window; admitted in `#1143` and staged in PR `#1144` (`scanner-adaptive-seam-window`).

### 1.2 `#1053` `a6b-endpoint-build` (`a6e427b1`, sha256 `623a7096f412aabf`) → `STOP/KILL` & the `<= 1.01×` 8W L3-footprint law
- 15 paired rounds (`source faa9df70f98d`, `+a6b-endpoint-redesign` vs default, 0 parity failures, `#1151`):
  - `wikidata/1` wall: `0.9824 [0.9776, 0.9872]` (`-1.76%`, `+24.7 ms`, RSS `+592 KiB` / `1.0077×`) — real 1W win, misses frozen `<= 0.965`.
  - `wikidata/8` wall: `1.0072 [0.9916, 1.0231]` (**RSS `+4,192 KiB` / `1.0374×`**); `wikidata/10` wall: `1.0091 [0.9783, 1.0410]` (**RSS `+5,460 KiB` / `1.0488×`**).
  - `wikidata/8/S` (ideal scanner, pure workers): `busy_ns_per_w` `1.0106 [0.9931, 1.0283]`, `wall_s` `1.0124 [0.9942, 1.0310]` — misses frozen `<= 0.965`.
  - `text/1`: `0.9919 [0.9809, 1.0029]`; `text/8`: `1.0028 [0.9818, 1.0243]`.
- **Law established:** `#1053`'s fixed 64K SoA slabs (`Box<[NodeLinks/u32/u128; 65536]>`, `1.75 MiB` virtual) added `+524–546 KiB/worker` resident RSS (`+4.19 MiB` across 8W), pushing 8-worker Comet Lake 20 MiB L3 occupancy past its cliff and erasing the 1W speedup. Any worker redesign targeting `k = 8` must keep 8W RSS `<= 1.01×`.
- **Corroboration (`#1161`, `ea56b18c`, `STOP`):** Local VM dissection reproduced i9's `-1.76%` within `0.11 pp` (`-1.87%`) and proved Linux demand-paging + `edges`-overlay refutes `Vec`/tiered-slab compact rescues of `#1053` cleanly. `#1053` stays off by default.

### 1.3 `#1034` `scanner-coalesced-u8-decode` (`9de7b6e0`, sha256 `0a5e398f7bd4465b`) → `STOP`, kept by owner override (`#1149`)
- 10 paired rounds (`head 5ed20c1d51af` vs `base 5c7fe5d24432`, `bmi2.asm`: `22 shlx`, `8 shrx`, `0 vzeroupper`, 0 parity failures, `#1151`):
  - Serial scan `phases/wikidata/1`: **`0.8950 [0.8923, 0.8978]`** (`-10.50%`, disjoint `[159.7, 160.9] → [143.1, 144.1] ms`) — **PASS** (`<= 0.920`).
  - Construction-free oracle: `C/wikidata/8` `0.9220 [0.9132, 0.9308]` (`-7.80%`), `C/wikidata/10` `0.9424 [0.9336, 0.9512]` (`-5.76%`).
  - Real `wikidata ×4`: `1T` `0.9880`, `2T` `0.9990`, `6T` `0.9996`, **`8T` `0.9807 [0.9626, 0.9991]` (`-1.93%` FASTER)**, **`10T` `0.9598 [0.9457, 0.9742]` (`-4.02%` FASTER)** — misses frozen `<= 0.970` upper CI bound by `0.0042` with overlapping ranges (`[0.1889, 0.2011]` vs `[0.1967, 0.2059]`), yielding frozen verdict **`STOP`**.
  - Guard corpora: **`0/30` regressions** (`random` 6T `0.9677` / 8T `0.9754` FASTER, `pattern` 6T `0.9724` FASTER and `1/2/8/10T` neutral `0.972–1.016`, `onebyte` 8T `0.9714` FASTER; `text`/`runs`/`records` neutral).
- **Disposition:** Retained in default & shipped in `v0.5.15` by Louis's 27 Nov decision (`#1149`). The `#1084` exact-head CI acceptance flag (`pattern` 4T `+7.3%` wall on run `45947633032`, not meeting `PERFORMANCE-POLICY.md` §2 Rule 3) was independently resolved in `#1164` (`7637add5`): the CI verdict stands as not-accepted, the change stays under owner override, and `v0.5.15` records the trade explicitly.

### 1.4 `#1119` `jcc-aligned-release-build` (`77236472`, sha256 `608502c3520fa9e5`) → `STOP` (`#1172`) & direct i9 `k*`
- 12 paired `ab run` rounds + 5×5 pinned `station_balance` rounds (`source 59cac2c5`, `recipe 256566b7`, 28/28 `libbz2` parity exact; `ab` `.text` `+2.94%`, `2,254 → 0` crossings):
  - Paired `ab run` (`aligned / default`): `wikidata/1` `1.0065 [0.9753, 1.0387]` (fails `<= 0.970`), `wikidata/4` `1.0045 [0.9967, 1.0125]`, `wikidata/8` `0.9242 [0.7860, 1.0868]` / RSS `1.0227×` (fails `<= 0.985` & RSS `<= 1.020`), `wikidata/10` `1.0226 [0.9861, 1.0605]` (fails `<= 1.010`).
  - **Direct physical i9-10900K `station_balance` (`k* = W / S`, pinned CPU 0):**
    - `wikidata`: `S` `35.24 → 35.32 ms` (`1.0017 [1.0001, 1.0033]`), `W` `333.48 → 342.65 ms` (**`1.0305 [1.0149, 1.0464]`**, **`+3.05%` slower**), **`k* = 9.46 → 9.70`**.
    - `text`: `S` `9.04 ms`, `W` `46.01 ms` → **`k* = 5.09`**.
    - `records`: `S` `19.89 ms`, `W` `73.14 ms` → **`k* = 3.68`**.
    - `random`: `S` `16.00 ms`, `W` `62.34 ms` → **`k* = 3.90`**.
- **Two decisive conclusions:** (1) Comet Lake-S stepping 5 carries Intel's JCC-erratum silicon fix — never ship `-x86-branches-within-32B-boundaries` as a release/wheel flag; retain strictly as a Skylake/Cascade Lake VM measurement control (`CG-JCC`). (2) Physical i9 station balance is settled directly: **`wikidata` (`k* = 9.46`) is worker-bound at `k = 8`**, whereas **`text` (`5.09`), `records` (`3.68`), and `random` (`3.90`) are scanner-bound at `k = 8`**.

---

## 2. `v0.5.15` (`aaaee363`) `release-scaling` (`bce3914d`, `#1174`)

Recorded in `bench/release-scaling/2026-11-27-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.5.15/` (10 rounds, Wikidata `209.7 MB` out, 100% byte-identical; SHA-256 `f7ef4216ee04adf970d8fd416c0f11e361f620d60f4078f23cc1236fdc536fb0`):

| Threads | Placement | `0.5.14` med (s) | `0.5.15` med (s) | `0.5.15` MB/s | `0.5.14/0.5.15` wall `[95% CI]` | `0.5.15` RSS (MiB) | `lbzip2` med (s) / RSS | `lbzip2 / 0.5.15` (geomean wall / RSS ratio) |
|---:|---|---:|---:|---:|---|---:|---|---|
| `1` | pinned | `0.3926` | `0.3607` | `581` | **`1.057 [1.011, 1.104]`** (`+5.7%`) | `27.7` | `2.5107` / `8.9 MiB` | `6.82×` / `0.32×` |
| `2` | pinned | `0.2258` | `0.2172` | `965` | **`1.034 [1.012, 1.055]`** (`+3.4%`) | `56.5` | `1.3314` / `19.1 MiB` | `6.17×` / `0.34×` |
| `4` | pinned | `0.1336` | `0.1331` | `1,575` | `1.000 [0.975, 1.027]` (flat) | `66.5` | `0.7140` / `33.6 MiB` | `5.34×` / `0.51×` |
| `8` | pinned | `0.0663` | `0.0663` | `3,165` | `1.002 [0.994, 1.010]` (`CPU 1.009`) | `41.8` | `0.5140` / `67.6 MiB` | `7.77×` / `1.62×` lower RSS |
| `10` | pinned | `0.0593` | `0.0594` | `3,530` | `0.996 [0.987, 1.005]` (flat) | `43.8` | `0.5057` / `91.2 MiB` | **`8.54×`** / `2.08×` lower RSS |
| `12` | unpinned | `0.0655` | `0.0600` | `3,498` | **`1.095 [1.030, 1.164]`** (`+9.5%`) | `46.5` | `0.5088` / `109.2 MiB` | `8.33×` / `2.35×` lower RSS |
| `16` | unpinned | `0.0606` | `0.0604` | `3,475` | `1.012 [0.954, 1.073]` (flat) | `46.4` | `0.4926` / `136.3 MiB` | `8.03×` / `2.94×` lower RSS |
| `20` | unpinned | `0.0639` | `0.0605` | `3,466` | `1.063 [0.990, 1.142]` (`+6.3%`) | `46.4` | `0.4911` / `167.7 MiB` | `7.91×` / `3.62×` lower RSS |

- `1T`, `2T`, and `12T` cleared the `>= 3%` disjoint attribution bar (`docs/RELEASING.md` §5b); recorded in `docs/journal/release-attribution.md` as class `unexplained` across candidates `#1034`, `#1040`, `#1080`, `#1129` (no split i9 run separated all four).
- Outside `4T` (where `0.5.15` still uses `EagerCells`), `v0.5.15` runs **`6.17–8.54×` faster than `lbzip2`** (`5.34×` at `4T`), and at `10–20T` uses **`2.08–3.62×` lower RSS** (`43.8–46.5 MiB` vs `91.2–167.7 MiB`).

---

## 3. `v0.5.15` `crossover` (`45ae3ff3`) & the `5T/7T/9T` cliff (`#1182`, `081938a8`)

The `v0.5.15` pinned i9 crossover sweep (`45ae3ff3`, 10 rounds, Wikidata `209.7 MB` out) exposed a sharp odd-thread sawtooth cliff in `src/parallel.rs`:

| `t` | `mibz2` plan | Workers `w` | Kernel | Wall med (s) | CPU med (s) | Peak RSS (MB) | `runq_s` med (ms) | Notes |
|--:|---|--:|---|--:|--:|--:|--:|---|
| `4` | `trim_cell(4)` | `3` | `Cells` | `0.132` | `0.449` | `68.0` | `26.40` | Clean trim, still `EagerCells` |
| **`5`** | **`trim_cell(5) = None`** | **`5`** | **`Cells`!** | **`0.114`** | **`0.512`** | **`97.5`** | **`123.90`** | **Odd-count cliff: 5 Eager + 124 ms `runq_s`** |
| `6` | `trim_cell(6)` | `5` | `OpenCells` | `0.081` | `0.426` | `40.2` | `33.35` | Clean trim (`OpenCells`) |
| **`7`** | **`trim_cell(7) = None`** | **`7`** | **`Cells`!** | **`0.103`** | **`0.611`** | **`129.7`** | **`113.35`** | **Cliff: `+27.2%` slower than `6T`, `+43.4%` CPU, `3.25×` RSS** |
| `8` | `trim_cell(8)` | `6` | `OpenCells` | `0.066` | `0.427` | `41.8` | `1.45` | Clean trim (`OpenCells`) |
| **`9`** | **`trim_cell(9) = None`** | **`9`** | **`OpenCells`** | **`0.060`** | **`0.449`** | **`51.5`** | **`60.10`** | **Oversubscribed: `+60.1 ms` `runq_s`, `+7.6 MB` RSS** |
| `10` | `trim_cell(10)` | `7` | `OpenCells` | `0.059` | `0.440` | `43.9` | `0.00` | Clean trim (`0.00 ms` `runq_s`) |

Dissection in `#1182` (`docs/journal/2026-11-27-explore/plan-schedule-and-lazy-threshold.md`) isolated two root causes plus two secondary bugs:
1. **Root Cause A (`trim_cell`):** `src/parallel.rs:3603-3613` only maps even pinned counts `{4, 6, 8, >=10}`. Odd pinned counts `{5, 7, 9}` fall through to untrimmed `Plan::Workers(t)`, causing `5T` and `7T` to suffer both `EagerCells` (`~18 MiB/w` thrashing 20 MiB L3) and `w = t` oversubscription (`113–124 ms` `runq_s`).
2. **Root Cause B (stale `LAZY_FROM_THREADS = 8`):** Never updated since `v0.5.0`'s pre-Open `LazyCells`. Physical i9 `kernel-count` (`run-20261125T022939Z`) already established that `OpenCells` beats `EagerCells` cleanly from `k = 4` (`-7.8%` wall / `0.49×` RSS at `k = 4`, `-16.6%` at `k = 5`, `-26.3%` at `k = 6`, `-36.1%` at `k = 7`, `-45.4%` at `k = 8`). Leaving `LAZY_FROM_THREADS = 8` hurt both odd i9 counts (`5T`/`7T`) and every non-i9 machine / cgroup-quota container at `4T–7T`.
3. **Secondary Bug 1 (`window()` slot scaling):** When `plan_with` caps workers (`n < threads`) on an 8–10 core non-i9 machine without `trimmed`, `window()` allocates `4 × threads` slots (e.g., `80` slots for `8` workers at `t = 20`) instead of `4 × n` (`32`).
4. **Secondary Bug 2 (`Cooperative` on `phys >= 11`):** `45ae3ff3` confirmed `mibz2-team2` (`Coop<2>`) runs `2.85×–4.53×` slower wall and `3.77×–9.71×` more CPU than `OpenCells` workers at `12–20T` (flagged as a separate follow-on).

`#1182` admitted `plan-schedule-and-lazy-threshold` at `status = "idea"` (P2).

---

## 4. Post-`v0.5.15` shipped wins, active review queue, and staged backlog

### 4.1 Shipped on `master` since `v0.5.15` (`aaaee363`)
- **`#1175` (`7cbbb78d`, `post-1080-takes-runs`):** Widened `takes_runs_slice` in `src/runs.rs` from `0.50` to `0.75` (`4 * len > 3 * max_len`) with a `WIDENED_RUN_SHARE = 8` (`>= 12.5%` run-symbol) guard in `(0.50, 0.75]` (0 new `unsafe`). Cuts MTF+inverse-BWT CPU `-24%..-32%` on markdown, `-14%..-25%` on ELF, `-9%..-10%` on JSON while keeping uniform noise on `Column`. Exact-head CI bench: `0/21` slower, `2` faster (`wikidata 2T -1.9%`, `pattern 1T -1.3%`).
- **`#1176` (`9001629a`, `python-readall-abi3`):** Overrode `BZ2Reader.readall()` and `BZ2Reader.read()` in `python/mibz2/__init__.py` (preserving `abi3-py310`, 0 `unsafe`, 28/28 pytest) to read 1 MiB chunks and join once instead of inheriting `io.RawIOBase.readall()`'s 25,829 × 8 KiB `readinto` loop (`-6.3..-8.5%` CPU, `-5.7..-20.0%` wall, `0.99×` RSS). Ledger row queued in PR `#1180`.

### 4.2 Active open PRs (review / workstation queue)
- **`#1117` (`scanner-a2-fast-build`):** Direct selector runs, multi-length code-length LUT, single-write table build.
- **`#1132` (`mtf-fold-l1-batch-redesign`):** L1d-batched packed maximal MTF fold + `OutQ16` `1008/1024` guard zone (`mtf-fold`).
- **`#1139` (`text-knee-i9`):** Frozen C6 registration & shallow-safe i9 recipe for pinned `text` knee probe.
- **`#1144` (`scanner-adaptive-seam-window`):** Adaptive per-block lookahead + one-shot seam window addressing `#1118`'s `7.49 ms` residual.
- **`#1168` (`forward-emission-run`):** Shallow-safe i9 kill-test recipe for `#1110`'s `forward-anchor-lanes`.
- **`#1179` (`crc32-avx2-ring-barrett-build`, B5 review):** 8-slot ROB-drained AVX2 `vpshufb` ring + 8-lane `vpclmulqdq` + zero-table Barrett reduction in `src/crc.rs` (`10.0` vs `12.0` Port-5 uops/64B, `0` `TABLES` refs, `0` `vzeroupper` `< 1280 B`; local isolated CRC `-17.4..-18.1%` on 32K–900K & `-14.1..-16.5%` L3-cold Wikidata; official CI paired `bench` on EPYC 7763: **`Better (CPU cost), and no measure worse`**, `wikidata` 1T `-0.6%` / 4T `-1.9%` CPU, `text` 1T `-1.6%`, `random` 1T `-1.4%`).
- **`#1180` (`docs/release-0-5-15-crossover`):** Records `45ae3ff3` crossover verdict in `CHANGELOG.md` and adds `#1176` to `frontier/ledger.tsv`.
- **`#1183` (`segmented-fused-postwalk`):** Fused Segmented reverse-assembly + RLE1 + L2-hot CRC32 and `u64`-store `build_lf`.

### 4.3 Smooth-zone explores admitted at `status = "idea"` (and scoped negative explores)
- **Admitted at `idea`:** `a6c-wire-redesign` (`#1152`), `scanner-a3-k3-pipe-u8` (`#1153`), `post-1080-open-admission` (`#1145`), `forward-rle1-crc-vectored-build` (`#1045`), `rle1-2load-filter-build` (`#1178`, `63cfc924`), `plan-schedule-and-lazy-threshold` (`#1182`, `081938a8`).
- **Scoped negative explores (`STOP` on 27 Nov):** `#1161` (A6b compact-footprint rescue `STOP`), `#1162` (non-Open `src/mtf.rs` `mtf_checkpoints_avx2` & `mtf_avx2` `STOP`), `#1166` (bzip2 Huffman self-synchronisation & selector-count obstruction `STOP`).
