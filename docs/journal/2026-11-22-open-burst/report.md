# Cycle 29 six-lane emission on the #719 construction stack

LeanRegBurst6_256 uses forward packed links once the construction retains at least 140,000 rows. Six scouts first measure the spelled length between cuts. A walk through at most 256 cuts places each segment in output order. Six emitters then fill their assigned ranges, and exact stores repair up to 15 bytes at each internal boundary. The scouts use a per-worker `u16` length table derived from #719's interned dictionary. Long words use #719's offsets in their inline slots. The extra scratch is two bytes per reserved length entry plus fixed stack arrays. An unsuitable length or a marked cycle with fewer than six complete segments takes the serial forward walk. Smaller packed blocks in #719's two-ended range keep their XOR links.

[Mechanisms candidate](https://github.com/lmmx/mibz2/blob/34ddbdd69e783de6d72d230056da5532819e3662/docs/journal/2026-11-21-carbon-cycle29/candidate_open_c29.rs) · [#719 baseline](https://github.com/lmmx/mibz2/commit/53f7ecf935d62ac7896eafaf55c5d249789614ca) · [Measured port](https://github.com/lmmx/mibz2/commit/4bee5990134fca0f56a8c810682c81169a9f00f8)

## Paired wall times

Each cell of times gives master / port. Ratios are port / master, with paired 95% intervals. A/A compares two copies of the same master kernel in the same executable. Lower ratios are faster. The authenticated and dense rows measure the full admitted Open kernel on prebuilt run records. The whole-stream rows include admission, fallback, decoder construction and draining all 209,700,196 decoded bytes.

| Workload | Workers | Master / port (ms) | Port / master [95% CI] | A/A [95% CI] |
| --- | ---: | ---: | --- | --- |
| Six authenticated | 1 | 6.529 / 6.557 | 1.004 [0.999, 1.010] | 0.983 [0.979, 0.987] |
| Six authenticated | 4 | 6.989 / 7.101 | 1.014 [1.010, 1.017] | 0.981 [0.977, 0.985] |
| Five dense | 1 | 7.096 / 6.921 | 0.977 [0.969, 0.985] | 0.989 [0.979, 0.999] |
| Five dense | 4 | 7.655 / 7.520 | 0.981 [0.974, 0.988] | 0.988 [0.983, 0.993] |
| Whole stream | 1 | 310.761 / 318.124 | 1.032 [0.972, 1.096] | 1.003 [0.956, 1.052] |
| Whole stream | 4 | 133.110 / 135.651 | 1.038 [0.936, 1.151] | 1.054 [0.952, 1.166] |

All five dense blocks ran faster in the port comparison, with the largest gain on dense-270 (3.2% at one thread and 3.3% at four workers). Its A/A intervals include parity. The three below-threshold authenticated blocks ran slower in this binary. The aggregate A/A controls themselves ran 1.7–1.9% faster on the six and 1.1–1.2% faster on the dense five, which limits attribution of small panel differences. Both whole-stream intervals include parity.

## Per-block panel timings

The threshold selects three of the six authenticated blocks and all five dense blocks. Times are microseconds.

| Block | Retained rows | 1T master / port | 1T ratio [95% CI] | 1T A/A [95% CI] | 4W master / port | 4W ratio [95% CI] | 4W A/A [95% CI] |
| --- | ---: | ---: | --- | --- | ---: | --- | --- |
| block-0.L | 89,874 | 727.9 / 739.4 | 1.017 [1.007, 1.026] | 0.976 [0.970, 0.982] | 781.9 / 796.9 | 1.022 [1.019, 1.026] | 0.977 [0.973, 0.980] |
| block-2.L | 112,070 | 885.4 / 907.3 | 1.027 [1.024, 1.029] | 0.978 [0.975, 0.981] | 944.5 / 970.0 | 1.030 [1.027, 1.034] | 0.979 [0.975, 0.982] |
| block-85.L | 156,404 | 936.5 / 917.2 | 0.980 [0.975, 0.986] | 0.983 [0.979, 0.988] | 993.2 / 993.5 | 1.000 [0.996, 1.004] | 0.984 [0.980, 0.988] |
| block-143.L | 196,841 | 1,560.8 / 1,555.5 | 1.001 [0.992, 1.010] | 0.984 [0.977, 0.991] | 1,653.5 / 1,674.4 | 1.009 [1.004, 1.013] | 0.980 [0.975, 0.986] |
| block-210.L | 204,281 | 1,394.1 / 1,372.3 | 0.985 [0.975, 0.994] | 0.986 [0.980, 0.993] | 1,485.3 / 1,483.4 | 0.997 [0.992, 1.001] | 0.983 [0.979, 0.987] |
| block-207.L | 135,921 | 1,033.8 / 1,063.8 | 1.029 [1.026, 1.032] | 0.983 [0.980, 0.987] | 1,103.7 / 1,140.8 | 1.036 [1.033, 1.038] | 0.981 [0.977, 0.984] |
| dense-41.L | 231,600 | 1,540.2 / 1,494.8 | 0.973 [0.964, 0.982] | 0.984 [0.975, 0.993] | 1,649.5 / 1,617.8 | 0.979 [0.972, 0.986] | 0.985 [0.980, 0.989] |
| dense-56.L | 230,034 | 1,503.7 / 1,470.5 | 0.977 [0.967, 0.987] | 0.988 [0.979, 0.998] | 1,612.8 / 1,588.7 | 0.979 [0.973, 0.985] | 0.986 [0.981, 0.990] |
| dense-165.L | 221,139 | 1,521.2 / 1,489.3 | 0.983 [0.974, 0.992] | 0.986 [0.976, 0.996] | 1,622.3 / 1,609.3 | 0.988 [0.982, 0.994] | 0.985 [0.981, 0.990] |
| dense-270.L | 199,189 | 1,204.9 / 1,164.3 | 0.968 [0.958, 0.979] | 1.000 [0.984, 1.017] | 1,310.1 / 1,266.8 | 0.967 [0.961, 0.974] | 1.003 [0.993, 1.012] |
| dense-278.L | 201,418 | 1,325.5 / 1,301.7 | 0.982 [0.975, 0.989] | 0.987 [0.978, 0.997] | 1,415.5 / 1,400.0 | 0.989 [0.984, 0.994] | 0.988 [0.984, 0.992] |

## Whole corpus by block

The [431-block table](corpus-blocks.csv) has two rows per corpus index, one at each worker count. It reports median worker decode time in microseconds, port / master and A/A, each with a paired 95% interval. These measurements come from separate instrumented runs of the production reader. Each clock covers one worker's `decode_block` call from its input symbols through the CRC. Jobs follow the reader's normal scheduling. The wall times above come from the binary with these clocks compiled out.

## Method

The benchmark uses #719's in-process ABBA block harness, release profile (fat LTO, one codegen unit), rustc 1.99.1 and an AMD EPYC 9V45 with a four-CPU quota. CPU 2 ran the one-thread block panels. CPUs 0–3 ran the four-worker panels and both stream configurations. Twelve fresh processes per comparison supplied the paired observations. The one-thread measurement takes p10 of 41 decodes per arm per block after warming both kernels. Its panel aggregate sums those p10s within each round. Four workers start at staggered block offsets behind a barrier, with private kernels and output buffers. Four warmup passes precede 30 measured passes per arm. Block and pass times use the median across worker samples. The stream pairs alternate AB and BA between rounds and use two slots per worker. Absolute times are medians across rounds. Ratios use the geometric mean and a Student 95% interval on the 12 log ratios (11 degrees of freedom).

The reference source is master `53f7ecf`. Master advanced to `6996139` during the run with a one-line workstation-queue change. The decoder and benchmark inputs are identical at those revisions. The tested release implementation is the one in `4bee599`.

[Panel and stream statistics](paired.csv) · [Uninstrumented rounds and parity counts](rounds.jsonl.gz) · [Per-block instrumented rounds](block-rounds.jsonl.gz) · [Reproduction](reproduce.md)

## Exactness

- Authenticated the six column sizes, SHA-256 digests and primary indices against the admitted manifest at `41c03a5`. The five dense columns extracted from the rebuilt stream also matched #719's sizes, hashes and primary indices. On both panels the port and master matched the independent plain inverse byte for byte.
- Rebuilt all 12 Wikidata windows with libbzip2 and lbzip2 checks. The resulting 11,854,933-byte stream has SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`. Compared all 431 blocks through the port's and master's OpenCells on each block's run or column route.
- The shipped `bzcat` at 1, 4, 8 and 10 workers and `bzip2 -dc` each emitted 209,700,196 bytes with SHA-256 `3f075e30f4f8c73414150655afbf57caa2647c7927e743fc0635b4912d65cdaf`.
- The release suite passed 263 tests with three ignored. Formatting and all-target, all-feature Clippy passed with warnings denied. The explicit OpenCells moonshot arm passed all 338 reported cases, including Wikidata and the 600-mutant batch.
- Both focused tests passed before mutation. Changing the dispatch from `m >= MIN_ROWS` to `m > MIN_ROWS` failed the threshold test at exactly 140,000. Skipping the first cut-start repair (`2..n_tasks`) failed the multi-cycle byte comparison. The latter also exercises partial output, repetition, mixed inline/long words, and fallback for a singleton cycle and an oversized word.
