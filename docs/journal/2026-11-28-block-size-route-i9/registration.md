# C1a Diagnostic Registration: Physical i9 Block-Size Route Calibration (`bench-block-size-route`)

- **Date:** 28 November 2026
- **Claim:** `frontier/claims/block-size-route-i9-probe.toml` (Step 1 calibration for `frontier/claims/open-admission-block-size.toml`)
- **Classification:** Diagnostic C1a probe (CLAUDE.md line 227; decides no production default, modifies zero `src/` files).
- **Recipe:** `just submit bench-block-size-route` (`.just/block-size-route.just`, `bench/block_size_route.py`)
- **Expected wall time on physical i9-10900K:** ~3–5 minutes.

## 1. Scientific Question

`docs/notebook/2026-11-28-explore-block-size-route.md` (`#1355`) observed on a 4-vCPU Xeon VM with **2 MiB L2 per core** that `OpenCellsOf::cap` (`src/open.rs`) admits blocks on `r/n` alone without inspecting block row count `n`, causing bzip2 `-1` (`99,981` rows) and `-2` (`199,981` rows) blocks to take `Open` and run `6–20%` slower than `Segmented` (`Open/Seg = 1.167–1.200` at `-1`, `1.062–1.091` at `-2`, flat across `100k–300k` and dropping between `300k` and `500k` as `Segmented`'s 4 B/row vector passes 2 MiB L2).

The physical Intel Core i9-10900K has **256 KiB L2 per core** (8× smaller than the VM) and **20 MiB shared L3**. Even a bzip2 `-1` block's 4 B/row vector (`~400 KiB`) exceeds the i9's L2 while fitting inside its L3. Step 1 of `open-admission-block-size.toml` therefore requires calibrating per-block `Open/Seg` directly on the physical i9 before touching `src/open.rs`.

## 2. Experimental Design (Frozen Pre-Data)

- **Binary:** `examples/route_census` built `--release --locked --features moonshot` on the pinned HEAD commit (no `census` feature, so production `OpenCells::invert_runs` is timed with zero `census` overhead), plus `--example ab` (for `env --facts` and `ab gen` `text` generation) and `--example windows` (for `wikidata` materialization from `bench-data` if `corpora/wikidata.bz2` is absent).
- **Build & host invariants:**
  - Rust `1.99.1`, clean git tree (no tracked modifications), baseline x86-64 codegen (ambient `RUSTFLAGS`, `CARGO_BUILD_RUSTFLAGS`, `CARGO_ENCODED_RUSTFLAGS`, and `CARGO_PROFILE_RELEASE_*` must be unset).
  - Physical `Intel(R) Core(TM) i9-10900K` (10 physical cores / 20 logical CPUs, unrestricted CPU affinity, zero cgroup CPU quota verified via `ab env --facts` both at runtime and offline in `score_dir`).
  - Every `route_census` invocation is pinned to physical core 0 (`taskset -c <core0_cpu>`) to eliminate cross-core migration noise.
- **Corpora & levels (7 files):**
  - `wikidata` (`209,700,196 B` decoded, canonical SHA-256 verified against `bench/wikidata.json` and C `libbzip2`) re-encoded with C `libbzip2` (`bz2.BZ2Compressor(compresslevel=lvl)`) at levels **`-1, -2, -3, -5, -9`**.
  - `text` (`ab gen <tmp>/gen 4`, `15,215,652 B` decoded, C `libbzip2` SHA-256 verified) re-encoded at levels **`-1, -9`**.
- **Parity gate:** Every re-encoded `.bz2` file is independently decoded via C `libbzip2` (`bz2.open`) to confirm exact byte-length and SHA-256 identity against the canonical decoded payload, AND `examples/route_census` asserts `out_p == out_s && out_p == out_l` (`OpenCells == Segmented == LazyCells`) on every invocation.
- **Timing protocol:**
  - `ROUTE_FULL=90000` (so bzip2 `-1` blocks at `99,981` rows are counted as full blocks).
  - Both shadow orders: `ROUTE_SHADOW=1` (`Open` timed first, `Segmented` second) and `ROUTE_SHADOW=first` (`Segmented` timed first, `Open` second) to eliminate cache-warming order bias.
  - **`ROUNDS = 3`** interleaved rounds × **`2`** orders = **`6`** order-balanced sweeps per file (`REPS = 2` inside `route_census`, so pass 1 warms scratch and pass 2 emits hot per-block timings on stderr), with **`SETTLE_S = 2`** seconds between sweeps. Order alternates (`after, first` on even rounds; `first, after` on odd rounds).

## 3. Row-Count & `r/n` Binning

All full `Open`-routed blocks (`route == "Open"` and `n >= 90,000`) are aggregated into:
- **Row-count bins:**
  - `<150k`: `90,000 <= n < 150,000` (`-1`)
  - `150k..250k`: `150,000 <= n < 250,000` (`-2`)
  - `250k..350k`: `250,000 <= n < 350,000` (`-3`)
  - `350k..400k`: `350,000 <= n < 400,000`
  - `400k..600k`: `400,000 <= n < 600,000` (`-5`)
  - `>=600k`: `n >= 600,000` (`-9`)
- **`r/n` bins (for 2D `(row_bin, r/n_bin)` threshold fitting):**
  - `<0.06`, `0.06-0.07`, `0.07-0.08`, `0.08-0.09`, `0.09-0.10`, `0.10-0.12`, `0.12-0.15`, `0.15-0.21`, `>=0.21`.

For each bin, `Open/Seg` is computed as `sum(open_ms) / sum(seg_ms)` pooled across all 6 order-balanced sweeps (3 `ROUTE_SHADOW=1` + 3 `ROUTE_SHADOW=first`), alongside order-separate `shadow_after` and `shadow_first` ratios, round-level paired spread, whole-file `probe_ms / segmented_ms`, and the share of blocks where `Open < Seg`.

## 4. Frozen Decision Rule (C1a)

- **`VOID`:** Any validity prerequisite fails (`--smoke`, `strict_i9 != true`, CLI overrides used, dirty git tree, wrong Rust toolchain, non-i9 host / restricted affinity, non-unlimited CPU quota, ambient `RUSTFLAGS` / `CARGO_PROFILE_RELEASE_*`, SHA-256 parity mismatch, incomplete sweeps, or non-finite numbers).
- **`CONFIRMED`:** Valid run AND at least one `< 400k` row-count bin (`<150k`, `150k..250k`, `250k..350k`, or `350k..400k`, with `>= 10` full `Open` blocks per sweep) on `wikidata` or `text` has pooled `Open/Seg >= 1.05`. The scorer also fits candidate `(N0, t0)` thresholds on the 2D grid (`N0` in `{150_000, 250_000, 350_000, 400_000}`, `t0` in `{0.06, 0.07, 0.075, 0.08, 0.09, 0.10, 0.12, 0.15}`) and reports projected rule vs per-block oracle savings for `open-admission-block-size` Step 2.
- **`STOP`:** Valid run AND no `< 400k` row-count bin reaches pooled `Open/Seg >= 1.05` on either `wikidata` or `text`. Concludes the VM-L2 crossover does not reproduce at `>= 5%` on the physical i9-10900K, fulfilling `open-admission-block-size` Step 1's STOP branch without editing `src/open.rs`.

## 5. Archival & Bundle Integrity

`.just/block-size-route.just` writes exclusively under `target/block-size-route/run-<UTC>/`, tees console output to `console.log`, writes `provenance.json`, `toolchain.txt`, `lscpu.txt`, `lscpu-e.txt`, `host.json`, `env-facts.json`, `inputs.json`, `parity.json`, `registration.md`, `sweeps.jsonl`, `summary.json`, `summary.txt`, and `outcome.txt`, and packs one tarball via `bash bench/bundle.sh`.
