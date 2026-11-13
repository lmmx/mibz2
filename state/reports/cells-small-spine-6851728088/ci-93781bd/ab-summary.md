## ab summary (reference: mibz2@base/ab)

### onebyte, 1 thread (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.017 | 0.018 | 0.019 | 0.019 | 33 MB | 0.949 [0.825, 1.092] | 0.955 [0.838, 1.088] | 0.03 | yes |
| mibz2@base/ab | 10 | 0.017 | 0.018 | 0.019 | 0.020 | 33 MB | 1 | 1 | 0.06 | yes |

### onebyte, 2 threads (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.018 | 0.018 | 0.019 | 0.020 | 33 MB | 0.989 [0.957, 1.023] | 0.991 [0.961, 1.022] | 0.01 | yes |
| mibz2@base/ab | 10 | 0.018 | 0.018 | 0.020 | 0.020 | 33 MB | 1 | 1 | 0.35 | yes |

### onebyte, 4 threads (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.018 | 0.018 | 0.019 | 0.020 | 33 MB | 1.001 [0.975, 1.028] | 1.000 [0.978, 1.023] | 0.03 | yes |
| mibz2@base/ab | 10 | 0.018 | 0.018 | 0.019 | 0.020 | 33 MB | 1 | 1 | 0.01 | yes |

### pattern, 1 thread (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.026 | 0.026 | 0.027 | 0.028 | 16 MB | 0.997 [0.991, 1.003] | 0.995 [0.988, 1.003] | 0.07 | yes |
| mibz2@base/ab | 10 | 0.026 | 0.026 | 0.027 | 0.028 | 16 MB | 1 | 1 | 0.07 | yes |

### pattern, 2 threads (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.016 | 0.017 | 0.033 | 0.034 | 22 MB | 1.010 [0.929, 1.098] | 0.999 [0.971, 1.027] | 0.20 | yes |
| mibz2@base/ab | 10 | 0.016 | 0.018 | 0.033 | 0.034 | 22 MB | 1 | 1 | 0.17 | yes |

### pattern, 4 threads (20 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.019 | 0.020 | 0.060 | 0.067 | 41 MB | 1.028 [0.966, 1.094] | 0.991 [0.946, 1.038] | 0.15 | yes |
| mibz2@base/ab | 10 | 0.019 | 0.020 | 0.065 | 0.068 | 41 MB | 1 | 1 | 0.02 | yes |

### random, 1 thread (8 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.108 | 0.109 | 0.110 | 0.111 | 23 MB | 1.000 [0.992, 1.009] | 1.000 [0.992, 1.009] | 0.00 | yes |
| mibz2@base/ab | 10 | 0.108 | 0.109 | 0.110 | 0.111 | 23 MB | 1 | 1 | 0.08 | yes |

### random, 2 threads (8 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.066 | 0.067 | 0.123 | 0.126 | 39 MB | 0.980 [0.946, 1.015] | 0.993 [0.968, 1.020] | 0.06 | yes |
| mibz2@base/ab | 10 | 0.064 | 0.068 | 0.123 | 0.125 | 38 MB | 1 | 1 | 0.08 | yes |

### random, 4 threads (8 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.054 | 0.057 | 0.160 | 0.170 | 49 MB | 1.020 [0.990, 1.050] | 1.003 [0.986, 1.020] | 0.07 | yes |
| mibz2@base/ab | 10 | 0.053 | 0.056 | 0.165 | 0.168 | 50 MB | 1 | 1 | 0.06 | yes |

### records, 1 thread (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.117 | 0.118 | 0.119 | 0.120 | 23 MB | 1.002 [0.986, 1.018] | 1.002 [0.986, 1.018] | 0.00 | yes |
| mibz2@base/ab | 10 | 0.117 | 0.118 | 0.119 | 0.120 | 23 MB | 1 | 1 | 0.00 | yes |

### records, 2 threads (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.070 | 0.072 | 0.135 | 0.135 | 41 MB | 1.013 [0.926, 1.108] | 1.005 [0.922, 1.096] | 0.06 | yes |
| mibz2@base/ab | 10 | 0.071 | 0.073 | 0.134 | 0.138 | 43 MB | 1 | 1 | 0.14 | yes |

### records, 4 threads (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.060 | 0.063 | 0.183 | 0.185 | 52 MB | 0.974 [0.928, 1.021] | 0.959 [0.936, 0.982] | 0.08 | yes |
| mibz2@base/ab | 10 | 0.062 | 0.063 | 0.184 | 0.193 | 54 MB | 1 | 1 | 0.10 | yes |

### runs, 1 thread (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.011 | 0.012 | 0.013 | 0.013 | 23 MB | 0.993 [0.970, 1.018] | 0.993 [0.973, 1.014] | 0.00 | yes |
| mibz2@base/ab | 10 | 0.011 | 0.012 | 0.013 | 0.013 | 23 MB | 1 | 1 | 0.00 | yes |

### runs, 2 threads (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.012 | 0.012 | 0.014 | 0.014 | 23 MB | 0.981 [0.964, 0.999] | 0.983 [0.970, 0.997] | 0.11 | yes |
| mibz2@base/ab | 10 | 0.012 | 0.012 | 0.014 | 0.014 | 23 MB | 1 | 1 | 0.22 | yes |

### runs, 4 threads (10 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.012 | 0.012 | 0.014 | 0.014 | 23 MB | 1.017 [0.996, 1.038] | 1.015 [0.997, 1.033] | 0.00 | yes |
| mibz2@base/ab | 10 | 0.012 | 0.012 | 0.013 | 0.014 | 23 MB | 1 | 1 | 0.23 | yes |

### text, 1 thread (12 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.052 | 0.053 | 0.054 | 0.055 | 19 MB | 1.003 [0.995, 1.012] | 1.003 [0.995, 1.011] | 0.10 | yes |
| mibz2@base/ab | 10 | 0.052 | 0.053 | 0.054 | 0.055 | 19 MB | 1 | 1 | 0.09 | yes |

### text, 2 threads (12 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.035 | 0.036 | 0.067 | 0.068 | 35 MB | 0.976 [0.943, 1.011] | 0.984 [0.959, 1.010] | 0.06 | yes |
| mibz2@base/ab | 10 | 0.035 | 0.037 | 0.067 | 0.068 | 36 MB | 1 | 1 | 0.06 | yes |

### text, 4 threads (12 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.032 | 0.033 | 0.098 | 0.105 | 55 MB | 0.997 [0.973, 1.022] | 1.003 [0.967, 1.041] | 0.09 | yes |
| mibz2@base/ab | 10 | 0.032 | 0.033 | 0.097 | 0.104 | 54 MB | 1 | 1 | 0.09 | yes |

### wikidata, 1 thread (210 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.426 | 0.430 | 0.427 | 0.432 | 26 MB | 0.998 [0.993, 1.004] | 0.998 [0.993, 1.004] | 0.02 | yes |
| mibz2@base/ab | 10 | 0.428 | 0.431 | 0.430 | 0.433 | 26 MB | 1 | 1 | 0.01 | yes |

### wikidata, 2 threads (210 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.235 | 0.241 | 0.467 | 0.469 | 53 MB | 0.996 [0.978, 1.013] | 0.997 [0.992, 1.003] | 0.04 | yes |
| mibz2@base/ab | 10 | 0.237 | 0.242 | 0.467 | 0.472 | 52 MB | 1 | 1 | 0.06 | yes |

### wikidata, 4 threads (210 MB out; ratios: arm / mibz2@base/ab, paired by round)

| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| mibz2 | 10 | 0.214 | 0.220 | 0.772 | 0.804 | 81 MB | 0.993 [0.980, 1.005] | 0.994 [0.980, 1.008] | 0.00 | yes |
| mibz2@base/ab | 10 | 0.216 | 0.221 | 0.759 | 0.810 | 81 MB | 1 | 1 | 0.04 | yes |
