# Guarded first-level POPCNT in CI run 44436826833

The Intel(R) Xeon(R) 6975P-C runner exposed 4 logical CPUs. Each sample decoded four copies of Wikidata, or 838,800,784 bytes. The runner used 4W slots at worker counts 1, 2 and 4 and a two-second settle between samples. The official schedule completed 20 paired rounds and 300 timed samples. Each of the 5 arms passed all 338 correctness records, including the Wikidata output and the mutation suite.

PR head `e3dc2c416fd6f10b8214940bea8eecfe66c78c14`, master reference build `735796c1882782e9b93fe7eb3d8d2068962d8c42`. The scorer projection came from `f8061b3c6e13047a787bc2169277389ed7b1b897` with path revision `604dac3f46569edd9ac9b6706b23a68d9d7d8f8d`. The manifest printed `scorer v1 at 6a8a366cd786d58a528000b58da12fd2cfb47be8`.

Candidate entry `research/astra-e/formulations/f2-pages/instructions/candidate.rs`, SHA-256 `9eedac9e1f7c54a4eaf966c5c1f34e8b919862995ee272085b4cac436d9bfd8b`. Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

The fifth arm built `research/astra-e/formulations/f2-front-counts/candidate.rs` at `252646aec0dd1a80a6969b2e1598c73486006efd` in a dedicated checkout with the same scorer projection. The two candidate executables have separate build hashes in the manifest.

The wall ratio uses the geometric mean of paired candidate/reference times. Its 95% Student interval uses paired log ratios. RSS is the candidate median peak divided by the reference median peak. The ZIP passed CRC checks, and the archived arm order, verification records, timing intervals and RSS ratios reproduced from the raw JSONL.

| Workers | Reference | Wall ratio [95% interval] | Median peak RSS ratio |
| ---: | --- | ---: | ---: |
| 1 | master LazyCells | 0.8772 [0.8681, 0.8863] | 0.9982 |
| 1 | master eager Cells | 1.1593 [1.1431, 1.1757] | 1.0077 |
| 1 | master opend | 0.8873 [0.8726, 0.9022] | 1.0302 |
| 1 | frozen predecessor | 0.9184 [0.9100, 0.9269] | 0.9996 |
| 2 | master LazyCells | 0.8893 [0.8828, 0.8958] | 0.9894 |
| 2 | master eager Cells | 1.1413 [1.1303, 1.1524] | 0.9862 |
| 2 | master opend | 0.9135 [0.9069, 0.9202] | 1.0462 |
| 2 | frozen predecessor | 0.9215 [0.9152, 0.9279] | 1.0006 |
| 4 | master LazyCells | 0.9366 [0.9327, 0.9406] | 0.9281 |
| 4 | master eager Cells | 1.0939 [1.0887, 1.0991] | 0.9874 |
| 4 | master opend | 1.0093 [1.0055, 1.0130] | 1.0743 |
| 4 | frozen predecessor | 0.9317 [0.9283, 0.9350] | 0.9994 |

GitHub published artifact `14489673181` with SHA-256 `ca632b90f6ba4a7e33b47040ea5f1d2830ff9a8239d647dce94b9d6b28596298`. The ZIP includes all correctness cases, raw timing rows, build identities, CPU information, console output and the original comparison tables. [Workflow run](https://github.com/lmmx/mibz2/actions/runs/44436826833) · [Archived artifact](guarded-popcnt-44436826833.zip)

