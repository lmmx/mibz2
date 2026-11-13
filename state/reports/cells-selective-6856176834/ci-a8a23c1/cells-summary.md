
### onebyte, 1 thread (20 MB out; ratios: arm / mibz2-cells@bin/original-cells-ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2@bin/ab | 10 | 0.017 | 0.017 | 0.019 | 0.019 | 33 MB | 0.903 [0.898, 0.909] | 0.910 [0.906, 0.914] | 0.04 | yes |
| mibz2-cells@bin/original-cells-ab | 10 | 0.019 | 0.019 | 0.021 | 0.021 | 36 MB | 1 | 1 | 0.00 | yes |
| mibz2-cells | 10 | 0.018 | 0.019 | 0.020 | 0.021 | 36 MB | 0.973 [0.963, 0.983] | 0.976 [0.968, 0.985] | 0.36 | yes |

### pattern, 1 thread (20 MB out; ratios: arm / mibz2-cells@bin/original-cells-ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2@bin/ab | 10 | 0.026 | 0.026 | 0.028 | 0.028 | 16 MB | 0.440 [0.432, 0.449] | 0.457 [0.447, 0.467] | 0.07 | yes |
| mibz2-cells@bin/original-cells-ab | 10 | 0.059 | 0.059 | 0.061 | 0.061 | 19 MB | 1 | 1 | 0.00 | yes |
| mibz2-cells | 10 | 0.046 | 0.046 | 0.048 | 0.048 | 19 MB | 0.775 [0.773, 0.777] | 0.781 [0.779, 0.783] | 0.04 | yes |

### random, 1 thread (8 MB out; ratios: arm / mibz2-cells@bin/original-cells-ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2@bin/ab | 10 | 0.109 | 0.109 | 0.110 | 0.111 | 23 MB | 1.004 [0.994, 1.014] | 1.004 [0.994, 1.014] | 0.08 | yes |
| mibz2-cells@bin/original-cells-ab | 10 | 0.108 | 0.109 | 0.110 | 0.110 | 23 MB | 1 | 1 | 0.00 | yes |
| mibz2-cells | 10 | 0.108 | 0.109 | 0.110 | 0.110 | 23 MB | 0.997 [0.992, 1.002] | 0.997 [0.993, 1.002] | 0.00 | yes |

### records, 1 thread (10 MB out; ratios: arm / mibz2-cells@bin/original-cells-ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2@bin/ab | 10 | 0.117 | 0.118 | 0.119 | 0.119 | 23 MB | 1.007 [0.999, 1.016] | 1.007 [0.999, 1.016] | 0.01 | yes |
| mibz2-cells@bin/original-cells-ab | 10 | 0.117 | 0.117 | 0.118 | 0.118 | 23 MB | 1 | 1 | 0.01 | yes |
| mibz2-cells | 10 | 0.117 | 0.117 | 0.118 | 0.119 | 23 MB | 1.006 [0.996, 1.016] | 1.006 [0.996, 1.016] | 0.01 | yes |

### runs, 1 thread (10 MB out; ratios: arm / mibz2-cells@bin/original-cells-ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2@bin/ab | 10 | 0.011 | 0.011 | 0.013 | 0.013 | 23 MB | 1.003 [0.988, 1.017] | 1.001 [0.987, 1.016] | 0.21 | yes |
| mibz2-cells@bin/original-cells-ab | 10 | 0.011 | 0.011 | 0.013 | 0.013 | 23 MB | 1 | 1 | 0.21 | yes |
| mibz2-cells | 10 | 0.011 | 0.011 | 0.013 | 0.013 | 23 MB | 1.005 [0.994, 1.016] | 1.004 [0.993, 1.014] | 0.42 | yes |

### text, 1 thread (13 MB out; ratios: arm / mibz2-cells@bin/original-cells-ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2@bin/ab | 10 | 0.057 | 0.057 | 0.059 | 0.059 | 19 MB | 0.998 [0.992, 1.004] | 0.998 [0.993, 1.004] | 0.01 | yes |
| mibz2-cells@bin/original-cells-ab | 10 | 0.057 | 0.058 | 0.059 | 0.059 | 19 MB | 1 | 1 | 0.01 | yes |
| mibz2-cells | 10 | 0.057 | 0.058 | 0.059 | 0.059 | 19 MB | 1.000 [0.997, 1.004] | 1.000 [0.997, 1.004] | 0.01 | yes |

### wikidata, 1 thread (210 MB out; ratios: arm / mibz2-cells@bin/original-cells-ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2@bin/ab | 10 | 0.429 | 0.431 | 0.431 | 0.433 | 26 MB | 0.998 [0.994, 1.001] | 0.998 [0.994, 1.002] | 0.01 | yes |
| mibz2-cells@bin/original-cells-ab | 10 | 0.432 | 0.432 | 0.434 | 0.434 | 34 MB | 1 | 1 | 0.01 | yes |
| mibz2-cells | 10 | 0.421 | 0.422 | 0.423 | 0.424 | 34 MB | 0.976 [0.973, 0.979] | 0.976 [0.973, 0.979] | 0.02 | yes |
