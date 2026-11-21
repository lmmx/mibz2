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
| 0.5.0 → 0.5.1 | 10t +7.3% (+6.2, +8.5); 12–20t +6.7 to +7.7% | 10t: #462, lazy workers at 10 threads instead of teams; 12–20t: #468, past the cores, lazy workers one a core | routing; memory | `bench-split-051` (split-051/run-20261116T145423Z): wall 10t #462 0.931 [0.908, 0.954], #468 0.996 (noise); 12/16/20t #468 0.924 / 0.933 / 0.942, #462 noise |
| 0.5.1 → 0.5.2 | 12t +4.7% (+2.4, +7.0); 16t +4.7% (+0.1, +9.5); 20t +5.0% (+1.2, +8.9) | #481, past the cores, one lazy worker a core but one, with the window sized to the workers | routing; memory | the only decoder merge (#503 and #508 are feature-gated research; #504 pins the toolchain). `bench-cores-less-one` (cores-less-one/run-20261116T203759Z): wall 0.956 / 0.960 / 0.959 at 12/16/20t |
| 0.5.2 → 0.5.3 | 12t +6.9% (+4.4, +9.5); 16t +6.7% (+2.8, +10.7); 20t +7.5% (+4.6, +10.4) | #524, on the measured i9, seven lazy workers from 10 threads asked, with the window sized to them | routing; memory | the only production decoder merge (#506 adds a bench-only 12-byte rules arm; production's `induce_lazy` is unchanged). `bench-workers-confirm` (workers-confirm/run-20261117T092105Z): 7/28 over 9/36, wall 0.921 / 0.930 / 0.923 at 12/16/20t, peak memory 0.78–0.81 |
| 0.5.3 → 0.5.4 | 8t +4.5% (+3.8, +5.3); 10t +12.3% (+9.4, +15.2) | #537, on the measured i9, the trimmed workers by mode, the machine recognised from its present and online CPUs (pinned one CPU a core: 6 workers at 8t, 7 at 10t, where 0.5.3 built 8 and 10) | routing; memory (10t) | the decoder merges are #537 and #545; #545 reads the machine once a process, a fixed cost a decoder that doesn't move these walls. The step's rows are pinned. Pinned sweeps: `bench-workers-sweep-low` (workers-sweep-low/run-20261117T102836Z) 6/24 over 8/32 at 8t 0.925; `bench-eager-trim` (eager-trim/run-20261117T114640Z) 7/28 over 10/40 at 10t 0.889, eager 5/20 at 6t 0.938, eager 3/12 at 4t 0.959. Unpinned, `bench-unpinned` (unpinned/run-20261117T130353Z): 10t 0.873, 8t 1.000 (the 8t gain is pinned only) |
| 0.5.4 → 0.5.5 | 8t +5.4% (+3.1, +7.8); 10t +13.5% (+11.9, +15.1); 12t +12.8% (+10.7, +15.0); 16t +13.2% (+7.9, +18.8); 20t +12.1% (+8.6, +15.7) | #559, the lazy kernel admits blocks to r/n 0.15 with a budget of 0.32 n, so fewer take the T-vector walk | memory | the only decoder merge (#558 is policy only). `bench-admit` (admit/run-20261118T100717Z), unpinned, Wikidata ×4: wall 0.838 / 0.878 at 8/10t; DRAM 10.70 → 8.05 B per output byte at 7 workers |
| 0.5.6 → 0.5.7 | 8t +10.4% (+9.0, +11.7); 10t +7.5% (+5.9, +9.1); 12t +8.0% (+4.7, +11.4); 16t +6.5% (+3.7, +9.4); 20t +6.9% (+5.7, +8.2) | #630, the open kernel as the workers' kernel on LazyCells' admitted blocks | kernel; memory | of the two decoder merges, #642 changes only the i9's 6-thread trim, a count this run doesn't measure. `bench-open-port-3` (open-port-3/run-20261119T171047Z) measured #630 alone against 0.5.6: wall 0.954 / 0.934 / 0.940 at 8/10/20t on Wikidata ×4, peak RSS 0.76–0.78 |
| 0.5.9 → 0.5.10 | 8t +6.5% (+5.9, +7.2); 10t +9.0% (+7.2, +10.8); 12t +6.6% (+3.5, +9.8); 16t +7.4% (+5.5, +9.4); 20t +5.2% (+1.4, +9.1) | #692, the Open kernel's memory diet: endpoint state in `edges`, thin-run queues in the dead output buffer (per-worker scratch 4,236 → 1,200 KiB natively) | memory | the only decoder merge. `bench-open-diet` (open-diet/run-20261121T121714Z), unpinned, Wikidata ×4: wall 0.952 / 0.917 / 0.931 at 6/8/10t, peak RSS 0.89 |
| 0.5.10 → 0.5.11 | 10t +4.2% (+3.3, +5.1); 12t +3.8% (+1.6, +6.1); 16t +3.6% (+0.7, +6.7); 20t +4.8% (+3.4, +6.2); 8t +2.1% | #703, the five densest full blocks (r/n 0.15–0.21) admitted to Open instead of Segmented's 5.6 MiB-per-worker fallback; peak RSS on this run 87.0 → 63.4 MiB at 8t, 93.9 → 66.5 at 10t | memory | the only decoder merge. Accepted as a memory gain (#704) on a native run that showed wall within noise on a 2 MiB-L2 Xeon; on the i9's shared 20 MiB L3 the memory cut also bought wall time |

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
  - `bench-split-051`'s older arms reproduce the i9's gains (v0.2.9 → v0.3.0 wall 0.872 at 1t, 0.927 at 4t; v0.3.2 → v0.3.3 0.950 at 4t, 1t within noise), but perf could not count on the workstation, so the cause is still untested. Its test needs `kernel.perf_event_paranoid` ≤ 1 on the i9.
