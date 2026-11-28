# Block-Size Route i9 Calibration Verdict (`block-size-route-i9-probe`): **STOP**

- **Claim:** `frontier/claims/block-size-route-i9-probe.toml` (C1a diagnostic probe, Step 1 of `frontier/claims/open-admission-block-size.toml`)
- **Frozen recipe:** `just submit bench-block-size-route` (`bench/block_size_route.py`, `.just/block-size-route.just`, PR `#1369`, queued in `#1370`)
- **Frozen registration:** `docs/journal/2026-11-28-block-size-route-i9/registration.md`
- **Commit measured:** `80dc353478f38c16e9d9cc4561dc9d18ee807831` (`ready/bench-block-size-route`)
- **Evidence bundle:** `origin/evidence` `7f03816ba0d67bd4a744f0c550dd15c769d7e433` — `block-size-route/run-20261128T140651Z.tar.gz` (`28,352` bytes), SHA-256 `47205b4bf4bab4df4618c5cbed4683f4a7907c0808d1db6488246cfc271f6d9f` (verified identical across `.tar.gz.sha256`, commit message, and noticeboard `#145`; runtime `14:06:50Z -> 14:11:35Z` = `4m 45s`).
- **Host & validity:** physical `Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz` (`256 KiB` L2/core, `20 MiB` shared L3), 10 physical cores / 20 logical CPUs unrestricted (`cpus_allowed = 0..19`), `cpu_quota = unlimited`, `throttling = false`, `strict_i9 = true`, `smoke = false`, `rounds = 3`, `reps = 2`, `settle_s = 2`, `pin_cpu = 0` (`taskset -c 0`), `ROUTE_FULL = 90000`, baseline x86-64 codegen (`RUSTFLAGS` and `CARGO_PROFILE_RELEASE_*` unset).
- **Parity (`7/7 PASS`):** every re-encoded `.bz2` file independently verified byte- and SHA-256-exact via C `libbzip2` (`wikidata-L{1,2,3,5,9}.bz2` = `209,700,196 B`, `3f075e30f4f8c734...`; `text-L{1,9}.bz2` = `15,151,880 B`, `5a0e4b039e96504b...`), plus `examples/route_census`'s `OpenCells == Segmented == LazyCells` assertion on every sweep.

## 1. Frozen Stop-Test Verdict: **STOP**

Frozen C1a rule (`docs/journal/2026-11-28-block-size-route-i9/registration.md` §4, matching `open-admission-block-size.toml` Step 1): **`CONFIRMED`** iff at least one `< 400k` row-count bin (`>= 10` full `Open` blocks/sweep) on `wikidata` or `text` reaches pooled `Open/Seg >= 1.05` across both `ROUTE_SHADOW=1` and `ROUTE_SHADOW=first`; else **`STOP`**.

Re-running `python3 bench/block_size_route.py score` offline on the verified bundle reproduces the workstation outcome bit-for-bit:

> **`STOP: no sub-400k row bin reached Open/Seg >= 1.05 (max=1.0321); VM-L2 artefact on physical i9`**

### Per-Level Pooled `Open / Segmented` (`3` rounds × `2` orders = `6` sweeps, core-0 pinned)

| Corpus / Level | Full `Open` blocks/sweep | Physical i9 Pooled `Open/Seg` | `ROUTE_SHADOW=1` (Open first) | `ROUTE_SHADOW=first` (Seg first) | Blocks Open Wins | Per-Block Oracle | Prior 2 MiB-L2 VM (`#1355`) |
|---|---:|---:|---:|---:|---:|---:|---:|
| **`wikidata -1`** (`99,981` rows) | `2,097.0` | **`1.0149`** | `0.9939` | `1.0365` | `42.9%` | `-7.81%` | `1.167–1.200` (`20%` Open wins) |
| **`wikidata -2`** (`199,981` rows) | `1,049.0` | **`0.8795`** | `0.8644` | `0.8946` | **`83.0%`** | `-3.42%` | `1.062–1.091` (`35–38%` Open wins) |
| **`wikidata -3`** (`299,981` rows) | `699.0` | **`0.8457`** | `0.8319` | `0.8594` | **`86.2%`** | `-4.60%` | `0.980–1.021` (`53–55%` Open wins) |
| **`wikidata -5`** (`499,981` rows) | `420.0` | **`0.8685`** | `0.8704` | `0.8667` | **`82.9%`** | `-5.27%` | `0.712–0.763` (`90%` Open wins) |
| **`wikidata -9`** (`899,981` rows) | `233.0` | **`0.8276`** | `0.8248` | `0.8304` | **`85.1%`** | `-5.17%` | `0.481–0.514` (`98–100%` Open wins) |
| **`text -1`** (`99,981` rows) | `80.0` | **`1.0321`** | `1.0433` | `1.0208` | `34.6%` | `-3.80%` | — |
| **`text -9`** (`899,981` rows) | `12.0` | **`1.0359`** | `1.0404` | `1.0315` | `30.6%` | `-3.73%` | — |

### Per-Row-Count Bin Summary (Sub-400k Gate `>= 1.05`)

| Corpus | Row-Count Bin | Blocks/Sweep | Pooled `Open/Seg` | `after` | `first` | Gate | Status |
|---|---|---:|---:|---:|---:|---|---|
| `wikidata` | `<150k` | `2,098.0` | `1.0149` | `0.9939` | `1.0364` | `sub400k` | `< 1.05` |
| `wikidata` | `150k..250k` | `1,049.0` | `0.8796` | `0.8645` | `0.8947` | `sub400k` | `< 1.05` (`Open` wins `-12.0%`) |
| `wikidata` | `250k..350k` | `699.0` | `0.8457` | `0.8319` | `0.8594` | `sub400k` | `< 1.05` (`Open` wins `-15.4%`) |
| `wikidata` | `400k..600k` | `419.0` | `0.8684` | `0.8702` | `0.8665` | `ref` | reference |
| `wikidata` | `>=600k` | `233.0` | `0.8276` | `0.8248` | `0.8304` | `ref` | reference |
| `text` | `<150k` | `80.0` | `1.0321` | `1.0433` | `1.0208` | `sub400k` | `< 1.05` |
| `text` | `>=600k` | `12.0` | `1.0359` | `1.0404` | `1.0315` | `ref` | reference |

## 2. Mechanism Proven: Zero Block-Size Dependence on the Physical i9 (2D Grid)

On the 2 MiB-L2 VM (`#1355`), `Open/Seg` within a fixed `r/n` bin was flat from `100k` to `300k` rows (`0.4–1.2 MB <= 2 MiB` L2) and fell sharply between `300k` and `500k` rows as `Segmented`'s 4 B/row vector spilled out of L2. On the physical i9-10900K (`256 KiB` L2/core, `20 MiB` L3), even a `-1` block's 4 B/row vector (`~400 KiB`) spills out of L2 into L3, and the 2D `(row_bin × r/n_bin)` grid (`Open/Seg (blocks/sweep)`) is **flat across every block size from `<150k` to `>=600k`**:

| `r/n` Bin | `<150k` (`-1`) | `150k..250k` (`-2`) | `250k..350k` (`-3`) | `400k..600k` (`-5`) | `>=600k` (`-9`) |
|---|---:|---:|---:|---:|---:|
| **`<0.06`** | `0.455` (`48`) | `0.419` (`10`) | `0.368` (`4`) | `0.356` (`3`) | `0.611` (`2`) |
| **`0.06–0.07`** | `0.721` (`33`) | `0.586` (`56`) | `0.653` (`114`) | `0.735` (`165`) | `0.717` (`132`) |
| **`0.07–0.08`** | `0.783` (`159`) | `0.788` (`332`) | `0.786` (`305`) | `0.831` (`127`) | `0.792` (`46`) |
| **`0.08–0.09`** | `0.911` (`475`) | `0.860` (`341`) | `0.833` (`130`) | `0.892` (`54`) | `0.850` (`15`) |
| **`0.09–0.10`** | `1.018` (`551`) | `0.931` (`109`) | `0.896` (`40`) | `0.913` (`10`) | `0.987` (`4`) |
| **`0.10–0.12`** | `1.102` (`466`) | `1.031` (`74`) | `1.097` (`30`) | `1.171` (`15`) | `1.135` (`13`) |
| **`0.12–0.15`** | `1.238` (`207`) | `1.183` (`72`) | `1.212` (`46`) | `1.286` (`35`) | `1.302` (`16`) |
| **`0.15–0.21`** | `1.201` (`159`) | `1.202` (`55`) | `1.375` (`30`) | `1.540` (`10`) | `1.572` (`5`) |

Key takeaways:
1. **No block-size effect on the i9:** Within every `r/n` band, `Open/Seg` is invariant across block sizes (`100k` through `900k`), crossing `1.0` around `r/n ≈ 0.10` everywhere. The modest `-9 -> -1` headline rise (`0.828 -> 1.015`) is pure `r/n` composition shift (`40%` of `-1` blocks have `r/n >= 0.10` vs `14.6%` at `-9`), and `text -1` (`1.0321`) matches `text -9` (`1.0359`).
2. **Even the best `(N0, t0)` rule yields only `-2.72%` inverse-only (`N0 = 350k..400k, t0 = 0.100`),** far below `open-admission-block-size` Step 1 (`Open/Seg >= 1.05`) and Step 3 (`>= 5%` whole-file `-1` wall), and switching `0.10–0.21` blocks onto `Segmented` would sacrifice `Open`'s multi-worker L3/RSS advantage.
3. **Disposition:** `block-size-route-i9-probe` closes `done` with frozen verdict **`STOP`**; `bench-block-size-route` is dequeued from `bench/workstation-queue.txt`; a scoped dead-end row is recorded in `frontier/dead-ends.tsv`. No `src/open.rs` change is warranted on the physical i9 (note: `open-admission-block-size.toml` itself was not touched here to keep zero overlap with `#1328` on `src/open.rs`).
