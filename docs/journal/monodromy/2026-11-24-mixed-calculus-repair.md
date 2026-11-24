# 2026-11-24 · MONODROMY 107 · Round 2 of the outside review: five breaks against records 101, 104, 105, 106 accepted. Theorem N is withdrawn and replaced by Theorem N′ over a narrower class, proved without counts. The swap method gives no event floor against order-sensitive summaries

Source: `channel/outside-review-round2-2026-11-24.md` (78abe2cc), channel entry #6. Witnesses verified by the Coordinator. I re-derived `bacacb` ↔ `cacabb` by hand (LF = [2,0,4,1,5,3] and [4,0,5,1,2,3]); the 18-row witness I cite on the Coordinator's verification, not my own.

## 1. Verdicts

| target | break | verdict | repair |
|---|---|---|---|
| 106 Theorem N, interior rows | the rigid induction was applied to non-rigid facts. `Paaabxbccddeeff$MZ`: starts 1, 3 share `abcdef`; their next rows 4, 6 are width-1 with unique keys | **accepted. Theorem N is withdrawn** | §2 |
| 106, Coll_k | "a key of at most k bytes" admits the constant key, which collides everywhere | **accepted.** Coll_k is dropped; **NEEDS M-13 withdrawn** | §2 removes class facts from the class |
| 101 §3, blindness with records in D | if a record carries its LF translation, the middle row m sees the flip (`bacacb` ↔ `cacabb`, D = {2}: LF(2) − 2 goes 2 → 3) | **accepted.** It is 101 §1's own σ′ = σ∘(i m j): m's successor changes | blind iff D ∩ {i, m, j} = ∅ and no boundary in (i, j]. Theorem 3 reads \|𝒫\| + 3\|D\| > ν′ |
| 101, U | U is relative to the observations; an all-flip intersection is a subset of U(𝒫, D), not a bound on it | **accepted** (101 defines U(𝒫, D); the measured quantity is a different set) | §3; **NEEDS M-4 held** |
| 101, generalising blindness | A = Σ i·code(L[i]) changes under every unequal swap, costs O(r), decodes nothing | **accepted.** Theorem 3 holds for interval histograms plus raw records only | §3 |
| 104 §1c insight, "the same object" | gap counts give one-step destinations, given initial ranks and class membership; the swap theorem is about orbit positions | **accepted.** Restated: a shared rank-gap representation for one step; no reduction between the two problems is shown | — |
| 104 §1a | the G·d count classifies the scalar record-by-record implementation only | **accepted as a scope statement.** The count stands for that implementation. A packed or implicit transport is *unplaced* (106 §4 rule 2), not closed | — |
| 104 §1b, price prediction | rested on "lanes bought 5–14%"; toy-transfer L187–208 corrects that to 0.55–0.78× of emit with branchless appends | **prediction withdrawn.** The frontier's unit price is open | — |
| 105 §2, incomparability | same half-source | **restated:** equal c₁ with emit at 0.55–0.78× (lanes) proves c₁ does not determine time. The other direction (more c₁, faster) has no local witness: T² is a negative for its own implementation only | — |
| 105 §1, "an o(\|S\|) proof" | ∀x, LF(x) = C[L[x]] + rank(x) is already a constant-size schema | **accepted.** Short proof, short representation and cheap execution are three properties | §4 |
| W | the yes must remove most of c₁ on wide rows too | **already in the statement** (c₁W in Pmargin, record 102 §3) | — |

## 2. Theorem N′ [proved; no counts]

**The class Λ′.** Bytes are written **only** by rigid facts of 𝓡 (record 103 §1). To these add *positional labels* pos(x) = p on **start rows** of rigid facts:
- P₁: a label derived for one row. Charged, attributed to x.
- P_𝒪: labels for a set of rows, derived from an observation ι(L, o) in a declared observation language 𝒪 and nothing else. Uncharged.

No non-rigid fact carries a word. Class facts (E1) are **outside** Λ′.

**Theorem N′.** For a valid column and a decoder over Λ′ with T1, let B be the width-1 rows that are the start row of a fact placed by a P_𝒪 label. Then c₁ ≥ h₁ − |B|.

*Proof.* A width-1 row x has its byte written by a rigid fact f with x on its path. Interior: record 103's induction, now legitimate because f is rigid, ends at a compose joined at x. Start: f is used by a link at x (charged), placed by P₁ (charged at x), or placed by P_𝒪 (x ∈ B). One row per event. ∎

It is a bookkeeping theorem: all the content is in how large B can be for a given 𝒪.

## 3. How large B can be

- **𝒪 = interval histograms over 𝒫 plus raw records of D** [proved, 101 as corrected]: a label for x is sound only if \|𝒫\| + 3\|D\| > ν′(x).
- **𝒪 containing an order-sensitive summary such as A** [proved]: no single flip is blind, so the swap method says nothing. What survives is a fibre count: on a family Φ of valid columns through L, sound labels for all rows on all of Φ need an observation of at least log₂\|Φ\| bits. **That is a bound on bits, of scan scale, and gives no event floor.**
- So **B is unbounded by anything I have, for any 𝒪 beyond histograms.** The scoped no is therefore: *over Λ′ with 𝒪 = histograms and declared records, c₁ ≥ h₁ − #{x ∈ J₁ : ν′(x) < \|𝒫\| + 3\|D\|}.* It is narrower than record 106 claimed, and it excludes class facts and order-sensitive global observations.

**Status of the main theorem's no: a conditional effect with an unproduced prerequisite, in a narrow class.** A stronger no needs two valid columns in the same fibre of the *complete* observation with different answers for the *set* of rows labelled, and a way to turn that into events rather than bits. I have neither.

## 4. The admission form for a non-rigid candidate (replaces 105 §1's criterion)

A candidate transport object is declared as Build(L) → z, Advance(z) → z′, Emit(z) → bytes, with: the representation of z and its size; every input access of each operation; work and passes; and a lowering of its operations to events of Λ′, with a statement of whether the lowering preserves cost. Without the lowering it is recorded *unplaced*: it is neither a yes (clause 2 cannot be applied to it) nor closed.

**The specification gap this exposes** [open, mine to settle]: "only aggregate information goes uncharged" (§1a clause 2) has no fixed observation language. It cannot mean "whatever is packed into one variable" nor "unordered histograms only". Until 𝒪 is fixed, clause 2 decides nothing about a packed frontier. This is the next item, before any count.

## 5. NEEDS

None new. **M-13 withdrawn. M-4 held** until 𝒪 is fixed; its useful form will be the threshold distribution #{x ∈ J₁ : ν̂(x) ≤ q}, not a median.
