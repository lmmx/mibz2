# Reproduce the adjacent-rank experiment

The directory contains the Rust implementation, input generators and observations. The frozen block is reproducible from the repository’s published Wikidata windows. [Journal entry](../2026-11-18-adjacent-rank-gaps.md) · [Proof](proof.md) The examples use a disposable checkout of mibz2 0.5.4. All commands below start from the repository root or continue in the worktree established by the first command block.

```sh
study="$PWD/docs/journal/2026-11-18-adjacent-rank-gaps"
tree=$(mktemp -d)/mibz2
git worktree add --detach "$tree" 111ff9ac70f25c3b59adb2fb9c51c29dbf2723dd
sh "$study/install.sh" "$tree"
cd "$tree"
export RUSTUP_TOOLCHAIN=1.90.0
cargo build --release --locked --example extrema_probe --example extrema_memory --example support_price
probe=target/release/examples/extrema_probe
python3 "$study/controls.py" family
"$probe" write family/power-218750.bz2 direct primitive.out
bzip2 -dc family/power-218750.bz2 | cmp - primitive.out
bash bench/corpus.sh
python3 "$study/../2026-11-18-phrase-bwt-rle1/prepare_streams.py" corpora/wikidata.bz2 reference
frozen=reference/frozen.bz2
"$probe" extrema "$frozen"
"$probe" write "$frozen" direct frozen.out
bzip2 -dc "$frozen" | cmp - frozen.out
sha256sum frozen.out
```

The five-run primitive example above needs Python and the Rust build. Rebuilding the Wikidata corpus also uses the repository’s normal `lbzip2`, Git and network prerequisites. The expected frozen output digest is `d2584c09f1d643d6abff8a25513b63af4e05969c15c30256aed3a05c74281210`. The 41,618-byte input is the independently wrapped first large block from the pinned corpus. The previous entry's `prepare_streams.py` reproduces it exactly. The experiment used rustc 1.90.0, the release profile in the repository and no added Rust dependencies.

## Implementations and checks

- `extrema.rs` contains both ways into the sparse inverse. The `direct` arm reads the production scanner's entropy symbols and emits maximal runs. The `extrema` arm implements `InverseBwt` on the normal capped run records, with the production Segmented column fallback. Both use the same contraction and grammar writer. The direct arm is available on the calling thread. The run-record arm also works in the regular worker pool.
- `valleys.rs` independently evaluates the local-minimum recurrence on dense arrays of endpoint and distance. The `valley` arm walks the reduced cycle once. The `recursive` arm reduces the weighted permutation through singleton cycles. Both recover the output by coordinates. They retain the original rank order, including for a primary in the middle of an excursion.
- `forest.rs` is the short-context pilot. It constructs the context intervals from run counts, chooses a majority predecessor, breaks each default circuit and resolves exceptional returns in reverse topological order. The implementation cuts an additional default edge when its path would exceed 64 bytes. The `census` command forces this experiment through every block and compares the result with Plain.
- `support_price.rs` runs the earlier SUP1 encoder followed by its independent support-only inverse inside the complete stream decoder. The installer copies the published source and appends a small entry point that calls its existing routines. This comparison includes the encoder's weighted bookkeeping and self-check. It omits the additional inverse and ordinary-column comparisons made by the earlier census driver.

```sh
cargo test --release --locked --lib extrema::tests -- --nocapture
"$probe" small
"$probe" corrupt "$frozen"
"$probe" valleys "$frozen"
"$probe" census "$frozen" 1,2,3,4,6,8
target/release/examples/support_price "$frozen" 5
```

The sparse test suite compares marked output against Plain on every primary of the enumerated arbitrary byte columns. Its interval audit expands each surviving row at each round against the original permutation, checking the destination, exact word, boundary certificate, halving and all-level interval bound. The second audit covers arbitrary ordered permutations and intervals with unequal widths. The direct MTF test compares 15,625 symbol sequences with the production scalar MTF inverse. The checksum test changes the block CRC and the combined stream CRC separately on both sparse entrances. [Recorded check results](evidence/checks.txt)

## Full corpora

The first command block rebuilt the pinned Wikidata corpus. The measured prose file is also pinned by SHA-256 in the checks. The `extrema` census always applies the sparse algorithm and independently compares the pre-RLE bytes against Plain. The `admitted` field records the production run-frontend predicate. Timed arm `extrema` honours it. Timed arm `direct` processes every block through the sparse algorithm.

```sh
git fetch --depth=1 origin 2e4b34e086ac95e8218f1ee822a4652c8db5bb19
git show FETCH_HEAD:corpora/prose.bz2 > corpora/prose.bz2
for corpus in wikidata prose; do
    "$probe" extrema "corpora/$corpus.bz2" > "$corpus-rounds.csv"
    python3 "$study/summarize.py" collapse "$corpus-rounds.csv" > "$corpus.csv"
    cmp "$corpus.csv" "$study/evidence/$corpus.csv"
    "$probe" write "corpora/$corpus.bz2" direct "$corpus.out"
    bzip2 -dc "corpora/$corpus.bz2" | cmp - "$corpus.out"
    sha256sum "$corpus.out"
done
```

## Time, memory and controls

The benchmark reads compressed input before timing. It includes the scanner, run preparation, inverse, RLE1, CRC and output allocation. It compares the result with the reference outside the timed interval. Arms rotate in order. Round zero warms them and the published summaries exclude it. Workers zero uses the calling-thread decoder. A positive count uses the normal stream decoder with that many workers and four slots per worker. The two entry points have different stream overheads. Comparisons in the table are within one entry point.

```sh
taskset -c 2 "$probe" bench "$frozen" 13 0 cells,lazy,segmented,valley,recursive,extrema,direct
taskset -c 2 "$probe" bench corpora/wikidata.bz2 5 0 cells,lazy,valley,extrema,direct
taskset -c 2 "$probe" bench corpora/prose.bz2 7 0 cells,valley,extrema,direct
for workers in 1 2 4; do
    taskset -c 0-3 "$probe" bench corpora/wikidata.bz2 4 "$workers" cells,lazy,extrema
done
taskset -c 2 target/release/examples/support_price "$frozen" 5
python3 "$study/controls.py" family
for input in family/*.bz2; do
    "$probe" extrema "$input"
    "$probe" valleys "$input"
    taskset -c 2 "$probe" bench "$input" 17 0 cells,forced,segmented,extrema,direct
done
python3 "$study/../2026-11-18-phrase-bwt-rle1/controls.py" phrase-controls
for name in noquote lex16 lex128 lex1024 random; do
    taskset -c 2 "$probe" bench "phrase-controls/$name.bz2" 7 0 cells,segmented,extrema
done
python3 "$study/summarize.py" medians "$study/evidence/timing.csv"
```

Each row of `family.csv` contains one input's compressed hash, number of maximal runs and grammar counts. The input label in `heap.csv` and `cachegrind.csv` called `primitive` refers to `family/power-218750.bz2`. The heap ledger measures additional peak live requested Rust allocation during one fresh, complete decode. The input and comparison output already exist when the counter starts. It includes the returned output and treats reallocations at their requested size. Cachegrind 3.24.0 measures the entire one-block process with the same simulated cache geometry for every arm.

```sh
for input in "$frozen" family/power-218750.bz2; do
    target/release/examples/extrema_memory "$input"
    for arm in cells extrema direct; do
        valgrind --tool=cachegrind --cache-sim=yes --I1=32768,8,64 --D1=32768,8,64 \
            --LL=20971520,20,64 --cachegrind-out-file=cg.out "$probe" write "$input" "$arm" /dev/null
    done
done
```

The host was a Xeon Platinum 8573C with five visible CPUs, one thread per core and a four-CPU quota. CPU 2 was used for the serial timing series. The worker series used CPUs 0 through 3. Hardware DRAM traffic and sibling-thread scaling were outside the measurements available here.
