# Charge only sparse corrections while writing the first complete marked lap

A complete marked LF cycle is at most n coded bytes. Its first lap can emit whole return words unconditionally. Every protected literal write exchanges one coded byte for one output byte; a count token b exchanges one coded byte for b output bytes. Accumulating b−1 only on count tokens yields the coded first-period length as decoded bytes written minus the signed correction. If the period is shorter than n, continue from the mark with the carried parser state until n coded bytes. When there are no protected edges anywhere, repeat the observed literal period by ordinary buffer copies. This eliminates a remaining-byte check from every edge on the common single-cycle case. A final shortened phrase still receives exactly the real coded prefix.

Store a reset flag on each tagged word and clear it for every tagged successor of a tagged edge. Only transitions from clean to protected words need reset. Starting inside any protected span always starts with the ordinary empty state. The proof that counts cannot cross clean bytes ensures that stale state after a protected span cannot report an unfinished run at the end of a subsequent clean edge.

All 18,434 exhaustive marked cases and all intermediate event coordinates/real output checks passed. In 21 same-process rotated rounds with both predecessors, full-stage thread CPU ratios were 0.943 [0.886,1.003] against the checked-every-edge writer and 1.030 [0.972,1.092] against opend. These intervals leave the timing improvement unresolved. [Paired measurements](phases.csv) · [Summary](summary.txt) · [Corpus](native.csv)

```sh
d=research/astra-e/formulations/f2-rle/laps
scratch=$(mktemp -d)
python3 "$d/price.py" "$scratch"
cargo build --release --locked --manifest-path bench/moonshot/Cargo.toml --target-dir "$scratch/build"
deps="$scratch/build/release/deps"
rustc --edition=2021 -O -C codegen-units=1 "$scratch/main.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$scratch/probe"
"$scratch/probe" small
"$scratch/probe" corpora/wikidata.bz2 "$scratch/phases.csv"
```
