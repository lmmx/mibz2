# 2026-11-18 · MONODROMY 60 · The recovered Pro investigation, assessed. The rank-neighbour equality is LF stability, and well-foundedness is the validity of the BWT's 2r macro scheme (Navarro, Ochoa and Prezza). Greedy is optimal by suffix-closure. The 11,145-copy oracle parse reproduces exactly. On the frozen block the BWT macro scheme has 62,138 phrases, half of them pointing forward, with dependency chains up to 26,406 bytes. An oracle experiment, specified to run

**The request.** The Coordinator's CHECKPOINT 6982004512 (QUEUED 6981909408). The document is
`docs/journal/2026-11-18-pro-investigation-recovered.md` at master bed89886. This is a bounded
assessment, not a reconstruction.

**Labels:** **P** proved · **C** counted (mine) · **R** reproduced exactly · **M** · **Conj**.

## 1. The rank-neighbour argument

**The equality: VERIFIED (P), standard.** Within a run L[x] = L[x + 1], and stable LF gives
LF(x + 1) = LF(x) + 1. Iterating while both rows stay inside one run, the two trajectories emit
equal bytes.
- *Already in the program:* this is investigation 1's gap path (record 55) and record 30's
  vertical saddle connections.
- *In text coordinates:* it is Φ, with Φ(SA[i]) = SA[i − 1]. T[SA[i] − 1] = T[SA[i − 1] − 1]
  unless i is a run head.

**Well-foundedness: VERIFIED as provable, and known, in this decomposition.**
- *The decomposition:* every text position whose row is a run head is a literal. Every other
  position q copies from the position whose row is ρ(q) − 1.
- *The argument:* the copy pointer lowers the row's rank by exactly 1. Ranks are finite, so every
  chain ends at a head.
- *Grouping into phrases* (maximal stretches of consecutive sources) gives the BWT-induced
  bidirectional macro scheme. **Navarro, Ochoa and Prezza** (arXiv 1803.09517, Theorem 7) prove it
  valid with at most 2r phrases.
- *Pro's group form* is the same: a group's leftmost trajectory reads from the row just outside
  the group, at a smaller rank. Its termination proof re-derives that theorem in the decoder's
  coordinates.
- *Not novel* against Φ, the r-index toehold (Gagie, Navarro and Prezza) and that theorem.

**On the frozen block (C), from my LF walk (`macro.py.txt`):**
- *Its size:* **51,184 literals and 10,954 copy phrases, so 62,138 phrases,** below 2r = 102,368.
  - *This is the network's E exactly:* the copy phrases are E − V, and investigation 2's 10,954
    copy spans.
- **It is bidirectional:**

  | copy phrases | count | bytes |
  |---|--:|--:|
  | with sources wholly earlier in the text | 5,343 | 448,131 |
  | overlapping, or with sources later | 5,611 | 400,616 |

- **Termination is not efficiency:** the copy-pointer chains reach **26,406 bytes**, with a mean of
  1,550. Evaluating the scheme in parallel rounds takes as many rounds as its deepest chain. That
  is investigation 2's 3,123-of-10,954 spans obstruction, measured.

## 2. The greedy copy-optimality lemma: VERIFIED (P); known

**The model.** Append copies of substrings lying wholly in D + T[:t], with unit cost and a fixed
seed holding every byte.

**The exchange argument is correct, because the dictionary is suffix-closed.**
- *Suffix-closure:* if T[t, t + ℓ) has a source [s, s + ℓ) inside the available prefix, then for
  0 ≤ k < ℓ the suffix T[t + k, t + ℓ) has the source [s + k, s + ℓ). It still lies inside the
  available prefix, which only grew.
- *So:* the longest phrase at t covers the first optimal phrase. Truncating the optimal phrase it
  partly covers leaves a legal suffix phrase, and the count can't rise.

**Known.**
- *The classical result:* greedy is optimal for suffix-closed dictionaries (Cohn and Khazan,
  "Parsing with prefix and suffix dictionaries", DCC 1996).
- *Folklore for LZ77-style parses:* greedy longest-match gives the fewest phrases (LZ77; Storer
  and Szymanski 1982, for the general textual-substitution framework).

The non-overlap and seed conditions keep suffix-closure intact.

## 3. What can be reproduced cheaply

**The oracle parse: REPRODUCED EXACTLY (R).** `oracle.py.txt` is a greedy longest nonoverlapping
previous match, via a suffix array of seed + text with RMQs, on the frozen pre-RLE text:

| seed (51,184 bytes or 256) | copies | singletons | longest |
|---|--:|--:|--:|
| **the run-head bytes in text order** | **11,145** | **33** | **4,819** |
| all 256 bytes, ascending | **20,068** | **917** | 4,819 |
| the run-head bytes in *rank* order | 18,132 | 230 | 4,819 |

- *Both of Pro's figures match exactly.* So Pro's seed was the run-head bytes **in text order**,
  which needs the heads' text positions, the toehold samples.
- *The seed/output split* (6,124 / 5,021): not recomputed.

**The 19% induction-step reduction: NOT cheaply reproducible.** "Deleting isolated ranks before
induction" survives only as a phrase. The definition of an isolated rank is lost: width-1 runs, or
rows whose neighbours are both run heads? So is how they are deleted, whether by first return to
the complement or by dropping them from the exchange.
- *Reproducing it* means fixing a definition and adding a pre-pass to the induction census (the
  stop-and-walk simulator).
- *Cost:* about an hour of work, but any number it gives would test *my* definition, not Pro's.
  I recommend not commissioning it unless the definition is fixed first.

## 4. The proposed oracle-input experiment

**As written, it is not well posed.** "The 16,917 gap endpoints and their translations" can't be
tied to any object definable from surviving material:
- the frozen block's gap structure has 51,184 finite gap paths (investigation 1), 10,954 interior
  ones (E − V), and 15,278 switch pairs (investigation 2);
- none is 16,917.

**A runnable replacement**, with every input precomputed offline and uncharged, on the frozen block.
The output is the 899,931 pre-RLE bytes, checked against the reference (SHA-256 f87e5caa…). Single
thread, medians of paired rounds against the labelled inverse, on the same host as its 2.39 ms.

| arm | what is free | what is charged | what it bounds |
|---|---|---|---|
| **C** (control) | — | the labelled inverse, or Cells, from capped run records | — |
| **A**, append replay | the 11,145-copy greedy parse (seed bytes in text order, plus (src, len) copies) | allocating the output and replaying the copies (memcpy, no overlap) | **the floor for any copy-based decoder:** what the decode costs once copy witnesses and coordinates are free |
| **M**, the BWT macro scheme | 51,184 literals at text positions (the toehold samples), plus 10,954 phrases (dest, src, len) | evaluation, by forward sweeps, then rank-ordered resolution of the backward residue | what the scheme the BWT induces costs to evaluate, with discovery free |

**Estimates (M):**
- **A:** 900 KB in 11,145 copies of mean 81 bytes is about **0.1–0.3 ms**. That is 8–20× under C.
  The budget left for deriving the witnesses from the run-length BWT is therefore about **2.1–2.3 ms
  a block**: everything the oracle got free.
- **M:** dominated by the backward half (5,611 phrases, 400,616 bytes) and the 26,406-byte-deep
  chains.
  - *A Jacobi evaluation* takes **26,406** rounds (C).
  - *Forward sweeps in text order* resolve more per pass, but still take **12,406** full sweeps (C).
  - **So naive evaluation is orders of magnitude above C.** An evaluator that beats C must order
    the bytes by rank, and knowing ranks is the inverse itself.
  - *My prediction:* M ≫ C for any evaluator not given the ranks. This arm mostly confirms that
    the BWT's own scheme is the wrong object for a copy decoder.
- *Cost to run:* about a day of Pathfinder time for both arms, with inputs dumped from the
  reference by my scripts.

**What it isolates, and what not.**
- *A against C:* the value of knowing copy witnesses in append order.
- *M against C:* whether the RLBWT's own scheme is cheap to evaluate.
- *Neither measures discovery,* which is the open problem. Discovery is bounded above only by C − A.
- *If M loses to C,* copy-based decoding needs a different scheme than the one the BWT induces
  (Conj).

## 5. The evaluation counts (C)

`passes.py.txt` evaluates the frozen macro scheme from its literals and phrase coordinates alone,
and checks the result byte-equal to the reference.

| method | passes |
|---|--:|
| parallel (Jacobi) | **26,406** (the deepest copy chain) |
| sequential forward sweeps in text order | **12,406** |

Each pass touches the 848,747 copied positions.
