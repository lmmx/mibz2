# 2026-11-10: Design E, the line table: FALSIFIED as built, by its registered row

**Result:** FALSIFIED under `2026-11-10-design-e-registration.md` §6. With the line table in
the T-vector's place, a one-thread inversion on the Wikidata blocks costs **3.67×**
production's, with a 95% interval of 3.59–3.74. The registered threshold was a lower bound
above 2.0. E as built is closed, for one thread and, by the knee model, for the ten-worker
plans it was proposed for. No i9 timing follows.

- **Source:** CI's bench run on #323's head `84bf177`
  ([run 42745744064](https://github.com/lmmx/mibz2/actions/runs/42745744064)), the decisive
  observation under §8.
- **Runner:** AMD EPYC 9V74, 4 CPUs, reported clock 3170.5 MHz, bench 3.5.0.
- **Not shown:** the other runner models (Xeon Platinum 8573C, EPYC 7763).

Labels: **measured** (CI, the registered rows), **post hoc** (computed after the run from the
same CI data, decides nothing), **counted**.

## The registered rows (measured)

The research table ("Rows not in the total"): head's `phases`, Wikidata, one pinned CPU, 5
runs. Each row is over the stage it stands against in the same run, as a geometric mean and
95% interval.

| row | c/B (median) | against | row / stage | registered prediction |
|---|--:|---|--:|---|
| **line table E: build + walk, 32 lanes** | 19.48 | build + walk | **3.665× (3.591–3.740)** | 2.0–3.2, centre about 2.5 |
| line table E: build | 2.59 | build (1.42) | 1.827× (1.775–1.881) | 0.4–0.9 |
| line table E: walk, 32 lanes, prefetch | 16.87 | walk (3.87) | 4.342× (4.256–4.430) | 2.2–3.6 |
| line table E: walk, 12 lanes | 20.76 | walk | 5.396× (5.278–5.516) | (mechanism) |

- **The decisive row is above the predicted range,** not only above the threshold. The model
  assumed time would scale less than instructions did: the counted instruction ratio was 3.41×
  on Wikidata. In fact time scaled a little more.
- **The build's premise is falsified on its own** (§6: hi ≥ 1.0).
  - E's sequential build is 1.83× the T-vector's scatter, though it runs 0.82× the
    instructions (counted).
  - The likely mechanism is not measured here: the loop over a line's distinct bytes has a
    data-dependent trip count, a branch per line, and a serial chain through `rest`. The
    scatter's stores, meanwhile, overlap freely.
- **The walk is instruction-bound,** as the counts said. The step is 22–26 instructions
  against the T-vector's 4, and the bookkeeping is about 22.

**The decoder's own figures (null check):** the decoder is unchanged, and 20 of 21 wall cells
hold no change. `text` at one thread reads +2.3% (+0.4%, +2.3% CPU), a mark at about the
chance rate of one cell in twenty. Both arms decode the same generated file, and nothing this
PR changes is on the decode path. Peak RSS is unchanged.

## Pathfinder's common gate for small tables (post hoc, descriptive)

`2026-11-10-share-nothing.md` (on #313, after this registration) gives small tables (C, D, E)
one gate: a one-thread walk within 1.44× of the plain walk's cost per decoded byte. Above that,
the cooperative plan on the plain table still wins at ten threads. It's reported here **post
hoc**, from the same CI run's per-run figures (`stages.jsonl`, paired by run, 95% t-interval
on logs over 5 runs). It doesn't change the registered decision rule or its verdict.

The ratios below are E's 32-lane walk over each walk on the plain table:

| plain table's walk | its c/B | ratio | within 1.44×? |
|---|--:|--:|---|
| room walk (production's) | 3.87 | 4.342× (4.256–4.430) | no |
| flagged walk, same skeleton (32 lanes) | 5.68 | 2.956× (2.897–3.016) | no |
| single-chain `walk` ("walk, plain") | 35.46 | 0.477× (0.467–0.488) | yes, trivially: one chain against 32 lanes |

- **The 12-lane rows against each other:** E's 12-lane walk over "walk, bitmap (before #4)"
  is 1.843× (1.771–1.919). Both have the same skeleton, so this ratio isolates the table and
  step.
- **Counted:** 48.2 instructions a row against the room walk's 10.4 (4.6×), and 61.6 against
  the bitmap walk's 34.2 (1.8×).
- **Reading:**
  - If the gate's "plain walk" means the walk production runs on the plain table, E fails it
    by about 3×.
  - It still fails at about 2× against the same skeleton on the plain table.
  - Only the single-chain walk is slower, and that compares lanes with no lanes.

## What this closes, and what it doesn't

- **Closed: E as built.**
  - It holds a step of 22–26 instructions, with the table's distinct-byte compare, slot
    popcount and rank popcount.
  - It uses the flagged walk's skeleton with a bitmap of starts.
  - Its build is a per-line loop over distinct bytes.
  - Under the knee model, a one-thread inversion at 3.7× can't be repaid by halving the live
    table at ten workers: production's own contention is 3–4×.
- **Reopen only on this condition:** a step and skeleton counted, before any timing, at no more
  than 1.44× the room walk's instructions a row (about 15). §4 of the registration models
  E in the room walk's skeleton at about 32, so no known variant of E meets it.
- **Not tested:** text and other classes; more than one thread; other runner models. These are
  untimed by design (registration §9). The verdict needs none of them, since the bound came
  from the cheapest class for E's table (Wikidata, 98% one-access steps).
- **What stays useful:** the census (2.04–2.16 bytes a row on text-like data, 96–98% one-access
  steps), the byte-identity harness for backward walks, and bench 3.5.0's research table.

## Negative findings

1. **Pathfinder's step estimate is low.** "About 16 instructions" a step, compared with the
   room walk's 10.5, set E's step alone against the room walk's whole loop. As compiled for
   AVX2/BMI2, the step alone is 22–26, and about 48 with the bookkeeping.
2. **A sequential build isn't cheaper per se.** E's build, at 0.82× the instructions, took 1.83×
   the time of the T-vector's scatter on the Wikidata blocks.
3. **The predicted range was too low.** It assumed the walk would run at more instructions a
   cycle than the latency-bound room walk. The measured 4.34× against 4.6× counted shows it
   barely does.
