# Rauzy output census (Round 2): LIMIT sweep, word representation & dictionary dedup, and induction-step cost

Claim [`rauzy-output`](../../../../../frontier/claims/rauzy-output.toml) — pricing the Rauzy machinery (`A6b`, 216 ms at 1 worker on i9) together with everything it drives (`A6a` 200.3 ms + `A6b` 216.0 ms + `A6c` 161.4 ms + `A7` walk+spelling 490.3 ms = **1,068.0 ms** of the 1,306 ms 1-worker wall on `wikidata ×4`). Characterisation only: zero edits to `src/`, zero `unsafe`, zero changes to production defaults.

**Harness:** [`examples/rauzy_output_census.rs`](../../../../../examples/rauzy_output_census.rs), mirroring the production Open kernel (`src/open.rs`, `src/open/endpoint.rs`, `src/open/outq.rs`, `LIMIT = 16`, `INLINE = 16`, `CERTIFIED_WIDTH = 32_768`, `Narrow`/`Wide` arena, `OutQ16`, `Dictionary`, `emit_bidirectional`/`emit`) with sub-phase timers (`A6a`, `A6b-setup`, `A6b-steps`, `A6b-dict`, `A6c`, `A7`) and pluggable policies for all three ideas.

**Machine, method & caveats:**
- **Hardware:** 2-vCPU Intel Xeon @ 2.30 GHz KVM VM (Haswell/Broadwell, AVX2, 32 KiB L1d, 256 KiB L2, 45 MiB L3), pinned to CPU 0 (`taskset -c 0`), single-threaded, reusable `OpenKernel` and `out` buffer across blocks (matching production's warm-cache block-by-block worker loop).
- **Repeats & exactness:** `7` interleaved repeats on `corpora/wikidata.bz2` (`233` production-admitted Open blocks, `209,697,662` rows) and `5` interleaved repeats on `wikidata ×4` (`932` Open blocks, `838,790,648` rows) for Ideas 2–3, preceded by a full warm-up pass verifying **100% byte-identical output to `bwt::walk` on every single admitted block** for every non-ablation arm.
- **i9-10900K scaling & frozen gate:** On the i9-10900K (`wikidata ×4`, [`character-stage1/report.txt`](../../math-targets/data/character-stage1/report.txt) & [`ideal-parts/results/floor/README.md`](../../ideal-parts/results/floor/README.md)), `A6a = 200.3 ms`, `A6b = 216.0 ms` (`setup 41.7`, `steps 110.4`, `dict 63.9`), `A6c = 161.4 ms`, `A7 = 490.3 ms` (of which `E_consume` spelling = `298.2 ms`), total `C+E = 1,068.0 ms` against `1,306 ms` 1-worker wall (`1,283.1 ms` in `character-stage1`). We report both **per-phase i9-anchored savings** (`i9_anc_Δ`, scaling each of `A6a`, `A6b`, `A6c`, `A7` against its exact i9 baseline) and **whole-`C+E` proportional savings** (`i9_prp_Δ`, scaling `C+E` against `1,068.0 ms`), evaluated against the **frozen stop test**: net saving across `A6a + A6b + A6c + A7` (cost included) **≥ 3.0% of the 1-worker wall (`≥ 39.18 ms` of `1,306 ms`)**, byte-identical to `bwt::walk` on every block.
- **8-worker caveat:** This VM has only 2 vCPUs, so 8-worker effects are modeled from the i9 `character-stage1` k=8 component breakdown (`wkr_busy = 184.7 ms`, `scan_busy = 185.0 ms`, `wall = 194.4 ms`, `A6a = 28.2`, `A6b = 30.4`, `A6c = 23.5`, `A7 = 69.5 ms/worker`) and the zero-copy floor (`scan_only = 179.1 ms`). Real 8-worker confirmation belongs on the i9 workstation under a frozen paired-oracle registration.

---

## Executive summary & verdicts (frozen 3.0% / 39.18 ms gate)

| idea | best mechanism tested | VM `C+E` ratio vs prod | i9-anchored Δ (`%` of 1,306 ms) | i9-prop Δ (`%` of 1,306 ms) | 8-worker effect (modeled, ungated) | verdict |
|---|---|---:|---:|---:|---|---|
| **1. `LIMIT` sweep & adaptive rules** | `limit_16_prod` is already optimal (`adapt_density` `1.0011×`; zero-cost per-block oracle `0.9660×` prod / `0.9794×` fixed-fwd) | `1.0011×` (`limit_12` `1.0050×`) | `+5.0 ms` (`+0.38%`); oracle `+36.3 ms` (`+2.78%`) | `−1.2 ms` (`−0.09%`) | `~0 ms` (neutral or regresses) | **STOP** |
| **2a. SLP expansion during emission** | `slp_emit_copy_out` vs `prod_forward_emit` (controlled); `slp_emit_dfs` vs `prod_forward_emit` | `1.077–1.080×` (`dfs` `1.556–1.562×`) | `−73..−80 ms` vs fwd (`dfs` `−560 ms`) | `−72..−75 ms` vs fwd | regresses worker CPU ~`+9–10 ms/wkr` | **KILL** |
| **2b. Dictionary dedup & slack trim** | `dict_no_contents_dedup` (`dict_trim_memo_slack` neutral `0.998–1.012×`) | `0.9876–0.9912×` | `+8.7..+12.9 ms` (`+0.67..+0.99%`) | `+9.4..+13.3 ms` (`+0.72..+1.02%`) | `~1.5 ms/wkr` (~0.8% worker busy; negligible) | **STOP** |
| **2c. Pre-shifted `inline_back` in `Dictionary`** | `bidi_pre_shifted` (pre-right-aligned back-side inline table, full bidi walk+spell) | **`0.9284–0.9353×`** | **`+76.4..+84.3 ms` (`+5.85..+6.46%`)** | **`+69.1..+76.5 ms` (`+5.29..+5.86%`)** | `~10.4 ms/wkr` (−5.6% wkr busy; ~2–4 ms wall alone, ~10 ms with `S ideal`) | **PASS** *(build claim `preshifted-inline-back`, superseded if `forward-emission` lands)* |
| **3. Cheaper induction steps** | `steps_inline7_node20` (−17.4–18.4% steps, −12.5–13.3% A6b; prefetch & interleaving 1.14–1.68× slower) | `0.964–0.982×` (`A6b` `0.867–0.875×`) | `+29.5..+32.4 ms` (`+2.26..+2.48%`) | `+19.3..+38.1 ms` (`+1.48..+2.92%`) | `~3.8–4.0 ms/wkr` (−2.1% wkr busy; ~1 ms wall alone, ~4 ms with `S ideal`) | **STOP** *(strict C2: < 3.0% gate)* |

*(Reference cross-check: `prod_forward_emit` — replacing `wire_xor` + `emit_bidirectional` with single-lane `wire` + forward `emit` — cuts `C+E` to `0.856–0.871×`, `+141.7..+157.4 ms` = `+10.9..+12.1%` of 1w wall, confirming `forward-emission` as the dominant representation win and superseding `preshifted-inline-back` once shipped.)*

---

## 1. Idea 1: Thin/thick `LIMIT` sweep & adaptive rules

**Hypothesis:** Motivated by [`spelling/README.md`](../../spelling/README.md) showing that 81.6% of emitted codes are 1-byte literals (mean streak 8.81) and `A6c + A7` costs ~652 ms on i9 against `A6b`'s 216 ms, lowering `LIMIT` below 16 (or adapting `LIMIT` per block / per byte from A6a's counts) might shrink retained rows `m` enough to cut `A6c + A7` by more than the added induction cost in `A6b`.

### 1.1 Structural counts across `LIMIT`s (`wikidata`, 233 Open blocks, `209,697,662` rows)

Every arm was verified **100% byte-identical to `bwt::walk`** across all 233 blocks. Zero blocks tripped `Wide` arena (`wideA = 0`) or `Wide` edges (`wideE = 0`).

| arm | `m` (codes) | thick strips | steps | bands | dict/blk | `spelling` KiB/blk | `lit%` | `long%` | `m/n%` | bidi blks |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `limit_2` | 14,055,081 | 2,688,203 | 9,009,727 | 2,193,424 | 2,316 | 281.3 | 81.58% | 8.43% | 6.70% | 44 |
| `limit_3` | 15,529,148 | 2,196,847 | 7,308,293 | 1,840,635 | 1,989 | 201.0 | 82.18% | 8.09% | 7.41% | 65 |
| `limit_4` | 17,059,348 | 1,814,297 | 6,204,802 | 1,645,940 | 1,797 | 156.5 | 82.07% | 7.67% | 8.14% | 81 |
| `limit_6` | 18,778,589 | 1,500,000 | 5,091,825 | 1,401,552 | 1,540 | 111.0 | 82.06% | 7.42% | 8.96% | 161 |
| `limit_8` | 20,390,399 | 1,285,937 | 4,373,604 | 1,239,982 | 1,384 | 86.6 | 82.09% | 7.22% | 9.72% | 218 |
| `limit_10` | 22,377,102 | 1,075,212 | 3,724,122 | 1,089,285 | 1,269 | 68.9 | 82.10% | 6.97% | 10.67% | 231 |
| `limit_12` | 24,023,630 | 931,753 | 3,255,677 | 969,950 | 1,174 | 57.1 | 82.03% | 6.76% | 11.46% | 232 |
| **`limit_16_prod`** | **26,567,581** | **755,021** | **2,668,970** | **827,858** | **1,059** | **42.4** | **81.58%** | **6.43%** | **12.67%** | **232** |
| `limit_24` | 30,778,141 | 547,472 | 2,001,614 | 661,195 | 922 | 28.0 | 80.72% | 5.81% | 14.68% | 233 |
| `limit_32` | 33,853,392 | 437,892 | 1,635,950 | 562,580 | 829 | 21.5 | 80.34% | 5.33% | 16.14% | 233 |
| `limit_64` | 41,574,728 | 266,039 | 1,006,548 | 369,031 | 632 | 11.4 | 80.69% | 4.43% | 19.83% | 233 |
| `adapt_density` | 21,472,348 | 1,173,196 | 3,995,572 | 1,145,068 | 1,313 | 78.5 | 82.08% | 7.03% | 10.24% | 218 |
| `adapt_hist_r12` | 24,023,630 | 931,753 | 3,255,677 | 969,950 | 1,174 | 57.1 | 82.03% | 6.76% | 11.46% | 232 |
| `adapt_per_byte` | 23,093,662 | 1,025,849 | 3,626,077 | 1,074,669 | 1,327 | 65.7 | 81.30% | 6.96% | 11.01% | 232 |

**Structural surprise:** Lowering `LIMIT` from `16` all the way to `2` cuts retained rows `m` by **−47.1%**, yet **`lit%` (fraction of emitted codes that are 1-byte literals) stays completely invariant at 81.6%–82.2%**. Remaining thin runs still land predominantly in gaps, while `strips` grows **3.56×**, `steps` grows **3.38×**, and `spelling` grows **6.6×** (`42.4 → 281.3 KiB/blk`, spilling the 256 KiB L2!).

### 1.2 End-to-end timings (`A6a + A6b + A6c + A7`, ns/row median & `min..max` over 7 interleaved repeats)

| arm | `A6a` | `A6b` | `A6c` | `A7` | `C+E` med (`min..max`) | ratio vs prod | `i9_anc_Δ` | `i9_prp_Δ` | `%` 1w wall |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `limit_2` | 0.7373 | 2.2917 | 0.2964 | 0.6582 | 3.9778 (`3.7944..4.8277`) | 1.3075× | −296.8 ms | −328.4 ms | −22.73% |
| `limit_3` | 0.7210 | 1.7962 | 0.2867 | 0.7312 | 3.5486 (`3.4349..3.5931`) | 1.1528× | −137.8 ms | −163.2 ms | −10.55% |
| `limit_4` | 0.7265 | 1.5228 | 0.3075 | 0.8044 | 3.3610 (`3.1804..3.4757`) | 1.0964× | −80.6 ms | −102.9 ms | −6.17% |
| `limit_6` | 0.6980 | 1.2134 | 0.3543 | 0.9010 | 3.1797 (`3.0780..3.4185`) | 1.0350× | −21.6 ms | −37.4 ms | −1.65% |
| `limit_8` | 0.6842 | 1.0208 | 0.3962 | 0.9978 | 3.1098 (`3.0488..3.3070`) | 1.0183× | −7.9 ms | −19.6 ms | −0.60% |
| `limit_10` | 0.6846 | 0.8808 | 0.4199 | 1.1015 | 3.0869 (`3.0092..3.3675`) | 1.0203× | −14.4 ms | −21.7 ms | −1.10% |
| `limit_12` | 0.6844 | 0.7771 | 0.4403 | 1.2097 | 3.1339 (`3.0063..3.6312`) | 1.0050× | −1.6 ms | −5.3 ms | −0.12% |
| **`limit_16_prod`** | **0.6755** | **0.6308** | **0.4550** | **1.2718** | **3.0220 (`2.9944..3.2164`)** | **1.0000×** | **+0.0 ms** | **+0.0 ms** | **+0.00%** |
| `limit_24` | 0.7090 | 0.4734 | 0.5078 | 1.4397 | 3.1443 (`3.0607..3.2281`) | 1.0379× | −46.3 ms | −40.5 ms | −3.54% |
| `limit_32` | 0.7254 | 0.3862 | 0.5629 | 1.5745 | 3.2446 (`3.1699..3.4446`) | 1.0710× | −83.3 ms | −75.8 ms | −6.38% |
| `limit_64` | 0.7641 | 0.2382 | 0.6541 | 1.9065 | 3.5802 (`3.4872..3.6276`) | 1.1692× | −195.9 ms | −180.7 ms | −15.00% |
| `limit_32_fast64q` | 0.7385 | 0.3781 | 0.5461 | 1.5693 | 3.2439 (`3.1736..3.3324`) | 1.0581× | −68.3 ms | −62.1 ms | −5.23% |
| `limit_64_fast64q` | 0.7794 | 0.2390 | 0.6676 | 1.9099 | 3.6080 (`3.5151..3.7242`) | 1.1721× | −202.1 ms | −183.8 ms | −15.47% |
| `adapt_density` | 0.6376 | 0.9443 | 0.3994 | 1.0440 | 3.0253 (`2.9829..3.2851`) | 1.0011× | +5.0 ms | −1.2 ms | +0.38% |
| `adapt_hist_r12` | 0.8694 | 0.7578 | 0.4278 | 1.1711 | 3.2260 (`3.1744..3.3842`) | 1.0544× | −42.4 ms | −58.1 ms | −3.24% |
| `adapt_per_byte` | 0.8801 | 0.8417 | 0.4207 | 1.1214 | 3.2695 (`3.2266..3.3328`) | 1.0734× | −62.1 ms | −78.4 ms | −4.76% |
| *Fixed-forward control:* | | | | | | | | | |
| `fwd_limit_4` | 0.7040 | 1.4731 | 0.2156 | 0.7468 | 3.1400 (`3.0737..3.3782`) | 1.0280× | −7.5 ms | −29.9 ms | −0.57% |
| `fwd_limit_6` | 0.6927 | 1.2019 | 0.2070 | 0.8324 | 2.9341 (`2.8602..3.1044`) | 0.9625× | +52.9 ms | +40.1 ms | +4.05% |
| `fwd_limit_8` | 0.6857 | 1.0252 | 0.2004 | 0.8928 | 2.7979 (`2.7546..3.0390`) | 0.9245× | +95.9 ms | +80.6 ms | +7.35% |
| `fwd_limit_12` | 0.7079 | 0.7822 | 0.1816 | 1.1065 | 2.8265 (`2.5867..2.9301`) | 0.8989× | +117.5 ms | +108.0 ms | +9.00% |
| **`fwd_limit_16`** | **0.6745** | **0.6428** | **0.1692** | **1.1878** | **2.6780 (`2.5478..2.8365`)** | **0.8718×** | **+139.9 ms** | **+137.0 ms** | **+10.71%** |
| `fwd_limit_24` | 0.7282 | 0.4830 | 0.1535 | 1.4601 | 2.8412 (`2.5935..3.6953`) | 0.9259× | +75.5 ms | +79.1 ms | +5.78% |

**Per-block oracle upper bound & dispatch-threshold confound:**
- Under production dispatch, a zero-cost per-block oracle picking the single fastest fixed `LIMIT` per block achieves only **`0.9660×`** (`+36.3 ms` = **`+2.78%`** of 1,306 ms < 3.0% gate). Moreover, its argmin (`limit_6 = 41`, `limit_16_prod = 76`, `limit_24 = 32`, ...) is artificially boosted by the `m >= 65_536` dispatch threshold (`src/open.rs:522`), where dropping `LIMIT` to 6 pushes 71 blocks from `emit_bidirectional` into faster forward `emit`.
- Holding dispatch fixed (`ForceForward`, matching `forward-emission`), `fwd_limit_16` wins on **158 of 233 blocks (67.8%)** and beats both `fwd_limit_12` (+5.5%) and `fwd_limit_24` (+6.1%). The fixed-forward per-block oracle ceiling is only **`0.9794×`** (**`+2.06%`** of forward `C+E` ≈ `+19 ms` = **`+1.45%`** of 1w wall).
- Widening `outq::Queues::push`'s fast path from `w <= 16` to `w <= 64` (`limit_32_fast64q`, `limit_64_fast64q`) leaves high `LIMIT`s 5.8–17.2% slower, proving the penalty at `LIMIT > 16` is `A6c + A7` (`m` growth), not `push_slow`.
- Any pre-pass over `records` (`adapt_hist_r12`, `adapt_per_byte`) adds `+0.19..+0.20 ns/row` in `A6a` — dwarfing any `LIMIT` gain.

**Verdict (Idea 1): STOP.** Production `LIMIT = 16` is already at the U-curve minimum under both bidirectional and forward emission; even a zero-cost per-block oracle fails the 3% gate.

---

## 2. Idea 2: Words' representation & dictionary dedup

**Hypothesis:** (a) Skipping A6b's flat `spelling` expansion in favor of SLP expansion during emission (copying earlier expansions from `out` via `copy_within`, or word-by-word DFS) might cut `A6b-dict` (63.9 ms on i9). (b) Deferring/simplifying `Dictionary` dedup, trimming `32.3%` dead `memo_dedup_slack` (`3.27 MB`), or pre-shifting `inline_back` in `Dictionary` might yield net savings across `A6b-dict + A7`.

### 2.1 Dictionary footprint & slack (`wikidata`, 233 Open blocks; identical per-block on `wikidata ×4`)

| mode | dict/blk | long/blk | `spelling` total KiB/blk | `spelling` used KiB/blk | `memo_dedup_slack` | `wideE` | bidi blks |
|---|---:|---:|---:|---:|---:|---:|---:|
| `prod_baseline` | 1,059 | 276 | 42.4 | 28.7 | 32.3% | 0 | 232 |
| `no_memo_expand` | 1,059 | 276 | 28.7 | 28.7 | 0.0% | 0 | 232 |
| `dict_trim_memo_slack` | 1,059 | 276 | 28.7 | 28.7 | 0.0% | 0 | 232 |
| `dict_no_contents_dedup` | 1,612 | 454 | 42.8 | 42.8 | 0.0% | 0 | 232 |
| `dict_no_dedup` | 1,839 | 459 | 42.9 | 42.9 | 0.0% | 0 | 232 |
| `dict_p2048_inline_dedup` | 1,236 | 454 | 42.8 | 42.8 | 0.0% | 0 | 232 |
| `dict_p2048_long_dedup` | 1,434 | 276 | 42.4 | 28.7 | 32.3% | 0 | 232 |
| **`bidi_pre_shifted`** | **1,059** | **276** | **42.4** | **28.7** | **32.3%** | **0** | **232** |
| `slp_emit_copy_out` / `slp_emit_dfs` | 1,236 | 454 | 0.0 | 0.0 | 0.0% | 0 | 0 |

### 2.2 End-to-end timings (`wikidata` 7 repeats & `wikidata ×4` 5 repeats, all 100% byte-identical to `bwt::walk`)

#### `wikidata` (233 blocks, `209.7 MB`, 7 interleaved repeats)

| mode | `A6b-dct` | `A6b` | `A6c` | `A7` | `C+E` med (`min..max`) | ratio | `i9_anc_Δ` | `i9_prp_Δ` | `%` 1w wall |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| **`prod_baseline`** | **0.2009** | **0.6217** | **0.4532** | **1.2412** | **2.9883 (`2.9526..3.3592`)** | **1.0000×** | **+0.0 ms** | **+0.0 ms** | **+0.00%** |
| `no_memo_expand` | 0.2447 | 0.6660 | 0.4530 | 1.2483 | 3.0226 (`3.0028..3.4299`) | 1.0210× | −22.2 ms | −22.5 ms | −1.70% |
| `dict_trim_memo_slack` | 0.2105 | 0.6314 | 0.4571 | 1.2567 | 3.0068 (`2.9815..3.1314`) | 1.0116× | −12.6 ms | −12.4 ms | −0.97% |
| `dict_no_contents_dedup` | 0.1704 | 0.5899 | 0.4545 | 1.2601 | 2.9677 (`2.9262..3.0753`) | 0.9912× | +8.7 ms | +9.4 ms | +0.67% |
| `dict_no_dedup` | 0.1674 | 0.5867 | 0.4509 | 1.2505 | 2.9378 (`2.9225..3.0005`) | 0.9914× | +8.5 ms | +9.1 ms | +0.65% |
| `dict_p2048_inline_dedup` | 0.1892 | 0.6084 | 0.4554 | 1.2444 | 2.9742 (`2.9239..3.2146`) | 1.0054× | −6.0 ms | −5.8 ms | −0.46% |
| `dict_p2048_long_dedup` | 0.1820 | 0.6019 | 0.4553 | 1.2471 | 2.9593 (`2.9404..3.0512`) | 0.9993× | +0.1 ms | +0.7 ms | +0.00% |
| **`bidi_pre_shifted`** | **0.2027** | **0.6171** | **0.4495** | **1.0312** | **2.7525 (`2.7401..2.8456`)** | **0.9284×** | **+84.3 ms** | **+76.5 ms** | **+6.46%** |
| *Forward-emit controls & SLP:* | | | | | | | | | |
| `prod_forward_emit` | 0.1982 | 0.6141 | 0.1621 | 1.1007 | 2.5405 (`2.5281..2.6977`) | 0.8561× | +157.4 ms | +153.7 ms | +12.05% |
| `prod_forward_unroll4` | 0.2034 | 0.6241 | 0.1659 | 1.2372 | 2.7097 (`2.5900..3.0146`) | 0.8974× | +111.1 ms | +109.6 ms | +8.51% |
| `slp_emit_copy_out` | 0.1178 | 0.5343 | 0.1632 | 1.3851 | 2.7496 (`2.7198..2.8303`) | 0.9210× *(1.0769× vs fwd)* | +78.4 ms *(−79.0 vs fwd)* | +84.3 ms | +6.00% *(−6.05% vs fwd)* |
| `slp_emit_dfs` | 0.1170 | 0.5356 | 0.1618 | 2.6167 | 3.9712 (`3.9396..4.3571`) | 1.3438× *(1.5619× vs fwd)* | −417.8 ms | −367.1 ms | −31.99% |

#### `wikidata ×4` confirmation (932 blocks, `838.8 MB`, 5 interleaved repeats)

| mode | `A6b-dct` | `A6b` | `A6c` | `A7` | `C+E` med (`min..max`) | ratio | `i9_anc_Δ` | `i9_prp_Δ` | `%` 1w wall |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| **`prod_baseline`** | **0.2025** | **0.6172** | **0.4527** | **1.2493** | **2.9794 (`2.9662..3.0459`)** | **1.0000×** | **+0.0 ms** | **+0.0 ms** | **+0.00%** |
| `no_memo_expand` | 0.2478 | 0.6736 | 0.4576 | 1.2762 | 3.0766 (`3.0021..3.2170`) | 1.0123× | −12.7 ms | −13.1 ms | −0.97% |
| `dict_trim_memo_slack` | 0.2073 | 0.6302 | 0.4550 | 1.2645 | 3.0240 (`2.9667..3.0382`) | 0.9975× | +2.7 ms | +2.7 ms | +0.21% |
| `dict_no_contents_dedup` | 0.1735 | 0.5916 | 0.4560 | 1.2655 | 2.9829 (`2.9416..3.0080`) | 0.9876× | +12.9 ms | +13.3 ms | +0.99% |
| `dict_no_dedup` | 0.1671 | 0.5870 | 0.4550 | 1.2540 | 2.9477 (`2.9419..3.0190`) | 0.9912× | +8.9 ms | +9.4 ms | +0.68% |
| **`bidi_pre_shifted`** | **0.2070** | **0.6299** | **0.4570** | **1.0615** | **2.8145 (`2.7489..2.8389`)** | **0.9353×** | **+76.4 ms** | **+69.1 ms** | **+5.85%** |
| `prod_forward_emit` | 0.2046 | 0.6294 | 0.1675 | 1.1397 | 2.6095 (`2.5848..2.6795`) | 0.8706× | +141.7 ms | +138.3 ms | +10.85% |
| `slp_emit_copy_out` | 0.1197 | 0.5454 | 0.1695 | 1.4570 | 2.8487 (`2.7713..2.8766`) | 0.9354× *(1.0795× vs fwd)* | +61.8 ms *(−79.9 vs fwd)* | +69.0 ms | +4.73% *(−6.12% vs fwd)* |
| `slp_emit_dfs` | 0.1184 | 0.5399 | 0.1673 | 2.6567 | 4.0283 (`3.9904..4.5037`) | 1.3574× *(1.5562× vs fwd)* | −431.9 ms | −381.7 ms | −33.07% |

**Key findings & verdicts (Idea 2):**
1. **SLP-during-emission (`slp_emit_copy_out`, `slp_emit_dfs`, `no_memo_expand`) → KILL:** Controlled against `prod_forward_emit`, skipping A6b's flat `spelling` buffer saves `−0.081..−0.085 ns/row` in `A6b-dict` (−41%), but `slp_emit_copy_out` adds **`+0.279..+0.291 ns/row` in `A7`** (+25–26%) because copying earlier expansions out of the 900 KiB `out` buffer (L3/DRAM) loses **3.5× harder** than copying from 42.4 KiB L2-resident `spelling`. Net vs `prod_forward_emit`: **`1.077–1.080×` slower** (`slp_emit_dfs`: `1.556–1.562×` slower). Meanwhile `no_memo_expand` confirms production's memoized `extend_from_within` inside A6b beats unmemoized DFS by `+1.2%–2.1%`.
2. **Dictionary dedup & trimming 32.3% `memo_dedup_slack` → STOP:** Eliminating 100% of `memo_dedup_slack` via undo-list truncation (`dict_trim_memo_slack`, `42.4 → 28.7 KiB/blk`) is neutral (`0.998–1.012×`) because `42.4 KiB` already fits inside 256 KiB L2 on 233/233 blocks. Dropping `contents` hashing (`dict_no_contents_dedup`) saves 15% of `A6b-dict` with zero `A7` regression (`wideE = 0`, `spelling` `42.8 KiB/blk`), but caps out at **`+8.7..+12.9 ms` (`+0.67..+0.99%` of 1w wall)** — mathematically far below the 3% gate because all of `A6b-dict` is only `63.9 ms` (`4.9%` of 1w wall).
3. **Pre-shifted `inline_back` in `Dictionary` (`bidi_pre_shifted`) → PASS:** Resolves `spelling-census`'s caveat cleanly: even **with the full pointer-chase walk running in `emit_bidirectional`**, pre-computing right-aligned `inline_back` once per dictionary entry (~1,059 entries/block, ~0–0.004 ns/row in `A6b-dict`) cuts FULL `A7` by **−15.0% to −16.9%** (`1.2493 → 1.0615 ns/row` on `wikidata ×4`, `1.2412 → 1.0312` on `wikidata`), giving **`0.9353×` (`wikidata ×4`, zero overlap in `min..max`: `2.7489..2.8389` vs `2.9662..3.0459`)** and **`+76.4 ms` anchored / `+69.1 ms` prop = `+5.85%` / `+5.29%` of the 1,306 ms wall** — nearly **2× the frozen 3.0% (39.18 ms) gate**. Opened as follow-on `build` claim [`preshifted-inline-back`](../../../../../frontier/claims/preshifted-inline-back.toml) at `status = "idea"` (noting it is superseded if `forward-emission` replaces `emit_bidirectional`).

---

## 3. Idea 3: Cheaper induction steps & dependent-load chain diagnosis

**Hypothesis:** Round 1 measured production induction at `24.6 ns/step` (`110.4 ms` on i9 = `8.45%` of 1w wall). If those ~55 cycles/step are stalled on a dependent-load pointer chain across the ~256 KiB L2 boundary, prefetch, inter-block interleaving, or layout compaction might break the chain and save ≥ 3% of the 1w wall.

### 3.1 Step anatomy (`wikidata`, 233 Narrow non-spiral blocks, `2,668,970` steps)

- **Arena `(nodes + rules)` footprint:** `p10 = 169.8`, **`p50 = 239.5 KiB`**, `p90 = 275.0`, `p99 = 356.1`, `max = 397.7 KiB` — **63/233 blocks (27.0%) exceed the 256 KiB L2 cache**.
- **Step branches:** `x == y` **0.00%**, `aw < bw` **23.68%**, `aw > bw` **39.34%**, **`aw == bw` 36.98%** (over a third of all steps are equal-width eliminations!). Simulated 2-bit predictor misprediction rate on `aw <= bw`: **26.58%**.
- **`join()` mix:** zero-arg (`left == 0 || right == 0`) **38.26%**, non-zero **61.74%** — of non-zero joins, **44.1% have `len ≤ 4`**, **60.0% have `len ≤ 7`**, **76.1% have `len < 16`**, 23.9% have `len ≥ 16`, and only 0.01% saturate (`≥ 1023`).

### 3.2 End-to-end step optimization timings (100% byte-identical on all 9 exact modes)

#### `wikidata` (233 blocks, 7 repeats) & `wikidata ×4` (932 blocks, 5 repeats)

| mode | arena KiB/blk | `×1` ns/step (`stp_rat`) | `×4` ns/step (`stp_rat`) | `×4` `A6b-dct` | `×4` `A6b` (`a6b_rat`) | `×4` `C+E` | `×1` / `×4` `i9_anc_Δ` (`%` 1w) |
|---|---:|---:|---:|---:|---:|---:|---:|
| **`prod_baseline`** | **231.4** | **23.82 (1.000×)** | **23.76 (1.000×)** | **0.2030** | **0.6209 (1.000×)** | **2.9918** | **+0.0 / +0.0 ms (+0.00%)** |
| `ablat_no_join` *(ablat)* | 120.9 | 11.53 (**0.487×**) | 11.67 (**0.497×**) | — | — | — | *(isolates 50% `join()` share)* |
| `steps_reg_xy` | 231.4 | 25.23 (1.064×) | 24.90 (1.053×) | 0.1987 | 0.6337 (1.022×) | 2.9846 | −7.3 / −0.5 ms (−0.04%) |
| `steps_hoist_next` | 231.4 | 29.36 (1.233×) | 28.84 (1.218×) | 0.1981 | 0.6831 (1.100×) | 3.0297 | −25.7 / −11.9 ms (−0.91%) |
| `steps_branchless_side` | 231.4 | 24.52 (1.030×) | 24.45 (1.027×) | 0.2001 | 0.6296 (1.010×) | 2.9940 | −2.4 / +7.2 ms (+0.55%) |
| `steps_typed_narrow` | 231.4 | 21.82 (0.927×) | 21.70 (0.918×) | 0.1874 | 0.5689 (0.916×) | 2.9743 | +3.1 / +5.3 ms (+0.41%)* |
| `steps_typed_prefetch` | 231.4 | 24.40 (1.024×) | 24.75 (1.040×) | 0.1861 | 0.6116 (0.974×) | 2.9941 | +18.4 / +2.9 ms (+0.22%) |
| **`steps_inline7_node20`** | **203.9** | **19.42 (0.817×)** | **19.64 (0.826×)** | **0.1893** | **0.5441 (0.875×)** | **2.9377** | **+29.5 / +32.4 ms (+2.26 / +2.48%)** |
| `steps_compact_8b_slp` | 176.2 | 15.64 (**0.656×**) | 15.58 (**0.663×**) | **0.3849** | 0.6879 (**1.110×**) | 3.0462 | −5.7 / −19.4 ms (−1.49%) |
| `steps_compact_8b_hoist` | 176.2 | 19.07 (0.808×) | 19.26 (0.815×) | 0.3848 | 0.7378 (1.191×) | 3.1065 | −45.0 / −42.9 ms (−3.28%) |

*\*`steps_typed_narrow` saves `−8.4%` of `A6b` (`≈ +18.1 ms` in A6b alone = `1.39%` of 1w wall; minor jitter in unchanged `A7` shifted the `C+E` median delta).*

### 3.3 Dependent-load chain test: 1-way vs 2-way vs 4-way inter-block interleaving (`wikidata ×4`, 5 repeats)

| subset | blocks | steps | 1-way 16B ns/step (`min..max`) | 2-way 16B ns/step (ratio) | 4-way 16B ns/step (ratio) | 1-way 8B ns/step | 2-way 8B ns/step (vs 1w-8B) |
|---|---:|---:|---:|---:|---:|---:|---:|
| **`all_blocks`** | 932 | 10,675,880 | **25.84** (`25.69..26.38`) | 31.71 (**1.227×**) | 42.54 (**1.646×**) | 21.42 | 24.30 (**1.134×**) |
| **`l2_fit_<=120KiB`** | 44 | 239,900 | **24.81** (`24.11..25.58`) | 31.45 (**1.268×**) | 41.79 (**1.684×**) | 20.63 | 23.76 (**1.152×**) |
| **`l2_near_>200KiB`** | 732 | 9,017,760 | **26.12** (`25.85..26.25`) | 31.66 (**1.212×**) | 42.94 (**1.644×**) | 21.94 | 24.45 (**1.114×**) |

*(Exact same pattern on `wikidata` 7 repeats: 2-way `1.231×`, 4-way `1.621×`, L2-fit 2-way `1.239×`.)*

**Why the ~24 ns/step is NOT a dependent-load chain (and why Idea 3 STOPs):**
1. **Half of step time (`50.3%–51.3%`) is `join()`**, not pointer chasing (`ablat_no_join`: `23.76 → 11.67 ns/step`).
2. **L2 footprint spills do not dominate:** despite 27% of blocks exceeding 256 KiB L2, 1-way `ns/step` differs by only `5.3%` between `<= 120 KiB` (`24.81`) and `> 200 KiB` (`26.12`), because dead rules and eliminated nodes are never re-read during induction.
3. **Prefetch, hoisting, and interleaving ALL regress:** safe speculative touch (`steps_typed_prefetch`) costs `+14%` vs `steps_typed_narrow`, load hoisting (`steps_hoist_next`) costs `+22%`, and 2-way / 4-way inter-block interleaving costs `+23% / +65%` — **even on `l2_fit_<=120KiB` blocks where 2-way fits inside the 256 KiB L2!** The loop is ALU + branch-mispredict + L1/L2 throughput bound, not stalled on DRAM/L3 load latency.
4. **Layout compaction illustrates "pricing with what it produces":**
   - `steps_compact_8b_slp` (8 B rules, `176.2 KiB/blk`) cuts `A6b-steps` by **−33.7%** (`15.58 ns/step`), but forces `A6b-dict` to DFS-expand short rules, nearly doubling `A6b-dict` (`0.203 → 0.385 ns/row`) for a net **1.110× A6b regression** (`−19.4 ms`).
   - `steps_inline7_node20` (20-byte `[u32; 5]` node storing words `≤ 7 B` inline directly in the node, avoiding 60% of rule allocations and shrinking arena `231.4 → 203.9 KiB/blk`) is a genuine algorithmic speedup: **−17.4%..−18.4% on `A6b-steps`**, **−6.7% on `A6b-dict`**, and **−12.5%..−13.3% on `A6b`**, saving **`+29.5..+32.4 ms` (`+2.26%..+2.48%` of 1w wall)**.
   - **Under C2 (never relax a frozen gate after data), Idea 3 is STOP** (`+2.26%..+2.48% < 3.0%`). Because all of `A6b-steps` is only `110.4 ms` (`8.45%` of 1w wall), clearing 3.0% (`39.18 ms`) from induction steps alone would require cutting steps by **>35.5%** with zero dictionary regression — not reachable by any tested mechanism.

---

## 4. Summary of actions taken

- **Opened `build` claim (at `status = "idea"`):** [`frontier/claims/preshifted-inline-back.toml`](../../../../../frontier/claims/preshifted-inline-back.toml) for pre-shifted `inline_back` in `Dictionary` (+5.85–6.46% 1w wall on `C+E`, noting supersession by `forward-emission`).
- **Appended to `frontier/dead-ends.tsv`:**
  1. `rauzy-output LIMIT sweep & adaptive rules` (`STOP`)
  2. `rauzy-output SLP-during-emission` (`KILL`)
  3. `rauzy-output induction-step prefetch, interleaving & compaction` (`STOP`)
- **Updated `frontier/surface.tsv`:** rows `C-A6b` and `E-spelling`.
- **Left delta:** `frontier/deltas/2026-11-26-rauzy-output.md` and marked `rauzy-output` `done`.
