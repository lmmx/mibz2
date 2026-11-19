# Feed the run front's maximal runs directly to the combined constructor

The fixed control is narrow-short/ahead at `80d8409359214b4e5abfd4430213921e82e9cd30`. The stock AVX2 inverse MTF/RLE2 front already knows the byte and full width of each run before splitting it at 127 rows. The combined constructor reads those capped records and reconstitutes maximal runs in its initial scan. Two experimental arms consume the original AVX2 bitmap/literal loop. The first feeds whole widths into the constructor immediately. The second feeds geometry in groups of 64 packed whole runs with a 256-byte bounded buffer. The remaining endpoint induction, edge generation and lookahead writer are generated from the same frozen control.

**Equivalence.** The number of production cap records for one closed run of width w>0 is 1+floor((w−1)/127); hence exact strict admission is computable from maximal widths without materializing those records. A zero-width initial run contributes nothing. True run boundaries match the parser's literal steps when its inverse MTF map has distinct active bytes. Observed equal adjacent closed symbols cause a complete stock replay, including the record-density decision. Each thick maximal run containing the mark emits the original left thick piece, retained marked singleton and right thick piece. Both remaining pieces keep their parent's classification even below width 17. The collected per-byte row counts, thick counts, thin rank queues and ordered strip translations are therefore identical to the fixed predecessor's first-pass quantities. The native driver checks exact uncapped inverse output, expanded output and stored CRC on real blocks, with comparable steps for tiny, rejected, and plain-column blocks.

The native experiment preserves the stock scalar front and errors by discarding partial speculative work and replaying stock when the AVX2 machine or dense crossover does not admit completion. During a speculative decode the running row bound is checked before storing geometry, bounding malformed input and preserving the stock front's final choice of error. Blocks with fewer than 256 symbols use stock immediately; this avoids speculation on the corpus's numerous tiny blocks. Stock finish buffers remain live after fallback. A candidate worker retaining custom bitmap scratch alongside the stock front would need to account for both; a shared front hook could reuse those bitmaps. No source under `src/` is required for this standalone native test.

The original run front and constructor closure are pinned by SHA-256 in [the generator](prepare.py). The first screening pass over the 383 `takes_runs` blocks completed 228 directly and 155 through the stock route, matching all capped counts, marked output bytes, decoded output and stored CRC. [Initial structural receipt](exact.txt) · [Raw observations](exact.csv)

The complete route differential at `da1ce6d` checked all 431 real scanner blocks. The 228 direct blocks have 205,197,794 inverse rows, 14,427,155 maximal runs, 15,583,349 cap records, 25,484,201 thin queue entries and 744,649 thick pieces. Thus the fused construction avoids a write and a reread of 31,166,698 bytes of caps over these blocks, in addition to the separate coalescing loop. Five large dense rejections, 150 small stock-run finishes and 48 plain-column blocks retain the actual predecessor route. A total of 11,200 adversarial detected/scalar comparisons agreed on valid bytes and the exact first error, including bad primaries, source bounds, oversized RLE2 and incomplete RLE1. [All route rows](checked.csv) · [Adversarial outcomes](adversarial2.txt) · [Full cap and geometry census](geometry.csv)

To reproduce the native test, run the following from the research checkout.

```sh
d=research/astra-e/front-fusion
scratch=$(mktemp -d)
cargo build --release --locked --manifest-path bench/moonshot/Cargo.toml --target-dir "$scratch/build"
deps="$scratch/build/release/deps"
python3 "$d/prepare.py" "$scratch/source"
rustc -O --edition 2021 -A unused -C codegen-units=1 "$scratch/source/main.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$scratch/native"
"$scratch/native" adversarial
"$scratch/native" corpora/wikidata.bz2 0
nice -n 10 taskset -c 3 "$scratch/native" corpora/wikidata.bz2 24 > "$scratch/panel.csv"
python3 "$d/summarize.py" "$scratch/panel.csv"
```
