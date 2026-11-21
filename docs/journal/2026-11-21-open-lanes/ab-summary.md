
### wikidata, 1 thread (210 MB out; ratios: arm / mibz2-workers-lazy@ab-base, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2-workers-lazy | 10 | 0.533 | 0.580 | 0.532 | 0.581 | 38 MB | 0.914 [0.856, 0.975] | 0.911 [0.855, 0.971] | 0.04 | yes |
| mibz2-workers-lazy@ab-base | 10 | 0.610 | 0.632 | 0.609 | 0.633 | 37 MB | 1 | 1 | 0.06 | yes |
| mibz2 | 10 | 0.472 | 0.588 | 0.470 | 0.578 | 28 MB | 0.875 [0.808, 0.948] | 0.869 [0.804, 0.940] | 0.05 | yes |
| mibz2@ab-base | 10 | 0.493 | 0.593 | 0.496 | 0.591 | 28 MB | 0.922 [0.860, 0.989] | 0.919 [0.862, 0.980] | 0.04 | yes |

### wikidata, 2 threads (210 MB out; ratios: arm / mibz2-workers-lazy@ab-base, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2-workers-lazy | 10 | 0.294 | 0.335 | 0.568 | 0.628 | 51 MB | 0.924 [0.860, 0.993] | 0.929 [0.856, 1.008] | 0.07 | yes |
| mibz2-workers-lazy@ab-base | 10 | 0.330 | 0.359 | 0.622 | 0.682 | 49 MB | 1 | 1 | 0.06 | yes |
| mibz2 | 10 | 0.306 | 0.346 | 0.597 | 0.651 | 57 MB | 0.952 [0.897, 1.011] | 0.953 [0.900, 1.008] | 0.08 | yes |
| mibz2@ab-base | 10 | 0.320 | 0.339 | 0.616 | 0.647 | 58 MB | 0.934 [0.879, 0.992] | 0.952 [0.896, 1.011] | 0.08 | yes |

### wikidata, 4 threads (210 MB out; ratios: arm / mibz2-workers-lazy@ab-base, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2-workers-lazy | 10 | 0.183 | 0.201 | 0.620 | 0.654 | 75 MB | 0.972 [0.907, 1.043] | 0.959 [0.909, 1.011] | 0.00 | yes |
| mibz2-workers-lazy@ab-base | 10 | 0.186 | 0.198 | 0.650 | 0.698 | 75 MB | 1 | 1 | 0.00 | yes |
| mibz2 | 10 | 0.176 | 0.205 | 0.625 | 0.705 | 89 MB | 0.993 [0.940, 1.049] | 1.002 [0.957, 1.050] | 0.01 | yes |
| mibz2@ab-base | 10 | 0.190 | 0.207 | 0.645 | 0.697 | 94 MB | 1.042 [0.946, 1.148] | 1.012 [0.951, 1.077] | 0.08 | yes |
