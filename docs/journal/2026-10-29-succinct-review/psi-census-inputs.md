# The ψ census on other inputs (atlas V; libbzip2 level 9; all blocks)

| input (MB) | G | RUN | FOR-(16−64/G) | EF-(14−64/G) | crossing | too wide | exceptions | bits a row |
|---|--:|--:|--:|--:|--:|--:|--:|--:|
| random bytes (2.7 MB, n/r 1.0) | 32 | 0.00% | 99.11% | 0.00% | 0.89% | 0.00% | 0.89% | 16.29 |
| a 4-letter alphabet, random (2.7 MB, n/r 1.4) | 32 | 3.44% | 96.42% | 0.09% | 0.03% | 0.01% | 0.04% | 16.01 |
| English prose (docs/journal/*.md) (0.5 MB, n/r 2.3) | 32 | 8.86% | 87.58% | 2.62% | 0.77% | 0.17% | 0.94% | 16.30 |
| Rust source (src, examples, tests) (0.4 MB, n/r 3.1) | 32 | 22.29% | 74.09% | 2.57% | 0.94% | 0.11% | 1.05% | 16.34 |
| an ELF binary (this program) (3.0 MB, n/r 3.4) | 32 | 34.84% | 54.73% | 8.61% | 1.08% | 0.75% | 1.83% | 16.58 |
| already compressed (the corpus's own bz2 bytes, its first 2.7 MB) (2.7 MB, n/r 1.0) | 32 | 0.00% | 99.05% | 0.07% | 0.88% | 0.00% | 0.88% | 16.28 |
| periodic, period 1000 (2.7 MB, n/r 892.0) | 32 | 96.60% | 0.24% | 1.15% | 0.85% | 1.16% | 2.01% | 16.64 |
