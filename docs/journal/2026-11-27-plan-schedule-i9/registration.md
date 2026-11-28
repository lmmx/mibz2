# Pre-registered physical i9-10900K verification: `plan-schedule-and-lazy-threshold` and `trim-at-measured-kstar`

*Frozen pre-data C2/C6 workstation registration. No physical i9 data has been collected under this recipe.*

- **Claims covered:**
  1. `frontier/claims/plan-schedule-and-lazy-threshold.toml` (`mode = plan-schedule`)
  2. `frontier/claims/trim-at-measured-kstar.toml` (`mode = trim-kstar`)
- **Runner:** `bench/plan_schedule.py`
- **Recipes:** `just bench-plan-schedule` / `just bench-trim-kstar` (`.just/plan-schedule.just`, `just submit`-compatible via `bench/bundle.sh`)
- **Tests:** `bench/test_plan_schedule.py`

---

## 1. Registered workstation & build invariants (VOID on violation)

Any violation of the invariants below invalidates the run (`verdict = VOID`, exit status 2):

1. **Host & topology:** Intel Core i9-10900K (`lscpu` contains `i9-10900K`), 10 physical cores × 2 hardware threads = 20 logical CPUs, full `os.sched_getaffinity(0)` availability, and unlimited effective cgroup CPU quota (`ab env --facts`).
2. **Baseline x86-64 codegen:** `RUSTFLAGS`, `CARGO_BUILD_RUSTFLAGS`, and `CARGO_ENCODED_RUSTFLAGS` MUST all be unset. Toolchain pinned to `rustc 1.99.1` (`RUSTUP_TOOLCHAIN=1.99.1`, `cargo build --release --locked`).
3. **Shallow-safe isolated builds:** Both `base` and `head` checkouts are materialized shallow-safely (`git fetch --depth=1 --no-tags`) when fetched by ref/SHA and compiled in strictly isolated `CARGO_TARGET_DIR` directories (`target-base` vs `target-head`).
4. **Distinct-binary guard:** `sha256(base_ab) != sha256(head_ab)` is verified before any timing run (`assert_distinct_binaries`). Identical binaries abort immediately as `VOID`.
5. **Untimed exactness parity:** Before timed rounds start, every `(corpus, threads)` pair is decoded with `AB_SHA256=1` on both `base` and `head` and checked byte-for-byte and SHA-256 against Python's independent `bz2.open` (`libbzip2`) reference. Every timed row is also verified for zero errors, `termination == "success"`, exact byte count, exact `ab` hash, finite positive `wall`, and positive `rss_kb`.
6. **Hardware clock & mode telemetry:** Timed invocations run with `AB_MACHINE_STATE=1` and `ab run --machine-state --prespin-ms 100 --settle 2.0 --max-busy 1.5` (matching `#1240` slow-mode control), recording `cycles`, `instructions`, `ref_cycles`, `cycles_per_ref`, and `runq_s`.
7. **Complete, balanced coverage:** Zero missing or duplicate `(corpus, pinning, threads, arm, round)` rows, and exact 50/50 `pair_pos ∈ {0, 1}` balance per arm over every even number of rounds.

---

## 2. Inputs

- **`wikidata` (`wikidata x4`):** 4× multistream concatenation of `corpora/wikidata.bz2` (materialized shallow-safely from `origin/bench-data` via `bench/wikidata.json` + `examples/windows.rs` when absent), verified to exact SHA-256 `0b820d14299af637bf7f644d172f5efe91ad59e169cf2a50bd0962f5c89ce08e`.
- **Guards (`text`, `records`, `random`):** Deterministically generated via `ab gen <dir> 8` (`scale = 8`) using the compiled `base` harness.

---

## 3. Experimental design & estimators

- **Rounds:** `12` paired rounds (default). Within each round, cells are shuffled deterministically (`seed = 1210 + round * 9973`) and arm execution order alternates (`(base, head)` on even rounds, `(head, base)` on odd rounds), giving exact 6/6 `pair_pos ∈ {0, 1}` balance per arm in every cell.
- **Pinning:**
  - `pinned`: `ab run --spread` (one first-sibling CPU per physical core for `T <= 10`).
  - `unpinned`: `ab run` without `--spread` (all 20 logical CPUs visible to the scheduler).
- **Estimators reported for every cell (`head / base`, `< 1.0` = faster/smaller):**
  1. **Pooled paired geometric mean & 95% t-CI** over the `R` paired rounds (`mtf_fold.t_crit_95(R - 1)`).
  2. **Position-stratified estimator:** separate Welch log-ratio CIs within `pair_pos = 0` and `pair_pos = 1`, combined by sample-weighted geometric mean to cancel any first-in-pair warm-up/thermal bias.
  3. **Within-mode (`cycles_per_ref`) estimator:** classifies rows into `fast` vs `slow` clock modes using `cycles_per_ref` (within `4%` at 1T / `15%` at multi-T of peak `cycles_per_ref`, falling back to wall-gap split only when PMU counters are absent), reporting `within_fast`, `within_slow`, `same_mode_paired`, and `cycles_per_ref_ratio`.
  4. **Peak RSS ratio:** `median(head rss_kb) / median(base rss_kb)`.

---

## 4. Frozen stop tests (judged verbatim; never relaxed post-data)

### Mode A: `plan-schedule` (`plan-schedule-and-lazy-threshold.toml` item 2)

- **Cells (22 total):**
  - Pinned `wikidata x4`: `t ∈ {4, 5, 6, 7, 8, 9, 10}`
  - Unpinned `wikidata x4`: `t ∈ {4, 8, 10, 12, 16, 20}`
  - Pinned guards (`text`, `records`, `random`): `t ∈ {4, 6, 8}`
- **PASS requires ALL gates below; any failure produces `STOP`:**
  1. `wikidata` pinned `5T`: paired wall ratio **`<= 0.85x`** AND peak RSS ratio **`<= 0.55x`**.
  2. `wikidata` pinned `7T`: paired wall ratio **`<= 0.75x`**, **monotonically faster than `6T` pinned** (`head_median_wall(7T pinned) < head_median_wall(6T pinned)`), AND peak RSS ratio **`<= 0.45x`**.
  3. `wikidata` pinned `9T`: paired wall ratio **`<= 1.02x`** AND peak RSS ratio **`<= 0.92x`**.
  4. `wikidata` pinned `{4, 6, 8, 10}` AND unpinned `{4, 8, 10, 12, 16, 20}`: paired wall ratio **`<= 1.015x`** AND peak RSS ratio **`<= 1.02x`**.
  5. Off-Wikidata guards (`text`, `records`, `random` at pinned `{4, 6, 8}`): zero **`> 1.03x`** disjoint wall regression (`not (disjoint_slower and wall_ratio > 1.03)`) AND peak RSS ratio **`<= 1.05x`**.

### Mode B: `trim-kstar` (`trim-at-measured-kstar.toml`)

- **Cells (4 total):** Pinned `10T` across `wikidata x4`, `text`, `records`, `random` (12 order-balanced position-stratified rounds comparing `head` = 8 workers vs `base` = 7 workers).
- **PASS requires ALL gates below; any failure produces `STOP`:**
  1. `wikidata` pinned `10T`: paired wall 95% CI upper bound **`<= 0.97`** AND peak RSS ratio **`<= 1.05x`**.
  2. Guards (`text`, `records`, `random` at pinned `10T`): paired wall ratio **`<= 1.01x`** AND peak RSS ratio **`<= 1.05x`**.

---

## 5. Bundle layout

Each run writes one self-contained directory `target/<mode>/run-<UTC>/` packed into `target/<mode>/run-<UTC>.tar.gz` via `bench/bundle.sh`:

- `outcome.txt` — one-line verdict (`PASS:`, `STOP:`, or `VOID:`)
- `summary.txt` / `summary.json` — per-cell pooled, position-stratified, within-mode (`cycles_per_ref`), RSS, CPU, and gate checks
- `runs.jsonl` — all timed observations (one JSON object per run)
- `parity.jsonl` — untimed `AB_SHA256=1` verification records
- `binaries.json`, `commits.txt`, `inputs.json`, `env.json`, `lscpu.txt`, `lscpu-e.txt`, `registration.md`, `console.log`, `toolchain.txt`
