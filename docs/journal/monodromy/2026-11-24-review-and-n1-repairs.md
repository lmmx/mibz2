# 2026-11-24 · MONODROMY 102 · Neon's N-1 breaks accepted and repaired in the statement; the outside review's four points checked. Copy schemes attack rows c₁ does not count: a repeated hex token has at most log₁₆N + 2 width-1 rows per occurrence

## 1. Neon's N-1 (8ec0f0b): all four breaks stand

| break | verdict | repair (main-theorem.md, this commit) |
|---|---|---|
| 1. G1 fails on multi-cycle columns (`aab`, p = 0) | **accepted** | G1 takes `c.singleCycle`; true on 𝓕. Record 101 already defines valid = n-cycle |
| 2. main omits c₁(Pglue) and state | **accepted** | Pmargin now charges c₁(Pglue); Pstate added, bounded by S |
| 3. mirror events attributed by subject cross N/R | **accepted** | events are attributed to the run they are built from |
| 4. record 100's PR bound is probabilistic | **accepted** | it is an expectation under model I; read "aR ≥ 0.996 in expectation", not per block |

**Neon's glue measurement** (N-2, 656d5e7; cited, not rechecked): full-orbit switches are 0.513–0.543·h₁ on the six admitted blocks. So seg_R ≈ 0.26·h₁ ≈ 0.32·h₁R, against record 101's argued 0.29. **NEEDS M-5 is withdrawn** as answered.

**With glue charged** [argued]: Pmargin at δ = 1/2 needs aN·h₁N + aR·h₁R ≤ 0.5·h₁ − g, and g ≈ 0.26·h₁ unless segment starts are placed by a multi-row fact, which record 101 forbids for coarse aggregates. Then with h₁R ≈ 0.82·h₁ the theorem needs **aR ≤ 0.29**, not 0.6.

## 2. The outside review (channel entry #2)

**1. Split by novelty.** Checked, and the worry is bounded.

**Lemma (repeats are wide)** [proved under model I]. Let a hex token w₁…w_k occur at two text positions. For 1 ≤ t < k the rows of the suffixes starting at offset t carry the same byte w_t and share the prefix w_{t+1}…w_k. Any row sorted between them shares that prefix too. Under model I the expected number of other suffixes with that (k−t)-nibble prefix is at most N/16^{k−t}, N the number of hex positions. So the two rows lie in one run of width ≥ 2 except with probability ≤ N/16^{k−t}, and the expected number of width-1 rows per occurrence is at most Σ_t min(1, N/16^{k−t}) ≤ log₁₆N + 16/15 + 1 (≈ 6–7 for N ≤ 10⁶), against ≈ 0.88·k for a novel token. ∎

- *So:* repeated hex contributes little to h₁R. σ_hex's R is already close to novel hex plus token tails. The three-part N-0 will show it; prediction: repeated-hex share of h₁ ≤ (repeated occurrences)·7/h₁.
- *The real impurity* is non-random digit content (σ_hex precision for GUIDs is 0.31–0.73, record 96). A number that occurs once is "novel" but not model-I. Model-I statements apply to GUID and hash nibbles only.
- *P0 caution:* novelty is a property of the text, not decidable from run records in O(r). A novelty split is admissible as a proof device only with its own cost charged. σ_hex stays the declared split; novelty is a reporting breakdown.

**2. c₁ is not machine cost.** Accepted. **DECIDED, stricter:** §1a clause 6. A yes must state a dependent-load count per block, no worse than Open's.

**3a.** Accepted (same as N-1 break 2). **3b.** Accepted in part. Record 100 §3c already says a no is relative to a declared Λ. §6 now states the scope: the no covers coarse-aggregate global rules (record 101) and leaves open fine-input global rules. It is a partial no until that residue closes.

**4. Leads for stream N: an adversarial pre-check** [argued].
- *Positional copy schemes.* By the lemma, copied content is rows of width ≥ 2, which c₁ does not count. A copy scheme can remove at most the token tails, ≤ 7 width-1 rows per repeated occurrence. It cannot touch novel random nibbles: they have no source to copy from. **Payoff bound: (repeated-hex share of h₁), which N-0 will measure and the lemma predicts is small.**
- *r-index φ and toeholds.* φ needs a text position sampled at every run boundary. Every width-1 run is its own boundary, so the samples alone are ≥ h₁ single-row positional facts, each charged (clause 1). Acquiring them is the walk. This is the "labels first" branch with positions for labels: closed unless the samples come from a multi-row fact, which is the open cell again.
- *Verdict:* neither lead enters the open cell on novel content. Neon should screen them at "payoff" first.

## 3. Addendum: N-1 break 3's other two parts (the c₁ convention)

**DECIDED, the stricter reading (Neon's option ii).** c₁ counts every event with a one-row subject, in a run of any width. A chain link's subject is its start row, so reading A (a link "uses a wide fact directly" and is free) is rejected: it is the handoff's trap in another form, and it would let the plain walk score h₁.

**What changes** [proved from the definition]:
- c₁(LF walk) = n. c₁(Open) ≥ m (Neon measured m/h₁ = 1.21–3.32 on the census). **Record 98 L52's "c₁(Open) = h₁ + pieces" is wrong and superseded.**
- G1 is unaffected: it is a lower bound, and its injection uses only the width-1 runs.
- σ partitions all rows into R, N and W (rows of runs of width ≥ 2). P1 is three-way, and Pmargin carries c₁W. A yes therefore also needs a width-preserving emit on wide runs (c₁W ≪ m − h₁), which shipped Open does not have.

**The target now reads:** aN·h₁N + aR·h₁R + c₁W + c₁(Pglue) ≤ h₁/2 per block. Shipped Open stands at ≥ 1.21·h₁.

**Neon's novelty measurement** (85–89% of h₁ unique at window 8) agrees with §2's lemma: repeats are not where h₁ lives.
