# Constructing the selected return words before marked output

The frequency-selected owner can be installed before expansion from the marked row. For a word above the threshold, spell its constituent first tokens directly into its final position in the compact dictionary. The first time a first token is needed, its node spells bytes from the first trie and its descriptor stores this compact location. On later uses during construction, copy the earlier fully initialized dictionary span. The marked pass can read a full or clipped hot word immediately. It can also use an initialized compact first-token span inside otherwise cold second words. Remaining first and second words use output ownership on first complete visit. The descriptors distinguish trie nodes, compact spans and earlier output spans by tags; word byte lengths remain unchanged. Full-word trie traversals, initial/final clips and one-layer fallback retain the invariants of the preceding constructions.

Every selected word is now constructed even when the marked cycle does not use it. Selection still costs a terminal-width accumulation and the compact dictionary occupies the selected distinct words' total byte width. On 228 admitted Wikidata blocks both native thresholds matched all 205,197,794 original marked bytes. The two/frequency-64 dictionaries consumed 7,811,667 and 363,314 bytes across blocks, respectively. Threshold 2 delivered 171,575,742 marked bytes as compact second words and 9,849,559 inside smaller first-word copies. Threshold 64 delivered 110,156,454 and 9,904,793. These counts include first marked visits because the selected word is already constructed. The table separately reports each size, second-word and first-word read volume. [Native census](geometry.csv) · [Arbitrary marked cycles](property.log)

Twenty-five five-arm rotations on each of nine systematic admitted blocks priced the change against the late cutoff-2 arm, the batched two-dictionary constructor, and Lazy's inverse. Eager cutoff 2/late cutoff 2 was 0.9575 [0.9061,1.0117]. Against batched and Lazy it was 1.0652 [1.0327,1.0987] and 1.1596 [1.1145,1.2066]. Eager cutoff 64/batched was 1.0909 [1.0569,1.1260]. The late cutoff-2 control was 1.1125 [1.0531,1.1753] against batched in that same run. The direct close contrast with late construction spans parity. [Raw rotated phases](phase-cpu.csv) · [Paired intervals](phase-summary.txt)

Use the same checked corpus, toolchain and dependency directory as the preceding mixed-policy experiment.

```sh
study=research/astra-e/formulations/f2d
scratch=$(mktemp -d)
cargo build --release --locked --manifest-path bench/moonshot/Cargo.toml --target-dir "$scratch/base"
deps="$scratch/base/release/deps"
rustc --edition=2021 -O --test "$study/mixed-eager/property.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$scratch/property"
"$scratch/property" candidate::checks::ownership_on_arbitrary_marked_cycles --nocapture
F2_CORPUS="$PWD/corpora/wikidata.bz2" F2_CSV="$scratch/geometry.csv" "$scratch/property" candidate::checks::checked_thresholds --ignored --nocapture
mkdir -p "$scratch/price"
bash "$study/mixed-eager/price.sh" "$scratch/price" "$deps" > "$scratch/price/phase-cpu.csv"
python3 "$study/summary.py" "$scratch/price"
```
