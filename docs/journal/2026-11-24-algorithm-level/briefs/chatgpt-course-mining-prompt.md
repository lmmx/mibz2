# Prompt for ChatGPT Pro: mine the performance courses against the scanner profile

Given to Louis to paste, 25 November 2026. Its answer is to be recorded verbatim as `../external/chatgpt-course-mining.md` and compared with `../leads/course-opportunities.md`.

---

Repository lmmx/mibz2, branch `docs/journal-algorithm-level`, directory `docs/journal/2026-11-24-algorithm-level/`.

We now have a measured profile of the decoder across 1–20 workers on an i9-10900K (10 cores, 20 threads), one corpus. Read, in full:
- journal `../2026-11-24-algorithm-level.md` §57 (the profile, what it found), and §59 (your own design review, as we recorded it);
- `math-targets/data/measurements-scan-profile.tsv` and `math-targets/data/scan-profile-report.txt` (the per-count numbers and the report);
- `methodology/performance-courses.md` and `methodology/sources/` (Jain's course; Harchol-Balter's 15-857 and 15-849b, the book's table of contents and the 15-857 reading list).

**Do not open `leads/course-opportunities.md` until you have written your own answer.** We want an independent reading, then a comparison.

The question: with these measurements as inputs, which tools in those courses (and the papers on their reading lists) give this decoder a concrete opportunity, and which are ruled out by the measurements?

For each opportunity:
1. the tool, with the chapter or paper;
2. how it maps onto the decoder (which server, which demand, which quantity in the TSV);
3. a predicted size, computed from the measured numbers, labelled proved, argued or conjectured, with its assumptions;
4. the cheapest measurement that would confirm or kill it on the workstation;
5. whether it is a production default change (it then needs an adoption registration).

Then: the tools the data rule out, and why, with numbers.

Then, and only then, read `leads/course-opportunities.md` and say where you agree, where you disagree (with the arithmetic), and what it missed. Check in particular its machine-bound estimate (about 0.109 s against a best wall of 0.191 s, from an SMT factor inferred from busy times) and its claim that the scanner, not the in-order reader, limits above the knee.

Keep proved, argued and conjectured apart as in your design review.
