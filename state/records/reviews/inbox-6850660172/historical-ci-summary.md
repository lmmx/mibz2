
### onebyte, 1 thread (20 MB out; ratios: arm / mibz2@bin/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2@bin/ab | 10 | 0.018 | 0.018 | 0.020 | 0.020 | 33 MB | 1 | 1 | 0.18 | yes |
| mibz2-cells | 10 | 0.020 | 0.021 | 0.022 | 0.022 | 36 MB | 1.109 [1.095, 1.123] | 1.097 [1.082, 1.113] | 0.14 | yes |

### pattern, 1 thread (20 MB out; ratios: arm / mibz2@bin/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2@bin/ab | 10 | 0.026 | 0.027 | 0.028 | 0.028 | 16 MB | 1 | 1 | 0.05 | yes |
| mibz2-cells | 10 | 0.059 | 0.059 | 0.061 | 0.061 | 19 MB | 2.232 [2.217, 2.247] | 2.152 [2.138, 2.165] | 0.00 | yes |

### random, 1 thread (8 MB out; ratios: arm / mibz2@bin/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2@bin/ab | 10 | 0.111 | 0.112 | 0.113 | 0.114 | 23 MB | 1 | 1 | 0.06 | yes |
| mibz2-cells | 10 | 0.111 | 0.112 | 0.113 | 0.114 | 23 MB | 1.006 [0.999, 1.014] | 1.006 [0.999, 1.014] | 0.00 | yes |

### records, 1 thread (10 MB out; ratios: arm / mibz2@bin/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2@bin/ab | 10 | 0.120 | 0.121 | 0.121 | 0.123 | 23 MB | 1 | 1 | 0.07 | yes |
| mibz2-cells | 10 | 0.120 | 0.121 | 0.122 | 0.122 | 23 MB | 0.997 [0.986, 1.008] | 0.996 [0.986, 1.007] | 0.00 | yes |

### runs, 1 thread (10 MB out; ratios: arm / mibz2@bin/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2@bin/ab | 10 | 0.012 | 0.012 | 0.014 | 0.014 | 23 MB | 1 | 1 | 0.00 | yes |
| mibz2-cells | 10 | 0.012 | 0.012 | 0.014 | 0.014 | 23 MB | 1.004 [0.983, 1.026] | 1.005 [0.984, 1.027] | 0.18 | yes |

### text, 1 thread (13 MB out; ratios: arm / mibz2@bin/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2@bin/ab | 10 | 0.059 | 0.059 | 0.060 | 0.061 | 19 MB | 1 | 1 | 0.00 | yes |
| mibz2-cells | 10 | 0.058 | 0.059 | 0.060 | 0.061 | 19 MB | 1.012 [0.980, 1.046] | 1.012 [0.980, 1.045] | 0.00 | yes |

### wikidata, 1 thread (210 MB out; ratios: arm / mibz2@bin/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2@bin/ab | 10 | 0.434 | 0.442 | 0.435 | 0.443 | 26 MB | 1 | 1 | 0.02 | yes |
| mibz2-cells | 10 | 0.436 | 0.442 | 0.438 | 0.444 | 34 MB | 1.000 [0.990, 1.011] | 1.000 [0.989, 1.011] | 0.02 | yes |
