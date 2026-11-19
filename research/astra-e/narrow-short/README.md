# Combining narrow node IDs with immediate endpoint words

`candidate.rs` composes the two independently correct changes in one endpoint constructor. On blocks passing the 16-bit node bound, the four links in each node use `u16`. The separate `u32` word field can still distinguish literals, immediate two- and three-byte payloads, and rule IDs. The transformations operate on distinct fields and their accessors preserve both the interval permutation and word spelling. Blocks exceeding the narrow node bound take the wide version of the same tagged-word endpoint. `ahead.rs` also reads the valid next edge before expanding the current spelling, including on the step that closes the primary cycle. This changes no edge or emitted byte. The unchanged LazyCells fallback handles density-rejected blocks. [Geometry proof](../narrow-endpoint/README.md) · [Word proof](../short-words/README.md) · [Emission](../lookahead/README.md)

The two combined arms each matched the independent marked inverse and the un-narrowed immediate-word kernel's bytes and structural counters in 24,590 tests. These include alternating wide/narrow admissions around the proven H bound, all binary columns up to length nine at every primary, multiple LF cycles, three width thresholds and capped runs. `prepare.py` deterministically rebuilds both constructors, their narrow endpoints and the common worker adapter using the pinned journal sources and committed transformations. [Combined check](check-short_words.txt) · [Combined with lookahead](check-ahead.txt)

## Eleven-block stage comparison

Run `bash research/astra-e/narrow-short/stage.sh corpora/wikidata.bz2 target/narrow-stage-repeat` from the repository. This reuses Stage 2's prespecified eleven eligible stream blocks, seven rotating arms, 21 measured passes and one untimed warm pass. The CPU and wall clocks enclose the complete `invert_runs` call on prepared records. Every arm's output was compared with released LazyCells on every lap. A concurrent correctness-only dry run was active during this first stage comparison. Wall intervals describe pass-to-pass variation in this run. The sampled inverse region excludes entropy decoding, MTF/run-front work, RLE1, CRC, delivery and process memory. [Raw laps](stage/laps.csv) · [Source, input and binary hashes](stage/manifest.txt) · [All cells](stage/summary.csv)

| Arm over historical width-16 queued endpoint | Wall geometric mean [95%] | Thread CPU geometric mean [95%] |
|---|---:|---:|
| Narrow links | 0.98573 [0.93012, 1.04466] | 0.98712 [0.93148, 1.04609] |
| Immediate words | 0.99267 [0.95740, 1.02924] | 0.99364 [0.95854, 1.03003] |
| Both | 0.96371 [0.93151, 0.99702] | 0.96467 [0.93165, 0.99887] |
| Both plus lookahead | 0.94960 [0.91498, 0.98553] | 0.95065 [0.91712, 0.98541] |

The last two intervals give a reason for complete-worker comparisons. Absolute gains are unverified outside this sample. The paired stage contrast is against stock opend on its admitted population. Eager Cells was faster than every endpoint arm here. Separate scores at fixed worker counts and a quiet repeat can decide whether either combined variant survives in the complete decoder. The scored stage source was prepared while commit `f5748db78ccc4f4cc29dafc11837be95f1e0964d` was checked out, before the shared worktree committed the combined sources. Its manifest identifies the actual generated Rust inputs and executable by SHA-256.

Rebuild the independent marked comparator using the commands in `../narrow-endpoint/README.md`, first invoking this directory's `prepare.py` and adding either `--cfg short_words` or `--cfg ahead` to the same `rustc` command. The stage source generator and script use the same Rust compiler and code-generation switches as Stage 2.
