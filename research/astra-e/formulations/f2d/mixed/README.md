# Mixed output ownership with a frequency-selected dictionary

An accepted second-level word w has width |w| and total BWT multiplicity f. Every terminal selector at its discovery node contributes its width to f. This is obtained from the actual ordered partition; the runtime decision requires one accumulator at the node and no subsequent token-table scan. In the full balanced marked tour that visits every token occurrence once, a complete word contributes f·|w| source bytes and requires only |w| bytes of compact owner storage. If an owner is installed on its first complete occurrence, the remaining idealized copy volume is (f−1)·|w|. Thus for a fractional byte budget, choosing bytes by descending f is optimal for this additive repeat-byte coverage model, by exchanging one equally sized lower-frequency portion for a higher-frequency portion. This argument prices neither instruction work nor cache misses. Whole-word choices at a hard capacity are a knapsack problem. A short marked cycle need not visit every occurrence in the BWT, and an initial/final clipped word can have no complete occurrence at all.

A census traverses the actual marked token order with the transported offset, counting byte coverage, visits, and full occurrences by word. On all 228 included blocks the potential dictionary/marked-byte coverage curve is as follows. Shares aggregate bytes across blocks. The potential hot output column includes first visits.

| Minimum multiplicity | Share of full distinct spelling bytes | Share of marked emitted bytes in those words |
|---:|---:|---:|
| 2 | 18.85% | 83.61% |
| 4 | 11.18% | 80.05% |
| 8 | 7.10% | 75.80% |
| 16 | 3.42% | 67.97% |
| 32 | 1.63% | 60.21% |
| 64 | 0.88% | 53.68% |

[All eight thresholds and 228 inputs](thresholds.csv)

## Complete reconstruction

A cold word uses an earlier full output occurrence as its owner. On the first full visit to a hot word the writer copies the completed output span into an append-only compact dictionary. Its descriptor distinguishes a trie node, an output offset and a dictionary offset. Every future slice of that hot word reads the dictionary. A partial initial or final occurrence still cannot establish an owner. Both requested measured threshold arms use the same eight-path first construction, two token tries and exact marked writer. The original one-layer fallback handles every refusal.

Both native arms agreed with Segmented on all 205,197,794 admitted inverse bytes. The threshold-2 compact dictionaries totalled 7,811,667 bytes across blocks, and their actual marked copy reads delivered 163,764,074 bytes. This equals the complete second-owner copy volume of the output-only arm on this corpus. Threshold 64 stored 363,314 bytes and delivered 109,793,139 bytes through the compact dictionary. The remaining visits include uncached/partial spellings and ordinary earlier-output copies. Frozen threshold 2 used 31,024 dictionary bytes with 753,748 copied from them; threshold 64 used 1,698 with 530,054 copied from them. [Native per-block accounted bytes](geometry.csv)

In 25 matched five-arm rotations across nine admitted blocks, threshold 2 versus output-only took 0.9953 [0.9158,1.0817], threshold 64 versus output-only 0.9823 [0.9054,1.0657]. Against the batched constructor with complete, construction-time dictionaries they took 1.1342 [1.0289,1.2504] and 1.1195 [1.0267,1.2206]. Frequency localization substantially concentrates the repeat-copy source. These stage measurements do not resolve a throughput benefit over full output ownership. [Raw observations](phase-cpu.csv) · [Intervals](phase-summary.txt)

```sh
study=research/astra-e/formulations/f2d
scratch=$(mktemp -d)
cargo build --release --locked --manifest-path bench/moonshot/Cargo.toml --target-dir "$scratch/base"
deps="$scratch/base/release/deps"
bash "$study/mixed/probe.sh" "$scratch/probe" "$deps"
rustc --edition=2021 -O --test "$study/mixed/property.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$scratch/property"
"$scratch/property" candidate::checks::ownership_on_arbitrary_marked_cycles --nocapture
F2_CORPUS="$PWD/corpora/wikidata.bz2" F2_CSV="$scratch/geom.csv" "$scratch/property" candidate::checks::checked_thresholds --ignored --nocapture
mkdir -p "$scratch/price"
bash "$study/mixed/price.sh" "$scratch/price" "$deps" > "$scratch/price/phase-cpu.csv"
python3 "$study/summary.py" "$scratch/price"
```
