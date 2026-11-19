# Locate exception rows and recover the marked block from predecessor contexts

For each fixed k, a state is the k-byte forward prefix of a cyclic rotation. Its most frequent preceding byte determines one backward transition to another state. Given the starting state, an ordered exception event with a true byte and a new state overrides this default. The diagnostic obtains ground-truth intervals and clocks from true marked rotations. It then takes only the unordered physical exception ranks and the primary into the delimiter hierarchy. At every ordinary cut it coalesces duplicate requested ranks in the next level. In the reverse direction it computes coordinates only on these selected ranks. Every recovered clock agreed with the oracle. Finally a fresh byte-by-byte state walk restored the entire original marked word using the state table and ordered events on all 80 block/order combinations. [Exact source](census.cpp) · [All 2,000 depth records](depths.csv)

Over ten original columns, k=8 requires **409,432 exceptions among 8,999,915 rows**. Locating those exceptions and the ten primaries takes 0.326 distinct live coordinates per original row, summed over all 25 levels. The state tables contain 815,844 contexts in aggregate, and 81.3% of output can use complete eight-byte default transitions between exceptions. At k=16, there are 229,013 exceptions, 1,170,188 contexts and 90.8% eight-byte coverage. The reference checks all physical clock coordinates and every replayed byte. Its dense tables, stable sorting, direct-access helper maps, true marked-coordinate oracle and sorting/deduplication of live requests are charged as structural machinery. Sparse counts alone give no runtime bound for this reference implementation. Packed event data uses nine bytes per exception, five per unit-state table entry and 24 header bytes per block. A jump table, output, context construction and clock acquisition add further costs. [Full exact sweep](blocks.csv) · [Totals and logical storage](summary.csv)

The ten pre-RLE streams were independently recovered from their original compressed blocks and matched through RLE1 and stored block CRC in the preceding coordinate experiment. This fixed-input clock replay covers the same pre-RLE bytes. Arbitrary marked/heterogeneous horizons are covered separately by the general theorem and exhaustive tests. [Original positions and libbzip2](../../backwards/) · [Marked clock theorem](../PROOF.md) · [Physical-rank jumps avoiding ordered clocks](../jumps/PROOF.md)

```sh
d=$(dirname "$(rg --files | rg '/exceptions/panel/PLAN.md$')")
g++ -std=c++17 -O3 -Wall -Wextra -pedantic "$d/census.cpp" -o /tmp/astra-clock-census
tmp=$(mktemp -d)
python3 -B "$d/panel.py" /tmp/astra-clock-census research/astra-e/dense/target/wikidata "$tmp" "$tmp/depths.csv" "$tmp/models" > "$tmp/blocks.csv"
diff "$tmp/blocks.csv" "$d/blocks.csv"
diff "$tmp/depths.csv" "$d/depths.csv"
python3 -B "$d/report.py" "$d/blocks.csv" | diff - "$d/summary.csv"
```

If the given wire directory already contains the pinned numerical fixtures, the script authenticates and reuses them. Otherwise it reconstructs them from original verified records. The optional final directory receives k=4,8,16 serialized predictors and physical exception masks for independent follow-up experiments.
