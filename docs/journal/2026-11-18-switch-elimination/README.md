# Reproducing Investigation 3

The study uses mibz2 0.5.5 at `f9bbd135e8e5512e9b1335e18468e1f3fd5a1d71`, Rust 1.90.0 and Python 3. The three research kernels are installed into a disposable checkout. Their normal entry point is `mibz2::decode_with`, including the scanner, Huffman, MTF/RLE2, inverse BWT, RLE1 and the block and stream CRCs. On the ordinary run route they read the capped run records. On the column route they form maximal runs from the decoded last column. [Journal](../2026-11-18-astra-investigation-3.md) · [Proofs](proof.md)

## Build and inputs

Start in a clone that contains the journal.

```sh
study="$PWD/docs/journal/2026-11-18-switch-elimination"
base=f9bbd135e8e5512e9b1335e18468e1f3fd5a1d71
tree=$(mktemp -d)/mibz2
git worktree add --detach "$tree" "$base"
sh "$study/install.sh" "$tree"
(cd "$tree" && cargo +1.90.0 build --release --locked --example switches --example switch_heap)
probe="$tree/target/release/examples/switches"
```

The Wikidata snapshot comes from the repository's corpus script. The prose control is a separately pinned stream. The frozen input is zero-based ordinal 1 of the Wikidata stream. It has primary 264045, 899,931 pre-RLE1 bytes and 900,000 final bytes.

```sh
bash bench/corpus.sh
git fetch origin 2e4b34e086ac95e8218f1ee822a4652c8db5bb19
git show FETCH_HEAD:corpora/prose.bz2 > corpora/prose.bz2
wiki="$PWD/corpora/wikidata.bz2"
prose="$PWD/corpora/prose.bz2"
python3 "$study/../2026-11-18-open-return/frozen.py" "$wiki" frozen.bz2
p=$("$probe" dump frozen.bz2 frozen.column)
```

## Correctness

```sh
(cd "$tree" && taskset -c 0-3 cargo +1.90.0 test --locked --lib -- --nocapture)
for arm in forest switches chains; do
    python3 "$study/verify.py" "$probe" frozen.bz2 "$wiki" "$prose" "$arm"
done
```

The CPU affinity in this command reproduces the four-CPU view expected by the host-sensitive topology test on this VM. The three mathematical audits enumerate all binary columns through length nine and all ternary columns through length six, with every primary. They expand the destination and the entire word of every surviving band, including bands outside the marked cycle. Wide, capped-run, fixed-point and periodic controls exercise different offsets and termination conditions. The full-stream checker compares every byte as it reads an independent libbz2 stream. It also checks concatenated streams, an empty member, and separate corruptions in the two CRC fields. [Library results](evidence/checks.txt) · [Forest](evidence/independent-forest.txt) · [Local elimination](evidence/independent.txt) · [Two-list chains](evidence/independent-chains.txt)

## Structural experiments

```sh
"$probe" forest "$wiki" > forest-wikidata.csv
"$probe" chains "$wiki" > chains-wikidata.csv
"$probe" census "$wiki" > wikidata.csv
"$probe" forest "$prose" > forest-prose.csv
"$probe" chains "$prose" > chains-prose.csv
"$probe" census "$prose" > prose.csv
"$probe" forest frozen.bz2 > forest-frozen.csv
"$probe" chains frozen.bz2 > chains-frozen.csv
python3 "$study/boundary.py" frozen.column "$p"
python3 "$study/boundary.py" --products frozen.column "$p"
python3 "$study/screens.py" frozen.column "$p"
python3 "$study/pair_screen.py" frozen.column "$p"
for index in 118 176; do
    python3 "$study/../2026-11-18-open-return/frozen.py" "$wiki" "$index.bz2" "$index"
    primary=$("$probe" dump "$index.bz2" "$index.column")
    python3 "$study/boundary.py" "$index.column" "$primary"
done
```

Each native census compares its result with a separately run dense `Plain` inverse for the same block. The Python checker independently constructs the marked maximum-weight forest, forgets all its interior intervals at the port-matching interface, runs both matching algorithms, and afterwards checks every surviving row against dense LF. The dense verifier and diagnostic column export are outside the timed decoder. The `--products` option counts the eager binary products for the same geodesics.

The forest CSVs contain a row per completed layer. A layer index of −1 records immediate interpretation of the initial system. `chords` counts cut arcs and `retained` sums their widths. `pair_tests` charges one stack match per active pair per slab. `bands` coalesces adjacent matching ordinates. `recipes` omits zero-interior returns that already have a word. `children_read`, `roots`, `copied_bytes` and `budget` belong to the **block** and repeat on each of its rows. The same convention applies to output counters in the two comparison censuses. A zero-round two-list block has a sentinel census row with `common=0`. Wikidata aggregates select the 233 blocks whose pre-RLE1 length is at least 65,536.

The native forest budgets $4n$ slab-pair visits and uses the current sparse inverse if the next layer would exceed the budget. The other two research kernels finish directly when at most 64 coordinates remain, when their system is already terminal, or on exhaustion of their respective $2n$ construction budgets. A fully straight local graph is terminal; the two-list arm also stops on a single band. These are explicit implementation choices to make adversarial columns executable. The recorded large Wikidata forest runs all reach one coordinate. [Block reports](evidence/summary.txt)

## Machine measurements

The host was an Intel Xeon Platinum 8573C VM with five visible physical cores, one hardware thread per core and cgroup quota `400000 100000`. Times use CPU 2 and the calling-thread decoder. Each lap rotates the arm order. Lap zero warms the process and is discarded. The complete-stream timer includes decoding into a returned vector, with equality checked outside the timer. The phase timer reuses each kernel and the precomputed production run records. Eager Cells is the comparator.

```sh
taskset -c 2 "$probe" price frozen.bz2 19 > frozen-time.csv
taskset -c 2 "$probe" stream "$wiki" 7 > stream-time.csv
"$tree/target/release/examples/switch_heap" frozen.bz2 > heap.csv
for arm in cells forest; do
    valgrind --tool=cachegrind --cache-sim=yes --I1=32768,8,64 --D1=32768,8,64 \
      --LL=20971520,20,64 --cachegrind-out-file="$arm.cg" \
      "$probe" write frozen.bz2 "$arm" "$arm.raw"
    bzip2 -dc frozen.bz2 | cmp - "$arm.raw"
done
python3 "$study/summarise.py" "$study/evidence"
```

The standalone allocator records the peak additional requested bytes above a baseline containing compressed input and the independent expected output. It counts a fresh complete call and includes its returned output. Cachegrind 3.24 records data accesses and simulated misses under the specified geometry. The study contains no hardware DRAM measurement or target-machine SMT trial. [Allocation counts](evidence/heap.csv) · [Simulated counts](evidence/cache.csv) · [Paired timing rounds](evidence/stream-time.csv)
