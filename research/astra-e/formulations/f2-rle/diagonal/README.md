# Restrict rank and select to the L/F diagonal

For each byte b, the three nested FM intervals I(b²), I(b³) and I(b⁴) all lie within I(b). A sweep across maximal L runs and sorted F buckets stores only intersections whose two bytes coincide. Each entry carries L position, occurrence rank before the entry, and its length. The rank at the start of every F bucket is kept separately. This suffices for every boundary rank query during backward extension. Ψ³ of a member of I(b⁴) still starts with b. Thus the first three selects need only this diagonal index. They preserve the order of the input occurrences within each b. Since their image starts with b, the resulting rank list is also in order globally across b. For each of the remaining two Ψ advances, partition that sorted list by F bucket and scan the capped L runs once to return precisely the queried occurrences. The resulting ranks are emitted in increasing L order. This proves that both full select-index construction and separate general select searches can be eliminated. The implementation sorts and deduplicates the small union of the five coordinate lists when forming tags and cuts.

The native implementation returned the same admissions, event, danger and seal masses and final bytes as the all-run index on the large corpus. All 18,434 exhaustive native marked cases passed, including 1,680 invalid endings. Total retained probe arrays on frozen fell from 890,960 to 55,152 bytes, with high water of 146,592 bytes on the corpus. These are the hazard-index capacities, separate from the endpoint, slot and process. With the same original stage protocol, the candidate/opend paired thread CPU ratio was 1.158 [0.962,1.395] and wall was 1.386 [1.112,1.728]. The diagonal index still costs several input passes and exceeds the stock RLE1 time. The two constructors' runs occurred in different host-load epochs; their wall/CPU point estimates cannot rank close designs head to head. [Full block counts](native.csv) · [Timings](phases.csv) · [Statistics](summary.txt)

```sh
d=research/astra-e/formulations/f2-rle/diagonal
scratch=$(mktemp -d)
python3 "$d/price.py" "$scratch"
cargo build --manifest-path bench/moonshot/Cargo.toml --release --locked --target-dir "$scratch/build"
deps="$scratch/build/release/deps"
rustc --edition=2021 -O -C codegen-units=1 "$scratch/main.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$scratch/probe"
"$scratch/probe" small
"$scratch/probe" corpora/wikidata.bz2 "$scratch/phases.csv"
```
