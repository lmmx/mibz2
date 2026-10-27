# Benchmarks

mibz2's speed is measured on GitHub's runners by the Benchmark workflow (`.github/workflows/bench.yml`), with the A/B harness `examples/ab.rs` and `bench/report.py`. Nothing about performance is decided on a developer's machine.

- [History](<https://lmmx.github.io/mibz2/pr-preview/pr-49/benchmarks/history/index.md>): master's figures over its commits.
- [Alternatives](<https://lmmx.github.io/mibz2/pr-preview/pr-49/benchmarks/alternatives/index.md>): the latest master commit against the other bzip2 decoders: pbz2, lbzip2, indexed\_bzip2, fbz, the `lbzip2` crate, libbzip2 and libbz2-rs.

These are decoder-level results: one process decoding one file, nothing else. End-to-end results, the decoder inside a pipeline, are in [wikidata-pq's benchmark history](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

## What runs when

| event | arms | result |
| --- | --- | --- |
| a pull request that touches the decoder, the harness, the corpora or the bench | its head and its base (the merge base with master), both built on the runner | one comment on the pull request, edited in place on every push |
| a push to master | mibz2 and the [alternatives](<https://lmmx.github.io/mibz2/pr-preview/pr-49/benchmarks/alternatives/index.md>) | the commit's result on the `bench-results` branch, and these pages |
| the Actions tab | either: a base commit given, as a pull request; none, as master | the run's summary, not recorded |

Pull requests never run the alternatives: their runs are the head against the base only. The alternatives that are Rust crates with dependencies of their own (fbz, the `lbzip2` crate, libbz2-rs) are built into a second build of the harness, `bench/alternatives`, so the crate's own dev-dependencies stay as they are.

Only master's commits are recorded. A pull request's runs, and experiments that did not land, are not recorded or shown here; experiments are written up in the [journals](<https://lmmx.github.io/mibz2/pr-preview/pr-49/background/journals/index.md>).

## The workload

- **The crate's corpora** (`ab gen`, `tests/common/mod.rs`), at scale 4, each compressed by libbzip2 1.0.8 at level 9: `text` (the crate's own sources and README, repeated: 12 MB), `random` (8 MB), `onebyte` (one byte repeated, 20 MB), `pattern` (a 7-byte pattern, 19.6 MB), `runs` (runs of random lengths, 10 MB) and `records` (binary records, 9.6 MB). The `text` corpus is made from the commit's own sources, so its bytes change a little from commit to commit.
- **The Wikidata dump's own blocks:** 12 byte-range windows of 1 MiB of `wikidata-20261019-all.json.bz2`, at (2i + 1)/24 of the file. `examples/windows.rs` keeps each window's whole blocks, their bits untouched, as a stream of its own, checked against libbzip2 and lbzip2: 431 blocks, 11.85 MB of bz2 → 209.7 MB of JSON (17.69×). `bench/wikidata.json` pins each window's offset, length and SHA-256, and the corpus's. Wikimedia serves a dated dump for about six weeks, so the windows are kept on the repository's `bench-data` branch.

## How it is measured

- **A run** is one process decoding one file from disk, end to end, its output hashed (the same hash for every arm, or the run fails: the same-bytes check). It records the wall time, the CPU time (user + sys), the peak RSS, and each thread's CPU.
- **A round** decodes every corpus once with each arm at each thread count: 1 thread (pinned to one CPU) and the runner's cores. The arms' order is rotated and reversed round to round. There are 10 rounds.
- **Block-parallel or single-threaded:** each arm is classed in its records (`parallel`). A single-threaded decoder (libbzip2, libbz2-rs) runs at 1 thread only, once a round; at the runner's cores its figures are those 1-thread runs, paired with mibz2's at that count, and shown as 1-thread runs, never as a decode on more threads.
- **On a pull request**, the base's own harness binary is an arm (`mibz2@base/ab`), so both builds run in the same process shape on the same runner.
- **Paired ratios:** each round's ratio of one arm's time to the reference arm's (the base; or mibz2, against the alternatives), their geometric mean, and its 95% t-interval over the rounds. A change is called faster or slower only where the interval excludes no change.
- **The bench version** (`bench/report.py`, semver) is stamped on every result. A change to what is measured or how is a new major version, and the history never joins a series across major versions, runner CPU models or sets of corpora.

## The full-dump figure is modelled

Each result also gives a whole dump's decode (103.5 GB of bz2), **modelled, not measured**, from the Wikidata blocks' runs, round by round:

- the scanner's time (the one serial thread: each block's header and Huffman symbols) per byte of bz2, scaled by the dump's bz2 bytes;
- the workers' time per byte of output, scaled by the dump's output, taken as its bz2 bytes times the sample's ratio (17.69×; another sample of the same dump, 24 windows of 25 MB, measured 18.68×; wikidata-pq's CI slice, lines recompressed, is 13.69×);
- at N threads, the larger of the scanner's time and the workers' over N, times the sample's own wall over that bound; the same gap read as a fixed cost is given beside it.

It assumes time linear in bytes, the sample's mix standing for the dump's, and the runner's CPU; its interval carries the rounds' spread and the sample ratio's.
