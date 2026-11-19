# What selective materialization can still change after frame retention

For a proper nonleaf subtree v, the all-frame interpreter executes G(v) terminal emissions and one child entry for each membership interval of every proper nonleaf descendant. Materializing the subtree with [bottom-up leaf absorption](../../bottom/) costs A′(v) = Σ R(u) over its nonleaf nodes. Its cut stream is then read in G(v) root-order fragments; re-entering a previously interrupted local run reads it again. The parent still enters v once per K(v) intervals in both plans. The **exact reduction in normalized actions** for a cut at v is therefore Σ K(u) over its proper nonleaf descendants minus A′(v). This refines the earlier restart-visitor tradeoff, which charged G for each of those descendants. A pure root writes its single output. A materialized full root already contains the final answer, with neither cut reads nor a second root output copy.

The oracle dynamic program minimizes the sum of routed overlaps, cut reads and materialization writes, plus a price for extra stored pairs. As before, the supported lower hull omits integer points strictly above its segments. Literal decoding of 3,030 cuts on 990 small trees checked the separate overlap and write accounts. Exhaustive enumeration agreed with all 4,950 scalarized optima. The structural prepass needed to obtain K and G for every node uses the fully known root sequence and is charged as an oracle, not as a decoder shortcut. [Test](check.json) · [Per-node membership and terminal-run counts](intervals.csv.xz) · [Supported frontier](frontier.csv)

On frozen input, 631 extra pairs reduce 18,707 normalized actions to 17,419 (−6.89%). Materializing all costs 10,230 extra pairs and 16,363 normalized actions. Across the ten archived inputs, the best supported point spending up to 10% of full extra storage saves 3.77–6.99% of merge actions. This includes the complete local stream construction, cut reads and remaining selector overlaps; it excludes recognizing the cut and stores all materialized arenas through the merge. Counts weight a selector overlap, a cut read and a bottom-up emission equally. The measured latency of those units differs.

## Local budget and stopping point

In the 40 paired frame rounds, first-level token merge used **9.85%** of summed complete inverse thread CPU on nine systematic admissions. Preparing the original LF rank index and transferring the mark used 19.12%, discovering/publishing the first phrase trie used 42.05%, and the second induction plus final output used 28.97%. Charging *zero* CPU to the entire first merge, while holding every other measured stage constant, gives .9363 [.8895,.9856] of Lazy inverse CPU. Reaching .80 on those same raw inverse measurements would still require a further 14.86% reduction in the mean remaining time. Frozen alone gives .9791 [.9133,1.0497] for a free first merge. Neither counterfactual models scanner overlap, target worker concurrency or changed cache residency. [Costs and paired calculations](account.txt)

The real geometry makes the dominant work explicit. Frozen / pooled 228-admission counts are 70,560 / 19,750,839 first-level original-rank queries and edge bytes; 3,689 / 987,682 nodes; 14,992 / 4,313,576 selectors; and 83,454 / 23,393,313 dictionary bytes. The second induction still performs 5,694 / 1,424,280 rank lookups, constructs 146,183 / 41,433,719 dictionary bytes, and uses 24,166 / 5,566,621 final token visits. These counts and first-stage fragment counts have different units. The earlier [prepaid-discovery experiment](../../../f2-scheduler/publication/) showed how much of first discovery could disappear given a new way to obtain final intervals and spellings.

Retain the frame constructor as the measured root-only milestone. Testing more selector cut thresholds in the current representation has diminishing returns for the throughput target. Even a free first merge leaves the local inverse short, and the proposed selective budget buys less than 7% of that phase's *normalized* work. A substantive next experiment should change how first-level interval answers are produced, or jointly eliminate discovery and final-output work. A changed construction warrants reopening this merge comparison if it changes the share or geometry. No target-machine or peak-RSS conclusion follows from the phase panel.

Run from this directory after the dense archived inputs are available:

```sh
export PYTHONDONTWRITEBYTECODE=1
python3 refine.py collect /tmp/f2-frame-intervals.csv.xz
cmp /tmp/f2-frame-intervals.csv.xz intervals.csv.xz
python3 refine.py frontier intervals.csv.xz | diff - frontier.csv
python3 check.py | diff - check.json
python3 account.py | diff - account.txt
```
