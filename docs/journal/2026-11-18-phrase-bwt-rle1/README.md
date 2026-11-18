# Reproducing the mibz2 phrase experiments

The native patch adds two research modules and example drivers to mibz2 at
f396ccc7b3d4290468b12e4ccdfcfcfe7708f3b2. The standard entry points retain the shipped decoder.
The evidence directory contains the recorded counts and elapsed times. The Python reference uses the standard library.
[Journal entry](../2026-11-18-phrase-bwt-rle1.md) · [Raw measurements](evidence/)

## Native code and the pinned corpus

Start at the repository root on the journal branch. Set RESEARCH to the companion directory before
creating a clean worktree at the tested revision. The measurement VM had rustc 1.90.0. The
checkout pins 1.99.1 for the project's own release measurements. The following commands use the
experimental compiler explicitly. The corpus recipe also uses Python, git and lbzip2.

```sh
export RESEARCH="$PWD/docs/journal/2026-11-18-phrase-bwt-rle1"
git worktree add --detach ../mibz2-phrase-experiment f396ccc7b3d4290468b12e4ccdfcfcfe7708f3b2
cd ../mibz2-phrase-experiment
git apply "$RESEARCH/phrase-bwt.diff.txt"
export RUSTUP_TOOLCHAIN=1.90.0
bash bench/corpus.sh
cargo build --release --example phrase_probe --example phrase_memory --example phrase_workers --example phrase_finish --example phrase_lexicon
target/release/examples/phrase_probe check
target/release/examples/phrase_probe stats corpora/wikidata.bz2 > grouped-counts.csv
target/release/examples/phrase_probe recurse-stats corpora/wikidata.bz2 > recursive-counts.csv
target/release/examples/phrase_probe fused-save corpora/wikidata.bz2 representative.bin
target/release/examples/phrase_probe grouped-save corpora/wikidata.bz2 decoded.bin
cmp representative.bin decoded.bin
bzip2 -dc corpora/wikidata.bz2 | cmp - decoded.bin
sha256sum decoded.bin
```

The expected output SHA-256 is 3f075e30f4f8c73414150655afbf57caa2647c7927e743fc0635b4912d65cdaf.
The corpus contains 209,700,196 decoded bytes. The first-level marker is byte 34. Recursive levels
select the most frequent token of frequency at most half the current size. Equal frequencies favor
the larger rank. A missing marker on the selected cycle leaves the program at its current level.

Run `python3 "$RESEARCH/summarize.py"` to regenerate the reported counts and medians from the
checked-in CSVs. The standard input SHA, output SHA and native checks are recorded in
[checks.txt](evidence/checks.txt).

## Rates and allocations

Choose an available CPU for the single-core comparisons. The timings in the note used CPU 2. The
worker experiment used CPUs 0 through 3 and scheduled the twelve different original streams across
workers. All outputs are compared after timing. Times include the normal scanner and CRC work. The
one-core run uses one concatenated input and one contiguous returned vector. In the worker
experiment each original stream has a separate output. Compare arms within each experiment.

```sh
python3 "$RESEARCH/prepare_streams.py" corpora/wikidata.bz2 scratch-phrase
taskset -c 2 target/release/examples/phrase_probe bench corpora/wikidata.bz2 7 cells,lazy,grouped,fused2
taskset -c 0-3 target/release/examples/phrase_workers scratch-phrase/streams 5
target/release/examples/phrase_probe corrupt scratch-phrase/frozen.bz2
target/release/examples/phrase_memory corpora/wikidata.bz2 blocks > memory-blocks.csv
target/release/examples/phrase_lexicon corpora/wikidata.bz2 scratch-phrase/raw-large.bin
python3 "$RESEARCH/holdout.py" scratch-phrase/raw-large.bin > holdout.csv
python3 "$RESEARCH/rle_census.py" scratch-phrase/raw-large.bin > edits.csv
taskset -c 2 target/release/examples/phrase_finish scratch-phrase/raw-large.bin 8 15
```

The rows in a single-core whole-corpus timing file are method, elapsed seconds and output bytes.
The first row warms Cells. The first row is excluded from the seven rotated rounds. The grouped
arm groups equal suffix contexts in its first pass, factors a second token BWT, and emits from
decoded dictionaries. The fused2 arm builds both levels by walking one representative per
phrase-BWT run and emits first-level decoded words. The r2 and r4 arms keep ordinary RLE finishing
and exercise the structural recursion. The phrase arm is a one-level dense delimiter-return
implementation. In the allocation driver the fused flag means adoption of phrase-level RLE for the
two fused arms. The allocator counts live requested Rust heap sizes above the start of a fresh
single-block decode. The input and stored comparison output precede that start. The returned
vector, scanner and decoder scratch are included. Reallocation is accounted at the requested size.
This is distinct from retained worker memory, RSS and DRAM traffic.

The writer-only experiment constructs a shared linear parse from plain inverse bytes before
timing. Its timed arms start from that dictionary and token order. They include CRC, dictionary
finishing as needed, and byte emission. Time spent finding the parse belongs to the whole-decoder
measures. The holdout and RLE censuses also use plain inverse bytes. The native compressed-input
constructors do not read these oracle files.

## Independent check and historical frozen block

```sh
python3 "$RESEARCH/verify_phrase_bwt.py"
git fetch --depth=1 origin bf88365ba80388e17488a31c80d2f3f250b19fb2
git show bf88365ba80388e17488a31c80d2f3f250b19fb2:docs/journal/2026-11-11-first-return-block/evidence/column.bin.gz | gzip -dc > scratch-phrase/column.bin
git show bf88365ba80388e17488a31c80d2f3f250b19fb2:docs/journal/2026-11-11-first-return-block/evidence/baseline-rle1.bin.gz | gzip -dc > scratch-phrase/inverse.bin
python3 "$RESEARCH/verify_phrase_bwt.py" scratch-phrase/column.bin 264045 34 scratch-phrase/inverse.bin
```

The independent Python check also runs directly from the journal checkout with no corpus or Rust
build. For the repository’s existing suite this VM used `taskset -c 0-3 cargo test --release --lib
--tests`. Restricting affinity to the four-CPU quota reconciles the reported five-core topology
with the parallel-policy test. There were 208 passes and two existing ignored tests.

The Python program checks source run widths, each affine image against an independent stable sort,
marked expansion, direct token BWTs, regeneration of the original column and recovery from
branching buckets alone given a phrase multiset. The Rust checks include arbitrary last columns
and every primary and marker in the enumerated alphabets.

## Controls and simulated cache traffic

```sh
python3 "$RESEARCH/controls.py" scratch-phrase/controls
git fetch --depth=1 origin data/induction-records
git show FETCH_HEAD:corpora/prose.bz2 > scratch-phrase/prose.bz2
taskset -c 2 target/release/examples/phrase_probe bench scratch-phrase/prose.bz2 7 cells,lazy,grouped
valgrind --tool=cachegrind --cache-sim=yes --I1=32768,8,64 --D1=32768,8,64 --LL=20971520,20,64 --cachegrind-out-file=cg.out target/release/examples/phrase_probe grouped scratch-phrase/frozen.bz2
```

Cachegrind 3.24.0 counted the entire single-block command including startup. The results include
the same run with cells, lazy and fused2, and an additional simulated 2 MiB 16-way last-level
cache. The memory-controller figures for the i9 in the journal entry come from mibz2's published
hardware measurements. The VM reported one thread per core, five visible CPUs and a four-CPU
quota.
