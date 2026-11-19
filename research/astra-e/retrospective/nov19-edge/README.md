# Retrospective after the independent edge i9 sweep

The official retrospective now sees **148 forecasts and 44 independent candidate/worker measurements** across 12 CI runs and two i9 runs. The November 19 edge sweep contributed four target measurements. The official target calibration still covers only three worker counts for the combined writer. At W7, W8 and W10 its v1 proxy mean absolute log error is 0.01491. There are zero comparable target rank pairs. The fixed original snapshot at d7703b0 had 122 forecasts and 31 independent measurements. [Official output](official.txt) · [First snapshot](../README.md)

## Newly included independent results

All ratios compare complete elapsed wall time with 0.5.6's LazyCells on the named host at the same W. The receipt directory includes compact/preload and the precounted frontend. The existing dose and edge i9 JSON files were independently regenerated and verified from their archives. Each measurement is discovered once. F5 combined word actions had already been recorded in the first snapshot. [Source and artifact verification](verified-inputs.json) · [New CI receipts](ci-receipts.json) · [Run-by-run coverage](coverage.csv)

| Entrant and run | W1 | W2 | W4 | Official local forecast at the same W |
|---|---:|---:|---:|---|
| Compact eager + eight response preloads, CI 44440381983 | .84678 | .86313 | .89036 | W1 and W4 |
| Precounted worker frontend, CI 44445335323 | .95606 | .96058 | .90292 | W1 and W4 |
| Fixed worker dose, CI 44448175373 | .72125 | .81710 | .83511 | None of these three |
| F5 combined word actions, CI 44436152031 | .91614 | .92362 | .86856 | None of these three |

A correctness-only registered score for dose or F5 contributes no local wall or proxy at those W. The separate dose experiments use their own worker assignments, instrumentation and comparisons. The compact row names the integrated preload entry and its actual five-arm CI. The measured standalone compact constructor has a different entry. Independent scalar receipts arriving after the frozen data cutoff belong to a later snapshot.

| Target W | Edge / Lazy i9 [95%] | Edge / combined i9 | Conditional edge / Lazy forecast | Conditional edge / combined forecast |
|---:|---:|---:|---:|---:|
| 7 | .86703 [.85996, .87416] | .97891 | .87911 | .96590 |
| 8 | .88363 [.87236, .89504] | .97335 | .87019 | .95441 |
| 9 | .89698 [.87876, .91559] | .97627 | — | — |
| 10 | .87591 [.86133, .89073] | .95846 | .85890 | .93822 |

Edge has official local walls at W1/W4 and a source-matched independent target receipt at W7–10. These worker sets have no common cell. Its prospective route substitution at a6baa5e prices six local runs on one changed block, substitutes three of the underlying 960 nominal rows and carries 957 rows from the predecessor. The three conditional Lazy ratios have mean absolute log error 0.01625. The separate comparison with the measured predecessor has error 0.01813. This diagnostic correctly anticipated an incremental improvement at each of its three W, and overstated that improvement at each one. It has its own evidence and model; the retrospective keeps its values out of the fully registered target calibration. [Verified point comparisons](conditional-edge.csv) · [Archive and frozen model lineage](../../results/edge-route-pricing/heldout.md)

## Join and accuracy audit

The reader emitted 185 rows. Ninety are local forecasts without a matching measurement at that worker count. Nineteen independent run/W cells have no local forecast. The remaining 76 rows map to 25 distinct independent cells, with 48 joins by entry SHA-256, 11 by directory, and 17 by path suffix. Every joined entry matched the bytes retrieved separately from its forecast build and measured build. Directory and path matching retain their official classifications. Shared entry bytes alone do not establish that every generated dependency or configuration is identical. The targeted combined proxy review includes generator and library path comparisons. [Every forecast and independent cell](joins.csv) · [Audit and unrounded results](audit.json)

The official v1 cross-host local-wall line reports 48 cells and mean absolute log error 0.16552. These represent 16 independent measurements and 24 distinct prediction/measurement tuples. Of the 152 eligible row-pairs in its 100/152 rank statistic, 88 compare the same entry, including 56 from the same run; among the other 64 strictly comparable row-pairs, 44 have the correct ordering. The official “other hosts” v1 proxy line reports 6 cells, error 0.03579 and 3/3 ranks. All three of those pair comparisons are for the same entry. Previously collected v0 target measurements joined by a path suffix are also present in the official v0 “other hosts” statistic. These aggregates describe the stored rows; their rank counts do not establish target ranking across multiple candidates. The separate source-audited combined-v0 error is 0.03685 on the same W7/8/10 target cells. [Independent rank and error recomputation](audit.py)

## Reproduction

The data cutoff is 6eaf1e8bace868fe8f8bba5f73e1f6ea9e83afca. In a new designated detached worktree we fetched origin/master at be74d5763d212864607d46521d74ebc0827f9f36 and checked out bench/moonshot, .just and Justfile from that revision. The live scorer path revision was 75297a1. The scorer's version reporter printed v1 at 81585172f5fb9e7a59f330124779ba084fd6bf04 for that worktree. The eval command prints the versions carried by forecasts and has no version banner of its own. The projection, evaluator digest and version are recorded. [Scorer identity](scorer.json) · [Official command](eval-command.txt)

In a checkout containing this snapshot, copy the two report scripts to the same relative directory in a detached worktree at the cutoff; the scripts write the two new receipts before the official reader scans the tree. Pin the master projection for historical replay. A new scoring round must first refresh from current master under the campaign protocol. The script below leaves the supplied original worktree untouched and takes a new worktree path.

```sh
bash research/astra-e/retrospective/nov19-edge/reproduce.sh /tmp/astra-retro-nov19-edge
```

A second fresh-cutoff worktree reproduced both CI receipts, the verified input IDs, official stdout and command echo, all join and coverage rows, the conditional edge cells and audit JSON byte-for-byte. Re-running the evaluator with Python hash seed 252 also reproduced its stdout.
