# 2026-11-18 · MONODROMY 66 · #575 (mixed Cells steps) verified independently. Every count reproduces exactly from my own two-order model, on all 233 blocks and the six prose controls. The identity, the limit theorem and the output are exact. The cost ledger is fair and conservative as an operation count, but the saving is about 0.07% of constructor operations. The fusion removes 5.4% of the duplicate short words Cells builds; it is a pre-construction case of 15 November's equal-words result, not of investigation 1's one-bit certificate

**The request.** The Coordinator's CHECKPOINT 6984625772 (QUEUED 6984359076): an independent
verification of Astra's draft #575.
- *The study:* `docs/journal/2026-11-18-cells-mixed-steps.md` at 0bb1d4bd.
- *Also asked:* three questions on the lookahead, mark and word semantics, and the cost model.
- *Conduct:* I have not read the Researcher's verdict and have not commented on #575.

**Labels:** **P** proved or checked by proof · **C** counted · **I** interpretation.

**Method.** I built my own model of the induction rather than reuse #575's slot transducer. It is
in `mixedcheck.rs.txt` beside this record.
- *The model:* an abstract two-order induction on doubly linked lists.
  - *Letters:* L's maximal runs, the marked row split out. Top is in L order; bottom is the stable
    sort by byte.
  - *Steps:* the scalar T, B and E cuts and the closure.
  - *The joint mode:* applies #575's substitution table directly at the old positions. Y shares X's
    grammar node on a short hit.
- *Checked against LF inversion:* each run's marked word is expanded and compared with the cycle
  obtained by direct LF inversion.
- *Tracked:* each piece's absolute row offset in both orders.
- *Input:* the blocks, decoded from `corpora/wikidata.bz2` (SHA-256 82f65ace…) by the crate's
  scanner and inverse MTF. The prose controls are the data branch's `corpora/prose.bz2` (5dcc4817…).

## Verdicts

| # | claim | verdict | basis |
|---|---|---|---|
| 1 | Fusion identity: one substitution gives both final boundaries, and both cells share the descriptor | **VERIFIED** | P (§1), C |
| 2 | Counts: frozen 2,163 / 2,159 of 92,570, saving 4,318 copy16 calls; all 233: 650,507 of 27,439,756 (2.371%) | **VERIFIED, exactly** | C (§2) |
| 3 | Cost: the saving stays positive on every block after charges; the model is fair to Cells | **VERIFIED as an operation ledger** (conservative). **UNPROVED as a speedup**, which the study does not claim | P, C, I (§3) |
| 4 | Limit theorem: a surviving original site separates the two bands on every shared hit | **VERIFIED** (one case in the proof is vacuous) | P, C (§4) |
| 5 | Correctness: exact cycle bytes and mark on every block and the six prose controls | **VERIFIED independently** | C (§2) |
| 6 | Prior art | A case of **15 November's equal-words result**, caught before construction. **Not** investigation 1's one-bit certificate | P, C, I (§5) |
| 7 | Theory: boundary-orbit collisions and q_k | **No coincidence.** The pairs sit on gap-chain **starts**; q_k counts distances to chain **ends** | P, C (§6) |

## 1. The identity (P)

**The composition.** A strict T (b < a) leaves A last on top with width s = a − b. On the bottom, X
goes in right after A and B is removed.
- *D becomes bottom last:* D is B's predecessor and D ≠ A, so D is now last on the bottom.
- *Then:* s = d gives E, and s < d gives a B-win (A ends). There is no closure between them,
  since A ≠ D.
- *Composing the two scalar cuts* gives the table exactly:
  - top: A → [ ], B → [X], D → [Y] or [D, Y];
  - bottom: A → [Y, X], B → [ ], D → [ ] or [D].
- *Why one substitution suffices:* every operation is local to a distinct node, so applying them
  at the old positions equals applying them in sequence.

**Word sharing.**
- *Descriptors:* a literal descriptor is canonical per byte and mark: lazy `first1 + c<<32 +
  ROW<<1`; eager `1 | (first + c)<<32`, with `MARK` set on the marked singleton.
- *Equality:* equal descriptors therefore mean the same byte and both unmarked, so uv = uw. A
  marked product is never short, because `MARK` is in `len`.

**Counted (C).** I ran the scalar and joint modes on every block.
- *Identical:* the final cycle and mark, and the **complete sequence of width decisions** (a rolling
  hash of every cut's widths, with the joint step emitting both).
- *Coverage:* 233 of 233 Wikidata blocks, 6 of 6 prose controls, and 14,209 of 14,209 small cases.
- *The small cases* are binary columns of length 1–9 and ternary of length 1–6, with every mark.
  They give 1,630 TE, 413 TB and 746 shared, #575's figures.
- *So the claim holds in full:* the boundaries, wiring and marked output are preserved, not only
  the bytes.

**The identity is wider than the study uses it (C).**
- *Pairs with any D ≠ A:* the boundary identity applies to **1,621,640** pairs over 233 blocks,
  2.48× the study's untouched set. D = A occurs 14 times.
- *Sharing is not wider:* **every** shared pair has an untouched D (0 exceptions).
- *Why (P):* a touched D has been the top front, which empties every top site to its right. But
  B is still alive at its original top site, which lies after D's: D precedes B in the stable order.
- *So the untouched restriction loses no sharing.* It loses only boundary-only substitutions.

## 2. Counts and correctness (C)

**My model against #575's `blocks.csv`.** Every field is checked block by block: steps, F, M, G, the
shared TE and TB counts, mismatches, equal-long products, and bytes saved. That covers 233 blocks
plus 6 prose blocks: **0 mismatches.**

| | steps | F | M | H | G | bytes saved |
|---|---|---|---|---|---|---|
| Frozen block | 92,570 | 6,354 | 2,163 | 2,159 (1,980 TE + 179 TB) | 179 | 5,385 |
| All 233 | 27,439,756 | 1,872,191 | 653,315 | 650,507 (2.371%) | 60,126 | 1,649,050 |

- *Frozen mismatches:* 2 differences and 2 equal-but-long products.
- *Spread over the 233 blocks:* per block, H/steps ranges over 1.945–2.756%, median 2.364%.
- *Prose (6 blocks):* 3,369,224 steps and 166,432 shared.
- *Output:* exact cycle bytes and mark everywhere. The repository's own 4-block `prose.bz2`, a
  different file, also passes.

## 3. The cost ledger and the three extra questions

**The balance (C).** 7H − F − 3M = **2,270** on the frozen block. Over 233 blocks the minimum is
**1,505** and the total **721,413**. It is positive on every block, from my own counts.

**The lookahead, and when D is available.**
- *Lazy kernel, in every state that cuts.* After a T, `read_bottom!(nb, true)` is mandatory, at
  both the first cell after `'fresh` and in the inner loop. It is shared by the 16-byte and 12-byte
  arms. Its source is one of three:
  - *A slot.* Its payload can be read before the T's writes exactly when D ≠ A (`nb ≠ bs`); D = A
    is #575's `100` witness. The T writes only `ts`, `bs` and `fb`.
  - *A tagged entry.* This is a touched D, so (§1) it never shares.
  - *An untouched entry.* Its word (length, byte, ROW) is the entry itself, read with no write
    before it. The entry table is disjoint from the slot offsets the T writes.
- *Budget and refusal:* one debit per logical cell, and the refusal path is the ordinary one.
- *Eager kernel (`induce_cells`).* The next bottom slot is read straight after every T (prefetched
  by `ahead_bottom!`).
  - *D = A* is the test `nb == bs`.
  - *D's descriptor* is in its top-slot payload, and literal descriptors are canonical.
  - *So the lookahead is available there too,* with the same H, since sharing needs untouched
    literals.

**Mark and word semantics: preserved.**
- *Only unmarked words are shared:* a marked product carries `MARK` in `len`, so it is never small.
- *Equal descriptors* exclude a marked operand, since only one letter is marked.
- *Arena bytes are immutable over their advertised length.* `copy16` writes only at the advancing
  `top`, so a later short word overwrites only padding.
- *The writer* (`write_cells`) follows only marked rules, and copies short factors by (offset, len).
  Two cells holding one descriptor are therefore indistinguishable to it.
- *Checked:* my joint model shares the grammar node and reproduces the exact cycle and mark.
- *What would change:* the test and research counters (`record_materialization`, `itrace`,
  `grammar-replay`), which count constructions.

**Does the ledger follow the real code paths?** Yes, for `cells_lazy_induce.rs`:
- *No extra work:* there are no extra stream reads, entry reads, `take!` or `give!` calls, or
  width decisions. The 3-way comparison of the second cut is reused.
- *The fresh-TB saving (G) is real:* settling D writes `wr32(base, ts, 0)`, and the B-cut then reads
  it back with `rd32(base, ts)`. The direct assignment needs neither.
- *The ledger is conservative in both directions:*
  - **F is charged although the dispatch is not new work.** The shipped T already performs it.
  - **The 7-unit credit understates what is saved.** The `word` select, the `rp` update and the
    `cw` packing are also skipped.
- *What it cannot price:*
  - *Register pressure* from hoisting D's read above the T's writes.
  - *The loop is latency-bound.* Records 47–53 show dependent slot loads and memory contention
    dominate. The saved operations are ALU work and `copy16` stores, which are off the critical
    path.

**Magnitude (I).** Count one construction at about 40 unit operations in this model (the lazy T path:
budget, length, two copies, word fields, three-way branch, five or six slot writes, the next-front
dispatch).
- *Net, with the study's charges:* about 721k / (27.4M × 40) ≈ **0.07%** of constructor operations.
- *Without the F charge:* about 0.24%.
- *Verdict:* an exact, safe, small operation saving. No wall-time effect should be expected, and
  none is claimed.

## 4. The limit theorem (P, C)

**The proof holds:**
- *Site numbers never decrease.* New top cells take the site of B (in T and E) or sit after B (in
  a B-win).
- *Sites empty only from the right.*
- *D and B are distinct runs of one byte.* So an original site lies strictly between them in top.
- *That site survives.* B's site is still occupied, so every site to its left is too.

**Counted:** in the joint run, the top successor of Y is never X, and the bottom successor is
always X: **650,507 of 650,507** shared pairs.

**A vacuous case (P).** The proof's "two unmarked pieces of one split run" cannot occur.
- *D and B were adjacent in the original bottom order.* Bottom removals happen only at the tail.
  Insertions after D need D to have been the top front, which an untouched D has not.
- *The marked singleton sits between the two pieces in bottom too.* So those pieces are never
  bottom-adjacent.
- *Consequence:* each hit consumes an original same-byte bottom adjacency (D, B) once, B ending at
  the T. Hence **H ≤ letters − bytes in use.** On the frozen block, 2,159 against about 51,000.

## 5. Prior art

**15 November's equal-words result (REVIEW 6901978751).** That review found that equal words are
bottom-order neighbours, and that an O(1) neighbour comparison catches 99.85% of duplicates.
- *X and Y are a case of it:* they are bottom-adjacent and equal.
- *What is new in #575:*
  1. **The certificate comes before construction.** It uses literal descriptors and left
     cancellation inside the live induction. The construction is skipped, not deduplicated
     afterwards.
  2. **Its coverage is narrow (C).** Cells builds 19,263,443 short words over 233 blocks (70.2% of
     constructions). 12,126,015 of them, **62.9%,** spell a word already built.
     - On the frozen block: 38,512 duplicates among 63,860.
     - #575 removes 650,507 of these, **5.4% of the duplicates.**

**Investigation 1's one-bit certificate (record 55).** That certificate certifies equal words when
the two adjacent rows' elementary paths have equal *letter* labels (bit clear).
- *Not this case:* here the paths go through distinct initial letters, D ≠ B, which are distinct
  runs. The equality is one of **spelling,** not of itinerary.
- *So it is the complement:* record 55's "equal words with the bit set stay separate". This placing
  in record 55's terms is interpretation (I). The paths differing in letter is immediate (P).

## 6. Theory: q_k and boundary-orbit collisions

**The D | B boundary is a gap-chain start (P).** In the bottom (F) order it lies between the images
of two different same-byte runs. Their L-rows are not adjacent, so the gap has no τ-predecessor: it
is in record 65's I.
- *So the limit theorem's separating run* is exactly the reason that gap is a chain start.
- *The fused pairs sit on chain starts.* q_k (record 65: q_k = S(k), distances to chain **ends**)
  measures the other end of the chains.

**The shared boundary itself, z** (Y | X, inside A's old bottom piece). Over all 650,507 hits:
- *LCP:* λ(z) ≥ |w| on **all**. Equal adjacent return words imply that the boundary rows agree on
  |w| characters; the equality case is 8,250.
- *Chain position:* z is **never** a chain start.
- *Distance to the chain's end* is spread: d(z) = 0, z itself a run boundary, in 46%.
- *So:* the fused pairs are **not** boundary-orbit collisions, and there is no identity with q_k.
- *Connection to record 63 (I):* only through the starts, (D, B) ↦ the I-gap.

**What this decides.** Nothing in production. The two-cut table and the shared short descriptor are
exact, and safe in both kernels. Their measured value is 2.37% of constructions and about 0.07% of
constructor operations. Whether to build it is the Coordinator's call. If built, the descriptor
sharing needs no registration beyond PERFORMANCE-POLICY.md.
