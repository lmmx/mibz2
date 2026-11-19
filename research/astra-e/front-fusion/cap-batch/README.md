# Reconstruct the constructor's maximal runs in batches

A maximal run in the stock run front is split into a sequence of 127-row cap records plus a final shorter cap, or one or more full caps if its length is divisible by 127. From a known run start, every record with low byte below 127 is therefore an entire maximal run. In any window starting at a run boundary, all records before its first full cap can be visited individually without probing their neighbors. At the first full cap, read subsequent records until the label changes, then restart the classification exactly at the next run boundary. This preserves marked thick splitting and each queue's stable order. The induction starts at record zero. Its cost is one vector classification per window plus one scalar coalescence per full-cap-starting run.

The 233 large edge-admitted real blocks contain 16,320,934 capped records for 15,141,602 maximal runs. The windows discharged 14,968,212 complete sub-127 runs directly (98.85%). Of these, 14,089,904 were in 880,619 full 16-record groups. The scalar coalescer handled 173,390 full-cap run starts. A conservative mark test using a maximum width of 126 could require individual marked-thick splitting on at most 27,451 sub-127 records (0.1834%). Compare against the original marked record walk and a scalar one-record shortcut. Charge the stock front, residual construction, writer and RLE1 on each block. [Exact traversal census](census.csv) · [Census source](census.rs)

Four complete block workers (frozen edge, identical frozen edge with independent scratch, scalar first-record shortcut, grouped AVX2 shortcut) ran in 24 balanced rotations, with CPU 3 and niceness 10. Every call included the normal decode_block input route, stock MTF, marked inverse, RLE1, block CRC and exact fallback. Parsed symbols are the input to this test. Every arm's decoded bytes matched on all 431 real blocks, and 11,520 boundary cases matched the original emitted marked geometry. The extra dense blocks and all short and column routes entered once, with nine ordinary blocks reweighted by 228/9. Scalar's weighted wall-independent thread-CPU ratio was 0.9625 [0.9175,1.0097] over frozen edge, grouped's was 0.9764 [0.9352,1.0194], and same-code was 1.0013 [0.9510,1.0542]. Neither interval establishes a complete-worker improvement. [All measurements](panel.csv) · [Control and route intervals](panel.txt) · [Checked corpus](panel.stderr) · [Boundary results](boundary.txt)

```sh
d=research/astra-e/front-fusion/cap-batch
scratch=$(mktemp -d)
cargo build --release --locked --manifest-path bench/moonshot/Cargo.toml --target-dir "$scratch/build"
deps="$scratch/build/release/deps"
python3 "$d/prepare.py" "$scratch/source"
rustc -O --edition 2021 -A unused -C codegen-units=1 "$scratch/source/main.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$scratch/panel"
"$scratch/panel" boundaries
nice -n 10 taskset -c 3 "$scratch/panel" corpora/wikidata.bz2 24 > "$d/panel.csv"
python3 "$d/summarize.py"
```
