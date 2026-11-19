# Combined narrow, short-copy and lookahead in CI run 44429350230

The AMD EPYC 7763 64-Core Processor runner exposed 4 logical CPUs. Each sample decoded four copies of Wikidata, or 838,800,784 bytes. The runner used 4W slots at worker counts 1, 2 and 4 and a two-second settle between samples. The official schedule completed 16 paired rounds and 192 timed samples. Each of the 4 arms passed all 338 correctness records, including the Wikidata output and the mutation suite.

PR head `e53d5e23228bf82c48bdd7e8099f4ac67e3ddb69`, master reference build `9c31461df7bef24cecf79348545b27ee26846835`. The scorer projection came from `604dac3f46569edd9ac9b6706b23a68d9d7d8f8d` with path revision `604dac3f46569edd9ac9b6706b23a68d9d7d8f8d`. The manifest printed `scorer v1 at 6a8a366cd786d58a528000b58da12fd2cfb47be8`.

Candidate entry `research/astra-e/narrow-short/ahead.rs`, SHA-256 `7b7741a017f12aaf5c9624028e32715ad4fd806210bef0c6b210c7c1d16631d6`. Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

The wall ratio uses the geometric mean of paired candidate/reference times. Its 95% Student interval uses paired log ratios. RSS is the candidate median peak divided by the reference median peak. The ZIP passed CRC checks, and the archived arm order, verification records, timing intervals and RSS ratios reproduced from the raw JSONL.

| Workers | Reference | Wall ratio [95% interval] | Median peak RSS ratio |
| ---: | --- | ---: | ---: |
| 1 | master LazyCells | 0.8894 [0.8727, 0.9065] | 0.9693 |
| 1 | master eager Cells | 1.2339 [1.2075, 1.2609] | 0.9787 |
| 1 | master opend | 0.9581 [0.9493, 0.9670] | 1.0002 |
| 2 | master LazyCells | 0.8938 [0.8844, 0.9032] | 0.9452 |
| 2 | master eager Cells | 1.2039 [1.1945, 1.2133] | 0.9514 |
| 2 | master opend | 0.9632 [0.9533, 0.9733] | 0.9899 |
| 4 | master LazyCells | 0.8332 [0.8024, 0.8651] | 0.8710 |
| 4 | master eager Cells | 1.0168 [0.9876, 1.0470] | 0.9091 |
| 4 | master opend | 0.9639 [0.9594, 0.9684] | 0.9801 |

GitHub published artifact `14486197942` with SHA-256 `119b35be77fdce1737a998ae5761a30185969484fa94ad94f459aa198d637278`. The ZIP includes all correctness cases, raw timing rows, build identities, CPU information, console output and the original comparison tables. [Workflow run](https://github.com/lmmx/mibz2/actions/runs/44429350230) · [Archived artifact](combined-ahead-44429350230.zip)

