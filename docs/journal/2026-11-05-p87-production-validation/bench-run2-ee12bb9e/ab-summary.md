## ab summary (reference: mibz2@base/ab)

### onebyte, 1 thread (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.012 | 0.016 | 0.013 | 0.017 | 33 MB | 0.988 [0.917, 1.065] | 0.992 [0.923, 1.067] | 0.06 | yes |
| mibz2@base/ab | 10 | 0.014 | 0.016 | 0.015 | 0.018 | 33 MB | 1 | 1 | 0.14 | yes |

### onebyte, 2 threads (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.013 | 0.016 | 0.015 | 0.018 | 32 MB | 1.007 [0.929, 1.092] | 1.011 [0.941, 1.087] | 0.08 | yes |
| mibz2@base/ab | 10 | 0.012 | 0.016 | 0.014 | 0.018 | 33 MB | 1 | 1 | 0.10 | yes |

### onebyte, 4 threads (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.015 | 0.016 | 0.016 | 0.018 | 32 MB | 1.007 [0.957, 1.060] | 1.007 [0.959, 1.058] | 0.11 | yes |
| mibz2@base/ab | 10 | 0.014 | 0.016 | 0.015 | 0.018 | 33 MB | 1 | 1 | 0.11 | yes |

### pattern, 1 thread (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.032 | 0.034 | 0.033 | 0.035 | 16 MB | 0.998 [0.971, 1.026] | 0.995 [0.967, 1.023] | 0.05 | yes |
| mibz2@base/ab | 10 | 0.032 | 0.034 | 0.034 | 0.035 | 17 MB | 1 | 1 | 0.12 | yes |

### pattern, 2 threads (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.019 | 0.019 | 0.037 | 0.039 | 22 MB | 0.966 [0.863, 1.080] | 0.989 [0.967, 1.012] | 0.01 | yes |
| mibz2@base/ab | 10 | 0.018 | 0.021 | 0.037 | 0.040 | 23 MB | 1 | 1 | 0.03 | yes |

### pattern, 4 threads (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.018 | 0.020 | 0.063 | 0.073 | 39 MB | 0.958 [0.897, 1.024] | 1.027 [0.957, 1.102] | 0.00 | yes |
| mibz2@base/ab | 10 | 0.020 | 0.021 | 0.059 | 0.074 | 42 MB | 1 | 1 | 0.08 | yes |

### random, 1 thread (8 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.115 | 0.118 | 0.116 | 0.120 | 22 MB | 1.008 [0.983, 1.034] | 1.008 [0.982, 1.034] | 0.01 | yes |
| mibz2@base/ab | 10 | 0.115 | 0.118 | 0.116 | 0.119 | 23 MB | 1 | 1 | 0.01 | yes |

### random, 2 threads (8 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.067 | 0.069 | 0.124 | 0.126 | 35 MB | 1.013 [0.980, 1.048] | 1.000 [0.988, 1.012] | 0.04 | yes |
| mibz2@base/ab | 10 | 0.066 | 0.068 | 0.122 | 0.126 | 37 MB | 1 | 1 | 0.01 | yes |

### random, 4 threads (8 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.053 | 0.055 | 0.156 | 0.162 | 49 MB | 0.992 [0.960, 1.025] | 0.999 [0.968, 1.031] | 0.04 | yes |
| mibz2@base/ab | 10 | 0.051 | 0.055 | 0.148 | 0.165 | 53 MB | 1 | 1 | 0.07 | yes |

### records, 1 thread (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.145 | 0.147 | 0.146 | 0.149 | 23 MB | 0.973 [0.910, 1.041] | 0.974 [0.911, 1.040] | 0.00 | yes |
| mibz2@base/ab | 10 | 0.144 | 0.148 | 0.146 | 0.149 | 24 MB | 1 | 1 | 0.01 | yes |

### records, 2 threads (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.080 | 0.084 | 0.153 | 0.157 | 37 MB | 0.992 [0.965, 1.020] | 0.999 [0.990, 1.007] | 0.03 | yes |
| mibz2@base/ab | 10 | 0.081 | 0.084 | 0.153 | 0.157 | 39 MB | 1 | 1 | 0.01 | yes |

### records, 4 threads (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.063 | 0.065 | 0.192 | 0.202 | 51 MB | 0.989 [0.960, 1.020] | 0.980 [0.967, 0.993] | 0.04 | yes |
| mibz2@base/ab | 10 | 0.062 | 0.065 | 0.198 | 0.206 | 56 MB | 1 | 1 | 0.03 | yes |

### runs, 1 thread (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.009 | 0.010 | 0.010 | 0.012 | 23 MB | 0.986 [0.930, 1.046] | 0.985 [0.928, 1.044] | 0.28 | yes |
| mibz2@base/ab | 10 | 0.009 | 0.011 | 0.010 | 0.012 | 23 MB | 1 | 1 | 0.00 | yes |

### runs, 2 threads (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.009 | 0.011 | 0.011 | 0.012 | 23 MB | 1.012 [0.962, 1.064] | 1.002 [0.954, 1.053] | 0.00 | yes |
| mibz2@base/ab | 10 | 0.009 | 0.011 | 0.010 | 0.012 | 23 MB | 1 | 1 | 0.00 | yes |

### runs, 4 threads (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.011 | 0.011 | 0.012 | 0.013 | 23 MB | 1.008 [0.982, 1.034] | 1.009 [0.979, 1.040] | 0.00 | yes |
| mibz2@base/ab | 10 | 0.010 | 0.011 | 0.011 | 0.013 | 23 MB | 1 | 1 | 0.00 | yes |

### text, 1 thread (12 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.067 | 0.069 | 0.069 | 0.070 | 18 MB | 1.001 [0.991, 1.012] | 1.001 [0.991, 1.013] | 0.07 | yes |
| mibz2@base/ab | 10 | 0.067 | 0.069 | 0.068 | 0.070 | 19 MB | 1 | 1 | 0.00 | yes |

### text, 2 threads (12 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.038 | 0.039 | 0.074 | 0.076 | 30 MB | 0.992 [0.964, 1.020] | 0.992 [0.978, 1.007] | 0.10 | yes |
| mibz2@base/ab | 10 | 0.037 | 0.040 | 0.073 | 0.077 | 32 MB | 1 | 1 | 0.05 | yes |

### text, 4 threads (12 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.030 | 0.032 | 0.102 | 0.111 | 48 MB | 0.977 [0.942, 1.014] | 1.000 [0.972, 1.029] | 0.00 | yes |
| mibz2@base/ab | 10 | 0.031 | 0.032 | 0.106 | 0.109 | 52 MB | 1 | 1 | 0.04 | yes |

### wikidata, 1 thread (210 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.651 | 0.655 | 0.653 | 0.656 | 25 MB | 1.000 [0.998, 1.002] | 1.001 [0.998, 1.003] | 0.01 | yes |
| mibz2@base/ab | 10 | 0.651 | 0.654 | 0.652 | 0.656 | 26 MB | 1 | 1 | 0.01 | yes |

### wikidata, 2 threads (210 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.346 | 0.376 | 0.665 | 0.669 | 45 MB | 0.989 [0.951, 1.029] | 0.998 [0.986, 1.010] | 0.00 | yes |
| mibz2@base/ab | 10 | 0.360 | 0.384 | 0.663 | 0.669 | 49 MB | 1 | 1 | 0.00 | yes |

### wikidata, 4 threads (210 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.296 | 0.303 | 0.967 | 0.986 | 66 MB | 0.989 [0.977, 1.002] | 0.993 [0.981, 1.005] | 0.00 | yes |
| mibz2@base/ab | 10 | 0.302 | 0.304 | 0.965 | 0.994 | 71 MB | 1 | 1 | 0.00 | yes |
