# The ψ census on other inputs (atlas V; libbzip2 level 9; all blocks)

| input (MB) | G | RUN | FOR-(16−64/G) | EF-(14−64/G) | crossing | too wide | exceptions | bits a row |
|---|--:|--:|--:|--:|--:|--:|--:|--:|
| random bytes (2.7 MB, n/r 1.0) | 32 | 0.00% | 99.11% | 0.00% | 0.89% | 0.00% | 0.89% | 16.29 |
| a 4-letter alphabet, random (2.7 MB, n/r 1.4) | 32 | 3.44% | 96.42% | 0.09% | 0.03% | 0.01% | 0.04% | 16.01 |
| English prose (docs/journal/*.md) (0.5 MB, n/r 2.3) | 32 | 8.88% | 87.59% | 2.59% | 0.75% | 0.20% | 0.95% | 16.30 |
| Rust source (src, examples, tests) (0.4 MB, n/r 3.1) | 32 | 22.39% | 73.81% | 2.75% | 0.92% | 0.14% | 1.05% | 16.34 |
| an ELF binary (this program) (3.0 MB, n/r 3.4) | 32 | 34.81% | 54.77% | 8.59% | 1.09% | 0.74% | 1.83% | 16.58 |
| already compressed (the corpus's own bz2 bytes, its first 2.7 MB) (2.7 MB, n/r 1.0) | 32 | 0.00% | 99.05% | 0.07% | 0.88% | 0.00% | 0.88% | 16.28 |
| periodic, period 1000 (2.7 MB, n/r 892.0) | 32 | 96.60% | 0.24% | 1.15% | 0.85% | 1.16% | 2.01% | 16.64 |

## The designs' exact sizes on the same inputs (G = 32)

| input | fixed-line bits a row | two-tier bits a row | two-tier steps with a second dependent load |
|---|--:|--:|--:|
| random bytes (2.7 MB, n/r 1.0) | 16.29 | 18.14 | 100.00% |
| a 4-letter alphabet, random (2.7 MB, n/r 1.4) | 16.01 | 17.46 | 96.56% |
| English prose (docs/journal/*.md) (0.5 MB, n/r 2.3) | 16.30 | 16.73 | 91.12% |
| Rust source (src, examples, tests) (0.4 MB, n/r 3.1) | 16.34 | 14.59 | 77.61% |
| an ELF binary (this program) (3.0 MB, n/r 3.4) | 16.58 | 12.72 | 65.19% |
| already compressed (the corpus's own bz2 bytes, its first 2.7 MB) (2.7 MB, n/r 1.0) | 16.28 | 18.14 | 100.00% |
| periodic, period 1000 (2.7 MB, n/r 892.0) | 16.64 | 2.86 | 3.40% |
