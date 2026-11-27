# C6 Adoption Registration: promote the disjoint-slab four-lane forward walk to the default `OpenCells` walk (`forward-emission-adopt`)

- **Claim:** `frontier/claims/forward-emission-adopt.toml` (P1, `build`, `striated`)
- **Kind:** C6 production adoption registration (`docs/team/PERFORMANCE-POLICY.md`; requires one independent review by a different agent before queueing on the workstation). C2 applies: no threshold or verdict rule is relaxed after hardware data is seen.
- **Recipe:** `just submit bench-forward-adopt` (`.just/forward-adopt.just` -> `bench/forward_adopt.py`)
- **Prior evidence (motivating adoption, never reused to decide adoption):**
  - `origin/evidence` `e98948b6` (`forward-lanes/run-20261127T223754Z.tar.gz`, SHA-256 `db53fe2ca106b69a4b85e63bfb223e6072b5e02017e70720306e8f482de00fff`), physical i9-10900K kill test of `#1110` + `#1217` (`#![forbid(unsafe_code)]` disjoint-slab four-lane walk at `v0.5.17` `01acd74d`):
    - `wikidata/1`: `1.353 -> 1.113 s`, `0.8282x [0.8160, 0.8337]` (`-17.18%`, `-230.6 ms`, `1.012x` RSS);
    - `wikidata/8` (real scan): `0.1959 -> 0.1844 s` (`4.55 GB/s`), `0.9434x [0.9377, 0.9483]` (`-5.66%`, `wkr_busy -10.64%`, `1.061x` RSS);
    - `wikidata/8/S` (ideal scan): `0.1906 -> 0.1683 s` (`4.99 GB/s`), `0.8793x [0.8570, 0.8954]` (`-12.07%`);
    - `text/1`: `0.9964x` (`-0.36%`, `9/15` faster, `1.000x` RSS; `RESCUE` solely on 15-round bootstrap upper CI `1.0166 > 1.010`).

---

## 1. Mechanism and scope

Promote the existing `#![forbid(unsafe_code)]` sampled-anchor four-lane forward walk (`src/open/lanes.rs`, `#1110` + `#1217`, currently gated behind `--features forward-anchor-lanes`) to the default `OpenCells` walk (`src/open.rs`, `Cargo.toml`).

- Each lane owns a recycled `32 KiB` slab (`Box<[u8; 32768]>`), `LONG` words that fit in the current slab expand inline (`99.7%` of `LONG` words on Wikidata), and `Walk::run<K>` compiles out of line per lane count with `0` `unsafe` and `0` `vzeroupper`.
- All downstream follow-ons (`L6a` zero-stitch vectored RLE1+CRC32 in `forward-rle1-crc-vectored-build`, and low-thread `t = 1..3` `OpenCells` unification retiring `Cells`) are separate claims and NOT bundled here.

---

## 2. Experimental design

- **Machine & toolchain:** Louis's physical Intel Core i9-10900K (unrestricted 10 physical cores / 20 CPUs, unlimited cgroup v2 quota, `rustc 1.99.1`), clean git checkout, baseline x86-64 codegen (`RUSTFLAGS`, `CARGO_BUILD_RUSTFLAGS`, `CARGO_ENCODED_RUSTFLAGS` all unset).
- **Arms (same commit, isolated `CARGO_TARGET_DIR` per arm):**
  - **`base`:** default features (shipped bidirectional `OpenCells` walk).
  - **`cand`:** `--features forward-anchor-lanes` (the `#1110` + `#1217` disjoint-slab four-lane forward walk).
  - **Distinct-binary VOID guard:** `binaries.json` records both SHA-256 digests; identical digests immediately fail `VOID`.
- **Corpora (shallow-safe materialization, independently hashed via C `libbzip2` `bz2.open`):**
  - `wikidata-x4`: `4x` multistream concatenation of `corpora/wikidata.bz2` (materialized from local cache or `--depth=1 --no-tags` fetch of `origin bench-data` + `examples/windows.rs`), verified SHA-256 `0b820d14299af637bf7f644d172f5efe91ad59e169cf2a50bd0962f5c89ce08e` (`838,800,784 B`).
  - **All 6 guard corpora** (`ab gen <dir> 4`): `text`, `records`, `random`, `runs`, `pattern`, `onebyte`.
- **Thread grid & `ab` arms:**
  - `wikidata-x4`: **`k ∈ {1, 6, 8, 10, 12}`** (`k = 1` pinned to core 0 via `mibz2-workers-lazy` so `OpenCells` is directly exercised at 1 worker; `k ∈ {6, 8, 10, 12}` unpinned via production `mibz2`).
  - All 6 guard corpora: **`k ∈ {1, 8}`** (`k = 1` pinned via `mibz2-workers-lazy`; `k = 8` unpinned via production `mibz2`).
- **Paired protocol & slow-mode control (`#1240`):**
  - **`ROUNDS = 16`** interleaved alternating AB/BA paired rounds per cell (`ab run`).
  - **`--settle 2`** (`ab`'s default `2.0 s` settle gap + `--max-busy 1.5`) and **`--prespin-ms 100`**, eliminating the `--settle 0.2` slow-mode (b) artefact documented in `#1240` (`0/160` slow at `2 s` settle across unpinned 6–12T runs).
  - **`--machine-state`** (`AB_MACHINE_STATE=1`): records `cycles_per_ref`, clock, and package temperature beside every timed repeat; `score` reports pooled paired geometric-mean 95% Student-t CIs, bootstrap paired medians, **position-stratified CIs** (`pos 0` vs `pos 1`), and `cycles_per_ref` ratios.
- **Exactness gate:**
  - Pre-sweep untimed `AB_SHA256=1` `ab child` on every one of the `17` `(corpus, k)` cells for both arms (`34` parity runs) verified byte- and SHA-256-exact against C `libbzip2`.
  - Every timed repeat (`17 * 16 * 2 = 544` decodes) verified for `err == ""`, exact `out`, and exact 64-bit `hash` matching `libbzip2`.

---

## 3. Frozen C6 Decision Rule (`ADOPT` / `DO NOT ADOPT` / `VOID`)

**`VOID`** (exit 1, no adoption decision) if any prerequisite or integrity check fails:
- Non-i9 host, restricted CPU affinity or quota, dirty tracked checkout, wrong `rustc` version, non-empty `RUSTFLAGS`;
- Identical `base` and `cand` binary SHA-256;
- Any parity, output length, `hash`, or `SHA-256` mismatch, any decode error, any missing or duplicate row, or any undefined/non-finite CI.

**`ADOPT`** iff ALL of the following hold simultaneously:
1. **Exactness & safety:** 100% byte- and error-exact parity across every timed repeat and `cargo test` / Miri (`0` UB, `#![forbid(unsafe_code)]` in `src/open/lanes.rs`);
2. **`wikidata/1`:** paired geometric-mean 95% CI upper bound **`<= 0.900`** (`>= 10%` 1-worker speedup);
3. **`wikidata/8`:** paired geometric-mean 95% CI upper bound **`<= 0.965`** (`>= 3.5%` real 8-worker speedup);
4. **`wikidata/10`:** paired geometric-mean 95% CI upper bound **`<= 0.985`** (`>= 1.5%` real 10-worker speedup);
5. **Zero guard regressions:** across all 6 guard corpora (`text`, `records`, `random`, `runs`, `pattern`, `onebyte`) at both `k = 1` and `k = 8` (`12` guard cells), NO cell has `t_ci95_lo > 1.020` OR `paired_median > 1.020`;
6. **Memory bounds:** paired median peak RSS ratio **`<= 1.020x`** on `wikidata/1`, **`<= 1.080x`** on `wikidata/{6, 8, 10, 12}`, and **`<= 1.050x`** across all 12 guard cells.

**`DO NOT ADOPT`** otherwise: record the complete 17-cell table and a scoped disposition in `frontier/dead-ends.tsv` naming the failing clause(s).
