# Peak-RSS noise bands per corpus × threads × runner (calibration census)

*28 Nov 2026, master `3715f0e5`. Claim `rss-bound-calibration` (smooth zone, no new benchmark runs). Every figure below is computed directly from archived strict-null CI `bench.yml` artifacts and `origin/evidence` physical i9-10900K bundles. Under C2 this census changes no frozen verdict: all past STOPs (`#1179`, `#1119`, `#1053`, `#1238`) stand.*

## 1. Executive summary

- **Tight point RSS bounds (`<= 1.005–1.020x`) across all 21 CI cells fail by pure chance.** Across **27 pure zero-`src/` strict-null CI `bench.yml` runs** (10 paired rounds × 21 cells = 5,670 paired observations across 6 runner CPU models), **27/27 (100.0%)** identical-binary runs have at least one `2T`/`4T` cell with paired GM `> 1.005x`, **25/27 (92.6%)** exceed `1.010x`, **17/27 (63.0%)** exceed `1.020x`, **11/27 (40.7%)** exceed `1.030x`, and **3/27 (11.1%)** exceed `1.050x` (observed max `1.0683x` on `wikidata 4T`). **0/27 (0.0%)** exceed `PERFORMANCE-POLICY.md` §2 rule 4's `1.100x` line.
- **Two sharp physical regimes on CI:**
  1. **Deterministic regime** (all 7 corpora at `1T`, plus `onebyte 2T/4T` and `runs 2T/4T` — 11 cells): `run_sd = 0.0013–0.0023`, 95% single-run prediction band `[0.9947, 1.0043]`, observed max across all 297 cell-runs `1.0047x`, **0/27** exceed `1.005x`.
  2. **Multi-threaded queue/arena jitter regime** (`2T/4T` on `wikidata`, `text`, `random`, `records`, `pattern` — 10 cells): `run_sd = 0.0094–0.0259` (**7–20× wider**), `pair_sd = 0.0301–0.0910`, 95% single-run upper prediction edge `1.0189x–1.0585x`.
- **Uncorrected 95% t-intervals over 21 cells also false-positive routinely:** **10/27 (37.0%)** strict-null CI runs flag at least one cell `**more**` (`CI_lo > 1.0`), and **13/27 (48.1%)** flag `CI != 1.0`.
- **Physical i9-10900K shows the exact same split, cleanly separating noise from real per-worker working-set growth:**
  - Zero-heap `.text` alignment (`77236472`, `#1119`) and zero worker-heap scanner bundles (`9de7b6e0`, `9dacc085`) have `k = 1` `pair_sd = 0.0000–0.0070`, rising to `0.0061–0.0265` on `wikidata`/`text` (`k = 4..10`) and `0.0120–0.0515` on `random`/`records` (`k = 6..10`, paired GM up to `1.0367x`, single-pair max `1.1319x`).
  - Real per-worker heap additions (`#1053` `+524 KiB/w`, `#1168` `+0.89 MiB/w`) fire unambiguously at **`k = 1`** (`+0.58 MiB / 1.0077x, t = +17.4` and `+0.89 MiB / 1.0120x, t = +24.8`) and scale linearly as `k × Δ_w` to `+4.10..+6.38 MiB` at `k = 8..10`.

---

## 2. CI strict-null peak-RSS census (27 runs, 5,670 paired observations)

### 2.1 Inclusion criteria and data sources

All 27 completed PR `bench.yml` runs on 27–28 Nov 2026 whose PR diff touched **zero** `src/`, `Cargo.*`, `examples/`, `bench/`, or `.github/workflows/bench.yml` files (docs/frontier-only PRs where `head` and `base` compile identical Rust code; `#1245` was excluded because it modified `bench.yml`). Each run executes 10 alternating paired rounds (`bench 3.9.0`, `ab run`) across 7 corpora × 3 thread counts (`1T`, `2T`, `4T`).

<details>
<summary>All 27 strict-null CI bench runs (run ID, UTC date, runner CPU, base..head, max 1T and 2T/4T paired GM)</summary>

| Run ID | Date | Runner CPU | `base..head` | Max 1T GM | Max 2T/4T GM (cell) |
|---|---|---|---|---:|---:|
| `45965544459` | `2026-11-27` | AMD EPYC 9V74 | `3decbce8`..`521cc3ce` | `1.0024x` | `1.0190x` (`pattern 4T`) |
| `45967204538` | `2026-11-27` | Intel Xeon Platinum 8370C | `2fed5f18`..`e8a7cb85` | `1.0022x` | `1.0131x` (`pattern 2T`) |
| `46008878004` | `2026-11-27` | AMD EPYC 9V74 | `dd935418`..`2620059a` | `1.0016x` | `1.0456x` (`pattern 2T`) |
| `46011508097` | `2026-11-27` | AMD EPYC 9V74 | `aaaee363`..`141fbd31` | `1.0034x` | `1.0147x` (`text 4T`) |
| `46012221475` | `2026-11-27` | AMD EPYC 7763 | `48061f2c`..`41c3ab3a` | `1.0028x` | `1.0279x` (`pattern 4T`) |
| `46015631473` | `2026-11-27` | AMD EPYC 9V45 | `8393e19d`..`1f614a53` | `1.0026x` | `1.0299x` (`wikidata 2T`) |
| `46035607186` | `2026-11-27` | AMD EPYC 9V74 | `af7ff482`..`c764224e` | `1.0020x` | `1.0232x` (`wikidata 2T`) |
| `46037480024` | `2026-11-27` | AMD EPYC 9V74 | `4e261140`..`d8a9cfbb` | `1.0016x` | `1.0683x` (`wikidata 4T`) |
| `46038471077` | `2026-11-27` | Intel Xeon 6975P-C | `b4005e84`..`556b8afa` | `1.0027x` | `1.0435x` (`wikidata 4T`) |
| `46041577974` | `2026-11-27` | Intel Xeon 6975P-C | `410c46f3`..`0aa317fc` | `1.0034x` | `1.0433x` (`text 2T`) |
| `46042589891` | `2026-11-27` | AMD EPYC 9V74 | `410c46f3`..`4c75d223` | `1.0016x` | `1.0179x` (`records 2T`) |
| `46045498908` | `2026-11-27` | AMD EPYC 9V74 | `b4005e84`..`e5f71f03` | `1.0022x` | `1.0471x` (`pattern 4T`) |
| `46045588156` | `2026-11-27` | AMD EPYC 7763 | `c5c34377`..`043fbf6f` | `1.0017x` | `1.0437x` (`random 4T`) |
| `46051416187` | `2026-11-27` | AMD EPYC 9V74 | `d7b00163`..`788348a5` | `1.0022x` | `1.0245x` (`records 2T`) |
| `46053807880` | `2026-11-27` | AMD EPYC 9V45 | `926ff31b`..`254d6dc7` | `1.0039x` | `1.0344x` (`pattern 4T`) |
| `46070942975` | `2026-11-27` | AMD EPYC 9V74 | `01acd74d`..`3b74074e` | `1.0019x` | `1.0431x` (`pattern 4T`) |
| `46071143084` | `2026-11-27` | Intel Xeon 6973P-C | `5dd73589`..`b28668d8` | `1.0020x` | `1.0312x` (`random 2T`) |
| `46074000286` | `2026-11-27` | AMD EPYC 9V74 | `2af67a67`..`e1f4e9e6` | `1.0017x` | `1.0120x` (`random 4T`) |
| `46074543990` | `2026-11-27` | AMD EPYC 9V74 | `2af67a67`..`9699b29b` | `1.0015x` | `1.0521x` (`pattern 4T`) |
| `46077229245` | `2026-11-27` | AMD EPYC 7763 | `022e94dc`..`1e2883a5` | `1.0017x` | `1.0235x` (`text 2T`) |
| `46078057930` | `2026-11-27` | AMD EPYC 9V74 | `ac708c52`..`d7544ed1` | `1.0015x` | `1.0182x` (`text 2T`) |
| `46080204367` | `2026-11-27` | AMD EPYC 7763 | `faa976e5`..`70ce1e48` | `1.0023x` | `1.0186x` (`random 2T`) |
| `46081315035` | `2026-11-27` | AMD EPYC 9V74 | `32b0c9ec`..`1050bb96` | `1.0018x` | `1.0516x` (`random 2T`) |
| `46082793404` | `2026-11-27` | AMD EPYC 7763 | `dd244f51`..`972ce23c` | `1.0011x` | `1.0086x` (`records 2T`) |
| `46086907222` | `2026-11-28` | AMD EPYC 7763 | `91ab4c8d`..`c0599e42` | `1.0017x` | `1.0193x` (`records 2T`) |
| `46087294867` | `2026-11-28` | AMD EPYC 9V74 | `9de1d9f0`..`653048a0` | `1.0015x` | `1.0297x` (`records 2T`) |
| `46089635803` | `2026-11-28` | Intel Xeon Platinum 8370C | `e435074c`..`84463b8f` | `1.0011x` | `1.0069x` (`pattern 2T`) |

</details>

### 2.2 Per-cell null distribution (21 cells, `n = 27` runs, 10 rounds/run)

`run GM` and `run SD` are computed on log-transformed per-run paired geometric means (`n = 27`). The **95% single-run prediction band** is `exp(mean ± t_{0.975,26} · run_SD · sqrt(1 + 1/27))` (`t_{0.975,26} = 2.0555`). `pair SD` is the log-SD across all 270 individual paired rounds; `r0 GM` vs `r1..r9 GM` splits round 0 (head first, no warm-up) from steady-state rounds `1..9`.

| Corpus | T | Base MB | Run GM | Run SD | 95% pred. band | Observed `[min, max]` | Pair SD | `r0` GM | `r1..r9` GM | `>0.5%` | `>1%` | `>2%` | `>3%` | `CI > 1` |
|---|---:|---:|---:|---:|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `onebyte` | 1 | 33.5 | 1.0003 | 0.0013 | `[0.9976, 1.0030]` | `[0.9982, 1.0034]` | 0.0041 | 1.0019 | 1.0001 | 0 | 0 | 0 | 0 | 2 |
| `pattern` | 1 | 16.2 | 0.9995 | 0.0023 | `[0.9947, 1.0043]` | `[0.9948, 1.0039]` | 0.0075 | 1.0001 | 0.9994 | 0 | 0 | 0 | 0 | 0 |
| `random` | 1 | 23.0 | 1.0003 | 0.0016 | `[0.9970, 1.0036]` | `[0.9964, 1.0028]` | 0.0054 | 0.9999 | 1.0004 | 0 | 0 | 0 | 0 | 1 |
| `records` | 1 | 23.1 | 1.0002 | 0.0013 | `[0.9974, 1.0030]` | `[0.9968, 1.0024]` | 0.0050 | 1.0000 | 1.0002 | 0 | 0 | 0 | 0 | 0 |
| `runs` | 1 | 23.2 | 0.9997 | 0.0015 | `[0.9967, 1.0028]` | `[0.9966, 1.0017]` | 0.0053 | 0.9988 | 0.9998 | 0 | 0 | 0 | 0 | 0 |
| `text` | 1 | 19.5 | 1.0001 | 0.0017 | `[0.9966, 1.0036]` | `[0.9969, 1.0034]` | 0.0067 | 1.0013 | 0.9999 | 0 | 0 | 0 | 0 | 0 |
| `wikidata` | 1 | 28.1 | 0.9997 | 0.0016 | `[0.9963, 1.0030]` | `[0.9965, 1.0029]` | 0.0045 | 0.9989 | 0.9997 | 0 | 0 | 0 | 0 | 0 |
| `onebyte` | 2 | 29.7 | 0.9999 | 0.0014 | `[0.9969, 1.0028]` | `[0.9973, 1.0025]` | 0.0040 | 0.9999 | 0.9999 | 0 | 0 | 0 | 0 | 1 |
| `runs` | 2 | 20.1 | 1.0002 | 0.0013 | `[0.9974, 1.0030]` | `[0.9972, 1.0023]` | 0.0050 | 1.0004 | 1.0001 | 0 | 0 | 0 | 0 | 0 |
| `onebyte` | 4 | 29.6 | 1.0003 | 0.0013 | `[0.9977, 1.0030]` | `[0.9984, 1.0027]` | 0.0046 | 1.0010 | 1.0002 | 0 | 0 | 0 | 0 | 0 |
| `runs` | 4 | 20.1 | 1.0004 | 0.0018 | `[0.9967, 1.0041]` | `[0.9967, 1.0047]` | 0.0067 | 1.0009 | 1.0003 | 0 | 0 | 0 | 0 | 0 |
| `records` | 4 | 54.2 | 0.9990 | 0.0094 | `[0.9796, 1.0189]` | `[0.9772, 1.0160]` | 0.0301 | 0.9990 | 0.9990 | 7 | 3 | 0 | 0 | 1 |
| `text` | 4 | 54.6 | 0.9980 | 0.0105 | `[0.9764, 1.0201]` | `[0.9752, 1.0179]` | 0.0470 | 1.0012 | 0.9977 | 8 | 3 | 0 | 0 | 0 |
| `random` | 4 | 49.1 | 1.0012 | 0.0145 | `[0.9711, 1.0321]` | `[0.9753, 1.0437]` | 0.0358 | 1.0100 | 1.0002 | 9 | 6 | 2 | 1 | 2 |
| `text` | 2 | 34.3 | 1.0042 | 0.0147 | `[0.9738, 1.0355]` | `[0.9762, 1.0433]` | 0.0507 | 0.9849 | 1.0063 | 13 | 10 | 2 | 1 | 0 |
| `pattern` | 2 | 21.1 | 0.9985 | 0.0163 | `[0.9650, 1.0331]` | `[0.9700, 1.0456]` | 0.0507 | 1.0019 | 0.9981 | 10 | 6 | 1 | 1 | 1 |
| `random` | 2 | 38.4 | 1.0068 | 0.0170 | `[0.9716, 1.0431]` | `[0.9754, 1.0516]` | 0.0538 | 1.0033 | 1.0072 | 15 | 11 | 4 | 2 | 2 |
| `wikidata` | 2 | 54.5 | 0.9986 | 0.0205 | `[0.9567, 1.0425]` | `[0.9546, 1.0369]` | 0.0649 | 0.9987 | 0.9986 | 11 | 7 | 5 | 1 | 1 |
| `records` | 2 | 41.6 | 0.9964 | 0.0212 | `[0.9533, 1.0415]` | `[0.9444, 1.0297]` | 0.0774 | 0.9933 | 0.9968 | 11 | 9 | 3 | 0 | 0 |
| `wikidata` | 4 | 84.3 | 0.9955 | 0.0243 | `[0.9461, 1.0475]` | `[0.9551, 1.0683]` | 0.0910 | 0.9938 | 0.9957 | 8 | 7 | 2 | 2 | 1 |
| `pattern` | 4 | 37.2 | 1.0026 | 0.0259 | `[0.9497, 1.0585]` | `[0.9523, 1.0521]` | 0.0865 | 0.9847 | 1.0046 | 11 | 10 | 7 | 5 | 0 |

### 2.3 Family-wise false-positive rate (FWER) across 21 cells per CI run

| Point-bound rule (`max_cell > threshold`) | `1T` cells only (7 cells) | `2T`/`4T` cells (14 cells) |
|---|---:|---:|
| `> 1.005x` (`+0.5%`) | `0 / 27` (0.0%) | **`27 / 27` (100.0%)** |
| `> 1.010x` (`+1.0%`) | `0 / 27` (0.0%) | **`25 / 27` (92.6%)** |
| `> 1.015x` (`+1.5%`) | `0 / 27` (0.0%) | **`22 / 27` (81.5%)** |
| `> 1.020x` (`+2.0%`) | `0 / 27` (0.0%) | **`17 / 27` (63.0%)** |
| `> 1.030x` (`+3.0%`) | `0 / 27` (0.0%) | **`11 / 27` (40.7%)** |
| `> 1.040x` (`+4.0%`) | `0 / 27` (0.0%) | **`9 / 27` (33.3%)** |
| `> 1.050x` (`+5.0%`) | `0 / 27` (0.0%) | **`3 / 27` (11.1%)** |
| `> 1.100x` (`+10.0%`, `PERFORMANCE-POLICY.md` §2 rule 4) | `0 / 27` (0.0%) | **`0 / 27` (0.0%)** |
| Uncorrected `CI_lo > 1.0` (`**more**`, any of 21 cells) | — | **`10 / 27` (37.0%)** |
| Uncorrected `CI != 1.0` (`**more**` or `**less**`, any of 21 cells) | — | **`13 / 27` (48.1%)** |

### 2.4 Bimodal `pattern 4T` regime and round-0 check

Across the 540 individual `pattern 4T` runs (`27 × 10 × 2` arms), peak RSS splits into two modes separated by a valley near `39.0 MiB`: a primary low mode at `35–37 MiB` (278/540 = 51.5%) and a secondary high mode at `42–44 MiB` (56/540, `+6–8 MiB` of queued output blocks when the writer falls behind). Stratifying the 270 paired rounds at `39.0 MiB` (`39,936 KiB`):

| `pattern 4T` pair stratum | Pairs (`n`) | Paired GM | 95% CI of mean | `pair_sd` |
|---|---:|---:|---|---:|
| All pairs (`r0..r9`) | 270 | `1.0026` | `[0.9923, 1.0130]` | `0.0865` |
| Both low (`< 39.0 MiB`) | 160 | `0.9980` | `[0.9924, 1.0036]` | `0.0364` |
| Both high (`>= 39.0 MiB`) | 15 | `0.9971` | `[0.9667, 1.0285]` | `0.0612` |
| Head high / Base low (`>= 39` vs `< 39`) | 51 | `1.1390` | `[1.1252, 1.1530]` | `0.0445` |
| Head low / Base high (`< 39` vs `>= 39`) | 44 | `0.8812` | `[0.8698, 0.8927]` | `0.0438` |
| Round 0 only (`r0`) | 27 | `0.9847` | `[0.9556, 1.0146]` | `0.0794` |
| Steady-state (`r1..r9`) | 243 | `1.0046` | `[0.9937, 1.0157]` | `0.0872` |

Note on round 0: the cross-mode swings are nearly symmetric (`1.1390 × 0.8812 = 1.0037`). In the earlier 11-run pool (`frontier/deltas/2026-11-27-ci-pattern-4t-arm-bias.md`, `docs/journal/2026-11-27-explore/ci-pattern-4t-arm-bias.md`), `pattern 4T` round 0 averaged `1.054x` (`1.0594` across those 11 runs), whereas in this larger 27-run zero-`src/` pool `r0` averaged `0.9847 [0.9556, 1.0146]` — confirming that `pattern 4T`'s `~9%` single-pair SD can swing a single round's mean by `±5–6%` in either direction depending on how many runs happen to land in the `head_hi/base_lo` vs `head_lo/base_hi` stratum. Across the other 20 cells, `r0` and `r1..r9` agree within sampling noise (largest `r0` deviation is `text 2T` at `0.9849` vs `1.0063`).

### 2.5 Runner CPU breakdown (all 6 GitHub Actions CPU models)

Multi-threaded RSS jitter appears across **every** CI runner CPU family (AMD Zen 3/4/5 and Intel Ice Lake / Granite Rapids):

| Runner CPU | Runs (`n`) | `1T` max range | `2T/4T` max range | `MT > 1.005x` | `MT > 1.020x` | `MT > 1.030x` | Any `CI > 1` |
|---|---:|---|---|---:|---:|---:|---:|
| AMD EPYC 9V74 (Zen 4) | 14 | `[1.0015, 1.0034]` | `[1.0120, 1.0683]` | `14 / 14` | `9 / 14` | `6 / 14` | `7 / 14` |
| AMD EPYC 7763 (Zen 3) | 6 | `[1.0011, 1.0028]` | `[1.0086, 1.0437]` | `6 / 6` | `3 / 6` | `1 / 6` | `1 / 6` |
| AMD EPYC 9V45 (Zen 5) | 2 | `[1.0026, 1.0039]` | `[1.0299, 1.0344]` | `2 / 2` | `2 / 2` | `1 / 2` | `1 / 2` |
| Intel Xeon Platinum 8370C | 2 | `[1.0011, 1.0022]` | `[1.0069, 1.0131]` | `2 / 2` | `0 / 2` | `0 / 2` | `0 / 2` |
| Intel Xeon 6975P-C | 2 | `[1.0027, 1.0034]` | `[1.0433, 1.0435]` | `2 / 2` | `2 / 2` | `2 / 2` | `1 / 2` |
| Intel Xeon 6973P-C | 1 | `[1.0020, 1.0020]` | `[1.0312, 1.0312]` | `1 / 1` | `1 / 1` | `1 / 1` | `0 / 1` |

---

## 3. Physical i9-10900K census (`origin/evidence` archived bundles)

### 3.1 Zero-heap and zero-worker-heap bundles (null & unchanged-guard baseline)

Three archived i9-10900K bundles carry zero per-worker heap changes:
1. **`77236472`** (`jcc-aligned-release/run-20261127T135104Z.tar.gz`, `#1119`, `n = 12` rounds): pure `.text` compiler flag (`-C llvm-args=-x86-align-branch-boundary=32`), zero heap or struct changes anywhere.
2. **`9de7b6e0`** (`scanner-coalesced/run-20261127T082548Z.tar.gz`, `#1034`, `n = 10` rounds): scanner-only change, zero worker-heap changes.
3. **`9dacc085`** (`scanner-a2-fast/run-20261127T233936Z.tar.gz`, `#1117`, `n = 10` rounds): scanner-only change, zero worker-heap changes.

| Bundle (commit) | Corpus | `k` | `n` | Base med (MiB) | Head med (MiB) | `medR` | Paired GM `[95% CI]` | `pair_sd` | Max pair |
|---|---|---:|---:|---:|---:|---:|---|---:|---:|
| `#1119` (`77236472`) | `wikidata` | 1 | 12 | 29.92 | 29.92 | `0.9999` | `0.9994 [0.9968, 1.0021]` | `0.0041` | `1.0051` |
| `#1119` (`77236472`) | `wikidata` | 4 | 12 | 48.05 | 48.15 | `1.0022` | `1.0016 [0.9900, 1.0132]` | `0.0182` | `1.0281` |
| `#1119` (`77236472`) | `wikidata` | 6 | 12 | 55.13 | 54.88 | `0.9956` | `0.9984 [0.9871, 1.0098]` | `0.0179` | `1.0377` |
| `#1119` (`77236472`) | `wikidata` | 8 | 12 | 61.26 | 62.65 | **`1.0227`** | `1.0089 [0.9939, 1.0241]` | `0.0235` | `1.0360` |
| `#1119` (`77236472`) | `wikidata` | 10 | 12 | 61.01 | 61.07 | `1.0010` | `0.9972 [0.9824, 1.0123]` | `0.0236` | `1.0388` |
| `#1119` (`77236472`) | `text` | 1 | 12 | 19.22 | 19.14 | `0.9959` | `0.9959 [0.9925, 0.9994]` | `0.0055` | `1.0082` |
| `#1119` (`77236472`) | `text` | 8 | 12 | 80.39 | 80.47 | `1.0010` | `0.9950 [0.9784, 1.0119]` | `0.0265` | `1.0361` |
| `#1119` (`77236472`) | `text` | 10 | 12 | 80.55 | 80.74 | `1.0024` | `1.0091 [0.9977, 1.0207]` | `0.0179` | `1.0376` |
| `#1119` (`77236472`) | `records` | 1 | 12 | 22.66 | 22.66 | `1.0000` | `0.9988 [0.9957, 1.0020]` | `0.0050` | `1.0069` |
| `#1119` (`77236472`) | `records` | 8 | 12 | 71.73 | 71.73 | `1.0000` | `1.0009 [0.9903, 1.0117]` | `0.0168` | `1.0457` |
| `#1119` (`77236472`) | `records` | 10 | 12 | 71.90 | 71.61 | `0.9960` | `0.9870 [0.9680, 1.0065]` | `0.0307` | `1.0047` |
| `#1119` (`77236472`) | `random` | 1 | 12 | 22.50 | 22.50 | `1.0000` | `0.9983 [0.9955, 1.0010]` | `0.0043` | `1.0070` |
| `#1119` (`77236472`) | `random` | 8 | 12 | 69.05 | 70.61 | `1.0226` | `1.0206 [0.9982, 1.0436]` | `0.0351` | `1.1269` |
| `#1119` (`77236472`) | `random` | 10 | 12 | 69.12 | 70.71 | `1.0230` | `1.0229 [0.9977, 1.0488]` | `0.0393` | `1.1319` |
| `#1034` (`9de7b6e0`) | `wikidata` | 1 | 10 | 78.89 | 78.89 | `1.0000` | `0.9999 [0.9992, 1.0006]` | `0.0010` | `1.0025` |
| `#1034` (`9de7b6e0`) | `wikidata` | 8 | 10 | 102.99 | 104.22 | `1.0120` | `1.0129 [1.0037, 1.0223]` | `0.0129` | `1.0414` |
| `#1034` (`9de7b6e0`) | `wikidata` | 10 | 10 | 111.21 | 112.57 | `1.0122` | `1.0134 [1.0090, 1.0179]` | `0.0061` | `1.0230` |
| `#1034` (`9de7b6e0`) | `text` | 1 / 8 / 10 | 10 | 23.6 / 80.4 / 95.4 | 23.6 / 81.4 / 94.4 | — | `1.0000` / `1.0118` / `0.9974` | `0.0000` / `0.0198` / `0.0163` | `1.0360` |
| `#1034` (`9de7b6e0`) | `random` | 6 / 8 / 10 | 10 | 60.1 / 72.6 / 78.2 | 62.6 / 75.1 / 80.2 | — | `1.0367` / `1.0233` / `1.0349` | `0.0120` / `0.0121` / `0.0515` | `1.1213` |
| `#1034` (`9de7b6e0`) | `records` | 6 / 8 / 10 | 10 | 72.0 / 79.6 / 91.8 | 72.0 / 79.6 / 91.8 | — | `0.9875` / `0.9907` / `0.9936` | `0.0430` / `0.0266` / `0.0396` | `1.0712` |
| `#1034` (`9de7b6e0`) | `pattern` | 6 / 8 / 10 | 10 | 49.2 / 63.5 / 75.9 | 49.2 / 63.6 / 77.0 | — | `0.9990` / `1.0001` / `1.0084` | `0.0275` / `0.0206` / `0.0223` | `1.0465` |
| `#1117` (`9dacc085`) | `wikidata` | 1 | 10 | 78.89 | 78.95 | `1.0007` | `1.0006 [1.0004, 1.0008]` | `0.0003` | `1.0013` |
| `#1117` (`9dacc085`) | `wikidata` | 8 / 9 / 10 | 10 | 103.9 / 110.7 / 113.1 | 104.4 / 111.4 / 113.8 | — | `1.0067` / `1.0103` / `1.0061` | `0.0112` / `0.0165` / `0.0091` | `1.0430` |
| `#1117` (`9dacc085`) | `text` / `records` / `random` | 8 | 10 | 79.2 / 79.2 / 74.3 | 79.5 / 79.6 / 74.4 | — | `1.0056` / `1.0030` / `1.0167` | `0.0089` / `0.0074` / `0.0316` | `1.0925` |

Note also that `d6bc8ca2` (`scanner-adaptive-seam`, `n = 16`, real seam-window reduction) shows the same multi-thread `pair_sd` (`0.0191–0.0283` on `wikidata-x4` `k = 6..12`) around a real `-4.2..-5.8 MiB` (`0.904–0.933x`) reduction.

### 3.2 Real per-worker hot working-set signal (`#1053` and `#1168`, `n = 15` rounds)

Contrast the null baseline above with two archived i9 bundles that genuinely allocate per-worker slabs (`a6e427b1` `#1053`, `+524 KiB/w` 64K slabs causing the 20 MiB L3 cliff; `e98948b6` `#1168`, `+0.89 MiB/w` disjoint forward-lane slabs):

| Bundle (commit) | Corpus | `k` | `n` | Base med (MiB) | Head med (MiB) | `Δ med` (MiB) | Paired GM `[95% CI]` | `pair_sd` | `t`-stat |
|---|---|---:|---:|---:|---:|---:|---|---:|---:|
| `#1053` (`a6e427b1`) | `wikidata` | **1** | 15 | 78.90 | 79.48 | **`+0.58`** | **`1.0077 [1.0067, 1.0086]`** | `0.0017` | **`+17.4`** |
| `#1053` (`a6e427b1`) | `wikidata` | 8 | 15 | 103.07 | 107.16 | **`+4.10`** | **`1.0374 [1.0311, 1.0437]`** | `0.0109` | **`+13.0`** |
| `#1053` (`a6e427b1`) | `wikidata` | 10 | 15 | 110.95 | 116.24 | **`+5.29`** | **`1.0488 [1.0445, 1.0532]`** | `0.0075` | **`+24.5`** |
| `#1053` (`a6e427b1`) | `text` | **1** | 15 | 40.16 | 40.94 | **`+0.78`** | **`1.0176 [1.0160, 1.0192]`** | `0.0029` | **`+23.7`** |
| `#1053` (`a6e427b1`) | `text` | 8 | 15 | 124.06 | 122.34 | `-1.72` | `0.9952 [0.9670, 1.0243]` | `0.0519` | `-0.4` |
| `#1168` (`e98948b6`) | `wikidata` | **1** | 15 | 78.89 | 79.79 | **`+0.89`** | **`1.0120 [1.0110, 1.0131]`** | `0.0019` | **`+24.8`** |
| `#1168` (`e98948b6`) | `wikidata` | 8 | 15 | 103.89 | 110.27 | **`+6.38`** | **`1.0591 [1.0534, 1.0648]`** | `0.0098` | **`+22.7`** |
| `#1168` (`e98948b6`) | `text` (unchanged) | **1** | 15 | 36.56 | 36.72 | `+0.16` | `1.0009 [0.9993, 1.0025]` | `0.0029` | `+1.1` |

**Diagnostic separation:** Real per-worker heap growth `Δ_w` is detected at **`k = 1`** with `t = +17..+25` (against a `k = 1` null SD of `0.001–0.005`) and scales linearly as `k × Δ_w` across `k = 1, 8, 10` (`+0.58 → +4.10 → +5.29 MiB` for `#1053`; `+0.89 → +6.38 MiB` for `#1168`). Queue/allocator scheduling jitter is `~0` at `k = 1` and fluctuates by `±2–6%` non-monotonically at `k >= 2`.

---

## 4. Retrospective comparison of recent RSS-gated verdicts (C2 preserved)

*Every frozen STOP below stands permanently under RULES C2; this table only shows how each run compares against the calibrated null bands and the recommended rule in §5.*

| PR / Claim | Heap change | Frozen RSS bound | Observed RSS highlights | Calibrated null / §5 rule reading |
|---|---|---|---|---|
| **`#1179`** `crc32-avx2-ring-barrett-build` (CI Zen 4 `46054981092`) | `0 B` heap (`-8 KiB` table) | `<= 1.005x` point bound across all 21 cells (100% null FWER) | `wikidata 1T 1.0009x`, `text 1T 0.9986x`, `pattern 1T 1.0088x (+0.14 MiB)`, `text 4T 1.0221x`, `wikidata 2T 1.0166x`, `text 2T 1.0155x`, `pattern 4T 0.9742x` | **Meets §5 rule:** all `1T` cells `<= 1.010x` and `<= +0.5 MiB`; all `2T/4T` cells sit inside 95% CI null bands (`text 4T` edge `1.0201x`, `wikidata 2T` `1.0425x`, `text 2T` `1.0355x`) and far below `1.10x`. *(Frozen STOP stands.)* |
| **`#1119`** `jcc-aligned-release-build` (i9 `77236472`) | `0 B` heap (`.text` align flag) | `<= 1.020x` on `wikidata 8T` median | `wikidata 1T 0.9994x`, `wikidata 8T` median `1.0227x` (paired GM `1.0089 [0.9939, 1.0241]`), `10T 0.9972x`; `random 8T/10T` `1.0206x / 1.0229x` | **Meets §5 RSS rule:** `1T` flat (`0.9994x`), `8T` paired GM `1.0089x` well inside i9 null band (`1.03x`). *(Independently failed i9 wall gates; frozen STOP stands.)* |
| **`#1053`** `a6b-endpoint-build` (i9 `a6e427b1`) | `+524 KiB/w` 64K slabs (L3 cliff) | L3 budget / 8W–10W wall | `wikidata k=1 +0.58 MiB (1.0077x, t=+17.4)` → `k=8 +4.10 MiB (1.0374x)` → `k=10 +5.29 MiB (1.0488x)`; `text k=1 +0.78 MiB (1.0176x, t=+23.7)` | **Caught cleanly by §5:** `k = 1` exceeds `+0.5 MiB` at `t > 17` and grows linearly `k × Δ_w` past the 20 MiB L3 budget, confirming real working-set growth rather than noise. *(Frozen STOP stands.)* |
| **`#1117`** `scanner-a2-fast-build` (i9 `9dacc085` + CI Zen 5 `46091387084`) | `0 B` worker heap (small static image delta) | `PERFORMANCE-POLICY.md` §2 (`<= 1.10x`) | i9: `wikidata k=1 1.0006x`, `k=8..10 1.0061–1.0103x`, `random k=8 1.0167x`. CI Zen 5: `wikidata 1T +0.4%`, `pattern 1T +0.9% (+0.15 MiB)`, `pattern 2T +1.7%`, `text 2T -3.5%` | **Meets §5 rule on both i9 and CI:** `1T` static binary-page delta stays `<= +0.25 MiB` (`<= 1.010x`), and every `k >= 2` cell lies well inside null bands. *(Merged / PASS.)* |
| **`#1238`** `runs-front-and-fill-build` (CI Zen 3 `46078359802` & Zen 5 `46086718747`) | `0 B` heap (`-1 KiB` struct/stack) | `<= 1.005x` point bound across all 21 cells | Attempt 1 (Zen 3): `wikidata 1T +0.0%`, `wikidata 2T +4.4%`. Attempt 2 (Zen 5): `wikidata 1T +0.1%`, `text 1T -0.2%`, `text 2T +3.6%`, `pattern 4T -3.4%` | **Meets §5 RSS rule:** `1T` flat (`<= +0.2%` across all 7 corpora); `wikidata 2T +4.4%` and `text 2T +3.6%` sit at the 95% CI null edge (`+4.25%` / `+3.55%`) with `0 B` heap change. *(Independently missed CI wall exclusion of zero; frozen STOP stands.)* |

---

## 5. Recommended calibrated peak-RSS rule for future registrations

Future registrations (starting with `crc32-avx2-ring-rescue` and `runs-front-i9-rescue`) should cite this table rather than freezing uncalibrated `<= 1.005–1.020x` multi-thread point bounds:

1. **Zero-heap / constant-scratch changes (CI `bench.yml`):**
   - **Heap-invariance check at `1T`:** require every `1T` cell (all 7 corpora) to satisfy `paired GM <= 1.010x` **or** absolute median delta `<= +0.5 MiB` (absorbs `~100–250 KiB` binary `.text`/`.rodata` page-alignment shifts on `16 MiB` `pattern 1T`, while flagging any `>= 0.5 MiB/worker` allocation with `> 10σ` power).
   - **Multi-threaded CI cells (`2T`/`4T`):** apply **`PERFORMANCE-POLICY.md` §2 rule 4 (`<= 1.10x` across all corpora)** as the hard CI gate. If a tighter diagnostic flag is desired on primary cells (`wikidata`, `text` at `2T`/`4T`), flag only when `paired GM > 1.050x` **and** `CI_lo > 1.0`, and treat `pattern 4T` as exempt below `1.10x` unless stratified by the `39.0 MiB` bimodal cut. Never gate a zero-heap PR on an uncorrected `<= 1.005–1.020x` point bound across 14 multi-thread CI cells (63–100% null FWER).
2. **Physical i9-10900K paired workstation runs (C5):**
   - Use **paired geometric mean + 95% t-CI** (never an uncalibrated ratio of arm medians, which turned `1.0089x` GM into `1.0227x` median ratio on `#1119`).
   - **Zero-heap changes:** `k = 1` paired GM `<= 1.005x` (or `<= +0.25 MiB`); `k >= 4` `wikidata`/`text` paired GM `<= 1.030x` (`pair_sd = 0.006–0.027`); `k >= 4` high-variance guards (`random`, `records`, `pattern`) `<= 1.050x` (`pair_sd = 0.012–0.052`), or `PERFORMANCE-POLICY.md` §2 rule 4 (`<= 1.10x`).
3. **Heap-adding designs (e.g., per-worker slabs or lanes):**
   - State the expected per-worker heap delta `Δ_w` statically before data and measure it at **`k = 1`** (where null SD is `<= 0.005` on both CI and i9), then verify linear `k × Δ_w` scaling across `k = 1, 8, 10` against the target machine's 20 MiB L3 budget and `PERFORMANCE-POLICY.md` §2 rule 4 (`<= 1.10x`, or traded growth `<= 2×` primary wall gain and `<= 64 MB`).
