# Combined word-action writer in CI run 44436152031

The AMD EPYC 9V74 80-Core Processor runner exposed 4 logical CPUs. Each sample decoded four copies of Wikidata, or 838,800,784 bytes. The runner used 4W slots at worker counts 1, 2 and 4 and a two-second settle between samples. The official schedule completed 20 paired rounds and 300 timed samples. Each of the 5 arms passed all 338 correctness records, including the Wikidata output and the mutation suite.

PR head `21218294764c887f16f6d72beb1c2ebbc3368eed`, master reference build `735796c1882782e9b93fe7eb3d8d2068962d8c42`. The scorer projection came from `f8061b3c6e13047a787bc2169277389ed7b1b897` with path revision `604dac3f46569edd9ac9b6706b23a68d9d7d8f8d`. The manifest printed `scorer v1 at 6a8a366cd786d58a528000b58da12fd2cfb47be8`.

Candidate entry `research/astra-e/formulations/f2-rle/combined/candidate.rs`, SHA-256 `7cd9f6bb7e5a65916aa76542d9e97785c81e71ce96402932da1d635292e31352`. Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

The fifth arm built `research/astra-e/narrow-short/ahead.rs` at `5c7704b3d149b05b8e9dc2ace06da0179459a8b3` in a dedicated checkout with the same scorer projection. The two candidate executables have separate build hashes in the manifest.

The wall ratio uses the geometric mean of paired candidate/reference times. Its 95% Student interval uses paired log ratios. RSS is the candidate median peak divided by the reference median peak. The ZIP passed CRC checks, and the archived arm order, verification records, timing intervals and RSS ratios reproduced from the raw JSONL.

| Workers | Reference | Wall ratio [95% interval] | Median peak RSS ratio |
| ---: | --- | ---: | ---: |
| 1 | master LazyCells | 0.9161 [0.8992, 0.9334] | 0.9753 |
| 1 | master eager Cells | 1.2183 [1.1915, 1.2458] | 0.9847 |
| 1 | master opend | 0.9666 [0.9445, 0.9891] | 1.0060 |
| 1 | frozen predecessor | 1.0046 [0.9918, 1.0176] | 1.0058 |
| 2 | master LazyCells | 0.9236 [0.9148, 0.9325] | 0.9542 |
| 2 | master eager Cells | 1.1898 [1.1730, 1.2068] | 0.9489 |
| 2 | master opend | 0.9839 [0.9698, 0.9983] | 1.0033 |
| 2 | frozen predecessor | 1.0136 [1.0052, 1.0220] | 1.0083 |
| 4 | master LazyCells | 0.8686 [0.8605, 0.8767] | 0.8750 |
| 4 | master eager Cells | 1.0273 [1.0182, 1.0365] | 0.9066 |
| 4 | master opend | 0.9884 [0.9813, 0.9955] | 0.9973 |
| 4 | frozen predecessor | 1.0142 [1.0063, 1.0221] | 1.0049 |

GitHub published artifact `14490291075` with SHA-256 `653874f0e7f64986e68b4aac9fd810fc69ad2807039bf9fa49dc0108a487909f`. The ZIP includes all correctness cases, raw timing rows, build identities, CPU information, console output and the original comparison tables. [Workflow run](https://github.com/lmmx/mibz2/actions/runs/44436152031) · [Archived artifact](combined-word-actions-44436152031.zip)

