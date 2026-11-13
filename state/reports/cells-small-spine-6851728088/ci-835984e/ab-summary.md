## ab summary (reference: mibz2@base/ab)

### onebyte, 1 thread (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.015 | 0.015 | 0.016 | 0.017 | 33 MB | 0.885 [0.732, 1.069] | 0.890 [0.737, 1.076] | 0.15 | yes |
| mibz2@base/ab | 10 | 0.015 | 0.015 | 0.017 | 0.017 | 33 MB | 1 | 1 | 0.13 | yes |

### onebyte, 2 threads (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.015 | 0.015 | 0.017 | 0.017 | 33 MB | 1.016 [0.989, 1.042] | 1.014 [0.988, 1.040] | 0.11 | yes |
| mibz2@base/ab | 10 | 0.014 | 0.015 | 0.016 | 0.017 | 33 MB | 1 | 1 | 0.11 | yes |

### onebyte, 4 threads (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.014 | 0.015 | 0.016 | 0.017 | 33 MB | 0.942 [0.841, 1.055] | 0.948 [0.853, 1.054] | 0.04 | yes |
| mibz2@base/ab | 10 | 0.015 | 0.016 | 0.017 | 0.017 | 33 MB | 1 | 1 | 0.10 | yes |

### pattern, 1 thread (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.019 | 0.019 | 0.020 | 0.021 | 16 MB | 0.993 [0.967, 1.018] | 0.995 [0.969, 1.021] | 0.15 | yes |
| mibz2@base/ab | 10 | 0.019 | 0.019 | 0.020 | 0.021 | 16 MB | 1 | 1 | 0.30 | yes |

### pattern, 2 threads (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.013 | 0.017 | 0.027 | 0.031 | 22 MB | 1.063 [0.959, 1.179] | 1.035 [0.972, 1.101] | 0.06 | yes |
| mibz2@base/ab | 10 | 0.013 | 0.016 | 0.027 | 0.030 | 23 MB | 1 | 1 | 0.02 | yes |

### pattern, 4 threads (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.016 | 0.018 | 0.052 | 0.061 | 41 MB | 1.012 [0.925, 1.106] | 1.023 [0.941, 1.112] | 0.29 | yes |
| mibz2@base/ab | 10 | 0.017 | 0.018 | 0.049 | 0.058 | 42 MB | 1 | 1 | 0.03 | yes |

### random, 1 thread (8 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.085 | 0.089 | 0.086 | 0.091 | 23 MB | 0.972 [0.900, 1.049] | 0.971 [0.900, 1.048] | 0.04 | yes |
| mibz2@base/ab | 10 | 0.084 | 0.089 | 0.086 | 0.091 | 23 MB | 1 | 1 | 0.00 | yes |

### random, 2 threads (8 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.053 | 0.056 | 0.098 | 0.105 | 39 MB | 1.009 [0.980, 1.038] | 1.011 [0.990, 1.033] | 0.12 | yes |
| mibz2@base/ab | 10 | 0.053 | 0.056 | 0.099 | 0.104 | 38 MB | 1 | 1 | 0.07 | yes |

### random, 4 threads (8 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.043 | 0.047 | 0.136 | 0.139 | 50 MB | 0.992 [0.943, 1.044] | 1.005 [0.971, 1.041] | 0.06 | yes |
| mibz2@base/ab | 10 | 0.044 | 0.047 | 0.128 | 0.138 | 50 MB | 1 | 1 | 0.10 | yes |

### records, 1 thread (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.091 | 0.094 | 0.093 | 0.096 | 23 MB | 1.013 [0.988, 1.038] | 1.013 [0.989, 1.038] | 0.04 | yes |
| mibz2@base/ab | 10 | 0.088 | 0.092 | 0.090 | 0.093 | 23 MB | 1 | 1 | 0.00 | yes |

### records, 2 threads (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.056 | 0.059 | 0.107 | 0.110 | 41 MB | 0.996 [0.955, 1.039] | 0.993 [0.960, 1.026] | 0.16 | yes |
| mibz2@base/ab | 10 | 0.057 | 0.059 | 0.107 | 0.112 | 42 MB | 1 | 1 | 0.08 | yes |

### records, 4 threads (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.049 | 0.051 | 0.146 | 0.155 | 55 MB | 0.967 [0.933, 1.002] | 0.990 [0.963, 1.018] | 0.17 | yes |
| mibz2@base/ab | 10 | 0.049 | 0.053 | 0.143 | 0.156 | 54 MB | 1 | 1 | 0.05 | yes |

### runs, 1 thread (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.010 | 0.010 | 0.011 | 0.012 | 23 MB | 0.996 [0.962, 1.032] | 0.995 [0.964, 1.028] | 0.00 | yes |
| mibz2@base/ab | 10 | 0.010 | 0.010 | 0.011 | 0.012 | 23 MB | 1 | 1 | 0.00 | yes |

### runs, 2 threads (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.010 | 0.010 | 0.012 | 0.012 | 23 MB | 1.002 [0.976, 1.029] | 1.007 [0.986, 1.028] | 0.00 | yes |
| mibz2@base/ab | 10 | 0.010 | 0.010 | 0.012 | 0.012 | 23 MB | 1 | 1 | 0.15 | yes |

### runs, 4 threads (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.010 | 0.011 | 0.011 | 0.012 | 23 MB | 0.999 [0.977, 1.022] | 0.998 [0.979, 1.017] | 0.14 | yes |
| mibz2@base/ab | 10 | 0.010 | 0.011 | 0.012 | 0.012 | 23 MB | 1 | 1 | 0.00 | yes |

### text, 1 thread (12 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.041 | 0.044 | 0.043 | 0.046 | 19 MB | 1.006 [0.979, 1.035] | 1.005 [0.978, 1.033] | 0.06 | yes |
| mibz2@base/ab | 10 | 0.042 | 0.044 | 0.044 | 0.045 | 19 MB | 1 | 1 | 0.08 | yes |

### text, 2 threads (12 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.030 | 0.033 | 0.058 | 0.062 | 36 MB | 1.017 [0.958, 1.080] | 1.011 [0.968, 1.055] | 0.04 | yes |
| mibz2@base/ab | 10 | 0.030 | 0.033 | 0.057 | 0.062 | 36 MB | 1 | 1 | 0.13 | yes |

### text, 4 threads (12 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.027 | 0.029 | 0.087 | 0.092 | 54 MB | 0.977 [0.949, 1.005] | 0.988 [0.958, 1.018] | 0.16 | yes |
| mibz2@base/ab | 10 | 0.028 | 0.030 | 0.087 | 0.094 | 56 MB | 1 | 1 | 0.17 | yes |

### wikidata, 1 thread (210 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.309 | 0.321 | 0.311 | 0.322 | 26 MB | 0.998 [0.954, 1.043] | 0.998 [0.954, 1.043] | 0.03 | yes |
| mibz2@base/ab | 10 | 0.313 | 0.320 | 0.314 | 0.322 | 26 MB | 1 | 1 | 0.04 | yes |

### wikidata, 2 threads (210 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.185 | 0.197 | 0.365 | 0.384 | 52 MB | 0.980 [0.910, 1.055] | 0.983 [0.922, 1.049] | 0.08 | yes |
| mibz2@base/ab | 10 | 0.189 | 0.195 | 0.369 | 0.379 | 52 MB | 1 | 1 | 0.05 | yes |

### wikidata, 4 threads (210 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.171 | 0.195 | 0.624 | 0.686 | 81 MB | 0.990 [0.955, 1.026] | 0.988 [0.962, 1.014] | 0.03 | yes |
| mibz2@base/ab | 10 | 0.185 | 0.194 | 0.649 | 0.675 | 81 MB | 1 | 1 | 0.03 | yes |
