## The 32-row ψ records: rows by class

| blocks | n | rows | FOR-14 | EF-12 | crossing a boundary | span too wide | all exceptions | bits a row, exceptions at 24 |
|---|--:|--:|--:|--:|--:|--:|--:|--:|
| all | 431 | 209.7 M | 98.11% | 1.40% | 0.29% | 0.21% | 0.49% | 16.12 |
| rows < 300 k | 198 | 0.0 M | 4.78% | 0.00% | 95.22% | 0.00% | 95.22% | 38.85 |
| > 700 k | 233 | 209.7 M | 98.11% | 1.40% | 0.28% | 0.21% | 0.49% | 16.12 |
| σ ≤ 64 | 198 | 0.0 M | 4.78% | 0.00% | 95.22% | 0.00% | 95.22% | 38.85 |
| σ 65-128 | 233 | 209.7 M | 98.11% | 1.40% | 0.28% | 0.21% | 0.49% | 16.12 |

## Rows by their group's span (all blocks; groups inside one bucket)

| span | rows |
|---|--:|
| [2^0, 2^1) | 0.00% (cumulative 0.00%) |
| [2^2, 2^3) | 0.00% (cumulative 0.00%) |
| [2^3, 2^4) | 0.00% (cumulative 0.00%) |
| [2^4, 2^5) | 77.60% (cumulative 77.60%) |
| [2^5, 2^6) | 3.10% (cumulative 80.71%) |
| [2^6, 2^7) | 1.69% (cumulative 82.40%) |
| [2^7, 2^8) | 1.88% (cumulative 84.27%) |
| [2^8, 2^9) | 2.95% (cumulative 87.23%) |
| [2^9, 2^10) | 3.84% (cumulative 91.06%) |
| [2^10, 2^11) | 2.62% (cumulative 93.69%) |
| [2^11, 2^12) | 2.03% (cumulative 95.72%) |
| [2^12, 2^13) | 1.60% (cumulative 97.32%) |
| [2^13, 2^14) | 1.07% (cumulative 98.39%) |
| [2^14, 2^15) | 0.69% (cumulative 99.09%) |
| [2^15, 2^16) | 0.47% (cumulative 99.56%) |
| [2^16, 2^17) | 0.23% (cumulative 99.79%) |
| [2^17, 2^18) | 0.14% (cumulative 99.93%) |
| [2^18, 2^19) | 0.05% (cumulative 99.98%) |
| [2^19, 2^20) | 0.02% (cumulative 100.00%) |

## F windows of 2^k rows holding a bucket boundary (all blocks)

| k | windows | mixed | rows in mixed windows | rows stepped (past a window's first boundary) | mixed with 1 / 2 / 3 / 4+ boundaries |
|--:|--:|--:|--:|--:|--:|
| 4 | 13106771 | 0.14% | 0.14% | 0.07% | 17766 / 548 / 129 / 419 |
| 5 | 6553457 | 0.29% | 0.29% | 0.15% | 17530 / 762 / 169 / 352 |
| 6 | 3276910 | 0.56% | 0.56% | 0.29% | 16755 / 961 / 283 / 346 |
| 7 | 1638651 | 1.08% | 1.07% | 0.56% | 15776 / 1065 / 411 / 436 |
| 8 | 819430 | 2.06% | 2.04% | 1.06% | 14661 / 1040 / 528 / 615 |
