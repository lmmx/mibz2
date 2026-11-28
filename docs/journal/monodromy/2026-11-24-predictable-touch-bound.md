# 2026-11-24 · MONODROMY 116 · The narrowed question. Counting cannot prove a bound: a pass carries more bits than random digits hold. A structural bound does hold in one class: with one build sweep, in-place composition, and rows processed block by block on their first k digits, at least 1/(k+1) of the steps break the chain, so Q ≥ n/(2(k+1)) on random digits

Answers channel entry #24. Labels as before.

## 1. No bound comes from counting [P]

The external-memory permuting bound counts arrangements: N items, log₂N! bits to fix, at most log₂(fan-out) bits gained per item per pass. Here the arrangement to be produced is the text order of the rows, and for a text of N independent digits over σ symbols it carries at most N·log₂σ bits. One sweep with s cursors routes each item one of s ways: N·log₂s bits. With s_max ≥ σ **a single sweep already carries enough bits.** So no argument that only counts configurations can show that two passes are too few. Any proof must use the *structure* of what a sweep can read, not its capacity. (This is the outside review's point that the BWT permutation has only O(n) bits of entropy, turned into a statement about passes.)

## 2. A structural bound in a declared class

**The class 𝒮_k.** One build sweep and one emit sweep (K = 2). Facts are built only by in-place composition E[x] ← prim(x) ∘ E[LF(x)] (record 112 §3). Rows are processed **block by block**: a block is the set of rows whose suffix begins with a given k-digit word; the blocks are taken in the order of an injective key κ on k-digit words, and inside a block in row order, either direction. There are σ^k blocks, so the cursor budget allows k with σ^k ≤ s_max.

**Model.** Digits independent and uniform on σ symbols; rows in lexicographic order of their suffixes. A step goes from the suffix d₀d₁d₂… to c·d₀d₁d₂… with c uniform and independent of the rest.

**Theorem O** [P under the model]. For every key κ, the probability that a step fails to move to an earlier-processed block is at least N_σ(k+1)/σ^{k+1} ≥ **1/(k+1)**, where N_σ(m) is the number of necklaces of length m.

*Proof.* A step is determined, up to the tie-break, by the (k+1)-word w = c·d₀…d_{k−1}: it goes from the block of w's last k digits to the block of its first k digits. All σ^{k+1} words are equally likely. Rotation w ↦ w₁…w_k w₀ sends a word to one whose first k digits are the previous word's last k, so the steps of a rotation class form a closed walk among blocks. Around a closed walk κ cannot strictly decrease at every step. So each rotation class contains a step that does not descend. There are N_σ(k+1) classes. ∎

**Corollary** [P under the model, with record 112 Theorem B2]. In 𝒮_k every non-descending step into a width-1 row forces a link at that row or the next. So **Q ≥ n_d/(2(k+1))**, n_d the number of random-digit rows, up to the fraction of such rows that are not width-1 (12% at σ = 16, record 96).

| alphabet in the region | largest k with σ^k ≤ 256 | bound |
|---|---|---|
| bytes, σ = 256 | 1 | Q ≥ n/4 |
| hex, σ = 16 | 2 | Q ≥ n/6 |

So inside 𝒮_k the answer to "can one predictable touch per novel digit fit in two passes" is **no, not below a sixth of them addressed**.

## 3. What the theorem does not cover

1. **Interleaved cursors.** 𝒮_k finishes one block before the next. Π also allows s cursors advancing in a data-dependent interleaving, where the order inside a block can depend on deeper digits. The rotation argument does not apply as written, because a step is then not determined by a (k+1)-word. I conjecture the same bound [C].
2. **Longer hops.** With σ^j ≤ s_max a cursor can also serve E[LF^j(x)], so a chain may skip over a later-processed row. B2 assumed the single hop. A bound of the form 1/(k + j) is plausible and unproved [C].
3. **Anything that is not in-place composition.** The pairing of record 113 relation C, if it could be had, is outside this class.
4. **Real content.** The model is independent digits. Templated text has descents built in.

## 4. The same lemma read forwards

Theorem O is tight in form: a key with exactly one non-descent per rotation class would give descents k/(k+1), against 1/2 for row order. On hex with k = 2 that is two descents in three. **So block order on a well-chosen 2-digit key should lengthen in-place chains beyond record 110's e.** This is the same family (sense B) taken one step further, not a new sense, and its ceiling is §2's bound. I have not constructed the key; finding it is a minimum feedback arc set on the 256-node de Bruijn graph.

## 5. Where the three corners now stand

| corner | best on record | proved floor in its class |
|---|---|---|
| few addressed reads, many passes | the frontier, doubling | none needed: it works, at P ≫ 2 |
| the same reads, shorter D | Open plus landmark sections: 1.1–6k at P = 4–5 (Neon, cited) | none |
| few reads in two passes | in-place sweep, about n/e links | **n/(2(k+1)) in 𝒮_k (this record); ½·min(a⁺, a⁻) for row order (record 112)** |

**Obstruction write-up.** *Obstruction:* a processing order keyed on k leading digits cannot descend around a rotation of a (k+1)-word. *Scope:* 𝒮_k, independent digits. *Escape:* an order that reads unboundedly deep (distance to a landmark), which costs rounds; or information that is not carried by composition. *Transferable insight:* the quality of a row order for inversion is a feedback-arc-set problem on the de Bruijn graph of its key; its optimum is governed by necklaces, 1/(k+1).
