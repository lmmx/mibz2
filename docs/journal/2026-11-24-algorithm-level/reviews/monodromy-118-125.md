# Review of MONODROMY records 118–125 (f868068b..df9ffaf8)

*An independent reviewer, read-only. It reran the strip and chunk checkers, resimulated the min-key order, and recomputed record 120's route table. Attribution is clean on all 15 commits.*


**Hygiene:** all 15 commits are authored and committed by Louis Maddox <louismmx@gmail.com>. No AI attribution in subjects or bodies. CLAIMS.md adds DONE, BREAK and NEEDS lines for M-25 to M-32, and they match the records.

**Checks I ran** (scratch `m118/`): A₁₆(2) = 0.15625 ✓. Record 120's route table recomputes exactly (2.56, 1.34, 0.88, 3.26, 1.00). Record 121's chunk checker passes (3,000 streams), and so does record 120's strip checker (1,800 systems). I resimulated the min-key order on suffix ranks (n = 120k to 250k). The rounds and δ arithmetic in records 123 and 125 hold.

## Table

| rec | claim | status ok? | new? | moves |
|---|---|---|---|---|
| 118 | Theorem I: any order monotone on B pieces, interleaved arbitrarily, fails on ≥ A_σ(q) − σ(B−1)σ^(−q) of steps. The min-key order τ_j fails on 1/(j+2). The record 116 conjecture is broken. | Theorem I: yes. Its least-deficit argument checks: e_i ≤ e_{i−1} < σe_{i−1}. **"Exactly 1/(j+2)" is overclaimed** (see Errors). | Theorem I is new. Min-key is a known minimizer idea, and its use as a cursor schedule is new. | T1 floor: on 16 interleaved pieces Q ≥ 0.078·n_d. Depth 1 is now bracketed at 0.156 to 1/3. No cut. |
| 119 | One in-place sweep is the first-entry map into T, the failing set (an induction). Lemma W: the surviving classes are the return words. Conservation law \|T\|log C + S log\|T\| + W ≥ n log σ. So the form is closed under composition, but compactness is not. | Yes for the proofs. The regimes are argued, as labelled. Prediction 2 is mis-stated (see Errors). | The law and the identification of closure with relation C are new. The induction view is the Ko–Aluru/SA-IS decoding direction (cited). | Closure: a negative answer within the declared class. R² by label pair is the only composition within 256 cursors. Explains the 21–24× fragmentation. |
| 120 | Proposition A: label-ordered streams contract strips with no search and no addressed access, at one pass per level. Lazy S* rule: thin strips contracted one level, exact. Depth-2 schedule costed. | Proofs fine, checker passes. The block-0 figures in §4 are correctly labelled conjectured (2.5 rows a run and 4/5 are guesses). | The merge is the move structure (Nishimoto–Tabei, cited). The lazy S* rule is new. | Acquisition: eager contraction trades CL4's searches for P (up to 87 passes). Lazy: one pass, A ≈ 0. Projected total Q ≈ 0.6·h₁ (floor 0.55·h₁), not h₁/2. Depth 2 beats the sweep by about 0.12·h₁ only with a cursor table. |
| 121 | Proposition M: MTF + RLE2 + run records have an exact, composable chunk summary. The bit parse has none. | M is proved and checked. **§5 and M-28 withdrawn by 122** (premise: the scanner does Huffman only). | M is close to Patel et al. and to the pugz/rapidgzip markers (cited, unverified). | T3: M lets a team's helpers share the leader's MTF. Not the 19%. |
| 122 | Corrects 121. Inside a block the parse state is (bit position, N), with no small transformer (argued). Across blocks the magic-number parse is exact (lbzip2/pbzip2). The non-zero-followed-by-non-zero count is h₁ exactly, so n, r, h₁ and the width histogram are free counters of the scan. | Yes. The h₁ identity is right (h₁ = width-1 runs, record 96). The 2^(−48) arithmetic gives about 2.8·10⁻⁵ per GB ✓. | The multi-block parse is prior art. The free-counter observation is small but useful. Record 96 already noted h₁ costs one compare a run. | T4: the serial residue becomes one block's parse a file. T5: rider counters. |
| 123 | Measure S(k) (scanner alone, under k workers' load) and W(k) (arm C) directly, and use X = min(S, W) without fitting. Seven omissions. Estimator k* = least k with W(k−1) > S(k). | Argued; standard theory cited. | Not new (Denning–Buzen, Gunther). | Measurement design for T4. **Flags that arm C also frees the 11th thread's core,** so part of the 19% may be SMT, not seriality. |
| 124 | First priority is the price of a dependent read (regress emission time on links and bytes over 232 census blocks). Then the worker split by cycle counters with oracle confirmation, then misses and stalls a phase. Screening inequality L − L′ > A. | Argued; sensible. | Method, not mathematics. | Gives Q a price in seconds, so candidates can be placed under T1 to T6. |
| 125 | Six corrections to the brief, a three-layer linear design (about 90 conditions), roofline per stage as the primary model, control coefficients as validation, Coz. | Argued. The δ and rounds arithmetic checks (72 and 18 rounds). | Standard theory, as it says. | Measurement design (below). |

## Strongest result

**Record 119's conservation law, with Lemma W.** It turns ChatGPT's 21–24× fragmentation from an observation into a theorem-level explanation. In compact coordinates the surviving classes are the return words. Any successor-stream representation pays |T|log C + W ≥ n log σ, so composition cannot stay compact on high-entropy rows. This answers the closure problem negatively for the declared class, and it identifies closure with relation C. **Record 118's Theorem I** is the most rigorous new bound: it covers every interleaving, not only sequential blocks.

## Errors and overclaims

1. **118, "ε(τ_j) = 1/(j+2) exactly" (proved in the model).** The reduction is right: a step fails iff S^(j+1)u is the least of the window. The step from there to probability 1/(j+2) is not justified, because consecutive suffixes are not exchangeable. My simulation, with ties broken in text order:
   - σ = 2: j = 1 gives 0.333, but j = 2 gives **0.238** and j = 3 gives **0.186**, against 0.25 and 0.20 (about 10 SE off).
   - σ = 4, j = 2: 0.248.
   - Hex: 0.3338, 0.2504, 0.2004, which matches the record.

   So the result should read "≈ 1/(j+2) for large σ". Exactness is shown only for j = 0 and is plausible for j = 1. The conjecture that 1/(k+2) is optimal on 𝓜_k is already false at σ = 2, k = 2, since τ₂ itself goes below 1/4. The BREAK of record 116 is unaffected, and in fact strengthened.
2. **118 §5, "Q of order n/log²n for every in-place composition sweep".** This carries no status label and is absent from CLAIMS. It is an argued asymptotic, and it should say so.
3. **119 prediction 2 (raw addresses, ≥ 0.8·|T|(1−ρ)).** It is inconsistent with §2:
   - ChatGPT counted segments over all m = 89,874 sources, and §2's own reasoning gives about m(1−ρ). That matches the observed 44.8k–52.3k: 0.92–0.94 of m(1−ρ).
   - With |T| in place of m, the floor is about 16.7k, which the existing data already beats about 3×. As written, the prediction cannot fail.
   - Fix: replace |T| with m, the number of sources.
4. **The model baseline against the data.** Records 118–120 take row order to fail on 1/2. On block 0 the measured row-order sweep (M-17) already fails on 32,833/89,874 = 0.365, close to the 1/3 of the min-key order.
   - Record 120 §4 calls 1/3 the "best order on record". It is a model figure, and it needs 256 cursors.
   - The real-block gain from min-key may be small. NEEDS M-25's rule tests exactly this, so that run should come before anything is built on it.
   - Record 119 also says "half the rows" for T, against a measured ρ of 0.37–0.47.
5. **121 to 122:** correctly retracted (BREAK M-28). **116 §3.1:** correctly broken in 118 §4. Neither record is silently amended.
6. **123's grid is stale.** It was written after #31 (the 1–12, 14, 16, 20 grid) and before #32 (1–20), and it was not revised. Record 125's Layer A does use 1–20.
7. Nothing contradicts the four-arm data. Record 123 item 1 is a legitimate challenge to how arm C is read: arm C frees a core as well as removing the serial work.

## Design recommendations for the recipe brief (123–125)

**Grid and rounds**
- Layer A: every k = 1…20, five rounds, ABBA-interleaved arms within a round.
- Dense rounds at the five counts around the estimated crossing, and on both sides of the 10-thread break.
- Rounds per point: n ≥ 2(σ/0.02)². That is 5 rounds at 3% spread and 13 at 5%.
- Report per block and pooled. If block times vary by more than 25%, do not claim ±1 worker.

**New oracles and model**
- **S(k):** the scanner alone into a sink, with k threads of worker load running.
- **W(k):** arm C, measured per plan (teams × threads), not per k.
- Throughput X = min(S, W). The residual is itself a result.
- Shift point: k* = least k with W(k−1) > S(k), with its SE by the delta method.
- Compare 1 scanner + (T−1) workers against 2 scanners + (T−2) workers.
- Optional soft form: (S^(−p) + W^(−p))^(−1/p), or the USL with a break at 10.
- Rule: a departure above 5% at three or more counts away from the crossing means the two-stage model is wrong.

**Counters (Layer A)**
- Cycle counters at every stage boundary, per thread, giving busy and waiting time.
- A separate leader-only time and helper idle time per team.
- LLC misses, stalled cycles and dTLB walks per stage.
- Construction and emission each split into a streamed timer and a dependent timer.
- Per-block covariates: n, r, h₁, m₁₆ (n, r, h₁ and the width histogram come free from the Huffman scan).
- Warm and cold first block, reported separately.
- Census regression (record 124): emission time on links and bytes over the 232 blocks, giving the price of a link and the misses a link.

**Perturbations (Layer B, at k = 1, 6, 10, 14, 20, one knob at a time)**
- Huge pages on and off.
- Turbo off.
- Pinning to distinct cores against sibling pairs.
- A cache antagonist (4 and 8 MB).
- A bandwidth antagonist.

**Slowdowns (Layer C, validation only)**
- A proportional spin of δ·(that block's stage time), touching no memory, with δ = 0.2.
- Near k*, use δ = 0.1 and 0.3.
- Run arms paired on the same input.

**Model form**
- A roofline per stage: compute/f(k) + dependent reads × ℓ(k) + bytes/β(k).
- Fit on k = 1, 2, 4, 8, 12, 16, 20 and predict the rest.
- Rule: a miss above 5% at three or more held-out counts means an element is missing. Suspect the team's internal split or address translation first.
- Try Coz: it may replace most of Layer C.

**Reason about, not measure**
- How ceilings combine: additive within the workers, a minimum across scanner and workers.
- Amdahl twice: across the line, and inside a team at 1/(f + (1−f)/s).
- Bandwidth from P·S × output rate. Drop the bandwidth antagonist if that product is under a tenth of the machine's bandwidth.
- Screen candidates with p(L − L′) > p·A before building anything.

## Disagreements with briefs/decomposition.md

- **Perturbation size:** the brief has 5–10%. Record 125 has δ = 0.2: at 0.1 a coefficient to ±0.05 needs about 70 rounds.
- **Summation theorem:** the brief reads a shortfall as a missing stage or resource. Record 125 says it checks completeness of *time* only (sleeps, back-off, quanta, I/O), not of resources.
- **Formalism:** the brief has control coefficients as a primary method. Record 125 says they are 0 or 1 away from the knee, and makes the roofline primary.
- **Stage table:**
  - add the leader/helper split, which is the biggest omission;
  - split S5 and S6 into streamed and dependent halves;
  - split R1 into cache share, bandwidth and TLB;
  - add the clock (turbo) and the serial tail (reorder and output, which the brief puts under the workers in S7);
  - drop NUMA;
  - keep heterogeneity as covariates.
- **Allocator (R2):** not addressed by record 125.
- **Queue:** the queue bound is a parameter of W, not a smoothing knob (record 123).
