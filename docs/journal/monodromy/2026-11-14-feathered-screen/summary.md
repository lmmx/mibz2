
### P1  (196 blocks)

checks: blocks 196, finished 196, hash_ok 196, rows_equal 196, one_point_left 196, nodes_equal_sum_m 196, S_equals_census_cells 189, r_equals_census_letters 188, S_equal_steps_is_r_minus_1 196, max_growth_ratio 0.5, rounds_with_r_growing_before_coalescing 871, total_rounds 2174999, total_joins 904, empty_m0 0, empty_no_row 0

| quantity | min / q1 / median / q3 / max |
|---|---|
| rounds | 5779 / 10551 / 11001 / 11402 / 14045 |
| empty rounds (m = 0) | 0 / 0 / 0 / 0 / 0 |
| empty rounds (no row removed) | 0 / 0 / 0 / 0 / 0 |
| total work / S  (cells scanned + fragments written before coalescing) | 474.47 / 850.98 / 882.34 / 917.01 / 1129.30 |
| total work / S  (fragments counted after coalescing) | 474.47 / 850.98 / 882.34 / 917.01 / 1129.30 |
| total work / n  (per output byte of the block) | 54.172 / 92.099 / 100.164 / 108.464 / 167.293 |
| selection: cells scanned / S | 316.06 / 567.04 / 587.94 / 611.05 / 752.60 |
| rebuild: fragments written / S | 158.41 / 283.94 / 294.40 / 305.96 / 376.71 |
| first round alone: (cells + fragments) / S | 0.99 / 1.03 / 1.05 / 1.06 / 1.11 |
| max r reached / r | 1.000 / 1.000 / 1.000 / 1.000 / 1.000 |
| largest r_new before coalescing / r | 0.686 / 0.707 / 0.711 / 0.714 / 0.726 |
| sum m / S  (word compositions) | 1.219 / 1.294 / 1.306 / 1.317 / 1.353 |
| max over rounds of (r_new - r)/m | 0.000 / 0.167 / 0.333 / 0.333 / 0.500 |
| rows removed in round 1 / n | 0.216 / 0.234 / 0.240 / 0.248 / 0.263 |
| rows removed in rounds 1-4 / n | 0.525 / 0.551 / 0.560 / 0.567 / 0.645 |
| rows removed in rounds 1-16 / n | 0.680 / 0.715 / 0.720 / 0.727 / 0.804 |
| rounds until 50% of rows removed | 3 / 3 / 4 / 4 / 4 |
| rounds until 90% of rows removed | 89 / 367 / 433 / 499 / 693 |
| rounds until 99% of rows removed | 2960 / 6961 / 7409 / 7734 / 10578 |
| S / r (serial) | 1.698 / 1.805 / 1.848 / 1.882 / 1.969 |

decision rule: median rounds 11001 (bound about 200: NOT met); median total work 882.34 S (bound 1.3 S: NOT met); blocks with rounds <= 200: 0 of 196; blocks with work <= 1.3 S: 0 of 196; least work on a block 474.47 S

### P2  (196 blocks)

checks: blocks 196, finished 196, hash_ok 196, rows_equal 196, one_point_left 196, nodes_equal_sum_m 196, S_equals_census_cells 189, r_equals_census_letters 188, S_equal_steps_is_r_minus_1 196, max_growth_ratio 0.0, rounds_with_r_growing_before_coalescing 0, total_rounds 24265, total_joins 15832, empty_m0 0, empty_no_row 0

| quantity | min / q1 / median / q3 / max |
|---|---|
| rounds | 47 / 67 / 108 / 172 / 318 |
| empty rounds (m = 0) | 0 / 0 / 0 / 0 / 0 |
| empty rounds (no row removed) | 0 / 0 / 0 / 0 / 0 |
| total work / S  (cells scanned + fragments written before coalescing) | 5.48 / 5.92 / 6.08 / 6.36 / 7.66 |
| total work / S  (fragments counted after coalescing) | 5.48 / 5.92 / 6.08 / 6.36 / 7.66 |
| total work / n  (per output byte of the block) | 0.582 / 0.661 / 0.696 / 0.745 / 1.026 |
| selection: cells scanned / S | 3.52 / 3.81 / 3.92 / 4.11 / 5.00 |
| rebuild: fragments written / S | 1.96 / 2.10 / 2.16 / 2.25 / 2.67 |
| first round alone: (cells + fragments) / S | 0.99 / 1.03 / 1.04 / 1.06 / 1.10 |
| max r reached / r | 1.000 / 1.000 / 1.000 / 1.000 / 1.000 |
| largest r_new before coalescing / r | 0.678 / 0.699 / 0.703 / 0.706 / 0.719 |
| sum m / S  (word compositions) | 0.770 / 0.792 / 0.805 / 0.814 / 0.848 |
| max over rounds of (r_new - r)/m | -0.250 / -0.167 / 0.000 / 0.000 / 0.000 |
| rows removed in round 1 / n | 0.360 / 0.368 / 0.371 / 0.374 / 0.383 |
| rows removed in rounds 1-4 / n | 0.780 / 0.798 / 0.802 / 0.807 / 0.827 |
| rows removed in rounds 1-16 / n | 0.981 / 0.989 / 0.990 / 0.991 / 0.993 |
| rounds until 50% of rows removed | 2 / 2 / 2 / 2 / 2 |
| rounds until 90% of rows removed | 6 / 6 / 7 / 7 / 8 |
| rounds until 99% of rows removed | 15 / 16 / 17 / 17 / 20 |
| S / r (serial) | 1.698 / 1.805 / 1.848 / 1.882 / 1.969 |

decision rule: median rounds 108 (bound about 200: met); median total work 6.08 S (bound 1.3 S: NOT met); blocks with rounds <= 200: 169 of 196; blocks with work <= 1.3 S: 0 of 196; least work on a block 5.48 S

### P3  (196 blocks)

checks: blocks 196, finished 196, hash_ok 196, rows_equal 196, one_point_left 196, nodes_equal_sum_m 196, S_equals_census_cells 189, r_equals_census_letters 188, S_equal_steps_is_r_minus_1 196, max_growth_ratio 0.333333, rounds_with_r_growing_before_coalescing 514, total_rounds 2064884, total_joins 403, empty_m0 0, empty_no_row 0

| quantity | min / q1 / median / q3 / max |
|---|---|
| rounds | 3176 / 10030 / 10830 / 11343 / 14044 |
| empty rounds (m = 0) | 0 / 0 / 0 / 0 / 0 |
| empty rounds (no row removed) | 0 / 0 / 0 / 0 / 0 |
| total work / S  (cells scanned + fragments written before coalescing) | 261.37 / 828.62 / 866.76 / 896.54 / 1095.11 |
| total work / S  (fragments counted after coalescing) | 261.37 / 828.62 / 866.76 / 896.54 / 1095.11 |
| total work / n  (per output byte of the block) | 28.102 / 89.285 / 96.911 / 106.745 / 167.186 |
| selection: cells scanned / S | 174.04 / 552.19 / 577.60 / 597.46 / 729.85 |
| rebuild: fragments written / S | 87.33 / 276.44 / 289.16 / 299.08 / 365.26 |
| first round alone: (cells + fragments) / S | 0.98 / 1.01 / 1.03 / 1.04 / 1.09 |
| max r reached / r | 1.000 / 1.000 / 1.000 / 1.000 / 1.000 |
| largest r_new before coalescing / r | 0.657 / 0.676 / 0.679 / 0.682 / 0.692 |
| sum m / S  (word compositions) | 1.118 / 1.197 / 1.216 / 1.228 / 1.267 |
| max over rounds of (r_new - r)/m | 0.000 / 0.071 / 0.292 / 0.333 / 0.333 |
| rows removed in round 1 / n | 0.255 / 0.272 / 0.279 / 0.284 / 0.308 |
| rows removed in rounds 1-4 / n | 0.565 / 0.595 / 0.603 / 0.610 / 0.672 |
| rows removed in rounds 1-16 / n | 0.717 / 0.747 / 0.755 / 0.764 / 0.839 |
| rounds until 50% of rows removed | 3 / 3 / 3 / 3 / 3 |
| rounds until 90% of rows removed | 72 / 290 / 350 / 422 / 665 |
| rounds until 99% of rows removed | 1570 / 6606 / 7276 / 7650 / 10575 |
| S / r (serial) | 1.698 / 1.805 / 1.848 / 1.882 / 1.969 |

decision rule: median rounds 10830 (bound about 200: NOT met); median total work 866.76 S (bound 1.3 S: NOT met); blocks with rounds <= 200: 0 of 196; blocks with work <= 1.3 S: 0 of 196; least work on a block 261.37 S

### P3L  (196 blocks)

checks: blocks 196, finished 196, hash_ok 196, rows_equal 196, one_point_left 196, nodes_equal_sum_m 196, S_equals_census_cells 189, r_equals_census_letters 188, S_equal_steps_is_r_minus_1 196, max_growth_ratio 0.333333, rounds_with_r_growing_before_coalescing 513, total_rounds 885288, total_joins 227, empty_m0 0, empty_no_row 0

| quantity | min / q1 / median / q3 / max |
|---|---|
| rounds | 2039 / 4247 / 4550 / 4870 / 6955 |
| empty rounds (m = 0) | 0 / 0 / 0 / 0 / 0 |
| empty rounds (no row removed) | 0 / 0 / 0 / 0 / 0 |
| total work / S  (cells scanned + fragments written before coalescing) | 122.65 / 294.53 / 329.62 / 368.58 / 627.66 |
| total work / S  (fragments counted after coalescing) | 122.65 / 294.53 / 329.62 / 368.58 / 627.66 |
| total work / n  (per output byte of the block) | 13.638 / 33.112 / 37.671 / 43.440 / 91.146 |
| selection: cells scanned / S | 81.62 / 196.20 / 219.60 / 245.57 / 418.29 |
| rebuild: fragments written / S | 41.03 / 98.33 / 110.01 / 123.01 / 209.38 |
| first round alone: (cells + fragments) / S | 0.98 / 1.01 / 1.03 / 1.04 / 1.09 |
| max r reached / r | 1.000 / 1.000 / 1.000 / 1.000 / 1.000 |
| largest r_new before coalescing / r | 0.657 / 0.675 / 0.678 / 0.681 / 0.691 |
| sum m / S  (word compositions) | 0.884 / 0.946 / 0.964 / 0.980 / 1.030 |
| max over rounds of (r_new - r)/m | 0.000 / 0.000 / 0.200 / 0.250 / 0.333 |
| rows removed in round 1 / n | 0.304 / 0.318 / 0.323 / 0.326 / 0.348 |
| rows removed in rounds 1-4 / n | 0.664 / 0.687 / 0.693 / 0.701 / 0.759 |
| rows removed in rounds 1-16 / n | 0.822 / 0.848 / 0.858 / 0.869 / 0.931 |
| rounds until 50% of rows removed | 2 / 2 / 2 / 2 / 3 |
| rounds until 90% of rows removed | 10 / 32 / 41 / 54 / 134 |
| rounds until 99% of rows removed | 622 / 1780 / 2256 / 2575 / 5008 |
| S / r (serial) | 1.698 / 1.805 / 1.848 / 1.882 / 1.969 |

decision rule: median rounds 4550 (bound about 200: NOT met); median total work 329.62 S (bound 1.3 S: NOT met); blocks with rounds <= 200: 0 of 196; blocks with work <= 1.3 S: 0 of 196; least work on a block 122.65 S
