# A two-level delimiter inverse with one byte dictionary

The nested delimiter construction uses the first-level word lengths, token buckets and run column to order its second return. It does not require contiguous first-level *spelling* until expanding second-level words. Keep the first trie and its compact edge bytes; for each first-level word store its terminal node and byte depth. Its terminal owns no separate contiguous byte string.

During discovery each node's depth is its parent's depth plus its contracted edge length. The temporary depth uses the existing cursor field before stable merging initializes that field. The word's length is copied when a terminal is first identified, allowing mark transfer by the unchanged weighted return rule. When second-level dictionary construction requests a first-level word, its first use spells the reversed edge segments from terminal to root directly into the second dictionary's output span. Its descriptor replaces the terminal ID with a tagged offset into that dictionary. Subsequent uses copy the earlier immutable span with `extend_from_within`. Vector growth preserves offsets. The largest realized dictionary is bounded by n since the distinct words have disjoint representative occurrences among all section return paths; the high tag bit is available. Lengths and first-column buckets remain unchanged.

An absent second delimiter on the marked cycle, a unanimous second alphabet or packed field overflow can still select the one-level inverse. Its fallback recovers terminal IDs by scanning the original node table, spells a flat first dictionary only on that path and runs the original expansion. This covers overflow even after the second construction has replaced some terminal IDs. The independent small check exercises short, periodic and multiply cyclic last columns and every primary on the tested alphabets.

## Exact cost and timing

All 228 large admissions on canonical Wikidata matched Segmented over 205,197,794 inverse bytes. Frozen eliminates its 83,454-byte first dictionary and still builds the same 146,183-byte final dictionary. Across the full census, first terminal IDs number 728,887 with 23,393,313 spelling bytes. All appear in the final dictionaries. First byte-dictionary length on every successful second-level route was zero, while the final dictionaries contained 41,433,719 bytes. There remained 4,418,276 first-token copy references. The algorithm still builds the first trie and token BWT, and still writes every final dictionary byte. [Census](geometry.csv) · [Full nested program](candidate.rs)

In the scalar experiment, summed per-block median discovery thread CPU on nine blocks fell from 19.03 to 18.09 ms. Second construction, traversal and final output rose from 7.83 to 9.47 ms. The 21-round whole-inverse ratio to F2c was 1.0445 [0.9781,1.1154]; the ratio to the LazyCells inverse was 1.3825 [1.2838,1.4888]. [Individual phase readings](phase-cpu.csv) · [Summary](phase-summary.txt)

The combination uses the eight independent discovery paths from d48e80f. Its full 228-block route census and small oracle agree. In 25 matched five-arm rotations on each of nine blocks, combined/batched was 0.9811 [0.9270,1.0383], combined/scalar ownership 0.8420 [0.7410,0.9568] and combined/Lazy 1.1980 [1.1161,1.2860]. Discovery medians sum to 14.53 ms versus 16.32 ms for the original batched arm; later stages differ. These are kernel-only experiments. The first dictionary is eliminated by both exact constructions. [Full batched program](batched.rs) · [Independent combined census](combined/geometry.csv) · [Five-arm raw readings](combined/phase-cpu.csv) · [Comparisons](combined/phase-summary.txt)

```sh
study=research/astra-e/formulations/f2d
scratch=$(mktemp -d)
cargo build --release --locked --manifest-path bench/moonshot/Cargo.toml --target-dir "$scratch/base"
deps="$scratch/base/release/deps"
python3 "$study/ownership/combine.py"
for stem in property combined-property; do
    rustc --edition=2021 -O --test "$study/ownership/$stem.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$scratch/$stem"
    "$scratch/$stem" candidate::checks::arbitrary_marks --nocapture
    F2_CORPUS="$PWD/corpora/wikidata.bz2" F2_CSV="$scratch/$stem.csv" "$scratch/$stem" candidate::checks::spelling_ownership --ignored --nocapture
done
mkdir -p "$scratch/price" "$scratch/combined"
bash "$study/ownership/price.sh" "$scratch/price" "$deps" > "$scratch/price/phase-cpu.csv"
bash "$study/ownership/combined.sh" "$scratch/combined" "$deps" > "$scratch/combined/phase-cpu.csv"
python3 "$study/summary.py" "$scratch/combined"
```
