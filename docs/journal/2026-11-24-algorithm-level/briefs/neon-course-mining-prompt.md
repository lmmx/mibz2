# Prompt for Antigravity (Neon): mine the performance courses against the scanner profile

Given to Louis to paste, 25 November 2026. The same question as `chatgpt-course-mining-prompt.md`, except that Neon commits its answer.

---

Work in the lmmx/mibz2 repository on your branch `research/neon-movement` (fetch `origin/docs/journal-algorithm-level` to read from; don't push to it). Don't touch `src/`. Commit your answer as `research/neon-movement/COURSE-MINING.md` with Louis's identity (`git config user.name "Louis Maddox"`, `git config user.email louismmx@gmail.com`), with no AI attribution in the commit, and push it to `research/neon-movement`. Then post a short `FROM: neon (movement) / TO: coordinator / TYPE: REVIEW` comment on #145 linking the file at its commit, as you did for DESIGN-REVIEW.md.

**Background.** We now have a measured profile of the decoder across 1–20 workers on the i9-10900K (10 cores, 20 threads), one corpus. On `docs/journal-algorithm-level`, under `docs/journal/2026-11-24-algorithm-level/`, read in full:
- the journal `../2026-11-24-algorithm-level.md` §57 (the profile and what it found) and §60 (the design-review decisions, which used your DESIGN-REVIEW.md);
- `math-targets/data/measurements-scan-profile.tsv`, `math-targets/data/scan-profile-report.txt` and `math-targets/data/scan-profile-runs.tsv` (per-count figures, the report, and every decode's row);
- `methodology/performance-courses.md` and everything in `methodology/sources/`: Jain's course; Harchol-Balter's 15-857 (with its reading list) and 15-849b; the book's table of contents.

You may also use the bundle's per-block records (`scan-profile/run-20261124T223748Z` on the `evidence` branch, `runs.jsonl`) and the code, as you did for the design review.

**Do not open `leads/course-opportunities.md` until you have written your own answer.** We want an independent reading, then a comparison.

**The question:** with these measurements as inputs, which tools in those courses (and the papers on their reading lists) give this decoder a concrete opportunity, and which are ruled out by the measurements?

For each opportunity:
1. the tool, with the chapter or paper;
2. how it maps onto the decoder: which server, which demand, which column of the TSV, and which lines of `src/` if relevant;
3. a predicted size, computed from the measured numbers, labelled `[PROVED]`, `[ARGUED]` or `[CONJECTURED]`, with its assumptions;
4. the cheapest workstation measurement that would confirm or kill it, and whether it fits into stage 2's design (`briefs/character-review-decisions.md` §2) or needs its own run;
5. whether it changes a production default (which then needs an adoption registration).

Then the tools the data rule out, and why, with numbers.

Then, and only then, read `leads/course-opportunities.md`. Say where you agree, where you disagree (with the arithmetic), and what it missed. Check in particular:
- its machine-bound estimate (about 0.109 s against a best wall of 0.191 s, from an SMT factor inferred from busy times);
- its claim that the scanner, not the in-order reader, limits above the knee;
- its reading that the 792 non-Open blocks are the 792 blocks under 10 kbit (0.1% of the bits);
- the Python binding's default of all logical CPUs (`src/python.rs`), and what an adaptive worker cap would need.
