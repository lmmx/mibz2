# One-byte F5 identity hints

A dictionary word containing no four identical adjacent bytes is transparent to RLE1 from an empty state. If it starts with a new byte following a nonpending state, the same conclusion holds. The hint is the word's final run length, one to three. On a full word under either guard, copy its original bytes, use its last byte and restore this count. A zero hint, a clipped word or an incompatible incoming state feeds the original bytes to the exact streaming transition. This removes the expanded dictionary, suffix cuts, per-word decoded offsets and lengths. It forgoes opportunities to reuse partially synchronizing or genuinely expanded words.

All 159,851 independent word checks passed. On every one of the 228 large corpus admissions, the whole decoded output agreed with stock opend and its stored CRC. The 21-round, three-arm diagnostic alternated direct actions, stock opend and hints on nine fixed blocks. Paired stage CPU for hints divided by opend was 1.047 [1.002,1.093]; hints divided by full actions was 0.983 [0.939,1.029]. The comparison includes construction, dictionary preparation, reconstruction and final CRC. The memory improvement is measurable, while the close head-to-head timing lacks independent confirmation. [Raw observations](phase.csv) · [Ratios](summary.txt) · [Hint capacities](hints.csv)

The clocks in the raw CSV's whole wall and CPU columns cover the complete inverse plus CRC. This three-arm driver only times CRC as a component. Its four zero component columns are uninstrumented. The original two-arm price study retains the per-phase timings. [Full action study](../README.md)

```sh
d=research/astra-e/formulations/f5/kernel/lean
scratch=$(mktemp -d)
python3 "$d/prepare.py"
python3 "$d/check.py" "$scratch/check.rs"
rustc --edition=2021 -O "$scratch/check.rs" -o "$scratch/check"
"$scratch/check"
cargo build --release --manifest-path bench/moonshot/Cargo.toml --target-dir "$scratch/build"
deps="$scratch/build/release/deps"
python3 "$d/price.py" "$scratch/price"
rustc --edition=2021 -O -C codegen-units=1 "$scratch/price/main.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$scratch/price/probe"
"$scratch/price/probe" corpora/wikidata.bz2 "$scratch/phases.csv"
python3 "$d/hints.py" "$scratch/geometry"
rustc --edition=2021 -O -C codegen-units=1 "$scratch/geometry/main.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$scratch/geometry/probe"
"$scratch/geometry/probe" corpora/wikidata.bz2
```
