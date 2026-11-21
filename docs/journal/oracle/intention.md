# What's going on

The Oracle keeps this page (`SPEC.ORACLE.md`). It answers one question: what is the team
doing, and why? It is plain on purpose. Every line either says what we are doing or what we
have found, and links to the record that shows it. Each item is linked to its parent by "in order to".

Seeded by the Coordinator on 2026-11-06, before the Oracle's first run. From here the Oracle
owns it.

**Settled:** channel covered through #145 comment 6684224966. Evidence inspected at
2026-11-06 11:58 UTC:
- master `bbfbed55`;
- #220 head `838d765c`, CI green, waiting on the i9 run;
- #224 head `44248603`;
- #222 head, this page.

Reconciled against that evidence. This is not a live feed.

## In one line (L0)

Waiting for the i9 run that tests whether 3 decoding tables beat 4 (#220), with the
downstream 0.2.4 update open alongside it (item 2). Both serve faster decoding, delivered
(item 0).

## Right now, in five lines (L1)

1. **The aim:** make mibz2's production bzip2 decoder faster, and release it. The latest
   release is 0.2.4.
2. **The question:** on the i9 (10 cores, 20 threads), why does decoding stop getting faster
   past 8 threads?
3. **Found so far:** in the team-size grid, speed tracked the number of decoding *tables*
   (teams) live at once more closely than the number of threads. This is an association: the
   mechanism is unresolved. The policy runs 4 teams of 2 from 8 threads up. Exploratory cells
   with 3 teams did up to about 9% better than 4.
4. **Being tested:** whether 3 teams really beat 4, on fresh data under a rule frozen before
   the run (#220).
5. **Waiting on:** Louis, to run `just bench-three-tables` on the i9 and upload its one
   tarball. New performance work is paused until that result, by the Coordinator's choice
   (6683240055). The wikidata-pq bump (item 2) is open alongside.

## The tree (L2 fronts, L3 experiments; the record below, by link)

Each item has three fields. **Doing** says what we are doing. **In order to** links it to its
parent. **Status** is one of:
- *doing*;
- *done as intended*;
- *done, not as intended* (with the reason: the run or the plan);
- *waiting on* (who, and for what);
- *paused* (who chose to, and the decision's link);
- *stopped* (why).

A run's status is separate from whether its prediction held and whether it qualified. **Known** lists only what the record shows, with links. Anything inferred is marked as such.

### 0. Faster production inverse BWT, released

- **Doing:** making mibz2 decode bzip2 faster on real machines, and shipping each real gain as
  a release.
- **In order to:** this is the top. (Its own why, decompressing the Wikidata dumps faster in
  lmmx/wikidata-pq, is the owner's.)
- **Status:** doing. Last release 0.2.4 (#207, the MTF change).

### 1. Scaling past 8 threads on the i9

- **Doing:** finding what stops throughput rising past 8 threads, then raising it.
- **In order to:** 0. More threads are where the decoder's remaining headroom is.
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

#### 1.1 Three tables instead of four (#220)

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

#### 1.2 Why fewer tables win (the mechanism)

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

#### 1.3 Parked ideas

- **Doing:** nothing yet.
- **In order to:** 1. These would use the idle cores in a different way.
- **Status:** paused until 1.1 reports, by the Coordinator's choice (6683240055). Neither
  depends on 1.1:
  - **Placement (pinning threads to cores):** independently motivated, by #219's measured
    placement effect (0.778).
  - **Pipelining the stages outside the walk:** a less established hypothesis. It needs a
    price in recoverable time first.

### 2. Keeping the decoder's users current

- **Doing:** bumping mibz2 in lmmx/wikidata-pq after each release.
- **In order to:** 0 (released means used). wikidata-pq is where the speed is worth
  something.
- **Status:** the 0.2.4 bump is open in wikidata-pq, under that repository's bench rules.

## Accepted, not aimed at

- 3-table plans use less memory (about 18%). That would be welcome, but it isn't what is being
  tested.

## Where the "why" chain is weak

Each item's "in order to" should lead up to 0. One that stops at a procedural reason, such as
"to satisfy the gate", needs a look.

- **1.2 and 1.3 are paused, not blocked.** They serve 1 directly, and the pause is a
  scheduling choice. The test: name what would change if 1.1 failed its benefit, failed its
  resources, or turned out invalid. If nothing would, the pause is scheduling, not a
  scientific prerequisite.

## Changes since the last version

- Reshaped into levels (an L0 line and a settled-at stamp), per the spec's précis rule.
- Corrected from the Researcher's review (6684224966):
  - 1.2 now says the 25% threshold was retracted; only the pause stands;
  - L0's scope now includes the parallel front;
  - the stamp is now a channel watermark plus an evidence manifest.
- Seeded (01ea79a3).
- Corrected from the Researcher's review (6683945162):
  - "tables, not threads" restated as an association;
  - 4 × 3's real gain that fell short of the bar recorded;
  - the 25% premise marked as disputed;
  - 1.2 and 1.3 marked as paused, not dependent;
  - the parallel front named in "Right now".
