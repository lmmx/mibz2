# Record 124: what to measure for T1–T6, and what to reason about (channel #33)

*MONODROMY, 24 November 2026. Advice, argued throughout. Short by request.*

## 1. What to measure, in order of value

**1. The exchange rate between Q and seconds.** This is the number the mathematics lacks. Every candidate is scored in addressed accesses, and no record says what one is worth.
- *How:* per block, time the emission phase and count its links and its last-level cache misses. Regress phase time on bytes written and on links, across the 232 census blocks, not the six. The slope on links is the price of one dependent read; the slope on bytes is the streamed price.
- *Why it matters:* a candidate's predicted gain is then (price) × (links removed) − (price) × (accesses added in construction), to be compared with T1's ceiling before anything is built. It also tests the argued bridge from Q to machine reads (record 112): misses per link should be near one and stable.

**2. The worker split** (MTF/RLE2, construction, emission, RLE1/CRC). It sets the ceilings for T2, T3 and T6, and bounds T1 from the other side.
- *How:* cycle counters read at phase boundaries, per thread and per block. Not sampling. Then confirm each share with an oracle that supplies the stage's output, because a phase's time and the gain from removing it differ: a removed stage also stops warming or evicting the cache for the next.

**3. Misses and stall cycles per phase.** Last-level misses and cycles stalled on them, scoped to construction and to emission, at 1, 6 and 10 workers.
- *Why:* stalled cycles per miss falls when misses overlap. That is the machine's view of dependent depth D, and the only measurement that says whether a D cut (record 114, Neon S8) is worth anything.

**4. CPU against wall time, per thread.** Needed to tell work from waiting; it is the scanner profile's business and already in review.

**5. Per-block variation, with covariates.** Report every per-block time beside n, r, h₁ and m₁₆. Item 1's regression needs it, and so does the knee of the pipeline (record 123).

**6. Candidate counts on the census.** h₁, r, m₁₆, links and segments, with distinct composed words, strip pieces and contraction depth (records 119, 120). Six blocks are enough to refute a prediction and too few to rank routes.

**Lower value:**
- *Memory bandwidth:* only at high worker counts. The slope of worker capacity against k already shows it; uncore counters are worth reading only if that slope bends before 10.
- *Time-averaged against peak RSS:* neither bears on T1–T6. What matters is the bytes a team touches during the walk against its share of the shared cache, and that is computed from the structures' sizes, not measured.

**Free:** h₁, r and the run-width histogram are counters of the Huffman scan (record 122), so they can be logged for every block of every run at no cost.

## 2. What to reason about, not measure

1. **How ceilings combine.** The four arms show they do not add at 10 workers. Reason with the pipeline model: additive inside the workers, a minimum across scanner and workers (record 123). No further oracle is needed for combinations.
2. **A bound on T1 given construction.** With item 1's prices p for a dependent read and q for a streamed byte: a candidate with L′ links in place of L, and A added accesses in construction, gains p·(L − L′) − p·A, less any streamed work at q. So it pays only if L − L′ > A. Records 116 and 118 bound L′ from below in their classes, and record 120 bounds A. This inequality, not a benchmark, should screen candidates.
3. **Impossibility.** T1 and T2 admit proofs only of this kind: no route in a declared class goes below a stated count (G1, Theorems O and I, the conservation law of record 119). A proof never shows a speed gain.
4. **Exactness.** Always a proof, with a differential test as a check, never a benchmark. That covers every rider for T5 and the chunk summary for T3.
5. **T4 inside a block.** Whether the parse can be split is a statement about the size of a chunk's state transformer (record 122). It can be made a theorem in a stated model; measuring speculation would only sample it.
6. **Experimental design.** Grids, rounds and stopping rules follow from the spread of one round and the slope sought (record 123 §5).

**Which target admits what:**

| target | a proof can give | only a benchmark can give |
|---|---|---|
| T1 emission | exactness; a floor on links in a class | the price of a link; any gain |
| T2 construction | exactness; a bound on added accesses | its share of worker time |
| T3 MTF/RLE2 | exactness of the chunk summary | whether sharing it pays |
| T4 Huffman scan | seriality inside a block, in a model | the shift point |
| T5 enrichment | exactness; that a rider is not the inversion restated | its worth |
| T6 RLE1, CRC, output | exactness of composed CRC | its share |

## 3. One caution

Oracle ceilings are upper bounds on gains from *removing* a cost. A mathematical candidate replaces a cost with a smaller one and adds another elsewhere. Until item 1 is measured, no candidate's place under a ceiling can be stated.
