# Exact local pruning of the segmentation recurrence

Within an old source interval I give every physical position its vote label (a,old target). For a maximal run of one such label t, retain only the first and last *strictly interior* allowed seams if they exist, together with any seams at changes of vote label. An optimum of the registered nonnegative-penalty problem always survives this restriction.

To see this, choose optimal segment labels and fix the labels carried at the first and last row of the run. This keeps the assignments on all adjoining segments compatible. If both ends choose t, remove every interior boundary. If just the left end chooses t, move its transition to the rightmost allowed seam and use the old right label afterward; apply the mirror operation when just the right end chooses t. If neither end chooses t but some interior segment does, take the left endpoint label up to the first allowed seam, t from there to the last, and the right endpoint label after that. The original solution had at least two seams in this case. If none of the segments chooses t, at most one seam is needed to switch between distinct endpoint labels, and its position does not affect the reward. Each change preserves or increases correctly predicted mass and uses no extra cut. Repeating independently on the runs proves the restriction. With labels a⁵b⁸a⁵, only allowed cuts 6,12, and charge one, the optimum is 14 with both cuts; permitting only one gives at most 11. Thus retaining both extremes can be necessary.

For source I let e=|I|−maxₜ count_I(t), already available from its histogram. Perfect prediction can gain at most e. If e≤λ, returning no cut in this source is optimal, including a possible tie. Likewise a source with no eligible cut needs only its original modal count. Its support ends may still generate allowed cuts in other sources, and must still be formed. The two pruning rules apply to the exact one-round optimization over the original simultaneously safe seam set. [Segmentation proof](../README.md)

On the registered ten-block trajectories from the native source at `623388b843a92d090767aca9a36205a30fe5f883`, the reference and reduced candidate sets achieved the same maximizing score at each of 307 snapshots. The instrument follows the reference boundaries when advancing to the next pass, so it makes no tie-breaking assumption about later rounds. Original and pruned solutions also happened to select the same number of new seams at every snapshot. Independently, brute enumeration of 62,348 arbitrary vote-sequence, seam-mask and charge cases confirmed the preserved optimum, including 27,354 cases with a strictly smaller candidate set. [Exact small cases](small.json) · [Reference data](real.csv)

| λ | Snapshots | Original DP fragments | After within-run reduction | After skipping unprofitable sources | Reduction from original |
|---:|---:|---:|---:|---:|---:|
| 1 | 88 | 6,673,841 | 6,564,518 | 5,245,682 | 21.40% |
| 4 | 77 | 5,656,673 | 5,592,342 | 4,895,925 | 13.45% |
| 16 | 75 | 5,537,937 | 5,495,238 | 5,017,803 | 9.39% |
| 64 | 67 | 4,749,898 | 4,726,177 | 4,394,878 | 7.47% |

These are DP fragment counts. The extra classifier, original run and histogram construction, allowed-seam bookkeeping, and later decoding each have a cost. The structural screen includes both complete reference and restricted optimizations to verify the objective. It provides no end-to-end speed or memory measurement. [Instrumentation](panel.cpp) · [Donor identities](sources.json) · [Original wire identities](inputs.json)

```bash
e=$(dirname "$(rg --files | rg '/all-marks/exceptions/PROOF.md$')")
d="$e/return/collation/interval-states/refine/proof/prune"
python3 -B "$d/small.py" | diff - "$d/small.json"
python3 -B "$d/build.py" "$tmp"
python3 -B "$d/run.py" "$tmp/prune" "$wires" | diff - "$d/real.csv"
```
