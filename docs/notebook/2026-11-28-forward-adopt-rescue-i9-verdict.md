# Physical i9-10900K C6 Adoption Verdict: `forward-emission-bounded-slab-rescue` (`DO NOT ADOPT`, C2 Preserved)

*28 November 2026. Unrelaxed RULES C2 readback of `origin/evidence 22f2d7d62a64f4537b2ddc67defb60a32b4dcd8e` (`forward-adopt-rescue/run-20261128T164925Z.tar.gz`), scored offline with `python3 bench/forward_adopt_rescue.py score` against the frozen Clause 3 registration in `docs/journal/2026-11-28-forward-emission-bounded-slab-rescue/registration.md` (`#1324`, `575fa98a`).*

## 1. Frozen Verdict & Provenance

- **Exact C2 verdict:** **`DO NOT ADOPT`** — `unmet C6 clause(s): wikidata RSS exceeded: wikidata/1 delta_rss=+868.0 KiB > +512.0 KiB` (**10 of 11 C6 clauses PASSED**, including every single wall, CPU, guard wall, guard RSS, and `wikidata/1..12` RSS geometric-mean bound; `wikidata/6, 8, 10, 12` all **reduced** peak RSS by `−132` to `−972 KiB`).
- **Evidence bundle:** `origin/evidence 22f2d7d62a64f4537b2ddc67defb60a32b4dcd8e`, file `forward-adopt-rescue/run-20261128T164925Z.tar.gz` (`338,607` bytes).
- **Bundle SHA-256 (verified 3-way: `.tar.gz`, `.sha256` sidecar, `22f2d7d6` commit message, and issue `#145` log):**
  `01c248a06a85250bf44cce4d71ee368624d2d691d7b142cc8c71968e1556b6dd`
- **Tested commit & arms:** `575fa98a9e80701c4a42c0799017cdc75691fe46` (`master`), `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, LLVM `23.1.1`.
  - `base` (`default`): SHA-256 `b761d2cc4224c202bce2e8ed3ac4b3dc9ab9c71e07841e6cd664ab5aa69ff336`
  - `cand` (`--features forward-anchor-lanes`): SHA-256 `3e0f55693296252f3cef7f77a0611bd53ea188acbb21d0866906911b06f1aa7f` (`distinct_binaries: PASS`)
- **Host & environment (`host.json` / `env-facts.json`):** unrestricted physical `Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz`, 10 cores / 20 CPUs (`cpus_allowed = 0..19`), `cpu_quota = unlimited`, `throttling = false`, `intel_pstate` (`powersave`, `balance_performance`, `turbo = true`, `perf_event_paranoid = 0`), `n = 16` paired rounds per cell (`544` timed decodes total), `--settle 2`, `--prespin-ms 100`, `--machine-state`.
- **Exactness (`parity.jsonl` + `runs.jsonl`):** `34/34` untimed `AB_SHA256=1` checks across all 7 corpora (`wikidata` `838,800,784` B SHA-256 `02949df2...`, `text`, `records`, `random`, `runs`, `pattern`, `onebyte`) and all `544/544` timed repeats matched C `libbzip2` (`exactness_ok: PASS`).

## 2. Complete 17-Cell Physical i9-10900K Table (`cand / base`, `n = 16` paired rounds)

| Cell | Arm | `base` wall | `cand` wall | Wall ratio `[95% CI]` | Paired med | CPU GM | Peak RSS (`base -> cand` MiB) | RSS GM | `delta_rss` (KiB) | `cpr` (`b/c`) | Eff GHz |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| **`wikidata/1`** | `mibz2-workers-lazy` | `1.5270s` | `1.3399s` | **`0.8773`** `[0.8751, 0.8795]` | `0.8784` | **`0.8773`** | `32.34 -> 33.19` | `1.0143` | **`+868.0`** (FAIL `> +512`) | `1.434/1.434` | `5.29` |
| **`wikidata/6`** | `mibz2` | `0.2399s` | `0.2094s` | **`0.8732`** `[0.8703, 0.8762]` | `0.8729` | **`0.8885`** | `55.70 -> 54.75` | **`0.9852`** | **`-972.0`** | `1.326/1.326` | `4.89` |
| **`wikidata/8`** | `mibz2` | `0.1936s` | `0.1760s` | **`0.9187`** `[0.9039, 0.9337]` | **`0.9153`** | **`0.8999`** | `60.93 -> 60.23` | **`0.9929`** | **`-722.0`** | `1.326/1.326` | `4.89` |
| **`wikidata/10`** | `mibz2` | `0.1938s` | `0.1756s` | **`0.9175`** `[0.8977, 0.9378]` | **`0.9085`** | **`0.9021`** | `60.68 -> 60.55` | **`0.9940`** | **`-132.0`** | `1.326/1.326` | `4.89` |
| **`wikidata/12`** | `mibz2` | `0.1941s` | `0.1759s` | **`0.9132`** `[0.8999, 0.9268]` | **`0.9051`** | **`0.8951`** | `60.86 -> 60.41` | **`0.9886`** | **`-462.0`** | `1.326/1.326` | `4.89` |
| `text/1` | `mibz2-workers-lazy` | `0.0522s` | `0.0519s` | `0.9839` `[0.9706, 0.9975]` | `0.9803` | `0.9853` | `23.36 -> 23.44` | `1.0070` | `+80.0` | `1.434/1.434` | `5.29` |
| `text/8` | `mibz2` | `0.0222s` | `0.0223s` | `1.0007` `[0.9885, 1.0131]` | `1.0023` | `0.9953` | `82.43 -> 82.55` | `0.9995` | `+124.0` | `1.331/1.332` | `4.88` |
| `records/1` | `mibz2-workers-lazy` | `0.1053s` | `0.1047s` | `0.9939` `[0.9857, 1.0021]` | `0.9981` | `0.9931` | `32.89 -> 32.97` | `1.0012` | `+80.0` | `1.434/1.433` | `5.29` |
| `records/8` | `mibz2` | `0.0384s` | `0.0378s` | `0.9934` `[0.9841, 1.0029]` | `0.9882` | `0.9920` | `71.73 -> 71.97` | `1.0033` | `+244.0` | `1.358/1.353` | `4.81` |
| `random/1` | `mibz2-workers-lazy` | `0.0895s` | `0.0872s` | `0.9815` `[0.9758, 0.9873]` | `0.9836` | `0.9826` | `29.69 -> 29.61` | `0.9987` | `-80.0` | `1.434/1.434` | `5.29` |
| `random/8` | `mibz2` | `0.0338s` | `0.0336s` | `0.9845` `[0.9737, 0.9955]` | `0.9880` | `0.9916` | `70.79 -> 69.32` | `0.9863` | `-1506.0` | `1.355/1.354` | `4.82` |
| `runs/1` | `mibz2-workers-lazy` | `0.0083s` | `0.0083s` | `1.0000` `[0.9826, 1.0176]` | `1.0000` | `0.9994` | `23.75 -> 23.75` | `1.0017` | `+0.0` | `1.433/1.433` | `5.25` |
| `runs/8` | `mibz2` | `0.0116s` | `0.0114s` | `0.9858` `[0.9723, 0.9996]` | `0.9787` | `1.0043` | `19.69 -> 19.77` | `1.0005` | `+80.0` | `1.408/1.410` | `4.81` |
| `pattern/1` | `mibz2-workers-lazy` | `0.0228s` | `0.0228s` | `0.9989` `[0.9612, 1.0380]` | `0.9885` | `0.9992` | `14.38 -> 14.38` | `1.0063` | `+0.0` | `1.433/1.433` | `5.27` |
| `pattern/8` | `mibz2` | `0.0159s` | `0.0156s` | `0.9857` `[0.9554, 1.0170]` | `0.9936` | `0.9917` | `65.47 -> 66.02` | `1.0102` | `+560.0` | `1.328/1.328` | `4.89` |
| `onebyte/1` | `mibz2-workers-lazy` | `0.0124s` | `0.0123s` | `1.0009` `[0.9818, 1.0204]` | `0.9961` | `0.9991` | `33.44 -> 33.44` | `1.0006` | `+0.0` | `1.433/1.432` | `5.26` |
| `onebyte/8` | `mibz2` | `0.0156s` | `0.0157s` | `1.0081` `[0.9884, 1.0283]` | `1.0097` | `1.0145` | `29.38 -> 29.38` | `1.0023` | `+0.0` | `1.417/1.415` | `4.88` |

## 3. Clause 1–3 Checklist

- **Clause 1 (Exactness & safety, `#![forbid(unsafe_code)]` in `src/open/lanes.rs`):** **PASS.** CI runs Miri on `open::lanes::tests::miri_*` (`0` UB); independent C6 reviewer `#1382` (`f47a2fda`) verified `cargo test --release --locked --features forward-anchor-lanes` (`218/218` lib + all integration suites PASS) at `e125ef95` (identical Rust tree to queued `575fa98a`); `22f2d7d6` verified `34/34` `AB_SHA256=1` parity and `544/544` timed repeats vs C `libbzip2`.
- **Clause 2 (Bounded scratch & local/CI RSS):** **PASS.** `#1312` statically bounds per-worker slab scratch at `<= 80 KiB/worker` (zero retained slab pool), verified on all 6 1T guards (`-80` to `+80 KiB`) and on `#1382`'s local 1T spot check.
- **Clause 3 (Physical i9-10900K C6 adoption — 10/11 PASS, 1/11 FAIL):**
  - `exactness_ok`: **PASS**
  - `distinct_binaries`: **PASS**
  - `wikidata_1_hi_le_0.900`: **PASS** (`0.8795 <= 0.900`, `−12.27%` wall; retired instructions cut `18.34B -> 15.36B`, `−16.2%`!)
  - `wikidata_6_hi_le_0.900`: **PASS** (`0.8762 <= 0.900`, `−12.68%` wall)
  - `wikidata_8_hi_le_0.975`: **PASS** (`0.9337 <= 0.975`, `−8.13%` wall — more than double `#1217`'s `−3.6%` 8W speedup because removing the `+6.71 MiB` pool eliminated 8-worker L3 pressure!)
  - `wikidata_10_hi_le_0.985`: **PASS** (`0.9378 <= 0.985`, `−8.25%` wall)
  - `wikidata_8_med_le_0.960`: **PASS** (`0.9153 <= 0.960`, `−8.47%` paired median)
  - `wikidata_8_cpu_le_0.940`: **PASS** (`0.8999 <= 0.940`, `−10.01%` CPU)
  - `guards_no_regression`: **PASS** (`12/12` cells clean; max guard `t_ci95_lo` `0.9885`, max paired median `1.0097 <= 1.020`)
  - `guards_rss_ok`: **PASS** (`12/12` cells `0.9863–1.0102 <= 1.050`)
  - `wikidata_rss_ok`: **FAIL** — every RSS geometric-mean bound **PASSED** (`wikidata/1` `1.0143 <= 1.020`, `wikidata/6` `0.9852`, `wikidata/8` `0.9929`, `wikidata/10` `0.9940`, `wikidata/12` `0.9886 <= 1.030`), and `wikidata/6, 8, 10, 12` all **saved** `132–972 KiB`, **but** the conjunctive `wikidata/1` median-delta sub-clause `delta_median_kib <= +512.0 KiB` tripped at **`+868.0 KiB`** (`33,116 -> 33,984 KiB`).

## 4. Root Cause of the Sole Failing Sub-Clause (`wikidata/1 delta_rss = +868 KiB`)

Inspecting all 16 paired `rss_kb` samples in `wikidata_1t.jsonl` (`runs.jsonl`, `mibz2-workers-lazy -t 1` pinned to CPU 0, running 4 threads `base/cand-ab`, `mibz2-read`, `mibz2-scan`, `mibz2-0`):

- **`base` sorted `rss_kb` (KiB):** `[32860, 32876, 32876, 32880, 32920, 33036, 33056, 33116, 33116, 33128, 33244, 33772, 34112, 36948, 37844, 38072]` (median `33,116 KiB`, range `5.21 MiB`).
- **`cand` sorted `rss_kb` (KiB):** `[32880, 33104, 33112, 33180, 33216, 33356, 33956, 33972, 33996, 34012, 34024, 34068, 37080, 37148, 37192, 37196]` (median `33,984 KiB`, range `4.32 MiB`).
- **Paired round diffs (`cand - base`, KiB):** `[-4964, -4060, -3844, -896, -16, +112, +184, +320, +952, +968, +1052, +1116, +4032, +4156, +4204, +4320]` (`t_ci95` for RSS ratio `[0.9705, 1.0601]`, crossing `1.000`).

Both arms occupy the **exact same three discrete pipeline-buffer modes** separated by one `~900 KiB` completed bzip2 output block (`Mode 0 -> Mode 1`) and a `+4 MiB` peak-queue jump (`Mode 2`):

| Mode (`rss_kb` band) | `base` rounds (`n=16`) | `cand` rounds (`n=16`) | Physical interpretation |
|---|---:|---:|---|
| Mode 0 (`32,860–33,356 KiB`) | `11 / 16` | `6 / 16` | Baseline steady-state floor (`base` min `32,860`, `cand` min `32,880`, `+20 KiB`) |
| Mode 1 (`33,772–34,112 KiB`) | `2 / 16` | `6 / 16` | `+1` ready `~900 KiB` block buffer waiting for `main` drain |
| Mode 2 (`36,948–38,072 KiB`) | `3 / 16` | `4 / 16` | `+4 MiB` peak queue / allocator high-water |

Because `#1312` makes the single worker `12.3%` faster (`1.527s -> 1.340s`) on the shared pinned CPU, the worker completes blocks slightly ahead of `main`'s drain cadence more often, shifting 5 rounds from Mode 0 into Mode 1 (`+900 KiB`) and stepping the 16-sample median across the discrete `900 KiB` pipeline quantum (`33,116 -> 33,984 KiB = +868 KiB`) — which exceeds the sub-quantum `+512 KiB` median-delta sub-clause. By contrast:
1. Every 1T guard corpus stays within **`−80` to `+80 KiB`** (`text/1 +80`, `records/1 +80`, `random/1 −80`, `runs/1 +0`, `pattern/1 +0`, `onebyte/1 +0`), directly confirming the `80 KiB` bounded slab.
2. Every multi-worker `wikidata` cell (`6, 8, 10, 12`) **reduces** peak RSS (`−972`, `−722`, `−132`, `−462 KiB`) relative to shipped `master` (eliminating `100%+` of `#1217`'s `+6.71 MiB` 8W RSS tax!).

## 5. Disposition

Under **RULES C2**, the frozen pre-data stop test is **never** relaxed after seeing the data: `forward-emission-bounded-slab-rescue` closes as **`DO NOT ADOPT`** (`status = "dropped"`), `bench-forward-adopt-rescue` is dequeued from `bench/workstation-queue.txt`, and a scoped row is recorded in `frontier/dead-ends.tsv`. This closes only the standalone `#1312` adoption run under the sub-quantum `+512 KiB` `wikidata/1` median-step rule; it does **not** close `#1312`'s bounded-slab four-lane forward walk (which delivered `−8.1%` to `−12.7%` physical i9 wall across `1..12T`, `−10.0%` 8W CPU, `12/12` clean guards, and negative RSS at `6..12T`), e.g. composed with `u8 SymBuf` / `#1328` (`−1.1 MiB` 1W RSS) or a fresh follow-on adoption registration.
