# One-word directories after counted POPCNT and compact end runs

The split64 control is the compact eager constructor frozen at ab7076541459b8b66a137c9f1cc0804e0cd72de0. The prior packed-directory series at bd7f74f used software population count and an original-run sweep in preparation. This comparison combines the newer counted and hardware paths explicitly. Arrival of a new run at offset j sets bit j in a temporary 64-bit page word. In the existing prefix pass, if A starts precede the page, its base is A + bit0 - 1. Add the word's popcount to A and replace the word with (base << 44) | (word >> 1). At widths up to 44, the internal cuts and 20-bit base occupy disjoint ranges. A rank query reads this single word. Every run and subsequent computation uses the same compact eager representation as the control.

Each width passed every original-row rank check in the synthetic tests and matched the 205,197,794 oracle inverse bytes on 228 real blocks. Two independent, position-balanced 64-lap crossovers on CPUs 4 and 2 measured the complete inverse and its phases on the same nine systematic blocks. The high-water directory capacities were split64 337,488 B, packed32 449,968 B, packed40 359,984 B and packed44 327,248 B. The 1,048,576 B high-water original-run array is common to all arms.

| Layout | CPU4 setup+mark | CPU2 setup+mark | CPU4 discovery | CPU2 discovery | CPU4 complete CPU | CPU2 complete CPU |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Packed32 | 1.0424 | 1.0245 | 1.0128 | 1.0096 | 0.9854 [0.9560,1.0157] | 0.9939 [0.9669,1.0217] |
| Packed40 | 1.1487 | 1.1002 | 1.0493 | 1.0405 | 1.0364 [1.0039,1.0699] | 1.0270 [0.9944,1.0607] |
| Packed44 | 1.0867 | 1.0906 | 1.0946 | 1.0248 | 1.0281 [0.9952,1.0621] | 1.0124 [0.9831,1.0426] |

The matched comparison reproduced the setup penalty for widths requiring a constant non-power-of-two quotient. Reducing the lookup to one packed word yielded no repeatable complete-stage speedup after the earlier improvements. The split64 eager endpoint remains the leading endpoint implementation.

```sh
d=research/astra-e/formulations/f2-pages/payload/packed
bash "$d/run.sh" /tmp/astra-e-packed-endpoints
python3 "$d/report.py" /tmp/astra-e-packed-endpoints/{first,repeat}.csv
```
