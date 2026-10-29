checked p4: every block's output production's
checked p1: every block's output production's
checked p2: every block's output production's
checked t1: every block's output production's
checked c1: every block's output production's
checked b1: every block's output production's
checked t4: every block's output production's
checked c4: every block's output production's
checked b4: every block's output production's
431 blocks, 209.7 MB, 8 timed rounds; the T-vector phase only (build + walk), not the decoder; each worker pinned to its own physical core, checked
| arm | table | workers | live tables (MB, largest block) | MB/s, median | vs p4, paired: mean ±95% (min-max) | vs its own 4 workers, paired | build MB/s | walk MB/s | rounds misplaced |
|---|---|--:|--:|--:|--:|--:|--:|--:|--:|
| p4 | T-vector (production) | 4 | 14.4 | 923 | 1.000 ±0.000 (1.000-1.000) | 1.000 ±0.000 | 3994 | 1185 | 0 |
| p1 | T-vector (production) | 1 | 3.6 | 247 | 0.265 ±0.007 (0.249-0.274) | 0.265 ±0.007 | 1019 | 324 | 0 |
| p2 | T-vector (production) | 2 | 7.2 | 495 | 0.529 ±0.019 (0.495-0.568) | 0.529 ±0.019 | 2033 | 660 | 0 |
| t1 | 32-bit ψ | 1 | 3.6 | 207 | 0.229 ±0.009 (0.221-0.251) | 0.279 ±0.026 | 1052 | 257 | 0 |
| c1 | packed 20-bit ψ | 1 | 2.3 | 85 | 0.092 ±0.006 (0.083-0.103) | 0.258 ±0.008 | 155 | 191 | 0 |
| b1 | split 16 + 4-bit ψ (libbzip2) | 1 | 2.3 | 133 | 0.145 ±0.012 (0.124-0.172) | 0.260 ±0.026 | 433 | 191 | 0 |
| t4 | 32-bit ψ | 4 | 14.5 | 794 | 0.828 ±0.078 (0.679-0.946) | 1.000 ±0.000 | 4154 | 974 | 0 |
| c4 | packed 20-bit ψ | 4 | 9.1 | 330 | 0.359 ±0.025 (0.321-0.416) | 1.000 ±0.000 | 617 | 716 | 0 |
| b4 | split 16 + 4-bit ψ (libbzip2) | 4 | 9.1 | 533 | 0.564 ±0.059 (0.441-0.681) | 1.000 ±0.000 | 1804 | 765 | 0 |
