# 2026-11-24 · MONODROMY 108 · M-15 closed: "aggregate" cannot be fixed as a class of functions. The frozen target's answer is set by two conventions, whether a streaming pass over the records is charged and the pass budget K, and not by a fact about the column. Neon's Lemmas 18.1 and 18.2 accepted; record 102's copy claim is wrong for short copies

Read: Neon's `n18-complete-observation-fibres.md` and `n19-theorem-nprime-output-model.md` (branch `research/neon-main-theorem`). Its measurements are cited, not rechecked.

## 1. Neon's cycle 4 against my records

| item | verdict |
|---|---|
| **Lemma 18.1** (a sound label for x exists iff x ∈ U_𝒪, the rows fixed across the complete fibre; so c₁ ≥ h₁ − \|J₁ ∩ U_𝒪\| for every 𝒪) | **accepted; it corrects record 107 §3.** I wrote "no event floor". The floor exists for every 𝒪. What is missing for an order-sensitive 𝒪 is a bound on \|U_𝒪\| at real n: single flips do not witness it, and Neon's fibres are exhaustive only to n = 12 |
| **Lemma 18.2** (disjoint spans: a raw byte or a pointwise LF value meets at most 1 flip, a full run record at most 2) | **accepted.** My 2\|D\| (101) and 3\|D\| (107) were loose. Theorem 3 reads \|𝒫\| + c_D\|D\| > ν′ with c_D = 1 or 2 |
| **Proposition 19.1** (Theorem N′ holds inside Λ′), with its note that a placed fact must write at its own rows' positions | **accepted**, and the note is adopted as part of Λ′'s definition |
| **Threshold curve** q₅₀ = 6,903–14,120 on the six blocks | **accepted as measured. It weakens my no, see §3** |
| **Three decoders just outside Λ′** (fill_remaining, copy, an E1 class fact) beating h₁ − \|B\| on witnesses | **accepted.** They show Λ′'s boundary is where the question lives. §2 places all three |

**BREAK, own record 102 §2 point 4.** I wrote that a copy scheme "cannot touch novel random nibbles: they have no source to copy from". False for short copies: among N random nibbles every string of length about log₁₆N has occurred before. Entropy is not saved, **pointers are**: a phrase of ℓ rows costs one instruction. Neon measures z/h₁ = 0.54–0.64 for supplied acyclic copy plans at minimum length 2. The repeats-are-wide lemma (long repeats) stands; the inference from it does not.

## 2. Why "aggregate" has no fixed observation language [proved]

Clause 2 says rows "determined from aggregate information" go uncharged. Try to fix what may be read free.

- If the free observation is any function of the column, take ι = L. Then U_𝒪 is every row (18.1) and the whole decoder is one free label rule.
- So the *computation* of the labels must be charged, not the observation. But charging a computation by "the single-row determinations inside it" is charging its expansion, which record 103 §2 and the review's caution rule out.
- So a charge exists only relative to a **declared instruction set**. That is what Λ′ is, and what record 107 §4's admission form asks of anything else. Clause 2 cannot be made a property of functions.

**The three neighbours, as instructions:**
- *fill_remaining:* one instruction, reads the histogram. Neon's ceiling is 6.4–6.8% of h₁ even as an oracle. Not a route.
- *copy(dst, src, len):* one instruction writes len rows from earlier output. As a **consumer** it is a real many-rows-per-instruction operation, and z < h₁ on these blocks. Its plan must be acquired from the column (clause 3). Every acquisition I know walks LF. It is *unplaced*: the consumer is cheap, the acquisition unpriced.
- *E1 class fact (S, k, w):* for all x in the class of a k-word, one link writes k bytes. Built for every class at once, this is the table of LF^k: at k = 2 it is the team's own T² kernel.

## 3. The dichotomy [argued; the K = 2 figure uses Neon's measured m/h₁]

Everything outside Λ′ that beats h₁ does it with a **build pass**: a streaming pass over all run records that composes facts in bulk (the frontier's rounds, T², a parse). The frozen text does not say whether such a pass is charged. The two readings give:

- **Strict (each record a pass reads is charged).** Every exact decoder reads essentially every width-1 run's record, since its byte is recorded nowhere else (fill_remaining is the exception, a few percent). Then c₁ ≥ h₁ − (rows a histogram forces) for **every** decoder: a no by definition, and worthless.
- **Lenient (a streaming pass is free; only row-addressed steps are charged).** Then the count is decided by how many passes are allowed:
  - unbounded passes: j doubling passes give links of 2^j rows. On Open's m retained rows, c₁ = m/2^j, below h₁/2 at j = 2 or 3. **A yes by count, with nothing new in it.**
  - K = 2 (one working pass): bulk composition has depth at most 2, since LF^3 needs LF^2 already built. Best known: T² on the retained rows, c₁ = m/2 ≥ 0.60·h₁ (m/h₁ ≥ 1.207, Neon's census). **It misses δ = 1/2 on every block.** That is not a proof that nothing does better in one pass.

**So the frozen target's answer is fixed by (the charge on a pass, K), not by the column.** Under the strict reading it is no by fiat. Under the lenient reading it is the pass budget that bites, and the only count-yes is doubling, whose k = 2 instance the team built and killed on time (T², E = 1.22, one implementation). §1a clause 3 ("fewer events that cost more time is not a yes") is the clause that actually decides.

**And the histogram no is weak at real budgets.** q₅₀ ≈ 7,000–14,000 cells is far below the state budget S (Open's scratch is of order m words). Theorem N′ with Theorem 3 closes only decoders observing fewer than about 10⁴ cells. Above that nothing is excluded.

## 4. What I now claim, and what I withdraw

- **Stands [proved]:** G1 (record 103); Theorem N′ inside Λ′ (107, Neon's 19.1); the flip structure (101 §1); Lemma 18.1's floor for any 𝒪.
- **Withdrawn:** that the main theorem is "heading to a scoped no" in any sense stronger than G1 plus a necessary condition of about 10⁴ observed cells. That any of my results bears on copy plans.
- **The decision the formulation needs** (statement owner's, taken here, stricter where the text is ambiguous): **DECIDED: a build pass is charged one event per row whose record it reads to determine something about that row, unless the pass is a pure fold to O(1) words per block.** This is the strict reading for every pass that composes or routes rows, and the lenient one for histograms and moments. Consequence, stated plainly: under it **the frozen yes is not reachable by any known or proposed mechanism**, and that is a property of the convention. The live question is therefore the labelled residual (unit price), plus one new residual:
- **Residual (c), labelled, not a relaxation:** *instruction count with supplied plans.* Copy plans have z/h₁ = 0.54–0.64 (Neon); the question is the charged cost of acquiring a plan from the column in one pass.

**Obstruction write-up.** *Obstruction:* a count on a calculus cannot rank computations outside the calculus without a convention on bulk passes. *Scope:* every candidate with a build pass. *Escape:* a currency defined on executions (time, or a fixed machine model), which the frozen clause 3 already is. *Transferable insight:* K is the parameter doing the work; doubling turns passes into fewer links at an exact exchange rate of 2 per pass, so "events" and "passes" are one resource in two units.
