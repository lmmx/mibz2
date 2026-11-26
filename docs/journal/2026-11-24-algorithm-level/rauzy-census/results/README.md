# Rauzy census results: A6b time split, induction-order census, online cost, and stop verdict

Thread 2 of [`../../plan.md`](../../plan.md), answering [`../README.md`](../README.md) and [`../prompt.md`](../prompt.md) (part 1). Characterisation only: zero changes to `src/`, zero `unsafe`, zero changes to production defaults.

**Harness:** [`examples/rauzy_census.rs`](../../../../../examples/rauzy_census.rs), mirroring the production Open kernel (`src/open.rs`, `src/open/endpoint.rs`, `src/open/outq.rs`, `LIMIT = 16`, `INLINE = 16`, `CERTIFIED_WIDTH = 32_768`, `Narrow`/`Wide` arena, `OutQ16`, `Dictionary`, `emit_bidirectional`) with sub-phase timers and ten pluggable induction orders on Open's exact auxiliary interval exchange.

**Machine & method (indicative, one thread):** Intel Xeon @ 2.30 GHz VM (2 vCPUs, AVX2), single-threaded, kernel struct reused across blocks. Every table reports the median and full `min..max` spread across 7 repeats (time split) or 5 repeats (order census). Every admitted block in every arm was verified byte-identical to `bwt::walk`, and baseline counts on `wikidata.bz2` match `examples/census.rs` (`--features census`) to the exact integer: `233` Open blocks, `209,697,662` rows, `15,141,602` runs, `26,567,581` retained rows, `755,021` thick strips, `368,867` words, `827,858` bands.

---

## Verdict (frozen stop test, `../README.md`)

**STOP — CLOSE THREAD 2.** Both stop conditions in `rauzy-census/README.md` fire decisively on Open's actual auxiliary interval exchange:

1. **Step share × step cut is well below the 3% wall gate (1.41% ceiling even at zero scheduling overhead):**
   - **A6b induction steps' share of A6b** on `wikidata` (`233` blocks): **51.13%** (spread `50.44..51.77%`; `wikidata ×4`: **51.15%**, `50.57..51.52%`). Setup takes **19.29%** and dictionary/expansion takes **29.59%**.
   - **Best step cut on Open's actual auxiliary IET** (`monodromy_1279`, free-first then largest cell): `2,668,970 → 2,223,511` steps = **−16.69%** (not MONODROMY's −29.4% on the full run-letter exchange, because Open already collapses thin runs `≤ 16` into gap nodes and every re-seating into the retained prefix `[0, m)` is already free of further elimination steps). Even against the proved single-cell floor `r2_E = 1,831,217`, the maximum conceivable cut is **−31.39%**.
   - **Implied 1-worker wall saving** (against `plan.md`'s 1,306 ms wall where A6b is 216 ms = 16.54%; `character-stage1` 1,283.1 ms gives 16.83%):
     - With `monodromy_1279` at **zero** scheduling overhead: `0.1654 × 0.5113 × 0.1669` = **1.41% of 1-worker wall** (~18.4 ms, or 1.44% against 1,283.1 ms) — **less than half the 3.0% gate** (~39 ms).
     - Even at the **proved floor `r2_E`** with zero overhead: `0.1654 × 0.5113 × 0.3139` = **2.65% of 1-worker wall** (< 3.0%).
2. **Every alternative order is slower on the clock than production right-end:**
   - **Cheap endpoint orders** (`left`, `two_sided_eq_max`, `two_sided_alt`, `right_div`, `right_zorich`) either increase step count or add branch/relink overhead: on `wikidata`, A6b runs at **1.014× to 1.110×** production right-end.
   - **Coordinate-tracked free-first orders** are computable online from what construction already holds (`strips_raw`, `image_order`, `thin`), **but cost 6.5× to 34× more per step**: `monodromy_1279` costs **831 ns/step** (**17.4×** A6b), and even a zero-`HashMap` flat open-addressed + log2-bucket online implementation (`online_flat`) costs **160 ns/step** (**3.79×** A6b) against production's **24.6 ns/step**. No rule gets within half the way of the step reduction at a cost below its saving.
3. **Consequence for `surface_dynamics`:** Do **not** port `surface_dynamics` (Rank #3 in `leads/research-software.md`). Thread 2 closes with this report; construction work moves to **A6a + A5 fusion** and **A6c + E forward emission** (threads 4–5).

---

## 1. A6b time split (production Open kernel mirror)

Corpora tested:
- `wikidata` (`corpora/wikidata.bz2`, SHA-256 checked against `bench/wikidata.json`, `431` blocks, `233` admitted by `OpenCells`);
- `wikidata ×4` (`corpora/wikidata-x4.bz2`, `1,724` blocks, `932` admitted by `OpenCells`);
- `text` (`corpora/text.bz2` from `ab gen corpora`, `25` blocks, `r/n = 0.2968` — refused by production `OpenCells` whose dense cap is `r/n < 0.28`, measured under `--forced`);
- `prose` (`corpora/prose.bz2`, repository `docs/**/*.md` compressed `bzip2 -9` as in MONODROMY §4, `11` blocks, `r/n = 0.3668` — 2 blocks pass pre-filter and are refused by `Cap::dense`, measured under `--forced`);
- `hex` (`corpora/hex.bz2`, uniform random `[0-9a-f]`, `5` blocks, `r/n = 0.9369` — refused by `OpenCells`, measured under `--forced`);
- `runs` (`corpora/runs.bz2` from `ab gen corpora`, `1` block, admitted by `OpenCells`).

Timer overhead check: the uninstrumented `Open (C+E)` run matches the sum of sub-phase timers within **0.18%–0.58%** on every multi-block corpus (`ratio 1.0018–1.0058`). Furthermore, the sub-phase split of `C` on `wikidata` (`A6a 33.6% / A6b 38.6% / A6c 28.1%`) matches `character-stage1/report.txt` on the i9-10900K (`200.3 / 216.0 / 161.4 ms` = `34.7% / 37.4% / 27.9%`) within ~1 percentage point.

### 1.1 Sub-phase breakdown (ns/row median, `min..max` over 7 repeats)

| corpus (admission) | Open blocks | open rows | steps | steps/run | steps/node | A6a ns/row | A6b-setup ns/row (share) | A6b-steps ns/row (share) | A6b-dict ns/row (share) | A6b total ns/row | A6c ns/row | A7 ns/row | ns/step |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| **wikidata** (prod) | 233/431 | 209,697,662 | 2,668,970 | 0.1763 | 1.4803 | 0.5260 | 0.1167 (**19.29%**) | 0.3071 (**51.13%**) | 0.1776 (**29.59%**) | 0.6037 | 0.4393 | 1.0821 | 24.13 |
| **wikidata ×4** (prod) | 932/1724 | 838,790,648 | 10,675,880 | 0.1763 | 1.4803 | 0.5254 | 0.1182 (**19.40%**) | 0.3092 (**51.15%**) | 0.1795 (**29.48%**) | 0.6039 | 0.4434 | 1.0865 | 24.29 |
| **text** (`ab gen`, forced) | 25/25 | 21,780,551 | 606,554 | 0.0938 | 1.5773 | 2.9911 | 0.1949 (**15.48%**) | 0.6701 (**53.41%**) | 0.3890 (**31.09%**) | 1.2413 | 2.1469 | 6.1430 | 24.06 |
| **prose** (`.md`, forced) | 11/11 | 9,323,029 | 267,845 | 0.0783 | 1.5893 | 4.1923 | 0.2119 (**18.83%**) | 0.5131 (**45.26%**) | 0.4067 (**35.92%**) | 1.1478 | 2.5035 | 7.3802 | 17.86 |
| **hex** (forced) | 5/5 | 3,600,807 | 143 | 0.0000 | 0.9795 | 10.9194 | 0.0028 (**30.61%**) | 0.0027 (**29.26%**) | 0.0037 (**38.40%**) | 0.0092 | 3.5152 | 13.5151 | 68.40 |
| **runs** (`ab gen`, prod) | 1/1 | 508,269 | 1,901 | 0.0210 | 3.3646 | 1.0352 | 0.0129 (**11.77%**) | 0.0687 (**62.55%**) | 0.0282 (**25.67%**) | 0.1104 | 0.8655 | 1.9314 | 18.37 |

### 1.2 Full spreads over 7 repeats (`wikidata` and `wikidata ×4`)

| part | wikidata ns/row (`min..max`) | wikidata share of A6b (`min..max`) | wikidata ×4 ns/row (`min..max`) | wikidata ×4 share of A6b (`min..max`) |
|---|---:|---:|---:|---:|
| A6a (first pass) | 0.5260 (`0.5142..0.5380`) | — | 0.5254 (`0.5149..0.5373`) | — |
| **A6b-setup total** | 0.1167 (`0.1142..0.1205`) | **19.29%** (`19.22..19.80%`) | 0.1182 (`0.1146..0.1194`) | **19.40%** (`19.12..19.60%`) |
| — A6b-pre (`image_order`) | 0.0094 (`0.0091..0.0097`) | 1.56% (`1.51..1.61%`) | 0.0097 (`0.0092..0.0098`) | 1.58% (`1.55..1.62%`) |
| — A6b-nodes (list build) | 0.1073 (`0.1048..0.1108`) | 17.72% (`17.63..18.25%`) | 0.1085 (`0.1054..0.1097`) | 17.84% (`17.57..18.01%`) |
| **A6b-steps (induction)** | 0.3071 (`0.3030..0.3125`) | **51.13%** (`50.44..51.77%`) | 0.3092 (`0.3021..0.3218`) | **51.15%** (`50.57..51.52%`) |
| **A6b-dict (expand+dedup)** | 0.1776 (`0.1749..0.1812`) | **29.59%** (`28.99..29.82%`) | 0.1795 (`0.1739..0.1833`) | **29.48%** (`29.36..29.86%`) |
| A6b total | 0.6037 (`0.5926..0.6089`) | 100.00% | 0.6039 (`0.5905..0.6245`) | 100.00% |
| A6c (wiring) | 0.4393 (`0.4325..0.4459`) | — | 0.4434 (`0.4306..0.4616`) | — |
| C total (`A6a+b+c`) | 1.5652 (`1.5393..1.5892`) | — | 1.5710 (`1.5360..1.6234`) | — |
| A7 (emit walk) | 1.0821 (`1.0699..1.0999`) | — | 1.0865 (`1.0583..1.1375`) | — |
| Open total (`C+E`) | 2.6516 (`2.6092..2.6742`) | uninst 2.6370 (`1.0056×`) | 2.6578 (`2.5942..2.7608`) | uninst 2.6531 (`1.0018×`) |

**Key structural observation on `prose` and `hex`:**
- Production `OpenCells` admits **0 blocks** on `text` (`r/n = 0.2968`), **0 blocks** on `prose` (`r/n = 0.3668`), and **0 blocks** on `hex` (`r/n = 0.9369`).
- Even under `--forced`, on `hex` almost every run has width `≤ 16` (`99.97%` of rows are retained, only `46` thick strips across `3.6M` rows), so A6b is **0.06% of C** (`0.0092 ns/row`, 143 steps total). On `prose`, A6b is only **14.6% of C** (`45.3%` of which is induction steps, i.e. 6.6% of C).

---

## 2. Census of induction orders on Open's auxiliary interval exchange

Open's `src/open/endpoint.rs` does **not** run Rauzy–Veech induction on the full `r`-run exchange; it runs on an auxiliary interval exchange on `[0, m + n)` where:
- the retained prefix `R = [0, m)` holds the `m` retained thin rows (`terminals[0]` = image gaps on side 0, `terminals[1]` = domain gaps on side 1);
- the eliminated suffix `E = [m, m + n)` holds `ns` thick strips plus the domain/image gaps (`open_nodes = ns + #domain_gaps + #image_gaps` total nodes), and induction stops as soon as `E` (width `n`) is emptied.

Because `m` is an invariant coincidence separating `R` from `E`, we can place a second sentinel `MID` between `terminals[side]` and `order[side]` (for `left` and `two_sided_*`) or restrict coincidences to `c > m` in coordinate space (for `monodromy_*` and `online_flat`). Any step whose loser lands in `R` (`p < m`) creates **zero** new boundaries in `E` and is therefore free for the `E`-elimination step count. By the exact boundary-count theorem on `E`:
- **Proved floor:** `steps ≥ r2_E = |{top_end > m} ∪ {bot_end > m}|`, with `steps = r2_E + U_E` (verified by assertion on every block).

### 2.1 Wikidata (all 233 Open blocks, `209,697,662` rows, `15,141,602` runs, `1,802,974` open nodes, 5 repeats)

Every order produced **exact `827,858` bands** and **100% byte-identical output to `bwt::walk`** across all 233 blocks. Proved floor on Open's auxiliary IET: **`r2_E = 1,831,217`** (`0.1209/run`, `1.0157/open_node`).

| order | steps | steps/run | steps/node | contracted/run | divs | rules | steps ns/row (`min..max`) | A6b ns/row (`min..max`) | ns/step | A6b ratio | vs `bwt::walk` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
| **right_prod** (baseline) | 2,668,970 | 0.1763 | 1.4803 | 0.1763 | 0 | 1,647,746 | 0.3131 (`0.3036..0.3579`) | 0.6113 (`0.5958..0.6773`) | 24.60 | **1.000×** | exact |
| left | 2,884,433 | 0.1905 | 1.5998 | 0.1905 | 0 | 1,938,205 | 0.3734 (`0.3634..0.4030`) | 0.6786 (`0.6547..0.7413`) | 27.15 | 1.110× | exact |
| two_sided_eq_max | 2,670,428 | 0.1764 | 1.4811 | 0.1764 | 0 | 1,650,789 | 0.3282 (`0.3263..0.3411`) | 0.6199 (`0.6091..0.6351`) | 25.77 | 1.014× | exact |
| two_sided_alt | 2,703,016 | 0.1785 | 1.4992 | 0.1785 | 0 | 1,727,941 | 0.3643 (`0.3601..0.3692`) | 0.6571 (`0.6500..0.6681`) | 28.26 | 1.075× | exact |
| right_div (always-on spiral div) | 2,668,970 | 0.1763 | 1.4803 | 0.1763 | 3 | 1,647,746 | 0.3557 (`0.3507..0.3589`) | 0.6455 (`0.6444..0.6511`) | 27.95 | 1.056× | exact |
| right_zorich (streak batch + div) | 2,668,970 | 0.1763 | 1.4803 | 0.1086 | 3 | 1,647,746 | 0.3672 (`0.3612..0.3797`) | 0.6574 (`0.6486..0.6777`) | 46.81* | 1.075× | exact |
| **monodromy_1279** (free, largest) | **2,223,511** | **0.1468** | **1.2332** | 0.1468 | 0 | 1,265,597 | 8.8142 (`8.7308..8.9517`) | 10.6445 (`10.5751..10.8215`) | 831.26 | 17.412× | exact |
| monodromy_free_right | 2,298,170 | 0.1518 | 1.2747 | 0.1518 | 0 | 1,308,924 | 6.1203 (`6.0929..6.2063`) | 7.6036 (`7.5675..7.7081`) | 558.45 | 12.438× | exact |
| monodromy_weak_max | 2,255,082 | 0.1489 | 1.2508 | 0.1489 | 0 | 1,298,170 | 9.0441 (`8.9394..9.6688`) | 10.9019 (`10.7801..11.6647`) | 841.00 | 17.833× | exact |
| **online_flat** (L2 flat map + buckets) | **2,236,586** | **0.1477** | **1.2405** | 0.1477 | 0 | 1,279,489 | 1.7017 (`1.6378..1.8171`) | 2.3190 (`2.2270..2.4784`) | 159.55 | 3.793× | exact |

*\*`right_zorich` reports ns per Zorich streak (`1,644,380` streaks, mean 1.62 sub-steps/streak; per raw sub-step it is `28.85 ns/step`).*

#### Per-block distribution across the 233 Wikidata Open blocks

| order | blocks | byte-identical | `steps/run` min / p10 / med / p90 / max | steps ns/row med (`p10..p90`) | A6b ns/row med (`p10..p90`) |
|---|---:|---|---|---:|---:|
| right_prod | 233 | 233/233 | 0.0198 / 0.1044 / 0.1949 / 0.2363 / 0.2650 | 0.3081 (`0.2219..0.3866`) | 0.6129 (`0.4290..0.7444`) |
| left | 233 | 233/233 | 0.0247 / 0.1089 / 0.2142 / 0.2591 / 0.2881 | 0.3802 (`0.2629..0.4940`) | 0.6869 (`0.4826..0.8543`) |
| two_sided_eq_max | 233 | 233/233 | 0.0198 / 0.1044 / 0.1948 / 0.2364 / 0.2646 | 0.3324 (`0.2449..0.4134`) | 0.6224 (`0.4801..0.7622`) |
| two_sided_alt | 233 | 233/233 | 0.0199 / 0.1030 / 0.2006 / 0.2396 / 0.2651 | 0.3728 (`0.2753..0.4488`) | 0.6785 (`0.5050..0.8032`) |
| right_div | 233 | 233/233 | 0.0198 / 0.1044 / 0.1949 / 0.2363 / 0.2650 | 0.3586 (`0.2726..0.4436`) | 0.6569 (`0.4827..0.7936`) |
| right_zorich | 233 | 233/233 | 0.0198 / 0.1044 / 0.1949 / 0.2363 / 0.2650 | 0.3715 (`0.2678..0.4543`) | 0.6729 (`0.4958..0.8101`) |
| monodromy_1279 | 233 | 233/233 | 0.0129 / 0.0847 / 0.1626 / 0.1970 / 0.2241 | 9.1853 (`6.2084..10.9045`) | 11.2202 (`7.4903..13.0326`) |
| monodromy_free_right | 233 | 233/233 | 0.0147 / 0.0903 / 0.1682 / 0.2018 / 0.2278 | 6.5043 (`4.6544..8.0417`) | 8.1505 (`5.6993..9.8574`) |
| monodromy_weak_max | 233 | 233/233 | 0.0133 / 0.0854 / 0.1654 / 0.1994 / 0.2250 | 9.5887 (`6.6206..11.5649`) | 11.6425 (`7.8241..14.0581`) |
| online_flat | 233 | 233/233 | 0.0133 / 0.0850 / 0.1645 / 0.1987 / 0.2217 | 1.6237 (`1.1437..2.0265`) | 2.2382 (`1.5401..2.7588`) |

### 2.2 Other corpora (`text`, `prose`, `hex` forced; `runs` production; 5 repeats, all byte-identical)

| corpus (floor `r2_E`) | order | steps | steps/run | steps/node | step Δ vs prod | steps ns/row (`min..max`) | A6b ratio |
|---|---|---:|---:|---:|---:|---:|---:|
| **text** (25 blk, floor `482,194`) | right_prod | 606,554 | 0.0938 | 1.5773 | 0.0% | 0.6677 (`0.6460..0.6927`) | 1.000× |
| | left | 588,619 | 0.0911 | 1.5306 | −3.0% | 0.7163 (`0.6789..0.7197`) | 1.038× |
| | two_sided_eq_max | 607,708 | 0.0940 | 1.5803 | +0.2% | 0.8412 (`0.8207..0.8520`) | 1.138× |
| | right_div | 606,554 | 0.0938 | 1.5773 | 0.0% | 0.6759 (`0.6569..0.6982`) | 1.007× |
| | right_zorich | 606,554 | 0.0938 | 1.5773 | 0.0% (0.0378 streaks/run) | 0.6468 (`0.6383..0.6547`) | 0.988× |
| | monodromy_1279 | 553,353 | 0.0856 | 1.4389 | **−8.8%** | 16.4765 (`16.3669..16.6164`) | 15.489× |
| | online_flat | 555,970 | 0.0860 | 1.4457 | −8.3% | 3.8950 (`3.8079..4.0164`) | 4.127× |
| **prose** (11 blk, floor `213,086`) | right_prod | 267,845 | 0.0783 | 1.5893 | 0.0% | 0.5323 (`0.5079..0.5403`) | 1.000× |
| | left | 255,872 | 0.0748 | 1.5182 | −4.5% | 0.5935 (`0.5722..0.5979`) | 1.031× |
| | two_sided_eq_max | 267,796 | 0.0783 | 1.5890 | −0.0% | 0.6124 (`0.5955..0.6814`) | 1.082× |
| | right_div (11 divs) | 267,845 | 0.0783 | 1.5893 | 0.0% | 0.6342 (`0.6147..0.6501`) | 1.070× |
| | right_zorich (11 divs) | 267,845 | 0.0783 | 1.5893 | 0.0% (0.0296 streaks/run) | 0.6340 (`0.6278..0.8480`) | 1.074× |
| | monodromy_1279 | 239,787 | 0.0701 | 1.4228 | **−10.5%** | 17.1390 (`16.3928..18.2550`) | 17.694× |
| | online_flat | 240,275 | 0.0703 | 1.4257 | −10.3% | 3.8073 (`3.6726..3.9339`) | 4.546× |
| **hex** (5 blk, floor `136`) | right_prod | 143 | 0.0000 | 0.9795 | 0.0% | 0.0023 (`0.0019..0.0024`) | 1.000× |
| | monodromy_1279 | 142 | 0.0000 | 0.9726 | −0.7% | 0.0216 (`0.0206..0.0220`) | 4.649× |
| | online_flat | 142 | 0.0000 | 0.9726 | −0.7% | 0.0093 (`0.0089..0.0096`) | 2.658× |
| **runs** (1 blk, floor `779`) | right_prod | 1,901 | 0.0210 | 3.3646 | 0.0% | 0.0733 (`0.0723..0.0741`) | 1.000× |
| | left | 851 | 0.0094 | 1.5062 | −55.2% | 0.0374 (`0.0324..0.0698`) | 0.684× |
| | right_zorich | 1,901 | 0.0210 | 3.3646 | 0.0% (0.0017 streaks/run) | 0.0552 (`0.0534..0.0644`) | 0.830× |
| | monodromy_1279 | 1,319 | 0.0146 | 2.3345 | −30.6% | 1.3586 (`1.2733..1.5795`) | 12.722× |
| | online_flat | 1,255 | 0.0139 | 2.2212 | −34.0% | 0.2718 (`0.2692..0.6531`) | 2.967× |

---

## 3. Online choice: decidability and measured cost per step

**Is the best step-count order (`monodromy_1279`) computable online during decode?**
**Yes — with zero oracle knowledge.** Before A6b begins, A6a has already packed the block's thick strips (`strips_raw`) and `image_order`, which completely determine the auxiliary interval exchange on `[0, m + n)`. Both `monodromy_1279` and `online_flat` use strictly those inputs.

**Why it still fails on the clock (cost per step vs break-even budget):**
- Production `right_prod` takes **24.6 ns/step** on `wikidata` (`0.313 ns/row`), because it only ever reads two sentinel neighbors (`prev(0, 0)` and `prev(0, 1)`), performs zero hash lookups, zero heap/priority-queue operations, and keeps the node arena compact (`Narrow`: 16 B/node, ~120 KiB/block, L2-resident).
- To beat `right_prod` on `wikidata` with `monodromy_1279`'s −16.69% step reduction (`2.669M → 2.224M`), an alternative order's step cost would have to stay strictly below `24.60 / 0.8331 = 29.5 ns/step` — i.e., **at most +4.9 ns/step** of scheduling overhead (≈11 cycles at 2.3 GHz, or ≈24 cycles on the i9-10900K), including setup!
- **Measured costs:**
  - `monodromy_1279` (`HashMap` + `BTreeSet` + `BinaryHeap`): **831 ns/step** (**+806 ns/step** overhead, **17.4×** A6b).
  - `monodromy_free_right` (drops the heap): **558 ns/step** (**12.4×** A6b).
  - `online_flat` (built specifically as a fast-path lower bound: power-of-two linear-probing `FlatMap`s reused across blocks, 21 `log2(cell)` bitmask buckets, zero `HashMap`/`BTreeSet`/`BinaryHeap`, zero per-block heap allocation): achieves almost the exact same step count (`2,236,586`, −16.20% vs −16.69%) at **159.6 ns/step** (**+135 ns/step** overhead, **3.79×** A6b) — still **27× over the +4.9 ns/step break-even budget**.
  - Even on `online_flat`, every step must execute at least 6–8 random probes into two open-addressed boundary maps (`top_end`, `bot_end`) to check whether the loser's landing point `p` coincides with a boundary and to maintain active coincidences. At ~15–20 ns per random table probe + update, a general interior-coincidence scheduler cannot fit inside a 4.9 ns/step budget.
  - Meanwhile, the link-only endpoint orders (`left`, `two_sided_*`) that *do* fit in ~26–28 ns/step give **zero or negative step reduction** on `wikidata` (`+0.05%` to `+8.1%` more steps).

---

## 4. Negative findings (what did not pay off)

1. **Open's auxiliary IET already captures much of MONODROMY's step gap:** On the full run-letter exchange (`r` runs), right-end took `1.812 r` steps against `1.279 r` (−29.4%, floor `1.220 r`). On Open's actual auxiliary IET (`open_nodes` thick strips + gaps), right-end takes `1.480` steps/node (`0.1763` steps/run) against `1.233` steps/node (`0.1468` steps/run) for `monodromy_1279` — a **−16.7% cut** (not −29.4%), because thin runs are already eliminated by retention and every landing inside `[0, m)` is already free.
2. **Zorich / division acceleration does nothing on natural corpora:** Across all 233 Wikidata blocks (`2.67M` steps), there are only **3 same-pair spirals** (`q ≥ 2`). Enabling the spiral check unconditionally (`right_div`) costs **+5.6% A6b** for zero step reduction. Full Zorich streak batching (`right_zorich`) averages only **1.62 sub-steps per streak** on Wikidata (`0.1763 → 0.1086` streaks/run), still has to execute every `join()` rule and relink every loser, and costs **+7.5% A6b**. (It helps only on degenerate synthetic `runs.bz2` with 12.5 sub-steps/streak, −17% A6b, where A6b is only 2.9% of Open time.)
3. **Left-end and two-sided orders lose on Wikidata:** `left` adds +8.1% steps and +17.6% rules (`1.110×` A6b); `two_sided_eq_max` adds +0.05% steps (`1.014×` A6b); `two_sided_alt` adds +1.3% steps (`1.075×` A6b).
4. **Band count is invariant across all ten orders:** Every order produces the exact same `827,858` retained bands on Wikidata (`330,858` on text, `147,531` on prose, `151` on hex, `785` on runs), so changing the induction order cannot save any work in `A6c` (wiring) or `A7` (emission).
5. **Non-step A6b work is half of A6b (48.9%):** Setup (`19.3%`, dominated by linked-list node construction at `17.7%`) and dictionary/expansion (`29.6%`) together match the induction loop (`51.1%`). Even if induction steps were eliminated entirely, A6b would only drop by half (~110 ms of 1,306 ms at 1 worker).
