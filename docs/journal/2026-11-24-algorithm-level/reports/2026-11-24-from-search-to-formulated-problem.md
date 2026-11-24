# Report: from a search for a breakthrough to a formulated, measurable problem

*24 November 2026, written at Louis's request: "we are now producing a proper formulation of the task and properly turning it into a toy problem, this is excellent. can you please write a report on what we found". It covers journal §§33–49 and the records they cite. Every figure here is taken from a record named beside it; nothing new is claimed.*

## 1. In one paragraph

The day began as a search for a new mathematical basis for inverting bzip2's block sort, and ended as a formulated problem with a referee, measured ceilings and a map of what has been ruled out. Along the way:
- The council fixed what "lowering the floor" means.
- Louis set the method: change what "moves together" (contraction), and make existing work yield more (enrichment). The cost rules act as referee, not generator.
- Twelve candidate senses of moving together were built and reviewed; none cut the read count below one per lone row once every charge was counted.
- A four-arm oracle showed where whole-decoder time can actually be won: up to 26% at 10 workers, masked until now by the serial Huffman scanner.
- The decoder is being decomposed into stages and shared resources, measured across every worker count from 1 to 20, systems-biology style.
- The mathematics now has a precise target (the inversion's emission and construction inside the workers), a negative theorem bounding one whole class of approaches, and a short list of open threads.

## 2. The formulation

| piece | what it says | record |
|---|---|---|
| The council | Three independent answers (MONODROMY, Neon, ChatGPT Pro), compared. The agreed statement: one uniform program, exact on every column, building its structures from the column within fixed budgets, at most half the old floor on every block. "Half the old floor" is meaningful only through a *bridge* theorem connecting it to machine cost. | `council/comparison.md`; §33 |
| The currency | Louis chose real machine cost: addressed reads Q, dependent depth D, passes P (K = 2), state S, acquisition included. The old bookkeeping target is kept as version 1. | §33; channel #18 |
| The essence | Two principles. **Contraction along composition:** change what moves together. **Enrichment along composition:** make existing work yield more, as automatic differentiation did for page-dewarp. The term is overloaded on purpose and every sense is pursued; prior work may be revisited but never presented as new. | `essence.md`; §§34–37 |
| Corrections | **G1 is not a machine lower bound:** it bounds charges in the rigid-interval calculus, and the bridge is unproved. **The 16% figure was never a ceiling:** it described one supplied-codes intervention. Both had been overstated in summaries. | §42 (from ChatGPT Pro) |
| The referee's rulings | Breakthrough means Q below one per lone row (target h₁/2) at K = 2. A large cut in D is a real but lesser find. Cursors are limited to s ≤ \|Σ\| + O(1), so R⁴ with its payload inlined (about 2k cursors) is inadmissible. | §§38, 49; channels #22, #36 |

## 3. The decoder as a system: where time can be won

**The four-arm oracle** (journal §44): whole-decoder wall time against the shipped decoder, byte-exact in all 20 rounds.

| workers | B: output-order codes supplied | C: Huffman scan supplied | D: both |
|---|---|---|---|
| 1 | 0.830 | 0.996 | 0.825 |
| 6 | 0.850 | 0.982 | 0.830 |
| 10 | 0.976 | 0.806 | 0.741 |

- At 1–6 workers, emission is the lever.
- At 10 workers the serial Huffman scan is the wall. With it removed, emission's gain reappears (D/C 0.920). The earlier "no gain at 10 workers" was masked.
- The scanner does the Huffman decode only. MTF/RLE2, the inversion and emission run in the workers (§44, corrected).

**The mathematical targets** (`math-targets/`): T1 inversion emission, T2 construction, T3 MTF/RLE2, T4 Huffman scan, T5 cross-stage, T6 RLE1/CRC. Only T1, T4 and T5 have oracle ceilings so far; a worker-split oracle is the next measurement. The induction, which the mathematics is about, runs in the workers.

**Decomposition** (`briefs/decomposition.md`; §§47–48). Two independent answers, MONODROMY's record 125 and Neon's `DECOMPOSITION.md`, refined the brief's eight stages and three resources and converged on these points:
- add a serial tail: block retirement, reordering and output;
- split memory into cache share, bandwidth and address translation;
- pin threads and separate SMT siblings. The decoder runs k + 3 threads, so oversubscription starts at 8–9 workers;
- use slowdowns of 20–25%, not 5–10%, because the noise is 4–9% at 10 workers;
- the summation theorem checks time, not resources;
- treat block heterogeneity as covariates;
- fit on some worker counts and validate on the rest.

They differ on the primary model: a roofline per stage (MONODROMY) against control coefficients (Neon). ChatGPT Pro's answer is pending.

**In flight:**
- The scanner profile, #838: work against waiting by component, every count from 1 to 20, peak and time-averaged RSS, CPU time, faults and context switches, each decode in a fresh process. Queued; the ready tag waits on Miri.
- Then a parallel scanner gated by worker count, as an adoption registration.

## 4. The mathematics: what was ruled out, what was proved, what is open

**Ruled out, with evidence.** Each of these was built, measured on the six real blocks and reviewed:
- Carbon's four objects: boundary chords, radix-fibre cones, quotient slabs and unitigs. Two repeat the kitchen-sink work of 20 November (`reviews/carbon-cycle1-critique.md`).
- Arches and copy plans built without the walk; the arches turned out to be SA-IS local-minimum cuts (`reviews/neon-cycle5-critique.md`).
- Landmark induction: Q rose 5–20% and D fell 50–75× (§38). Count-targeted landmarks bring D to 0.07–0.22% of n, a reproduced D record, but Q stays at about n (§41).
- Pointer doubling and independent-set contraction: zero addressed reads, at 117–600 passes. These are external-memory list ranking (§39).
- k-letter words through cursors, three times over (§35).
- Two-direction sweeps for closure: more fragmented than M-17 on ChatGPT's metric (§43).
- Lazy thin-run contraction: NEEDS M-27 closed by a pre-registered rule; it fragments the read schedule 4–9× (§49).
- Every composed route, all-in: 1.3–2.8 reads per lone row (§49).

**Proved or established:**
- **Record 119's conservation law, with Lemma W.** A sweep is an induction, and composition keeps the access schedule's form but not its compactness on high-entropy rows. This explains the 21–24× fragmentation ChatGPT Pro measured on block 0 (2,148 segments, rising to 44.8k–52.3k), and answers the closure problem negatively *within this class* (§§42, 48).
- **Record 118's Theorem I:** a lower bound on failing steps for every interleaving of monotone cursors (§48).
- **Record 122:** n, r, h₁ and the run-width histogram are free counters of the Huffman scan, and a parallel parse across blocks is exact (lbzip2 prior art).
- **Record 121:** an exact composable chunk summary for MTF, RLE2 and the run records.
- **IET acquisition** puts construction's cost back at 0.10–0.34·h₁ (§49).

**Open:**
1. **Acquisition.** On real blocks it is the binding cost. Record 120's search-free strip contraction projects about 0.6·h₁; that is unmeasured, and NEEDS M-25 runs first.
2. **A representation outside record 119's class,** one that composes without fragmenting. The law says a cleverer sweep cannot do this.
3. **Relation C** (the same template offset in repeated records): the pairing obstruction claimed against it proved invariance only, not an access bound (§41).
4. **Cross-stage riders.** Neon's T5 theorems rule out final LF values before the end of block or MTF, not deferred riders, which record 121 carries (§49).
5. **Engineering at the scanner:** worker-gated parallel scan, which the four-arm C arm prices at up to 19% at 10 workers.
6. **The vectorised-compaction lead** (Louis's copy_if post): an AVX2 left-pack for filter steps inside the workers, held until the decomposition shows which stage controls throughput. MONODROMY is asked which other operations share its shape (channel #37).

## 5. The toy problem, as it now stands

Fix the machine (i9-10900K, AVX2, 10 cores and 20 threads), the six admitted blocks plus synthetic hex, and the referee (Q, D, P, S at K = 2). Then:

- **Mathematics:** on random-looking content inside JSON, find a representation of the inversion whose acquisition and emission together cost under h₁/2 addressed reads, and which escapes record 119's class. A no-go result within a stated class counts as progress.
- **System:** measure each stage's share and controlling power at every worker count from 1 to 20, fit and validate a model, and use it to place the scanner's shift point and to price any candidate in seconds. Record 124 gives the exchange rate of a link, regressed over the 232 census blocks.
- **Payoff ceilings:** emission is worth up to 15–17% at 1–6 workers and about 8% behind the scanner at 10. The scanner is worth up to 19% at 10. Together they are worth up to 26% at 10.

## 6. How the work was done, and what to keep

- Every claimed result was reviewed independently before it was believed. Most headlines shrank. Neon's overclaims cost a review per round, until headlines had to be stated in the referee's terms (from round 4).
- Oracles measured potential before any building: §30, and the four-arm test at §44.
- Decision rules were registered before the data, and grids follow Louis's direction (1–20, all counts).
- Records live in subdirectories with lowercase names; data is TSV with its script (`math-targets/`); every reply to Louis is summarised in `notes/coordinator.md`.

## 7. Index

- **Formulation:** `essence.md`, `council/`, `math-targets/`, `briefs/`, `tracks/cross-stage.md`, `leads/`
- **Reviews:** `reviews/` (carbon cycle 1; neon cycle 5; neon-movement cycles 1–4; MONODROMY 118–125)
- **Outside answers:** `external/` (ChatGPT Pro rounds, the closure answer, the JAX verdict)
- **Narrative:** `docs/journal/2026-11-24-algorithm-level.md` §§33–49; plain accounts in `plain-explanations.md`
- **MONODROMY's records:** `docs/journal/monodromy/2026-11-24-*.md` on `claude/sharp-meitner-cx0v78`, records 108–125
- **Neon:** `research/neon-movement` (cycles 1–4, DECOMPOSITION.md)
- **Evidence bundles:** branch `evidence` (supplied-codes; `four-arm/run-20261124T204154Z`)
