# Critique of the Neon-role cycle 5 (N-20 to N-22) against rubric R1–R7

*Material: `research/neon-main-theorem`, cc3b76f3..c1d1beb5. An independent reviewer applied the rubric fixed in journal §32 and recomputed block-0 and the hex figures. The Coordinator spot-checked `build_retained_map`'s LF walk at `construct_n20_copy_and_sweeps.py`:226–233.*


Abbreviations: N20 = `n20-…md`; PY = `construct_n20_copy_and_sweeps.py`. Read-only review. Nothing on the branch was modified.

## Candidate × rubric

| Candidate | R1 | R2 | R3 | R4 | R5 | R6 | R7 |
|---|---|---|---|---|---|---|---|
| C1 Φ-sibling copy | not | not: 3.77–10.08·h₁ (N20:53–58) | **not**: the build is an LF walk from p, and its co-travel loop steps `lf` row by row (PY:318–352) | not | partial: "J₁ saved=0" is a constant asserted ==0 and never incremented (PY:377) | not | not |
| C2 attractor/LZ77 | not | not | **not**: LZ77 is computed on `T_true` (PY:417), an oracle audit | not | measured | not | not |
| C3 Gauss–Seidel on m₁₆ (M-17) | partial (MONODROMY's macro-edge) | not: rate e is a constant | **not**: R is built by an LF walk through thick rows (PY:226–233); sweeps cost Q = m₁₆ | not: 1 sweep 0.762–1.197·h₁; 2 sweeps 0.28–0.44·h₁ only at K=3 | partial | not | not |
| C4 arches on raw LF | partial: exact (N20:119), but an orbit segment, i.e. a macro-edge | partial: π ≈ n/3 on any content; on the six blocks 3.18–9.03·h₁ (N20:140–145) | **met**: cursors only, three passes, decode exact (PY:675–727) | not on blocks; on hex Q = 0.366·h₁ but P ≥ 3 > K = 2 and S = Θ(n) | partial | **not** (Astra/LMS, M-17; disclaimed at N20:121) | not |
| C4 at LIMIT=1 (m₁ = h₁) | partial | not | **not**: m₁ is built by walking LF (PY:226–233) | not: relocates Q(Acq) ≥ h₁ (N20:153) | partial | not | not |

**Verdict.** None starts to overthrow induction. C4 on raw LF is the only candidate that meets R3, but it fails R1 (partial), R4 and R6.

## (a) Free, or uncharged

"Q(Acq)=0" means uncharged, the same trap as Carbon's. The costs reappear elsewhere:
- **Passes.** Three streaming passes over L with n-entry J/w tables: the histogram (PY:675) and two sweeps, then the consumer. Neon itself says "at K=3" (N20:135), and K is frozen at 2.
- **Cursors.** s = 257 per sweep is within |Σ|+O(1) only if one register indexes three arrays.
- **State.** S ≈ n·(4+cap) bytes: about 10 MB on block-0, against openPeakScratch of about 0.36 MB.
- **G1's sum** is not lowered, because cursor composes touch all n rows.

Record 110's single cursor sweep already gives **0.412·h₁ at K=2** on the same hex. I recomputed it: GS1 has 21,776 links, the arches 19,321. That is 11% for one extra pass. Calling 3.000 "> e" sets two sweeps against one; two GS sweeps give 0.151·h₁.

## (b) Where the figures come from

- **0.366·h₁** is synthetic hex only (19,306 valleys; h₁ = 52,812).
  - Raw-LF C4 on the six blocks gives 3.18–9.03·h₁.
  - Recomputed on block-0: 307,347 links = 8.75·h₁.
- **0.325·h₁** (block-0: 11,417 links / 35,116, recomputed) bundles whole thick strips into words:
  - the longest word is 4,714 bytes;
  - 3,239 words are over 16 bytes;
  - 864,815 of 899,931 bytes (96%) ride free.
- **Missing from 0.325:** thick-run acquisition, the strip glue, and Q = m₁ per sweep.
- **Neon's lower bound is too low.** Its "≥ 1.36·h₁" undercounts its own items: two sweeps at Q = m₁ each, plus Q(Run), already exceed 2.3·h₁.
- **The commit subject is misleading.** 7ddd3cd pairs "Q(Acq)=0" with the LIMIT=1 figure.

## (c) Hidden walks

- **C4 on raw LF:** none. y comes from cursor state, never from `lf`, and the decode assertion passes.
- **`build_retained_map`** is a row-by-row LF walk over every thick row (PY:226–233). Its "Open uses strip jumps" (PY:220) is neither implemented nor costed.
- **C1** is a walk throughout.
- **C2** reads `T_true`.
- **Sweep keys** use uncharged `sorted()` (PY:813–821).

## (d) Proofs

- **Prop 20.1** is proved by witness, reproduced here: n=31, flip (1,8,10). Three caveats:
  - The flip is not "distant": rows 1 and 10 sit next to phrase rows 3 and 11.
  - "2-step neighbourhoods" is checked at only one step (PY:171–173).
  - It is near-trivial given record 110 §1 ("positions are the walk").
- **Lemma 20.2(1)** is trivial and mis-argued: LF^ℓ is a permutation, so x≠y gives LF^ℓ(x)≠LF^ℓ(y) whatever the match boundary.
- **Lemma 20.2(2)** follows only from the declared unit costs. "Strictly less S" is false for ℓ > 8: ℓ inline bytes against about 8 bytes of pointers.
- **Thm 20.3(1)** is trivial: peaks and valleys alternate.
- **Thm 20.3(2)** holds under i.i.d. ranks but is marked [proved]. Measured: 3.108.
- **Thm 20.3(3)** is true for Q and silent on P and S.

## (e) Prior art

- **Astra.** Astra investigation 1 (lines 575–685) has "cutting at each local minimum … two ordered sweeps". The Coordinator identified this as SA-IS LMS and induced sorting, with L/S types read locally from (F[x], L[x]). That is C4's object together with its two sweeps.
- **M-17.** Record 110's sweep contracts exactly the descent chains ("ascending runs of the inverse SA"). C4's sweep 1 is the same operation, and sweep 2 is its mirror. So the arch is an M-17 macro-edge of mean length 3 instead of e: chain contraction, the class of T²/Open.
- **Unsupported claim.** "Explains why Astra needed 12 rounds" has no analysis behind it in the file.
- **Kitchen-sink cycles.** A search found no other match.

## (f) N-21, |J₁ ∩ U_O| = 1

- **In words:** apart from origPtr's own row, no lone row has its text position fixed by (histogram, A, A₂, origPtr).
- **It follows for the scalar O:**
  - lower bound: p ∈ U_O always, and p ∈ J₁ on every block;
  - upper bound: 13–19 covering witnesses, each O(n)-verified for one cycle, the scalar moments and the positions.
- **REVIEW-5 handoff 2 overreaches.** It says the result holds "even with all 768 per-symbol moments". Per-symbol moments are asserted only for the minimal witness (N21 script:425). `full_verify_witness` checks scalars only (240–242), and the cover includes general quads (530–550).
- **Scope.** A negative result on a weak observation, not a construction.

## Discrepancies

1. **NEEDS M-17's cap unit.** Record 110 §4 says "words capped at 8 and at 16 bytes". On the byte reading, the 8-byte arm gives 2.056–2.286 (6/6 blocks below 2.3) and withdraws. Neon made the step cap primary and declared "stands". The verdict rests on a unit that was not fixed before the data.
2. **The commit subject** of 7ddd3cd is the mismatch described in (b).
3. **No raw output is committed**, only the md tables. Block-0 matched on recomputation.
4. **C1's zero** is labelled "proved and measured" but is a hard-coded constant.
5. **The "Reach: exact" row** overstates C3 and C4.

## Spot run (each under 2 s)

- Witness 20.1: reproduced.
- Hex arches: 19,306 links (0.366·h₁).
- GS1 on hex: 0.412·h₁.
- Block-0 C4: 307,347 links on raw LF and 11,417 at LIMIT=1.

## What would close each gap

- **C4, R4 at K=2.** Fuse the histogram into sweep 1, or run sweep 2 inside the consumer. Report P in bytes/n and S against S₀.
- **C4, R2 and R6.** Find a cursor-monotone cut rule (record 110's κ) whose segments lengthen on the real blocks. Contracting thick runs while keeping per-symbol ±1 monotonicity is Neon's own escape condition, and the right target.
- **C4-m₁.** Cost thick acquisition and glue in (Q, D, P, S), with byte caps and strip ids kept out of line.
- **M-17.** Get a ruling on which cap unit record 110 meant.
- **N-21.** Re-verify the cover for per-symbol moments, or drop the 768-moment sentence.
- **R7.** No candidate offers a supplied-state whole-decoder payoff.
