# Learning run successors during eight-path discovery

Each lane of the d48e80f frontier scheduler carries its preceding unclassified run and its current run index. When a whole-run successor is cached, that lane advances directly to the known index. The remaining lanes issue a page-rank query together; on consuming their results each originating unknown run learns the same whole-image certificate as the single-path construction. The current BWT interval still performs the normal run-boundary and delimiter checks. Two lanes may independently classify the same run while both requests are outstanding; the certificate's truth is a property of the full run and both answers are consistent. The packed run remains 12 bytes.

The oracle checked every admitted large Wikidata output, 205,197,794 bytes over 228 blocks, and all covered/refused marks in the original small suite. Across these admissions the constructor still undertook 19,750,839 logical discovery steps. It realized 5,404,952 known-successor advances and 14,345,887 actual page lookups. [Native counts](geometry.csv) · [Implementation](candidate.rs)

The combination was slower despite eliminating these lookups. In 25 five-arm rotations over nine systematic admitted blocks, sum of per-block median discovery CPU was 20.89 ms for the combination and 13.92 ms for the eight-path reference. Paired complete inverse CPU (round sums) was 1.2266 [1.1172,1.3468] versus batched, 1.0008 [0.8661,1.1564] versus scalar learned, and 1.3639 [1.2524,1.4852] versus Lazy. The same run measured batched/F2c at 0.8478 [0.7723,0.9307]. The combination pays lane-specific mask maintenance, classification writes and per-run branching. These measurements establish the loss of this implementation without assigning it to any one instruction or cache effect. [Raw phase readings](phase-cpu.csv) · [Complete comparisons](phase-summary.txt)

```sh
study=research/astra-e/formulations/f2d
scratch=$(mktemp -d)
cargo build --release --locked --manifest-path bench/moonshot/Cargo.toml --target-dir "$scratch/base"
deps="$scratch/base/release/deps"
rustc --edition=2021 -O --test "$study/frontier/property.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$scratch/property"
"$scratch/property" candidate::checks::arbitrary_marks --nocapture
F2_CORPUS="$PWD/corpora/wikidata.bz2" F2_CSV="$scratch/geometry.csv" "$scratch/property" candidate::checks::frontier_lookups --ignored --nocapture
mkdir -p "$scratch/price"
bash "$study/frontier/price.sh" "$scratch/price" "$deps" > "$scratch/price/phase-cpu.csv"
python3 "$study/summary.py" "$scratch/price"
```
