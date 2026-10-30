## wikidata-bz6 (bz6: unchanged); 1 hash(es), 0 error(s)

| threads | mibz2 / mibz2-auto | rounds | peak RSS | CPU-s |
|--:|--:|--:|--:|--:|
| 4 | 0.989 (0.968-1.010) | 20 | 1.01 | 1.02 |
| 6 | 1.030 (0.979-1.084) | 20 | 1.00 | 0.99 |
| 8 | 1.019 (1.006-1.033) | 20 | 1.00 | 0.97 |
| 10 | 0.992 (0.971-1.013) | 20 | 1.00 | 1.00 |
| 20 | 0.985 (0.971-1.000) | 20 | 1.01 | 1.02 |

At 20 threads, the unchanged gate: **met**

## wikidata-mix93 (mix93: unchanged); 1 hash(es), 0 error(s)

| threads | mibz2 / mibz2-auto | rounds | peak RSS | CPU-s |
|--:|--:|--:|--:|--:|
| 4 | 0.996 (0.983-1.009) | 20 | 0.99 | 1.01 |
| 6 | 0.999 (0.990-1.009) | 20 | 1.01 | 1.00 |
| 8 | 0.998 (0.987-1.010) | 20 | 1.00 | 0.99 |
| 10 | 0.996 (0.990-1.003) | 20 | 1.01 | 1.00 |
| 20 | 0.999 (0.991-1.007) | 20 | 1.00 | 1.00 |

At 20 threads, the unchanged gate: **met**

## wikidata-bz3 (bz3: rule); 1 hash(es), 0 error(s)

| threads | mibz2 / mibz2-auto | rounds | peak RSS | CPU-s |
|--:|--:|--:|--:|--:|
| 4 | 1.004 (0.988-1.021) | 20 | 1.00 | 0.99 |
| 6 | 1.000 (0.988-1.012) | 20 | 1.00 | 1.00 |
| 8 | 1.001 (0.985-1.017) | 20 | 1.00 | 1.01 |
| 10 | 1.090 (1.049-1.134) | 20 | 1.13 | 1.08 |
| 20 | 1.078 (1.060-1.097) | 20 | 1.13 | 1.11 |

At 20 threads, the rule gate: **met**

## wikidata-mix39 (mix39: mixed); 1 hash(es), 0 error(s)

| threads | mibz2 / mibz2-auto | rounds | peak RSS | CPU-s |
|--:|--:|--:|--:|--:|
| 4 | 1.000 (0.985-1.014) | 20 | 1.00 | 0.99 |
| 6 | 0.995 (0.981-1.009) | 20 | 1.00 | 1.01 |
| 8 | 0.989 (0.977-1.001) | 20 | 1.00 | 1.01 |
| 10 | 0.954 (0.939-0.970) | 20 | 1.15 | 1.03 |
| 20 | 0.945 (0.930-0.959) | 20 | 1.15 | 1.04 |

At 20 threads, the mixed gate: **not met**

## Verdict

- wikidata-bz6 (unchanged): met
- wikidata-mix93 (unchanged): met
- wikidata-bz3 (rule): met
- wikidata-mix39 (mixed): not met

All met: **no**
