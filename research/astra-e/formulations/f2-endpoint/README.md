# Endpoint elimination for the compound-word section

The direct compound-marker trie pays for every unary left extension. This round removes trie discovery, token bucketing and stable inverse merges entirely. It asks whether the measured Open-Return constructor can build the same labelled byte return more cheaply as an interval exchange.

Compute the marker interval I(P) by FM extension with flat capped-run scans. Split the original maximal L runs at either endpoint and around the primary. Keep S=I(P) ∪ {primary}. Removing the outgoing LF arrows of S leaves open paths from LF(S) to S and possibly closed components disjoint from S. A closed component cannot affect the marked output because the primary is retained. On the sorted rows of S, the rank map into LF(S) is the stable LF permutation of the L subcolumn. Composing it with each labelled path's exit permutation yields the exact return on S. The inverse arrow spells the path in reverse order. The extra primary can split one original return or add a marked cycle disjoint from the marker. No search for an offset or rotation of the complete output is needed. This is the pinned Open-Return restriction theorem applied to the compound-marker section. [Original proof](../../../../docs/journal/2026-11-18-open-return/proof.md) · [Section conjugacy](../f2-word/)

The code compares two complete constructors. The first eliminates the complement by Rauzy endpoint moves with labelled concatenation rules. The second traces each certified constant-translation interval and emits its spelling explicitly. Both include the FM scans, stable restricted matching, table materialization and marked output. The primary scorer entry uses endpoint elimination. It provisions the same density fallback as F2b. The generator checks the source hash of the measured Open-Return implementation. Allocator settings are inherited from the crate.

The property test passed 90,354 marked endpoint constructions. Both constructors agreed with Segmented on all 205,197,794 bytes on the 228 real admissions, including every one of the 12 rotated measured rounds on the nine systematic admissions. On frozen the final table takes 96,664 bytes, but the complement has 50,986 strips. Endpoint elimination performs 82,576 moves and constructs 79,586 rules. Both methods spell 243,726 bytes over 5,009 entrance-order bands. These bands use LF(S) entrance order; they are not token-BWT runs in section order. Inverse thread CPU across nine blocks was 2.516× Lazy for endpoint elimination and 2.372× for tracing. The candidate solves the label order without a byte trie. The cost of handling almost every source run in the complement independently dominates this implementation. [Per-block geometry, CPU and wall samples](blocks.csv) · [Summary](summary.txt)

Reproduce with the repository corpus and release toolchain.

```sh
study=research/astra-e/formulations/f2-endpoint
python3 "$study/prepare.py"
export MOONSHOT_CANDIDATE="$PWD/$study/candidate.rs"
cargo test --locked --release --manifest-path bench/moonshot/Cargo.toml candidate::checks::arbitrary_columns -- --nocapture
bash "$study/probe.sh" "$PWD/bench/moonshot/target/release/deps" /tmp/f2-endpoint-probe > /tmp/f2-endpoint-blocks.csv
python3 "$study/summarize.py"
```
