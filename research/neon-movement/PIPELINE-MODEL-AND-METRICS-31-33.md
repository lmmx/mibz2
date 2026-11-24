# Answers to Channel #31/#32 (Pipeline Model on Grid 1–20) and #33 (What to Measure vs Reason)

- **Prior art:** Tandem-line bottleneck analysis is operational analysis (Denning & Buzen 1978); finite-buffer blocking/starvation knees are standard tandem queueing theory; sublinear worker contention/coherency scaling is Gunther's Universal Scalability Law (USL); agrees with and complements MONODROMY Records 123–124.
- **Script & TSV:** [`pipeline_model_audit.py`](pipeline_model_audit.py) derives [`data/pipeline_model_four_arm_audit.tsv`](data/pipeline_model_four_arm_audit.tsv) directly from `a3f7d9ed:docs/journal/2026-11-24-algorithm-level/math-targets/data/measurements-four-arm.tsv`.

---

## 1. Channel #31/#32: Is `wall(k) = c + max(b, a/k)` the Right Model on a 10-Core/20-Thread i9 (Grid 1–20)?

**No — it is the right fluid skeleton, but fitting `c + max(b, a/k)` across `k = 1..20` is structurally misspecified and already fails quantitatively on the four-arm data.**

### What `wall(k) = c + max(b, a/k)` misses (and why it biases `k*`)
1. **Sublinear worker scaling *before* SMT (already measured in `measurements-four-arm.tsv`):**
   Fitting `W(k) = c + a/k` to Arm C (`k = 1, 6`: `a = 1.2816 s`, `c = 0.0290 s`) predicts `0.1572 s` at `k = 10` against measured **`0.1790 s` (+13.90% error)**; on Arm D it misses by **+23.58%** (`0.1355` vs `0.1675 s`). That happens *below* the 10-core SMT boundary because of all-core turbo droop (~5.2 → 4.8 GHz) and 20 MiB L3 cache / DDR4 contention across workers. Plugging Arm A's `k = 10` floor (`b = 0.197 s`) into `c + max(b, a/k)` puts `k* = 6.51` — off by ~2 workers (at `k = 6`, Arm C saves only `1.8%`!).
2. **The `k + 1` SMT kink at `k = 9 → 10` (scanner shares a physical core at `k ≥ 10`):**
   With 1 serial scanner + `k` workers, `k = 10` is already the **11th** active thread on a 10-core i9-10900K. Thus `b` is not constant — `b(k)` *rises* for `k ≥ 10` under SMT co-scheduling — and `W(k)` has a hard slope break across `k = 10..20` where SMT siblings add far less than a core (or go negative from L1/L2 thrashing). Fitting one smooth `a/k` across `1..20` lets the flat SMT tail (`11..20`) distort `(a, b, c)` and misplace `k*`.
3. **Bounded-queue rounding and per-block variance:**
   Wikidata blocks vary >3× in `r/n` and symbol count. Through a finite queue (`4` blocks/worker), block-time variance (`CV²`) rounds the sharp `max(b, W(k))` kink into a smooth knee (`~ CV² / Q_depth`), and finite-corpus pipeline fill/drain varies with `k` rather than staying constant `c`.
4. **Shift point ≠ raw crossover:**
   Switching to a parallel scanner costs an extra thread (on a fixed core budget, `(1, k)` vs `(2, k-1)` or `(2, k)` with SMT overhead) plus block-sync/memory overhead `δ_par` (which caused #741 to regress at `k = 6`).

### Recommended estimator & rounds on the fixed 1–20 grid
- **Do not fit `c + max(b, a/k)` globally across `1..20`.** Because `#838` already measures all 20 integer counts `k = 1..20` with per-stage timers (`S_busy(k)`, `W_busy(k)`, `W_idle(k)`):
  - **Direct non-parametric crossover:** Define per-round starvation gap $\Delta_k = S_{\text{busy}}(k) - \overline{W}_{\text{busy}}(k)$ (and worker idle fraction $f_{\text{idle}}(k)$). Estimate the raw crossover as $\hat{k}^* = \min \{ k \in \{1..20\} : \operatorname{median}(\Delta_k) > 0 \}$ and the Part 2 parallel-scan gate as $\hat{k}_{\text{gate}} = \min \{ k : \operatorname{median}(f_{\text{idle}}(k)) \ge \delta_{\text{par}} \}$, with a paired-round bootstrap 95% CI on $\hat{k}^*$.
  - **If a parametric check is wanted:** Fit **piecewise** on `k = 1..9` (physical cores) only, using Gunther USL $W(k) = W_1(1 + \alpha(k-1) + \beta k(k-1))/k$ and a soft-knee combiner $\big(S(k)^p + W(k)^p\big)^{1/p}$ (never extrapolate `11..20` back into `1..9`).
  - **Rounds for ±1 worker:** Near `k = 8..9`, $|d \ln W / dk| \approx 6.3\%$/worker. From four-arm's 20-round CIs (`σ ≈ 0.8%` at `k=6`, `5.6%` at `k=10`), pinning `k*` to ±1 worker (95% half-width `≤ 3.15%`) needs **`n ≥ 13` interleaved rounds** — so Pathfinder's **20 rotated rounds** on `1..20` easily pins `k*` to ±1 worker (provided per-block `CV` does not flatten the knee over >2 workers).

---

## 2. Channel #33: What to Measure vs What to Reason About

### What to measure (ordered by decision value for T1–T6)
1. **Worker phase split per block (`MTF/RLE2`, `Open construct`, `Open emit`, `RLE1/CRC`)** — fills the three unmeasured `targets.tsv` rows (`T2, T3, T6`) via boundary cycle counters + single-stage oracles (confirming cache-warming interaction).
2. **Q-to-seconds exchange rates across the 232 census blocks** — regress emit time & LLC misses on `(links, n, flat_dict_bytes)` (for T1, per MONODROMY 124) **and** regress construction time on `(n_strips, iet_steps, m₁₆)` (for T2), so candidates like `R⁴` (`-65%` links vs `+3.5 MB` flat build) can be screened before any Rust port.
3. **Per-thread busy/idle & LLC/IPC drop across `k = 8..12`** — separates `k = 9 → 10` SMT core-sharing from DDR4 bandwidth saturation on the 1–20 grid (`#838` already collects busy/idle + `getrusage`).
4. **Per-block variance alongside `(n, r, h₁, m₁₆)`** — quantifies how wide the bounded-queue knee is and tests whether T1/T2 routes regress on dense vs sparse blocks.
5. **Low priority for math (keep as adoption guardrail only):** time-averaged vs peak RSS (adoption gate `≤ 1.10×` for parallel scan; cache pressure per active worker is better computed analytically from buffer sizes against 20 MiB / `k`).

### What to reason about (do not spend workstation runs on)
1. **Exactness & algebraic equivalence** — proof + independent Python differential oracle (every T1/T2/T5 candidate).
2. **Referee screening inequalities before prototyping** — full `Q_total / h₁` (acquisition + build + emit + payload reads) and `ΔT = p_emit Δlinks - p_acq ΔQ_acq - p_byte ΔS_build` using the measured exchange rates.
3. **Cross-stage & pipeline combinations** — additive inside workers, `min`-rate across scanner and workers (no factorial oracle matrix over T1–T6).
4. **Structural obstructions & class floors** — G1 / Theorem O / Bayer–Diaconis rising-sequence bound / Theorems T5.1–T5.2 (Pre-EOB causality, Pre-MTF equivariance).
5. **Sample-size & shift-point precision** — delta-method / bootstrap power from per-round `σ` and slope $|d \ln W / dk|$.
