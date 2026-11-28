# 2026-11-19 · MONODROMY 79 · Gemini 4 Neon's Theorem 3′ verified exhaustively: exact and pointer-disjoint, its backward mark sound. Admissible batches are rare on small columns (4.4% of states) but common on admitted Wikidata blocks (67–91% of rounds; the dependent chain 3.8–4.9× shorter). Theorem 2′'s counts are exact, but right-end Rauzy–Veech does not pass through I(P_k). Neither shipped induction is latency-bound on this machine (IPC 2.9 and 2.6; the fetch-ahead is worth 8–12%)

**The request.** The Coordinator's update to CHECKPOINT 7005355308, in two cross-session
messages.
- *The text:* PR #627's `response-2.md`, at 457e234c and then 76eb5e65.
- *The asks:* audit Theorem 3′ and Theorem 2′; give the batch rate on small cases and on an
  admitted Wikidata block; say whether the shipped induction is latency-bound at all.

I did not open Neon's `verify_thm3.py.txt`, `batchrate.py` or the README's "Round 3" before
these checks.

**Labels:** **P** · **C** · **M** · **I**.

**Scripts** (beside this record as `.txt`):
- `neon3p.py`: Theorem 3′ exhaustively. It loads record 78's `neon3.py`, the coordinate model;
- `neon3scale.py`: the batch rate on real blocks;
- `neon2p.py`: Theorem 2′;
- `latency.rs` and `latency-patch.diff`: the induction timer. It was built in a scratch worktree
  of master 0c852a07 and is not for the repository.

## Verdicts

| # | claim | verdict |
|---|---|---|
| 3′.2 | Under Conditions 1–3, a batch is exact and confluent | **VERIFIED**. 29,395 batches were exact |
| 3′ Part 2 | No two fronts share a node, sentinels included | **VERIFIED**. No shared node, read or written, in any batch |
| 3′.3 | The backward mark in domain coordinates, with Case 2's relabel | **VERIFIED**. All 242,216 runs output the right rotation |
| Defect 1 | Cached (A_j, B_j) go stale across components | **VERIFIED by hand** on its n = 4 example. Record 78 did not see it: my pointer model always read the fronts off the sentinels, which is 3′'s repair |
| Defect 3 | Case 3a's footprint omits cod_prev(A_j) | **VERIFIED as a node-level conflict.** At field level the two fronts write opposite links, which is why record 78 counted no conflict |
| rate | How often \|S\| ≥ 2 is admissible | **Small columns: rare** (4.4% of states with two or more live fronts). **Admitted Wikidata: common** (67–91% of rounds; mean width 4.0–5.3) |
| 2′ | Fibonacci counts | **VERIFIED**, all 120 (q, k) for q ≤ 18 |
| 2′ | "Factored RV from X₀ to I(P_k) in k − 3 steps" | **VERIFIED WITH CORRECTION**. Right-end RV reaches I(P_k) itself only at k = 4. It reaches a section with I(P_k)'s size and return times |
| 2′.1 | Kac scope | **VERIFIED** (it now says "explicit row stepping only") |
| latency | Is the shipped induction latency-bound? | **No, on this machine** (M). Eager runs at IPC ≈ 2.9 and is flat in the fetch-ahead distance; lazy runs at IPC ≈ 2.6 |

## Theorem 3′ (P, C)

**The checker.** The response states its construction exactly, and I implemented it as stated.
- *Lists and fronts:* sentinels bracket each component's domain and codomain lists. The fronts
  are A_j = dom_prev(dom_right[j]) and B_j = cod_prev(cod_right[j]).
- *Footprints:* the per-case sets F_j.
- *The mark:* (I_p, θ_p, δ) in domain coordinates, transported backward.
- *Simultaneous execution:* each front of a batch reads the pre-state, except where it has
  written the field itself. The writes are merged afterwards.
- *What is counted, per batch:* write–write collisions, stale reads, and any node shared by two
  fronts. This counts every field of every node, sentinels included.

**Coverage.**
- *Columns:* all over 2 letters to length 8, 3 letters to length 7 and 4 letters to length 6.
- *Pins and marks:* every subset of internal alphabet boundaries as pins, and every start row.
- *Batches:* at every state, every subset of two or more live fronts.
- *Runs:* every run then advances by the greedy 3′ batch, closures alone.
- *Comparison:* everything is checked against the coordinate model, which is itself checked
  against direct first returns (record 78).

| | count |
|---|--:|
| full runs; output equal to p₀'s LF stream rotated by δ | **242,216 of 242,216** |
| states | 1,357,220 |
| subsets of ≥ 2 live fronts tested | 822,112 |
| …meeting only Condition 2 (A_j ≠ B_j′) | 228,843 |
| …3′-admissible (Conditions 1–3) | **29,395**: all exact; 0 write collisions, 0 stale reads, 0 shared nodes (sentinels included); mark invariant 0 ≤ δ < \|ω(I_p)\| held throughout |
| rounds advanced by greedy 3′ batches | 1,357,220: 0 conflicts, 0 mismatches against the coordinate model |

**The proof.**
- *Part 1* is the argument record 78 accepted for round 1: T_X(Y) ∩ Y = ∅ follows from
  A_j ≠ B_j′, and Condition 3 implies Condition 2, as stated.
- *Part 2's* sentinel case analysis matches the count: no admissible batch shares a sentinel.
- *Part 4's* backward transport closes in O(1). Whenever p ∈ Y_j, the predecessor T_X⁻¹(p) lies
  in dom(B_j) at a known offset, so no coordinate search is needed. That is the defect record 78
  noted from the other side, where a forward mark needed codomain coordinates.

**What round 1 missed.** Record 78 did not find Defect 1, because its pointer model never cached
the fronts. For the same reason, its "exact at pointer level" verdict covered the sentinel
reading only, not the response's literal `cod_front[j] = B_j.cod_prev` maintenance.

## How often a batch is admissible (C)

**Small columns: rarely.** Few intervals means fronts collide.
- *States with ≥ 2 live fronts:* 482,668. Of these, 21,343 (4.4%) admit any 3′ batch of two or
  more.
- *Greedy runs:* 16,782 of 432,961 such rounds (3.9%) execute two or more fronts.
- *Agreement:* this matches the Coordinator's 15 of 3,501 from Neon's verifier, a different
  sample.

**Admitted Wikidata blocks: mostly.** The density gate refuses records/n ≥ 0.10 and below 1/64
(master's `src/cells.rs:575–583`), and the floor and budget gates refuse more.
- *Which blocks are admitted:* I decoded `corpora/wikidata.bz2` with `Cells` and read the admitted
  count block by block. 196 of the 233 large blocks are admitted, at run-count r/n 0.053–0.088.
- *So "admitted at 0.10–0.15" is outside the gate.* Block 207 (0.097), which I first took as the
  top, is refused.
- *The blocks used:* block 0 (the frozen column), block 2 (near the median) and block 85 (0.088,
  the highest admitted).
- *Pins:* the internal alphabet boundaries nearest equal mass.
- *Model:* the pointer model at full size, greedy 3′ batches each round, closures first.

| block | r/n | K | cells (K = 1) | cells | rounds | mean width | rounds with \|S\| ≥ 2 | rounds running every live front | chain shortened |
|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| 0 | 0.057 | 16 | 92,570 | 95,776 | 24,079 | 3.98 | 0.801 | 0.990 | 3.84× |
| 2 | 0.061 | 16 | 101,906 | 109,733 | 20,764 | 5.28 | 0.911 | 0.986 | 4.91× |
| 85 | 0.088 | 16 | 140,543 | 136,734 | 30,070 | 4.55 | 0.672 | 0.988 | 4.67× |
| 2 | 0.061 | 28 | | 103,890 | 28,128 | 3.69 | 0.352 | 0.987 | 3.62× |
| 85 | 0.088 | 29 | | 139,754 | 33,810 | 4.13 | 0.420 | 0.976 | 4.16× |

Block 207, which is refused, gave 4.89× at K = 16 under the same model.

**What limits the width.** The footprint condition almost never does: about 99% of rounds take
every live front. What sets the rounds is the heaviest component, which does 19–25% of the cells
at K = 16 (24,000 of 95,776 on block 0; on block 85, the `"` bucket's component has 22%). Pinning
changes the cell count by −2.7% to +7.7%.

**K is not monotone.** Asking for 32 pins gives 28 or 29 distinct ones, and that moves the heavy
buckets into worse components: there are more rounds than at 16.

## Is the shipped induction latency-bound? (M, I)

**The machine.** This container is a Xeon of model 207 (Emerald Rapids). Each core has 2 MB of
L2, and the L3 is 260 MB. A dependent 3-cycle `imul` chain gives a clock of 3.12–3.24 GHz. A
block's whole working set (about 5.5 MB) fits in L3.

**The measurement.**
- *What was timed:* only the induction call (`induce_variant`), with `Instant`, on one pinned
  core.
- *Blocks:* every 2nd block of `corpora/wikidata.bz2`, 216 blocks, of which the admitted ones
  give 7,142,076 eager cells.
- *Repeats:* the best of 3 passes per run, 3 interleaved runs per variant.
- *The variants:* rebuilt with FAR (NEAR) at 4 (2), 8 (4), 12 (6, shipped), 24 (12) and 32 (16),
  and with the prefetch removed. For FAR > 16, BPAD was raised to 512.

| variant | ns a cell (min / median of 3) |
|---|--:|
| no prefetch | 7.33 / 7.34 |
| FAR 4 | 6.52 / 6.74 |
| FAR 8 | 6.62 / 6.68 |
| FAR 12 (shipped) | 6.47 / 7.04 |
| FAR 24 | 6.58 / 6.68 |
| FAR 32 | 6.63 / 6.69 |
| lazy (no prefetch; its own cells, 9,859,817) | 12.45 / 12.63 |

**Instructions.** Callgrind on every 24th block:
- *eager induction:* 61.7 instructions and 4.0 conditional branches a cell;
- *lazy induction:* 104 instructions a cell. Its per-cell figure includes the letter work that the
  eager kernel's `slots` does before the induction.

**What follows.**
- *Eager:* 6.5–7.0 ns at 3.2 GHz is 21–22 cycles a cell, so IPC ≈ 2.9.
- *Lazy:* 12.5 ns is about 40 cycles a cell, so IPC ≈ 2.6.

**Neither is latency-bound.** A chain of dependent L2 or L3 misses would run below IPC 1.
- *The fetch-ahead* is worth 8–12% here, and is flat from 4 to 32 slots. Whatever latency
  remains is already covered at a distance of 4.
- *On the workstation (i9)* it was worth about a quarter (record of 10 November, §6.2), so the
  memory component is larger there. That machine was not measured today.

**Interpretation, not measured: what a shorter dependent chain could buy.**
- *The instruction floor:* at 62 instructions a cell and a sustained width of 4–6, about 10–15
  cycles.
- *What is left:* the remaining 6–11 cycles a cell are the loop-carried chain (an L1 load, then
  the three-way compare) and branch mispredictions. They cannot be told apart here: this
  container has no hardware counters, and callgrind's branch model is not the core's.
- *Interleaving fronts* can hide only the first of those. A mispredict flushes every front.
- *The ceiling:* about 1.4–2× for an interleaved K-front eager kernel. Against that, add the
  footprint checks and 1–8% more cells.

This decides nothing until a front-interleaved kernel is timed. It needs a workstation run, and
ideally hardware counters for mispredicts.

## Theorem 2′ (P, C)

**What I checked.** I used the response's own indexing: standard F (F₁ = F₂ = 1); w₁ = b,
w₂ = a; P_k the prefix of length F_k − 2. For every q from 4 to 18 and every k from 4 to q
(120 cases):
- |I(P_k)| = F_{q−k+3};
- there are two pieces;
- the return times are F_{k−1} with multiplicity F_{q−k+2}, and F_{k−2} with multiplicity
  F_{q−k+1};
- W = F_k.

**All exact.** This is record 78's corrected statement, shifted to standard indexing.

**The one claim that fails** is "factored Rauzy–Veech induction from X₀ to I(P_k) performs k − 3
steps".
- *What right-end induction visits:* the sections [0, m).
- *Where I(P_k) lies:* for k ≥ 5 it is an interior block of rows (q = 10: rows 13–33 at k = 5,
  rows 20–21 at k = 10). The right-end chain reaches it only at k = 4, the cylinder of a: 15 of
  the 120 cases.
- *What the chain does reach:* after k − 3 steps, a section with exactly I(P_k)'s size, return
  times and multiplicities. That holds in all 120 cases, and those sections are prefixes of rows.

**The correction.** "Right-end RV reaches, in k − 3 steps, a section with I(P_k)'s size and
return words." The separation, O(log n) factored against Θ(n) explicit, is unaffected.

## What this decides

Nothing in production.
- *Theorem 3′* is correct as stated, and on admitted Wikidata blocks it would shorten the
  induction's dependent chain about 3.8–4.9×.
- *The shipped kernels* are not bound by that chain's memory latency on this machine.
- *What would decide its value:* timing a front-interleaved kernel.
