# Absorbing pure terminal leaves in the root visitor

A contracted nonterminal with a single byte choice would have been contracted further. Thus a node with one selector has only its own terminal. Once a parent's current selector chooses this child, the child's remaining mass covers the rest of that selector, by stable projection and total-width conservation. Returning its terminal immediately is exact. A pure root emits its one label at the full section width. Other nodes with a self-terminal and child choices still participate in the minimum-width calculation, including when their own terminal is selected.

The implementation tests the reached node's selector count. It avoids the leaf's selector load, path-vector push, used/cursor update and second pass over that path entry. Testing the count still requires reaching the node. No extra leaf flags, selector encoding, preprocessing pass or vector are introduced.

Across 228 real admissions the count of visited selector cursors falls from 7,739,733 to **6,254,289 (−19.19%)**. The total section size, nodes, dictionary cardinality, selector count and root runs agree for every block. All 205,197,794 admitted bytes matched the independent Segmented inverse. The [per-node audit](../cut/nodes.csv.xz) predicts the exact reduced count on all nine comparable sampled blocks. Included regressions cover a pure root throughout, several marks in the one-cycle deep reopening family, and all 11,264 admitted small-column marks. [Census](geometry.csv) · [Property results](property.log) · [Corpus check](corpus.log)

The four arms rotated in each trial on CPU 4, with one warm-up followed by 32 balanced rotations on each of nine systematic real blocks. All four outputs were compared with an independent reference after every inversion. Complete inverse thread CPU relative to the original root cursor was .989 [.950,1.028], relative to bottom-up refill .996 [.940,1.056], and relative to Lazy 1.062 [1.015,1.111]. The merge alone took .949 [.872,1.032] of original root merge CPU and 1.600 [1.504,1.703] of bottom-up merge CPU. The local phase interval permits both a gain and a regression against the root cursor. A leaf cursor can be omitted even on inputs where testing its type costs more than skipping it saves. [Raw rotations](phase.csv) · [Paired results](summary.txt)

The full v1 scorer gate passed 338 cases against libbzip2, including CRCs. The refresh identified scorer source `604dac3`; the scorer printed `6a8a366`. The receipt is preserved at `4383954`. [Scorer report](../../../results/v1-delimiter-root-leaf-gate/score/summary.txt)

Reproduce the first-level geometry and local paired phase observations from the worktree root with a release scorer build and the frozen corpus present.

```sh
d=research/astra-e/formulations/f2-rootstream/leaf
export MOONSHOT_CANDIDATE="$PWD/$d/candidate.rs"
cargo test --locked --release --manifest-path bench/moonshot/Cargo.toml candidate:: -- --nocapture
F2_CORPUS="$PWD/corpora/wikidata.bz2" F2_CSV=/tmp/leaf-geometry.csv cargo test --locked --release --manifest-path bench/moonshot/Cargo.toml candidate::checks::corpus -- --ignored --nocapture
taskset -c 4 bash "$d/price.sh" /tmp/f2-leaf-price "$PWD/bench/moonshot/target/release/deps"
PYTHONDONTWRITEBYTECODE=1 python3 "$d/summarize.py"
```
