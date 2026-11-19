# How much can concurrency buy on the second delimiter level?

After dynamic refill on the first byte-level contraction, the second contraction works on the exact integer token BWT. Its graph differs sharply. On nine systematic admissions there are only 54,511 rank actions for 32,639 physical nodes (1.67 per node), with 93,414 branch selectors and 1,554,280 dictionary bytes to publish. The first-level graph had 751,521 queries feeding 38,624 nodes on the same blocks (see the original graph CSV for exact totals). Frozen has only 20 dependent waves on the critical path, at most 2,282 simultaneous queries, and one branch releases 763 children. Fixed batches use 871 waves there; immediate refill uses 715 with a lower bound of 712. The nine-block aggregate is 8,583, 6,848 and 6,817 respectively. [Second-level DAGs](graphs.csv.xz) · [Wave simulation](waves.csv)

Isolating each phase of the existing scalar second-level kernel put 3.066 ms in discovery, 0.665 ms in column preparation and shifting the mark, 0.927 ms in inverse merge, 0.434 ms in token-link construction and 1.643 ms in spelling the output. In that measurement the full two-level inverse took 27.323 ms. This priced a possible gain before constructing the two schedulers. [Phase observations](profile.csv)

The nested first level stays identical in all three arms. The derived second-level kernels carry private u32 token paths in eight lanes and either hold a batch until every lane completes or refill immediately. Both prove the same interval and publish the same per-parent selectors before child assignment. Varying node and word IDs does not alter ordered token buckets or completed words. A successful corpus audit exercised both levels in the refill version on all 228 normal admissions and matched 205,197,794 raw inverse bytes. [Native census](native.csv) · [Property test](property.log)

The 32 rotated complete-inverse passes per block did not resolve a gain for either second-level scheduler. To reduce upstream variation, 96 additional rotated passes pinned the thread to one allowed CPU and reused each block's actual precomputed first-level token BWT. Every arm rebuilt the second level, materialized and emitted the decoded bytes, and matched Segmented outside its timed region. Refill relative to scalar cost 1.0677 [1.0032,1.1364] on **second-level discovery**, and 1.0240 [0.9686,1.0826] on the complete second level. Fixed-eight relative to scalar cost 1.0291 [0.9600,1.1032] and 0.9961 [0.9459,1.0490]. Hot-input replay and whole-inverse trials have distinct cache histories. Neither demonstrates a production gain. The preserved leading F2 candidate keeps the scalar second level. [Complete inverse observations](phase.csv) · [Scoped replay](replay.csv) · [Ratios](summary.txt)

The data establish that independent paths exist at both levels. In this constructor the number of useful continued unary loads per physical node is much smaller on the second level, where retirement, spelling copies and new integer-alphabet children still demand work. For its paid time we leave changing the dictionary/selector computation open. A further change of lane count alone has no supporting evidence here.

```sh
d=research/astra-e/formulations/f2-scheduler/second
deps="$PWD/bench/moonshot/target/release/deps"
python3 "$d/prepare.py"
MOONSHOT_CANDIDATE="$PWD/$d/candidate.rs" cargo test --locked --release --manifest-path bench/moonshot/Cargo.toml candidate::checks::arbitrary_marks -- --nocapture
F2_CORPUS="$PWD/corpora/wikidata.bz2" F2_CSV=/tmp/f2-ss.csv MOONSHOT_CANDIDATE="$PWD/$d/candidate.rs" cargo test --locked --release --manifest-path bench/moonshot/Cargo.toml candidate::checks::native_second_level -- --ignored --nocapture
bash "$d/profile.sh" /tmp/f2-ss-profile "$deps" /tmp/f2-ss-graph.csv
xz -c /tmp/f2-ss-graph.csv > /tmp/f2-ss-graph.xz
python3 "$d/../waves.py" /tmp/f2-ss-graph.xz
bash "$d/price.sh" /tmp/f2-ss-price "$deps"
bash "$d/replay.sh" /tmp/f2-ss-replay "$deps"
```
