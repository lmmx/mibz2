# N-1: Adversarial audit of MONODROMY records 96, 98, 99, 100 and `main-theorem.md`

*Cycle 1 adversarial check on `origin/claude/sharp-meitner-cx0v78` through commit `6737734` (`c191322`, `eae9ca9`, `a7c2e4c`, `6737734`).*

**Verifier:** [`research/neon/scripts/verify_monodromy_breaks.py`](scripts/verify_monodromy_breaks.py) (standard-library Python, run on exhaustive binary columns `n = 1..6`, witness columns, and the 232 distinct two-ended census blocks of `#805`).

## 0. The book (METHOD.md §6a)

1. **Input features relied on:**
   - Arbitrary `Column` definitions from [`PROOF-PROGRAMME.md`](../../docs/team/research-charter/PROOF-PROGRAMME.md) §1.2 (`0 < n ≤ 900_000`, `p < n`, arbitrary `L : Fin n → UInt8`, not restricted to single-cycle or BWT columns).
   - Exact `(m, runs_1, runs)` on the 232 distinct two-ended Wikidata blocks in [`docs/journal/2026-11-24-wikidata-block-census/census.tsv`](../../docs/journal/2026-11-24-wikidata-block-census/census.tsv) (`#805`).
   - Exact `σ_hex` orbit-switch counts measured on the six admitted blocks of `origin/data/admitted-bwt` (reported in full in [`n2-f-baseline-and-needs-m2.md`](n2-f-baseline-and-needs-m2.md)).
2. **Real code path:**
   - `src/open.rs` (`OpenCells`, `LIMIT = 16`, retaining every row of maximal runs of width `≤ 16` into `m` individual `edges` entries).
   - `src/bwt.rs` (`walk`, `walk_back`, `repeat_cycle`: emits the marked cycle of `p` repeated to `n` bytes).
3. **Assumptions knowingly unrealistic:**
   - None for the formal counterexamples (they evaluate the exact specification `Column.decode` and the exact pseudo-Lean definitions in `main-theorem.md`).

## 1. Break 1: `theorem G1` fails on arbitrary `Column` (`[proved]`, `[measured]` exhaustively for `n ≤ 6`)

- **Targets broken:**
  - `origin/claude/sharp-meitner-cx0v78` (`a7c2e4c`), `docs/journal/monodromy/2026-11-24-main-theorem/main-theorem.md` L49: `theorem G1 (D : Decoder 𝓡) (c : Column) : c₁ (D.trace c) ≥ c.h₁`.
  - `eae9ca9`, `docs/journal/monodromy/2026-11-24-fewer-than-h1.md` L56 ("every decoder in `𝓡` has `c₁(D) ≥ h₁` on every column").
  - `c191322`, `docs/journal/monodromy/2026-11-24-guid-wall.md` L48 ("for every column and every route in `𝓡`, `D_width-1 ≥ h₁`").
- **Counterexample witness `[proved]`:** `L = "aab"`, `n = 3`, `p = 0`.
  - Valid `Column` under `PROOF-PROGRAMME.md` §1.2.
  - Maximal runs: `[0, 2)` (`'a'`, width 2) and `[2, 3)` (`'b'`, width 1), so `c.h₁ = 1`.
  - `LF = [0, 1, 2]`. The marked cycle of `p = 0` is `{0}` (period 1); `c.decode = "aaa"`.
  - The 1-link output chain using the single primitive fact `([0, 2), 1, 0, "a")` (width 2) emits `"a"` repeated to `n = 3` = `"aaa" = c.decode` with **`c₁ = 0 < 1 = c.h₁`**.
  - Generalisation: `L = "aaabcdef"`, `p = 0` gives `h₁ = 5`, `c₁ = 0`.
- **Exhaustive check (`verify_monodromy_breaks.py`) `[measured]`:** Across all 642 binary `(L, p)` pairs for `n = 1..6`, **350/642 (54.5%)** satisfy `h₁_marked < h₁` (and 134 allow `c₁ = 0 < h₁` in `𝓡`).
- **Repair:** Either add hypothesis `c.singleCycle` to `G1` (which holds on all 232 census blocks and all 6 admitted blocks in `𝓕`), or replace `c.h₁` with `c.h₁_marked` (the number of width-1 runs intersecting `p`'s cycle).

## 2. Break 2: `theorem main` is not provable from its stated hypotheses (`[proved]` syntactically, `[measured]` on `𝓕`)

- **Target broken:** `origin/claude/sharp-meitner-cx0v78` (`a7c2e4c`), `main-theorem.md` L56–65 (`theorem main`).
- **Two missing hypotheses `[proved]`:**
  1. **Uncharged `Pglue` in `Pmargin`:** §4.5 (L109) correctly notes that each N↔R boundary on the orbit is a width-1 event charged to `c₁` unless computed positionally. Yet `Pmargin` (L62) bounds only `aN c * c.h₁N + aR c * c.h₁R ≤ (1 - δ) * c.h₁`, omitting `c₁(Pglue)`. If `c₁(Pglue) > 0`, the conclusion `c₁(D.trace c) ≤ (1 - δ) * c.h₁` does not follow.
     - **Quantitative bite `[measured]` (see `n2-f-baseline-and-needs-m2.md`):** Under `σ_hex`, the width-1 N↔R boundary count `b` on the six admitted blocks is **8,746–21,750**, which is **`1.19–1.63 × h₁N`** on every block (and full-orbit switches are `0.513–0.543 × h₁`). If `Pglue` pays even 1 event per width-1 boundary, `c₁(Pglue) > h₁N` wipes out 100% of any Stream N gain.
  2. **Missing `state` hypotheses:** `theorem main` concludes `D.state c ∈ O(c.r)` (L64), but none of `P0`, `PN`, `PR`, `Pglue`, `Pbudget`, `Pmargin` bounds `DN.state`, `DR.state`, or `Pglue.state`.
- **Repair:** Add `c₁(Pglue c)` to `Pmargin` (or require `c₁(Pglue c) = 0` inside `Pglue` via positional boundary determination) and add `O(c.r)` state bounds on `DN`, `DR`, and `Pglue`.

## 3. Break 3: `c₁_split` (P1) and `c₁(Open)` inconsistency (`[proved]` on witnesses, `[measured]` on 232 blocks)

- **Targets broken:**
  - `a7c2e4c`, `main-theorem.md` L52–53 and §4.1–4.2 (`c₁_split` + `σ_hex`).
  - `eae9ca9`, Record 98 L52 (`Open ... with c₁ = h₁ + (the width-1 pieces its induction creates)`).
- **Three concrete defects:**
  1. **Mirror-rule attribution crosses `N`/`R` `[proved]`:** In `Rule.mirror R`, the subject interval is `LF(R)`, not `R`. On the single-cycle witness `L = "ag0"` (`LF = [1, 2, 0]`), width-1 run `x = 0` (`L[0] = 'a'`, stream `R` under `σ_hex`) has mirror subject `{LF(0)} = {1}`, where `L[1] = 'g'` (stream `N`). Attributing events by subject row charges an `R`-run's mirror step to `c₁|N`.
  2. **Non-width-1 runs unattributed under `σ_hex` `[proved]`:** §4.2 defines `R` as a subset of width-1 runs (`σ_hex`). Any width-1 event whose join row or link start lies in a run of width `≥ 2` (for example, a width-1 piece created by endpoint induction inside a thick strip, or Open's retained walk on width `2..16` runs) lies in neither `h₁N` nor `h₁R`. If `σ` instead partitions all `n` rows, then `PR` (`c₁(DR) ≤ aR * h₁R`) fails at `aR = 1` for any decoder (like shipped Open) that emits width `2..16` runs via width-1 links.
  3. **`c₁(Open)` vs `c₁(LF walk)` ambiguity `[proved]` / `[measured]`:**
     - **Reading A** (a chain link may reference a fact `(I, h, t, w)` of width `≥ 2` directly without restricting to `[x, x+1)`): the **plain LF walk** uses only primitive facts `(R, 1, t_R, L[R])` and therefore achieves **`c₁(LF walk) = h₁`**, strictly beating Open whenever Open's induction creates any width-1 piece.
     - **Reading B** (each per-row emit lookup in `src/open.rs` is a width-1 link, matching `edges[i]`): shipped Open pays `c₁(Open) ≥ m` (not `h₁ + induction width-1 pieces`). Across the 232 distinct census blocks (`verify_monodromy_breaks.py`), `m / h₁` is **1.207 / 2.586 / 3.316** (min / median / max), with `m - h₁ = 24,345–139,815` (median `63,350`) width `2..16` retained rows per block.
- **Repair:** Fix one convention explicitly: either (i) define `c₁` to count width-1 events **whose underlying step traverses a width-1 maximal run** (matching Record 96's `D_width-1`, so `c₁(Open) = h₁` and `σ_hex` partitions all counted events, while accepting that plain LF also has `c₁ = h₁` and separating LF from Open via total steps / `r`), or (ii) keep Record 98's all-width-1-events `c₁` and adjust `c₁(Open) ≥ m` and `PR` accordingly (requiring a width-preserving emit for `2..16` runs to reach `aR = 1`).

## 4. Break 4 (scope tightening): Record 100 (`6737734`) `PR` bound is probabilistic at `k ≥ 6`, not deterministic on `𝓕` (`[argued]`)

- **Target:** `6737734`, Record 100 §4 L40 (`for Λ = 𝓡 ∪ {k-local Φ}, aR ≥ 1 − (expected partners) ≥ 0.996 at k ≥ 6 on model-I rows`).
- **Two scope gaps `[argued]`:**
  1. `1 - (expected partners)` is an expectation under model I, whereas `PR` in `main-theorem.md` L59 is a deterministic universal bound `∀ c ∈ 𝓕`. Turning the expectation into a bound on `𝓕` requires either measuring the actual `k`-key collision fraction on `𝓕` or replacing model I with a per-block empirical collision count.
  2. For `k ≤ 5` (e.g., `k = 2` or `k = 4`), `k`-local non-positional facts have large subjects (`|D_u| ≥ 2` on >50% of model-I rows at `k = 4`, ~100% at `k = 2`), so their closure relies specifically on Record 99 §2 (chain links still cost 1 per row) + 3-byte global sensitivity (no `k`-local *positional* fact is sound for `k` below the sensitivity radius). Stating `k ≥ 6` only is unnecessarily weak for positional facts (closed for all `k = o(n)`) and needs the chain-link closure explicitly for non-positional `k ≤ 5` facts.

## 5. Postmortem (METHOD.md §6a)

1. **Input assumptions:** Verified exact on `n = 1..6` binary columns, witness columns, and `#805`'s 232 census blocks.
2. **Model assumptions:** All four findings operate directly inside MONODROMY's stated pseudo-Lean definitions (`main-theorem.md` `a7c2e4c`, Record 98 `eae9ca9`, Record 100 `6737734`).
3. **Method:** Direct counterexample construction (`verify_monodromy_breaks.py`), syntactic check of `theorem main`, and census join.
4. **Outcome:** Four concrete repairs identified before Lean formalisation: single-cycle / `h₁_marked` hypothesis in `G1`; `c₁(Pglue)` and `state` hypotheses in `main`; clean separation of `c₁` vs `D_width-1` and mirror attribution in `c₁_split`; deterministic empirical vs model-I wording in Record 100.
