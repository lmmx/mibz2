# Whole-worker scoring the eager eight-byte F2 run

The frozen source is ab7076541459b8b66a137c9f1cc0804e0cd72de0. After refreshing the scorer from master, the frozen tree ran v1 at 6a8a366cd786d58a528000b58da12fd2cfb47be8, master path 604dac3f46569edd9ac9b6706b23a68d9d7d8f8d. The complete 338-case gate passed. Nine paired rounds measured the canonical corpus twice per call at one and four workers, with opend and LazyCells in the same rounds. The original running process continued across the orchestration restart and produced all 54 observations.

| Workers | Wall vs LazyCells 0.5.6 | Wall vs opend, directly paired | Official peak RSS vs LazyCells | Median candidate peak KiB |
| --- | ---: | ---: | ---: | ---: |
| 1 | 0.8251 [0.6920,0.9838] | 0.8800 [0.7604,1.0183] | 0.9944 | 79,752 |
| 4 | 0.9839 [0.6855,1.4122] | 1.2421 [0.9498,1.6244] | 0.9209 | 124,584 |

The scorer classified the W1 comparison with Lazy as faster. The W4 interval was wide. The scored predecessor POPCNT used a different randomized nine-round series; its marginals cannot identify the compact representation's incremental wall effect. The native crossover at ab70765 directly measured that predecessor. The smaller original-run allocation and the two whole-process median peaks are preserved separately. The CI record for the earlier POPCNT candidate covers a distinct immutable source. The i9 throughput and memory objective remains unverified for this compact implementation.

The immutable candidate, compiler and exact command are in manifest.txt and official-register.jsonl. Direct-pairs.py reapplies the scorer's paired statistic to comparisons from the same rounds. The official memory ratios in the table are the ratios of marginal medians.
