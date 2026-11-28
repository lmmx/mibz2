# i9-10900K Clock-Modes Probe Verdict (`i9-clock-modes`): `PASS (mechanism found)`

**Date:** 2026-11-28
**Claim:** `i9-clock-modes` (`frontier/claims/i9-clock-modes.toml`, recipe PR #1201)
**Status:** i9 bundle read and verified; `0.5.15` restatement in `CHANGELOG.md` and `docs/journal/release-attribution.md` plus `docs/RELEASING.md` §5 note routed to a non-`antigravity` participant per frozen `stop_test` provenance rule (`antigravity` authored `v0.5.15`).

---

## 1. Provenance and Integrity Verification

- **Evidence commit:** `origin/evidence` `d5658cfbc8837c8ae832be85263d34b34b369d10`
- **Archive:** `clock-modes/run-20261128T013907Z.tar.gz` (`90,638 B`)
- **SHA-256 (verified against `.sha256` sidecar):** `9a26b90dd4e7416f19c1d1d636d73d144bac2bdfa3d587b9cd694eb41a3da083`
- **Code pinned at:** `head = 91f4e61fb38d8d3a5e485a303b5d6cf7a33912eb` (`src` tree `272e1f24`, `examples/ab.rs` `a55e92bf`, `rustc 1.99.1`)
- **Design:** `rounds = 12`, `threads = [1, 6]`, `pinnings = [pinned, unpinned]`, `settles = [0.0, 0.2, 2.0]` s, `prespin_ms = [0]`, `seed = 1189`, `2 × 3 × 2 × 12 = 144` runs per thread count (`288 / 288` total, `288 / 288` perf-covered) on `wikidata x4` (`838,800,784 B` decoded).
- **Independent re-score:** `python3 bench/clock_modes.py score runs.jsonl` reproduced `outcome.txt` and `summary.txt` byte-for-byte:
  > `PASS (mechanism found): 1T: pinning=pinned & settle_s=0.2 explains 4/5 (80.0%) of slow_clock runs (rate 16.7% vs 0.8%, med cpr_ratio=0.9983); 6T: pinning=pinned & settle_s=0.2 explains 13/13 (100.0%) of slow_clock runs (rate 54.2% vs 0.0%, med cpr_ratio=0.8794)`
- **Zero real-work slow runs (`slow_work = 0 (cpi: 0, ins: 0)`) and zero unclassified runs (`0`) at both `1T` and `6T`.**

---

## 2. Full `2 × 3 × 2` Factorial Tables (`1T` and `6T`)

### 2.1 `6T` (`PASS (mechanism found)`, `fast_wall = 0.2598 s`, slow threshold `1.30×` = `0.3377 s`, `fast_cpb = 6.568`, `fast_ipb = 19.531`, `fast_cpi = 0.3363`, `fast_cpr = 1.7643`)

Mode counts (`144 / 144` perf-measured): `fast = 131`, `slow_clock = 13`, `slow_work = 0 (cpi: 0, ins: 0)`, `slow_mixed = 0`, `unclassified = 0`.

| Pinning | Settle (s) | Pair pos | `n` | `med_wall` (s) | `min..max` (s) | `pre_f0` med (GHz) | `med_cpr` | `med_cpi` | `med_ipb` | `fast / clk / work / mix` |
|---|---:|---:|---:|---:|---|---:|---:|---:|---:|---|
| `pinned` | `0.0` | `0` | 12 | `0.2979` | `0.2956..0.3054` | `4.93` | `1.763` | `0.335` | `19.52` | `12 / 0 / 0 / 0` |
| `pinned` | `0.0` | `1` | 12 | `0.2973` | `0.2940..0.3029` | `4.90` | `1.767` | `0.334` | `19.55` | `12 / 0 / 0 / 0` |
| `pinned` | `0.2` | `0` | 12 | `0.3032` | `0.2967..0.3597` | `0.80` | `1.760` | `0.335` | `19.53` | `10 / 2 / 0 / 0` |
| **`pinned`** | **`0.2`** | **`1`** | **12** | **`0.3513`** | **`0.3326..0.3555`** | **`0.80`** | **`1.552`** | **`0.334`** | **`19.51`** | **`1 / 11 / 0 / 0`** |
| `pinned` | `2.0` | `0` | 12 | `0.2991` | `0.2948..0.3043` | `0.80` | `1.766` | `0.334` | `19.55` | `12 / 0 / 0 / 0` |
| `pinned` | `2.0` | `1` | 12 | `0.2985` | `0.2951..0.3077` | `0.80` | `1.764` | `0.334` | `19.54` | `12 / 0 / 0 / 0` |
| `unpinned` | `0.0` | `0` | 12 | `0.2470` | `0.2457..0.2557` | `5.16` | `1.771` | `0.338` | `19.53` | `12 / 0 / 0 / 0` |
| `unpinned` | `0.0` | `1` | 12 | `0.2461` | `0.2455..0.2526` | `4.90` | `1.771` | `0.338` | `19.52` | `12 / 0 / 0 / 0` |
| `unpinned` | `0.2` | `0` | 12 | `0.2471` | `0.2446..0.2735` | `0.80` | `1.753` | `0.336` | `19.53` | `12 / 0 / 0 / 0` |
| `unpinned` | `0.2` | `1` | 12 | `0.2604` | `0.2455..0.2824` | `0.80` | `1.749` | `0.337` | `19.50` | `12 / 0 / 0 / 0` |
| `unpinned` | `2.0` | `0` | 12 | `0.2475` | `0.2461..0.2516` | `0.80` | `1.761` | `0.337` | `19.53` | `12 / 0 / 0 / 0` |
| `unpinned` | `2.0` | `1` | 12 | `0.2487` | `0.2469..0.2502` | `0.80` | `1.749` | `0.338` | `19.53` | `12 / 0 / 0 / 0` |

- **Exact `6T` mechanism:** `pinning = pinned & settle_s = 0.2` accounts for **13 / 13 (100.0%)** of `slow_clock` runs (`11 / 12` at `pos = 1`, `2 / 12` at `pos = 0`; `0 / 120` in every other cell, including **`0 / 48` at `settle = 2.0 s`** and **`0 / 48` at `settle = 0.0 s`**).
  - Across all 13 `slow_clock` runs: `wall = 0.3412..0.3597 s` (median `0.3515 s`, `1.353×` pooled `fast_wall` and `1.180×` pinned fast `0.298 s`), `cpb_ratio = 0.9859..0.9954` (median `0.9926`), `ipb = 19.47..19.59` (median `19.51`), `cpi = 0.3320..0.3354` (median `0.3339`), while `cpr` drops from `1.764` to **`1.552`** (`1.490..1.600`, median `cpr_ratio = 0.8794`, `-12.1%` achieved frequency).
  - Notice `unpinned, settle = 0.2, pos = 1` exhibits the **same HWP dip in miniature** (`med_wall = 0.2604 s` vs `0.2470 s` = `+5.4%`, with round 2 hitting `wall = 0.2824 s`, `cpr = 1.531`, `cpb_ratio = 0.9962`), which sits below the `1.30×` pooled threshold (`0.3377 s`) only because unpinned `6T` runs `-17%` faster than pinned `6T`. At `settle = 2.0 s`, `unpinned` is tight and unimodal (`0.2461..0.2516 s` at `pos = 0`, `0.2469..0.2502 s` at `pos = 1`).
- **Secondary finding — `6T` pinned vs unpinned oversubscription:** Even in the fast mode (`settle = 0.0` or `2.0`), `pinned` `6T` takes **`0.298 s`** vs `unpinned` `6T` **`0.247 s`** (**`-17.1%` faster unpinned**). The 6-worker parallel decoder spawns `6` workers + `1` scanner + `1` reader = `8` active threads; pinning to 6 physical CPUs (`cpus = 0..5`) forces 8 threads to time-share 6 cores, whereas unpinned spreads all 8 threads across 8 of the i9-10900K's 10 physical cores with identical executed work (`ipb = 19.53`, `cpi = 0.334–0.338`).

### 2.2 `1T` (`PASS (mechanism found)`, `fast_wall = 1.3825 s`, slow threshold `1.04×` = `1.4378 s`, `fast_cpb = 5.824`, `fast_ipb = 17.006`, `fast_cpi = 0.3425`, `fast_cpr = 1.9073`)

Mode counts (`144 / 144` perf-measured): `fast = 113`, `slow_clock = 5`, `slow_work = 0 (cpi: 0, ins: 0)`, `slow_mixed = 26`, `unclassified = 0`.

| Pinning | Settle (s) | Pair pos | `n` | `med_wall` (s) | `min..max` (s) | `med_cpr` | `med_cpi` | `med_ipb` | `fast / clk / work / mix` |
|---|---:|---:|---:|---:|---|---:|---:|---:|---|
| `pinned` | `0.0` | `0` | 12 | `1.4094` | `1.3891..1.4758` | `1.905` | `0.345` | `17.01` | `9 / 0 / 0 / 3` |
| `pinned` | `0.0` | `1` | 12 | `1.3922` | `1.3862..1.5757` | `1.913` | `0.343` | `17.00` | `9 / 0 / 0 / 3` |
| `pinned` | `0.2` | `0` | 12 | `1.4192` | `1.3911..1.6164` | `1.877` | `0.343` | `17.00` | `9 / 1 / 0 / 2` |
| **`pinned`** | **`0.2`** | **`1`** | **12** | **`1.4345`** | **`1.3959..1.5950`** | **`1.883`** | **`0.348`** | **`17.01`** | **`6 / 3 / 0 / 3`** |
| `pinned` | `2.0` | `0` | 12 | `1.4091` | `1.3922..1.5751` | `1.905` | `0.346` | `17.00` | `10 / 0 / 0 / 2` |
| `pinned` | `2.0` | `1` | 12 | `1.4122` | `1.3895..1.4611` | `1.905` | `0.344` | `17.01` | `11 / 0 / 0 / 1` |
| `unpinned` | `0.0` | `0` | 12 | `1.3939` | `1.3618..1.4510` | `1.899` | `0.341` | `17.01` | `11 / 0 / 0 / 1` |
| `unpinned` | `0.0` | `1` | 12 | `1.3952` | `1.3585..1.5782` | `1.908` | `0.342` | `17.00` | `10 / 0 / 0 / 2` |
| `unpinned` | `0.2` | `0` | 12 | `1.4047` | `1.3529..1.5061` | `1.907` | `0.337` | `17.00` | `10 / 0 / 0 / 2` |
| `unpinned` | `0.2` | `1` | 12 | `1.3837` | `1.3418..1.5028` | `1.913` | `0.340` | `17.00` | `9 / 0 / 0 / 3` |
| `unpinned` | `2.0` | `0` | 12 | `1.4009` | `1.3404..1.4976` | `1.912` | `0.344` | `17.01` | `9 / 1 / 0 / 2` |
| `unpinned` | `2.0` | `1` | 12 | `1.3632` | `1.3347..1.4953` | `1.911` | `0.336` | `17.00` | `10 / 0 / 0 / 2` |

- **`1T` `slow_clock` (`n = 5`):** `pinning = pinned & settle_s = 0.2` explains **4 / 5 (80.0%)** (`1` at `pos = 0`, `3` at `pos = 1`, all with `pre_f0 = 800 MHz`, `wall = 1.4419..1.4528 s`, `cpr = 1.858..1.887` vs `1.907` fast, `cpb_ratio = 1.0058..1.0285`); the 5th is one `unpinned, settle = 2.0, pos = 0` run (`rd = 7`, `wall = 1.4764 s`, `cpr = 1.816`, `cpb_ratio = 1.0048`). `pinned, settle = 2.0` had **`0 / 24` `slow_clock` runs**.
- **Dissection of the `26` `slow_mixed` runs at `1T`:**
  - The frozen classifier marks a slow run (`wall >= 1.04 × fast_wall`) as `slow_clock` if `|cpb_ratio - 1| <= 0.03`, `slow_work` if `cpb_ratio >= 1.20`, and `slow_mixed` in the intermediate band (`1.03 < cpb_ratio < 1.20`).
  - Inspecting all 26 `slow_mixed` runs in `runs.jsonl` shows they are **not** a clock drop and **not** extra instructions:
    - **Clock is full turbo:** `cpr = 1.864..1.916`, median **`1.910`** vs `fast_cpr = 1.907` (`cpr_ratio = 1.0013` — zero frequency drop).
    - **Instructions are invariant:** `ipb = 16.96..17.05`, median **`17.001`** vs `fast_ipb = 17.006`.
    - **Pure CPI / memory-latency variance:** `cpi = 0.3543..0.3876`, median **`0.3663`** vs `fast_cpi = 0.3425` (`+6.9%`, `cpb_ratio = 1.0315..1.1313`, `wall = 1.4444..1.6164 s`, `1.045..1.169×`).
    - **Uniform across cells, front-loaded in early rounds:** `14` pinned / `12` unpinned, spanning all settles (`0.0`: `9`, `0.2`: `10`, `2.0`: `7`) and positions (`pos 0`: `12`, `pos 1`: `14`), with `7 / 26` occurring in round `0` (including 5 of the 6 slowest: `1.570–1.595 s`) and `0` in rounds `9–11` (`runq_s <= 0.0002 s` on every row). On `wikidata x4` at `1T`, the `~5–13%` CPI spread reflects physical page-placement / cache-set / DRAM-bank layout variance across fresh `ab` child processes (consistent with zero `slow_mixed` rows at `6T`, where `thr = 1.30×` is well above a `~7%` CPI spread).

---

## 3. Physical Mechanism: Why `--settle 0.2` Triggers HWP Throttling While `0.0` and `2.0` Do Not

Every run recorded `sysfs_pre` (`scaling_cur_freq` on CPU 0 immediately before launch) and `governor = powersave`, `epp = balance_performance`, `no_turbo = 0`:
1. **`--settle 0.0 s`:** CPU 0 is still hot from the preceding burst (`pre_f0` median `4.90–5.16 GHz`), so the next run enters at full turbo (`0 / 48` slow at `6T`).
2. **`--settle 0.2 s`:** `200 ms` is just long enough for Intel HWP to drop idle cores to the `800 MHz` floor (`pre_f0 = 0.80 GHz`), while catching the package inside HWP's short-window hysteresis / ramp-up transition right after the first child of a pair (`pos = 1`). Under `6T pinned` (`pos = 1`), `11 / 12` runs stay at depressed average frequency (`cpr = 1.552`, `-12.1%` effective clock -> `+18%` wall vs pinned fast / `+35%` vs pooled fast). Even `6T unpinned, settle = 0.2, pos = 1` shifts `+5.4%` in median wall (`0.2604` vs `0.2470 s`).
3. **`--settle 2.0 s`:** `pre_f0` also sits at `0.80 GHz`, yet **`0 / 48` `6T` runs and `0 / 24` `1T pinned` runs** enter `slow_clock` (`cpr = 1.766 / 1.764` pinned `6T`, `1.761 / 1.749` unpinned `6T`). Two seconds lets HWP / RAPL short-tau state reset cleanly so the next decode snaps straight back to `4.90 GHz`.

---

## 4. Mode-Robust Protocol and Analysis Path (for `docs/RELEASING.md` §5)

The mode-robust analysis path is already implemented and tested in `bench/clock_modes.py analyze` (`bench/test_clock_modes.py`):
1. **Prevent the HWP hysteresis regime at collection time:** Use **`--settle 2.0`** (never `--settle 0.2`, which triggers the `200 ms` HWP ramp-down trap; `#1240` already verified `0.2 s` vs `2.0 s` across the historical evidence). Avoid pinning `N`-worker runs (`N + 2` threads including scanner + reader) to only `N` CPUs when measuring unpinned production throughput.
2. **Report mode-robust figures beside pooled ratios (`python3 bench/clock_modes.py analyze <runs.jsonl>`):**
   - When `perf` counters (`cycles`, `ref_cycles`, `instructions`) are present, classify modes by `cycles_per_byte` (separating clock drops from real executed work) and report `within-fast`, `within-slow`, `same-mode paired`, and `position-stratified` ratios alongside the pooled paired log-ratio.
   - On legacy bundles without `perf` counters, `clock_modes.py analyze` falls back to gap-detected `wall_only` mode splitting and flags unequal mode occupancy between arms (`arm_mode_counts` vs `ref_mode_counts`).

---

## 5. Verified `0.5.15` Release-Scaling Restatement Figures (for Independent Agent)

Per `i9-clock-modes.toml`'s frozen `stop_test`, the restatement of the `0.5.15` `1T` and `12T` rows in `CHANGELOG.md` and `docs/journal/release-attribution.md` **must be performed by an agent other than `v0.5.15`'s author (`antigravity`)**. The exact figures below were independently computed from `origin/evidence` `bce3914d` (`release-scaling/run-20261127T140323Z.tar.gz`, SHA-256 verified `f7ef4216ee04adf970d8fd416c0f11e361f620d60f4078f23cc1236fdc536fb0`) via `python3 bench/clock_modes.py analyze runs.jsonl --threads 1,12 --json`:

| Thread count | Metric (`0.5.14 / 0.5.15` wall) | Pooled paired (recorded) | Mode occupancy (`wall_only`) | Within-fast ratio `[95% CI]` (med) | Within-slow ratio `[95% CI]` (med) | Same-mode paired `[95% CI]` (`n`) | Position-stratified combined |
|---:|---|---|---|---|---|---|---|
| **`1T`** (pinned) | `0.5.14 / 0.5.15` | **`1.0567 [1.0115, 1.1039]`** (`+5.7%`, `n = 10`) | `0.5.14`: `2` fast / `8` slow; `0.5.15`: `7` fast / `3` slow (`split = 0.3733 s`) | **`1.0175 [0.9721, 1.0651]`** (med `1.0212`, `n = 2/7`) | **`1.0019 [0.9899, 1.0141]`** (med `0.9984`, `n = 8/3`) | **`1.0159 [0.9905, 1.0419]`** (`n = 3`) | `1.0543` |
| **`12T`** (unpinned) | `0.5.14 / 0.5.15` | **`1.0950 [1.0303, 1.1638]`** (`+9.5%`, `n = 10`) | `0.5.14`: `7` fast / `3` slow (`0.074–0.078 s`); `0.5.15`: `10` fast / `0` slow (`split = 0.0696 s`) | **`1.0430 [1.0033, 1.0841]`** (med `1.0809`, `n = 7/10`)* | `—` (`n = 3/0`) | **`1.0458 [1.0135, 1.0790]`** (`n = 7`) | `1.0942` |

*\*Note on `12T`: `clock_modes.py analyze`'s two-mode split (`0.0696 s`) separates the 3 slowest `0.5.14` runs (`0.074–0.078 s`), reducing the ratio from `1.095` to `1.043–1.046`; as noted in `docs/notebook/2026-11-27-synthesis-fifteenth-pass.md`, the `< 0.0696 s` cluster itself contains two sub-modes (`~0.060 s` and `~0.065 s`), within which `0.5.14 / 0.5.15` is `~1.006–1.02`. At `1T`, the frequency-free stage-cycles-per-byte check (`-1.1% [-2.1%, -0.0%]`) agrees with the within-mode `~0–2%` estimate rather than the pooled `+5.7%`.*

---

## 6. Next Steps (Non-`antigravity` Handoff)

1. A participant other than `antigravity` (e.g., `claude-code` or `chatgpt`) updates the `0.5.15` `1T` and `12T` rows in `CHANGELOG.md` and `docs/journal/release-attribution.md` with the within-mode figures beside the pooled ones above.
2. Add the concise `--settle 2.0` + `bench/clock_modes.py analyze` mode-robust note to `docs/RELEASING.md` §5.
3. Mark `frontier/claims/i9-clock-modes.toml` `status = "done"` with a closing delta.
