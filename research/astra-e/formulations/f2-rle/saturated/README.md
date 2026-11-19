# Saturating dangerous positions by the original return partition

Keep the original width-16 Open section S and the primary in it. It partitions every cycle meeting S into intervals of visits between consecutive S positions. For each protected byte coordinate x, rewind by LF to its first preceding member s of S, with zero steps when x already lies in S. Label the original outgoing edge of s. The union of those return words is precisely the least union of whole S-return words containing the protected positions on all represented cycles. This follows because each such position lies in one and only one return interval. LF is a permutation, so a rewind closing at x before meeting S belongs to a closed component disjoint from the marked one. These components do not affect output.

The constructor takes this saturated superset as its protected set. Every original RLE1 count group lies within it; the same fresh-parser proof permits copying unlabelled whole words and processing labelled words in sequence. Original spelling and return geometry remain available, and words touching the marked n-byte truncation are clipped. Before patching any edges a 4,096-step block-wide budget bounds all owner rewinds. When exceeded, the already constructed original graph spells the coded output and calls stock RLE1. In this experiment the extra label width is conservatively sized for the complete danger set. A separate 1,024-occurrence threshold can refuse indexed discovery before materialization, using the normal worker path.

The exact structural screen covered all 18,434 arbitrary columns/marks and ten real blocks. On frozen, 95 original edges suffice to cover all 170 protected positions; their total byte length is exactly 170. Only 186 LF rewinds are needed. On stream 134 the 470 positions occupy 168 edges spanning 33,780 bytes, and independent rewinds take 97,316 steps. Fragmentation and LF depth vary independently of the count mass. [Per-block evidence](geometry.csv)

The native implementation passed the same exhaustive cases including 1,680 unfinished-run errors. Of the 227 large admissions below the occurrence threshold, 161 completed sparse owner projection and 66 used the bounded stock finish. All 227 matched full decoded bytes and CRC. Twenty paired repetitions on the nine systematic admissions put the stage at 1.135 [1.050,1.228] in thread CPU ratio relative to opend, including the standalone diagonal recognition index. [Native census](native.csv) · [Full price](phases.csv) · [Summary](summary.txt)

```sh
d=research/astra-e/formulations/f2-rle/saturated
scratch=$(mktemp -d)
cargo build --release --locked --manifest-path bench/moonshot/Cargo.toml --target-dir "$scratch/build"
deps="$scratch/build/release/deps"
python3 "$d/probe.py" "$scratch/check.rs"
rustc --edition=2021 -O "$scratch/check.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$scratch/check"
"$scratch/check" small
"$scratch/check" corpora/wikidata.bz2
python3 "$d/price.py" "$scratch/price.rs"
rustc --edition=2021 -O -C codegen-units=1 "$scratch/price.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$scratch/price"
"$scratch/price" small
"$scratch/price" corpora/wikidata.bz2 "$scratch/phase.csv"
```
