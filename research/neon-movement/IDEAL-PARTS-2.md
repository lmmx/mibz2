# Ideal-Parts Pass 2: Mathematics Against the Measured Ceiling Table

*Branch `research/neon-movement`. Reads `origin/master` at `384eac4` (`docs/journal/2026-11-24-algorithm-level/ideal-parts/results/full/{README.md,report.txt,ideal-matrix.tsv,scoring-neon-1.tsv}`, `results/slice-a/{README.md,scoring-neon-1.tsv}`, `antigravity/{prompt-2.md,notes-for-neon.md}`, `math-targets/data/{four-arm-runs.tsv,character-stage1/report.txt}`), `origin/bench/ideal-parts` (`e9d5e778`, oracle implementations `src/{scan_oracle.rs,supplied_codes.rs,open/supplied.rs,ideal/*.rs}`), and `research/neon-movement/IDEAL-PARTS-1.md` (`92d3723`). Evidence run: `ideal-matrix/run-20261125T125334Z` (`7dd7b070`, 2,492 decodes, valid on all checks). Labels `[PROVED]`, `[ARGUED]`, `[CONJECTURED]` follow `programme.md`. Design and mathematics only; `src/` untouched.*

---

## 1. Predictions Against the Measurement: Error Audit and Model Revision

In `results/slice-a/scoring-neon-1.tsv`, Pass 1 hit **8 of 9** values (missing only `E` at $k = 10$: predicted `0.990`, measured `0.975 [0.965, 0.986]`). In `results/full/scoring-neon-1.tsv`, Pass 1 hit **14 of 28** values, missed all 8 `MASKED` labels, hit 2 of 4 `NO ROOM` labels (`D` at $k = 6, 10$), and correctly named **`S × C`** as the dominant interaction (already unmasking at $k = 6$: `q_SC = −0.1103`, `16.9%` of variation; `q_SC = −0.0705`, `13.1%` at $k = 8$, `report.txt` L47, L71).

### 1.1 Where Pass 1 Held (Models Confirmed, No Revision Needed)

1. **Single-thread stage shares and the `C > E` ordering (`IDEAL-PARTS-1.md` §§1.2–1.5):**
   - `C` at $k = 1$: predicted `0.570 [0.545, 0.600]`, measured **`0.5792 [0.5733, 0.5851]`** (`scoring-neon-1.tsv` L6).
   - `E` at $k = 1, 6, 8$: predicted `0.830, 0.850, 0.980`, measured **`0.8345, 0.8507, 0.9652`** (3/3 hits; $k = 6$ to `0.07%`, L10–12).
   - `S` at all four counts ($k = 1, 6, 8, 10$): **4/4 hits** in both slice A and the full matrix (L2–5).
   - `M` at $k = 6, 8$: predicted `0.916, 0.983`, measured **`0.9160, 0.9755`** ($k = 6$ exact to 4 digits, L15–16).
2. **The two-halves split of `A7` (`IDEAL-PARTS-1.md` §1.2, §2.1 Subtlety A):** Pass 1 derived that `supplied-codes` removes only the `edges` pointer-chase (`44.4%` of `A7`) and leaves `~55.6%` (`~272 ms` at $k = 1$) in word spelling. Confirming this directly: in `four-arm-runs.tsv` (arm B at $k = 1$), `deliver_ns = 25.8 ms` and `consume_ns = 259.4 ms`, sum `285.2 ms` — matching `ideal-matrix.tsv` L260 `hand_E = 285.55 ms` to `0.35 ms`, and giving `E net of handover = 0.6165 = 1 − 0.3835` (`A7`'s exact `38.3%` share in `character-stage1/report.txt` L32).
3. **The `C × E` and `C × M` sign flips at $k = 1 \to 6$ (`IDEAL-PARTS-1.md` §2.4):** Predicted 2×2 super-additivity $\text{SA}(C, E) = 0.84$ at $k = 1$ (algebraic log-convexity) flipping to `1.17` at $k = 6$ (scanner-wall censoring); measured **`0.8438`** at $k = 1$ and **`1.1843`** at $k = 6$ (`ideal-matrix.tsv` L93–103, L136–146; `C × M` predicted `0.92 / 1.09`, measured **`0.9160 / 1.0801`**).
4. **The `R` 1-copy handover trap (`IDEAL-PARTS-1.md` §2.1 Subtlety B, L312):** Pass 1 registered zero-copy `R` (`0.942, 0.941, 0.985, 0.992`) as its primary line — which hit `R` net of handover at $k = 1, 6$ (`0.9428, 0.9523`, `scoring-neon-1.tsv` L18–19) — and explicitly warned in L312 that a 1-copy `839 MB` handover would regress past `1.00` (`0.968, 1.025, 1.015, 1.005`), bracketing all four measured values (`0.983, 0.996, 1.009, 1.021`).

### 1.2 Where Pass 1 Was Wrong — Four Root Causes and What Each Teaches

The 14 value misses and 8 `MASKED` label misses collapse into **four physical causes**, two of which expose real decoder physics that Pass 1's 2-station model omitted:

| Miss group | Missed cells (`scoring-neon-1.tsv`) | Predicted vs Measured | What the error teaches about the decoder |
|---|---|---|---|
| **A. 1-copy `R` + DRAM/lock-saturated floor** | `R` at $k = 1, 6, 8, 10$ (4); `all` at $k = 1, 6, 8, 10$ (4) | `R`: `0.942–0.992` vs `0.983–1.021`; `all`: `0.245–0.330` vs **`0.299–0.721`** (`391 / 94 / 112 / 142 ms`) | At $k = 1$, the `all` miss (`314` vs `391 ms`) is purely 1-copy `R` (`+52.4 ms`) + `E::deliver` (`+25.8 ms`). At $k = 6 \to 8 \to 10$, however, `all-ideal` **rises** (`94 → 112 → 142 ms`) even as per-thread handovers *fall* (`71.5 → 59.0 → 56.0 ms`): simultaneous oracle copies (`~4.9–5.7 GB` bus traffic from `495 MiB` cold stores, `25×` the `20 MiB` L3) plus 4 global `RwLock`s slam into the i9-10900K's dual-channel DDR4 bandwidth wall (`~35–42 GB/s`). |
| **B. Cross-station worker → scanner LLC/IMC coupling** | `C` at $k = 8, 10$ (2); `E`, `M` at $k = 10$ (2); **all 8 `MASKED` labels** | `C` at $k = 8, 10$: `0.973, 0.980` vs **`0.9437, 0.9467`**; `E, M` at $k = 10$: `0.990, 0.992` vs **`0.9780, 0.9794`** | **Major decoder lesson `[ARGUED]`.** In `all-real` at $k = 8\text{–}10$, wall is `195–196 ms`, yet when `C` alone is ideal, wall drops to **`183.2 / 184.0 / 185.5 ms`** at $k = 6, 8, 10$ — identical to `all-but-S` (`184.7 / 181.1 / 184.6 ms`, `ideal-matrix.tsv` L130, 173, 216), and adding `E` ideal on top of `C` saves **0%** (`E with C = 1.008 / 1.013 / 0.993`). Thus the pinned scanner's uncontended time is `~182–184 ms`, and active workers tax the pinned scanner by **`+10–12 ms` (`+5.5–6.5%`)** via shared L3/IMC contention (confirmed independently by `hand_S` on the exact same pinned 4.91 GHz core with idle sibling rising **2.2×**: `15.3 → 20.3 → 24.9 → 33.6 ms`, `ideal-matrix.tsv` L258–288!). Because removing `C` (especially `A6c`'s `106.3M` scattered L3 RMWs) unloads the ring bus and cuts worker duty cycle from `~95%` to `~55%`, `C`, `E`, and `M` leak **`2.1–5.6%`** through the scanner wall even at $k = 10$ — making `NO ROOM` (`±2%`) impossible and causing every `MASKED` label to miss (`registration.md` Clarification 2). |
| **C. Emerging memory-bandwidth wall in `S+C` and `D` at $k = 8 \to 10$** | Pairwise `S+C` at $k = 8, 10$, `q_EC` at $k = 8, 10$, `q_SD` & `q_CD` | `S+C` predicted `0.585 / 0.560`, measured **`0.6405 (124.9 ms)` at $k = 8$ and `0.7587 (148.7 ms)` at $k = 10$** (`+23.8 ms` slower at $k = 10$ than $k = 8$!); `q_EC` predicted negative, measured **`+0.0262 / +0.0214`** | **Second major decoder lesson `[ARGUED]`.** When both `S` and `C` are fast, aggregate per-worker working set crosses the `20 MiB` L3 between $k = 6$ (`~15 MB`), $k = 8$ (`~20 MB`), and $k = 10$ (`~25 MB` + 1 SMT pair), driving `S+C` from `124.9 ms` ($k = 8$) **up** to `148.7 ms` ($k = 10$, slower than $k = 6$'s `143.1 ms`!). Once fast arms approach the `~112–148 ms` memory-bus regime, (i) `C` and `E` are censored from below by the memory wall (`q_EC > 0`), and (ii) `D`'s `1.68 GB` `copy_out` moves directly onto the critical path (`q_SD = −0.032 / −0.035`, `q_CD = −0.027 / −0.020`, `floor share D = 1.452 / 1.287` = `+50.7 / +40.7 ms`, `ideal-matrix.tsv` L208, 251). |
| **D. Minor 1–2% calibration misses** | `M` at $k = 1$ (1); `D` at $k = 10$ (1) | `M` $k = 1$: `0.905` vs `0.9248` (`0.9178` net: **hit**); `D` $k = 10$: `0.985` vs `0.9938` | `all-real` at $k = 1$ was `1.306 s` (vs stage 1's `1.283 s`, `+1.8%` from `495 MiB` stores + `staged` hooks), and `hand_M = 9.2 ms` (`0.7%`); net of handover, `M` hit (`[0.8995, 0.9364]`). For `D` at $k = 10$, `copy_out` contention in `all-real` is `0.6%` rather than `1.5%`, confirming `D` is hidden behind `S` in `all-real` and only emerges behind `S+C`. |

### 1.3 Revision of Pass 1's Performance Model

Pass 1 used a **2-station sharp max** (`IDEAL-PARTS-1.md` §2.1) with uncoupled scanner time $T_{\text{scan}}(k)$ and no memory-bandwidth station:
$$T_{\text{Part 1}}(k, \text{arm}) = \max\bigl(T_{\text{scan}}(k),\; T_{\text{wkr}}(k, \text{arm})\bigr) + \varepsilon_{\text{startup/drain}}.$$
That model is **falsified** at $k \ge 8$ by (i) `C, E, M` leaking `2.1–5.6%` at $k = 10$, (ii) `S+C` inverting from `124.9 ms` ($k = 8$) to `148.7 ms` ($k = 10$), (iii) `all-ideal` inverting `94 → 112 → 142 ms`, and (iv) `floor share R < 1` (`0.841` at $k = 8$) and `floor share E < 1` (`0.914` at $k = 10$).

**Revised 3-station coupled max-plus model `[ARGUED]`:**
$$T_{\text{rev}}(k, \text{arm}) = \operatorname{smax}_\tau\Bigl(\, T_{\text{scan}}(\text{arm}, k),\; T_{\text{wkr}}(\text{arm}, k),\; T_{\text{mem}}(\text{arm}, k) \,\Bigr)$$
with three explicit couplings calibrated directly from `ideal-matrix.tsv` and `report.txt`:

1. **Contention-coupled scanner station:**
   $$T_{\text{scan}}(\text{arm}, k) = \begin{cases} T_{\text{scan}}^0 + \kappa_S \, \rho_{\text{wkr}}(\text{arm}, k)\, \beta_{\text{LLC}}(\text{arm}, k) & (S = \text{real}), \\ h_S(k) & (S = \text{ideal}), \end{cases}$$
   where $T_{\text{scan}}^0 \approx 182\text{–}184\text{ ms}$ (read directly from `C alone`: `183.2 / 184.0 / 185.5 ms` and `all-but-S`: `184.7 / 181.1 / 184.6 ms`), and the worker-induced L3/IMC tax $\kappa_S \rho_{\text{wkr}} \beta_{\text{LLC}} \approx 10\text{–}12\text{ ms}$ in `all-real` at $k = 8\text{–}10$ (dropping toward `~0–2 ms` once `C` is ideal).
2. **Smooth-max knee ($\tau \approx 4\text{–}7\text{ ms}$ at $k = 8$):** Finite-window block-level variability (`E[max(S_j, W_j)] > max(E[S_j], E[W_j])`) plus startup/drain accounts for `~5–8 ms` when $T_{\text{scan}} \approx T_{\text{wkr}}$, collapsing once either station is well below the other.
3. **Shared DRAM-bandwidth & lock-contention station:**
   $$T_{\text{mem}}(\text{arm}, k) = \frac{B_{\text{DRAM}}(\text{arm}, k)}{\text{BW}_{\text{eff}}(k)} + T_{\text{lock}}(\text{arm}, k),$$
   where $B_{\text{DRAM}}$ sums (a) cold oracle store reads + write-allocated target copies (up to `~4.9–5.7 GB` in `SECMRD`), (b) real `D`'s `1.68 GB` `copy_out` when `D = real`, and (c) spilled per-worker L3 working sets once $k \cdot W_{\text{wkr}} > 20\text{ MiB}$ ($k \ge 8$). On the i9-10900K ($\text{BW}_{\text{eff}} \approx 35\text{–}42\text{ GB/s}$), adding `D` (`+1.68 GB`) on top of a saturated bus adds `1.68 / 38 ≈ +44 ms` — matching the measured `floor share D` jump of **`+50.7 ms` ($k = 8$)** and **`+40.7 ms` ($k = 10$)** (`ideal-matrix.tsv` L208, 251). Conversely, turning `R` from ideal back to real in `all-ideal` replaces `1.68 GB` of cold DRAM copy with L3-warm in-place RLE1+CRC, **cutting** wall by `−17.8 ms` ($k = 8$) and `−15.9 ms` ($k = 10$).

**Practical consequence for tuning:** Even after removing oracle copies, the `20 MiB` L3 spill boundary makes **`k = 8` (or `k = 8–9` pooled with no dedicated scanner thread) the true production target, not `k = 10`**.

---

## 2. Re-Ranking the Targets by Measured Room, Masking Map, and Currency-Based Capture

### 2.1 Measured Room and Masking Map (`results/full/README.md` §§1–3, `ideal-matrix.tsv`)

A part's room behind another part $Q$ counts only if $Q$ can plausibly be made cheaper too:
- **Does `C`'s room behind `S` (`−39.5%` at $k = 6$, `−33.9%` at $k = 8$) count?** **YES `[ARGUED]`:** Cross-block magic-indexed parallel scanning inside the worker pool cuts `S`'s serial depth $D$ by $k\times$ (`[PROVED]`, `IDEAL-PARTS-1.md` §1.1), freeing the dedicated scanner core and eliminating the `184 ms` serial wall.
- **Does `S`'s room behind `C` (`−21.9%` at $k = 6$, `−32.1%` at $k = 8$) count?** **YES `[ARGUED]`:** `C`'s three sub-phases (`A6a = 200 ms`, `A6b = 216 ms`, `A6c = 161 ms` at $k = 1$, `character-stage1/report.txt` L32) carry concrete, non-asymptotic structural waste (`A5+A6a` 127-cap deforestation, `A6c`'s `106.3M` scattered RMWs, `A6b`'s `~103` cycles/IET-step hashing).
- **Does `D`'s room behind `S` and `C` (`q_SD = −0.032/−0.035`, `floor share D = 1.452 / 1.287`) count?** **Conditionally YES `[PROVED]`:** Zero-copy `read_block` already exists (`src/parallel.rs:880`), and matters once `S+C` pushes wall toward `~120–130 ms`.

| Rank | Part (Target) | Measured room alone ($k = 1 / 6 / 8 / 10$) | Net of handover ($k = 1 / 6 / 8 / 10$) | Masked room behind partner ($k = 6 / 8 / 10$) | Var. share ($1/6/8/10$) | Verdict |
|---:|---|---|---|---|---|---|
| **1a** | **C** (Construction, **T2**) | **`0.579` / `0.754` / `0.944` / `0.947`** | `0.556 / 0.729 / 0.918 / 0.924` | Behind `S`: **`0.605 / 0.661 / 0.825`** (`S+C = 0.589 / 0.641 / 0.759`) | **80% / 46% / 19% / 9%** | **#1 worker target** (`42%` room at $k=1$, `34%` behind `S` at $k=8$; co-limits at $k=6\text{–}8$). |
| **1b** | **S** (Huffman scan, **T4**) | `1.010 / 0.974 / 0.968` / **`0.919`** | `0.998 / 0.891 / 0.841 / 0.749` | Behind `C`: **`0.781 / 0.679 / 0.801`** | **0% / 22% / 46% / 53%** | **#1 gatekeeper** (mandatory to unlock `C` and `E` at $k = 8\text{–}10$; serial LUT spent, topology open). |
| **3** | **E** (Emission, **T1**) | **`0.835` / `0.851`** / `0.965 / 0.978` *(chase only)* | `0.617 / 0.634 / 0.752 / 0.784` *(chase + spelling!)* | Behind `S`: `0.845 / 0.866 / 0.909`; behind `C`: `1.008 / 1.013 / 0.993` | 14% / 8% / 1% / 0% | **#2 worker target** (`16.5%` chase + `19.9%` unremoved `consume` spelling = `38.3%` total `A7`; best attacked jointly with `C`'s `A6c`). |
| **4** | **D** (Delivery) | `1.011 / 0.989 / 0.992 / 0.994` (`NO ROOM` at 6, 10) | — | Behind `S`: `0.994 / 0.979 / 0.963`; **floor share `1.16 / 1.45 / 1.29`** | 0% / 0% / 4% / 10% | **Conditional #4**: zero standalone room today; becomes critical once `S+C` enters the `~120 ms` memory-bus regime. Already solved (`read_block`). |
| **5** | **M** (MTF/RLE2, **T3**) | **`0.925` / `0.916`** / `0.976 / 0.979` | `0.918 / 0.908 / 0.966 / 0.970` | Behind `S`: `0.923 / 0.914 / 0.929` | 4% / 2% / 1% / 1% | Mostly spent standalone (`Q_DRAM = 0`); capture via `A5+A6a` deforestation with `C`. |
| **6** | **R** (RLE1/CRC, **T6**) | `0.983 / 0.996 / 1.009 / 1.021` | `0.943 / 0.952 / 0.956 / 0.962` | Behind `S`: `1.002 / 1.004 / 1.018` (`~0.95` net) | 0% / 0% / 1% / 3% | Smallest net ceiling (`4–5.7%`); only worth rewriting as a segmented single-pass consumer for $L$-lane `E`. |

### 2.2 Estimated Capture Share of the Top Three Ceilings (in Referee Currencies $Q, D, P, S$)

1. **Construction `C` (T2) — Measured ceiling: `42.1%` at $k = 1$ (`44.4%` net = `577.7 ms` in `character-stage1/report.txt` L32: `A6a 200.3 + A6b 216.0 + A6c 161.4 ms`); `33.9%` behind `S` at $k = 8$:**
   - **C1. `A5 + A6a` deforestation (direct symbol-to-`OutQ16` fold, eliminating `CAP = 127` `Runs`):**
     - Currencies: $\Delta Q = 0$, $\Delta D = 0$, $\Delta P = -1$ full streaming pass (`−360 MB` read+write of `runs: Vec<u16>`, **`[PROVED]`**), $\Delta S = -180\text{ KB/worker}$ (`[PROVED]`).
     - Ceiling share: eliminates `~45–60%` of `A6a` (`90–120 ms` at $k = 1$) + `~15–20%` of `A5` (`20–26 ms`) = **`~16–21%` of `C`'s ceiling** (plus `~20%` of `M`'s; `~8.5–11.2%` of worker CPU; `~7–9%` of wall behind `S` at $k = 8$) **`[ARGUED]`**.
   - **C2. Zero-RMW unidirectional `A6c` (`Reader::wire`, `src/open/outq.rs:327`) paired with $L$-lane forward `E`:**
     - Currencies: $\Delta Q = -m = -106.3\text{M}$ scattered random L3 RMWs (`-2.1` to `-3.3 h₁`, **`[PROVED]`**), $\Delta P = -1.28\text{ GB}$ (`edges.fill(0)` `425 MB` + `m` scattered RMWs `850 MB`, **`[PROVED]`**), $\Delta D = 0$, $\Delta S = 0$.
     - Ceiling share: cuts `A6c` from `161.4 ms` to `~35–45 ms` (`−72%` to `−78%`) = **`~20–22%` of `C`'s ceiling** (`~9–10%` of worker CPU; `~8–10%` behind `S` at $k = 8$, while removing `106.3M` L3 RMWs from the shared ring bus) **`[ARGUED]`**. *(Fallback if `E` stays 2-lane XOR: 16-step lookahead prefetch on `OutQ16`'s `v` stream captures `~8–11%` of `C`'s ceiling `[ARGUED]`.)*
   - **C3. `A6b` dictionary streamlining (direct `(rule_id, byte)` indexing without content-hash dedup):**
     - Currencies: removes `~3k–15k`/block double-hash probes (`pairs` + `contents`) and recursive `inline_word` dedup expansions; $\Delta S \approx +8\text{–}16\text{ KB/block}$ in `inline16` (still `< 50 KB`, L2-resident).
     - Ceiling share: `A6b` spends `~103` cycles/IET-step (`216 ms` over `~10.3M` steps); if dictionary/expansion is `50–65%` of `A6b`, captures **`~12–18%` of `C`'s ceiling** (`~5–8%` of worker CPU) **`[CONJECTURED]`** pending sub-phase timing.
   - **Combined `C` capture:** **`~48–61%` of `C`'s measured ceiling** (`~22–28%` of worker CPU at $k = 1$; `~17–21%` behind `S` at $k = 8$) `[ARGUED / CONJECTURED]`.

2. **Huffman Scan `S` (T4) — Measured ceiling: `8.1%` alone at $k = 10$; `32.1%` behind `C` at $k = 8$:**
   - **Serial LUT arithmetic:** Spent (`≤ 6–8%` of scanner time left, §82) **`[ARGUED]`**.
   - **S1. Cross-block magic-indexed parallel Huffman in a unified bounded worker pool ($k = 8\text{–}9$):**
     - Currencies: $\Delta Q_{\text{L1}} = 0$ (`0.54 M` LUT lookups spread across $k$ cores); $\Delta D$: serial depth drops **$k\times$** (`0.54 M → 0.54 M / k` plus `~1–2 ms` SIMD magic scan on `47.4 MB`, **`[PROVED]`**); $\Delta P$: `+47.4 MB` magic scan minus `400.6 MB` inter-thread `syms` queue write+read = **net `−353 MB` (`−0.42 B/out-byte`)** **`[PROVED]`**; $\Delta S$: `+(k − 1) × 54 KiB` (`~0.45 MB`) + `~2–4 MB` compressed window minus pooled `syms` buffers (net lower RSS).
     - **Realistic capture accounting for CPU conservation (unlike `scan-oracle`, pooling moves S's `173 ms` onto workers rather than deleting it, while freeing core 5):**
       - *Standalone (on shipped workers, $k = 8\text{–}9$):* Total CPU `(1,478 + 173 − 12) = 1,639 ms` across `~9.5` effective cores (avoiding $k = 10$'s L3 spill) gives **`~172–175 ms`** (`−10% to −12%` vs pinned `195 ms`), capturing **`~100%` of `S`'s standalone `8.1%` ceiling at $k = 10$** and beating `S alone` at $k = 8$ (`188.8 ms`) because it recovers the 10th physical core **`[ARGUED]`**.
       - *Jointly with `C`'s `22–28%` capture (`1,224–1,314 ms` total CPU over `~8.7–9.5` cores):* **`~130–145 ms`** (`0.67–0.74` of `195 ms`), capturing **`~70–85%` of `S+C`'s `35.9%` measured room (`124.9 ms`) at $k = 8$** **`[ARGUED]`**.

3. **Emission `E` (T1) — Two ceilings: `16.5%` chase (`supplied-codes` at $k = 1$; `13.4%` behind `S` at $k = 8$) PLUS `19.9%` unremoved word-spelling (`self.consume = 259.4 ms` at $k = 1$; `~19%` at $k = 8$):**
   - **E1. $L$-lane forward-only emission (`L = 8` active lanes from `K = 32` anchors) + fast forward spelling + segmented RLE1/CRC:**
     - Currencies: $\Delta Q = 0$ (`m = 2.1–3.3 h₁` reads of `edges`, inside G1); $\Delta D$: chase depth drops `3–4×` (`m/2 → m/L`, MLP `2 → 6.5–7.2` with `K = 4L` work-stealing anchors, **`[PROVED]`** for arc-length order statistics / **`[ARGUED]`** for L3 MLP); $\Delta P = 0$ extra passes when fused with segmented `A9` (`[CONJECTURED]`); $\Delta S \approx +1.0\text{–}1.5\text{ MB/worker}$ in 64 KiB slab vectors.
     - Ceiling share: captures **`~50–65%` of the chase ceiling** (`110–140 ms` at $k = 1$) + **`~20–30%` of the unremoved `consume` spelling ceiling** (`52–78 ms` by deleting backward `u128` shifts and `hi − lo` guards) = **`~34–46%` of full `A7`** (`162–218 ms` = `~12–17%` of worker CPU), **plus** unlocks C2's `~120 ms` in `A6c` (`~21–26%` of worker CPU combined!) **`[ARGUED]`**.
   - **E2. Bounded-length ($\le 16$ B) 2-word inline fusion (`CL1` closure-respecting contraction):** Cuts $Q$ and `consume` word count $m$ by `~20–35%` while capping $S_{\text{pay}} \le 48\text{ KB}$ (avoiding `CL1`/`M-17`'s `2.5 MB` dictionary blow-up); captures **`~15–25%` of full `A7`** (`~5–8%` of worker CPU) **`[CONJECTURED]`**, lower priority than E1 because pair detection adds construction work to `C`.

---

## 3. The Floor: Residual Arithmetic, Cheapest Zero-Copy Oracle, and Mathematical Content

### 3.1 Critical Audit of `hand_E` and Residual Arithmetic From Existing Data

Reading `origin/bench/ideal-parts:src/open/supplied.rs:93-117` exposes a crucial accounting fact that `results/full/README.md` §4 conflates: **`HandE` wraps BOTH `sup::deliver` (the `404 MB` oracle code copy) AND `self.consume` (all of word spelling — `inline16` lookup, `u128` backward shifts, and `838.8 MB` of output stores)!**

`four-arm-runs.tsv` (arm B) measures the two pieces separately on the exact same `wikidata-x4.bz2` corpus:
- At $k = 1$: `deliver = 25.8 ms` (`9.05%`), `consume = 259.4 ms` (`90.95%`), sum `285.2 ms` — matching `ideal-matrix.tsv` L260 `hand_E = 285.55 ms` to `0.1%`.
- At $k = 6$: `deliver = 31.7 ms` (`5.3 ms/thr`, `9.96%`), `consume = 286.6 ms` (`47.8 ms/thr`, `90.04%`), sum `318.3 ms` (`53.0 ms/thr`) — matching `ideal-matrix.tsv` L270 `hand_E = 52.80 ms/thr`.
- At $k = 10$: `deliver = 39.7 ms` (`4.0 ms/thr`, `9.61%`), `consume = 373.4 ms` (`37.3 ms/thr`, `90.39%`), sum `413.1 ms` (in `ideal-matrix.tsv` L290, `hand_E = 38.08 ms/thr` at pinned 4.91 GHz ⇒ `deliver ≈ 3.7 ms/thr`, `consume ≈ 34.4 ms/thr`; at $k = 8$, `10.0%` gives `deliver ≈ 4.2 ms/thr`, `consume ≈ 37.6 ms/thr`).

Furthermore, `results/full/README.md` §4 summed `S + E + C + M + R = 392.6 ms` at $k = 1$, placing `S` (`15.3 ms`) in *series* with the worker (`377.3 ms`). Because `S` runs concurrently on the scanner thread, the critical-path handover sum at $k = 1$ is $\max(h_S,\; h_E + h_C + h_M + h_R) = 377.3\text{ ms}$, leaving **`391.0 − 377.3 = 13.7 ms` (`1.05%` of `1,306 ms`)** of pure non-handover, non-consume infrastructure!

Putting every measured wall (`results/full/README.md` §4), per-thread handover (`ideal-matrix.tsv` L258–296), `deliver`/`consume` split (`four-arm-runs.tsv`), and stage-1 plumbing (`character-stage1/report.txt` L32–41, 56, 249–258) together yields the complete floor decomposition:

| Quantity (ms, per decode / per thread) | $k = 1$ | $k = 6$ | $k = 8$ | $k = 10$ | Source / arithmetic |
|---|---:|---:|---:|---:|---|
| All-real wall | `1306.0` | `243.0` | `195.0` | `196.0` | `results/full/README.md` §4 |
| **Measured `all-ideal` (`SECMRD`) wall** | **`391.0`** | **`94.0`** | **`112.0`** | **`142.0`** | `results/full/README.md` §4 (`floor = 0.299 / 0.387 / 0.578 / 0.721`) |
| `hand_S` (scanner thread, concurrent) | `15.3` | `20.3` | `24.9` | `33.6` | `ideal-matrix.tsv` L258, 268, 278, 288 (**2.2× bus-contention thermometer!**) |
| Worker `hand_E = deliver + consume` | `25.8 + 259.7` | `5.3 + 47.5` | `4.2 + 37.6` | `3.7 + 34.4` | `ideal-matrix.tsv` L260–290 split by `four-arm-runs.tsv` arm B |
| Worker pure copies (`deliver_E + C + M + R`) | `117.6` | `23.9` | `21.4` | `21.6` | `deliver_E + h_C(30.2/6.1/5.0/4.5) + h_M(9.2/1.9/1.8/1.8) + h_R(52.4/10.6/10.4/11.7)` |
| Total worker `hand_(E+C+M+R)` (single-arm) | `377.3` | `71.5` | `59.0` | `56.0` | `consume + worker pure copies` |
| **Excess wall = `all-ideal − worker hand_total`** | **`+13.7`** | **`+22.5`** | **`+53.0`** | **`+86.0`** | Plumbing + simultaneous-copy DRAM/lock inflation |
| **Est. Residual B (pure zero-copy plumbing)** | **`~13.7` (`1.0%`)** | **`~12–15` (`5–6%`)** | **`~14–17` (`7–9%`)** | **`~15–19` (`8–10%`)** | `max(A0 10–14, scan_issue 3–5, wkr_plumb+minflt 4–9) + startup/drain 4–5 ms` (`stage1/report.txt`) `[ARGUED]` |
| **Est. Residual A (`consume` + plumbing, zero-copy oracles)** | **`~273` (`0.209`)** | **`~60–62` (`0.25`)** | **`~51–55` (`0.27`)** | **`~50–54` (`0.27`)** | `consume + Residual B` (matches Pass 1's `0.245–0.33` zero-copy floor!) `[ARGUED]` |
| **Implied simultaneous-copy DRAM + `RwLock` inflation** | **`~0`** | **`~8–11`** | **`~36–39`** | **`~67–71`** | `all-ideal − (Residual A + single-arm worker copies)` `[ARGUED]` |

**Cross-check of the `+39 / +69 ms` artificial inflation at $k = 8, 10$:** In `SECMRD`, the five copy handovers execute back-to-back with no compute between them, generating `0.40 (S) + 0.26 (M) + 0.91 (C) + 0.81 (E deliver) + 0.84 (E consume writes) + 1.68 (R) ≈ 4.9 GB` of bus traffic (`~5.7 GB` with L3 write-allocate spills at $k = 10$, including `~2.03 GB` cold reads from `495 MiB` stores), PLUS every block on every worker hammers four global `RwLock`s (`src/scan_oracle.rs:196`, `src/ideal/{mtf.rs:252,construction.rs:323,rle1.rs:122}`, and `src/supplied_codes.rs:166`, which holds `STORE.read()` **across the entire `copy_from_slice`**, plus `RECS.lock()` at `supplied_codes.rs:249`). At `~38–42 GB/s` dual-channel DDR4, `4.9–5.7 GB` takes `~115–145 ms` — independently verified by `floor share R = 0.841 / 0.888` (`−17.8 / −15.9 ms` when `R`'s `1.68 GB` copy is removed!) and `floor share E = 0.914` at $k = 10$ (`−12.3 ms` when `E`'s `0.81 GB` cold copy + `RwLock` is removed!).

### 3.2 Cheapest Oracle Change to Measure the Floor Directly (~120 LOC, Zero Production Impact)

Four surgical `#[cfg(feature = "ideal-*")]` edits eliminate `> 85%` of copy traffic and `100%` of per-block oracle lock contention without touching any production code path:

1. **Freeze stores lock-free after `Record` (`~30 LOC` across `scan_oracle.rs`, `supplied_codes.rs`, `ideal/{mtf,construction,rle1}.rs`):** Snapshot each `Store` into an immutable `Arc<Store>` (or `OnceLock<&'static Store>`) when switching to `Replay` and clone the `Arc` once per worker thread at spawn; gate `supplied_codes::record()` (`RECS.lock()`) off during timed matrix decodes. Cost: `0` atomic ops per block.
2. **Zero-copy `R` via `BlockBuf` (`~25 LOC` in `src/ideal/rle1.rs` + `src/parallel.rs`):** Under `ideal-rle1`, change the ready-queue payload from `Vec<u8>` to `enum BlockBuf { Owned(Vec<u8>), Shared(Arc<[u8]>) }` (implementing `Deref<Target = [u8]>` so `read()` and `ideal::delivery::Mode::Discard` are unchanged). `rle1::replay_replace` swaps in `BlockBuf::Shared(Arc::clone(&entry.bytes))` — deleting **`1.68 GB`** of cold DRAM copy.
3. **Zero-copy borrowed `C` (`~30 LOC` in `src/ideal/construction.rs` + `src/open.rs`):** Pre-populate the 256 literal slots inside `Built::{inline16, lens}` during `Record`, and expose `Kernel::walk_slices(&[u32], &[u32], &[u8], &[[u8; 16]], &[u8], ...)` called directly on `&Built` during `Replay` — deleting **`0.91 GB`** of copy traffic (`Kernel::walk` never mutates those slices).
4. **Zero-copy borrowed `E` + split `HandE` + optional `E_full` floor mode (`~35 LOC` in `src/supplied_codes.rs` + `src/open/supplied.rs`):**
   - Pass `&[u32]` from the frozen store straight into `emit_codes` / `emit_bidirectional_codes` instead of copying `404 MB` into `self.edges[..L]` (deleting **`0.81 GB`** of copy traffic).
   - Split `HandE` (`src/open/supplied.rs:93-117`) into `HandE_deliver` and `E_consume` so `consume` is never conflated with oracle handover.
   - Add an `E_full` switch (floor arm only) that skips `self.consume` when `R` is also zero-copy ideal (leaving `out` untouched). Running just **4 arms** (`none`, `SE(chase)CMRD`, `SE(full)CMRD`, `SE(full)CMR`) measures **Residual A** (`~51 ms`), **Residual B** (`~14–19 ms`), and true unconfounded **`D` share of the floor** directly in **~4 minutes**.

*(Optional 5th edit: `Cow<'static, [u16]>` for `S` and `M` deletes the remaining `0.66 GB`, making every arm of a partial matrix zero-copy too.)*

### 3.3 Is Anything in the Residual Mathematical Rather Than Engineering?

Split the zero-copy floor into its two halves:
- **Residual B (`~14–19 ms` pure plumbing: `A0` feeder, `issue`/`publish` channels, `A8` on 192 tiny column blocks, startup/drain, minor page faults):** **100% engineering / OS runtime** (buffer pre-faulting, lock-free queues, zero-copy I/O). Nothing mathematical.
- **The `self.consume` slice of Residual A (`259.4 ms` at $k = 1$, `~34–38 ms/thr` at $k = 8\text{–}10$):** **Part mathematical, part engineering, and bounded by a physical write floor:**
  1. *Mathematical component (Lever 5 — retained-set cardinality and closure):* `consume` executes $m = 106.3\text{M}$ dictionary lookups and variable-length stores (`mean word = 7.89 B`), where $m = 2.1\text{–}3.3\,h_1$ is set by Open's retained set (`T1-T2-LAZY-ACQ-AND-DEPTH2.md` §3). Cutting $m$ toward $h_1$ without blowing `inline16` out of L2 is our `CL1`/`M-17` **closure-preserving contraction** problem.
  2. *Engineering component:* At fixed $m$, `consume` pays `~2.44 ns/word` (`~12.0` cycles/word at 4.91 GHz) vs `~1.0–1.5` cycles/word (`~30–45 ms` at $k = 1$) because of backward `u128` dynamic shifting (`src/open.rs:1035`) and `hi − lo >= INLINE` loop checks — removable purely by forward-only $L$-lane emit or right-aligned backward tables (`[ARGUED]`).
  3. *Physical floor `[PROVED]`*: Writing `838.8 MB` of output once across 8 cores is floored by memory write bandwidth at `~21 ms` (DRAM-cold at `40 GB/s`) or `~8–12 ms` (L3-warm into a zero-copy caller).

---

## 4. S and C Together: Single Reformulation vs Separate Attacks

The matrix proves that winning at $k = 8$ requires making **both** `S` and `C` cheaper (`S alone = 0.968`, `C alone = 0.944`, `S+C = 0.641`, `q_SC = −0.0705`). Can a single reformulation — specifically cross-stage enrichment across the scanner/worker boundary (target **T5**) — change both at once?

**Answer: Classical T5 enrichment across the *serial* scanner/worker boundary is mathematically obstructed in both directions `[PROVED]`. The one structural move that touches both is *dissolving* that boundary (cross-block parallel scan), which unlocks local `S → M → A6a` deforestation — but `A6b` and `A6c` (`65%` of `C`) MUST still be attacked separately post-EOB.**

1. **Why pushing `C` upstream into the serial scanner fails (`[PROVED]`):**
   - *Queueing obstruction:* At $k = 8$, `S` (`182–184 ms`) is already on the critical path; adding *any* branching RLE2, MTF, or counting logic to `src/block.rs` strictly slows $T_{\text{scan}}$ (already witnessed by the 22 Nov fused Huffman+MTF kill: `74.5 → 107.9 ms`, `1.185×` whole decoder, `tracks/cross-stage.md`).
   - *Theorem T5.1 (Pre-EOB Causality Obstruction, `T5-CROSS-STAGE-HUFFMAN-ENRICHMENT.md` §2 `[PROVED]`):* Flipping the final byte `L[n − 1]` of a block shifts `first[b]` and exact `LF(x)` targets on **`15.4%–50.0%`** of all rows across all 6 Wikidata blocks. Therefore **zero** work of `A6b` (`first[256]`, thick-strip sorting, IET, `Dictionary::code`) or `A6c` (`wire_xor`) can be computed before `EOB`.
   - *Theorem T5.2 (Pre-MTF Equivariance Obstruction, `T5-CROSS-STAGE-HUFFMAN-ENRICHMENT.md` §2 `[PROVED]`):* Permuting `seq_to_unseq` with bit-identical `syms` shifts `C[L[x]]` on **`100.0%`** of rows. Without running MTF, the Huffman scan determines `100%` of run lengths and `0%` of byte identities or F-buckets.
2. **What dissolving the boundary (pulling `S` into the worker pool) *does* change jointly (`[ARGUED]`):**
   - Moving `S` into $k = 8\text{–}9$ end-to-end workers (Proposal 2 below) **both** removes `S`'s `184 ms` serial wall **and** deletes the `200.3 MB` inter-thread `syms` queue (`400.6 MB` traffic).
   - Once Huffman runs locally inside each worker thread (no longer on a serial critical path!), fusing `huff → M (A5) → C Pass 1 (A6a)` inside L1/L2 cache becomes safe — eliminating both `syms: Vec<u16>` and `runs: Vec<u16>` (`~760 MB` combined intermediate traffic `[PROVED]`). Even without touching `huff.rs`, decoding `syms` hot into the worker's own L2 makes `A5 + A6a` immediately cache-warm.
3. **Why `A6b` and `A6c` must still be attacked separately (`[PROVED]`):**
   - Even with full `S → M → A6a` fusion, **`A6b` (`216.0 ms`, `37.4%` of `C`)** and **`A6c` (`161.4 ms`, `27.9%` of `C`)** sit strictly downstream of `EOB` (Theorem T5.1). They require worker-internal reformulations (unidirectional zero-RMW `A6c` paired with $L$-lane `E`, plus `A6b` dictionary streamlining) that are mathematically orthogonal to `S`.

---

## 5. One Concrete Proposal Per Top Target (Design Only; No Patches)

### Proposal 1 — Top Target `C` (Construction, T2): Deforesting the `CAP = 127` Interface (`A5 + A6a` Direct Fold into `OutQ16`)

- **Mathematical object changed:** Replace the intermediate capped-run free monoid $\text{Run}_{127}^* = (\Sigma \times \{1,\dots,127\})^*$ — which splits maximal runs at `CAP = 127` (`+3,990–5,284` extra records/block, `3.5–10.2%` of $r$, `T5-CROSS-STAGE-HUFFMAN-ENRICHMENT.md` §1) only for `open::maximal` to immediately re-coalesce them — with the direct homomorphism
  $$\Phi: \text{Sym}^* \longrightarrow (\text{thin}[256],\, \text{thick}[256],\, \text{thick\_rows}[256],\, \text{ThickStrips},\, \text{OutQ16})$$
  folding each maximal `(byte, total_len)` emitted by the `lit`/`one` bitmap MTF loop directly into thick strip descriptors (`total_len > 16`, splitting the single run containing `orig_ptr`) or `OutQ16` thin-rank chunks (`total_len <= 16`).
- **What it replaces in `src/`:** Replaces the two-pass sequence `runs::front_avx2` (`src/runs.rs:436-690`) $\to$ `open::maximal` (`src/open.rs:119-145`) + Pass 1 of `Kernel::construct_capped` (`src/open.rs:350-445`) on the Open run-records path (`src/lib.rs:614-652`). Keeps `runs::front` strictly as the fallback if Open admission (`Cap::dense`) trips (`0/932` full wikidata blocks refuse).
- **Cost in $(Q, D, P, S)$:** $\Delta Q = 0$; $\Delta D = 0$ (same serial MTF permutation chain); $\Delta P = -1$ streaming pass (`−360 MB` read+write of `runs: Vec<u16>`, **`[PROVED]`**); $\Delta S = -180\text{ KB/worker}$ (`[PROVED]`). Predicted saving: `~90–120 ms` in `A6a` + `~20–26 ms` in `A5` = `~110–145 ms` at $k = 1$ (`~8.5–11%` of worker CPU; `~7–9%` behind `S` at $k = 8$) **`[ARGUED]`**. Risk: branch mispredicts inside the MTF loop on `total_len <= 16` (mitigated because `total_len == 1` is `83–85%` in hex groups and `total_len > 16` is `98%` in template groups, `T5-CROSS-STAGE-HUFFMAN-ENRICHMENT.md` §1).
- **Smallest measurement that kills it fastest:** A **10-minute $k = 1$ micro-profile** on wikidata ×4 splitting `A6a` (`200.3 ms`) into `maximal() + counting` vs `OutQ16::push`. **Kill rule:** if `OutQ16::push` is `> 70%` of `A6a` (`> 140 ms`) and `maximal()` + `CAP = 127` chop/merge is `< 50 ms` (`< 3.8%` of $k = 1$ wall), kill the deforestation immediately and pivot `C` effort straight to `A6c` (Proposal 3) and `A6b` (simultaneously timed for `setup` vs `IET` vs `Dictionary::code`).

### Proposal 2 — Top Target `S` (Huffman Scan, T4): Cross-Block Magic-Indexed Parallel Scan in a Unified Worker Pool

- **Mathematical object changed:** Replace the global serial stream automaton $\mathcal{A}_{\text{stream}}: \text{BitStream} \to \text{Symbols}^J$ (serial depth $D = \sum_{j=1}^J 0.54 M_j$) with a two-stage decomposition:
  1. A data-parallel SIMD locator scanning all 8 bit alignments of the compressed input for the 48-bit `BLOCK_MAGIC` (`0x314159265359`, depth $O(|B|/W_{\text{SIMD}}) \approx 1\text{–}2\text{ ms}$ on `47.4 MB`, false-positive probability $\le 8|B|\cdot 2^{-48} \approx 1.3\times 10^{-6}$ **`[PROVED]`**), producing candidate block start bit-offsets $\{b_0^{(j)}\}$;
  2. $k$ independent per-block Huffman automata $\mathcal{A}_{\text{blk}}(b_0^{(j)})$ run directly on the worker threads, certified exact by the deterministic boundary chain $b_{\text{end}}^{(j)} = b_0^{(j+1)}$ (serial continuation fallback from $b_{\text{end}}^{(j)}$ on any mismatch).
- **What it replaces in `src/`:** Replaces the dedicated `scanner()` thread (`src/parallel.rs:1089-1255`), `ChannelSource` seam maintenance (`maint` `10–12 ms`), and the `Symbols` inter-thread queue (`st.jobs` / `free_syms`). A lightweight chunked feeder supplies bounded compressed windows (`~2–4 MB` lookahead) + magic bit-offsets to **`k = 8–9`** end-to-end workers (capped below $k = 10$ to stay inside the `20 MiB` L3 and avoid SMT thrashing!), each owning a `54 KiB` `ScanScratch` and executing `scan → M → C → E → R` locally before publishing to `st.ready`.
- **Cost in $(Q, D, P, S)$:** $\Delta Q_{\text{L1}} = 0$ (distributed across $k$ cores); $\Delta D$: **$k\times$ reduction** (`~183 ms → ~20 ms` effective serial overhead + `~1–2 ms` magic scan, **`[PROVED]`**); $\Delta P$: `+47.4 MB` magic scan minus `400.6 MB` `syms` queue write+read = **net `−353 MB`** **`[PROVED]`**; $\Delta S$: `+(k − 1) × 54 KiB` (`~0.45 MB`) + `~2–4 MB` compressed window minus pooled `syms` buffers (net lower peak RSS than `C-37`). Predicted wall on `wikidata-x4.bz2`: `~172–175 ms` at $k = 8\text{–}9$ standalone (`−10–12%` vs pinned `195 ms`), and **`~130–145 ms`** combined with `C` improvements **`[ARGUED]`**.
- **Smallest measurement that kills it fastest:** (1) **5-minute offline SIMD magic-scan timing** on `47.4 MB` `wikidata-x4.bz2`: kill if bit-aligned scan takes `> 10 ms` single-threaded; (2) **Minimal prototype at $k = 8, 9$ (spun, peak RSS checked)**: kill if wall fails to beat shipped pinned `195 ms` by `≥ 7%` (`> 182 ms`), or peak RSS exceeds `75 MB` (regression into `C-37`'s memory regime).

### Proposal 3 — Top Target `E` (Emission, T1) *Jointly with `C` (`A6c`)*: $L$-Lane Forward Emission on Unidirectional `edges` with Work-Stealing Anchors + Segmented RLE1/CRC

- **Mathematical object changed:** Replace the involutive XOR-linked cycle `edges[j] = (code << bits) | (prev ^ next)` (which forces $m = 106.3\text{M}$ scattered L3 RMWs in `A6c` and limits `A7` to MLP $= 2$ with backward `u128` shifts) with the **unidirectional functional graph** `edges[j] = (code << bits) | next` (built strictly sequentially with `Q = 0` random accesses!) partitioned into $K$ cycle arcs by $K$ uniformly sampled anchor vertices $a_0 = \text{mark}, a_1, \dots, a_{K-1}$ (marked via a 1-bit tag in `edges` or a small sorted anchor table).
  - **Crucial order-statistic theorem (`[PROVED]`):** On a single cycle of length $m$, $L$ uniform random anchors yield expected longest arc $\mathbb{E}[\max_i \ell_i] = \frac{H_L}{L}\,m$ (Dirichlet/exponential spacing order statistics). For naive $L = 8$, $H_8 / 8 = 0.340\,m$ (`2.72×` average!), collapsing effective MLP from `8` to `~2.9`. Choosing **$K = 4L = 32$ anchors** drops the longest arc to $H_{32}/32 = 0.127\,m$ (`~1.01 × m/8`!), and running a **work-stealing pool of $L = 8$ active forward lanes** (when any lane hits an anchor, it immediately starts the next unvisited anchor) lifts effective MLP from `2` to **`~6.5–7.2`** while walking strictly forward (zero `u128` backward shifts!). The $K$ small forward segments are then stitched in anchor-successor order by a single-pass segmented RLE1+CRC consumer.
- **What it replaces in `src/`:** Replaces `Reader::wire_xor` (`src/open/outq.rs:241-325`) with existing sequential `Reader::wire` (`src/open/outq.rs:327-344`); replaces `emit_bidirectional` (`src/open.rs:991-1068`) with `emit_lanes::<8>` (using 64 KiB slab vectors per active arc to bound peak scratch); and replaces `rle::unrle_in_place` (`src/rle.rs`, `src/lib.rs:672`) with a segmented `unrle_segments_crc(&[&[u8]], out)`.
- **Cost in $(Q, D, P, S)$:**
  - `C` (`A6c`): $\Delta Q = -106.3\text{M}$ scattered L3 RMWs (`[PROVED]`), $\Delta P = -1.28\text{ GB}$ (`edges.fill(0)` + RMWs, `[PROVED]`).
  - `E` (`A7`): $\Delta Q = 0$ (`m` reads of `edges`); $\Delta D$: chase depth drops **`~3.3–3.6×`** (`m/2 → ~0.14 m`, MLP `2 → ~6.5–7.2`, **`[ARGUED]`**); eliminates backward `u128` shifts and `hi − lo` guards in `consume` (`[PROVED]`).
  - `R` (`A9`): $\Delta P \le 0$ (`unrle_segments_crc` replaces `unrle_in_place`'s `copy_within` + forward scan: `2n` instead of `~3–5n`, **`[ARGUED]`**).
  - $\Delta S$: `~1.0–1.5 MB/worker` of slab buffers (total across $K$ arcs bounded by $n + K \cdot 64\text{ KiB}$).
  - Combined capture: `~120 ms` (`A6c`) + `~110–140 ms` (`A7` chase) + `~52–78 ms` (`A7` spelling) = **`~280–340 ms` at $k = 1$ (`~21–26%` of worker CPU; `~16–20%` behind `S` at $k = 8$)** **`[ARGUED]`**.
- **Smallest measurement that kills it fastest:** A **50-line standalone $k = 1$ harness** on the 932 Open blocks of `wikidata-x4` comparing `(wire_xor + emit_bidirectional + unrle_in_place)` (`728 ms` at $k = 1$) against `(wire + emit_lanes::<8> [K=32] + segmented stitch/unrle)`. **Kill rule:** if the new path does not save **`≥ 190 ms` at $k = 1$ (`≥ 15%` of worker time)** — e.g., because `456 KB` `edges` saturates L3 bandwidth by $L = 4$ or slab-management eats the `A6c` win — kill segmented multi-lane immediately and fall back to 16-step lookahead prefetch on `wire_xor` (zero `E`/`R` changes).

---

## 6. What the Matrix Could Not See, and the Next Characterisation Run

### 6.1 Five Blind Spots of `ideal-matrix/run-20261125T125334Z`

1. **The true decoder floor (Residuals A and B):** Buried under `~4.9–5.7 GB` of simultaneous oracle copies and 4 global `RwLock`s (`+39 ms` at $k = 8$, `+69 ms` at $k = 10$), with `hand_E` conflating `deliver` (`~10%`) and unremoved `consume` (`~90%`).
2. **The internal split of `C` (`A6a` vs `A6b` vs `A6c`) and of `E` (chase vs `consume` spelling):** Binary `C` could not distinguish how much of `C`'s `42%` room (`34%` behind `S` at $k = 8$) or its `~10 ms` worker-to-scanner L3 tax comes from `A6a`, `A6b`, or `A6c`'s `106.3M` scattered RMWs; binary `E` never separated chase from `consume`.
3. **The non-oracle L3-spill transition at $k \in \{7, 8, 9, 10\}$ and pooled-scan capacity:** The matrix skipped $k = 7, 9$ (straddling the `20 MiB` L3 spill boundary and SMT onset), kept the dedicated serial scanner core, and deleted `S`'s CPU rather than pooling it — so it could not separate *real* L3/DRAM saturation in `S+C` from `C`'s `114 MiB` cold store copy.
4. **Corpus generality:** All 2,492 decodes ran on one 4× repeated corpus (`wikidata-x4.bz2`, `932/1,724` Open blocks, `7.9 B` mean word, `30–47%` `RUNA/B`). Prose, source/tarball, and high-entropy/binary corpora shift `h₁/n`, Open admission rate, and `S : C : E` shares.
5. **Physical cache-warming interaction between `C` and `E`:** Because `ideal::construction` copies `edges` from a `114 MiB` cold store right before `E`, it confounds the true `~1–2%` L2/L3 cache-warming synergy of real `A6c → A7`.

### 6.2 Next Characterisation Run (Minimal, ~7 Minutes Total)

Do **not** re-run a 4-hour 44-arm matrix. Two surgical runs close all five blind spots:

1. **`bench-zero-copy-floor` (~5 min on the i9-10900K):** Apply the ~120-LOC zero-copy `OnceLock` + `BlockBuf::Shared` (`R`) + borrowed `C` + borrowed & split `E` (`HandE_deliver` / `E_consume` / `E_full`) change from §3.2, and run **6 arms** (`none`, `S`, `SC`, `SE(chase)CMRD`, `SE(full)CMRD`, `SE(full)CMR`) across **$k \in \{1, 6, 7, 8, 9, 10\}$** with **hardware IMC read/write + LLC-miss counters** (`perf` uncore IMC). Directly measures Residuals A & B, unconfounded `SC` scaling across the L3 boundary ($k = 6 \dots 10$), and the true DRAM bandwidth budget left for `S+C` and `D`.
2. **Offline $k = 1$ sub-phase & multi-corpus probe (~2 min):** Time `A6a` (`maximal` vs `OutQ16`), `A6b` (`setup` vs `IET` vs `Dictionary::code`), and `A7` (`chase` vs backward `u128` shift vs forward `inline16` store) on `wikidata`, English prose, and a binary/source corpus — providing the exact kill-or-go numbers for Proposals 1, 3, and C3 before writing any production patch.

---

## 7. Optional Note (`notes-for-neon.md`: Helmer / Whitney Stratifications)

**Does Helmer bear on our answer here?** **No — not for any reformulation in this pass.** Proposals 1–3 are discrete/combinatorial algorithms, and our revised 3-station model $T_{\text{rev}} = \operatorname{smax}(T_{\text{scan}}, T_{\text{wkr}}, T_{\text{mem}})$ is piecewise-linear / max-plus with explicit closed-form regime boundaries (no high-codimension algebraic singularities requiring Gröbner bases or conormal spaces). Helmer would only become informative if a later design-space pass writes a smooth/rational MVA or soft-max surrogate over $\ge 4$ continuous tuning parameters (e.g., worker count $k$, lane count $L$, anchor ratio $K/L$, compressed lookahead window) and seeks a certified algebraic regime partition of that map.
