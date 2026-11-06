# What's going on

The Oracle keeps this page (`SPEC.ORACLE.md`). It answers one question: what is the team
doing, and why? It is plain on purpose. Every line either says what we are doing or what we
have found, and links to the record that shows it. Each item is linked to its parent by "in order to".

Seeded by the Coordinator on 2026-11-06, before the Oracle's first run. From here the Oracle
owns it.

## Right now, in five lines

1. **The aim:** make mibz2's production bzip2 decoder faster, and release it. The latest
   release is 0.2.4.
2. **The question:** on the i9 (10 cores, 20 threads), why does decoding stop getting faster
   past 8 threads?
3. **Found so far:** speed follows the number of decoding *tables* (teams) that are live at
   once, not the number of threads. The policy runs 4 teams of 2 from 8 threads up. In an
   exploratory grid, 3 teams did up to about 9% better than 4.
4. **Being tested:** whether 3 teams really beat 4, on fresh data under a rule frozen before
   the run (#220).
5. **Waiting on:** Louis, to run `just bench-three-tables` on the i9 and upload its one
   tarball. Nothing else is in flight.

## The tree

Each item has three fields. **Doing** says what we are doing. **In order to** links it to its
parent. **Status** is one of:
- *doing*;
- *done as intended*;
- *done, not as intended* (with the reason);
- *waiting on* (who, and for what);
- *stopped* (why).

**Known** lists only what the record shows, with links. Anything inferred is marked as such.

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
  - Throughput follows the number of live tables. The rate per table falls from about 570
    at 2 tables to about 465 at 3, about 320 at 4, and about 110 at 8.
  - Neither registered primary passed (1.017 and 1.003). More helpers per team, on the same
    4 tables, didn't help.
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
- **In order to:** 1. A cause tells us whether to cut a table's footprint, change the
  scheduling, or both.
- **Status:** doing, mostly parked until 1.1's result.
- **Known:**
  - The Optimiser reckons each live table's active walk set is the T vector, about 3.6 MB a
    block.
  - For 4 teams to match 3, each team's footprint would have to shrink by about 25%. No
    current candidate does that.
- **Inferred, not shown:** contention in the shared L3 cache is consistent with the data.
  The occupancy pass can't isolate it: no two of its plans share a team size.

#### 1.3 Parked ideas

- **Doing:** nothing yet.
- **In order to:** 1. These would use the idle cores in a different way.
- **Status:** stopped until 1.1 reports. The ideas are pipelining the stages outside the walk,
  and pinning threads to cores.

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

- None open. (1.2's link to 0 runs through 1.1. If 1.1 fails, the Oracle should say whether
  1.2 still serves 1.)

## Changes since the last version

- Seeded.
