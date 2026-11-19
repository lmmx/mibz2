# Delimiter-phrase refill scheduling in CI run 44433942216

The Intel(R) Xeon(R) 6973P-C runner exposed 4 logical CPUs. Each sample decoded four copies of Wikidata, or 838,800,784 bytes. The runner used 4W slots at worker counts 1, 2 and 4 and a two-second settle between samples. The official schedule completed 20 paired rounds and 300 timed samples. Each of the 5 arms passed all 338 correctness records, including the Wikidata output and the mutation suite.

PR head `daf8a86f0bc6f07e6dc0d24d837f92104219081c`, master reference build `9c31461df7bef24cecf79348545b27ee26846835`. The scorer projection came from `6fb777541e3ce4613c34e14d8f5def7fbada8b4a` with path revision `604dac3f46569edd9ac9b6706b23a68d9d7d8f8d`. The manifest printed `scorer v1 at 6a8a366cd786d58a528000b58da12fd2cfb47be8`.

Candidate entry `research/astra-e/formulations/f2-scheduler/candidate.rs`, SHA-256 `1a659b1c65663db8901146d58623261c238c87dcd1a5ce6cb234c372cd59d4bf`. Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

The fifth arm built `research/astra-e/formulations/f2-frontier/candidate.rs` at `d48e80ff01eeee6d23277945e2f58b61004470aa` in a dedicated checkout with the same scorer projection. The two candidate executables have separate build hashes in the manifest.

The wall ratio uses the geometric mean of paired candidate/reference times. Its 95% Student interval uses paired log ratios. RSS is the candidate median peak divided by the reference median peak. The ZIP passed CRC checks, and the archived arm order, verification records, timing intervals and RSS ratios reproduced from the raw JSONL.

| Workers | Reference | Wall ratio [95% interval] | Median peak RSS ratio |
| ---: | --- | ---: | ---: |
| 1 | master LazyCells | 1.0252 [1.0116, 1.0390] | 0.9992 |
| 1 | master eager Cells | 1.3121 [1.2855, 1.3392] | 1.0093 |
| 1 | master opend | 1.0511 [1.0338, 1.0687] | 1.0319 |
| 1 | frozen predecessor | 0.9561 [0.9437, 0.9687] | 1.0021 |
| 2 | master LazyCells | 1.0153 [1.0003, 1.0306] | 0.9949 |
| 2 | master eager Cells | 1.2562 [1.2371, 1.2757] | 1.0068 |
| 2 | master opend | 1.0488 [1.0321, 1.0658] | 1.0439 |
| 2 | frozen predecessor | 0.9586 [0.9477, 0.9696] | 1.0029 |
| 4 | master LazyCells | 1.0196 [1.0021, 1.0374] | 0.9461 |
| 4 | master eager Cells | 1.1644 [1.1419, 1.1874] | 0.9945 |
| 4 | master opend | 1.1256 [1.1032, 1.1486] | 1.0677 |
| 4 | frozen predecessor | 0.9784 [0.9594, 0.9977] | 1.0025 |

GitHub published artifact `14488669091` with SHA-256 `9312481e765e9f790fb2e76a4a3be6376ab9418d895a66076b8e482b734edcd0`. The ZIP includes all correctness cases, raw timing rows, build identities, CPU information, console output and the original comparison tables. [Workflow run](https://github.com/lmmx/mibz2/actions/runs/44433942216) · [Archived artifact](delimiter-refill-44433942216.zip)

