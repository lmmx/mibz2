# Compact eager runs with eight response preloads in CI run 44440381983

The AMD EPYC 9V74 80-Core Processor runner exposed 4 logical CPUs. Each sample decoded four copies of Wikidata, or 838,800,784 bytes. The runner used 4W slots at worker counts 1, 2 and 4 and a two-second settle between samples. The official schedule completed 20 paired rounds and 300 timed samples. Each of the 5 arms passed all 338 correctness records, including the Wikidata output and the mutation suite.

PR head `5cefe941ff0c9bf6e436ef1bf171592fbc927d9c`, master reference build `735796c1882782e9b93fe7eb3d8d2068962d8c42`. The scorer projection came from `75297a1d57b2bdf262c3a19f3064848dedf2c493` with path revision `75297a1d57b2bdf262c3a19f3064848dedf2c493`. The manifest printed `scorer v1 at 81585172f5fb9e7a59f330124779ba084fd6bf04`.

Candidate entry `research/astra-e/formulations/f2-pages/payload/slots/preload.rs`, SHA-256 `6bfd17944f3e76a54525d0f27d661321a189d7d8c9da9392d5bbec3cd5f9a77a`. Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

The fifth arm built `research/astra-e/formulations/f2-pages/instructions/candidate.rs` at `3651470cb0ef5458e75cf97d3de363a1c73f2f76` in a dedicated checkout with the same scorer projection. The two candidate executables have separate build hashes in the manifest.

The wall ratio uses the geometric mean of paired candidate/reference times. Its 95% Student interval uses paired log ratios. RSS is the candidate median peak divided by the reference median peak. The ZIP passed CRC checks, and the archived arm order, verification records, timing intervals and RSS ratios reproduced from the raw JSONL.

| Workers | Reference | Wall ratio [95% interval] | Median peak RSS ratio |
| ---: | --- | ---: | ---: |
| 1 | master LazyCells | 0.8468 [0.8339, 0.8599] | 0.9931 |
| 1 | master eager Cells | 1.1279 [1.1079, 1.1483] | 1.0030 |
| 1 | master opend | 0.8817 [0.8622, 0.9016] | 1.0248 |
| 1 | frozen predecessor | 0.9134 [0.9066, 0.9203] | 0.9945 |
| 2 | master LazyCells | 0.8631 [0.8534, 0.8729] | 0.9819 |
| 2 | master eager Cells | 1.1264 [1.1117, 1.1413] | 0.9920 |
| 2 | master opend | 0.9112 [0.8982, 0.9245] | 1.0369 |
| 2 | frozen predecessor | 0.9151 [0.9047, 0.9256] | 0.9963 |
| 4 | master LazyCells | 0.8904 [0.8878, 0.8929] | 0.9140 |
| 4 | master eager Cells | 1.0701 [1.0649, 1.0754] | 0.9839 |
| 4 | master opend | 1.0069 [1.0033, 1.0105] | 1.0735 |
| 4 | frozen predecessor | 0.9225 [0.9198, 0.9251] | 0.9867 |

GitHub published artifact `14491856447` with SHA-256 `308cbaabcbda6cfcd588e171adbc871597ace97661c61745e8ba216a7630b558`. The ZIP includes all correctness cases, raw timing rows, build identities, CPU information, console output and the original comparison tables. [Workflow run](https://github.com/lmmx/mibz2/actions/runs/44440381983) · [Archived artifact](compact-preload-44440381983.zip)

