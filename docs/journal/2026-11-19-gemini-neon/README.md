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

## Round 4: total work, not depth

**The prompt:** Louis asked whether an induced representation can cut *total* paid work below Cells', rather than only overlapping it. The prompt included corrections of round 1's premises: Cells prefetches; the admitted regime is r/n below 0.15; scalar-cap is a run-record kernel.

**Neon's claims** ([response-4.md](response-4.md)):
- **Lemma 1:** the r run boundaries cut every boundary-hitting cycle into r disjoint *persistence arcs*.
- **Theorem 1, an exact work identity:** for *any* monotone component-suffix induction schedule, the number of elementary steps equals r′ plus the prefix and suffix records of the elimination order on those arcs. Stopped schedules and refined partitions are included.
- **Corollary 1:** at least 2r − |D̄ ∩ C̄| steps for any unaccelerated monotone schedule, and at least r with Zorich acceleration.
- **An analytical model:** Cells pays about r(2 ln(n/r) + 2γ − 1), roughly 3.8–4.8r.
- **Max-Cut** over pinned alphabet boundaries as the one available escape.

**What the Coordinator ran** (`verify_boundary_records.py.txt`, verbatim; `round4_adversarial.py`):

| Check | Result |
|---|---|
| Neon's Part 1: all binary BWTs n ≤ 8 and ternary n ≤ 5; dIETs with r ∈ {4, 5}; single and Max-Cut schedules; full and stopped induction | **PASS**: 6,360 runs, 0 mismatches |
| Adversarial: 400 random and repetitive BWTs, n = 20–400, under single, Max-Cut and three random schedules, each at stops 0, n/4 and n/2 | **PASS**: 6,000 runs, 0 mismatches in the identity, Corollary 1 never violated |
| A prefix-doubling BWT builder against Neon's rotation sort, on every small string | Identical. Used for Part 2, where the rotation sort would materialise about 400 MB of rotations at n = 20,000. |
| Neon's Part 2, its synthetic "admitted-regime" blocks, n = 20,000 | r/n is **0.23**, outside the admitted 0.10–0.15. Cells takes **1.92r** steps (full), not the 3.8–4.8r its model predicts. The floor is **1.28r**: \|D̄∩C̄\| ≈ 0.72r, not ≈ r²/n. Max-Cut against Cells: **0.96** (full) and **0.945** (stopped at 0.15n). |

**Assessment:**
- **Theorem 1 and Corollary 1:** strongly supported over a bounded domain. It is an exact accounting tool for comparing schedules without implementing them. The proof is routed to MONODROMY.
- **The Zorich bound (≥ r):** untested.
- **The harmonic cost model:** contradicted by the data, at 1.92r against the predicted 3.8–4.8r, and its floor estimate is off for structured text.
- **Max-Cut:** fails Neon's own registered criterion (a ratio of 0.80 or less to support it; ≥ 0.80 refutes it).
- **On these blocks:** Cells sits about 1.5× above the floor of *any* monotone schedule, and induction is only part of the decoder's cost. So little total-work headroom remains inside monotone interval induction, as Neon itself anticipated if Max-Cut failed.
- **One overreach:** it says that if Max-Cut fails, *no* monotone schedule can do much better. That does not follow: Max-Cut is one heuristic. The corollary's floor, though, does bound every monotone schedule, and the floor is the stronger statement.
- **Not yet measured:** the same three figures (steps, floor, overlap) on a real admitted Wikidata block.

## MONODROMY's second verdict, on round 2 (REVIEW 7006279867; record 79)

- **Theorem 3′ is verified.** 29,395 admissible batches are exact, with no shared nodes, sentinels included. All 242,216 runs output the right mark rotation. Neon's Defects 1 and 3 are real. Record 78 missed Defect 1, because its model read the fronts off the sentinels.
- **Batch rate on admitted Wikidata blocks** (0, 2 and 85; 85 is the highest admitted) at K = 16:
  - 67–91% of rounds batch two or more fronts;
  - the mean width is 4.0–5.3;
  - the dependent chain is 3.8–4.9× shorter.

  Small columns batch only 4.4%. This replaces record 78's frozen-column figure of 3.84.
- **Not latency-bound on the cloud CPU.** Eager IPC is about 2.9 and flat in the prefetch distance from 4 to 32; prefetching is worth 8–12%. Lazy IPC is about 2.6.
  - On the i9, prefetching was worth about 25%, so the i9 may differ.
  - Only a timed interleaved kernel on the workstation would decide it.
- **Theorem 2′:** the counts are exact. Right-end Rauzy–Veech reaches I(P_k) itself only at k = 4; it reaches a section of the same shape in k − 3 steps.

**The regime figure, settled by record 80.** LazyCells admits up to r/n 0.15: 228 of 233 Wikidata blocks, spanning about 0.057–0.143. Record 79's "0.053–0.088" referred to Cells' own narrower line, and record 80 corrects it. The prompts' "0.10–0.15" was too narrow. Neon's round-4 synthetic blocks, at r/n ≈ 0.23, are outside the admitted regime either way.

## MONODROMY's verdict on round 4 (REVIEW 7006472980; record 80)

| Claim | Verdict |
|---|---|
| Theorem 1, the exact work identity | **VERIFIED WITH CORRECTION.** The proof needs one surjectivity line. 0 failures in 165,340 runs. It gives the **shipped** Cells step count exactly on 7 Wikidata blocks (S = Cells' steps + 1 closure). |
| Corollary 1, S ≥ 2r − \|D̄∩C̄\| | **VERIFIED.** |
| The Zorich bound, S ≥ r | **VERIFIED** once an accelerated step is defined (a winner-run). **The shipped Cells takes 1.013–1.019r accelerated steps.** |
| Headroom on the frozen block | floor 62,138 (1.214r) against shipped 92,571 (1.809r) unaccelerated: a ratio of 1.49. Across r/n 0.057–0.143 the ratio is 1.39–1.50. |
| §4's \|D̄∩C̄\| ≈ r²/n | **REFUTED:** it is about 0.79r on Wikidata. |

**Outcome:**
- **Neon's identity is verified.** Applied to the shipped decoder, it shows that Cells' induction schedule is within about 2% of optimal in accelerated steps, among monotone component-suffix schedules.
- **Any monotone schedule** can save at most about 1.4–1.5× in unaccelerated steps, and Cells' accelerated steps already take most of that.
- **So the audition closed its own line:** induction *scheduling* is exhausted. Further total-work reductions must come from outside monotone interval induction: construction, emission, or a different reconstruction principle.

## Round 5: outside monotone induction (real admitted blocks)

**The setup:** Neon could not fetch the binary blocks, and stopped as instructed. It was then given a text packet: the corrected regime, record 80, the project's cost findings, and per-block statistics computed by the Coordinator. Its verifiers are run on the six authenticated blocks of `data/admitted-bwt` at 41c03a5b.

**Neon's claims** ([response-5.md](response-5.md)):
- **Theorem 1, weight-only Rauzy–Veech:** accelerated RV with integer weights only recovers the text-order (LF-orbit) position of every run head, every run tail and the primary mark. It therefore gives every boundary gap's separation depth, in "≈ 1.015r accelerated steps" and O(r) memory, without walking n rows.
- **Theorem 2:** from those positions alone, the Navarro–Ochoa–Prezza Theorem 7 copy scheme is extracted exactly: r literals and k = r − |D̄∩C̄| copies. Each byte's copy depth equals its offset within its run.
- **The proposed decoder, RV-Copy:** replay the copies by multi-pass interval peeling. It registered its own criteria:
  - positive: no more than 3 fragments per copy, and a mean fragment of at least 15 B;
  - sharp negative: at least 3r fragments, a mean below 5 B, or more than 50 passes.

**What the Coordinator ran:** `verify_rv_copy.py.txt`, verbatim, on all six blocks. Output: [round5_output.txt](round5_output.txt).

| Block (r/n) | Positions and bytes against an independent LF walk | Steps | Copies | Fragments per copy | Fragments / r | Mean fragment | Passes |
|---|---|---|---|---|---|---|---|
| 0 (0.057) | **PASS** | 1.809r | 10,954 | 40.6 | 8.69 | 1.9 B | 14,931 |
| 2 (0.061) | **PASS** | 1.869r | 13,223 | 35.6 | 8.64 | 1.8 B | 14,361 |
| 85 (0.088) | **PASS** | 1.771r | 17,119 | 25.5 | 5.49 | 1.9 B | 10,195 |
| 207 (0.097) | **PASS** | 1.764r | 20,803 | 22.6 | 5.41 | 1.7 B | 11,559 |
| 143 (0.120) | **PASS** | 1.790r | 26,964 | 16.4 | 4.09 | 1.8 B | 12,720 |
| 210 (0.143) | **PASS** | 1.684r | 26,748 | 16.4 | 3.41 | 1.8 B | 10,866 |

**Theorems 1 and 2: verified on every real block.** Every run head, every run tail, the primary mark and all ~900,000 reconstructed bytes match an independent LF walk.

**The step-count claim is wrong.** The "accelerated" and unaccelerated counts are identical on every block, so Neon's acceleration condition never fires on Wikidata. The figure is 1.68–1.87r, not 1.015r. On block 0 it is exactly 92,571, MONODROMY's figure for the shipped Cells (record 80). So weight-only RV does the same induction work as Cells, not less, and its saving is only the SLP and word state per step.

**RV-Copy is refuted by Neon's own registered criterion.** All six blocks meet the sharp-negative condition: 3.4–8.7r fragments, a mean of 1.7–1.9 B, and 10,000–15,000 passes. The copy description is compact, at 11–27k copies, but its copies depend on each other in both directions across long runs. Replay shatters into byte-scale pieces, which is the obstruction Neon itself named in its Theorem 2.3.

**What survives:** a verified construction that recovers the text positions of all run boundaries, and so every boundary gap's separation depth, with integer-only state in Cells' step count.

**Two questions are routed:**
- **Novelty, to the Researcher.** It is close to the r-index's sampling of suffix-array values at run boundaries, and to toehold and φ.
- **Proof and scope, to MONODROMY.** Does it answer the project's open question for *every* gap within a run, or only for boundary gaps?

### Round 5 reviews

**MONODROMY (REVIEW 7007723857; record 81):**
- **Theorem 1 (position recovery): VERIFIED WITH CORRECTION.** 0 failures in 70,706 runs: 60,159 with several cycles, 27,877 with λ > 1 bands, 10,485 with quotient steps. The correction: on a λ > 1 band, tails resolve on the right endpoint's cycle (phase-aligned), not on the heads' cycle.
- **Scope: boundary gaps only, in O(r).** Every gap's separation depth *is* determined, by one tail-successor search per row of run offset; this is exact on all 848,747 interior gaps of block 0. But one gap then costs a mean of 1,643.5 searches against a walked depth of 262.2. So the project's open question is answered for boundary gaps cheaply, and for interior gaps only at a cost above walking.
- **The "≈ 1.015r accelerated steps": REFUTED.** Neon's quotient fires zero times on blocks 0, 85 and 210, so on block 0 accelerated = raw = 92,571. 1.015r is MONODROMY's winner-run lower-bound count. Each loser still costs O(1), so weight-only RV does 1.68–1.87r steps, as Cells does.

**Researcher (REVIEW 7007681052):**
- **The targets are known.** The recovered positions are the r-index's run-boundary suffix-array samples, up to origin and direction. The classical construction takes a full LF lap; toehold and φ consume the samples.
- **Sublinear routes already exist:**
  - Kempa (SODA 2019, Thm. 5.1): O(n/log n + r log⁵ n) time;
  - Agol–Hass–Thurston weighted interval pairing: polynomial in r and log n.

  Neither has Neon's O(r) words of working state.
- **No worst-case guarantee.** On `2·1·0^N` (r = 3), the pinned schedule takes n steps: 4,098 at n = 4,098.
- **What is distinctive:** the weight-only O(r)-state implementation, and its measured 1.68–1.87r on real admitted blocks without a separate unit LF lap.

**Round 5 overall:**
- **Kept:** a correct, verified construction that recovers the run-boundary positions in Cells' step count with integer-only state. It is practically neat, and not theoretically new.
- **Refuted:** the RV-Copy decoder (by Neon's own criterion), the step-count claim, and the claim to have answered the open question for every gap.
