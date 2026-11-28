# 2026-11-24 · MONODROMY 99 · Main-theorem cycle 1. Rauzy–Veech is inside 𝓡 in one currency, so it is not the answer. Record 98 corrected: list ranking and contraction are inside 𝓡 too, and class facts are no escape. That leaves E2's global cell as the only open escape. The target needs a margin δ. The main theorem is drafted conditionally, with an additivity lemma and three open premises

**The requests:**
- the Coordinator's CORRECTION 7128643478, two additions to CHECKPOINT 7128531277;
- the withdrawal 7128808679, acknowledged;
- QUEUED 7128837429, the main-theorem challenge with Neon.

Neon's branch `research/neon-main-theorem` did not exist at 10:33Z, so this cycle has nothing to cite or break.

**The draft:** [`2026-11-24-main-theorem/main-theorem.md`](2026-11-24-main-theorem/main-theorem.md) is the section for PROOF-PROGRAMME.md, in pseudo-Lean.

## 1. Rauzy–Veech is not the answer [argued; it rests on G1]

**Induction steps and walk steps are one currency.** An RV step acts on a domain piece, and on a width-1 piece it is a rule application on a width-1 subject. So c₁ counts it.

**The consequence:** G1 gives c₁ ≥ h₁ for Cells and for every RV schedule. No separate appeal to the induction floor r is needed. Record 92's Theorem 4 already said that every run transported costs at least one operation.

## 2. Corrections to record 98 (`break:` on my own claim)

- **E3 is inside 𝓡.**
  - *What record 98 said:* "sort-based list ranking has c₁ = 0". **That is wrong.**
  - *Why:* a pointer jump composes x→LF(x) with LF(x)→LF²(x), which is a rigid fact on a width-1 interval. Independent-set contraction splices x by composing pred(x)'s fact with x's.
  - *The counts:* list ranking has c₁ = Θ(n log n), and contraction Θ(n). Both are ≥ h₁. Parallelism lowers the depth, never the count.
  - *What changes:* the budget K is not needed to exclude them. K stays as a utility guard against wasteful non-rigid procedures.
- **E1 is no escape [argued].**
  - *The fact:* a class fact "LF^k restricted to D_u is order-preserving onto I_u" is, at k = 1, LF's definition in rank form.
  - *Why it doesn't help:* evaluating it at a row is that row's primitive fact. It gives successors in bulk, not text positions, and turning it into output order needs the chain or E2.
  - *What changes:* record 98's E1 row stops being an escape.
- **The only open escape is E2's global cell:** positions by a non-rigid, non-O(1)-depth procedure within K passes. That is stream N's territory.

## 3. The target needs a margin

**The problem:** "c₁ < h₁" is met by saving one event. `origPtr` alone gives one free positional fact on every block.

**The proposal:** c₁ ≤ (1 − δ)·h₁, with δ fixed in advance. I suggest 1/2; it is Louis's to fix.

## 4. Utility, chain by chain (CORRECTION item 2)

- **Within 𝓡:** none can clear the target, so utility doesn't arise.
- **E2 local, O(1)-depth or labels-first:** closed.
- **E2 global (open):** utility unknown until a construction exists. Its ceiling is the cost-weighted share of h₁ it could remove.
- **The split decides that ceiling.** By §5's additivity, with stream R at its supportable aR = 1, the margin needs h₁R ≤ (1 − δ)·h₁, and stream N nearly free.
  - *h₁R under σ_hex:* is a census column. I haven't measured it: **TODO**, for the Coordinator or an agent.
  - *On the six blocks, from record 96:* hex width-1 runs are 25–40% of m₁₆, and h₁ is 31–48%. So roughly 75–85% of h₁ is hex. That makes **δ = 1/2 out of reach unless stream R beats aR = 1** [measured on six blocks, arithmetic].

## 5. The main theorem (drafted, in pseudo-Lean)

**The statement:** a conditional existence theorem over 𝓕 and K, with:
- **P0:** a cheap split σ, decidable from records;
- **P1:** additivity, c₁ = c₁|N + c₁|R [proved: each width-1 event has exactly one row];
- **PN, PR:** the streams' coefficients aN and aR;
- **Pglue:** exact interleaving of the two streams' outputs;
- **Pbudget:** passes within K;
- **Pmargin:** the margin met.

**The open premises:**
- *Pglue:* each boundary between an N-segment and an R-segment of the orbit is an event, so it is charged to c₁ unless computed positionally. That is a named missing step.
- *PR beyond aR = 1.*
- *PN:* Neon's.

**Also in the draft:** what a "no" is (the open cell shown empty on 𝓕), and utility kept as receipts outside the proof.

## 6. Stream R: status

**Worst case:** aR ≥ 1 for every decoder in 𝓡 [proved, given G1].

**Best case:** 0, only through E2's open cell.

**The conjecture** (aR ≥ 1 − o(1) for model-I content within K passes) is **not attempted beyond this:**
- every non-positional escape is now closed;
- a positional procedure for random rows must respect 3-byte global sensitivity.

**Next cycle:** try to turn global sensitivity plus model I into a lower bound on positional procedures for random rows. That is item M-3.

## 7. Postmortem

1. **Inputs:** nothing new was measured. The hex shares are record 96's, on six blocks.
2. **Model:** 𝓡 and c₁ as in record 98, corrected here.
3. **Method:**
   - P1 is proved by attribution.
   - E1's and E3's closures are argued, by expressing them in the rules.
4. **Outcome:**
   - Two of record 98's claims are corrected (E3, and E1's status).
   - The target's weakness without a margin is found.
   - The theorem is drafted. Its open premises are PR, PN and Pglue.
