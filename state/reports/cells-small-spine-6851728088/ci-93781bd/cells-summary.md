
### onebyte, 1 thread (20 MB out; ratios: arm / mibz2@bin/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2@bin/ab | 10 | 0.017 | 0.018 | 0.019 | 0.019 | 33 MB | 1 | 1 | 0.04 | yes |
| mibz2-cells | 10 | 0.019 | 0.020 | 0.021 | 0.021 | 36 MB | 1.095 [1.081, 1.109] | 1.085 [1.072, 1.099] | 0.00 | yes |

### pattern, 1 thread (20 MB out; ratios: arm / mibz2@bin/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2@bin/ab | 10 | 0.026 | 0.026 | 0.028 | 0.028 | 16 MB | 1 | 1 | 0.07 | yes |
| mibz2-cells | 10 | 0.044 | 0.045 | 0.046 | 0.046 | 18 MB | 1.717 [1.712, 1.723] | 1.674 [1.669, 1.679] | 0.08 | yes |

### random, 1 thread (8 MB out; ratios: arm / mibz2@bin/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2@bin/ab | 10 | 0.108 | 0.109 | 0.110 | 0.111 | 23 MB | 1 | 1 | 0.00 | yes |
| mibz2-cells | 10 | 0.108 | 0.109 | 0.110 | 0.110 | 23 MB | 0.989 [0.963, 1.015] | 0.989 [0.963, 1.015] | 0.00 | yes |

### records, 1 thread (10 MB out; ratios: arm / mibz2@bin/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2@bin/ab | 10 | 0.117 | 0.118 | 0.119 | 0.119 | 23 MB | 1 | 1 | 0.01 | yes |
| mibz2-cells | 10 | 0.117 | 0.117 | 0.118 | 0.119 | 23 MB | 0.992 [0.985, 1.000] | 0.992 [0.985, 0.999] | 0.01 | yes |

### runs, 1 thread (10 MB out; ratios: arm / mibz2@bin/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2@bin/ab | 10 | 0.011 | 0.012 | 0.013 | 0.013 | 23 MB | 1 | 1 | 0.00 | yes |
| mibz2-cells | 10 | 0.011 | 0.012 | 0.013 | 0.013 | 23 MB | 0.998 [0.985, 1.012] | 0.999 [0.984, 1.014] | 0.20 | yes |

### text, 1 thread (13 MB out; ratios: arm / mibz2@bin/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2@bin/ab | 10 | 0.057 | 0.058 | 0.059 | 0.059 | 19 MB | 1 | 1 | 0.01 | yes |
| mibz2-cells | 10 | 0.058 | 0.058 | 0.059 | 0.060 | 19 MB | 1.034 [0.973, 1.100] | 1.033 [0.973, 1.097] | 0.02 | yes |

### wikidata, 1 thread (210 MB out; ratios: arm / mibz2@bin/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2@bin/ab | 10 | 0.427 | 0.431 | 0.429 | 0.433 | 26 MB | 1 | 1 | 0.03 | yes |
| mibz2-cells | 10 | 0.432 | 0.434 | 0.433 | 0.436 | 34 MB | 1.005 [0.995, 1.014] | 1.005 [0.995, 1.014] | 0.01 | yes |
