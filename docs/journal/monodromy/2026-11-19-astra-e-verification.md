# 2026-11-19 · MONODROMY 77 · Astra E's four delimiter-programme claims verified independently. The run bounds hold, with injective seams, and all three attainments (t = r; t = r + 1; a contiguous interval at r + 2) are found. The compound-marker conjugacy holds, overlaps included. The Ω(r) adversary holds; its swap receipt counts one parity class. The ⌊n/f⌋ selection bound and every frozen count reproduce exactly. No refutation

**The request.** The Coordinator's CHECKPOINT 6997116788, pinned at 2b19f2e2 on `research/astra-e-moonshot`
(the ledger, f2-bounds, f2-word, f2d and f2-heaviest).

**Labels:** **P** · **C** · **I**.

**Method.** I wrote my own checkers (`small.py.txt` and `select.py.txt` beside this record), using
the study's definitions and not its scripts.
- *Rows and maps:* rows of an arbitrary column L, with stable LF and Ψ = LF⁻¹, so multiple cycles are
  allowed. I(P) is the set of rows whose Ψ-forward spelling starts with P.
- *A section row's label:* the bytes read along LF until the first positive return to the section.
- *The frozen column:* `db876422…`, which equals the study's frozen SHA-256.

## Verdicts

| # | claim | verdict |
|---|---|---|
| 1 | Run bounds: t ≤ r for one word cylinder; t ≤ r + k − 1 for k equal-length markers; both attained; a contiguous non-cylinder section can exceed r | **VERIFIED** |
| 2 | Compound-marker conjugacy, and X·dw = dw·Y, overlaps included; mark inside a phrase | **VERIFIED** |
| 3 | Ω(r) lower bound for an ordered-symbol family, histogram supplied | **VERIFIED.** The receipt's swap count covers one parity class only; scope noted |
| 4 | Selection needs at most ⌊n/f⌋ expanded intervals; frozen 43 expansions → 3,232 rows; frequency one → 874,086 | **VERIFIED.** Every count is exact |

## 1. Run bounds (P, C)

**The proof holds.** Take two adjacent cylinder rows whose predecessor bytes are equal.
- *They move together:* LF translates them to adjacent rows. The prepended equal bytes followed by P
  fix the next |P| bytes, so both rows re-enter S together or neither does.
- *So a label can differ only at a seam:* a label change needs an L-run seam on the shared path.
- *The seams are distinct:* LF is a bijection, so the path back from a seam is unique. Two internal
  gaps can't charge the same seam, because the longer path would already have re-entered S.

**Counted, binary columns of length 1–9:**

| check | cases | outcome |
|---|---|---|
| Nonempty word cylinders (\|P\| = 1–4) | **17,392** | all intervals; **t ≤ r** on all |
| Charged gaps → seams | | the pair stays adjacent until the first byte mismatch, never hits S unequally, and the seams are **distinct**: 0 injection failures |
| Pairs of distinct equal-length words | **36,604** | **t ≤ r + 1** on all |
| Every nonempty section (all row subsets: the study's 348,502) | **348,502** | **t ≤ r + k − 1 + h** on all (k maximal intervals, h membership boundaries) |

**Attainments:**
- *t = r* with \|P\| = 2: **108** cases.
- *t = r + 1* with k = 2: **346** pairs.
- *A contiguous non-cylinder interval exceeding r:* **793** cases, by at most **2**. So t > r, and r + 2
  is the tight maximum here, as stated.

## 2. Compound-marker conjugacy (P, C)

**The proof holds.**
- *Ψ^|P| is order-preserving on S = I(P):* stable occurrence matching.
- *Its image is the after-occurrence section,* the rows whose preceding |P| bytes spell P.
- *It conjugates the first returns:* a first return from x to S over t steps lifts to a first return
  from Ψ^|P|(x) to the image over the same t steps.
- *Both words are one stretch of output:* X·P and P·Y are the same t + |P| consecutive bytes starting
  at the earlier occurrence. That stays valid for t < |P|, where the occurrences overlap.
- *Why P = dw is right:* w has only its terminal d, so "after token w" equals "after the byte word
  dw". Induction through d and then w gives the same section, by transitivity of first returns.

**Counted.** Binary columns of length ≤ 9 and ternary of length ≤ 6; every delimiter d; every token w
(d, xd or xyd, with x and y ≠ d) whose cylinder is nonempty: **7,912** cases.
- *Ψ^|P|:* strictly increasing on S, and its image equals the after-occurrence section.
- *Conjugacy:* exact at every row.
- *X·P = P·Y:* holds at every row, **7,501** of them with t < |P|, i.e. overlapping.
- *0 failures.*

**A mark inside a phrase (P).** Rewinding the primary along LF to its first section row gives an
offset k into the outgoing word sequence. The forward walk from that row, emitted from offset k, is
by definition the walk from the primary. So the n-byte output, a truncated last phrase included, is
exact. A marked cycle that misses S needs the fallback, as stated.
- *Not rerun:* the study's broader native property test (245,820 covered or refused cases).

## 3. The Ω(r) lower bound (P, C)

**The family.** L = x₀ … x_{m−1} dᵐ, with d below the data alphabet, even positions drawn from {a, b}
and odd positions from {c, e}.
- *Runs:* r = m + 1.
- *The token column:* LF(m + j) = j and LF(j) ≥ m, so the token column in section order is x_j d,
  with **m token runs.**
- *Counted:* all **1,020** columns (m = 2–9) give r = m + 1, tokens x_j d and m runs.

**The adversary holds** in the stated record-query model. With the histogram supplied, the consistent
columns number C(⌈m/2⌉, #a) · C(⌊m/2⌋, #c) = 2^{m − O(log m)}. A queried data run reveals one bit,
so Ω(m) = Ω(r) queries are needed.

**A correction to the receipt:** "3,868 count-preserving swaps" counts swaps inside the **even** class
only (I reproduce 3,868 with that restriction). Over both classes there are **6,484.** The argument
needs only one class, so nothing changes.

**Scope (I), as the ledger itself says:**
- *The bound equals the input size.* Any decoder already reads Θ(r) capped records, so it says
  "read a constant fraction of the input" and separates nothing from Cells or an r-index-style
  method.
- *What it does not do:* force the discovery of every unary node (q − b can be far larger than r).
  It bounds no marked-only decoder that doesn't expose the whole token BWT.

## 4. The selection bound (P, C)

**The proof holds.** Every expanded interval has width at least the winning frequency f:
- *before the optimum is found,* a live ancestor of it has width ≥ f, and pops go widest-first;
- *afterwards,* the search stops below the incumbent.

Nonterminal partial-return cylinders are disjoint across depths: a row's first future hit of S is
unique. So the expanded widths sum to at most n, and **at most ⌊n/f⌋ intervals are expanded.**
- *A sharper fact:* the expanded set is exactly the nonterminal cylinders of width ≥ f, whatever
  order equal widths come off the heap. The count is deterministic.

**The frozen block, independently** (my heap search over L-runs with stable rank; quote section
[C['"'], C['"'+1])):

| level | section rows | winning f | expanded | run pieces | ⌊n/f⌋ | expanded widths |
|---|---|---|---|---|---|---|
| 1 | **135,716** | **24,165** | **4** | **5,243** | 37 | 218,621 |
| 2 | 24,165 | **3,232** | **39** | **420** | 278 | 187,492 |
| 3 | 3,232 | **241** | **520** | **1,375** | 3,734 | 215,094 |
| 4 | 241 | **1** | **874,086** | 874,481 | 899,931 | **899,931 = n** |

- *Every count is exact.* 4 + 39 = **43** expansions cut 135,716 rows to 3,232. The frequency-one
  level needs **874,086.**
- *No child straddled a section,* so the inside-or-disjoint claim holds at every expansion.
- *Level 4 attains the disjointness bound:* the expanded widths sum to exactly n.

## 5. Interpretation (I, after verification)

**Item 1** is the token-level form of a familiar fact: inducing on a section can't create more runs
than the seams it inherits. Prefix-free parsing's parse BWT and the run-preserving splits of move
structures are close relatives.
- *k equal-length markers* add at most k − 1 seams where their cylinders are concatenated.
- *An arbitrary section* adds its membership boundaries.
- *Practical value:* it prices token alphabets and link widths before discovery, as the study uses it.

**Item 4** is best-first branch and bound with a disjoint-cylinder packing bound. It shows selection
is cheap while the winning frequency is large, and degenerates to a full traversal at frequency one.
The frozen levels show exactly that.

**None of the four** says anything about time. The ledger's own measurements still put the delimiter
constructions above LazyCells or opend in whole-worker cost.

**What this decides:** nothing in production.
