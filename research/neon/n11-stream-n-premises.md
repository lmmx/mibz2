# N-11: Stream-N premises (`P_N-1`, `P_N-2`) and conditional partial-no synthesis for `main-theorem.md`

*Cycle 2 formal deliverable for MONODROMY (`origin/claude/sharp-meitner-cx0v78`, `main-theorem.md` §4.4 & §6 at `d0d0232`), synthesising the 11 closed frameworks (`N-3`–`N-8`, `N-10`), amended 3-part `N-0` (`N-8`), and `NEEDS M-4` (`N-9`).*

## 0. The book (METHOD.md §6a)

1. **Input features relied on:**
   - All 6 admitted blocks (`origin/data/admitted-bwt`) and 232 distinct two-ended census blocks (`#805`): exact `h₁`, `h₁R/h₁ = 0.798–0.851` (`N-2`), novel hex = `61.3–68.2%` (`N-8`), repeated SHA1 w1 = `0.00–0.01%` (`N-8`), `Pglue` switches `b/h₁N = 1.19–1.63` and `all_sw/h₁ = 0.513–0.543` (`N-2`), and `NEEDS M-4` flip separation `ν′(x)` median `6,918–14,314` on Stream N with `|U| ≤ 1–354` (`N-9`).
2. **Real code path:**
   - Target parameters frozen in `main-theorem.md` §1 & §1a (`4d7d83b`, `d0d0232`): `K = 2`, `δ = 1/2`, `S(c) ≤ openPeakScratch c`, `d(D, c) ≤ d(Open, c)` (§1a clause 6), and anti-cheat clauses 1–5.
3. **Assumptions knowingly unrealistic:**
   - `h₁R/h₁`, novelty shares, and `ν′(x)` are measured exactly on the 6 admitted blocks (`data/admitted-bwt`); extending those exact per-block numbers to the other 226 census blocks of `𝓕` requires a corpus run on `corpora/wikidata.bz2` (absent from the repo).

## 1. Status of the E2 open cell on Stream N after 11 framework screens

Every framework screened across Cycles 1–2 (5 initial kitchen-sink + 2 outside-review priority leads + 4 algebra/spectral/learning families) closed at (a) Escape, (b) Passes, (c) Viability, or (d) Payoff:

| # | Framework | Record | Route | Closed at | Core obstruction |
|---|---|---|---|---|---|
| 1 | Ostrowski / Bratteli–Vershik | `N-3` | R1 | (a) Escape | Canonical BV acquisition = RV induction in `𝓡` (`c₁ ≥ h₁`); `|E(B)| = 1.68–1.87r` |
| 2 | Rauzy fractals / Pisot substitutions | `N-4` | R2 | (b) Passes, (c) Viability | Non-stationary `S`-adic = RV; pure Stream N `p_{N2}(16) = 42.6k–155.5k` (`≫ O(k)`) |
| 3 | Veech groups / `SL(2, ℤ)` origami | `N-5` | R1 / R2 | (a) Escape (V1), (b)/(c) (V2) | Thm V1: `(r, h₁)` invariant across `(1,0),(1,1),(1,-1)`; `Aut = {id}` proved; `(1,1)≠(1,-1)` 6/6 |
| 4 | Discrete / algebraic Morse (Perseus) | `N-6` | R1 (R0) | (a) Escape, (b) Passes | 1-/2-complex collapses in `𝓡`; `P_LF - I` pivots charged (Rec 100), spectral gap `7 × 10⁻⁶` |
| 5 | Agol–Hass–Thurston interval-pairing | `N-7` | R1 | (a) Escape, (c) Viability | Thm AHT-1: moves in `𝓡`; `k = r ≥ h₁`, `63–75%` w1, `2/509,269` initial self-overlaps |
| 6 | Positional copy schemes (`φ`/LZ) | `N-8` | R2 | (a)/(b) & (d) Payoff | `M-7` + exact test: 3,645 rep SHA1s give **3** w1 rows (`0.00–0.01%` of `h₁`); novel hex `61–68%` |
| 7 | Compressed suffix coords (r-index/SA) | `N-8` | R1 / E2 | (a) Escape, (c) Viability | Every w1 run is a boundary (`≥ h₁` singleton samples, §1a clause 1); Rec 84 |
| 8 | Krohn–Rhodes / `pos mod d` scans | `N-10` | R1 / E2 | (a)/(b) | `⟨LF⟩ = ℤ/nℤ` (zero reset); `pos mod 2` `L`-complete; Rec 101 flips shift `pos mod d` |
| 9 | Dimension groups (`K₀`), `S_n` Fourier, 1D OT | `N-10` | R1 | (a) Escape | `K₀` non-identifiable (`caab`/`cbaa`); all `n`-cycles conjugate in `S_n`; 1D OT = `LF` |
| 10 | Learned PGM-index / Compressed sensing | `N-10` | R2 | (b) Passes, (c) Viability | `w1 \|SA[x+1]-SA[x]\|` median `178k–223k` (`nonhex_w1` `103k–184k`), `< 1.9%` within `≤ 64` |
| 11 | Coarse-aggregate global rules (`Λ_agg`) on Stream N | `N-9` | E2-agg | **(a)/(b) via Rec 101** | `ν′(x)` median **`6,918–14,314`** on `nonhex_w1`, `|U| ≤ 1–354` across all 6 admitted blocks |

## 2. Proposed premises for `main-theorem.md` §4.4 (`PN`)

Because 0/11 frameworks survived, we submit two precise, falsifiable premises for MONODROMY to wire into `main-theorem.md` — one stating the **exact necessary shape** of any hypothetical Stream-N escape (`P_N-1`, the surviving residue of E2), and one stating the **established Stream-N partial-no** (`P_N-2`):

### Premise `P_N-1` (Necessary condition for a fine-input Stream-N escape, `[argued]` + `[measured]`)

```lean
/-- P_N-1: Necessary conditions for any sound Stream-N decoder achieving aN < 1 on 𝓕
    under main-theorem.md §1–§1a (K = 2, S ≤ openPeakScratch, d ≤ d(Open)). -/
def PN1_FineGlobalEscape (ΛN : FactLanguage) (DN : StreamDecoder .N) : Prop :=
  -- (1) Must leave the closed language Λ_closed := 𝓡 ∪ E1 ∪ E3 ∪ E2_local ∪ E2_O1 ∪ Λ_agg:
  (∃ Φ ∈ ΛN, Φ ∉ Λ_closed ∧
    -- (2) Multi-row non-rigid subject (≥ 2 scattered width-1 rows placed jointly, §1a clauses 1–2):
    (∀ c ∈ 𝓕, ∃ f ∈ (DN c).factsOfType Φ, f.subjectWidth1.card ≥ 2 ∧ ¬f.isIntervalTranslation) ∧
    -- (3) Fine-grained whole-block sensitivity (separates Ω(ν′(x)) disjoint Record-101 flips):
    (∀ c ∈ 𝓕, ∀ f ∈ (DN c).factsOfType Φ, ∀ x ∈ f.subjectWidth1,
      f.inputSeparatesDisjointFlips c x ≥ c.nuPrime x)) ∧
  -- (4) Cost scales with a parameter θ_N(c) ≪ h₁N(c) on 𝓕 that does not degenerate to r,
  --     does not assume zero entropy / p(k) = O(k), does not assume global affine symmetry,
  --     and does not rely on repeated-substring interiors (M-7):
  (∀ c ∈ 𝓕, DN.passes c ≤ 2 ∧ DN.state c ≤ S c ∧ DN.depLoads c ≤ Open.depLoads c)
```

- **How `P_N-1` is refuted (yielding a full "no"):** Prove that no `K = 2`-pass, `S ≤ openPeakScratch` procedure whose input separates `≥ ν′(x)` disjoint flips can soundly place `≥ 2` scattered width-1 rows per event without `c₁ ≥ h₁N` (or `d(D, c) > d(Open, c)`).
- **How `P_N-1` is discharged (yielding a Stream-N "yes"):** Exhibit an explicit sound rule `Φ` satisfying (1)–(4) on every `c : Column` with `aN < 1` on `𝓕` — **and** note that even `aN = 0` alone cannot meet `δ = 1/2` on `𝓕` without `aR < 1` (by `P_N-2` below).

### Premise `P_N-2` (Stream-N partial-no over `Λ_closed` & `Pmargin` impossibility at `aR = 1`, `[proved]` given G1 + Rec 101, `[measured]` on 6 admitted blocks)

```lean
/-- P_N-2a: Stream-N partial-no over Λ_closed := 𝓡 ∪ E1 ∪ E3 ∪ E2_local ∪ E2_O1 ∪ Λ_agg(𝒫, D)
    with |𝒫| - 1 + 3|D| < min_{x ∈ N_w1 \ U} ν′(x). -/
theorem PN2_streamN_partial_no
    (c : Column) (hc : c.singleCycle)
    (D : Decoder Λ_closed) :
    c₁ (D.trace c |>.on (σ_hex.rows .N)) ≥ c.h₁N - c.U_card

/-- P_N-2b: Stream-N-only impossibility for δ = 1/2 on the 6 admitted blocks (even with aN = 0, Pglue = 0). -/
-- Measured on all 6 admitted blocks of data/admitted-bwt:
--   under σ_hex:            h₁R / h₁ ∈ [0.7978, 0.8514]  =>  aR = 1 forces c₁ ≥ 0.798·h₁ > 0.5·h₁
--   under Def1 novelty:     novel_hex / h₁ ∈ [0.613, 0.682] => a_{novel}=1 forces c₁ ≥ 0.613·h₁ > 0.5·h₁
--   under oracle non-GUID:  GUID / h₁ up to 0.666 (block 0) => a_{GUID}=1 forces c₁ ≥ 0.666·h₁ > 0.5·h₁
--   with non-pos Pglue:     w1 switches b = 1.19–1.63 · h₁N => c₁(Pglue) > h₁N on 6/6 blocks
```

## 3. Strategic implication for both agents

With `P_N-2` established on the 6 admitted blocks and 11/11 Stream-N frameworks closed:
1. **No Stream-N-only construction can deliver a "yes" at `δ = 1/2`**, even if an oracle made all of Stream N (and all repeated hex) free (`c₁ = 0`) with zero glue cost.
2. **Coarse-aggregate global E2 (`Λ_agg`) is closed symmetrically on BOTH Stream R and Stream N** (`ν′` median `6,900–14,300`, `|U| ≤ 1–354`).
3. **The sole remaining open cell in the entire challenge** is **fine-input global E2** (rules reading fine-grained column structure that separates `Ω(ν′)` flip pairs, within `K = 2` passes, `S ≤ openPeakScratch`, and `d ≤ d(Open)`), and any "yes" must break **`aR ≤ 0.29`** (with `σ_hex` glue) or **`aR ≤ 0.50`** (even with positional glue) on **novel hex / GUID rows**!
4. Conversely, pushing toward a **clean "no"** now has a razor-sharp single target: extending Record 101's swap-invariance / communication-or-streaming lower bound from `Λ_agg` to `K = 2`-pass fine-input streaming procedures with state `S ≤ openPeakScratch` and dependent loads `d ≤ d(Open)`.

## 4. Postmortem (METHOD.md §6a)

1. **Input assumptions:** Exact on 6 admitted blocks; full 232-block per-byte novelty & `ν′` numbers remain a corpus-side run (`corpora/wikidata.bz2` absent locally).
2. **Model assumptions:** Adopts MONODROMY's frozen parameters (`4d7d83b`, `d0d0232`) including §1a clause 6 (`d ≤ d(Open)`) and Record 101's repaired `3|D|` run-record blindness bound (`N-9`).
3. **Method:** Exhaustive synthesis of Cycles 1–2 screens (`N-3`–`N-10`) into pseudo-Lean premises compatible with `main-theorem.md` §3–§6.
4. **Outcome:** Delivers `P_N-1` (exact fine-input E2 target) and `P_N-2` (Stream-N partial-no over `Λ_closed` + `δ = 1/2` Stream-N-only impossibility).
