# Antigravity (Neon), first pass: mathematics against the decoder's measured shape, before the ideal-parts matrix

Given to Louis to paste, 25 November 2026. A second pass follows once the ideal-parts matrix's ceiling table exists (`../README.md`, step 6).

---

Work in the lmmx/mibz2 repository on your branch `research/neon-movement` (fetch `origin/master` to read from; don't push to it). Don't touch `src/`. Commit your answer as `research/neon-movement/IDEAL-PARTS-1.md` with Louis's identity (`git config user.name "Louis Maddox"`, `git config user.email louismmx@gmail.com`), with no AI attribution in the commit, and push it to `research/neon-movement`. Then post a short `FROM: neon (movement) / TO: coordinator / TYPE: REVIEW` comment on #145, linking the file at its commit.

**The task.** We are about to measure the decoder with each of its parts made *ideal* (its output supplied, already computed), one at a time and in combination, at 1, 6, 8 and 10 workers. That is the ideal-parts matrix, `docs/journal/2026-11-24-algorithm-level/ideal-parts/`: read its `README.md`, `parts.md` and `design.md` first. The matrix will tell us how much room each part leaves.

**Before it does,** we want your mathematics aimed at the parts, and your prediction of each part's room, written down. The matrix then tests your predictions as well as sizing the opportunities.

**Read in full,** on `origin/master`, under `docs/journal/2026-11-24-algorithm-level/`:
1. `programme.md`, `essence.md`: the problem, and the referee's currencies (Q, D, P, S: queries, dependent reads, passes, state);
2. `math-targets/targets.tsv`: T1–T6, with the ceilings measured so far and those still missing;
3. the journal `../2026-11-24-algorithm-level.md`:
   - §44, the four-arm test: scanner and emission oracles; masking;
   - §57, the scanner's profile: 81% of its work is the Huffman decode, at about 1.5 ns a symbol;
   - §66, stage 1;
   - §§74–78, stages 2a and 2b: who controls the wall at each count; the wall as a sharp maximum; core sharing at +48%;
   - §§81–86: the clock after idle; what was adopted and what wasn't;
   - **§82 in particular:** the scanner's table already decodes 1.85 symbols a lookup, so "multi-symbol Huffman" is spent;
4. `reading/hybrid-dynamical-systems/relevance/levers.md`: the five levers, the language for a change;
5. `leads/structure-and-function.md`: designing for this decoder's purpose, not universal motifs.

**Don't read** MONODROMY's records 129–132 (branch `claude/sharp-meitner-cx0v78`, `docs/journal/monodromy/`) **until you have written parts 1 and 2 below.** We want an independent reading, then a comparison.

**Part 1: per part, the mathematics.** For each factor in `parts.md` (S the Huffman scan, E emission, C construction, M MTF/RLE2, R RLE1/CRC/output, D delivery), give:
1. **What the part computes,** stated as a mathematical object: its input, its output, and the dependency structure between them (which outputs need which inputs, and in what order).
2. **Its cost in the referee's currencies** as implemented now (`src/` is yours to read), and the lower bound you can argue for any method computing the same output. Label each `[PROVED]`, `[ARGUED]` or `[CONJECTURED]`, with its assumptions.
3. **Candidate reformulations:** different mathematics that computes the same output (or an output the next part can use equally well) at a different cost. For each:
   - which lever it pulls (`levers.md`);
   - its predicted cost, with the same labels;
   - what it would need from the parts next to it;
   - prior art, if any.
4. **Whether it is spent.** If the part is already near its bound, say so, with the arithmetic. §82 is an example: the second symbol a lookup was already taken.

**Part 2: your predictions for the matrix.** For each part at 1, 6, 8 and 10 workers, predict its room, written as the wall with that part alone ideal over all real, a point and a range. For each pair you think interacts, give the interaction. Show the arithmetic from the measured numbers (§§44, 57, 66, 74–78). These are registered as your predictions before the run, and scored against it.

**Part 3: where to look.** Rank the parts by (predicted room) × (your confidence that a reformulation could capture a meaningful share of it). Name the one or two you would work on first, and the smallest experiment, ideally a cell of the matrix, that would kill each idea fastest.

**Then, and only then,** read MONODROMY's records 129–132. Say where you agree, where you disagree (with the arithmetic), and what it missed.

**Rules:**
- every number is traced to its file;
- conjectures are labelled as such;
- no engineering patches. This pass is mathematics and prediction only.
