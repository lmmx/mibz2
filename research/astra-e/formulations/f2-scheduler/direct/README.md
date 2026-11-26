# Dynamic refill on the compound-word cylinder

This arm uses the identical ready-path refill policy as the nested quote constructor, applied to the direct return on the original three-byte FM cylinder. The root has empty spelling. At each candidate extension, landing in the cylinder completes the backward word, including the byte that closes the return. Those two differences from the one-byte token constructor are preserved at lane assignment and at the completion test. The transported offset, terminal buckets, stable inverse merge and native packed-token expansion follow the original compound constructor. The first-level dictionary and second token BWT never need to be constructed on this route.

All 245,820 forced binary pattern/mark cases matched the dense oracle or its exact uncovered-cycle fallback. Every one of 228 admitted large Wikidata blocks matched Segmented, 205,197,794 bytes. The inherited test-census lookup field is a scalar-loop counter and remains zero for both batched entries. Its vector-capacity tally excludes private lane paths. [All measured admissions](native.csv) · [Arbitrary marks](property.log)

Thirty passes on each of nine systematic admitted blocks rotated direct refill, direct fixed eight, nested refill, nested fixed eight and Lazy in one process. Every output matched independently outside the timer. Direct refill/fixed eight averaged 0.9696 [0.9441,0.9958] in paired complete inverse thread CPU and 1.0414 [0.8812,1.2308] in wall time. The corresponding nested contrast was 0.9115 [0.8791,0.9451] CPU and 0.9704 [0.8442,1.1155] wall. Direct versus nested refill CPU was 1.0096 [0.9834,1.0365]. Both comparisons to Lazy include parity. The direct CPU difference is small and the wall interval is wide, so the native direct arm has no established production advantage. [Full observations](phase.csv) · [Five-arm summaries](summary.txt)

```sh
d=research/astra-e/formulations/f2-scheduler/direct
python3 "$d/prepare.py"
export MOONSHOT_CANDIDATE="$PWD/$d/candidate.rs"
cargo test --locked --release --manifest-path bench/moonshot/Cargo.toml candidate::checks::arbitrary_marks_and_overlaps -- --nocapture
F2_CORPUS="$PWD/corpora/wikidata.bz2" F2_CSV=/tmp/f2-refill-direct.csv cargo test --locked --release --manifest-path bench/moonshot/Cargo.toml candidate::checks::corpus -- --ignored --nocapture
bash "$d/price.sh" /tmp/f2-direct-price "$PWD/bench/moonshot/target/release/deps"
```
