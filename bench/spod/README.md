# SPOD-Coset512 on idle workers

Direct manual assignment. The comparison base is mibz2 0.5.11, master at `8ff1187`. The research source is Kitchen Sink cycle 10 at `62e3507`, Theorem 10.2, its Python probe and results. The probe assigns unit cost to a retained hop. Its modeled speedups of 3.9–15.3× cover 4–16 lanes.

**No end-to-end gain over master workers on the tested workload.** One Wikidata block engaged SPOD in all 32 candidate runs. It took 29–52% longer than workers and 70–176% more process CPU. Its wall times against cooperative teams were within noise at every budget. Two or three blocks sometimes beat teams, but master workers already beat teams in those cases. The evidence supports keeping SPOD opt-in.

## Decoder

Build with `--features spod` and select `StreamDecoder::spod_workers` or `BZ2_INVERSE=spod` in bzcat. The reference worker plan uses Cells below eight workers and OpenCells from eight. The candidate uses that same plan, with one extra opportunity after the MTF run front. A column must pass the existing Open count and construction gates. Text fell back throughout this experiment.

After scanner EOF, the number of issued but untaken blocks can only fall. It includes queued, decoding and ready-but-unread blocks. If this number is B with 0 < B < T, one worker may try-lock a decoder-local Open scratch and use up to T − B scoped helpers plus itself. Another block never waits for that lock. There is no speculation during startup or an input stall. A block that reaches the check before EOF continues on the usual kernel. The scoped threads borrow CPU capacity, and the stream's original worker threads may still exist as OS threads. The default constructor and all existing team plans retain their dispatch.

The new walk uses the active prefix of the retained-edge arena. It dynamically claims seeds divisible by 512 and adds the primary once if necessary. Each walk spells into its own byte vector, including wide codes and words longer than 16 bytes. Once all walks finish, the caller follows successor seeds from the primary, copies each piece, and repeats a short primary orbit to the block length. This implementation includes a serial final copy. Pieces from unrelated permutation cycles are discarded. Elimination, word inlining and the existing emit function have no edits. The candidate calls the existing capped construction and inline table builder directly.

## Measured stream results

AMD EPYC 9V45, Rust 1.99.1, release fat LTO, Linux, pinned to CPUs 0–3 under a four-CPU quota. Eight and ten are requested worker budgets on these four cores. Each of the eight rounds includes two invocations of the same master workers binary (A/A), master cooperative teams of two, and the candidate. All use four slots per requested thread. The four-arm order rotates across rounds and the case order reverses. Every invocation uses a fresh process and the same file, flags and driver. Wall time covers construction of the streaming decoder through joining it, with the compressed input already resident. CPU is whole-process user plus system time from wait4. RSS is the child's VmHWM.

Each cell below is candidate wall time divided by master worker / team wall time, from paired log ratios. Smaller values mean less elapsed time. The final row times from delivery of block 428 to EOF and join in one continuous decode of the 431-block file. The reader never pauses at the boundary. The raw records also contain whole-file wall, CPU and RSS for these runs.

| Input | 2 threads | 4 threads | 8 threads | 10 threads |
|---|---:|---:|---:|---:|
| Wikidata, 1 block | 1.52 / 1.05 | 1.43 / 0.99 | 1.33 / 0.88 | 1.29 / 1.00 |
| Wikidata, 2 blocks | 1.02 / 0.59 | 1.29 / 0.85 | 1.29 / 0.82 | 1.30 / 0.82 |
| Wikidata, 3 blocks | 1.03 / 0.65 | 1.12 / 0.73 | 0.99 / 0.62 | 1.29 / 0.68 |
| Text, 1 block | 0.76 / 1.07 | 1.08 / 1.11 | 1.07 / 1.10 | 0.95 / 1.08 |
| Text, 2 blocks | 0.83 / 0.78 | 1.26 / 1.21 | 0.87 / 1.12 | 0.96 / 1.11 |
| Text, 3 blocks | 1.11 / 0.81 | 1.20 / 1.16 | 1.04 / 1.11 | 1.00 / 1.19 |
| Wikidata, final 3 blocks | 0.79 / 0.50 | 1.11 / 1.23 | 0.70 / 0.77 | 1.17 / 1.11 |

For the one-block Wikidata input, the following are absolute medians across rounds in the order workers / teams / SPOD.

| Threads | Wall ms | CPU ms | Peak RSS MiB |
|---:|---:|---:|---:|
| 2 | 2.12 / 2.99 / 3.08 | 1.51 / 4.89 / 3.91 | 4.96 / 8.92 / 5.07 |
| 4 | 2.14 / 2.92 / 3.05 | 2.05 / 4.71 / 4.87 | 4.98 / 8.97 / 5.13 |
| 8 | 2.37 / 3.09 / 3.33 | 3.02 / 4.94 / 5.59 | 4.02 / 9.03 / 5.19 |
| 10 | 2.63 / 3.35 / 3.36 | 3.29 / 5.29 / 5.59 | 4.06 / 9.04 / 5.30 |

Against workers, the single-block wall ratios have 95% intervals of [1.39, 1.66], [1.33, 1.55], [1.15, 1.54] and [1.16, 1.44]. Peak RSS grew 29–30% at 8 and 10 workers. Teams used about 9 MiB at all four budgets. On the natural tail, SPOD ran in 0, 0, 3 and 3 of the eight candidate rounds at the respective budgets. Tail wall intervals are wide. Even the 8-thread A/A ratio was 0.31 [0.12, 0.78]. The 4-thread one-block text A/A also excluded unity. These cells do not establish a speedup. Tail CPU samples have 10 ms tick resolution, coarser than most tail intervals. Whole-process CPU and peak RSS are comparable across all arms.

[All 896 process measurements](results/paired.csv) · [Input sizes and SHA-256](results/inputs.json)

## Isolated Open emit

The two serial arms call the existing emit on the same constructed table as SPOD. All three reuse the output vector and validate against Plain after each timed call. The SPOD interval includes scoped thread creation, segment allocation, the walks and the final splice. Construction and word inlining are outside the interval. The entries below are paired geometric mean speedups (serial time divided by SPOD time). Above one favors SPOD.

| Manifest column | 2 lanes | 4 lanes | 8 lanes | 10 lanes |
|---|---:|---:|---:|---:|
| block-0 | 0.22× | 0.26× | 0.24× | 0.20× |
| block-2 | 0.56× | 0.49× | 0.51× | 0.51× |
| block-85 | 0.50× | 0.66× | 0.65× | 0.51× |
| block-143 | 0.41× | 0.59× | 0.73× | 0.60× |
| block-210 | 0.82× | 1.26× | 1.33× | 1.08× |
| block-207 | 0.84× | 1.07× | 1.03× | 0.72× |

Only block-210's four- and eight-lane speedups excluded 1.0 in the favorable direction, respectively 1.26× [1.09, 1.46] and 1.33× [1.17, 1.51]. The corresponding serial A/A ratios were 0.97 [0.91, 1.03] and 1.00 [0.97, 1.02]. A hop-count makespan omits word expansion, thread and allocation overhead, copying and the cost of the rest of the decoder. This experiment does not fit a dispatch threshold to those six columns.

[All 576 emit measurements](results/emit.csv)

## Reproduction

Run from the repository root on Linux with cargo, python3, bzip2, bzip2recover and lbzip2. Set the affinity to CPUs available on the host. Use a fresh destination for each result. The Wikidata inputs use original compressed blocks 1–3 (zero-based) of the pinned corpus. Text uses 0.8, 1.6 and 2.4 MB of the base revision's source corpus, compressed at level 9. Preparation counts the blocks and hashes bzip2's output. All inputs and outputs are shared between arms.

```sh
bash bench/moonshot/corpus.sh
git fetch origin data/admitted-bwt:refs/remotes/origin/data/admitted-bwt
mkdir -p /tmp/spod
git archive origin/data/admitted-bwt admitted-bwt | tar -x -C /tmp/spod
git worktree add --detach /tmp/spod/base 8ff1187
cp examples/spod_driver.rs /tmp/spod/base/examples/
(cd /tmp/spod/base && cargo build --locked --release --example spod_driver)
cargo build --locked --release --features spod --example spod_driver --example bzcat
python3 bench/spod/bench.py prepare /tmp/spod/inputs corpora/wikidata.bz2
python3 bench/spod/bench.py check /tmp/spod/inputs target/release/examples/bzcat /tmp/spod/admitted-bwt
taskset -c 0-3 python3 bench/spod/bench.py run /tmp/spod/inputs /tmp/spod/base/target/release/examples/spod_driver target/release/examples/spod_driver /tmp/spod/paired.csv
python3 bench/spod/bench.py report /tmp/spod/paired.csv
taskset -c 0-3 python3 bench/spod/bench.py emit /tmp/spod/admitted-bwt /tmp/spod/emit.csv
python3 bench/spod/bench.py emit_report /tmp/spod/emit.csv
```

The report gives candidate/workers, candidate/teams and workers/workers for wall time, CPU and RSS at every cell. Each 95% interval uses the mean paired log ratio ± 2.365 standard errors (seven degrees of freedom), exponentiated as in PR #692.

```sh
MOONSHOT_CANDIDATE=bench/moonshot/candidates/spod.rs cargo build --release --locked --manifest-path bench/moonshot/Cargo.toml --features mibz2/spod
bench/moonshot/target/release/moonshot check --arm candidate --wikidata corpora/wikidata.bz2
taskset -c 0-3 cargo test --release --locked
taskset -c 0-3 cargo test --release --locked --features spod
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --all-targets --all-features -- -D warnings
```

## Exactness and negative controls

The moonshot candidate forces a four-lane SPOD inversion of every column, including all 431 Wikidata blocks and arbitrary multi-cycle columns. The gate compares bytes with libbzip2, verifies block and stream CRCs and compares corrupt-input behavior. The separate authenticated test checks six raw columns against Plain using the manifest's primary indices. The Python driver verifies each column's length, maximal run count and SHA-256 before invoking it. The stream checks compare both default and SPOD bzcat against bzip2 on all seven files at 1, 4, 8 and 10 threads (56 comparisons). The whole-file SHA-256 is `3f075e30f4f8c73414150655afbf57caa2647c7927e743fc0635b4912d65cdaf`.

Two hand edits in a disposable checkout were tested separately, with the original restored between edits.

| Edit in src/open/spod.rs | Release test that failed |
|---|---|
| Remove the stop condition `\|\| row == self.mark` | `sub_stride_periodic_and_disjoint_cycles` revisited a non-primary seed while stitching |
| Substitute `slot = first` for `slot = piece.next` | `splice_permuted_segments_and_primary_seed` found a byte mismatch for mark 0 with 2,053 retained rows |

The unpinned host exposes five physical cores in its affinity mask and four CPUs through Rust's quota-aware available_parallelism. Master's existing `the_policy_here` assertion failed on that mismatch. The release suite and timings use the four-CPU affinity above.
