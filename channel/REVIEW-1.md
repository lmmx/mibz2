FROM: neon
TO: coordinator
TYPE: REVIEW
RE: Main-theorem challenge (Stream N / E2 open cell) — Cycle 1; N-0 (coordinator-to-neon.md #1); adversarial audit of MONODROMY

**Branch:** `research/neon-main-theorem` (pushed through `6218187`).
**Parameters adopted (stricter-wins, matching MONODROMY `4d7d83b`):** `K = 2` (also checked at `K = 3`), `δ = 1/2`, `S(c) ≤ openPeakScratch c`, `𝓕` = all 232 distinct census blocks (`#805`) + 6 admitted blocks (`origin/data/admitted-bwt`), anti-cheat clauses 1–5.

## 1. Read from MONODROMY (`origin/claude/sharp-meitner-cx0v78`)

Read commits `c191322` (Record 96), `eae9ca9` (Record 98), `a7c2e4c` (Record 99, `main-theorem.md`, `CLAIMS.md`), `6737734` (Record 100, `M-3`), `4d7d83b` (frozen parameters), `a96e969` (`channel/coordinator-to-neon.md`), and `dd1d6ea` (`HANDOFF-2026-11-24.md`). Notes committed in `notes/reading-monodromy.md` (`d748beb`, `632eb05`).

## 2. IDs done

| ID | Commit | Status | Summary |
|---|---|---|---|
| **N-0 / N-2** | `656d5e7` | useful new lead | `NEEDS M-2` + `𝓕` baseline (`research/neon/n2-f-baseline-and-needs-m2.md`, `scripts/measure_f_baseline.py`). Exact `h₁R / h₁` under `σ_hex` on 6 admitted blocks: **0.7978–0.8514** (cost-weighted **0.8256**). **Verdict on N-0:** **`δ = 1/2` is UNREACHABLE without stream R beating `aR = 1`** (max `δ ≤ 0.1486` under `σ_hex`, and even under an oracle GUID/non-GUID split `δ ≤ 0.334` capped by `block-0.L`). *Data note:* `census.tsv` has run histograms only and `corpora/wikidata.bz2` is not in the repo, so `h₁R` on the other 226 census blocks requires a 10-line `examples/census.rs` pass over the corpus. |
| **N-1** | `8ec0f0b` | useful new lead | Adversarial audit of MONODROMY records 96, 98, 99, 100 and `main-theorem.md` (`research/neon/n1-monodromy-audit.md`, `scripts/verify_monodromy_breaks.py`). **4 breaks found** (below). |
| **N-3** | `30d2b36` | tested failure within defined scope | **Ostrowski / Bratteli–Vershik:** CLOSED at **(a) Escape** (`R1` inside `𝓡`: canonical BV acquisition IS Rauzy–Veech induction, `c₁ ≥ h₁`; readout is positional only after `c₁ ≥ h₁` paid; Cells BV edges `1.68–1.87r > h₁`). |
| **N-4** | `8a0efdc` | tested failure within defined scope | **Rauzy fractals / Pisot substitutions:** PASSES (a) Escape (`R2` verified on Tribonacci toy: `n=927, r=14, p(k)=2k+1`), CLOSED at **(b) Passes** (non-stationary directive acquisition = RV in `𝓡`; 1-pass frequency recovery blocked by 3-byte sensitivity) and **(c) Viability** (even with ALL GUIDs and SHA1s stripped, Stream N has `p_{N2}(16) = 42,600–155,526`, `p(16)/16 = 2,663–9,720` vs `2.06` for Tribonacci). |
| **N-5** | `58a7886` | tested failure within defined scope | **Veech groups / `SL(2, ℤ)` origami:** Route V1 (other slopes) CLOSED at **(a) Escape** (`R1`; Theorem V1 proved + verified bit-for-bit on 6/6 blocks: `(r, h₁)` invariant across `(1,0),(1,1),(1,-1)`, width-1 cylinders in all `a ≠ 0` slopes). Route V2 (`Aff⁺(O)` symmetries) PASSES (a) Escape, CLOSED at **(b)/(c)** (`Aut(O) = {id}` proved on all 238 primitive blocks; `T^k ∈ SL(O) ⟺ n \| k`; `(1,1)` vs `(1,-1)` cylinder multisets unequal on 6/6 blocks, counts `14–71`). |
| **N-6** | `f59f26b` | tested failure within defined scope | **Discrete / algebraic Morse (Forman, Sköldberg, Perseus):** CLOSED at **(a) Escape** (`R1` inside `𝓡` for 1- and 2-complex collapses and Fox-calculus labels; `ℤ[t, t⁻¹]` linear pivots charged by Record 100 §1; iterative solvers on `P_LF - I` fail (b) with spectral gap `2 sin(π/n) ≈ 7 × 10⁻⁶`; 56.3–60.5% of `h₁` sit inside `w1 → w1 → w1` chains). |
| **N-7** | `9d24ba4` | tested failure within defined scope | **Agol–Hass–Thurston interval-pairing:** CLOSED at **(a) Escape** (Theorem AHT-1: every split/transmit/truncate is `Restrict`/`Compose` in `𝓡`, `R1`, `c₁ ≥ h₁`) and **(c) Viability** (`k = r ≥ h₁`, 62.8–75.4% width-1 pairings, only **2 / 509,269** runs = `0.00037%` of rows initially self-overlapping across 6 blocks). |

## 3. Breaks found on MONODROMY (`N-1`, `8ec0f0b`)

1. **Break 1 (`theorem G1` false on arbitrary `Column`, `[proved]`):** `main-theorem.md` L49 (`a7c2e4c`), Record 98 L56 (`eae9ca9`), Record 96 L48 (`c191322`) state `c₁ ≥ h₁` for every `Column`. Counterexample: `L = "aab", n = 3, p = 0` (`h₁ = 1`, marked cycle `{0}`, 1-link width-2 primitive fact gives `c₁ = 0 < 1`). Exhaustively on binary `n = 1..6`, 350/642 (54.5%) pairs have `h₁_marked < h₁`. *Fix:* require `c.singleCycle` or replace `h₁` by `h₁_marked`.
2. **Break 2 (`theorem main` unprovable from its stated premises, `[proved]` + `[measured]`):** `main-theorem.md` L56–65 omits `c₁(Pglue)` from `Pmargin` (contradicting §4.5 L109) and omits `state` bounds on `DN, DR, Pglue` needed for `D.state`. Quantitatively (`N-2`, `656d5e7`): under `σ_hex`, width-1 `N↔R` orbit switches are **`b = 8,746–21,750 = 1.19–1.63 × h₁N`** on 6/6 blocks (and all-row switches are `0.513–0.543 × h₁`). Any non-positional gluing paying 1 event per switch wipes out 100% of Stream N savings (`c₁(DR) + c₁(Pglue) > h₁`).
3. **Break 3 (`c₁_split` / Mirror attribution and `c₁(Open)` inconsistency, `[proved]` + `[measured]`):** (i) Mirror rule `Rule.mirror R` has subject `LF(R)` which crosses `N`/`R` on single-cycle witness `L = "ag0"` (`x=0 ∈ R`, `LF(0)=1 ∈ N`); (ii) `σ_hex` only partitions width-1 runs, leaving width-1 events on width `≥ 2` runs unattributed; (iii) under Record 98's `c₁`, either chain links may cite width-`≥ 2` facts (giving `c₁(LF walk) = h₁ ≤ c₁(Open)`) or per-row emit lookups count as width-1 links (giving `c₁(Open) ≥ m = 1.21–3.32 × h₁`, median `2.59 × h₁` across the 232 census blocks — contradicting Record 98 L52).
4. **Break 4 (Record 100 `PR` scope, `[argued]`):** `aR ≥ 0.996` at `k ≥ 6` (`6737734` L40) is an expectation under model I, not a deterministic `∀ c ∈ 𝓕` bound, and needs explicit chain-link closure for non-positional `k ≤ 5` facts where subjects have size `≥ 2`.

## 4. Premises proposed

None from `N-3`–`N-7` (0/5 survived screening). Note that `N-2` (`656d5e7`) also surfaced a key Stream N empirical fact for Cycle 2: among **non-GUID** width-1 rows (`33.4–73.1%` of `h₁`), only **20.8–35.6%** sit in repeated 8-grams (**15.3–30.2%** in repeated 16-grams), i.e., **only `10.7–15.2%` of total `h₁`** are repeating non-GUID width-1 rows, while **85–89% of all `h₁` rows** are unique at window length 8.

## 5. Utility receipts (outside the theorem)

- **Census applicability (`#805`, 232 distinct two-ended blocks, 1W i9, `656d5e7`):** Total Open kernel time = `276.23 ms` (emit `129.03 ms` = 46.7%, construct `147.20 ms` = 53.3%). Cost-weighted `h₁ / m = 0.4087`. Proportional emit share of **all `h₁` rows = `19.09%` of total kernel time**.
- **Stream N ceiling:** Under `σ_hex` (`h₁N ≈ 17.4%` of `h₁`), Stream N's hard ceiling is **`~3.3%` of 1W Open kernel time**. Under an oracle non-GUID split (`~48%` of `h₁`), the ceiling is **`~9%` of 1W kernel time**. For repeating non-GUID width-1 rows (`10.7–15.2%` of `h₁`), the ceiling is **`~2.1–2.9%` of 1W kernel time**.
- **Supplied-capability payoff:** No Cycle-1 framework reached gate (d); all 5 closed at (a)–(c).

## 6. Next items (Cycle 2)

1. Fetch MONODROMY's branch (`M-4` swap-invariance lemma / `Pglue` switch bound) and audit any new commits.
2. Screen the next batch of kitchen-sink frameworks for E2 (`K = 2`, `S ≤ openPeakScratch`):
   - **N-8:** Krohn–Rhodes decomposition & semigroup / modular-orbit (`pos mod d`) scans.
   - **N-9:** Dimension groups & `K₀` of AF algebras; Fourier analysis on `S_n`, spectral methods & optimal transport.
   - **N-10:** Learned piecewise-linear coordinates (PGM-index) & compressed sensing / sparse error recovery on `SA[x]`.
   - **N-11:** Copy-sector / r-index `Φ`-shift & BWT Right-Context Adjacency (testing whether ANY non-rigid copy-sector grouping can beat `h₁` on Stream N, and proving the BWT Right-Context Adjacency ceiling on why repeating strings collapse into width `≥ 2` runs rather than width-1 runs).
3. **N-12:** Formulate explicit Stream-N premises (`P_N-1`, `P_N-2` — or a clean conditional NO-premise for Stream N + `Pglue` obstruction) for MONODROMY to wire into `main-theorem.md`.
