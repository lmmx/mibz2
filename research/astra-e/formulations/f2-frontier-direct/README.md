# Schedule the direct compound section alongside the nested construction

The independent-frontier argument carries over from quote tokens to their compound byte marker. Each lane retains its [lo,hi) interval and private word. For this section the root starts with an empty spelling, and a lane ends if its next interval lies in I(P). Publication follows the same breadth-first order as the proved serial direct algorithm. The complete constructor again reconstructs the compound token BWT without producing the first-level token BWT. A committed generator changes only the traversal scheduling of the pinned direct source.

All 245,820 forced arbitrary pattern and mark cases passed, including 147,740 delimiter-free refusals. The native worker route matched 205,197,794 inverse bytes on 228 real admissions. [Small test](property.log) · [All blocks](native.csv)

A fresh process compared both sources and both schedules with LazyCells in 25 rotated passes per block on nine systematic admissions. Batched/serial whole-inverse thread CPU was 0.810 [0.741,0.885] for the direct marker and 0.889 [0.815,0.971] for the nested construction. For the two batched algorithms, direct/nested was 1.000 [0.898,1.113] in CPU and 0.984 [0.788,1.228] in wall. Both still took about 1.15 of LazyCells' thread CPU in this replication. This separately confirms the benefit of overlapping independent discoveries on both formulations and leaves the ordering of those formulations unresolved. [Paired contrasts and per-stage medians](summary.txt) · [Raw 1,125 observations](phase-cpu-wall.csv)

```sh
study=research/astra-e/formulations/f2-frontier-direct
export MOONSHOT_CANDIDATE="$PWD/$study/candidate.rs"
python3 "$study/prepare.py"
cargo test --locked --release --manifest-path bench/moonshot/Cargo.toml candidate::checks::arbitrary_marks_and_overlaps -- --nocapture
F2_CORPUS="$PWD/corpora/wikidata.bz2" F2_CSV=/tmp/direct8.csv cargo test --locked --release --manifest-path bench/moonshot/Cargo.toml candidate::checks::corpus -- --ignored --nocapture
bash "$study/price.sh" /tmp/both-return-schedules "$PWD/bench/moonshot/target/release/deps"
python3 "$study/summarize.py"
```
