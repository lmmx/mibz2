# Rank and select on the compiled Open graph

The same queued Open-Return representation that emits marked text contains a navigable factorization of the original stable LF map. Its thick strips are sorted by source and, separately, by their LF image. Each thin queue stores the sorted retained source ranks of one byte. A source-width prefix array and an image-width prefix array, one entry per thick strip in each order, make all original rank and select queries exact after construction. The constructor fills those entries while visiting each thick strip. The byte bucket starts and per-byte starts in the two restricted sets are already computed at that stage.

At an original row x, subtract the total thick width strictly before x (including a partial strip) to get the number of preceding retained rows. The lower bound of that number in queue b counts preceding thin b's. To count preceding thick b's, find the last b strip whose source precedes x. In the b group of image order, the strips are sorted by source, by stability. Subtracting the group's starting image-width prefix from that strip's prefix and adding its occupied prefix gives the required count. Their sum is rank_b(x), including at x=n. Thus all I(b⁴) can be found using the original Open graph.

For Ψ at an F-coordinate y, search the thick image order. A containing strip inverts by translation. Otherwise subtract the preceding thick image width, then the start of b's thin image bucket, and index the result into queue b. This returns a retained source rank j. Its original row is j plus the total width of the thick strips preceding it. Search their source-order insertion coordinates (strip start minus preceding thick width); all insertion coordinates at or before j precede that retained source. This recovers Ψ exactly. Four advances per event now suffice because full-return saturation does not require a fifth cut. Recognition incurs zero further passes over the original capped run stream. Residual tag capacity is checked after the actual owner count is known, preserving the original packed-table width. A budget overflow spells with the original graph and stock RLE1.

In exhaustive native full reconstructions, every event and protected rank also matched the independent full-run FM index. The same coordinate equality held on all 227 real blocks under the 1,024-occurrence threshold. All 228 density-admitted blocks verified full decoded bytes and CRC. Frozen extra navigation capacities were 33,376 B, and the high water over these blocks was 73,808 B. This includes prefix arrays, event/owner vectors, saved labels and bucket starts, separate from existing endpoint arrays, fallback coded buffer, and final output.

Across 20 paired rounds on nine systematic blocks the sparse/opend stage CPU ratio was 1.039 [1.000,1.080]; wall was 1.080 [0.878,1.329]. The median summed navigation time was 0.419 ms against 0.931 ms of stock RLE1. Marked traversal, preparation and occasional fallback still cost time. Owner timing here covers projections completing under budget; other operations remain inside the full-arm time. This near-parity result requires independent measurements for any future claim of a small gain. [Exact per-block outcomes](native.csv) · [All phase observations](phases.csv) · [Summary](summary.txt)

```sh
d=research/astra-e/formulations/f2-rle/compiled
scratch=$(mktemp -d)
python3 "$d/price.py" "$scratch"
cargo build --release --locked --manifest-path bench/moonshot/Cargo.toml --target-dir "$scratch/build"
deps="$scratch/build/release/deps"
rustc --edition=2021 -O -C codegen-units=1 "$scratch/main.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$scratch/probe"
"$scratch/probe" small
"$scratch/probe" corpora/wikidata.bz2 "$scratch/phases.csv"
```
