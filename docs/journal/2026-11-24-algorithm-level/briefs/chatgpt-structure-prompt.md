# Prompt for ChatGPT Pro (Pro effort): structure and function in the decoder

Given to Louis to paste, 25 November 2026. The answer is to be recorded verbatim as `../external/chatgpt-structure.md`.

---

Repository lmmx/mibz2, branch `docs/journal-algorithm-level`, directory `docs/journal/2026-11-24-algorithm-level/`. Read in full: `essence.md`, `programme.md`, the journal `../2026-11-24-algorithm-level.md` §§44, 57–62, `math-targets/data/measurements-scan-profile.tsv`, and your own two earlier answers on Alon, recorded at `external/chatgpt-alon-motifs.md` (the second withdrew the first's motif catalogue).

**Don't open `leads/structure-and-function.md` until you have written your own answer.**

The framing (Louis): not universal motifs, but structures *optimal for our algorithmic purpose*, under the decoder's cost model. Don't force the Alon analogy; reach for whichever thinkers fit best.

Six questions. Label every claim proved, argued or conjectured, and cite prior work as prior work.

1. **Transformations between equivalent computations.** Contraction along composition and enrichment along composition (`essence.md`) emerged from this project. How do they relate to the algebra of programming: fusion, tupling ("banana split"), accumulation, and the homomorphism theorems (Gibbons 1996)? Map the decoder's passes onto these laws: Huffman scan, MTF/RLE2, inversion, emission, RLE1/CRC. Which fuse, which tuple, which are homomorphisms? What does each law say the transformed step must cost?
2. **The scanner, which is the bottleneck above about 8 workers.** Huffman decoding is 81% of its work at about 1.5 ns a symbol. Treat the decoder as a finite-state machine whose transition on a chunk is a function on states. Assess within-block parallel decoding by composing those functions (data-parallel FSMs, Mytkowicz et al. 2014; self-synchronising Huffman decoding, Klein and Wiseman 2003). Then:
   - Consider bzip2's obstruction: the table changes every 50 symbols, so the state includes a symbol count that does not self-synchronise.
   - Predict the cost per symbol of a k-way version, with what must be measured to know.
   - Compare it with cross-block parallel parsing.
3. **Optimality.** Alon's Part III argues from cost against benefit, and from Pareto fronts with archetypes (Shoval et al. 2012). Is the decoder's configuration (workers, window, scanner organisation) on the Pareto front of wall time, CPU and memory, from the measured data? What would its archetypes be?
4. **A generative test.** Enumerate the scanner–worker boundary as a finite design space: obligations (boundary discovery, code interpretation, symbol determination, selector/group bookkeeping, MTF, run records) × placements (serial in the scanner, in a worker, speculative by composition from all start states, eliminated by an intermediate object). Score each point by the referee (Q, D, P, S) and the measured demands. Do any points beat the shipped organisation, and which principle separates them (Ma et al. 2009 as the method)?
5. **The max-plus form.** Write the pipeline (serial scanner, k workers, admission issued − taken < W, in-order reader) as a max-plus recurrence (Baccelli et al. 1992). Does its cycle mean reproduce the profile's 20 walls better than wall(k) = c + max(b, a/k)?
6. **A retrospective reconstruction.** Louis's page-dewarp change of representation (lmmx/page-dewarp; `external/jax-idea-verdict.md`): which framework accounts for how it became possible, and which only describes it afterwards?

Then read `leads/structure-and-function.md` and say where you agree, where you disagree (with arithmetic or a counterexample), and what it missed.
