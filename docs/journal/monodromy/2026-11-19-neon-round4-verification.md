# 2026-11-19 · MONODROMY 80 · Neon's boundary-record work identity verified: exact, with one surjectivity line to add. It reproduces the shipped step count on seven Wikidata blocks. The floor 2r − |D̄ ∩ C̄| holds, but on Wikidata it is 1.21–1.25 r, not 1.85–1.90 r, and Cells already runs at 1.68–1.87 r. So no monotone schedule can save more than 1.39–1.50×, and with acceleration Cells is within 2% of the floor r. Record 79's "no admitted block at 0.10–0.15" is corrected for LazyCells

**The request.** The Coordinator's CHECKPOINT 7006331464 (#346). The source is PR #629, branch
`docs/neon-round4` at 009e269f, file `docs/journal/2026-11-19-gemini-neon/response-4.md`. The
findings are for Louis.

I did not open `verify_boundary_records.py.txt`, `round4_adversarial.py` or the README's "Round 4"
before these checks.

**Labels:** **P** · **C** · **M** · **I**.

**Scripts** (beside this record as `.txt`):
- `neon4.py`: every small column. It loads record 78's `neon3.py`.
- `neon4real.py`: the Wikidata blocks.
- `admit.rs`: eager and lazy admission per block, from master 0c852a07 in a scratch worktree.

## Verdicts

| # | claim | verdict |
|---|---|---|
| L1 | Persistence arcs are disjoint and cover the cycles that meet C̄ | **VERIFIED** |
| T1.1 | Surviving count r_π(X) = #arcs meeting X | **VERIFIED** |
| T1.2 | S = Σ (p + s + 1[arc ∩ X = ∅]); r′ + Σ(p + s) when full | **VERIFIED WITH CORRECTION.** The proof shows each step charges a distinct record, but never that every record is charged. The missing line is below. 0 failures in 165,340 runs, and it reproduces the shipped steps on 7 of 7 blocks |
| T1, invariant | The live codomain and domain left endpoints are the first and last surviving points of each arc | **VERIFIED**, after every step of every run |
| T1, scope | Covers stopped, refined, multi-cycle and periodic cases, and the shipped Cells and LazyCells | **VERIFIED.** Neither shipped kernel stops and walks; the stopped form covers a right-end stop anyway |
| C1 | S ≥ 2r − \|D̄ ∩ C̄\| for full unaccelerated induction | **VERIFIED** |
| C1, Zorich | S_Z ≥ r | **VERIFIED WITH CORRECTION.** It needs a definition of an accelerated step; mine is below. Shipped Cells makes 1.013–1.019 r accelerated steps |
| §4.1–4.2 | \|D̄ ∩ C̄\| ≈ r²/n; floor ≈ 1.85–1.90 r; Cells ≈ r(2 ln(n/r) + 2γ − 1) ≈ 3.8–4.8 r; "2–2.5× available" | **REFUTED on Wikidata.** \|D̄ ∩ C̄\| ≈ 0.79 r; floor 1.21–1.25 r; Cells 1.68–1.87 r; at most 1.39–1.50× available |

## Theorem 1 (P, C)

**Counted.**
- *Columns:* all over 2 letters to length 9, 3 letters to length 7 and 4 letters to length 6.
  These include tied rotations and several cycles.
- *Pins:* every subset of the internal alphabet boundaries, which gives refined partitions with
  r′ > r.
- *Schedules:* per column and pin set, three random monotone component-suffix schedules (each
  step at a uniformly random nonempty component, closures included), plus one stopped at a random
  step.
- *The result:* 132,428 full and 32,912 stopped runs.
- *Arcs, π and the records* are computed from their definitions, independently of the schedule.

| check | failures |
|---|--:|
| arcs pairwise disjoint (Lemma 1) | 0 |
| after every step: live codomain left endpoints = first surviving point of each arc; live domain left endpoints = last | 0 |
| surviving interval count = #arcs meeting X | 0 |
| S = Σ (p_{π,X} + s_{π,X} + 1[A′(c) ∩ X = ∅]) | 0 |

**The proof holds, with one line to add.**
- *The case analysis is right:* the only boundary in Y_t is its left endpoint, max(a_j, b_j).
- *Case 1:* b_j is replaced by T_{X_t}(b_j), the first point of the arc in X_{t+1}. The arc's last
  survivor x_{j_t} lies ahead of b_j, so the orbit cannot leave the arc first.
- *Case 2:* the dual.
- *Case 3:* a_j = b_j means the arc meets X_t only there, and D̄ and C̄ each lose that one point.

**The missing line: why every record is charged.** "Every step is in bijection with one term"
needs that every record is charged, and the proof shows only that each step charges one. The
converse is short:
- A prefix record x_i (i < i*) outlives every earlier point of its arc. So when it is cut, it is
  the arc's first surviving point, which is a live codomain left endpoint.
- It is therefore the left endpoint of that step's Y_t. The step cannot be Case 3, since that
  would make x_i the arc's global maximum, so it is the Case 1 step that charges x_i.
- Suffix records are the dual.

With that line the identity is proved.

**Scope.**
- *Stopped (X ≠ ∅):* the 32,912 stopped runs.
- *Refined (r′ > r):* the pin sets.
- *Multi-cycle and periodic:* every column, ties included. Passive cycles, which meet no boundary,
  ride inside intervals and cost nothing, as the remark says.
- *The shipped schedules:*
  - Cells is K = 1 right-end Rauzy–Veech: a monotone component-suffix schedule with π
    decreasing in the row.
  - LazyCells runs the same steps. On the blocks both admit, its cell counts equal eager's
    exactly: 92,570 on block 0 and 140,543 on block 85.
  - Neither kernel stops and walks. Each runs to the closure that holds the mark, which is a
    stopped X when other cycles remain.
- *Counting:* the shipped cell counter leaves the closure out, so S = cells + closures. Neon's S
  counts the closure.

## Corollary 1 and Zorich (P, C)

**C1 holds.**
- *An arc with ℓ ≥ 1, unsplit:* its π-maximum can't be at both ends. x₀ is a prefix record
  whenever i* > 0, and x_ℓ is a suffix record whenever i* < ℓ, so p + s ≥ 1.
- *An arc split by refinement:* it adds at least one to r′ instead.
- *In the counts:* 0 failures in 132,428 full runs, with equality in 114,799.

**What an accelerated step is.** Neon's "collapsing consecutive Case 1/2 steps on the same pair"
needs a definition. Mine is a maximal run of consecutive steps at one front with the same winner:
- *The winner:* A_j in Case 1, B_j in Case 2.
- *The end of the run:* when the winner changes, or when a Case 3 step deletes the winner. That
  step is included in the run.

**The floor S_Z ≥ r′ ≥ r follows.**
- Only Case 3 steps delete an interval, and each deletes exactly one.
- A Case 3 step can only end a run.
- Full induction must delete all r′ intervals.

**Counted.** 0 failures in 132,428 full runs, with equality in 87,382.

## The frozen block and the admitted regime (C)

**What was run.**
- *The model:* `neon4real.py` runs the K = 1 pointer model to the end, which is what shipped Cells
  does, and evaluates Theorem 1's record sum from the arcs alone.
- *The blocks:* block 0 is the frozen column. Blocks 2, 85, 207, 208, 143 and 210 cover eager
  Cells' admitted range and LazyCells' range up to 0.143.

| block | r/n | admitted by | \|D̄∩C̄\|/r | floor 2r − \|D̄∩C̄\| | shipped steps S (cells + 1) | identity | S / floor | accelerated steps | random-order model |
|--:|--:|---|--:|--:|--:|---|--:|--:|--:|
| 0 (frozen) | 0.057 | both | 0.786 | 62,138 = 1.214 r | 92,571 = 1.809 r | = | **1.490** | 51,849 = 1.013 r | 5.89 r |
| 2 | 0.061 | both | 0.758 | 1.242 r | 1.869 r | = | 1.504 | 1.016 r | 5.76 r |
| 85 | 0.088 | both | 0.784 | 1.216 r | 1.771 r | = | 1.457 | 1.017 r | 5.01 r |
| 207 | 0.097 | lazy | 0.761 | 1.239 r | 1.764 r | = | 1.424 | 1.016 r | 4.82 r |
| 208 | 0.102 | lazy | 0.775 | 1.225 r | 1.727 r | = | 1.410 | 1.015 r | 4.71 r |
| 143 | 0.120 | lazy | 0.751 | 1.249 r | 1.790 r | = | 1.433 | 1.019 r | 4.39 r |
| 210 | 0.143 | lazy | 0.792 | 1.208 r | 1.684 r | = | 1.394 | 1.014 r | 4.04 r |

**The shipped counters agree.**
- *Eager:* block 0 is 92,570, block 85 is 140,543, and block 207 is refused.
- *Lazy:* 92,570, 140,543, 153,649, 158,995, 193,865 and 216,768.
- In every block, equal steps plus closures equal r.

**What this decides.**
- *Any monotone unaccelerated schedule* (K-front Max-Cut, two-sided, any adaptive order) can save
  at most 1.39–1.50× in steps on Wikidata, against Neon's estimate of 2–2.5×.
- *Neon's two inputs are both wrong here.*
  - |D̄ ∩ C̄| ≈ r²/n would be about 0.06–0.14 r. It is 0.75–0.79 r: three quarters of the run
    boundaries are arcs of length 0.
  - Real arcs are far from randomly ordered. The random-order record model predicts 4.0–5.9 r,
    and Cells takes 1.68–1.87 r.
- *With acceleration,* Cells' own steps are already 1.013–1.019 r against the floor r. Grouping
  steps is no source of work either.
- *The remaining 1.4–1.5× of unaccelerated steps* is all Case 1 and 2 steps. They cost about the
  same as an equal step in the shipped kernel (one cell each). A schedule that removed them would
  still pay its own selection cost, for example Max-Cut's tournament.

## Correction to record 79 (C)

Record 79 said no admitted Wikidata block lies at r/n 0.10–0.15. That holds for eager `Cells`
only, whose gate refuses records/n ≥ 0.10.
- *LazyCells* admits to 0.15, with a 0.32 n budget (`src/cells_lazy.rs:9–39`).
- *Per block:* it admits 228 of the 233 large blocks, at run-count r/n 0.053–0.143, including
  block 207. Eager admits 196.
- *So the Coordinator's regime is LazyCells'.*

**The batch rate there (K = 16, greedy Theorem 3′, record 79's model):**

| block | r/n | rounds with \|S\| ≥ 2 | mean width | chain shortened |
|--:|--:|--:|--:|--:|
| 143 | 0.120 | 87.9% | 5.25 | 5.15× (37,677 rounds against 193,865 steps) |
| 210 | 0.143 | 78.2% | 4.46 | 4.45× |

These match the lower-r/n blocks.
