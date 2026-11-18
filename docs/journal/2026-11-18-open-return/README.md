# Reproducing the open-return investigation

[Journal](../2026-11-18-astra-investigation-2.md) · [Proof](proof.md) · [Observations](evidence/)

The files here install research examples into a disposable checkout of mibz2 0.5.5. The input to every timed decoder is a bzip2 stream. The experimental kernels read the ordinary capped MTF run records on the production run route and inherit the Segmented fallback on the column route. Forced censuses use the expanded column to compare each block with the independent Plain implementation. They include blocks outside production run admission.

## Build and check

The commands use Rust 1.90.0, Python 3 and the repository’s ordinary build dependencies. Start in a checkout containing this journal.

```sh
study="$PWD/docs/journal/2026-11-18-open-return"
tree=$(mktemp -d)/mibz2
git worktree add --detach "$tree" dbe9b40687c8531e11d4d6745523262dea2c35e2
sh "$study/install.sh" "$tree"
cd "$tree"
export RUSTUP_TOOLCHAIN=1.90.0
cargo build --release --locked --example open_return --example return_heap
cargo test --release --locked --lib
probe=target/release/examples/open_return
"$probe" small
```

The measurement host exposes five cores under a four-CPU quota. Its existing Linux topology assertion passes with the suite bound to four cores, using `taskset -c 0-3 cargo test --release --locked --lib`.

For the input files, `bench/corpus.sh` uses the published Wikidata windows and the normal Git, network and `lbzip2` prerequisites. The prose snapshot used here is pinned separately.

```sh
bash bench/corpus.sh
git fetch --depth=1 origin 2e4b34e086ac95e8218f1ee822a4652c8db5bb19
git show FETCH_HEAD:corpora/prose.bz2 > corpora/prose.bz2
python3 "$study/frozen.py" corpora/wikidata.bz2 frozen.bz2
python3 "$study/verify.py" "$probe" frozen.bz2 corpora/wikidata.bz2 corpora/prose.bz2
p=$("$probe" dump frozen.bz2 frozen.column)
python3 "$study/screen.py" frozen.column "$p"
```

`verify.py` compares byte length and SHA-256 with Python's libbz2 on complete streams. It also mutates the block CRC and the combined CRC independently. The small column enumeration fixes every possible primary, including marks on short cycles. The arrow tests inspect each retained row, including those outside the marked component. [Verification](evidence/checks.txt)

## Constructors and arms

| Arm | Construction and emission |
|---|---|
| `cells`, `lazy` | The release's eager Cells and widened LazyCells kernels. |
| `16` | Trace one representative per open band. Rescan narrow run records to compose the residual table. Numeric arms 8, 32 and 64 change the threshold. |
| `i16` | Eliminate the wide complement by endpoint induction. Empty terminal arrows cost no word rule. Materialise the band dictionary with memoised rule expansion. Prefix `i` also supports thresholds 2, 4, 8 and 32. |
| `q16`, `qi16` | Queue retained row indices per byte during the initial run pass. A band projects a consecutive slice of one queue into the result. `q16` traces, and `qi16` uses the open endpoint constructor. |
| `s16` | Trace both colours from the interface and emit one word per switch row. Also available at 8, 32 and 64. |
| `ranked` | The queued endpoint, with eight independent residual walks cut at ranks congruent to the primary modulo 128. Stages word codes and expands the ranked segments in text order. |

The one-sided construction coalesces capped fragments into their true runs. A primary in a wide strip is retained as one row and its two sides inherit their parent's classification. The switch construction additionally includes the primary in its interface. The width sweep also exposes queued endpoint arms `qi8`, `qi24` and `qi32`.

## Structural evidence

```sh
mkdir -p raw output
for corpus in wikidata prose; do
    file="corpora/$corpus.bz2"
    "$probe" census "$file" 8,16,32,64 > "raw/$corpus-census.csv"
    "$probe" induced "$file" 2,4,8,16,32 > "raw/$corpus-induced.csv"
    "$probe" switches "$file" 16,32 > "raw/$corpus-switches.csv"
done
for mode in census induced switches; do
    "$probe" "$mode" frozen.bz2 1,2,4,8,12,16,24,32,64,128,256 > "raw/frozen-$mode.csv"
done
python3 "$study/evidence.py" counts raw output
for name in blocks widths frozen; do
    cmp "output/$name.csv" "$study/evidence/$name.csv"
done
```

`blocks.csv` gives the exact width-16 counts for every block. `ordinal` starts at zero over the concatenated file. `run_front` records production frontend admission, independently of either Cells kernel's internal admission. `H` is the number of complementary strips after marking, `m` is the number of retained rows, `B` counts all band instructions, `A` is the represented wide area and `U` sums the representative heights. `h` counts the nonempty open-path occurrences, or one direction of the unmarked colour cut, for the classification after the primary split. `unit_work` counts wide letters traced in width-one bundles. The induced fields additionally count live input nodes, elimination steps and binary rules. `widths.csv` gives min, median and max of per-block fractions for blocks with at least 65,536 rows. `frozen.csv` keeps each constructor's distinct byte-cut convention.

## Timing, allocation and traffic

The calling-thread arm specifies zero workers. Positive counts use the normal stream pool with four slots per worker. Each complete-stream benchmark loads compressed input before the timer, includes the scanner, inverse, RLE1, CRC and returned output, and compares the result after the timer. Arms rotate with each round. Round zero warms all arms. An output byte is a final decompressed byte. Frozen repeats use a fresh complete decoder per repeat. The inverse-only `price` command prepares run records outside its timer and reuses each kernel's allocated scratch.

```sh
taskset -c 2 "$probe" bench corpora/wikidata.bz2 cells,qi16,lazy 11 0 1 > calling.csv
taskset -c 0-1 "$probe" bench corpora/wikidata.bz2 cells,qi16,lazy 9 2 1 > two.csv
taskset -c 0-3 "$probe" bench corpora/wikidata.bz2 cells,qi16,lazy 11 4 1 > four.csv
taskset -c 2 "$probe" bench corpora/prose.bz2 cells,qi16,lazy 9 0 1 > prose.csv
python3 "$study/evidence.py" timing calling.csv two.csv four.csv prose.csv
taskset -c 2 "$probe" price frozen.bz2 51 > phases-frozen.csv
taskset -c 2 "$probe" price corpora/wikidata.bz2 2 > phases-wikidata.csv
target/release/examples/return_heap frozen.bz2
```

`timings.csv` includes the raw rounds from these runs and the broader variants and frozen width trials, labelled by experiment. The frozen phase file contains all laps. The per-block phase file summarises the two warmed laps on each admitted block. A Cells phase row's first field is its entire inversion. The open arms report construction and writing separately. Times are native wall times on the host described in the journal. The heap ledger uses a separate executable and a counting allocator, with the compressed input and comparison output established before each measurement.

For the single-process cache simulation, the study used Cachegrind 3.24, a 32 KiB eight-way instruction cache and data cache, and a 20 MiB twenty-way last-level cache, all with 64-byte lines.

```sh
mkdir -p cg
for arm in cells lazy 16 i16 q16 qi16 s16; do
    valgrind --tool=cachegrind --cache-sim=yes --I1=32768,8,64 --D1=32768,8,64 \
        --LL=20971520,20,64 --cachegrind-out-file="cg/$arm.cg" \
        "$probe" write frozen.bz2 "$arm" trial.out
    bzip2 -dc frozen.bz2 | cmp - trial.out
done
for idx in 118 176; do
    python3 "$study/frozen.py" corpora/wikidata.bz2 "$idx.bz2" "$idx"
    for arm in cells qi16; do
        valgrind --tool=cachegrind --cache-sim=yes --I1=32768,8,64 --D1=32768,8,64 \
            --LL=20971520,20,64 --cachegrind-out-file="cg/$idx-$arm.cg" \
            "$probe" write "$idx.bz2" "$arm" trial.out
        bzip2 -dc "$idx.bz2" | cmp - trial.out
    done
done
python3 "$study/evidence.py" cache cg cache.csv
```

`Dr` and `Dw` count data reads and writes. `D1mr + D1mw` counts modelled L1 data misses. `DLmr + DLmw` counts modelled last-level data misses. Binary layout, the host and library versions can affect the absolute counters. The committed values are observations from these builds.
