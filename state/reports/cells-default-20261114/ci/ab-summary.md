## ab summary (reference: mibz2@base/ab)

### onebyte, 1 thread (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.021 | 0.022 | 0.024 | 0.024 | 36 MB | 1.080 [1.072, 1.089] | 1.073 [1.062, 1.085] | 0.00 | yes |
| mibz2@base/ab | 10 | 0.020 | 0.020 | 0.022 | 0.022 | 33 MB | 1 | 1 | 0.29 | yes |

### onebyte, 2 threads (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.020 | 0.021 | 0.023 | 0.023 | 33 MB | 1.002 [0.984, 1.021] | 1.001 [0.983, 1.018] | 0.18 | yes |
| mibz2@base/ab | 10 | 0.020 | 0.021 | 0.022 | 0.023 | 33 MB | 1 | 1 | 0.00 | yes |

### onebyte, 4 threads (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.021 | 0.021 | 0.023 | 0.023 | 33 MB | 1.015 [0.998, 1.033] | 1.015 [0.999, 1.030] | 0.17 | yes |
| mibz2@base/ab | 10 | 0.020 | 0.021 | 0.022 | 0.023 | 33 MB | 1 | 1 | 0.20 | yes |

### pattern, 1 thread (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.053 | 0.054 | 0.055 | 0.056 | 18 MB | 1.596 [1.579, 1.613] | 1.560 [1.544, 1.576] | 0.08 | yes |
| mibz2@base/ab | 10 | 0.033 | 0.033 | 0.035 | 0.035 | 16 MB | 1 | 1 | 0.11 | yes |

### pattern, 2 threads (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.021 | 0.022 | 0.043 | 0.044 | 23 MB | 1.006 [0.966, 1.049] | 1.024 [1.007, 1.042] | 0.16 | yes |
| mibz2@base/ab | 10 | 0.021 | 0.022 | 0.042 | 0.043 | 22 MB | 1 | 1 | 0.00 | yes |

### pattern, 4 threads (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.024 | 0.025 | 0.081 | 0.087 | 40 MB | 0.988 [0.947, 1.030] | 0.991 [0.936, 1.050] | 0.07 | yes |
| mibz2@base/ab | 10 | 0.024 | 0.025 | 0.077 | 0.087 | 39 MB | 1 | 1 | 0.07 | yes |

### random, 1 thread (8 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.132 | 0.134 | 0.134 | 0.136 | 23 MB | 1.001 [0.996, 1.006] | 1.002 [0.997, 1.007] | 0.03 | yes |
| mibz2@base/ab | 10 | 0.132 | 0.134 | 0.134 | 0.136 | 23 MB | 1 | 1 | 0.04 | yes |

### random, 2 threads (8 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.082 | 0.085 | 0.151 | 0.154 | 38 MB | 0.998 [0.972, 1.024] | 0.990 [0.967, 1.014] | 0.08 | yes |
| mibz2@base/ab | 10 | 0.080 | 0.085 | 0.150 | 0.156 | 38 MB | 1 | 1 | 0.07 | yes |

### random, 4 threads (8 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.067 | 0.070 | 0.202 | 0.206 | 49 MB | 1.003 [0.965, 1.042] | 1.002 [0.974, 1.030] | 0.07 | yes |
| mibz2@base/ab | 10 | 0.068 | 0.070 | 0.201 | 0.209 | 49 MB | 1 | 1 | 0.04 | yes |

### records, 1 thread (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.145 | 0.147 | 0.147 | 0.149 | 23 MB | 0.999 [0.987, 1.011] | 0.999 [0.987, 1.011] | 0.02 | yes |
| mibz2@base/ab | 10 | 0.145 | 0.146 | 0.147 | 0.148 | 23 MB | 1 | 1 | 0.03 | yes |

### records, 2 threads (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.087 | 0.089 | 0.168 | 0.170 | 42 MB | 1.008 [0.985, 1.032] | 1.005 [0.992, 1.018] | 0.11 | yes |
| mibz2@base/ab | 10 | 0.086 | 0.089 | 0.166 | 0.169 | 42 MB | 1 | 1 | 0.07 | yes |

### records, 4 threads (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.075 | 0.079 | 0.225 | 0.243 | 53 MB | 1.009 [0.996, 1.022] | 1.005 [0.966, 1.045] | 0.10 | yes |
| mibz2@base/ab | 10 | 0.074 | 0.078 | 0.227 | 0.241 | 54 MB | 1 | 1 | 0.11 | yes |

### runs, 1 thread (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.013 | 0.013 | 0.015 | 0.015 | 23 MB | 1.001 [0.990, 1.011] | 1.003 [0.990, 1.016] | 0.00 | yes |
| mibz2@base/ab | 10 | 0.013 | 0.013 | 0.015 | 0.015 | 23 MB | 1 | 1 | 0.29 | yes |

### runs, 2 threads (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.014 | 0.014 | 0.016 | 0.017 | 23 MB | 1.010 [1.000, 1.020] | 1.004 [0.992, 1.015] | 0.12 | yes |
| mibz2@base/ab | 10 | 0.014 | 0.014 | 0.016 | 0.017 | 23 MB | 1 | 1 | 0.14 | yes |

### runs, 4 threads (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.014 | 0.014 | 0.016 | 0.016 | 23 MB | 1.001 [0.986, 1.016] | 1.005 [0.988, 1.023] | 0.13 | yes |
| mibz2@base/ab | 10 | 0.014 | 0.014 | 0.016 | 0.016 | 23 MB | 1 | 1 | 0.13 | yes |

### text, 1 thread (12 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.065 | 0.065 | 0.066 | 0.067 | 19 MB | 0.990 [0.967, 1.013] | 0.990 [0.967, 1.014] | 0.05 | yes |
| mibz2@base/ab | 10 | 0.065 | 0.065 | 0.067 | 0.067 | 19 MB | 1 | 1 | 0.01 | yes |

### text, 2 threads (12 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.045 | 0.047 | 0.084 | 0.086 | 36 MB | 0.999 [0.977, 1.021] | 0.991 [0.975, 1.008] | 0.08 | yes |
| mibz2@base/ab | 10 | 0.045 | 0.047 | 0.084 | 0.086 | 35 MB | 1 | 1 | 0.08 | yes |

### text, 4 threads (12 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.041 | 0.041 | 0.128 | 0.133 | 55 MB | 0.961 [0.930, 0.994] | 1.015 [0.981, 1.050] | 0.11 | yes |
| mibz2@base/ab | 10 | 0.040 | 0.043 | 0.121 | 0.132 | 54 MB | 1 | 1 | 0.01 | yes |

### wikidata, 1 thread (210 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.524 | 0.528 | 0.526 | 0.530 | 34 MB | 0.992 [0.969, 1.017] | 0.992 [0.969, 1.017] | 0.03 | yes |
| mibz2@base/ab | 10 | 0.520 | 0.524 | 0.522 | 0.526 | 26 MB | 1 | 1 | 0.03 | yes |

### wikidata, 2 threads (210 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.287 | 0.294 | 0.567 | 0.578 | 52 MB | 1.011 [0.989, 1.034] | 1.009 [0.990, 1.027] | 0.04 | yes |
| mibz2@base/ab | 10 | 0.285 | 0.293 | 0.564 | 0.577 | 52 MB | 1 | 1 | 0.06 | yes |

### wikidata, 4 threads (210 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.257 | 0.264 | 0.932 | 0.951 | 81 MB | 1.008 [0.988, 1.029] | 0.991 [0.978, 1.004] | 0.06 | yes |
| mibz2@base/ab | 10 | 0.256 | 0.263 | 0.912 | 0.954 | 81 MB | 1 | 1 | 0.04 | yes |
