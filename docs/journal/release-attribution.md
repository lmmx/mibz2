# Release attribution: which merge moved which thread count

A ledger, kept under `docs/RELEASING.md` §5b. It has one row for each release-on-release step that clears the bar there, and nothing else. Most releases add no row.

**How rows are made.**
- **The steps:** the i9-10900K release-scaling lineage (`docs/site/assets/release-scaling.json`, the History page's Release scaling view). Each figure is the step against the release before, modelled, on wikidata.bz2. A positive figure is faster. The 95% interval comes from the two releases' pooled ratios to lbzip2.
- **The candidates:** the merges between the two release tags (`bench-history.json`'s `releases`).
- **The attribution:**
  - at 1, 2 and 4 threads, CI's per-merge paired deltas (the merge-by-merge view) can name the merge;
  - above 4 threads CI has no data, so a row is attributed only when one decoder merge is in the interval, or a run has split them.

**The bar.** A step gets a row when its interval excludes zero and it is at least 3% at some thread count.

**Mechanism classes:**
- **kernel:** fewer instructions on a block;
- **port:** an existing kernel reaching another path;
- **routing:** which plan runs at a thread count;
- **topology:** how threads are grouped and pipelined;
- **memory:** a smaller or colder working set;
- **unexplained.**

## The ledger

Backfilled on 16 November 2026, from 0.2.4 (the first release with an i9 run) to 0.5.1.

| step | where it moved (95% interval) | merge | class | evidence |
|---|---|---|---|---|
| 0.2.1 → 0.2.4 | 2t +10.4% (+8.8, +12.0); 4t +18.0% (+16.5, +19.5); 8t +3.1% | #171 staged walk; #180 window of 4 blocks a thread | kernel; memory | CI per merge: #171 4t +17.6%; #180 2t +14%, 4t +10% |
| 0.2.4 → 0.2.5 | 10t +11.3% (+10.2, +12.5); 12–20t +7.6 to +8.4% | #234, three teams of three at 10 threads | topology | the only decoder merge; CI can't see it (4 CPUs) |
| 0.2.5 → 0.2.6 | 10t +6.8% (+5.5, +8.1); 12–20t +4.4 to +7.1% | #250, the three teams pipelined | topology | the only decoder merge |
| 0.2.7 → 0.2.8 | 1t +12.7% (+11.6, +13.8); 2t +13.5%; 4t +8.8% | #324, run-record front and LF table on blocks of few symbols | kernel | CI per merge: #324 +17%, +18%, +11%. Not at 8t and above, where teams ran without it |
| 0.2.8 → 0.2.9 | 8t +14.8% (+13.9, +15.6); 10t +14.1%; 12–20t +11.0 to +11.2% | #359, team paths read the run-record front | port (#324 to teams) | nothing at 1–4t, which already had it |
| 0.2.9 → 0.3.0 | 1t +14.8% (+11.7, +18.1); 4t +8.5% (+7.2, +9.7) | #383 Cells | kernel; memory | the only decoder merge. CI saw 1t +0.0%, 4t +3.1%: see *Open* |
| 0.3.0 → 0.3.1 | 1t +4.8% (+1.4, +8.2); 4t +4.8% (+3.5, +6.1); 12t +3.0% | #391, Cells' 20-byte slot | memory | CI per merge: #391 +1.6%, +3.5%, +4.7% at 1/2/4t |
| 0.3.1 → 0.3.2 | 1t −3.3% (−5.6, −1.0) | candidate #408 (Cells: recycle the dead front slot) | unexplained | CI per merge: #408 1t −0.5% (noise). Below the bar to chase alone |
| 0.3.2 → 0.3.3 | 1t +4.2% (+2.1, +6.4); 2t +4.4%; 4t +6.3% (+5.0, +7.7) | #414, Cells' 8-byte bottom slots | memory | the only decoder merge. CI saw +0.1/+0.2/+1.9%: see *Open* |
| 0.3.3 → 0.3.4 | 8t +8.2% (+6.8, +9.7) | #425, workers below 10 threads | routing (8t from teams to Cells) | the only decoder merge |
| 0.3.4 → 0.4.0 | 8t +9.5% (+7.7, +11.3) | #433, Cells' bottom slots in the dead output buffer | memory | of the three decoder merges, #431 and #432 change no shipped path |
| 0.4.0 → 0.5.0 | 8t +4.2% (+2.9, +5.5) | #452, lazy letters from 8 threads | memory | #447 is opt-in arms only |
| 0.5.0 → 0.5.1 | 10t +7.3% (+6.2, +8.5); 12–20t +6.7 to +7.7% | #462 or #468 (not yet split) | routing; memory | `bench-split-051` (#489) splits them |

0.2.6 → 0.2.7 has no row: no thread count moved by 3% or more.

## What the ledger says, once

Since 0.2.4, 10 to 20 threads have moved only by:
- topology (#234, #250);
- a kernel port (#359);
- the high counts' move onto Cells in 0.5.1.

Cells' memory diet (#391, #414, #433, #452) moved 8 threads and below only, because teams ran above. Since 0.5.1 it runs at 10 to 20 threads too, so working-set cuts are now the direct lever there.

## Open

- **The i9 sees more than CI from Cells' memory changes** (#383, #414: 1t +15% and +4% on the i9, about 0 on CI).
  - Hypothesis, unconfirmed: cache size (the i9's 20 MB L3 against a 4-vCPU slice of an EPYC 7763).
  - Tested by `bench-split-051`'s older arms, with LLC misses per byte where the i9 lets perf count.
- **0.5.1's split** between #462 and #468: `bench-split-051`.
