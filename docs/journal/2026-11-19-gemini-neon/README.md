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
| [response-2.md](response-2.md) | Neon's second response, after a follow-up asking it to re-examine Theorem 3 and repair Theorem 2. Verbatim and unedited. |
| [verify_thm3.py.txt](verify_thm3.py.txt) | Neon's round-3 exhaustive verifier for Theorem 3′, verbatim; kept as text so it is neither linted nor edited. Neon could not run it; the Coordinator ran it (results below): `cp verify_thm3.py.txt v.py && python3 v.py`. |
| [batchrate.py](batchrate.py) | The Coordinator's measurement of how often fronts can be batched, using Neon's own code on real text. |
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

## Round 2

Asked to re-examine Theorem 3 and repair Theorem 2, Neon ([response-2.md](response-2.md)) did the following.

**What it got right:**
- **It refuted its own Theorem 3,** with three concrete defects:
  - cached front pointers go stale when a cut in one component changes another component's rightmost codomain interval. It gives an n = 4 counterexample with a single front;
  - forward mark transport is not local in a coordinate-free representation;
  - the Case 3a write footprint was incomplete.
- **It replaced it with Theorem 3′:**
  - per-component sentinels;
  - backward mark transport;
  - exact footprints.

  Its dynamical core is that if T(Y) ∩ Y = ∅ for the union Y of the cut pieces, every surviving point returns within two steps. That is sound, and is the part most worth keeping.
- **Theorem 2′:** the Fibonacci indexing is repaired, and the Kac identity is now limited to explicit row tracing. It says plainly that this is not a bound on interval-level discovery.

**Still not established:**
- **No executable verification.** "Verified" rests on three hand-worked examples. There is no checker, no case count and no reproducible output.
- **Test A's repair is impossible.** It proposes splitting I₂ and I₅ into two non-empty intervals each, but both have length 1. The two-front, conflict-free batch it claims to demonstrate is therefore not demonstrated.
- **Batching may rarely be possible.** In Test B, the only multi-front example that runs, Condition 2 (A_j ≠ B_j′) rejects every attempted batch. Whether real blocks admit batches of two or more fronts often enough to matter is untested, and it decides the practical value. Measure it first: the fraction of rounds with |S| ≥ 2 on Wikidata blocks.

**Classification:** Theorem 3′ is a plausible corrected construction with a sound dynamical core. Its pointer-level and mark-transport claims need an executable reference model. MONODROMY now audits Theorem 3′ rather than the withdrawn Theorem 3.

**Round 2 also shows how Neon behaves under correction:** it found a real defect in its own construction and repaired it rather than defending the original. It still labels claims "proved" ahead of its evidence.

## Round 3

Neon corrected Test A with a valid two-front batch (n = 10, r = 8, three unequal cycles). It also supplied an exhaustive verifier (`verify_thm3.py`), and said plainly that it had no interpreter and had **not run it**. It noted two subtleties:
- the update must unlink a node before splicing it, to avoid a self-loop;
- the result is a valid refinement of the maximal partition, not always the maximal partition itself.

**The Coordinator ran it** (Python 3, standard library only, 1.3 s): `ALL TESTS PASSED`.

| Domain | Configurations | Admissible batches | of which ≥ 2 fronts |
|---|---|---|---|
| BWTs: binary n ≤ 8, ternary n ≤ 5 | 2,094 (each also decoded end to end) | 3,501 | 15 |
| General dIETs, r ∈ {4, 5}, K ∈ {2, 3} | 2,736 | 7,463 | 1,131 |

Every case was compared against direct first-return computation, with return words, every primary mark, every order of each batch and the representation invariants. There were 635 strict refinements and no failures.

**What this supports:** Theorem 3′ is verified exhaustively over this bounded domain. Within that scope this is computational verification, not a proof. MONODROMY's audit still stands.

**Batch rate.** The small BWT cases suggest batching is almost never possible (15 of 3,501). That is an artefact of tiny blocks. `batchrate.py` runs Neon's own greedy decode on real text, with K pinned C[c] boundaries:

| Source (r/n) | K | Rounds with ≥ 2 fronts | Mean fronts per round | Serial-chain reduction |
|---|---|---|---|---|
| markdown blog post, n = 6,000 (≈ 0.41) | 1 | 0% | 1.0 | 1× |
| | 4 | 97% | 3.4 | ≈ 3.4× |
| | 15 | 91% | 7.1 | ≈ 7× |
| JSON (calibration.json), n = 20,000 (≈ 0.31) | 14 | 36% | 3.6 | ≈ 3.6× |
| | 26 | 23% | 3.3 | ≈ 3.4× |

**What the batch rate shows:**
- The total number of induction steps is essentially unchanged; batching shortens the dependent chain, it does not remove work.
- The gain saturates at a small K on repetitive data.
- **Neither sample is in Cells' admitted regime** (r/n below 0.15; Wikidata's admitted blocks are 0.097–0.148).

**Still open:**
- the batch rate on a real admitted Wikidata block;
- whether Cells' shipped induction is latency-bound in the first place. It already prefetches. If it is not latency-bound, a shorter chain buys little.

## Routing

- **MONODROMY:** verify Theorems 1, 2′ and 3′ (CHECKPOINT 7005355308, redirected to 3′ after round 2). Theorem 3′ comes first: an exhaustive small-case comparison against direct first-return computation, with return words and marks. Then the batch rate: how often |S| ≥ 2 is admissible.
- **Louis's optional follow-up to Neon:** ChatGPT's suggested follow-up asks Neon to re-examine Theorem 3 exhaustively and to repair Theorem 2. That is a good second-round test of calibration.

## MONODROMY's verdict (REVIEW 7005920415; record 78)

| Claim | Verdict |
|---|---|
| Theorem 3′ (multi-boundary parallel RV) | Exact (8,049 batches, every order agrees). The sufficient condition is weaker than the footprint rule: {A} ∩ {B} = ∅. The mark rule needs a relabelling clause. |
| ">99% full batches" | Refuted. On the frozen Wikidata column the mean batch width is 3.84 at K = 16, bounded by the heaviest component (25.6% of the cells). |
| "Cells is strictly serial, MLP = 1" | Refuted. The shipped Cells kernel fetches 12 slots ahead. |
| Theorem 1 | Verified, with the definition of t corrected. It sharpens Astra E's f2-bounds term h to h′ = \|∂ ∖ (D ∪ C)\|. |
| Theorem 2 | The Kac identity is verified, with a narrow scope. The Fibonacci indexing is off by one; the corrected range is f_d − 1 ≤ \|P\| ≤ f_{d+1} − 2, for 2 ≤ d ≤ q − 1. |

**Outcome of the audition:**
- **Kept:** a correct multi-front construction, and a sharper run bound.
- **Not supported:** the performance case. Cells already overlaps about 12 dependent loads, and real Wikidata exposes only about 4 independent fronts.
