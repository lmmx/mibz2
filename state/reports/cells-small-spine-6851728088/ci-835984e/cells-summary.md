
### onebyte, 1 thread (20 MB out; ratios: arm / mibz2@bin/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2@bin/ab | 10 | 0.015 | 0.016 | 0.017 | 0.017 | 33 MB | 1 | 1 | 0.06 | yes |
| mibz2-cells | 10 | 0.016 | 0.017 | 0.018 | 0.019 | 36 MB | 1.044 [0.967, 1.128] | 1.032 [0.953, 1.117] | 0.08 | yes |

### pattern, 1 thread (20 MB out; ratios: arm / mibz2@bin/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2@bin/ab | 10 | 0.020 | 0.020 | 0.021 | 0.022 | 16 MB | 1 | 1 | 0.14 | yes |
| mibz2-cells | 10 | 0.034 | 0.035 | 0.035 | 0.036 | 18 MB | 1.782 [1.599, 1.986] | 1.721 [1.553, 1.907] | 0.09 | yes |

### random, 1 thread (8 MB out; ratios: arm / mibz2@bin/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2@bin/ab | 10 | 0.083 | 0.090 | 0.085 | 0.091 | 23 MB | 1 | 1 | 0.00 | yes |
| mibz2-cells | 10 | 0.087 | 0.089 | 0.088 | 0.091 | 23 MB | 1.001 [0.984, 1.018] | 1.001 [0.984, 1.018] | 0.01 | yes |

### records, 1 thread (10 MB out; ratios: arm / mibz2@bin/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2@bin/ab | 10 | 0.089 | 0.094 | 0.091 | 0.095 | 23 MB | 1 | 1 | 0.04 | yes |
| mibz2-cells | 10 | 0.089 | 0.094 | 0.091 | 0.096 | 23 MB | 0.993 [0.979, 1.007] | 0.993 [0.979, 1.007] | 0.02 | yes |

### runs, 1 thread (10 MB out; ratios: arm / mibz2@bin/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2@bin/ab | 10 | 0.010 | 0.010 | 0.012 | 0.012 | 23 MB | 1 | 1 | 0.00 | yes |
| mibz2-cells | 10 | 0.010 | 0.010 | 0.012 | 0.012 | 23 MB | 0.936 [0.800, 1.095] | 0.940 [0.818, 1.081] | 0.00 | yes |

### text, 1 thread (13 MB out; ratios: arm / mibz2@bin/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2@bin/ab | 10 | 0.046 | 0.051 | 0.048 | 0.053 | 19 MB | 1 | 1 | 0.07 | yes |
| mibz2-cells | 10 | 0.047 | 0.049 | 0.049 | 0.051 | 19 MB | 0.978 [0.935, 1.024] | 0.978 [0.937, 1.022] | 0.00 | yes |

### wikidata, 1 thread (210 MB out; ratios: arm / mibz2@bin/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2@bin/ab | 10 | 0.317 | 0.334 | 0.319 | 0.335 | 26 MB | 1 | 1 | 0.04 | yes |
| mibz2-cells | 10 | 0.349 | 0.361 | 0.351 | 0.363 | 34 MB | 1.099 [1.041, 1.160] | 1.098 [1.041, 1.159] | 0.01 | yes |
