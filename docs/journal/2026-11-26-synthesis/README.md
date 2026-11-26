# Synthesis: what the 26 November deltas mean together

*Recurring claim [`synthesise`](../../../frontier/claims/synthesise.toml) — first cross-delta synthesis since the frontier opened (`#945`–`#953`), anchored against the preceding characterisation and kill-test evidence (`#931`, `#933`, `#942`). Pure analytical synthesis: no new benchmarks, no `src/` edits, no C2 relaxations.*

## Evidence and deltas read

- **Frontier deltas (`frontier/deltas/`):**
  - `2026-11-26-coordinator-handoff.md` (`#945`, commit `23a23cb`)
  - `2026-11-26-frontier-hardening.md` (`#946`, commit `3e535c8`)
  - `2026-11-26-the-frontier-outline.md` (`#947`, commit `4ae37ee`)
  - `2026-11-26-stress-test-frontier.md` (`#948`, commit `c893085`) + three open flags in `frontier/flags/`
  - `2026-11-26-floor-and-pooled-scan-outline.md` (`#949`, commit `d88951c`)
  - `2026-11-26-the-frontier-draft.md` (`#950`, commit `daf0224`)
  - `2026-11-26-spelling-census.md` (`#951`, commit `3539780`)
  - `2026-11-26-explore.md` (`#952`, commit `2aaded3`)
  - `2026-11-26-floor-and-pooled-scan-draft.md` (`#953`, commit `9564f80`)
- **Primary evidence & reports:**
  - `docs/journal/2026-11-24-algorithm-level/ideal-parts/results/floor/README.md` (`#931`, `311923e`; `origin/evidence` `cc8ba1374f32`, SHA-256 `10df0d895755494ec6f06369071d649674d855ab953201a2b2888845ab265e34`)
  - `docs/journal/2026-11-24-algorithm-level/rauzy-census/results/README.md` (`#933`, `1aca801`)
  - `docs/journal/2026-11-24-algorithm-level/pooled-scan/results/README.md` (`#942`, `d54c450`; `origin/evidence` `c71c4a56c145`, SHA-256 `97e397ad586a6700c04a874e50dbc5e26ee9fc91912b115111cbe0550ddc8867`)
  - `docs/journal/2026-11-24-algorithm-level/spelling/README.md` (`#951`, `3539780`)
  - `docs/journal/2026-11-26-explore/cold-store-and-rle1-llc.md` (`#952`, `2aaded3`; also cites `origin/evidence` `7dd7b07083b2`, SHA-256 `f0acf19929c8e2f59284e8271b0e109d51c47cd284e1fc511959334f60970fe5`)
- **Shared state & source anchors:** `frontier/surface.tsv`, `frontier/dead-ends.tsv`, all `frontier/claims/*.toml` and `frontier/blog/*.toml`, `src/rle.rs:103-170`, `src/parallel.rs:47,74,880-915`, `src/open.rs:991-1058`.

---

## Five cross-delta relations (what no single delta states alone)

### 1. One physical boundary — the 20 MiB L3 — couples `R`, `D`, and worker-side speedups into a giveback trap at 8–10 workers

`#952` reported four separate cache-capacity findings (`D` degrading 29.1→77.9 ms, `R`/`A9` exploding 2.55–2.9×, `E_consume` cold-store inflation, and `K1` cold-store asymmetry). Reading `#952`'s Tables 1 and 4 alongside `#931` (`floor/README.md`) and `#942` (`pooled-scan/results/README.md`) reveals a stronger structural coupling:

1. **`R` and `D` are active competitors for the same 20 MiB L3, and they blow up *together* when `C` gets faster.**
   Look at `c71c4a56` at `k = 10` across `none → PC → SC` (`cold-store-and-rle1-llc.md` §§1, 4):

   | arm (`c71c4a56`, `k = 10`) | `ready_bytes` hwm | `D` (`copy_out`) | `R` (`A9` total CPU) |
   |---|---:|---:|---:|
   | `none` (shipped) | 14.6 MiB | 43.7 ms | 133.0 ms |
   | `PC` (ideal `C` + pooled scan) | — | 67.4 ms | 191.2 ms |
   | `SC` (ideal `S` + ideal `C`) | **16.3 MiB** | **77.9 ms** | **215.6 ms** |

   Why do `copy_out` and `A9` both jump sharply whenever workers outpace the reader? Because `READY_BYTES = 12 MiB` (`src/parallel.rs:74`) and `SLOTS_PER_THREAD = 4` (`src/parallel.rs:47`) let unread decoded blocks occupy **60–82% of the 20 MiB L3** (`12.0–16.3 MiB`), leaving only `4–8 MiB` for 8–10 active workers whose `unrle_in_place` (`src/rle.rs:103–170`) executes **four full-buffer passes (two full-buffer writes)** over ~900 KiB blocks whenever `shift > 0`.
2. **Consequence for every worker-side claim (`rauzy-output`, `mtf-fold`, `forward-emission`):**
   Making `C`, `E`, or `M` faster pushes the shipped decoder from the `none` regime toward the `PC`/`SC` regime at `k = 8..10`. Unless `R`'s 2-write-pass traffic (`single-pass-rle1-crc`) and `D`'s `READY_BYTES` L3 occupancy (`delivery-l3-working-set`) are reduced together, **part of every future `C`/`E`/`M` gain will be given back to coupled `R + D` L3 eviction** once workers push `ready_bytes` past ~10 MiB.
3. **Refinement of `dead-ends.tsv` row 8 (without overturning it):**
   `#931` (`floor/README.md` §4) concluded "memory as a third station at 8–10 workers: not supported" because `SECMRD` wall stayed flat (`42.4 → 42.9 → 40.5 ms`) and DRAM bandwidth stayed at `13–17 GB/s`. That checked **`SECMRD`**, where *both* `R` and `D` were ideal! In the very same `cc8ba137` run, restoring *only* `R` (`SECMD`) drove DRAM traffic up by `+766.7 MiB`, LLC store misses up by `+3.19M`, bandwidth to `25.91 GB/s`, and wall **backwards** (`55.38 → 55.82 → 56.62 ms`). DRAM bandwidth is indeed not saturated; **20 MiB L3 capacity** is the real third resource that binds `R + D` at `k = 8..10`.

---

### 2. Three worker-side targets (`C-A6c`, `E-walk`, `E-spelling`) converge on `forward-emission` — and `#952` recalibrates `#951`'s dollar figures and `rauzy-output`'s gate

Combining `#933` (`rauzy-census`), `#951` (`spelling-census`), and `#952` (`explore`) reshapes the worker-side payoff map:

1. **`forward-emission` now collapses three surface rows at once, not two.**
   `forward-emission.toml` originally scoped its value around `C-A6c` (`161 ms` at 1W) and `E-walk` (`16.5%` oracle), treating `E-spelling` as a separate representation problem. `#951` proved that **41.6–42.6%** of `E_consume` (`24.2–24.3%` runtime `u128 << 8*(16 - lb)` shift on 232/233 Wikidata blocks + `17.4–18.3%` dual-cursor control) is pure bidirectional spelling overhead — and adding a 4× unrolled inline fast path reaches **−44.6 to −45.8%** (`0.542–0.554×`). Dropping XOR bidirectional links for forward lanes therefore captures a large chunk of `E-spelling` *for free*, alongside simplifying `A6c` wiring and exposing walk MLP.
2. **Cross-calibrating `#951` (warm-cache ratios) against `#952` (warm-cache i9 baseline):**
   `#951` measured its structural shares warm-cache on a single-threaded VM with reusable `out`, then multiplied those ratios by `SECMRD`'s `298 ms` headline. `#952` proved `SECMRD`'s `298 ms` includes `~55–65 ms` of synthetic cold-store DRAM reads (`504.3 MiB` DRAM vs `151.1 MiB` for all of `none`; warm-`C` arm `E` already drops `consume_ns` from `300.9 → 259.4 ms`), placing true in-situ i9 `E_consume` near **`~230–245 ms`** (`~17–18%` of 1W wall). Applying `#951`'s warm-cache ratios to `#952`'s warm-cache baseline scales every 1W i9 estimate down by ~18–23%:

   | lever (`#951` ratio vs `mirror_e_consume`) | `#951` naive (`× 298 ms`) | calibrated warm i9 (`× 230–245 ms`) |
   |---|---:|---:|
   | Drop bidirectional spelling (`0.569–0.574×`, −42%) | ~125 ms | **~97–103 ms** |
   | Forward + 4× inline unroll (`0.542–0.554×`, −45%) | ~134 ms | **~103–112 ms** |
   | `bidi_pre_shifted` while keeping bidi (`0.732–0.737×`, −26.5%) | ~79 ms | **~61–66 ms** |
   | `len != LONG` misprediction ceiling (−7.2..7.9%) | ~23 ms | **~17–19 ms** |
   | `fused_16b_upper_bound` (−78.4%, −81.3% codes) | ~234 ms | **~180–192 ms** |

3. **Multiplicative coupling through one master variable (`m`) — and why `rauzy-output`'s gate is tighter:**
   - `#933` showed that induction-order changes cannot alter retained rows `m = 26.57M` or band count `827,858` (`STOP` at 1.41% ceiling).
   - `#951` showed that `81.58%` of those `26.57M` codes are 1-byte literals arriving in mean `8.81`-literal streaks (`93.57%` inline for `18.71%` of bytes; `6.43%` long words for `81.29%` of bytes at near-`memcpy` speed in L2).
   - Shrinking `m` (`rauzy-output`: lowering `LIMIT` or fusing thin literals) cuts `A6c` wiring, `E-walk` pointer chases, *and* `E-spelling` code count simultaneously.
   - Because `rauzy-output` cuts code count `m` while `forward-emission` cuts cost per code by `42–45%`, their spelling savings compose **multiplicatively, not additively**: cutting `m` by fraction `f` shrinks the remaining spelling budget to `(1 − f) × (230–245 ms)` before `forward-emission` takes its 42% cut. Meanwhile, revising warm `E_consume` from `298 ms` to `~230–245 ms` means `rauzy-output`'s frozen 3% (`39 ms`) gate represents `~16–17%` of warm spelling alone, reinforcing why `rauzy-output` must price `A6a + A6b + A6c + A7 + spelling` jointly.
   - Separately, `#933` (`A6b-dict = 29.6%` of A6b ≈ `64 ms` at 1W) + `#951` (`32.32%` of `spelling` = `3.27 MB` dead `memo_dedup_slack`, flat `spelling` already L2-resident and near-`memcpy`) jointly close SLP-during-emission while leaving `memo_dedup_slack` as a clean A6b-dict trim.
   - Finally, `#951`'s `bidi_pre_shifted` (`0.732–0.737×`, ~61–66 ms warm i9) was measured only on decoupled `E_consume`, not with `edges[]` pointer-chase latency overlapping the shift; checking it inside a full `emit_bidirectional` walk is a cheap, independent smooth-zone probe (`bidi-preshifted-inline`, `idea`) if `forward-emission` takes multiple rounds.

---

### 3. Paired-oracle `K1` methodology splits cleanly into two asymmetric traps (`C`-paired vs `S`-paired) — C2 intact

Reading `#942` (`pooled-scan` KILL on K1) and `#952` (§3 cold-store `C` bias) alongside the `S`-paired kill tests already written into `mtf-fold-build.toml` and `forward-emission.toml` reveals two distinct oracle distortions that future registrations must handle before data:

1. **`C`-paired tests (`pooled-scan-hybrid`) — cold-store baseline asymmetry:**
   In `c71c4a56`, zero-copy `C` inflated worker `A7` by `+20..+33 ms/worker` on `PC` and `SC` (worker-bound, `job_wait` 0.2–3.6 ms/w) while adding `0 ms` to `C` (scanner-bound, `job_wait` 50.6–91.8 ms/w), compressing `k = 8, 9` capture from `~0.73–0.79` to `0.52–0.55`. **Under C2, `pooled-scan`'s KILL stands unconditionally** (`P/none` is still `+5.2%` at `k = 8` and `+10.4%` at `k = 6`). What changes is how `pooled-scan-hybrid` must be registered: never pair a worker-bound scanner candidate against a scanner-bound cold-store `C` baseline without eliminating or pre-registering the `ΔA7` cold-store bias — better still, test after real worker-side speedups (`rauzy-output`, `mtf-fold`, `forward-emission`) create real slack without a `C` oracle.
2. **`S`-paired tests (`mtf-fold-build`, `forward-emission`) — L3-eviction ceiling at `k = 9..10`:**
   Because `mtf-fold-build` and `forward-emission` pair against **`S` ideal** (where `C` is real/warm and both `S` and `XS` are worker-bound), they do **not** suffer `C`'s cold-store `A7` asymmetry. However, Relation 1 shows they *do* hit the coupled `R + D` L3-eviction ceiling as soon as `k` reaches `9..10` (`SC` at `k = 10`: `copy_out = 77.9 ms`, `A9 = 215.6 ms`, `ready_bytes = 16.3 MiB`), which will attenuate measured capture of worker-side speedups! At `k = 8`, `SC` stays moderate (`ready_bytes = 8.6 MiB`, `copy_out = 39.5 ms`). Therefore `S`-paired kill tests should judge at `k = 8` and log `A9`, `copy_out`, and `ready_bytes` hwm alongside wall.

This shared methodology gap motivates the smooth `idea` claim `warm-paired-oracle-k1`.

---

### 4. Re-mapping `R` and `D` on `surface.tsv` and reframing `delivery-copyout`

Two rows of `frontier/surface.tsv` were contradicted by `#952`:
- **Row `R` (resurrected from `"small / -0.9%"`):** `7dd7b070`'s copy-based `R` oracle masked `~55 ms` of real `R` behind an 839 MiB store copy. Primary zero-copy (`cc8ba137` `SECMD − SECMRD`) and traced (`c71c4a56` `A9`) measurements show `R` is **66.6–74.4 ms at 1W (`5.0–5.7%`, comparable to `M`'s `7.5%`)**, **104–115 ms CPU at 8W (`7.1%`)**, and **2.55–2.9× larger at 10W (`133–216 ms`, `+766.7 MiB` DRAM, `+3.19M` LLC store misses)**. Updated `surface.tsv` row `R` and linked `single-pass-rle1-crc`.
- **Row `D` (reframed from `"48 ms flat"`):** `SFCMR`'s `48 ms` was the single-core DRAM read of `R`'s 209.7 MiB cold oracle store (`853.5 MiB` DRAM reads vs `151.1 MiB` for all of `none`). Real `copy_out` is **29.1 ms at 1W**, **27.6 ms at 6W**, **34.0 ms at 8W**, and **43.7–77.9 ms at 10W** under L3 eviction (`ready_bytes` hwm `12.0–16.3 MiB`). Moreover, zero-copy block lending **already exists** as `#[doc(hidden)] pub fn StreamDecoder::read_block` (`src/parallel.rs:880–915`, used by `BzReader`'s `BufRead` and `examples/bzcat.rs`). Updated `surface.tsv` row `D` and linked `delivery-copyout; delivery-l3-working-set`.
- **Implication for `delivery-copyout`:** When next claimed, `delivery-copyout`'s question should be reframed around (a) measuring whether exposing `StreamDecoder::read_block` publicly (or a caller-slice `read` fast path) avoids `copy_out` at `k = 1..8` (`28–34 ms`) and `k = 10` (`44–78 ms`), and (b) coordinating with `delivery-l3-working-set` on `READY_BYTES`/`SLOTS_PER_THREAD` L3 eviction. *(Rows `C-A6b`, `E-spelling`, and `rauzy-output.toml` are intentionally left untouched in this PR because `rauzy-output` is actively held live.)*

---

### 5. Protocol–science convergence: `#952` + `#953` are a live proof-point for `#948`'s premise-tracking flag

Reading `#948` (`stress-test-frontier`) alongside `#949`/`#953` (`floor-and-pooled-scan` outline and draft), `#950` (`the-frontier-draft`), and `#952` (`explore`) shows a concrete interaction between the frontier protocol and the technical work:

- Within a single afternoon of parallel smooth-zone work (`max_live_holds = 3`), `#952` invalidated three headline readings from `#931` (`results/floor/README.md`: `D = 48 ms flat`, `E_consume = 298 ms` unadjusted, and no 8–10W memory cliff) that were simultaneously cited by an open claim (`delivery-copyout`), a live census (`rauzy-output`), a deferred registration (`pooled-scan-hybrid`), and two blog drafts (`the-frontier` `#950` and `floor-and-pooled-scan` `#953`, which merged immediately after `#952` pinned to `daf02245` and still citing `298.2 ms` `E_consume`, `47.3–49.3 ms` flat delivery, and no 8–10W memory cliff).
- Because `#948` found that `frontier.py` neither reads `premises` nor fetches `origin/master:frontier/flags/` (`2026-11-26-frontier-remote-flags-and-premises.md`), nothing in the tooling automatically alerted downstream claims or drafts when `#952` landed.
- **Concrete actions:**
  1. **`floor-and-pooled-scan`** (now at stage `draft` after `#953`, awaiting independent `review`) must fold `#952`'s cold-store caveats (`D` 29→78 ms, warm `E_consume` ~230–245 ms, `R` L3 cliff, `K1` cold-store asymmetry) into `review`/`revise` *before* `ready`/`published`, avoiding a second post-publication correction card like `bottlenecks-floor-correction`.
  2. **`the-frontier`** (at `draft`, awaiting independent `review`) quotes `#931`'s figures strictly as the historical 26 November handoff snapshot — which remains accurate — and can note `#952` + `#953`'s live premise drift as empirical validation of `harden-frontier-tooling`'s premise checks.
  3. **All new claims** (including the three `idea` claims opened here) populate `premises = [...]` with real repo paths so `harden-frontier-tooling`'s planned premise validator has live targets to check.

---

## Summary of claims opened (`status = "idea"`) and surface updates

| item | action | mechanism / rationale |
|---|---|---|
| `frontier/surface.tsv` row `R` | updated | `5.0–5.7% (66.6–74.4 ms)` at 1W; `104–115 ms CPU (7.1%)` at 8W; `2.55–2.9×` 10W LLC cliff (`133–216 ms`, `+767 MiB` DRAM); links `single-pass-rle1-crc` |
| `frontier/surface.tsv` row `D` | updated | `29.1 ms` at 1W (`48 ms` was cold-store artifact); `34.0 ms` at 8W; `43.7–77.9 ms` at 10W under 20 MiB L3 eviction; notes `StreamDecoder::read_block`; links `delivery-copyout; delivery-l3-working-set` |
| `frontier/claims/bidi-preshifted-inline.toml` | opened (`idea`) | Smooth probe testing pre-shifted `inline_back` inside full `emit_bidirectional` (with `edges[]` pointer-chasing) |
| `frontier/claims/delivery-l3-working-set.toml` | opened (`idea`) | Smooth probe sweeping `READY_BYTES` / `SLOTS_PER_THREAD` to break the coupled `R + D` 20 MiB L3 eviction cliff at 8–10W |
| `frontier/claims/warm-paired-oracle-k1.toml` | opened (`idea`) | Smooth methodological spec for cold-store-free `C`-paired and L3-aware `S`-paired `K1` registrations |
