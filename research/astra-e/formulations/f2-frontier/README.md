# Eight independent delimiter intervals in flight

A contracted trie node depends on the endpoint of its own unary path, and its children depend on it. Nodes already present in the breadth-first frontier have no such dependency on one another. Select up to eight of them. In each wave, locate every live job's current original run; only after all these independent rank requests have been issued does the second loop inspect the runs, advance the live jobs and stop the returning ones. Each job stores its spelling and its interval privately. Publish the eight path spellings and process each node's ordered selectors in the original FIFO order. Parents have materialized their paths before any child's word spell-out, the node numbering and merge order remain deterministic, and mark transfer and the entire second level are unchanged. This is a scheduling consequence of the delimiter theorem.

The main extra work is indexing the live lanes, one copy of each path into the final edge buffer, and storing up to eight reusable private spelling vectors. At a given batch the paths belong to disjoint trie edges. One unusually long path can leave other lanes idle. The CPU model is tested by full matched inverse-stage execution; hardware IPC and cache counters were not collected in this screen.

On 21 rotated paired passes per block, every output matched Segmented outside timing. Nine admitted systematic blocks gave summed per-block median discovery CPU of 14.51 ms scalar and 8.76 ms batched. The complete inverse took 24.79 ms scalar, 18.41 ms batched and 17.05 ms LazyCells. Pairing the sum of the same nine blocks within each pass gives batched/scalar discovery 0.588 [0.549,0.630], complete inverse 0.733 [0.687,0.782], and batched/Lazy complete inverse 1.111 [1.050,1.176]. The intervals describe variation on this host and these fixed blocks. [All observations](phase-cpu.csv) · [Summed medians](summary.txt)

```sh
study=research/astra-e/formulations/f2-frontier
export MOONSHOT_CANDIDATE="$PWD/$study/candidate.rs"
python3 "$study/prepare.py"
cargo test --locked --release --manifest-path bench/moonshot/Cargo.toml candidate::checks::arbitrary_marks -- --nocapture
F2_CORPUS="$PWD/corpora/wikidata.bz2" F2_CSV=/tmp/f2-frontier-all.csv cargo test --locked --release --manifest-path bench/moonshot/Cargo.toml candidate::checks::native_second_level -- --ignored --nocapture
bash "$study/price.sh" /tmp/f2-frontier-price "$PWD/bench/moonshot/target/release/deps"
bash research/astra-e/score.sh delimiter-frontier "$MOONSHOT_CANDIDATE" --with-opend --workers 1,4 --rounds 9 --repeat 2 --settle 2 --no-proxy
```
