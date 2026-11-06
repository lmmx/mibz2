# 1. Scaling past 8 threads on the i9

- **Doing:** finding what stops throughput rising past 8 threads, then raising it.
- **In order to:** 0 (the production decoder, faster and released). More threads are where the decoder's remaining headroom is.
- **Status:** doing.
- **Known:**
  - The release-scaling run on the i9 (#218) flattens at 8 threads.
  - The team-size screen (#219, `docs/journal/2026-11-06-team-size.md`) measured 16 cells
    (`mibz2`, `mibz2-team2`/`3`/`4`) at 8, 10, 12 and 16 threads, over 20 paired rounds.
  - Across the grid, throughput tracks the number of live tables. The rate per table falls
    from about 570 at 2 tables to about 465 at 3, about 320 at 4, and about 110 at 8. This is
    an association within that grid, not a shown cause.
  - Neither registered primary qualified. 4 × 3 at 12 threads improved by 1.017
    (1.008–1.026), a real gain that fell short of the 1.03 bar. 4 × 4 at 16 gave 1.003. Extra
    helpers on the same 4 tables gave at most a small gain.
  - The two best cells had 3 tables. They are *exploratory*, chosen after seeing the data:
    - 3 × 4 at 12 threads: 1.097;
    - 3 × 3 at 10 threads: 1.075;
    - both used about 18% less peak memory.

## 1.1 Three tables instead of four (#220)

- **Doing:** testing 3 × 4 at 12 threads and 3 × 3 at 10 as registered primaries, against
  the policy's 4 × 2 at 8, on fresh data.
- **In order to:** 1. If either passes and confirms, the policy's cap on this machine moves
  from 4 teams to 3.
- **Status:** waiting on Louis, for the i9 run (`just bench-three-tables`) and its tarball.
- **Rule:** frozen before the run (`docs/journal/2026-11-06-three-tables.md`). The Guardian
  froze it in D030 and the Researcher reviewed it. A pass needs:
  - a mean of at least 1.03, with the interval's lower bound above 1.00;
  - every predicate holding;
  - memory and CPU within their limits;
  - the same cell passing again in one predeclared second run.
- **Prediction (recorded):**
  - 3 × 4 at 12 threads: 1.04–1.09;
  - 3 × 3 at 10 threads: 1.03–1.07.
- **What a result would mean:**
  - A pass is a policy *candidate*. The policy change is its own PR, with its own CI and
    parity evidence.
  - A fail qualifies nothing, and it doesn't show that the screen was noise.

## 1.2 Why fewer tables win (the mechanism)

- **Doing:** explaining the per-table slowdown, so the fix is chosen by its cause.
- **In order to:** 1, directly. It keeps its purpose whatever 1.1 finds.
- **Status:** paused until 1.1 reports, by the Coordinator's choice (6683240055). It doesn't
  depend on 1.1.
- **Known:**
  - The Optimiser reckons each live table's active walk set is the T vector, about 3.6 MB a
    block (b6d99258).
  - Cutting 25% from each team's footprint would make 4 teams' total effective footprint
    equal 3 teams' (Optimiser).
- **Retracted:** the Coordinator had made a 25% cut in the T vector touched per team the
  condition for reopening footprint work. That threshold is withdrawn (6683965024), after the
  Researcher's counterexample (6683945162). In the Optimiser's own model (20 MiB of L3,
  5.4 MB per team), a 2.9% cut lets 4 teams fit, and equal hit rates don't need equal
  footprints.
  - What stays is the dispatch pause until #220.
  - Footprint work reopens on its own measured price, with no fixed threshold.
  - No gain is predicted on the i9, and C15's measured failure stands.
- **Inferred, not shown:** contention in the shared L3 cache is consistent with the data.
  The occupancy pass can't isolate it: no two of its plans share a team size. The grid's
  exploratory same-size comparisons still mix concurrency and window with table count.

## 1.3 Parked ideas

- **Doing:** nothing yet.
- **In order to:** 1. These would use the idle cores in a different way.
- **Status:** paused until 1.1 reports, by the Coordinator's choice (6683240055). Neither
  depends on 1.1:
  - **Placement (pinning threads to cores):** independently motivated, by #219's measured
    placement effect (0.778).
  - **Pipelining the stages outside the walk:** a less established hypothesis. It needs a
    price in recoverable time first.

