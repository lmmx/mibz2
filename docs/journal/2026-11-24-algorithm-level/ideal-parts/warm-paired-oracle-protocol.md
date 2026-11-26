# Unbiased paired-oracle K1 protocol: warm construction baselines and L3-aware scan-ideal baselines

*Frozen methodology note for claim [`warm-paired-oracle-k1`](../../../../frontier/claims/warm-paired-oracle-k1.toml) (26 November 2026). Design and measurement protocol only — no production code, no new workstation run, and **strictly prospective** under RULES B4 and C2 (`docs/team/RULES.md`). No prior registered verdict is relaxed or re-graded. Per `warm-paired-oracle-k1`'s frozen stop test and RULES C6, this protocol must be reviewed by a different agent before any striated adoption registration cites it as a binding standard.*

---

## 1. Purpose and scope

At the shipped 8-worker configuration on the i9-10900K (`wikidata ×4`), the serial Huffman scanner (`FCMRD = 179.1 ms`, `cc8ba137`) and the 8 parallel workers (`S = 184.9 ms`, `c71c4a56`) are co-limited within 4%. A standalone candidate on either side is masked by the other station, so diagnostic kill tests (`K1` / `K2`) pair the candidate against an oracle that makes the opposing station ideal:

- **`C`-paired scanner tests** (`pooled-scan`, future `pooled-scan-hybrid`): pair a scanner candidate `X` against the construction oracle `C` (`XC / C` against `SC / C`).
- **`S`-paired worker tests** (`mtf-fold-build`, future `forward-emission`): pair a worker candidate `X` against the scan oracle `S` (`XS / S` alongside standalone `X / none`).

Primary evidence from `zero-copy-floor` (`cc8ba137`, SHA-256 `10df0d895755494ec6f06369071d649674d855ab953201a2b2888845ab265e34`), `pooled-scan` (`c71c4a56`, SHA-256 `97e397ad586a6700c04a874e50dbc5e26ee9fc91912b115111cbe0550ddc8867`), and `ideal-matrix` (`7dd7b070`, SHA-256 `f0acf19929c8e2f59284e8271b0e109d51c47cd284e1fc511959334f60970fe5`) — analysed in [`cold-store-and-rle1-llc.md`](../../2026-11-26-explore/cold-store-and-rle1-llc.md) (#952), [`2026-11-26-synthesis/README.md`](../../2026-11-26-synthesis/README.md) (#954), and [`2026-11-26-synthesis/third-pass.md`](../../2026-11-26-synthesis/third-pass.md) — exposed two distinct measurement hazards that future registrations must prevent before spending workstation time:

1. **Asymmetric cold-store bias in `C`-paired scanner tests:** zero-copy `C` adds `+24.5..+32.7 ms/worker` of cold-store `A7` DRAM reads to worker-bound `PC` and `SC`, while scanner-bound `C` absorbs `0 ms`, systematically compressing measured capture.
2. **Cold-`C` reference distortion (`SC`), SMT oversubscription, and L3-occupancy tracking in `S`-paired worker tests:** `S` itself keeps construction real and warm (and scales through `k = 10`, unlike `SC`), but gating on `SC`-anchored capture reverses the bias (inflating apparent capture), and `k = 9..10` introduce 10-core SMT oversubscription and elevated `ready_bytes` high-water marks.

**Convention on traced activity sums:** In `c71c4a56`, non-pooled arms (`none`, `S`, `C`, `SC`) record three ledger phases (`startup`, `steady`, `drain`), whereas pooled arms (`P`, `PC`) have no block-issue points (`pooled-scan/registration.md` §7.1) and record all activity outside the steady window. Tables below report **all-phase sums** (`startup + steady + drain`, `n = 3` traced medians, matching `#952`) so all six arms are comparable on the same basis, with **steady-phase sums** (matching `third-pass.md` §1, within 1–2 ms) noted alongside where relevant. Timed walls are medians of `n = 12` untimed-ledger decodes (`pooled-scan/results/report.txt`).

---

## 2. Physical mechanism: oracle store footprint and station-binding asymmetry

### 2.1 Zero-copy oracle stores vs. the 20 MiB L3

From `c71c4a56` (`runs.jsonl`, `oracle_store` on `wikidata ×4`: 431 distinct blocks repeated 4× = 1,724 blocks, 233 distinct Open blocks × 4 = 932):

| oracle | distinct blocks | store size (bytes) | store size (MiB) | ratio to 20 MiB L3 | streamed per decode (MiB) | downstream stage exposed to cold read |
|---|---:|---:|---:|---:|---:|---|
| `S` (symbols) | 431 | 50,086,174 | 47.77 | 2.39× | 191.06 | `A5` (MTF/RLE2) — small: `+0.8–0.9 ms/w` at `k = 6..8` |
| `C` (Open `Built`) | 233 | 120,613,698 | 115.03 | 5.75× | 460.10 | `A7` (walk + spelling) — large: `+24.5..+32.7 ms/w` at `k = 8..9` |
| `E` (`u32` codes) | 233 | 106,270,324 | 101.35 | 5.07× | 405.39 | `E_consume` (spelling) — `+55–65 ms` at `k = 1` with cold `C` |
| `M` (BWT `L`) | 431 | 33,508,597 | 31.96 | 1.60× | 127.83 | `A6a` (construction pass 1) |
| `R` (final bytes) | 431 | 209,707,092 | 200.00 | 10.00× | 799.97 | `copy_out` (`D`) — inflates `D` from `29 ms` to `48–54 ms` |
| **Total** | — | **520,185,885** | **496.09** | **24.80×** | — | — |

In production (`none`, `S`, `P`), construction (`A6a–c`) writes a block's `Built` (~0.52 MB) directly into the worker's L2/L3 right before `A7` walks it. In zero-copy `C` arms (`C`, `PC`, `SC`), workers borrow `&Built` from the 115.0 MiB global store, turning L2/L3 hits in `A7` into DRAM reads (`cc8ba137` at `k = 1`: `SCMRD` reads `546.6 MiB` from DRAM vs. `151.1 MiB` for all of `none`, inflating `A7` from `545.2` to `697.9 ms`).

### 2.2 Station-binding asymmetry theorem (why `C`-paired capture is biased downward)

Let $C_{\text{warm}}$, $XC_{\text{warm}}$, and $SC_{\text{warm}}$ be the true warm-cache walls of the baseline (`C`), candidate (`XC`), and ceiling (`SC`) arms at worker count $k$, with $0 < SC_{\text{warm}} < XC_{\text{warm}} < C_{\text{warm}}$. Suppose the `C` oracle injects an additive per-worker cost $\Delta > 0$ into worker execution (either cold-store `A7` misses under zero-copy `C`, or copy handover `hand_C` under a copying `C` oracle).

- In the **scanner-bound baseline `C`**, workers carry idle slack `job_wait(C)/k`. Whenever $\text{job\_wait}(C)/k \ge \Delta$ (in `c71c4a56`, `50.6 ms/w` at `k = 8` and `91.8 ms/w` at `k = 9`, vs. $\Delta \approx 24.5\text{–}32.7\text{ ms/w}$), `C`'s wall absorbs **0 ms** of $\Delta$:
  $$C_{\text{obs}} = C_{\text{warm}}.$$
- In the **worker-bound arms `XC` and `SC`** (`job_wait ≈ 0.2–5.0 ms/w`), both walls absorb **100%** of $\Delta$:
  $$XC_{\text{obs}} = XC_{\text{warm}} + \Delta, \qquad SC_{\text{obs}} = SC_{\text{warm}} + \Delta.$$

**Proposition (monotone compression of linear and log-scale capture).** Under equal additive bloat $\Delta > 0$ absorbed by `XC` and `SC` but not by `C`:

1. **Linear capture** $\text{cap}_{\text{lin}}(\Delta) = \frac{C - (XC_{\text{warm}} + \Delta)}{C - (SC_{\text{warm}} + \Delta)} = 1 - \frac{XC_{\text{warm}} - SC_{\text{warm}}}{(C - SC_{\text{warm}}) - \Delta}$ is **strictly decreasing** in $\Delta$ for all $\Delta \in [0, C - XC_{\text{warm}})$.
2. **Log-scale capture** $\text{cap}_{\log}(\Delta) = \frac{\ln(C / (XC_{\text{warm}} + \Delta))}{\ln(C / (SC_{\text{warm}} + \Delta))} = 1 - \frac{\ln(1 + u)}{\ln(C / t)}$, where $t = SC_{\text{warm}} + \Delta$ and $u = (XC_{\text{warm}} - SC_{\text{warm}}) / t > 0$, has derivative
   $$\frac{d}{d\Delta}\text{cap}_{\log}(\Delta) = -\frac{\ln(1 + u) - \frac{u}{1 + u}\ln(C / t)}{t\,(\ln(C / t))^2}.$$
   Since $(1 + u)\ln(1 + u) > u$ for every $u > 0$, we have $(1 + u)^{1 + 1/u} > e \approx 2.718$. Therefore $\frac{d}{d\Delta}\text{cap}_{\log}(\Delta) < 0$ strictly whenever $C / (SC_{\text{warm}} + \Delta) \le e$ (equivalently $SC_{\text{obs}} / C_{\text{obs}} \ge e^{-1} \approx 0.368$). In `c71c4a56`, $SC_{\text{obs}} / C_{\text{obs}} \in [0.746, 0.884] \gg 0.368$, so **both linear and log-scale capture are strictly compressed downward** by $\Delta$, and the K1 gate $XC/C < \sqrt{SC/C} \iff \text{cap}_{\log} > 0.5$ is strictly harder to pass than on warm `C`.

Combining this with `third-pass.md` §2's work-conservation identity for pure pooling ($PC(k) \approx SC(k) + \text{scan}/k$ within 1.5% at $k = 6\text{–}9$), the numerator $PC - SC \approx \text{scan}/k$ is invariant to $\Delta$ while the denominator $C - SC$ shrinks by $\Delta$:

$$\text{cap}_{\text{lin}} \approx 1 - \frac{\text{scan}/k}{C - SC}.$$

### 2.3 Primary verification on `c71c4a56`

Timed medians (`n = 12`, `pooled-scan/results/report.txt`) paired with traced all-phase `A7` and `job_wait` medians (`n = 3`, `c71c4a56` `runs.jsonl`):

| `k` | `ΔA7/k` (`C−none`) | `ΔA7/k` (`PC−P`) | `ΔA7/k` (`SC−S`) | `job_wait(C)/k` | obs `PC/C` vs `√(SC/C)` | obs `cap_log` / `cap_lin` | warm-adj `PC/C` vs `√(SC/C)` | warm-adj `cap_log` / `cap_lin` |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 6 | +29.3 ms | +30.3 ms | +28.2 ms | 20.8 ms | 1.061 vs 0.943 | −0.50 / −0.54 | 0.897 vs 0.858 | +0.36 / +0.39 |
| 7 | +25.1 ms | +27.6 ms | +26.5 ms | 40.5 ms | 0.942 vs 0.892 | +0.26 / +0.28 | 0.792 vs 0.807 | +0.54 / +0.60 |
| **8** | **+29.3 ms** | **+27.5 ms** | **+24.5 ms** | **50.6 ms** | **0.863 vs 0.860** | **+0.49 / +0.52** | **0.713 vs 0.778** | **+0.67 / +0.73** |
| **9** | **+19.8 ms** | **+32.7 ms** | **+28.6 ms** | **91.8 ms** | **0.868 vs 0.872** | **+0.52 / +0.55** | **0.691 vs 0.778** | **+0.74 / +0.78** |
| 10 | +24.3 ms | +24.5 ms | +36.6 ms | 73.7 ms | 0.860 vs 0.886 | +0.62 / +0.65 | 0.729 vs 0.768 | +0.60 / +0.66 |

*(Note: `pooled-scan/results/report.txt` computes `PC/C` and `SC/C` as block-paired geometric means, giving `cap_log = 0.521` at `k = 8` and `0.481` at `k = 9`; the table above uses arm medians to align with the `n = 3` traced `ΔA7/k` medians. Warm-adjusted rows subtract `ΔA7(PC−P)/k` from `PC` and `ΔA7(SC−S)/k` from `SC` while leaving scanner-bound `C` unchanged. Notice that copying `C` in `7dd7b070` reduces $\Delta$ from `+25..+33 ms/w` to the copy handover `hand_C ≈ 3–6 ms/w`, which is smaller but still structurally asymmetric whenever `C` is scanner-bound.)*

---

## 3. Protocol for `C`-paired scanner tests (`pooled-scan-hybrid` and successors)

### 3.1 Tier 1 (Preferred): real fast-worker baseline, zero `C` oracle

The cleanest way to eliminate cold-store `C` asymmetry is to **not use a `C` oracle**.

1. **Pre-registration screening from `S`-paired worker runs (zero extra workstation time).** Every `S`-paired worker run (`mtf-fold-build`, `forward-emission`) directly measures `S'(8)` — the real warm-construction 8-worker wall with scan ideal. By `third-pass.md` §2:
   - Pure pooled work-conservation forecast: $P'(k) \approx S'(k) + \text{scan}/k$ (verified within 1.5% at $k = 6\text{–}9$);
   - Slack-only hybrid work-bound ceiling (serial scanner + $k$ workers scanning only from slack):
     $$H_{\text{ceil}}(k) = \frac{k \cdot S'(k) + \text{scan}}{k + 1};$$
   - Shipped co-limited forecast at $k = 8$: $\text{none}'(8) \approx \max(\text{scan}, S'(8)) + 8\text{ ms}$ (with $\text{scan} \approx 178\text{ ms}$).
   - **Admission threshold for `pooled-scan-hybrid`:** Do not queue a workstation run for `pooled-scan-hybrid` until landed worker improvements put $S'(8) \le 165\text{ ms}$ (where $H_{\text{ceil}}(8) \le 166.5\text{ ms}$, ~10% below $\text{none}'(8) \approx 186\text{ ms}$; if `mtf-fold` + `forward-emission` bring $S'(8)$ to `152–161 ms`, $H_{\text{ceil}}(8) \approx 155\text{–}163\text{ ms}$, 9–13% below $\text{none}'(8)$).
2. **Arm design on the real fast-worker baseline:** Run `none_fast`, `H` (candidate scanner on the same real fast workers), and `S_fast` (ideal `S` on the same real fast workers) — **zero `C` oracle in any arm**.
   - **Primary K1 gate (standalone real-worker speedup):** 95% upper bound of `H / none_fast ≤ τ_1` at `k = 8` (e.g. `τ_1 = 0.96` or `0.95`, chosen before data from $H_{\text{ceil}}(8) / \text{none}'(8)$).
   - **Primary K1b gate (optional capture against real warm `S_fast`):** 95% upper bound of `H / none_fast ≤ (S_fast / none_fast)^α` (e.g. `α = 0.5` for 50% log-scale capture of the real scan room behind real fast workers). Because `none_fast` and `H` run real warm construction and `S_fast` adds only `+0.8–0.9 ms/w` of symbol-store `A5` cost, `C`'s `+25–33 ms/w` cold-store bias is **identically zero**.
   - **Non-regression guards:** `H / none_fast` lower bound `≤ 1.01` (or `1.03`) at `k = 1, 6`; peak RSS `H / none_fast ≤ 1.10` at `k = 6..10`.

### 3.2 Tier 2 (Fallback only if a `C`-paired test is unavoidable before real worker wins land)

If a scanner prototype must be screened before real worker wins bring $S'(8) \le 165\text{ ms}$:

1. **Prohibition:** Never freeze unadjusted `XC / C` vs `√(SC / C)` with the zero-copy `C` oracle as a primary kill gate when `C` is scanner-bound and `XC`/`SC` are worker-bound.
2. **Tier 2A (Preferred fallback — regime-symmetric worker-bound gates):**
   - Pair **`X` against `S`** (both real warm `C`, worker-bound): measures total scan overhead `X(k) − S(k)` directly with zero `C` oracle. For a slack-only hybrid, `X` should stay strictly below `S(k) + scan/k` and approach `none(k)` below the knee.
   - If `C` is included, gate on **`XC / SC`** (both worker-bound under `C`, cancelling first-order cold-store bloat $\Delta$), paired with a trace check that `|ΔA7(XC − X) − ΔA7(SC − S)| / k ≤ 5 ms/w` at judged counts.
3. **Tier 2B (Pre-registered warm-adjusted capture — diagnostic only unless paired block-wise with `n ≥ 8` traces):**
   - Define the per-worker cold-store excess from matched real-`C` twins:
     $$\hat\delta_{XC} = \frac{\text{A7}_{XC} - \text{A7}_{X}}{k}, \qquad \hat\delta_{SC} = \frac{\text{A7}_{SC} - \text{A7}_{S}}{k}, \qquad \hat\delta_{C,\text{eff}} = \max\!\left(0,\; \frac{\text{A7}_{C} - \text{A7}_{\text{none}}}{k} - \frac{\text{job\_wait}(C)}{k}\right),$$
     and warm-adjusted walls $\widetilde{XC} = XC - \hat\delta_{XC}$, $\widetilde{SC} = SC - \hat\delta_{SC}$, $\widetilde{C} = C - \hat\delta_{C,\text{eff}}$.
   - **Why Tier 2B is secondary to Tier 1 and Tier 2A:** Subtracting an `n = 3` traced-ledger difference (which carries ledger instrumentation overhead) from `n = 12` untimed-ledger walls mixes two measurement modes and understates variance unless `n ≥ 8` paired traced blocks are collected and $\hat\delta$'s uncertainty is propagated into the 95% confidence interval. Tier 2B must **never** be applied post-hoc to rescue a failed unadjusted gate (RULES C2); it is valid only when pre-registered before data.
4. **Mandatory Regime-Asymmetry Diagnostic (all `C`-paired runs):** Report `job_wait/k` in `C`, `XC`, `SC` and `ΔA7/k` (`C − none`, `XC − X`, `SC − S`). Label any unadjusted `XC / C` capture **regime-asymmetric** whenever `job_wait(C)/k > 10 ms` AND `ΔA7(SC − S)/k > 5 ms/w` (or `> 10%` of `C − SC`).

---

## 4. Protocol for `S`-paired worker tests (`mtf-fold-build`, `forward-emission`, and successors)

### 4.1 Primary baseline evidence: cite `S` (real warm `C`), NOT `SC` (cold `C` oracle)

Earlier synthesis (#954) warned that `S`-paired worker tests hit a severe `R + D` L3 eviction cliff at `k = 9..10` (`copy_out = 77.9 ms`, `A9 = 215.6 ms`). `third-pass.md` §1 and the direct `c71c4a56` breakdown below show that **those cliff numbers belong to `SC` (which carries the 115.0 MiB cold `C` store), not to `S` (real warm `C`)**:

| `k` | arm | timed wall (ms) | `A5` sum (ms) | `A7` sum (ms) | `A9` sum all / steady (ms) | `copy_out` all / steady (ms) | `read_wait` (ms) | `job_wait/k` (ms/w) | `ready_bytes` hwm (MiB) | `ready_bytes` avg full / steady (MiB) |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 6 | `none` | 242.1 | 140.0 | 575.9 | 89.5 / 88.4 | 27.6 / 27.3 | 212.0 | 3.4 | 5.15 | 0.21 / 0.31 |
| 6 | **`S`** | **238.4** | 145.2 | 573.6 | 85.9 / 84.8 | 27.4 / 27.1 | 209.5 | 0.3 | 6.01 | 0.23 / 0.33 |
| 6 | `SC` | 164.2 | 147.3 | 742.9 | 94.2 / 93.0 | 29.0 / 28.6 | 134.8 | 0.3 | 6.01 | 0.27 / 0.44 |
| 8 | `none` | 192.9 | 146.0 | 581.0 | 104.0 / 102.8 | 34.0 / 33.5 | 157.2 | 8.5 | 6.87 | 0.39 / 0.60 |
| **8** | **`S`** | **184.9** | **152.9** | **583.9** | **96.9 / 95.4** | **34.0 / 33.5** | **148.2** | **0.2** | **6.01** | **0.41 / 0.64** |
| 8 | `SC` | 135.3 | 155.5 | 780.1 | 119.7 / 117.8 | 39.5 / 38.9 | 91.6 | 0.2 | 8.58 | 0.57 / 1.02 |
| 9 | `none` | 195.3 | 160.3 | 629.5 | 126.5 / 125.6 | 43.2 / 42.7 | 153.9 | 20.5 | 12.02 | 1.22 / 1.86 |
| **9** | **`S`** | **174.5** | **158.1** | **583.4** | **106.9 / 105.0** | **36.6 / 35.9** | **128.3** | **0.3** | **8.58** | **0.54 / 0.88** |
| 9 | `SC` | 140.2 | 172.1 | 841.1 | 152.3 / 150.2 | 54.9 / 54.1 | 75.6 | 0.6 | 10.30 | 1.10 / 1.98 |
| 10 | `none` | 197.6 | 166.3 | 655.3 | 133.0 / 131.3 | 43.7 / 43.1 | 168.5 | 42.5 | 14.59 | 1.28 / 1.92 |
| **10** | **`S`** | **169.6** | **173.5** | **625.9** | **129.8 / 127.7** | **48.7 / 47.8** | **115.1** | **1.7** | **16.31** | **1.40 / 2.30** |
| 10 | `SC` | 147.0 | 204.3 | 992.0 | **215.6 / 213.4** | **77.9 / 76.7** | 65.0 | 2.9 | 16.31 | 2.58 / 4.41 |

*(Primary source: `c71c4a56` timed medians `n = 12` from `pooled-scan/results/report.txt`; traced medians `n = 3` from `c71c4a56` `runs.jsonl`. `ready_bytes` avg full divides `occupancy.ready_bytes.integral` by `end_ns`; steady divides by `t2_ns − t1_ns`.)*

Four structural facts govern `S`-paired worker tests:

1. **`S` keeps construction real and warm.** `A7` in `S` (`583.9 ms` at `k = 8`) matches `none` (`581.0 ms`), vs. `SC`'s `780.1 ms`. Both `S` and any candidate `XS` are worker-bound (`job_wait/k = 0.2–0.3 ms/w` at `k = 6..9`), so there is no station-binding asymmetry.
2. **`S`'s own 47.8 MiB symbol store adds a tiny, symmetric `A5` cost.** At `k = 6..8`, `A5(S) − A5(none) = +5.2..+6.9 ms` total (`+0.8–0.9 ms/w`, `< 0.5%` of wall). Because both `S` and `XS` read the identical `S` store, this cost is symmetric across arms and cannot bias `XS / S` meaningfully.
3. **`S` scales cleanly through `k = 10` (`184.9 → 174.5 → 169.6 ms`), whereas `SC` scales backwards (`135.3 → 140.2 → 147.0 ms`).** At `k = 10` — at the **identical** `16.31 MiB` `ready_bytes` high-water mark — `S` shows `A9 = 129.8 ms` (`127.7 ms` steady, lower than `none`'s `133.0 / 131.3 ms`) and `copy_out = 48.7 ms` (`47.8 ms` steady, off the critical path with `read_wait = 115.1 ms`). The `215.6 ms` `A9` and `77.9 ms` `copy_out` cliff occurs only when `SC` streams the 115.0 MiB cold `C` store concurrently across 10 busy workers.
4. **Why `k = 8` still beats `k = 9..10` as the primary judged multi-worker count:**
   - **10-core SMT boundary:** On the 10-core i9-10900K, `none` has `1 scanner + 1 reader + k workers = k + 2` threads and hits SMT oversubscription at `k = 9` (`A6a..A7` jump `+9–11%`, `A9` `+22.5 ms`). `S` removes the busy scanner (`0.3 ms` handover), staying at `k + 1` threads (`≤ 10` cores through `k = 9`) and entering SMT oversubscription at `k = 10` (`11` threads on 10 cores: `A6a..A7` rise `+7–8%`, `A9` `+22.9 ms` over `k = 9`).
   - **Headroom against future L3 pressure:** At `k = 8`, `S` has zero worker SMT sharing (`9 ≤ 10` cores), `ready_bytes hwm = 6.01 MiB` (half of `READY_BYTES = 12 MiB`), `ready_bytes` time-average `0.41 MiB` (`0.64 MiB` steady), and `copy_out` 4.4× off the critical path. At `k = 10`, `ready_bytes hwm` is already `16.31 MiB` and faster future `XS` arms (`~140–155 ms`) will raise ready occupancy and SMT contention further.

### 4.2 Frozen rules for `S`-paired worker registrations

1. **Judged worker counts:**
   - **`k = 1` (primary, unmasked):** `X / none` (95% upper bound `≤ τ_1w`, set before data from the target part's real 1-worker budget).
   - **`k = 8` (primary, paired against `S` ideal):** `XS / S` (95% upper bound `≤ τ_8w`, set before data from the target part's real 8-worker budget in `S`).
   - **`k = 6, 8` (standalone non-regression guard):** `X / none` 95% lower bound `≤ 1.01` (or registered guard), plus peak RSS `X / none ≤ 1.05` (or `1.10` per `PERFORMANCE-POLICY.md`) across all tested counts.
   - **`k = 6, 9, 10` (diagnostic, reported-only):** Always run and report `none`, `X`, `S`, `XS` at `k = 6, 9, 10` to track scaling past the knee, SMT onset (`k = 9` in `none`, `k = 10` in `S`), and any `R + D` L3 giveback as workers speed up — never as primary kill counts unless a registration explicitly targets an L3/oversubscription lever (`delivery-l3-working-set`).
2. **Primary gate form — direct `XS / S`, NEVER `SC`-anchored capture:**
   - **Mandatory:** Gate the 8-worker paired kill test directly on **`XS / S ≤ τ_8w`** (paired block geometric mean, 95% t-interval upper bound, zero `C` oracle in the gate).
   - **Prohibition (reverse cold-store bias):** Never gate an `S`-paired worker test on capture anchored to `SC`, $\frac{\ln(S/XS)}{\ln(S/SC)}$. Because `SC` carries `+24.5 ms/w` of cold-`C` `A7` bloat at `k = 8` (`135.3 ms` observed vs. `~111 ms` warm), `SC` artificially shrinks $\ln(S / SC)$ and biases `SC`-anchored capture **upward** (overstating the candidate's capture). If `SC` or `SM` is run as an informational reference ceiling, report it as **reported-only** beside `ΔA7(SC − S)/k`.
3. **Mandatory trace diagnostics (every `S`-paired run):**
   Each registration's analysis script must extract and print, for every arm and count (stating explicitly whether sums are all-phase or steady-phase):
   - **`A9` sum (ms)** and `ΔA9 = A9(XS) − A9(S)` (detects any RLE1/CRC L3 giveback);
   - **`copy_out` and `read_wait` (ms)** (verifies delivery remains off the critical path);
   - **`ready_bytes` high-water mark (MiB) AND time-average (MiB)** (`integral / end_ns`, plus steady-phase `integral / (t2_ns − t1_ns)`);
   - **`A5` in `S` vs `none` and `XS` vs `X`** (verifies `S` symbol-store overhead stays symmetric, `< 1–2 ms/w`);
   - **`job_wait/k` in `S` and `XS`** (verifies both remain worker-bound, `< 2 ms/w` at `k = 8`);
   - **`ΔA7(SC − S)/k`** if `SC` is included as a reference arm;
   - **Downstream scan-readiness readout at `k = 8`:** compute `H_ceil(8) = (8·XS(8) + scan)/9` and `P'(8) = XS(8) + scan/8` (using the run's traced `none` `A2+A3` for `scan`), noting whether `XS(8) ≤ 165 ms` (the §3.1 admission threshold for `pooled-scan-hybrid`).
4. **Standard VOID conditions:**
   - Any byte-exactness, error-exactness, or Miri obligation unmet;
   - Wrong hardware (not i9-10900K) or scanner pin broken in `> 10%` of traced pinned decodes;
   - Removed-part busy residual `> 5%` (`A2+A3` in `S` over `none`);
   - Undefined 95% CI on any judged bound (`K1`, `K2`, `K3`);
   - Candidate fallback or bypass rate violating the registration's execution check (e.g. `fold_fallbacks > 0` on Open-admitted blocks).

---

## 5. Strict C2 compliance and impact on existing verdicts and claims

Under **RULES B4 and C2** (`docs/team/RULES.md`, `CLAUDE.md`), a decision rule frozen before data is never relaxed or re-graded after data is seen. This protocol applies **prospectively only**:

| artifact / claim | status under this protocol | action |
|---|---|---|
| **`pooled-scan` (`c71c4a56`, `#942`)** | **KILL stands permanently (C2).** Although cold-store `C` compressed observed `k = 8, 9` capture from `~0.67–0.74` (log) / `~0.73–0.78` (linear) to `0.52 / 0.48`, the frozen K1 rule produced KILL at both judged counts, and standalone `P/none` was `+5.2%` at `k = 8` and `+10.5%` at `k = 6`. | None. Recorded in `frontier/dead-ends.tsv` row 7; never reopened. |
| **`mtf-fold-build` (`mtf-fold/registration.md`, `#960`)** | **Frozen as written (C2); already compliant.** Its primary gates (`K1`: `k = 1` `F/none ≤ 0.975`; `K2`: `k = 8` `FS/S ≤ 0.975`; `K3`: `k = 6, 8` `F/none` lower bound `≤ 1.01`) use direct `FS / S` at `k = 8`, keep `SC` and `SM` reported-only, keep `k = 6, 9, 10` diagnostic, and log `A9`, `copy_out`, and `ready_bytes`. | No change to frozen gates. When reporting results, compare `FS` against `S` (not `SC`) at `k = 9..10` and include `ready_bytes` time-average alongside hwm. |
| **`forward-emission` (idea)** | Next striated `src/open.rs` worker build. | Once this protocol is reviewed by a different agent, register `forward-emission` under §4 (`k = 1` `X/none` + `k = 8` `XS/S` direct ratio gates, `k = 6, 9, 10` diagnostic, §4.3 trace table including `H_ceil(8)` readout). |
| **`pooled-scan-hybrid` (idea)** | Next scanner build. | Follow §3.1: wait until an `S`-paired worker run lands `S'(8) ≤ 165 ms`, then register real-warm-`C` `H / none_fast` and `H / S_fast` with **zero `C` oracle**. |
| **This protocol (`warm-paired-oracle-protocol.md`)** | Frozen methodology draft, smooth zone. | Requires an independent cold review by a different agent (leaving a review delta) before any striated adoption registration cites it. |

---

## 6. Registration pre-flight checklist

Before freezing a paired-oracle kill test or queueing a workstation run:

- [ ] **No asymmetric `C` oracle in a primary gate:** Scanner candidates either wait for real fast workers (`S'(8) ≤ 165 ms`, zero `C` oracle, §3.1) or gate on regime-symmetric worker-bound pairs (`X/S` or `XC/SC`, §3.2) — never unadjusted `XC/C` vs `√(SC/C)` with a scanner-bound `C` baseline.
- [ ] **No `SC`-anchored capture in an `S`-paired primary gate:** Worker candidates gate directly on paired geometric-mean `XS / S ≤ τ` at `k = 8` (plus `X / none` at `k = 1`), keeping `SC`/`SM` reported-only (§4.2).
- [ ] **Correct baseline cited for `k = 9..10`:** Cite `S` (real warm `C`: `A9 = 129.8 / 127.7 ms`, `copy_out = 48.7 / 47.8 ms`, wall `169.6 ms` at `k = 10`), not `SC` (`A9 = 215.6 ms`, `copy_out = 77.9 ms`).
- [ ] **Judged vs. diagnostic counts separated:** Primary judged counts are `k = 1` and `k = 8` (`k = 6, 8` standalone non-regression); `k = 6, 9, 10` reported diagnostically (§4.2).
- [ ] **Mandatory trace columns wired into the analysis script:** `A9` (sum), `copy_out`, `read_wait`, `ready_bytes` hwm **and** time-average, `A5` (`S` vs `none`), `job_wait/k`, `ΔA7(SC − S)/k` (if `SC` present), and `H_ceil(8)` / `P'(8)` scan-readiness projection, with phase convention (all-phase vs. steady) explicitly labelled (§4.3).
- [ ] **VOIDs complete and tested on synthetic fixtures:** Exact bytes/errors, Miri, i9 hardware + pin check (`≤ 10%` broken), removed-part busy residual `≤ 5%`, defined 95% CIs, and execution/fallback check (§4.4).
- [ ] **C2 intact:** Prior verdicts untouched; decision thresholds frozen before any workstation data is collected (§5).
