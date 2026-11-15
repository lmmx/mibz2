# 2026-11-15 · MONODROMY 11 · The Rauzy path as a code for the BWT's run lengths: 1.19 bits a run against 2.05 for adaptive run-length models (Wikidata, pooled); an induction driven by it needs no lengths

Thread 2 of the thought experiment (#145, 6900062843): the type word of the right-end
induction as a representation of the runs.

**Labels:** **P** proved · **C** counted (exact for its input) · **Conj** conjecture.

## 1. The bijection (P)

Fix a column's run bytes b₁ … b_r, in L's order. Let τ ∈ {top, bottom, equal}^S be the sequence
of step types of the right-end induction (§2 of `2026-11-10-rauzy-veech.md`). Then (b, τ)
determines the run lengths λ₁ … λ_r.

*Proof.*
1. **The combinatorics run forward without lengths.** The two orders start as the runs in L's
   order on top and stably sorted by byte below, a function of b. Each step's letters are the
   last on top and the last below, and how it rearranges the orders is a function of its type
   alone.
2. **The lengths run backward.** The last step leaves one letter. For a column of one cycle
   that letter has length 1; for a power v^k, length k (§2 there).
3. **Undoing a step** is linear: a top win sets λ_A ← λ_A + λ_B; a bottom win sets
   λ_B ← λ_B + λ_A; an equal step sets λ_A ← λ_B, re-inserting A where the orders say it was.
4. Undoing the steps in reverse order recovers every letter's length at the start, which are
   the runs' lengths. ∎

**The comparison is fair.** Both representations carry the same run bytes. τ replaces
exactly the lengths.

**Which orders give a code.** Only an order whose step location is implicit gives a code on its
own, as the right end's is. An order like "free steps first, else the largest cell" would also
have to encode where each step is taken.

## 2. Which is cheaper (C)

`tau.rs.txt`, here, is the schedules simulator with the step types recorded. Every final word is
still checked, by hash, to be a rotation of the block. It compares adaptive code lengths: each
model pays for what it learns as it goes, with no model sent ahead.
- **τ:** a KT estimator per order-k context, k = 0 … 6.
- **The lengths:** a frequency model per context, with an escape (probability 1 / (t + 1)) to an
  Elias-gamma code. The contexts tried are none; the byte; the byte and the previous byte; the
  previous length (capped at 8). The best of the four is taken per block.

| bits a run | Wikidata, 233 blocks (min / median / max; pooled) | prose, 4 blocks |
|---|--:|--:|
| the lengths, the best of the four models | 0.537 / 2.201 / 2.662; pooled 2.047 | 1.760 / 1.856 / 1.963 |
| τ, order 6 | 0.547 / **1.229** / 1.376; pooled **1.185** | 1.664 / 1.694 / 1.782 |
| ratio, τ ÷ the lengths | 0.499 / **0.554** / 1.020; pooled **0.579** | 0.902 / 0.916 / 0.946 |

τ is the cheaper on 232 of 233 Wikidata blocks and on all 4 prose blocks. The exception is
block 145 (r = 179,972), where both cost about 0.54 bits a run.

**Why τ is cheap.** It is mostly predictable from its last few symbols:

| order | 0 | 1 | 2 | 4 | 6 |
|---|--:|--:|--:|--:|--:|
| bits a run, median | 2.63 | 1.49 | 1.32 | 1.24 | 1.23 |

- *What the context sees.* The equal steps are about 54% of the steps. They are the ends of
  the stretches (§5.5 there), and a single-row stretch is a run of equal steps that its
  context sees coming.
- *What the length models cannot see.* Their contexts see only neighbouring runs in L's order.
  τ is written in the order of the induction: suffix rank, the order the structure lies in.

**Static entropies mislead here.** A static count over the same contexts had put the lengths
at 1.19 bits a run. With tens of thousands of sparse contexts scored for free, that was the
artefact; the adaptive figures above are the honest ones.

**Limits.**
- The length models are simple context models, not a tuned coder.
- bzip2 codes runs inside MTF, RLE2 and Huffman, so its cost cannot be split out for a direct
  comparison.
- The claim is relative to the models stated, on one corpus and prose.

## 3. What it means

**A new representation (Conj on its use).** In a format that stored τ in place of the run
lengths, the decoder's induction would be driven by τ.
- It would compare no lengths. The slot's λ field (4 bytes) would go, and with it every
  length read and subtraction.
- The writer would still need the word lengths |w|, which the induction makes.
- *So τ is both a smaller code for the runs, about 0.58× here, and a smaller state for the
  decoder.*

**bzip2's format is fixed,** so this cannot change mibz2's decoding of bzip2. It belongs to:
- a future format, or a container for BWT blocks;
- an index (r-index run-length storage);
- the question of what a BWT's runs *are*. A BWT's runs are a path in the Rauzy diagram, and a
  path in the Rauzy diagram is cheaper to write than the runs.

**For r = 2 (K):** this is the continued-fraction expansion of a Christoffel word's slope. Its
digits are the lengths of the τ blocks of one kind.

**Open.**
- τ for orders other than the right end, with their location cost.
- A coder conditioned on the evolving permutation, not only on τ's own past.
- Whether the r-index community's run-length encodings (move structures, length capping)
  would gain from it.

## Reproduce

```
cp docs/journal/monodromy/2026-11-15-rauzy-path-code/tau.rs.txt examples/tau.rs
cargo run --release --example tau -- corpora/wikidata.bz2 1   # also corpora/prose.bz2; minutes
```

Outputs: `tau_wikidata.jsonl` and `tau_prose.jsonl`. Their fields:
- `len_bits`: the lengths' static entropies;
- `len_adaptive`: the four adaptive length models;
- `right_end.bits_k0..4`: τ's static entropies;
- `right_end.adaptive_k0..6`: τ's adaptive code lengths.
