# The ψ census, the groups' shapes (corpora/wikidata.bz2; #99)

Sections 1–4 of the same run are unchanged from psi-census.md (diffed: identical), and its per-block lines keep psi-census.jsonl's fields (identical on all 431 blocks), adding d, breaks_in, k32, k32_psi and the joint tables (plateau bin × column, by rows and by groups).

## 5. The groups' shapes at G = 32: plateaus × bucket × span, the 233 full blocks

209697662 rows, 6553071 groups; non-translation rows (a break inside, or crossing) 22.61%

### 5.1 By rows, as a share of all rows

| plateaus | one bucket, FOR-14 span | one bucket, EF-12 span | one bucket, too wide | crossing | all |
|---|--:|--:|--:|--:|--:|
| 1 | 77.39% | 0.00% | 0.00% | 0.00% | 77.39% |
| 2 | 4.01% | 0.14% | 0.03% | 0.01% | 4.19% |
| 3–4 | 3.95% | 0.28% | 0.03% | 0.03% | 4.29% |
| 5–8 | 3.92% | 0.41% | 0.05% | 0.07% | 4.45% |
| 9+ | 8.85% | 0.57% | 0.10% | 0.17% | 9.68% |
| all | 98.11% | 1.40% | 0.21% | 0.28% | 100.00% |

### 5.2 By groups, as a share of all groups

| plateaus | one bucket, FOR-14 span | one bucket, EF-12 span | one bucket, too wide | crossing | all |
|---|--:|--:|--:|--:|--:|
| 1 | 77.39% | 0.00% | 0.00% | 0.00% | 77.39% |
| 2 | 4.01% | 0.14% | 0.03% | 0.01% | 4.19% |
| 3–4 | 3.95% | 0.28% | 0.03% | 0.03% | 4.29% |
| 5–8 | 3.92% | 0.41% | 0.05% | 0.07% | 4.45% |
| 9+ | 8.85% | 0.57% | 0.10% | 0.17% | 9.68% |
| all | 98.11% | 1.40% | 0.21% | 0.28% | 100.00% |

### 5.3 The two-plateau share

Two-plateau groups inside one bucket: 4.18% of all rows, 18.46% of the non-translation rows; of them FOR-14 span 4.01%, EF-12 0.14%, too wide 0.03% (of all rows)

| per full block | min / p10 / median / p90 / max |
|---|--:|
| two plateaus, one bucket, % of all rows | 0.91 / 2.43 / 4.23 / 5.48 / 7.12 |
| two plateaus, one bucket, % of non-translation rows | 3.78 / 10.10 / 19.34 / 24.25 / 27.33 |
| two plateaus, one bucket, FOR-14 span, % of all rows | 0.87 / 2.39 / 4.09 / 5.26 / 6.98 |

### 5.4 Rows by exact plateau count (% of all rows)

| plateaus | one bucket | crossing | cumulative |
|--:|--:|--:|--:|
| 1 | 77.39% | 0.00% | 77.39% |
| 2 | 4.18% | 0.01% | 81.58% |
| 3 | 2.46% | 0.02% | 84.05% |
| 4 | 1.80% | 0.02% | 85.87% |
| 5 | 1.42% | 0.02% | 87.31% |
| 6 | 1.16% | 0.02% | 88.48% |
| 7 | 0.98% | 0.02% | 89.48% |
| 8 | 0.82% | 0.02% | 90.32% |
| 9 | 0.70% | 0.02% | 91.03% |
| 10 | 0.60% | 0.02% | 91.65% |
| 11 | 0.53% | 0.02% | 92.19% |
| 12 | 0.47% | 0.02% | 92.68% |
| 13 | 0.43% | 0.01% | 93.12% |
| 14 | 0.39% | 0.01% | 93.52% |
| 15 | 0.37% | 0.01% | 93.90% |
| 16 | 0.34% | 0.01% | 94.25% |
| 17 | 0.32% | 0.01% | 94.58% |
| 18 | 0.30% | 0.01% | 94.89% |
| 19 | 0.28% | 0.01% | 95.18% |
| 20 | 0.27% | 0.01% | 95.45% |
| 21 | 0.26% | 0.01% | 95.72% |
| 22 | 0.25% | 0.00% | 95.97% |
| 23 | 0.24% | 0.00% | 96.21% |
| 24 | 0.24% | 0.00% | 96.46% |
| 25 | 0.24% | 0.00% | 96.70% |
| 26 | 0.25% | 0.00% | 96.95% |
| 27 | 0.27% | 0.00% | 97.22% |
| 28 | 0.36% | 0.00% | 97.58% |
| 29 | 0.51% | 0.00% | 98.09% |
| 30 | 0.70% | 0.00% | 98.80% |
| 31 | 0.74% | 0.00% | 99.54% |
| 32 | 0.46% | 0.00% | 100.00% |

Crossing rows by span (max − min of ψ): FOR-14 0.00% (384 rows), EF-12 0.00% (3648 rows), too wide 0.28% (586244 rows) (of all rows)

## 6. Cross-checks (the full blocks)

- the joint table's columns and plateau-1 cells equal the census's classes (RUN, FOR-14, EF-12, too wide, crossing) and K₃₂ on every block (asserted)
- d, every break: 15141365; inside an aligned group: 14668337 (96.88%); at a group edge (j ≡ 31 mod 32): 473028
- K₃₂ (census rule): 1481882; groups with a break inside (ψ's rule): 1481882

| per full block | min / p10 / median / p90 / max |
|---|--:|
| d ÷ K₃₂ | 7.57 / 8.46 / 9.39 / 13.03 / 26.69 |
| breaks inside a group ÷ K₃₂ | 7.33 / 8.20 / 9.10 / 12.63 / 25.86 |
| breaks inside a group ÷ groups with a break inside | 7.33 / 8.20 / 9.10 / 12.63 / 25.86 |
