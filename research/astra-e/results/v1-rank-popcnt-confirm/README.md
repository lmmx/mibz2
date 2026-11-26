# Guarded population count on the complete local worker

The frozen implementation is 3651470cb0ef5458e75cf97d3de363a1c73f2f76. Its v1 gate again passed all 338 cases. The official scorer ran nine paired rounds at one and four workers, processing the checksum-verified full corpus twice per call. The local host reported four available CPUs. The output was 419,400,392 bytes per measured call. Master scorer path revision 604dac3f46569edd9ac9b6706b23a68d9d7d8f8d and the printed branch import v1 at 6a8a366cd786d58a528000b58da12fd2cfb47be8 are retained in the receipts. The manifest gives the exact invocation.

| Workers | Wall vs LazyCells 0.5.6 | Wall vs opend, directly paired | Official peak RSS vs LazyCells | Median candidate peak KiB |
| --- | ---: | ---: | ---: | ---: |
| 1 | 0.7436 [0.6343,0.8716] | 1.0421 [0.7661,1.4174] | 0.9973 | 79,912 |
| 4 | 0.7870 [0.6588,0.9401] | 1.1278 [0.9372,1.3571] | 0.9034 | 128,704 |

The official scorer's opend lines show opend/Lazy. The direct-pairs script applies its same paired estimator to the candidate and opend within every matched round. Additional CPU and peak-RSS comparisons are in direct-pairs.json. Peak-RSS values in the table follow the scorer's ratio of marginal medians. The supplementary paired peak-RSS ratios divide peaks within each observation pair and take their median.

Both local point estimates relative to LazyCells are at or below 0.80 with lower median peak memory. Both confidence intervals extend above 0.80. The nine same-run opend comparisons span parity. These measurements neither cover the required worker range nor establish the target i9 outcome. The distinct focused comparison to the immediate counted predecessor is in the F2 pages study at c3d56f7. CI has a separate candidate selection for the frozen implementation.
