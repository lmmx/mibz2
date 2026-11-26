# Compile only the fifth byte, and certify a complete marked lap by decoded mass

For each start of four equal cyclic bytes b, let its five-position window include the following byte k. Assume these windows are pairwise disjoint as row sets and the primary lies outside them. A different live run ending immediately before the quartet cannot make its first byte a count: that preceding four-run would overlap this window. An equal immediately preceding literal would itself create an overlapping start. Consequently the group at each start is exactly four emitted b literals followed by expansion of k further b's, independent of the other groups. All four literal bytes already equal the required output. Only the fifth source byte changes, to k copies of b; k=0 deletes it.

Bundle-trace the leftmost unresolved starts as in the previous construction, retaining both the exit translation and the number of steps to it. That step count is the first start's byte offset in its owning return. Advance each start four Ψ steps. A retained row begins a new owner at offset zero; every other step increments the offset. This determines the exact residual source rank and local spelling offset of each count byte without reconstructing plaintext. Group edits by residual rank and order by offset. Copy the old word's literal slices and append its replacements into an additional dictionary entry. On this version a block-wide budget of n/16 bytes bounds these copied new spellings. Original edge codes stay available for rollback. Retain the old uninstrumented prefetch/copy loop, with no parser state or edge-level remaining-byte count.

**Decoded-mass certificate.** Compute G = n + Σ(k−1) over *every* event in the finite original LF permutation, including cycles disjoint from the primary. Each LF cycle contributes its positive corrected byte length: a complete five-input count group contributes at least four bytes, and other bytes contribute one each. Distinct groups do not overlap. Starting from the primary outside all windows, one entire lap spells exactly the corrected cyclic length of its component. Hence this first lap emits G bytes if and only if the marked cycle covers all n source rows. Otherwise discard the provisional output, restore the original edge codes, and apply the ordinary stock parser to exactly n marked raw bytes. Overlaps, a marked split inside a window, budgets and field-width limits also have exact fallbacks. A checksum comparison would not prove cycle coverage. Here the criterion follows from positivity of the omitted components.

An independent exhaustive verifier enumerated every binary column through length fifteen and every primary. Among 125,680 certified marks, it distinguished 30,548 true single-cycle and 95,132 multi-cycle cases; 38,408 of these had at least one nonempty edit set. All first-lap bytes matched the stock parser, and every decoded-mass decision agreed with the original marked cycle length. [Independent proof test](certificate.rs) · [Receipt](certificate.txt)

**Native accounting.** The fully integrated narrow/immediate endpoint form completed the count-only route on 222/228 admitted large blocks. Five exceeded the extra corrected-spelling budget and one exceeded the event cap. All outputs matched the stored CRC and the original opend decoder. The 222 blocks had 9,372 events, 8,811 patched owners and 1,129,906 extra copied spelling bytes. Frozen had 34 patched words occupying 175 bytes. Directly measured peak retained bytes among the four arms were 7,636,174 for count-only, 6,730,182 for the clean/protected combined writer, 7,558,316 for narrow-short/ahead and 7,910,380 for opend. An earlier fallback can force a larger retained raw vector on subsequent blocks. The 32-round paired stage CPU ratio was 1.006 [0.958,1.057] vs the prior combined writer. The structural theorem survives, while duplicating entire corrected words did not establish a speed improvement. [Full comparison](summary.txt) · [Timings](phases.csv) · [Capacities and outcomes](capacity.csv)

```sh
d=research/astra-e/formulations/f2-rle/count-only
scratch=$(mktemp -d)
python3 "$d/price.py" "$scratch" > "$scratch/inputs.sha256"
cargo build --release --locked --manifest-path bench/moonshot/Cargo.toml --target-dir "$scratch/build"
deps="$scratch/build/release/deps"
for source in main adversarial; do
  rustc --edition=2021 -O -C codegen-units=1 "$scratch/$source.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$scratch/$source"
done
rustc --edition=2021 -O "$d/certificate.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$scratch/certificate"
"$scratch/certificate"
"$scratch/adversarial"
"$scratch/main" corpora/wikidata.bz2 "$scratch/phases.csv" "$scratch/capacity.csv"
```
