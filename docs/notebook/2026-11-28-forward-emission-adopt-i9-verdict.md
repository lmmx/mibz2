# Physical i9-10900K C6 adoption verdict: `forward-emission-adopt` (`DO NOT ADOPT`, C2 preserved)

- **Claim:** `frontier/claims/forward-emission-adopt.toml` (P1, `build`, `striated` -> `status = "dropped"`)
- **Registration:** `docs/journal/2026-11-27-forward-adopt/registration.md` (`#1257` at `e505407b`, queued at `9875c7129e2dfd75ffd09234050956e6e7fbc753`, `rustc 1.99.1 (58cf8f9c3 2026-10-13)`)
- **Archived bundle:** `origin/evidence` `ccfa473f` (`forward-adopt/run-20261128T020039Z.tar.gz`)
  - **Bundle SHA-256:** `2514bb60d89ca634483486824d0124e69e336023a4f73b1783f5ec04e030617d`
  - **Base binary (`default`):** `e2d6d9842f3f358ed40154c8ea07e81f79e00d8acbc8aca527c9024b89b27147`
  - **Cand binary (`forward-anchor-lanes`):** `3afced3a0ac699c7be651db5ff23838a5b1bf9a3d05e276ad200431d58c228aa` (`distinct_binaries = true`)
- **Verdict (unrelaxed under RULES C2):** **`DO NOT ADOPT`**
  - Passed: `exactness_ok` (100% parity across 34 untimed `AB_SHA256=1` runs vs C `libbzip2` and all 544 timed repeats), `distinct_binaries`, `wikidata_1_hi_le_0.900`, `wikidata_10_hi_le_0.985`, `guards_no_regression` (12/12 guard cells clean!), `guards_rss_ok` (12/12 guard RSS `0.9965–1.0022×`).
  - Failed: **`wikidata_rss_ok`** (all 5 Wikidata cells exceeded the frozen RSS gates by `+1.72 MiB` at 1W and `+4.89..+6.94 MiB` at 6–12W, reaching `1.1049–1.1081×` at 8–12W) and **`wikidata_8_hi_le_0.965`** (`t_ci95_hi = 0.9859 > 0.965`, despite `13/16` faster wall rounds, paired median `0.9553`, and `16/16` faster CPU rounds).

---

## 1. Complete 17-cell physical i9-10900K table (`n = 16` paired rounds, `--settle 2`, `--prespin-ms 100`)

Every timed repeat verified `err == ""`, exact byte count, exact 64-bit hash tied to `AB_SHA256=1` `libbzip2` parity, and `cycles_per_ref = 1.0000` on every Wikidata cell (`0/160` slow-mode clock drops at `2 s` settle).

| Cell | `ab` arm | Base wall (s) | Cand wall (s) | Pooled `t_ratio` `[95% CI]` | Paired med | Pos-strat `[95% CI]` | Base → Cand RSS (MiB) | `Δ` RSS (MiB) | RSS ratio | `cpr` ratio | Clause |
|---|---|---:|---:|---|---:|---|---:|---:|---:|---:|---|
| `wikidata/1` | `mibz2-workers-lazy` | `1.5591` | `1.3201` | **`0.8462 [0.8450, 0.8474]`** | `0.8464` | `0.8462 [0.8451, 0.8474]` | `32.12 → 33.84` | **`+1.72`** | **`1.0541×`** | `1.0000` | wall PASS / **RSS FAIL** (`> 1.020`) |
| `wikidata/6` | `mibz2` | `0.2447` | `0.2116` | **`0.8648 [0.8594, 0.8702]`** | `0.8642` | `0.8648 [0.8594, 0.8701]` | `54.83 → 59.73` | **`+4.89`** | **`1.0890×`** | `1.0000` | **RSS FAIL** (`> 1.080`) |
| `wikidata/8` | `mibz2` | `0.1980` | `0.1899` | **`0.9638 [0.9422, 0.9859]`** | `0.9553` | `0.9638 [0.9427, 0.9854]` | `60.90 → 67.61` | **`+6.71`** | **`1.1081×`** | `1.0000` | **wall FAIL** (`> 0.965`) / **RSS FAIL** (`> 1.080`) |
| `wikidata/10` | `mibz2` | `0.1976` | `0.1900` | **`0.9649 [0.9541, 0.9758]`** | `0.9633` | `0.9649 [0.9537, 0.9762]` | `60.79 → 67.73` | **`+6.94`** | **`1.1067×`** | `1.0000` | wall PASS / **RSS FAIL** (`> 1.080`) |
| `wikidata/12` | `mibz2` | `0.1982` | `0.1898` | `0.9681 [0.9320, 1.0055]` | `0.9537` | `0.9681 [0.9309, 1.0067]` | `61.14 → 67.66` | **`+6.52`** | **`1.1049×`** | `1.0000` | **RSS FAIL** (`> 1.080`) |
| `text/1` | `mibz2-workers-lazy` | `0.0517` | `0.0513` | `0.9906 [0.9752, 1.0062]` | `0.9942` | `0.9906 [0.9749, 1.0065]` | `23.28 → 23.12` | `-0.16` | `1.0000×` | `0.9998` | PASS |
| `text/8` | `mibz2` | `0.0219` | `0.0222` | `1.0143 [0.9968, 1.0320]` | `1.0114` | `1.0143 [0.9981, 1.0307]` | `80.59 → 80.50` | `-0.09` | `0.9976×` | `1.0001` | PASS |
| `records/1` | `mibz2-workers-lazy` | `0.1040` | `0.1046` | `1.0039 [0.9967, 1.0112]` | `1.0034` | `1.0039 [0.9970, 1.0109]` | `32.97 → 32.81` | `-0.16` | `1.0000×` | `1.0001` | PASS |
| `records/8` | `mibz2` | `0.0378` | `0.0376` | `0.9945 [0.9889, 1.0002]` | `0.9960` | `0.9945 [0.9895, 0.9996]` | `71.46 → 71.54` | `+0.08` | `1.0009×` | `0.9927` | PASS |
| `random/1` | `mibz2-workers-lazy` | `0.0877` | `0.0884` | `1.0010 [0.9949, 1.0072]` | `1.0000` | `1.0010 [0.9947, 1.0074]` | `29.53 → 29.38` | `-0.16` | `1.0000×` | `1.0001` | PASS |
| `random/8` | `mibz2` | `0.0331` | `0.0335` | `1.0074 [0.9963, 1.0186]` | `1.0120` | `1.0074 [0.9963, 1.0186]` | `69.06 → 69.21` | `+0.15` | `1.0022×` | `1.0030` | PASS |
| `runs/1` | `mibz2-workers-lazy` | `0.0083` | `0.0083` | `0.9942 [0.9779, 1.0107]` | `0.9941` | `0.9942 [0.9773, 1.0114]` | `23.75 → 23.75` | `+0.00` | `1.0000×` | `1.0006` | PASS |
| `runs/8` | `mibz2` | `0.0116` | `0.0117` | `1.0094 [0.9886, 1.0307]` | `1.0043` | `1.0094 [0.9883, 1.0311]` | `19.69 → 19.69` | `+0.00` | `1.0000×` | `0.9954` | PASS |
| `pattern/1` | `mibz2-workers-lazy` | `0.0226` | `0.0225` | `0.9939 [0.9720, 1.0164]` | `0.9956` | `0.9939 [0.9716, 1.0167]` | `14.30 → 14.30` | `+0.00` | `1.0000×` | `1.0001` | PASS |
| `pattern/8` | `mibz2` | `0.0156` | `0.0155` | `0.9899 [0.9671, 1.0132]` | `0.9904` | `0.9899 [0.9667, 1.0135]` | `65.78 → 65.08` | `-0.70` | `0.9965×` | `0.9999` | PASS |
| `onebyte/1` | `mibz2-workers-lazy` | `0.0123` | `0.0123` | `0.9959 [0.9814, 1.0106]` | `0.9959` | `0.9959 [0.9831, 1.0088]` | `33.44 → 33.44` | `+0.00` | `1.0000×` | `1.0003` | PASS |
| `onebyte/8` | `mibz2` | `0.0157` | `0.0155` | `1.0044 [0.9855, 1.0236]` | `1.0063` | `1.0044 [0.9860, 1.0231]` | `29.22 → 29.22` | `+0.00` | `1.0000×` | `1.0006` | PASS |

---

## 2. Smoking-gun root cause: unbounded per-worker slab pool (`Arena::pool` + `Arena::used`)

The RSS failure is **100% real linear per-worker heap growth (`k × Δ_w`)**, not allocator jitter (matching the diagnostic signature in `docs/notebook/2026-11-28-rss-bound-calibration.md` §3.2):

- `wikidata/1` (`1` worker): `32.12 → 33.84 MiB` (`+1.72 MiB`, `1.0541× > 1.020×`)
- `wikidata/6` (`6` unpinned `OpenCells` workers): `54.83 → 59.73 MiB` (`+4.89 MiB`, `~0.815 MiB/worker`, `1.0890× > 1.080×`)
- `wikidata/8` (`8` unpinned `OpenCells` workers): `60.90 → 67.61 MiB` (`+6.71 MiB`, `~0.839 MiB/worker`, `1.1081× > 1.080×` and exceeding `PERFORMANCE-POLICY.md` §2's `10%` ceiling)
- `wikidata/10` (`8` unpinned `OpenCells` workers via `UNPINNED_FROM_TEN`): `60.79 → 67.73 MiB` (`+6.94 MiB`, `~0.868 MiB/worker`, `1.1067× > 1.080×`)
- `wikidata/12` (`8` unpinned `OpenCells` workers): `61.14 → 67.66 MiB` (`+6.52 MiB`, `~0.815 MiB/worker`, `1.1049× > 1.080×`)

In `src/open/lanes.rs`, `Arena` holds `pool: Vec<Box<[u8; 32768]>>` and `used: Vec<Box<[u8; 32768]>>`. Because all 4 lanes write into disjoint 32 KiB slabs across the entire 900 KiB block before stitching into `out` at the end of `walk()`, each worker allocates up to `ceil(900_000 / 32_768) + 4 = 32` distinct `Box<[u8; 32768]>` slabs (`~1.0 MiB` of slab memory **on top of** `out`'s `~0.9–1.0 MiB`, plus `slices`/`pending` metadata and glibc malloc chunk overhead). `Arena::reset()` moves `used` back into `pool` without releasing slabs, so every worker permanently retains a full extra copy of the block (~`0.84–1.0 MiB/worker`, `+6.71 MiB` across 8 workers).

By contrast, on all 6 guard corpora (`text`, `records`, `random`, `runs`, `pattern`, `onebyte`), `OpenCells` either runs on smaller blocks or falls back to `Segmented`, leaving guard RSS completely flat (`0.9965–1.0022×`).

---

## 3. Diagnosis of `wikidata/8` wall `t_ci95_hi = 0.9859` (3 unpinned SMT-placement rounds)

At `--settle 2`, the `#1240` clock slow-mode artefact was completely eliminated: every single round of `wikidata/6, 8, 10, 12` ran at **`4.88–4.89 GHz`** with **`cycles_per_ref = 1.3258–1.3264`** (`cpr ratio = 1.0000`), `runq_s <= 0.0003 s`, and `others_busy <= 0.15`.

On `wikidata/8`, `cand` beat `base` on **13/16 wall rounds** (paired median `0.9553`, `-4.47%`) and **16/16 CPU rounds** (`base` CPU `1.774–1.852 s`, median `1.792 s` vs `cand` CPU `1.596–1.768 s`, median `1.653 s`, **`-7.8%` total CPU**). Three unpinned rounds pulled `t_ci95_hi` from `~0.955` to `0.9859 > 0.965`:

| Round | Ratio | Base wall / CPU | Cand wall / CPU | Cand `cpr` / `eff_ghz` / `runq_s` / `others_busy` | Mechanism |
|---|---:|---|---|---|
| `r00` | `1.0254` | `0.1972 s` / `1.786 s` | `0.2022 s` / `1.768 s` | `1.3259` / `4.892 GHz` / `0.0000 s` / `0.06` | Unpinned scanner SMT sibling placement (`base` hit `0.2068 s` on `r04` and `0.2063 s` on `r13`) |
| `r08` | `1.0777` | `0.1969 s` / `1.788 s` | `0.2122 s` / `1.602 s` | `1.3262` / `4.884 GHz` / `0.0001 s` / `0.07` | Lower CPU (`1.602 s < 1.788 s`), longer wall (`+15.3 ms`): scanner sharing a core with an active worker |
| `r12` | `1.0182` | `0.1975 s` / `1.788 s` | `0.2011 s` / `1.669 s` | `1.3260` / `4.890 GHz` / `0.0001 s` / `0.05` | Same unpinned SMT placement tail (`cpu -6.7%` vs `base`) |

Note also that `wikidata/6` (`0.8648 [0.8594, 0.8702]`, **16/16 faster**, `-13.52%`) proves the huge speedup holds cleanly whenever the serial scanner is not the bottleneck (`6` workers on 10 physical cores), and `wikidata/10` (`0.9649 [0.9541, 0.9758]`, **15/16 faster**, `-3.51%`) passed its frozen `<= 0.985` gate cleanly.

---

## 4. Disposition and P1 rescue successor (Research persistence)

- **Unrelaxed verdict:** `forward-emission-adopt` is closed `status = "dropped"` with a scoped row in `frontier/dead-ends.tsv` preserving `DO NOT ADOPT` for the unbounded slab-pool variant (`#1217` `Arena::pool` retaining `~32 × 32 KiB` slabs per worker alongside `out`).
- **The opportunity is NOT dead — it is huge (`-15.4%` 1W, `-13.5%` 6W, `-4.5%` med / `-7.8%` CPU 8W, `-3.5%` 10W, 12/12 guards clean, `#![forbid(unsafe_code)]`)!**
- **P1 rescue successor filed:** `frontier/claims/forward-emission-bounded-slab-rescue.toml` (`status = "idea"`, `priority = 1`) targets the exact `+1.72 MiB` 1W / `+6.71 MiB` 8W slab-pool RSS cost via bounded-slab / streaming-RLE1+CRC32 emission (`<= 4 × 32 KiB = 128 KiB/worker` active scratch, eliminating the retained `~1.0 MiB/worker` slab pool and full-block double buffering so RSS stays `<= 1.020×` at 1W and `<= 1.030×` at 6/8/10/12W on i9 while keeping the full walk speedup and adding zero-stitch RLE1+CRC32 gains).
