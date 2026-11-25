# The ideal-parts matrix

> **THE TASK: measure the decoder with each of its parts made ideal, one at a time and in combination, at the worker counts that matter, and so find how much room each part leaves.**
>
> - **It is characterisation.** It changes no production code and decides no production default. It adopts nothing.
> - **It is not an engineering A/B.** Fixes, tunings and "does this patch help?" runs belong elsewhere. If a run here suggests one, write it down in `results/` and leave it.
> - **Its output is a table:** for each part, at each count, the decoder's wall with that part ideal, and the interactions between parts. That table is what the mathematical search (Antigravity, MONODROMY) is aimed by.
>
> *Louis, 25 November 2026: "make it very clear that the task is an ideal parts matrix … let's not become sidetracked".*

## Why

**The night of 24–25 November** (journal §§74–86) measured *who controls* the wall. Stages 2a and 2b slowed each activity and read the response: workers below about 8, the scanner's Huffman decode from 8 up.

That is an elasticity: the slope for *slowing* a part. It does not say how much would be won by making a part *cheap*. Stage 2b showed the two differ: at 8 workers, a faster scanner hands the bottleneck to the workers almost at once (§78).

A mathematical idea pays only if the part it improves has room. So before looking for new mathematics, we measure the room.

## Where it comes from

- **The oracle:** the four-arm test (journal §44) made the scanner ideal (arm C), emission ideal (arm B), and both (arm D). That was a 2 × 2 factorial, and it found masking: emission's gain was invisible until the scanner was ideal.
- **The matrix:** Jain's experimental design (2^k and 2^k·r factorial designs, `../methodology/performance-courses.md`). Each part is a factor with two levels, *real* and *ideal*.
  - A factorial over the parts gives each part's main effect (its room) and the interactions between parts (the masking).
  - Jain's sign table and allocation of variation give the analysis.
- **The programme:** steps 2 and 5 of `../programme.md` (payoff ceilings by oracle; experimental design). They feed step 6 (the model) and step 7 (where new mathematics fits).
- **The hybrid-systems frame** (`../reading/hybrid-dynamical-systems/relevance/levers.md`): an ideal part is lever 1, the flow map, pushed to its limit, with that activity's rate infinite. The matrix's results are measured inputs to that model: achievable bounds per part, the measured counterpart of §70's lower bounds.

## Layout

| path | what it is |
|---|---|
| `parts.md` | the factors: each part of the decoder, how it is made ideal, and whether its oracle exists |
| `design.md` | the matrix: factors, levels, counts, controls, rounds, measures, and the analysis |
| `registration.md` | the decision rule for labelling each cell and effect, frozen before any data (RULES C1a) |
| `status.tsv` | one row per part: oracle built or not, checked or not, measured or not |
| `antigravity/` | the prompts given to Antigravity: the mathematics (`prompt-1.md`) and the oracle build (`prompt-build-oracles.md`), and links to its answers |
| `results/` | each run's bundle reference, analysis program, tables and reading, once they exist |

## Status

| step | what | owner | state |
|---|---|---|---|
| 0 | this subproject, the design and the rule | Coordinator | written 25 November |
| 1 | slice A: the oracles that exist (the scanner's output, emission's codes) in one binary, run at 6, 8 and 10 workers | Pathfinder | assigned |
| 2 | the oracles that don't exist yet (construction, MTF/RLE2, RLE1/CRC/output, the reader) | **Antigravity (Neon)**, `antigravity/prompt-build-oracles.md`; the Pathfinder integrates them | prompt ready |
| 3 | the full matrix run | Pathfinder (recipe), Louis's workstation | after step 2 |
| 4 | the analysis: main effects, interactions and the ceiling table | Coordinator, MONODROMY | after each run |
| 5 | Antigravity, first pass: mathematics against the shape we have, gaps marked | Louis runs it | prompt ready (`antigravity/prompt-1.md`) |
| 6 | Antigravity, second pass: mathematics against the full ceiling table | Louis runs it | after step 4 |
