# C6 Adoption Registration: promote the bounded-slab four-lane forward walk to the default `OpenCells` walk (`forward-emission-bounded-slab-rescue`)

- **Claim:** `frontier/claims/forward-emission-bounded-slab-rescue.toml` (P1, `build`, `striated`)
- **Kind:** C6 production adoption registration (`docs/team/PERFORMANCE-POLICY.md`; requires one independent review by a different agent before queueing on the workstation). RULES C2 applies strictly: no threshold, metric, or verdict rule is relaxed after hardware data is seen.
- **Recipe:** `just submit bench-forward-adopt-rescue` (`.just/forward-adopt-rescue.just` -> `bench/forward_adopt_rescue.py`)
- **Scope boundary (`synth-23` §1):** This registration decides C6 adoption of the `#![forbid(unsafe_code)]` bounded-slab four-lane forward walk (`src/open/lanes.rs`, `#1312` behind `--features forward-anchor-lanes`). The default feature flip (`Cargo.toml` default features and the `forward-anchor-lanes` `cfg` gates in `src/open.rs`) is a separate follow-on PR only after this run returns `ADOPT`.

---

## 1. Prior evidence (motivating adoption, never reused to decide adoption)

1. **Prior i9 C6 verdict on the unbounded slab-pool variant (`origin/evidence ccfa473f`):**
   - Archived bundle `forward-adopt/run-20261128T020039Z.tar.gz` (SHA-256 `2514bb60d89ca634483486824d0124e69e336023a4f73b1783f5ec04e030617d`, `docs/notebook/2026-11-28-forward-emission-adopt-i9-verdict.md`, queued at `9875c712`).
   - Proved `#1110` + `#1217` wins decisively on wall/CPU with 12/12 guard cells clean (`wikidata/1` `0.8462 [0.8450, 0.8474]`, `wikidata/6` `0.8648 [0.8594, 0.8702]`, `wikidata/8` paired median `0.9553` / CPU `-7.8%` across `16/16` rounds, `wikidata/10` `0.9649 [0.9541, 0.9758]`; all 12 guard cells `0.9899–1.0143×` wall and `0.9965–1.0022×` RSS), while failing `DO NOT ADOPT` (preserved unrelaxed under C2) on two items:
     - **RSS failure:** `#1217`'s unbounded slab pool (`~32 × 32 KiB` slabs retained alongside `out`) drove peak RSS to `+1.72 MiB` (`1.0541×`) at `1W` and `+4.89..+6.94 MiB` (`1.0890–1.1081×`) at `6–12W`;
     - **`wikidata/8` upper-CI tail (`t_ci95_hi = 0.9859 > 0.965`):** three unpinned rounds (`r00`, `r08`, `r12`) had `cpr = 1.326` at `4.88–4.89 GHz` and `-6.7%..-10.4%` lower CPU than `base`, yet scanner-worker SMT sibling sharing pulled wall up by `+1.8%..+7.8%`.
2. **Physical i9 PMU confirmation of the 8W L3-eviction mechanism (`origin/evidence a7ac01d8`, `#1315`):**
   - Archived bundle `scanner-contention/run-20261128T052545Z.tar.gz` (SHA-256 `ce86eaf2b812e3e0beae93684941daf98b72caf9864eba8d37873cd029550d09`, `docs/notebook/2026-11-28-scanner-contention-residual-verdict.md`), built at `c7fa3284` (already including `#1117` and `#1294` `a6c-wire-redesign`).
   - `CONFIRMED`: moving `#1217` from `4W` to `8W` at constant clock (`clk_r = 1.326`) added `+49.28M` scanner cycles (`+6.47%`), of which `76.4%` (`+37.64M` cycles, `2.44×` base's L3-stall excess) were `cycle_activity.stalls_l3_miss:u`, directly matching `#1217`'s `+6.53 MiB` slab pool pushing 8 workers + scanner past the i9-10900K's 20 MiB L3. Even with that L3 penalty on the post-`#1294` base, `lanes/1` ran `1.3121 -> 1.0645 s` (`0.811×`) and pinned-scanner `lanes/8` ran `0.1926 -> 0.1787 s` (`0.928×`, `4.69 GB/s`).
3. **Bounded-slab rescue build (`#1312`, merged `0762c7ea`, Clauses 1 & 2 passed):**
   - Strictly inside `src/open/lanes.rs` (`#![forbid(unsafe_code)]`, unchanged `lanes::walk` signature), `#1312` bounds each worker's `Arena` statically to `<= 80 KiB` (`4 × 4 KiB = 16 KiB` active slabs + `<= 64 KiB` tails, zero retained full-block slab pool) by flushing full `4 KiB` chunks straight into `out`, permuting chunks in place via cycle-following moves reusing `slabs[0]`, and inserting `<= 16` tails in one backward `copy_within` pass.
   - Passed `3/3` Miri (`0` UB), `5/5` `open::lanes::tests::`, `7/7` `AB_SHA256=1` `libbzip2` parity (`1W` lazy + `8W` `mibz2`), `Walk::run<4..1>` `501 / 423 / 284 / 201` insns & `0 vzeroupper`, and paired 9-round `ab` (`wikidata.bz2`):
     - Baseline `1W` lazy: RSS `0.9970×` (`-0.09 MiB` vs `#1217` `+0.98 MiB`), CPU `0.8863×` (`99.1%` of `#1217`'s gain);
     - Baseline `8W` `mibz2`: RSS `1.0076×` (`+0.30 MiB` vs `#1217` `+7.69 MiB`, **`-96.1%` excess removed**), CPU `0.8858×` (`1.7%` faster than `#1217`);
     - JCC-32B `1W` lazy: RSS `1.0060×` (`+0.11 MiB`, `-90.9%` excess removed), CPU `0.9130×` (`97.4%` of `#1217`'s gain);
     - JCC-32B `8W` `mibz2`: RSS `1.0220×` (`+1.29 MiB`, `-82.1%` excess removed), CPU `0.8829×` (`1.3%` faster than `#1217`).
4. **Calibrated peak-RSS noise bands (`docs/notebook/2026-11-28-rss-bound-calibration.md` §5):**
   - Establishes paired geometric mean (never uncalibrated ratio of arm medians) as the i9 RSS estimator, with calibrated bounds `<= 1.020×` (`<= +0.5 MiB`) at `wikidata/1`, `<= 1.030×` at `wikidata/{6, 8, 10, 12}` (`pair_sd = 0.006–0.027`), and `<= 1.050×` on high-variance guards (`random`, `records`, `pattern`, `pair_sd = 0.012–0.052`).

---

## 2. Exact base composition & `synth-23` §3 accounting

- **Base commit:** `origin/master` at queue time (`>= 3214a002e07dc4cd9693b19a970e3276067ce423`; exact queued commit SHA recorded in `provenance.json` and `bench/workstation-queue.txt`).
- **Software commits present in BOTH `base` and `cand` (relative to `ccfa473f` at `9875c712`):**
  - `#1246` (`91ab4c8d`, `open-cap-prescan-cost`, already present in `9875c712`);
  - `#1117` (`15c289b5`, `scanner-a2-fast`: direct selector runs, multi-length code-length LUT, single-write table build);
  - `#1294` (`0e9b59ff`, `a6c-wire-redesign`: safe `Cell` 4× `wire_xor` + overlapping-SIMD register-cursor `wire`);
  - `#1274` (`2f02a288`, `runs-front-i9`: `front_avx2` 64-symbol `vptest` Pass 1, `u32` counts, hoisted XMM steps).
- **Only difference between `base` and `cand`:**
  - `base`: default features (shipped bidirectional `OpenCells` walk).
  - `cand`: `--features forward-anchor-lanes` (the `#1312` `<= 80 KiB/worker` bounded-slab four-lane forward walk in `src/open/lanes.rs`).
- **Base-movement accounting (`docs/notebook/2026-11-28-synthesis-twenty-third-pass.md` §3):**
  - Because `#1294` (`a6c-wire-redesign`) is already merged in `base` (and `cand`), `base`'s bidirectional `wire_xor` gained a larger projected reduction (`~33–44 ms` at 1W) than `cand`'s forward `wire` (`~9–20 ms`). `synth-23` §3 projects this narrows the `ccfa473f` ratios from `0.846 -> ~0.859–0.865` at `wikidata/1`, `0.865 -> ~0.878–0.885` at `wikidata/6`, and `0.964 -> ~0.964–0.974` at `wikidata/8` before accounting for `#1312`'s L3-contention relief. Both arms are built from the same queued `master` SHA (`>= 3214a002`), so any margin change vs `ccfa473f` reflects the faster `#1294`/`#1117` base rather than a regression in `#1312`.
- **Rationale for the frozen `wikidata/8` compound rule (`synth-23` §3):**
  - Compared to dropped `forward-emission-adopt`'s single `t_ci95_hi <= 0.965` bar, this new candidate freezes `t_ci95_hi <= 0.975` **together with** `paired_median <= 0.960` **and** `CPU paired GM <= 0.940` (plus a new `wikidata/6 t_ci95_hi <= 0.900` gate and much tighter `wikidata/{6,8,10,12}` RSS `<= 1.030×` vs `<= 1.080×`). The `+0.010` allowance on the unpinned `wikidata/8` t-CI upper edge explicitly absorbs the occasional unpinned scanner-worker SMT-sibling placement outlier documented in `docs/notebook/2026-11-28-forward-emission-adopt-i9-verdict.md` §3 (`r00/r08/r12`), while `paired_median <= 0.960` and `CPU <= 0.940` guarantee a genuine `>= 4%` typical wall gain and `>= 6%` total CPU cut at 8 workers.

---

## 3. Experimental design

- **Machine & toolchain:** Louis's physical Intel Core i9-10900K (unrestricted 10 physical cores / 20 CPUs, unlimited cgroup v2 quota verified via `ab env --facts`, `rustc 1.99.1`), clean git checkout (zero tracked changes), baseline x86-64 codegen (`RUSTFLAGS`, `CARGO_BUILD_RUSTFLAGS`, `CARGO_ENCODED_RUSTFLAGS`, `CARGO_PROFILE_RELEASE_LTO`, `CARGO_PROFILE_RELEASE_CODEGEN_UNITS`, `CARGO_PROFILE_RELEASE_OPT_LEVEL` all unset). No CLI overrides permitted on registered runs.
- **Isolated builds & distinct-binary VOID guard:** `base` and `cand` are built from the same commit into isolated `CARGO_TARGET_DIR`s (`--release --locked`). `binaries.json` records both SHA-256 digests; identical digests immediately fail `VOID`.
- **Corpora (shallow-safe materialization, independently hashed via C `libbzip2` `bz2.open`):**
  - `wikidata-x4`: `4×` multistream concatenation of `corpora/wikidata.bz2`, verified SHA-256 `0b820d14299af637bf7f644d172f5efe91ad59e169cf2a50bd0962f5c89ce08e` (`838,800,784 B`).
  - All 6 guard corpora (`ab gen <dir> 4`): `text`, `records`, `random`, `runs`, `pattern`, `onebyte`.
- **Thread grid & `ab` arms (`17` cells):**
  - `wikidata-x4`: `k ∈ {1, 6, 8, 10, 12}` (`k = 1` pinned to core 0 via `mibz2-workers-lazy` so `OpenCells` is directly exercised at 1 worker; `k ∈ {6, 8, 10, 12}` unpinned via production `mibz2`).
  - All 6 guard corpora: `k ∈ {1, 8}` (`k = 1` pinned via `mibz2-workers-lazy`; `k = 8` unpinned via production `mibz2`).
- **Paired protocol & slow-mode control (`#1240`):**
  - `ROUNDS = 16` alternating AB/BA paired rounds per cell (`ab run`).
  - `--settle 2`, `--max-busy 1.5`, `--prespin-ms 100`, `--machine-state` (`cycles_per_ref`, clock, temperature recorded beside every timed repeat).
  - `score` reports pooled paired geometric-mean 95% Student-t CIs, bootstrap paired medians, position-stratified CIs (`pos 0` vs `pos 1`), paired CPU ratios, paired RSS geometric means + absolute median KiB deltas, and `cycles_per_ref` ratios.
- **Exactness gate:**
  - Pre-sweep untimed `AB_SHA256=1` `ab child` on all `17 × 2 = 34` `(arm, corpus, k)` runs verified byte- and SHA-256-exact against C `libbzip2`.
  - Every timed repeat (`17 × 16 × 2 = 544` decodes) verified for `err == ""`, exact `out`, exact 64-bit `hash` tied to `libbzip2` parity, and finite positive `wall`, `cpu`, and `rss_kb`.

---

## 4. Frozen C6 Decision Rule (`ADOPT` / `DO NOT ADOPT` / `VOID`)

Matches `frontier/claims/forward-emission-bounded-slab-rescue.toml` Clause 3 verbatim.

**`VOID`** (exit 1, no adoption decision) if any prerequisite, registration parameter, or integrity check fails:
- Non-i9 host, restricted CPU affinity, active/unknown/missing cgroup CPU quota (`env-facts.json`), dirty tracked checkout, wrong `rustc` version, non-empty `RUSTFLAGS` or `CARGO_PROFILE_RELEASE_*`, or any CLI/provenance parameter override (`strict_i9 != true`, `rounds != 16`, `settle_s != "2"`, `prespin_ms != 100`, or altered cell grid);
- Identical `base` and `cand` binary SHA-256;
- Any parity, output length, `hash`, or `SHA-256` mismatch, any decode error, any missing/duplicate/unexpected row, or any invalid/non-finite `wall`, `cpu`, `rss_kb`, or CI.

**`ADOPT`** iff ALL of the following hold simultaneously:
1. **Exactness & safety:** 100% byte-, CRC-, and error-exact parity across all 34 untimed `AB_SHA256=1` runs, all 544 timed repeats, and `cargo test` / Miri (`0` UB, `#![forbid(unsafe_code)]` in `src/open/lanes.rs`);
2. **`wikidata/1` wall:** paired geometric-mean 95% Student-t CI upper bound **`<= 0.900`**;
3. **`wikidata/6` wall:** paired geometric-mean 95% Student-t CI upper bound **`<= 0.900`**;
4. **`wikidata/8` wall & CPU:** paired geometric-mean 95% Student-t CI upper bound **`<= 0.975`** AND paired median wall ratio **`<= 0.960`** AND paired geometric-mean CPU ratio (`cpu_s.t_ratio`) **`<= 0.940`**;
5. **`wikidata/10` wall:** paired geometric-mean 95% Student-t CI upper bound **`<= 0.985`**;
6. **Zero guard regressions:** across all 6 guard corpora (`text`, `records`, `random`, `runs`, `pattern`, `onebyte`) at both `k = 1` and `k = 8` (`12` guard cells), NO cell has `wall_s.t_ci95[0] > 1.020` OR `wall_s.paired_median > 1.020`;
7. **Calibrated peak RSS bounds (paired geometric mean `rss_kib.t_ratio`, per `docs/notebook/2026-11-28-rss-bound-calibration.md` §5):**
   - `wikidata/1`: paired GM **`<= 1.020×`** AND absolute median RSS delta (`cand_median - base_median`) **`<= +512 KiB`** (`<= +0.5 MiB`);
   - `wikidata/{6, 8, 10, 12}`: paired GM **`<= 1.030×`**;
   - All 12 guard cells (`text`, `records`, `random`, `runs`, `pattern`, `onebyte` at `k = 1, 8`): paired GM **`<= 1.050×`**.

**`DO NOT ADOPT`** otherwise: record the complete 17-cell table and a scoped dead-end row in `frontier/dead-ends.tsv` naming the failing clause(s).
