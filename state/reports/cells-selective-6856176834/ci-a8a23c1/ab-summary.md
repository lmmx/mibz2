## ab summary (reference: mibz2@base/ab)

### onebyte, 1 thread (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.017 | 0.018 | 0.019 | 0.019 | 33 MB | 1.000 [0.973, 1.027] | 0.998 [0.974, 1.024] | 0.03 | yes |
| mibz2@base/ab | 10 | 0.017 | 0.018 | 0.019 | 0.019 | 33 MB | 1 | 1 | 0.03 | yes |

### onebyte, 2 threads (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.017 | 0.018 | 0.019 | 0.020 | 33 MB | 1.013 [0.976, 1.052] | 1.007 [0.974, 1.042] | 0.03 | yes |
| mibz2@base/ab | 10 | 0.018 | 0.018 | 0.019 | 0.020 | 33 MB | 1 | 1 | 0.01 | yes |

### onebyte, 4 threads (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.017 | 0.018 | 0.019 | 0.019 | 33 MB | 1.000 [0.982, 1.018] | 0.999 [0.981, 1.018] | 0.02 | yes |
| mibz2@base/ab | 10 | 0.018 | 0.018 | 0.019 | 0.020 | 33 MB | 1 | 1 | 0.03 | yes |

### pattern, 1 thread (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.026 | 0.026 | 0.027 | 0.028 | 16 MB | 1.007 [1.000, 1.013] | 1.006 [0.999, 1.013] | 0.08 | yes |
| mibz2@base/ab | 10 | 0.026 | 0.026 | 0.027 | 0.028 | 16 MB | 1 | 1 | 0.08 | yes |

### pattern, 2 threads (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.016 | 0.018 | 0.033 | 0.034 | 23 MB | 1.091 [0.937, 1.269] | 1.013 [0.997, 1.030] | 0.00 | yes |
| mibz2@base/ab | 10 | 0.015 | 0.016 | 0.032 | 0.033 | 22 MB | 1 | 1 | 0.00 | yes |

### pattern, 4 threads (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.018 | 0.020 | 0.064 | 0.067 | 39 MB | 0.953 [0.911, 0.997] | 0.988 [0.945, 1.032] | 0.03 | yes |
| mibz2@base/ab | 10 | 0.020 | 0.020 | 0.064 | 0.068 | 40 MB | 1 | 1 | 0.20 | yes |

### random, 1 thread (8 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.108 | 0.109 | 0.110 | 0.111 | 23 MB | 0.985 [0.957, 1.014] | 0.986 [0.958, 1.014] | 0.00 | yes |
| mibz2@base/ab | 10 | 0.109 | 0.109 | 0.110 | 0.111 | 23 MB | 1 | 1 | 0.00 | yes |

### random, 2 threads (8 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.064 | 0.067 | 0.122 | 0.124 | 39 MB | 0.989 [0.969, 1.009] | 0.992 [0.968, 1.016] | 0.03 | yes |
| mibz2@base/ab | 10 | 0.065 | 0.068 | 0.122 | 0.125 | 40 MB | 1 | 1 | 0.06 | yes |

### random, 4 threads (8 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.054 | 0.057 | 0.162 | 0.165 | 50 MB | 1.028 [1.005, 1.052] | 0.995 [0.969, 1.020] | 0.08 | yes |
| mibz2@base/ab | 10 | 0.054 | 0.055 | 0.162 | 0.166 | 50 MB | 1 | 1 | 0.07 | yes |

### records, 1 thread (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.117 | 0.118 | 0.119 | 0.119 | 23 MB | 0.980 [0.945, 1.016] | 0.980 [0.946, 1.015] | 0.01 | yes |
| mibz2@base/ab | 10 | 0.117 | 0.118 | 0.119 | 0.120 | 23 MB | 1 | 1 | 0.01 | yes |

### records, 2 threads (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.070 | 0.072 | 0.134 | 0.136 | 42 MB | 0.964 [0.900, 1.033] | 0.975 [0.912, 1.043] | 0.07 | yes |
| mibz2@base/ab | 10 | 0.070 | 0.073 | 0.134 | 0.136 | 42 MB | 1 | 1 | 0.07 | yes |

### records, 4 threads (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.060 | 0.061 | 0.184 | 0.191 | 53 MB | 0.993 [0.968, 1.017] | 1.003 [0.980, 1.027] | 0.11 | yes |
| mibz2@base/ab | 10 | 0.059 | 0.062 | 0.182 | 0.189 | 53 MB | 1 | 1 | 0.05 | yes |

### runs, 1 thread (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.011 | 0.012 | 0.013 | 0.013 | 23 MB | 1.001 [0.973, 1.029] | 1.002 [0.975, 1.029] | 0.00 | yes |
| mibz2@base/ab | 10 | 0.011 | 0.012 | 0.013 | 0.013 | 23 MB | 1 | 1 | 0.00 | yes |

### runs, 2 threads (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.012 | 0.012 | 0.014 | 0.014 | 23 MB | 0.999 [0.979, 1.020] | 0.994 [0.975, 1.014] | 0.22 | yes |
| mibz2@base/ab | 10 | 0.012 | 0.012 | 0.014 | 0.014 | 23 MB | 1 | 1 | 0.00 | yes |

### runs, 4 threads (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.012 | 0.012 | 0.013 | 0.014 | 23 MB | 0.989 [0.972, 1.005] | 0.988 [0.974, 1.003] | 0.25 | yes |
| mibz2@base/ab | 10 | 0.012 | 0.012 | 0.013 | 0.014 | 23 MB | 1 | 1 | 0.00 | yes |

### text, 1 thread (12 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.053 | 0.053 | 0.054 | 0.055 | 19 MB | 1.002 [0.997, 1.007] | 1.002 [0.997, 1.006] | 0.09 | yes |
| mibz2@base/ab | 10 | 0.053 | 0.053 | 0.054 | 0.055 | 19 MB | 1 | 1 | 0.00 | yes |

### text, 2 threads (12 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.034 | 0.036 | 0.066 | 0.068 | 36 MB | 1.012 [0.959, 1.069] | 0.994 [0.957, 1.033] | 0.08 | yes |
| mibz2@base/ab | 10 | 0.034 | 0.036 | 0.065 | 0.068 | 36 MB | 1 | 1 | 0.05 | yes |

### text, 4 threads (12 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.033 | 0.033 | 0.102 | 0.106 | 55 MB | 1.039 [1.007, 1.072] | 0.996 [0.978, 1.014] | 0.13 | yes |
| mibz2@base/ab | 10 | 0.031 | 0.033 | 0.104 | 0.106 | 53 MB | 1 | 1 | 0.10 | yes |

### wikidata, 1 thread (210 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.430 | 0.432 | 0.432 | 0.434 | 26 MB | 1.024 [0.998, 1.050] | 1.024 [0.998, 1.050] | 0.03 | yes |
| mibz2@base/ab | 10 | 0.429 | 0.430 | 0.430 | 0.432 | 26 MB | 1 | 1 | 0.02 | yes |

### wikidata, 2 threads (210 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.236 | 0.240 | 0.465 | 0.469 | 52 MB | 1.000 [0.960, 1.042] | 1.000 [0.961, 1.041] | 0.04 | yes |
| mibz2@base/ab | 10 | 0.237 | 0.241 | 0.466 | 0.468 | 52 MB | 1 | 1 | 0.05 | yes |

### wikidata, 4 threads (210 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.214 | 0.221 | 0.724 | 0.781 | 81 MB | 1.023 [1.004, 1.043] | 1.000 [0.977, 1.023] | 0.04 | yes |
| mibz2@base/ab | 10 | 0.210 | 0.215 | 0.744 | 0.778 | 80 MB | 1 | 1 | 0.03 | yes |
