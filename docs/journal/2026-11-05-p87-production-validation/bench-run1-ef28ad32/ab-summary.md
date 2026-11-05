## ab summary (reference: mibz2@base/ab)

### onebyte, 1 thread (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.021 | 0.022 | 0.023 | 0.024 | 33 MB | 0.995 [0.985, 1.006] | 1.002 [0.990, 1.014] | 0.00 | yes |
| mibz2@base/ab | 10 | 0.021 | 0.022 | 0.023 | 0.024 | 33 MB | 1 | 1 | 0.27 | yes |

### onebyte, 2 threads (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.021 | 0.022 | 0.023 | 0.024 | 32 MB | 1.000 [0.981, 1.020] | 1.000 [0.981, 1.020] | 0.00 | yes |
| mibz2@base/ab | 10 | 0.021 | 0.022 | 0.023 | 0.024 | 33 MB | 1 | 1 | 0.17 | yes |

### onebyte, 4 threads (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.021 | 0.022 | 0.023 | 0.023 | 32 MB | 0.991 [0.978, 1.004] | 0.992 [0.978, 1.006] | 0.20 | yes |
| mibz2@base/ab | 10 | 0.021 | 0.022 | 0.023 | 0.024 | 33 MB | 1 | 1 | 0.18 | yes |

### pattern, 1 thread (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.045 | 0.047 | 0.047 | 0.048 | 16 MB | 0.998 [0.990, 1.006] | 0.999 [0.990, 1.007] | 0.04 | yes |
| mibz2@base/ab | 10 | 0.046 | 0.047 | 0.048 | 0.049 | 17 MB | 1 | 1 | 0.01 | yes |

### pattern, 2 threads (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.026 | 0.028 | 0.054 | 0.056 | 22 MB | 0.970 [0.926, 1.017] | 0.984 [0.968, 1.001] | 0.11 | yes |
| mibz2@base/ab | 10 | 0.027 | 0.028 | 0.056 | 0.057 | 23 MB | 1 | 1 | 0.11 | yes |

### pattern, 4 threads (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.027 | 0.028 | 0.099 | 0.102 | 38 MB | 0.970 [0.895, 1.051] | 1.014 [0.973, 1.058] | 0.06 | yes |
| mibz2@base/ab | 10 | 0.028 | 0.029 | 0.098 | 0.103 | 41 MB | 1 | 1 | 0.01 | yes |

### random, 1 thread (8 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.160 | 0.167 | 0.162 | 0.169 | 22 MB | 1.064 [0.962, 1.177] | 1.063 [0.962, 1.175] | 0.01 | yes |
| mibz2@base/ab | 10 | 0.163 | 0.167 | 0.165 | 0.169 | 23 MB | 1 | 1 | 0.01 | yes |

### random, 2 threads (8 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.092 | 0.096 | 0.172 | 0.175 | 35 MB | 0.978 [0.958, 0.998] | 0.982 [0.971, 0.992] | 0.06 | yes |
| mibz2@base/ab | 10 | 0.090 | 0.098 | 0.171 | 0.180 | 36 MB | 1 | 1 | 0.07 | yes |

### random, 4 threads (8 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.073 | 0.077 | 0.226 | 0.234 | 49 MB | 0.986 [0.941, 1.034] | 0.980 [0.964, 0.997] | 0.04 | yes |
| mibz2@base/ab | 10 | 0.074 | 0.077 | 0.230 | 0.239 | 53 MB | 1 | 1 | 0.06 | yes |

### records, 1 thread (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.199 | 0.205 | 0.200 | 0.207 | 23 MB | 1.026 [0.964, 1.093] | 1.026 [0.964, 1.092] | 0.01 | yes |
| mibz2@base/ab | 10 | 0.199 | 0.205 | 0.201 | 0.207 | 24 MB | 1 | 1 | 0.02 | yes |

### records, 2 threads (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.113 | 0.116 | 0.214 | 0.218 | 37 MB | 0.993 [0.974, 1.013] | 0.994 [0.984, 1.005] | 0.04 | yes |
| mibz2@base/ab | 10 | 0.113 | 0.117 | 0.216 | 0.221 | 39 MB | 1 | 1 | 0.04 | yes |

### records, 4 threads (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.089 | 0.093 | 0.281 | 0.294 | 52 MB | 0.992 [0.966, 1.019] | 0.974 [0.961, 0.986] | 0.01 | yes |
| mibz2@base/ab | 10 | 0.090 | 0.094 | 0.296 | 0.301 | 56 MB | 1 | 1 | 0.08 | yes |

### runs, 1 thread (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.014 | 0.015 | 0.016 | 0.016 | 23 MB | 0.992 [0.984, 1.001] | 0.991 [0.982, 1.000] | 0.23 | yes |
| mibz2@base/ab | 10 | 0.014 | 0.015 | 0.016 | 0.016 | 23 MB | 1 | 1 | 0.21 | yes |

### runs, 2 threads (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.015 | 0.015 | 0.017 | 0.017 | 23 MB | 1.015 [0.971, 1.062] | 0.997 [0.986, 1.008] | 0.12 | yes |
| mibz2@base/ab | 10 | 0.015 | 0.015 | 0.016 | 0.017 | 23 MB | 1 | 1 | 0.11 | yes |

### runs, 4 threads (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.015 | 0.015 | 0.016 | 0.017 | 23 MB | 1.003 [0.989, 1.018] | 1.004 [0.991, 1.017] | 0.15 | yes |
| mibz2@base/ab | 10 | 0.015 | 0.015 | 0.016 | 0.017 | 23 MB | 1 | 1 | 0.14 | yes |

### text, 1 thread (12 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.088 | 0.091 | 0.090 | 0.092 | 18 MB | 1.002 [0.992, 1.012] | 1.002 [0.992, 1.012] | 0.04 | yes |
| mibz2@base/ab | 10 | 0.089 | 0.090 | 0.090 | 0.092 | 19 MB | 1 | 1 | 0.00 | yes |

### text, 2 threads (12 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.052 | 0.053 | 0.101 | 0.102 | 31 MB | 1.000 [0.961, 1.040] | 0.999 [0.992, 1.006] | 0.11 | yes |
| mibz2@base/ab | 10 | 0.052 | 0.054 | 0.101 | 0.102 | 33 MB | 1 | 1 | 0.10 | yes |

### text, 4 threads (12 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.043 | 0.045 | 0.145 | 0.148 | 50 MB | 0.982 [0.959, 1.004] | 0.986 [0.968, 1.004] | 0.03 | yes |
| mibz2@base/ab | 10 | 0.045 | 0.046 | 0.145 | 0.151 | 54 MB | 1 | 1 | 0.07 | yes |

### wikidata, 1 thread (210 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.845 | 0.858 | 0.847 | 0.860 | 25 MB | 1.000 [0.994, 1.007] | 1.000 [0.994, 1.007] | 0.01 | yes |
| mibz2@base/ab | 10 | 0.835 | 0.858 | 0.836 | 0.860 | 26 MB | 1 | 1 | 0.01 | yes |

### wikidata, 2 threads (210 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.490 | 0.504 | 0.883 | 0.910 | 45 MB | 0.997 [0.982, 1.014] | 0.997 [0.989, 1.006] | 0.00 | yes |
| mibz2@base/ab | 10 | 0.483 | 0.507 | 0.882 | 0.915 | 48 MB | 1 | 1 | 0.00 | yes |

### wikidata, 4 threads (210 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.383 | 0.394 | 1.327 | 1.361 | 66 MB | 0.978 [0.960, 0.995] | 0.987 [0.971, 1.003] | 0.00 | yes |
| mibz2@base/ab | 10 | 0.392 | 0.402 | 1.326 | 1.385 | 71 MB | 1 | 1 | 0.00 | yes |
