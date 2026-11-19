# An ordered delimiter-token root with one mutable run cell per final run

The usual F2 inverse merge allocates a run stream for every retained trie node. Its Θ(ΣᵥR(v)) materialization cost follows from that choice. The ownership of stable selectors gives a constructive alternative. Each nonroot child's stream belongs to one parent. Multiple occurrences of the same child in the parent selectors consume successive, nonoverlapping prefixes and exhaust that stream exactly. Consequently a parent can take the cells of a child destructively.

**Invariant.** Give each terminal selector one run cell. The cell stores the terminal's dictionary-node ID, row width and singly linked successor. The active output of a node has a head, tail and remaining row count. A parent takes a full remaining child in constant time from head and tail. For a partial selection it follows successors up to the cut; a cut on a cell boundary needs no new cell. An interior cut creates exactly one new suffix with the same terminal ID and the remaining width. Append the selected list to the parent's output by relinking its tail. The ordered selectors and original stable prefix lengths are unchanged, proving the resulting labels in every row are the original token BWT.

**Exact output bound.** Adjacent distinct selectors at a node cannot contribute the same label: a terminal emits the current word; each child produces disjoint longer words distinguished by the left extension byte. Adjacent equal selectors were already coalesced during discovery. Adjacent cells cut by different selectors of the same child go to distinct, nonadjacent occurrences in its unique parent. Each terminal allocation and each split therefore survives to the completed root, and every root run has one cell. There are exactly R(root) mutable cells independent of trie depth. The implementation flattens the final list into R(root) eight-byte entries for the unchanged second-layer input. Logical merge storage is 12R(root) list bytes plus 8R(root) final vector bytes, in addition to the existing trie, selectors and allocator capacities. Node cursors reuse three original fields for list metadata. Time also pays all selectors, the prefix searches for nonterminal partials, and one final traversal. A partial can visit several cells; this is an entry-space bound.

**Complete census.** On all 228 admitted large blocks, every root run ID and row width matched F2c's explicit inverse merge. The two-level expansion then agreed with Segmented over 205,197,794 bytes. There were 788,908 terminal allocations and 997,564 splits, exactly the 1,786,472 final runs, against 5,390,514 explicit node runs in F2c. The constructor executed 987,454 whole and 2,537,214 partial prefix selections; the latter read 3,326,892 linked cells. Frozen had 2,898 terminal allocations, 3,235 splits and 11,374 partial reads for 6,133 final runs. [Native root and byte audit](geometry.csv) · [Algorithm](merge.rs)

**Pricing.** Across nine equally weighted blocks and 21 rotated phase measurements each, the sum of per-block median merge CPU times rose from 1.826 ms to 2.223 ms. Complete inverse CPU in paired round sums was 1.033 [0.887,1.203] versus F2c and 1.390 [1.263,1.529] versus Lazy. This tests the mutable linked representation under the unchanged F2c two-level finish. It establishes exact root-size storage, with measurable costs in link traversal and the adapter's final flatten. [Raw phase observations](phase-cpu.csv) · [Summaries](phase-summary.txt)

Reproduce with the canonical corpus and a release research-enabled crate dependency directory.

```sh
study=research/astra-e/formulations/f2d
build=$(mktemp -d)
cargo build --locked --release --manifest-path bench/moonshot/Cargo.toml --target-dir "$build/base"
bash "$study/splice/audit.sh" "$build/audit" "$build/base/release/deps" > "$build/root.csv"
mkdir -p "$build/price"
bash "$study/splice/price.sh" "$build/price" "$build/base/release/deps" > "$build/price/phase-cpu.csv"
python3 "$study/summary.py" "$build/price"
```
