# Gemini 4 Neon: an independent audition on induction scheduling

**Date:** 2026-11-19.

**What this is:** Louis gave an independent model (Gemini 4 Neon) a research prompt about whether
a better induction schedule can beat Cells' Rauzy–Veech reconstruction. The prompt was prepared
in the Coordinator's session. This directory files the exchange and its review.

**Scope:** a mathematical audition, outside the Astra E programme. Nothing here goes to Astra
unless Louis decides so.

| File | Contents |
|---|---|
| [prompt.md](prompt.md) | The prompt, verbatim. |
| [response.md](response.md) | Neon's response, verbatim and unedited. |
| [check_theorem1.py](check_theorem1.py) | The Coordinator's exhaustive check of Theorem 1 and of the 7-cycle example. |

## Status of the claims

Each claim is marked as one of: checked (by whom), disputed, or open (routed to whom).

### Theorem 1: the discontinuity budget

The claim is r(T_X') ≤ r + (c − 1) + |∂X' \ (D ∪ C)|.

- **The 7-cycle tightness example:** checked. L = `ccabbbb` gives exactly the stated T. Inducing on {0,1,3,4,5,6} gives four pieces, against the bound 3 + 1 + 0 = 4.
- **The bound:** no violation found by the Coordinator's exhaustive check (`check_theorem1.py`).
  - Domain: every ternary BWT with n ≤ 7, and every section meeting every cycle; 143,952 cases.
  - Pieces are counted by both map and return word.
  - This is a bounded verification, not a proof. The proof goes to MONODROMY.
- **Prior art in this repository:** the ledger Neon was linked to already holds a closely related bound: Astra E's f2-bounds, t ≤ r + k − 1 + h with h ≤ 2k, together with the tight r + 2 single-interval witness in f2-dual. Neon's form is sharper: it charges only section boundaries outside D ∪ C. Even so, it lists the bound as a "distinct deduction" without citing the ledger. Its relation to f2-bounds is for MONODROMY.

### Theorem 2.1: the Kac identity

- **The identity:** sound, and classical. The sum of first-return times equals the length of the covered cycles. As ChatGPT's review notes, it bounds explicit orbit tracing only. It is not a lower bound on every algorithm that discovers an induced map.
- **Disputed: its application to sparse exceptions.** Astra's exception constructor works from the original run records (690,484 records over ten blocks; `exceptions/contexts`), not from row-level traces. So the 2.35× acquisition cost is not explained by a "Kac floor n".

### Theorem 2.2: the Fibonacci separation

- **Disputed, as written.** The indexing is inconsistent (ChatGPT's review; confirmed here). With f₁ = 1 and f₂ = 2, the cylinder at d = q − 2 has f₃ = 3 occurrences, not the 2 stated.
- The intended separation between factored and unfactored induction is plausible. The statement needs repair.

### Theorem 3: multi-boundary parallel Rauzy–Veech

- **Open, routed to MONODROMY.** ChatGPT's review is right that disjoint touched records are not shown to imply commuting first-return transformations when the exchange maps across pinned components.
- **Needed:** an exhaustive small-case comparison against direct first-return computation, with words and marks. It should either give a corrected theorem or the smallest counterexample.

### Its picture of mibz2

Disputed, on three points:

- **"Cells: strictly serial, MLP = 1."** `src/cells.rs` already issues software prefetches for upcoming nodes during induction (lines 880–895). Whether Cells or LazyCells already overlaps several induction steps is for MONODROMY to establish from the source. How much latency the induction still exposes is an empirical question. Neon's own decisive experiment is the right test of it.
- **"Scalar-cap = dense LF, no induction."** Wrong. Scalar-cap is a run-record kernel from Astra E's combined-writer lineage, not a dense LF table.
- **"r ≈ 150k–350k at n = 900k".** This is outside the regime that matters. Cells admits blocks with r/n below 0.15 (`src/cells_lazy.rs`); Wikidata's admitted blocks lie at 0.097–0.148, and text starts at 0.28.

## Grading against the hidden answer key

These items were withheld from the prompt.

| Item | Found? |
|---|---|
| D ≤ r − 1 − S for exception ranges (the Researcher, 7003886829) | No |
| The ratio of exception rows to r: 8.2× at k=1, 0.59× at k=8, 0.33× at k=16 | No |
| Brown–Langmead (SEA 2026): O(r) LF-table construction, O(n) inversion in O(r) workspace | No |
| LF-shifted exceptions are (k+1)-byte triggers, ordered by cyclic prefix-free parsing | No. It equates PFP with word-cylinder induction, which is only partly right. |

## Assessment

**Strengths:**
- It takes the induction *schedule*, not the section alone, as the object to optimise.
- Its reading of the level-skipping blowup as lost SLP factoring agrees with the Coordinator's earlier reading.
- Its decisive experiment is concrete and falsifiable.

**Weaknesses:**
- Theorem 3, the headline, is not yet proved.
- It overextends the Kac identity.
- It mis-describes the incumbent kernels.
- It claims novelty for a bound already in the ledger it was given.

**Classification so far:** a promising, testable conjectural construction, not a proved new theorem.

## Routing

- **MONODROMY:** verify Theorems 1–3. Theorem 3's confluence comes first, by an exhaustive small-case comparison against direct first-return computation, with return words and marks.
- **Louis's optional follow-up to Neon:** ChatGPT's suggested follow-up asks Neon to re-examine Theorem 3 exhaustively and to repair Theorem 2. That is a good second-round test of calibration.
