# 2026-11-19 · MONODROMY 82 · Neon's run-collision floor F_self_ns is valid only for in-place, write-once, Φ-sourced fragment schedules; outside that class it fails (a 16-byte text needs 12 Φ fragments but 4 LZ77 copies). Inside it, no constructed schedule reaches it, but per-run potentials need about 110,000 fragments (6.9–7.7 B) on every block, 0.87–2.15 r. That is below Cells' step count on the high-r/n blocks, though its labelling still needs an O(n) arc walk. The depth floor bounds depth only. The sweep floor holds for byte sweeps but does not cover round 5's peeling, which beat it on block 0

**The request.** The Coordinator's CHECKPOINT 7008079186 (#346). The source is PR #632, branch
`docs/neon-round6` at 5e44e754, file `docs/journal/2026-11-19-gemini-neon/response-6.md`. The
findings are for Louis.

I did not open `verify_phi_obstruction.py.txt` before these checks.

**Labels:** **P** · **C** · **I**.

**Scripts** (beside this record as `.txt`):
- `neon6.py`: the real blocks. Its input is a column and its primary index; record 80's export
  supplies both.
- `neon6small.py`: every small binary text.

## Verdicts

| # | claim | verdict |
|---|---|---|
| 1 | F_self_ns bounds every schedule of the Φ copy scheme, "even omniscient" | **VERIFIED WITH CORRECTION, as a class bound only.** It holds for schedules that write each byte once, in place, by a fragment copied from its Φ-source after that source is final. It fails with temporary buffers or other sources; the explicit counterexample is below |
| 2 | Can any schedule approach the floor? | **Not reached; but in-class schedules are closer to Cells than the floor suggests.** Per-run potentials need 107–114k fragments on every block (0.87–2.15 r, 6.9–7.7 B) against floors of 0.40–0.85 r. BFS gives 3.9–9.6 r. That is below Cells' steps on blocks 85 and 210 (0.76× and 0.52×), above them on blocks 0 and 2 (1.19× and 1.12×). Building τ needs every byte's run label, an O(n) arc walk, unless that can be avoided |
| 3 | The depth floor λ_max − 1 | **VERIFIED as a bound on dependency depth** (and so F ≥ λ_max − 1). It does not bound total work, which is n bytes for every schedule. The potential schedule is within 0.1% of it |
| 5 | The alternating-sweep floor max_a alt(R_a) | **VERIFIED WITH CORRECTION.** The argument holds for byte-level alternating sweeps. On blocks 0, 2, 85 and 210 it is 0.42–0.61 λ, and it exceeds round 5's pass count on each (16,102 against 14,931 on block 0). So it does not "explain" those passes: peeling is not in its class |

## 1. The class the floor binds (P, C)

**The argument is sound inside its class.**
- *The setting:* take a disjoint copy (S_i ∩ D_i = ∅) and a fragment I ⊆ D_i holding t(x) and
  t(y) of one run, x < y.
- *Why that fails:* t(y − 1) ∈ Φ(I) depends, through its run chain, on t(x) ∈ I. So Φ(I) cannot
  be final before I is written.
- *The greedy:* the left-to-right greedy duplicate-free partition is optimal by the stated
  exchange argument.
- *Self-overlapping copies* are charged one fragment each. On block 0 there are none.

**The assumptions it rests on:**
1. Every destination byte is written exactly once, at its final position.
2. A fragment's source is its Φ-image, the run-predecessor bytes, and the copy reads only final
   bytes.

So it is a floor for replaying M_Φ in place.

**Each relaxation the Coordinator lists breaks it.**
- *Temporary buffers:* "fragments" stop counting operations. A schedule can produce any byte
  order in scratch and write each D_i with one memcpy. The work moves into producing the scratch,
  which the floor does not price.
- *Other sources:* copying from any equal, already-final stretch of output, as LZ77 does with
  self-overlap, is a valid replay. It is not bound by run collisions.
  - **Counterexample** (`neon6small.py`): text `aaaaaabaaaaaaaab` (n = 16, r = 4). F_self_ns = 12
    fragments, plus 4 literals. A left-to-right LZ77 schedule writes the same output in **4
    operations in all**.
  - *How often:* over all 131,068 binary texts of length 2–16 whose BWT is one cycle, LZ77 uses
    fewer operations than r + F_self_ns in 128,936.
- *Arbitrary order or right-to-left self-overlap* within the Φ scheme does not escape the floor.
  The duplicate-run argument uses only the dependency, not the direction.
- *A secondary structure built from the positions,* such as a grammar, an LZ parse, or a per-run
  order, is again a different copy scheme. Cells' SLP is one. The floor binds none of these.
  Their cost is the cost of building that structure: Policriti–Prezza's RLBWT-to-LZ77 conversion
  is O(n log r), and Cells' induction is 1.68–1.87 r steps.

**The exact class:** in-place, write-once, Φ-sourced fragment schedules of M_Φ, in any execution
order, with self-overlap allowed only within a self-overlapping copy. For that class the floor is
valid.

## 2. Attainability in that class (C, I)

**How a valid schedule is built.**
- *The rule:* each non-literal row x gets a time τ(x), strictly increasing along its run, since a
  byte's source is its run predecessor. A fragment is a maximal stretch of one copy D_i at one τ.
- *Why it is valid:* execute in increasing τ. Every source has a smaller τ.
- *What it covers:* every in-place Φ schedule is of this form, so the best τ is the class
  optimum.

**Constructed.**
- *BFS* is τ = θ, the run offset.
- *Potential* is τ = θ + φ(run). A pair of text-adjacent bytes from runs a and b merges exactly
  when φ_b − φ_a = d_b − c_a. φ is chosen on a maximum-weight forest of the (a, b) overlaps,
  weighted by mergeable pairs, and every pair is kept that is consistent around a cycle.
  - *Why one slope for all runs:* two runs can merge across an overlap only with equal slope, so
    this is the natural affine optimum.
  - *Not tried:* step-function potentials, which could satisfy overlaps in part.

| block | r | k copies | floor F_self_ns | BFS | potential | mergeable pairs kept | depth: floor / potential | sweep floor max alt |
|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| 0 | 51,184 | 10,954 | 41,684 = 0.814 r (20.4 B) | 493,738 = 9.65 r (1.7 B) | **110,204 = 2.15 r (7.7 B)** | 738,543 / 837,793 | 26,406 / 26,431 | 16,102 |
| 2 | 54,532 | 13,223 | 46,310 = 0.849 r (18.3 B) | 516,717 = 9.48 r (1.6 B) | **114,432 = 2.10 r (7.4 B)** | 731,036 / 832,245 | 26,318 / 26,389 | 15,593 |
| 85 | 79,375 | 17,119 | 43,479 = 0.548 r (18.9 B) | 487,229 = 6.14 r (1.7 B) | **107,456 = 1.35 r (7.6 B)** | 713,104 / 803,441 | 25,629 / 27,166 | 10,638 |
| 210 | 128,732 | 26,748 | 51,741 = 0.402 r (14.9 B) | 499,968 = 3.88 r (1.5 B) | **111,991 = 0.87 r (6.9 B)** | 659,236 / 744,479 | 27,233 / 27,307 | 11,623 |

Against Cells' steps (record 80: 92,571, 101,907, 140,544 and 216,769), the potential schedule's
fragments are **1.19×, 1.12×, 0.76× and 0.52×**. Its fragment count is nearly constant, at about
0.12 n, while Cells' steps grow with r.

**What this says.**
- *Against the floor:* the best in-class schedule found is 2.2–2.6× the floor on every block. On
  the high-r/n blocks it is below Cells' step count. The floor ignores the cycles of the copy graph, which is where the giant SCC acts.
  Every overlap left unsatisfied by the potentials (99,250 pairs on block 0) is a break.
- *Interpretation:* a tighter class bound would count the overlaps that stay unsatisfiable around
  cycles. It is not proved here.
- *Against Cells, by count only:* a fragment (a memcpy of about 7 B, scheduled by τ) and a Cells
  step (an interval rewrite plus its share of the writer) are not the same unit. The
  comparison is unmeasured.
- *The real obstacle is the labelling.* Building τ needs each copy's sequence of run labels, and
  that means walking every arc: n rows, which is the walk Cells avoids. The boundary positions
  alone do not give the fragment boundaries. If those labels, or the overlap transitions, could
  be enumerated in O(k + #overlaps) from the positions, in-place copy replay would be a
  candidate on high-r/n blocks.
- *Outside the class,* replay competes only after building another parse, whose cost is the
  question.

## 3. The depth and sweep floors (P, C)

**Part 3, the depth floor.** The run chain t(d_a) → … → t(e_a) has λ_a − 1 edges. In a DAG
schedule they lie in distinct fragments on one path. So depth ≥ λ_max − 1, and therefore
F ≥ λ_max − 1 (26,406 on block 0, weaker than F_self_ns).
- *What it does not bound:* total work. Every schedule moves n bytes.
- *How close a schedule gets:* the potential schedule's depth of 26,431 levels on block 0 is within
  0.1% of the floor, at 2.15 r fragments.
- *Scope:* the floor holds for non-self-overlapping schedules, as stated.

**Part 5, the sweep floor.** In a left-to-right byte sweep, t(x − 1) → t(x) resolves only when
t(x − 1) < t(x), and in a right-to-left sweep only when t(x − 1) > t(x). Successive monotone
pieces of a run need passes of alternating direction, so P ≥ max_a alt(R_a). This holds for
byte-level alternating sweeps.
- *On block 0:* the longest run (λ = 26,407) has alt = **16,102 = 0.610 λ**, not ≈ λ/2.
- *On blocks 2, 85 and 210:* 15,593 = 0.592 λ, 10,638 = 0.415 λ and 11,623 = 0.427 λ.
- *Against round 5:* its interval peeling took fewer passes than this floor on every block
  measured: 14,931, 14,361, 10,195 and 10,866 on blocks 0, 2, 85 and 210. It escapes because a
  peeling pass executes ready fragments in any order, not one sweep direction. So the floor is valid for its class but does not explain round 5's passes, as
  response-6 says it does.

## What this decides

Nothing in production.
- *The in-place Φ copy scheme is not closed.* About 110k fragments per block is fewer
  operations than Cells' steps when r/n ≥ 0.088.
- *What it needs:* a labelling that avoids the O(n) arc walk.
- *What remains open:* whether the overlap transitions can be listed from the positions in
  O(#overlaps), and whether an acyclic parse (LZ-type) can be built from the positions in o(n).

## Addendum: the minimum acyclic interval refinement (C)

This answers the Coordinator's follow-up: its Kahn check found Neon's minimal partition cyclic on
all six blocks, and it asked for three quantities on block 0.

**(a) F\*, the minimum fragments of any in-place Φ-copy schedule.** Only bounds are known.
- *Lower:* F_self_ns = 41,684 = 0.814 r is still the only proved lower bound.
- *Upper:* 106,887 = 2.088 r, from (b).
- *Why a cycle-packing bound fails:* it would bind only refinements of the greedy partition,
  while F\* may use a different partition altogether. So it is not a bound on F\*.
- *Open:* a tighter lower bound needs an argument about unbalanced cycles in the run-overlap
  graph.

**(b) A constructive scheduler I can run.** `neon6merge.py` starts from the per-run potential
schedule above. It then merges adjacent fragments of a copy whenever a single time exists that is
later than all their sources and earlier than all their dependents, with times as floats. Each
result is checked valid: every byte's source fragment executes strictly earlier.

| block | start (potentials) | after 51 merge rounds (cap reached) |
|--:|--:|--:|
| 0 | 110,204 = 2.153 r | **106,887 = 2.088 r** (7.9 B) |
| 210 | 111,991 = 0.870 r | **107,008 = 0.831 r** (7.2 B) |

**(c) The cost of discovering (b).**
- *What it needs:* every copy's sequence of run labels along its arc, and every byte's source and
  dependent fragment. That is a byte-level pass over all n rows, the walk Cells avoids.
- *What the positions give:* the copies and their lengths (O(k)), but not where inside a copy the
  run label changes.
- *So:* below about 1 r (block 210, 0.83 r) the count would make copy replay a candidate, but only
  with an o(n) way to list each copy's run transitions. On block 0 (2.09 r) it is above Cells'
  1.81 r even before that cost.
