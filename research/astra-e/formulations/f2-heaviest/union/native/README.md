# Native stable merge for a union of unequal return types

The native implementation selects the most frequent quote-return types by their source ranks, merges adjacent source buckets, and initializes one actual trie root per contiguous component. A nonterminal interval supplies its root's stable selector order. At a hit, the original byte run gives the destination component's local word slot and the flattened source rank in the union. Each root owns its terminal word identities, so equal spellings ending in different components can have different ranks. Reverse node order materializes the component streams, which are read in ascending root-component order when filling the final links. The mark rewinds in the original LF order to the first rank in the union; a disjoint cycle falls back. No parent token BWT is materialized.

All 228 canonical admissions at k=1,4,8 agreed byte-for-byte with the independent Segmented inverse, in addition to 27,135 forced small marked outputs and 5,641 missed-section cases for k=1,2,4,8. Every systematic admitted block also had exactly the colored run counts and section sizes found by the independent affine-band constructor. The k=8 worker at `4dfdcb5` passed 338 v1 libbzip2/CRC cases. [All admissions](geometry.csv) · [Property check](small.log) · [Full worker gate](../../../../results/v1-union-eight-gate/score/summary.txt)

Across the corpus, changing k=1 to k=8 reduced direct lookup steps from 28,053,831 to 24,729,324 and explicit spelling from 41,433,719 to 29,598,573 bytes. The residual grew from 5,566,401 to 16,964,080 rows. On 32 paired rotations of nine blocks the k=8 discovery ratio was 0.926 [0.886,0.967] versus k=1, but residual output took 2.610 [2.341,2.911] and the full inverse took 1.048 [1.001,1.097]. k=4 against k=1 was 1.015 [0.974,1.058] overall. k=8 took 1.609 [1.543,1.678] of LazyCells' complete inverse on these blocks. The tradeoff identifies what remains to be displaced: induction on many rows and their ordered output. Increasing k alone did not produce a whole-stage gain. [Phase observations](phase.csv) · [Paired estimates](summary.txt)

```sh
d=research/astra-e/formulations/f2-heaviest/union/native
python3 "$d/prepare.py"
export MOONSHOT_CANDIDATE="$PWD/$d/candidate.rs"
cargo test --locked --release --manifest-path bench/moonshot/Cargo.toml union_marks -- --nocapture
F2_CORPUS="$PWD/corpora/wikidata.bz2" F2_CSV=/tmp/union-geometry.csv cargo test --locked --release --manifest-path bench/moonshot/Cargo.toml candidate::checks::corpus -- --ignored --nocapture
bash "$d/price.sh" /tmp/f2-union-price "$PWD/bench/moonshot/target/release/deps"
python3 "$d/summarize.py"
```
