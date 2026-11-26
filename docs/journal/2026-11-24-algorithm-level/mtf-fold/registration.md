# The MTF-into-construction fold: prototype design and kill test (registration)

*Frozen 26 November 2026 before any `src/` prototype code, as part of [`mtf-fold-census`](../../../../frontier/claims/mtf-fold-census.toml) admitting [`mtf-fold-build`](../../../../frontier/claims/mtf-fold-build.toml). Diagnostic under RULES C1a: this kill test stops or continues the thread and adopts no production default. C2 is never relaxed after data is seen.*

---

## 1. What the prototype is for

In the shipped decoder (`src/runs.rs` + `src/open.rs`), every run-route block runs two separate passes back to back:
1. **A5 (`runs::front`, `front_avx2`):** builds `lit`/`one` bitmaps, runs the 2-level AVX2 MTF list + BMI2 `bzhi` RLE2 decode, splits runs longer than `CAP = 127` into multiple `u16` records, writes `runs: Vec<u16>` (`119–362 KiB/block`, `124.5 MiB` across `wikidata ×4`), and computes `counts[256]` + `per[256]` (which `OpenCells` never reads).
2. **A6a (`OpenCells` + `Kernel::construct_capped` first pass):** reads `runs: &[u16]` back, runs `maximal()` to re-merge cap-127 splits into maximal runs, classifies thin (`≤ 16`) vs thick (`> 16`), pushes thin ranks into `OutQ16` inside `out`, and packs thick strips into `edges`.

On the i9-10900K (`wikidata ×4`, `character-stage1/report.txt`):
- At **1 worker:** `A5 = 132.4 ms`, `A6a = 200.3 ms`, combined **`332.7 ms`** (`26.0%` of `1,280.1 ms` worker busy; `M-ideal + A6a` anchor `~298.3 ms`).
- At **8 workers:** `A5 = 19.3 ms/w`, `A6a = 28.2 ms/w`, combined **`47.5 ms/w`** (`25.7%` of `184.7 ms/w` worker busy), masked behind the serial scanner (`185.0 ms` busy; `S` alone `179.1 ms` in `ideal-parts/results/floor/README.md`).

The smooth-zone census ([`README.md`](README.md)) established that:
- `1.14%` of maximal runs (`width > 127`) hold `76.19%` of rows and add `+7.79%` extra `u16` records (`4.72M` extra records on `wikidata ×4`);
- `syms + bitmaps + runs` totals `331–801 KiB/block`, spilling the 256 KiB L2 on nearly every block;
- Direct Pass-2 folding (`p2_folded / p2_unfolded`, 100% bit-for-bit A6a state-identical) saves **`18.4%`** (`27.6%` of `prod A5+A6a`) because `OutQ16::push`'s scattered 32B stores overlap MTF's serial `f0`/`f1` dependency chain via OoO execution (cutting `OutQ16` marginal cost by 46%) alongside removing the `124.5 MiB` `runs` L2/L3 round-trip and `maximal()` rescan.

**The question for `mtf-fold-build`:** When `front_avx2`'s AVX2 MTF Pass 2 feeds maximal runs directly into `Open`'s A6a (`Pass::take` + `OutQ16` + `edges`) on the i9-10900K, how much wall time does it actually capture at **1 worker** (unmasked) and at **8 workers behind ideal `S`**?

---

## 2. Prototype design (`mtf-fold` research feature)

- **Feature gate:** `#[cfg(feature = "mtf-fold")]` in `src/runs.rs`, `src/open.rs` (and minimal glue in `src/lib.rs` / `src/parallel.rs` if `out` is acquired before Pass 2, or via `OpenCells` entry point). With `mtf-fold` disabled, the decoder compiles identically to shipped master.
- **Mechanism (named by mechanism, never by model):**
  1. On a run-route block (`runs::takes_runs(b)` and `b.max_len <= 900_000`) targeting `OpenCells`, pre-acquire the pooled `out` buffer sized `b.max_len + INLINE` (steady-state `len >= 900_016`, zero memset).
  2. Run `front_avx2` Pass 1 (`lit` and `one` bitmaps up to `eob`, 32 symbols/step AVX2).
  3. Run `front_avx2` Pass 2 (AVX2 2-level MTF + BMI2 `bzhi` RLE2), emitting each maximal run `(c, pos, total)` directly into `Open`'s `Pass::take::<true>` (`OutQ16::push` + `edges` thick strips + `First` counts). No `runs: Vec<u16>`, no cap-127 splitting loop, no `counts[256]`/`per[256]` writes, no `maximal()` rescan.
  4. **Exact admission & safe fallback:**
     - Maintain exact cap-127 record count cheaply (`recs += 1` normally; `recs += total.div_ceil(127)` on the `1.14%` runs with `total > 127`) so `OpenCells::cap(recs, n, p)` matches production **100% identically** on every input.
     - Run Pass 2 with an upper bound protecting `OutQ16` / `edges` (`Cap::NONE` or `DENSE_M` / `DENSE_STRIPS`; note all 233 Wikidata Open blocks have `m <= 231,600 <= 256,000` and `ns <= 5,200 <= 16,384`).
     - If Pass 1/Pass 2 hits any `front_avx2` fallback condition (`eob` missing, `q > 21`, `span > 22`, `pos + total > max_len`, `orig_ptr >= n`), or `OpenCells::cap(recs, n, orig_ptr)` returns `None` or a tighter `Cap::dense` that `f.m`/`ns` exceeded, leave `b.syms` intact and fall back cleanly to shipped `runs::front` + `OpenCells::invert_runs` (`LazyCells`). Increment a `fold_fallbacks` counter so traces can verify `0` Open-block fallbacks on `wikidata ×4`.
  5. Keep `OutQ16::push`'s unconditional 32-byte iota store unchanged (the census proved a `w == 1` 2-byte branch regresses A6a by 10–11%).
- **Safety & exactness obligations (VOID if any fail):**
  - **100% byte-identical** decoded output on `wikidata`, `wikidata ×4`, all `ab gen` corpora (`text`, `random`, `onebyte`, `pattern`, `runs`, `records`), levels 1..9, and all `cargo test` suites with `--features mtf-fold`.
  - **100% exact errors** (`Error { kind, block, bit }`) on all error/fuzz suites (`RunTooLong`, `BlockTooLong`, `BadOrigPtr`, truncated, bad CRC).
  - **Zero new `unsafe`** outside the existing `front_avx2` SIMD/BMI2 kernel invariants (every unsafe block justified with `// SAFETY:`); scalar folded fallback passes **Miri** and `cargo test --features mtf-fold` is green.

---

## 3. The kill test (frozen before any prototype code)

### 3.1 Why paired against `S` ideal at `k = 8` (and L3-aware per `2026-11-26-synthesis/README.md` §3)

At `k = 8`, the serial scanner (`S`) masks worker-side gains (`C` alone is `0.944`, `M` alone is `0.976`), whereas pairing against **`S` ideal** (zero-copy scan oracle) exposes `100%` of worker-side `A5 + A6a` (`47.5 ms/w` = `25.7%` of worker busy) while keeping `C` real and warm — avoiding the cold-store `A7` asymmetry that affects `C`-paired tests (`2026-11-26-synthesis/README.md` §3).

Per `2026-11-26-synthesis/README.md` §§1 and 3, `S`-paired tests at `k = 9..10` hit the coupled `R + D` 20 MiB L3 eviction ceiling (`ready_bytes` hwm `12.0–16.3 MiB`, `copy_out` `43.7–77.9 ms`, `A9` `133–216 ms`), which attenuates worker-side capture. At **`k = 8`**, `SC` stays moderate (`ready_bytes = 8.6 MiB`, `copy_out = 39.5 ms`). Therefore:
- **Judged worker counts:** **`k = 1`** (unmasked worker-side check) and **`k = 8`** (paired against `S` ideal).
- **Reported only:** `k = 6`, `9`, and `10`.
- **Mandatory L3 diagnostics:** Every trace MUST log **`A9` (`R`), `copy_out` (`D`), and `ready_bytes` hwm** alongside `A5`, `A6a`, and wall.

### 3.2 Run design

- **Machine & input:** i9-10900K, `wikidata ×4` (`0b820d14...ce08e`), scanner pinned (as in `bench-zero-copy-floor`), 100 ms pre-spin, **12 randomised blocks** (traces in the first 3).
- **Binary:** One binary combining the zero-copy oracles (`ideal-scan`, `ideal-mtf`, `ideal-construction`) with `mtf-fold` (switchable at runtime per arm).
- **Arms:**

| arm | scan (`S`) | A5 + A6a (`M` + construction first pass) | rest of `C` (`A6b + A6c`) |
|---|---|---|---|
| `none` | real | shipped (`runs::front` + `maximal` A6a) | real |
| `F` | real | **folded (`mtf-fold`)** | real |
| `S` | ideal (zero-copy) | shipped | real |
| `FS` | ideal (zero-copy) | **folded (`mtf-fold`)** | real |
| `SM` | ideal (zero-copy) | ideal `M` (zero-copy) + shipped A6a | real (reference) |
| `SC` | ideal (zero-copy) | shipped `M` + ideal `C` (zero-copy) | ideal (reference) |

*(At `k = 1`, `none`, `F`, `S`, `FS` suffice; `SM` and `SC` are run at `k = 8` as reference ceilings.)*

### 3.3 Decision rule (KILL / CONTINUE — frozen; C2 applies)

**KILL `mtf-fold` if ANY of K1–K4 holds:**
- **K1 (1-worker room too small):** At **`k = 1`**, the 95% upper bound of **`F / none`** exceeds **`0.975`** (fails to prove ≥ **2.5%** 1-worker wall speedup — i.e., `~32 ms`, ~10% of `A5+A6a`).
- **K2 (8-worker paired-oracle room behind `S` ideal too small):** At **`k = 8`**, the 95% upper bound of **`FS / S`** exceeds **`0.975`** (fails to prove ≥ **2.5%** wall speedup behind ideal `S` — i.e., `~4.6 ms` of `~185 ms` `S`-ideal wall, ~10% of `A5+A6a`'s `47.5 ms/w`).
- **K3 (standalone regression where it ships):** At **`k = 6`** or **`k = 8`**, the 95% lower bound of **`F / none`** exceeds **`1.01`** (regresses shipped wall by >1% with the real scanner).
- **K4 (memory):** Median peak RSS of `F / none` exceeds **`1.05`** at any count `k ∈ {1, 6, 8, 9, 10}` (eliminating `runs: Vec<u16>` should keep RSS ≤ `1.00`).

**CONTINUE otherwise:** Proceed to a separate adoption registration under `docs/team/PERFORMANCE-POLICY.md` (covering `wikidata ×4`, `text`, `runs`, `records`, `random` across `k = 1..20`, and composition with `forward-emission` / `pooled-scan-hybrid`).

### 3.4 Reported figures (deciding nothing)

- Per-worker busy time and total CPU for `A5`, `A6a`, and fused `A5+A6a` at every `k`;
- **`A9` (`R`), `copy_out` (`D`), and `ready_bytes` hwm** at `k = 1, 6, 8, 9, 10` (quantifying any 20 MiB L3 giveback);
- Capture ratios at `k = 8`: `log(S / FS) / log(S / SM)` (relative to ideal `M` behind `S`) and `log(S / FS) / log(S / SC)` (relative to ideal `C` behind `S`);
- Standalone `F / none` at `k = 1, 6, 8, 9, 10`;
- Folded-path fallback count on `wikidata ×4` (and a separate sanity timing on `ab gen` `text` at `k = 1, 8` to verify fallback cost is negligible on non-Open dense blocks).

### 3.5 VOID conditions

The run is **VOID** if:
- Any exactness, error, or Miri obligation in §2 fails;
- Any Open-admitted block in `wikidata ×4` falls back (`fold_fallbacks > 0` on Open blocks, or `F`/`FS` does not actually execute the folded path);
- The machine is not the i9-10900K, the scanner pin fails, or any check pass is not byte-identical;
- Any judged 95% confidence bound (`K1`, `K2`, `K3`) is undefined;
- The `S` oracle's removed scan busy residual exceeds 5%.
