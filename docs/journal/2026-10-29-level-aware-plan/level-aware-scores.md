## wikidata-mix93 (mix93: unchanged); 1 hash(es), 0 error(s)

| threads | mibz2 / mibz2-auto | rounds | peak RSS | CPU-s |
|--:|--:|--:|--:|--:|
| 4 | 1.004 (0.991-1.018) | 20 | 1.01 | 1.00 |
| 6 | 1.008 (0.998-1.017) | 20 | 1.00 | 1.00 |
| 8 | 0.996 (0.989-1.003) | 20 | 0.99 | 1.00 |
| 10 | 1.001 (0.995-1.007) | 20 | 0.98 | 1.00 |
| 20 | 0.997 (0.990-1.005) | 20 | 1.02 | 1.00 |

At 20 threads, the unchanged gate: **met**

## wikidata-mix39 (mix39: mixed); 1 hash(es), 0 error(s)

| threads | mibz2 / mibz2-auto | rounds | peak RSS | CPU-s |
|--:|--:|--:|--:|--:|
| 4 | 1.000 (0.986-1.015) | 20 | 1.00 | 0.99 |
| 6 | 0.996 (0.984-1.009) | 20 | 1.00 | 1.01 |
| 8 | 1.008 (0.995-1.021) | 20 | 1.00 | 1.00 |
| 10 | 0.992 (0.980-1.005) | 20 | 1.14 | 1.16 |
| 20 | 0.986 (0.974-0.997) | 20 | 1.14 | 1.18 |

At 20 threads, the mixed gate: **met**

## wikidata-bz6 (bz6: unchanged); 1 hash(es), 0 error(s)

| threads | mibz2 / mibz2-auto | rounds | peak RSS | CPU-s |
|--:|--:|--:|--:|--:|
| 4 | 0.990 (0.972-1.009) | 20 | 1.00 | 1.02 |
| 6 | 1.003 (0.988-1.017) | 20 | 0.99 | 1.00 |
| 8 | 0.987 (0.975-1.000) | 20 | 1.00 | 1.02 |
| 10 | 0.954 (0.881-1.032) | 20 | 1.01 | 1.01 |
| 20 | 0.983 (0.956-1.009) | 20 | 1.00 | 1.02 |

At 20 threads, the unchanged gate: **not met**

## wikidata-bz3 (bz3: rule); 1 hash(es), 0 error(s)

| threads | mibz2 / mibz2-auto | rounds | peak RSS | CPU-s |
|--:|--:|--:|--:|--:|
| 4 | 0.985 (0.963-1.008) | 20 | 1.00 | 1.01 |
| 6 | 0.993 (0.981-1.005) | 20 | 1.00 | 1.00 |
| 8 | 1.008 (0.991-1.025) | 20 | 1.01 | 1.00 |
| 10 | 1.106 (1.081-1.131) | 20 | 1.13 | 1.09 |
| 20 | 1.092 (1.065-1.119) | 20 | 1.13 | 1.11 |

At 20 threads, the rule gate: **met**

## wikidata-bz2 (bz2: rule); 1 hash(es), 0 error(s)

| threads | mibz2 / mibz2-auto | rounds | peak RSS | CPU-s |
|--:|--:|--:|--:|--:|
| 4 | 1.003 (0.993-1.014) | 20 | 1.00 | 0.99 |
| 6 | 1.011 (0.999-1.023) | 20 | 1.00 | 1.00 |
| 8 | 0.996 (0.982-1.010) | 20 | 1.00 | 1.00 |
| 10 | 1.200 (1.185-1.215) | 20 | 1.10 | 1.02 |
| 20 | 1.195 (1.180-1.210) | 20 | 1.11 | 1.03 |

At 20 threads, the rule gate: **met**

## wikidata (native: unchanged); 1 hash(es), 0 error(s)

| threads | mibz2 / mibz2-auto | rounds | peak RSS | CPU-s |
|--:|--:|--:|--:|--:|
| 4 | 1.005 (0.992-1.018) | 20 | 1.00 | 0.99 |
| 6 | 1.009 (0.989-1.031) | 20 | 1.00 | 1.00 |
| 8 | 1.016 (1.006-1.028) | 20 | 1.00 | 1.00 |
| 10 | 0.989 (0.974-1.004) | 20 | 1.00 | 1.01 |
| 20 | 0.995 (0.979-1.011) | 20 | 1.00 | 1.00 |

At 20 threads, the unchanged gate: **met**

## wikidata-bz1 (bz1: rule); 1 hash(es), 0 error(s)

| threads | mibz2 / mibz2-auto | rounds | peak RSS | CPU-s |
|--:|--:|--:|--:|--:|
| 4 | 1.001 (0.995-1.008) | 20 | 1.00 | 1.00 |
| 6 | 0.996 (0.991-1.000) | 20 | 1.00 | 1.00 |
| 8 | 0.998 (0.992-1.004) | 20 | 1.00 | 1.00 |
| 10 | 1.078 (1.063-1.093) | 20 | 1.04 | 1.01 |
| 20 | 1.092 (1.072-1.111) | 20 | 1.05 | 1.01 |

At 20 threads, the rule gate: **met**

## Verdict

- wikidata-mix93 (unchanged): met
- wikidata-mix39 (mixed): met
- wikidata-bz6 (unchanged): not met
- wikidata-bz3 (rule): met
- wikidata-bz2 (rule): met
- wikidata (unchanged): met
- wikidata-bz1 (rule): met

All met: **no**
