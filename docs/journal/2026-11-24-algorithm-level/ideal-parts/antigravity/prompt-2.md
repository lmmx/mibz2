# Antigravity (Neon), second pass: mathematics against the measured ceiling table

Given to Louis to paste, 25 November 2026, once the full ideal-parts matrix was analysed (`../results/full/README.md`).

---

Work in the lmmx/mibz2 repository on your branch `research/neon-movement` (fetch `origin/master` to read from; don't push to it). Don't touch `src/`. Commit your answer as `research/neon-movement/IDEAL-PARTS-2.md` with Louis's identity (`git config user.name "Louis Maddox"`, `git config user.email louismmx@gmail.com`), with no AI attribution, and push it. Then post a short `FROM: neon (movement) / TO: coordinator / TYPE: REVIEW` comment on #145 linking it at its commit. Never put a PR or issue number straight after fix, close or resolve.

**The measurement is in.** In brief, from `results/full/README.md`:
- **Below the knee,** construction (C) has the largest room: 0.58 at 1 worker and 0.75 at 6. Emission (E) comes next.
- **At 10 workers,** the scan (S) has the room.
- **At 6–8 workers,** S and C co-limit (q_SC −0.110 at 6 and −0.071 at 8). Either part's room appears only when the other is ideal too.
- **The floor is mostly the oracles' handover cost,** so the decoder's own residual is not measured.
- **Your first pass** hit 14 of 28 values, and named S × C as the largest interaction, correctly.

 Read, on `origin/master`, under `docs/journal/2026-11-24-algorithm-level/ideal-parts/`:
- `results/full/README.md`: the full ceiling table (each part's room at 1, 6, 8 and 10 workers), the interactions, the masking map and the floor;
- `results/slice-a/README.md`;
- the scoring of your first-pass predictions (`results/*/scoring-neon-1.tsv`);
- your own `IDEAL-PARTS-1.md`.

**The task:**
1. **Your predictions against the measurement.** Where were you wrong, and what does the error teach about the decoder? Say which of your Part 1 models need revising, and revise them.
2. **Re-rank the targets by measured room,** with the masking map taken into account: a part's room behind another part counts only if that other part can plausibly be made cheaper too. For the top two or three, the room is the ceiling. Estimate what share of it each of your reformulations could capture, in the referee's currencies, labelled `[PROVED]`, `[ARGUED]` or `[CONJECTURED]`.
3. **The floor.** §4 of `results/full/README.md` shows the measured floor is mostly the oracles' handover copies, so the decoder's irreducible residual is unmeasured.
   - Estimate it from the data that exists, showing your arithmetic: the per-thread handovers and the walls.
   - Design the cheapest oracle change that would measure it directly, such as a zero-copy handover.
   - Say whether anything in that residual is mathematical rather than engineering.
4. **S and C together.** The matrix says a gain at 8 workers needs both the scan and construction made cheaper. Is there a single reformulation that changes both, for example by moving work across the scanner/worker boundary (cross-stage enrichment, target T5)? Or must they be attacked separately?
5. **One concrete proposal per top target:** the mathematical object to change, what it would replace in `src/`, the cost it would have, and the smallest measurement that would kill it fastest. Design only; no patches.
6. **What the matrix could not see,** and the next characterisation run that would see it, if any.

**Rules:**
- every number is traced to its file;
- conjectures are labelled as such;
- no engineering patches. This pass is mathematics and the case for it.
