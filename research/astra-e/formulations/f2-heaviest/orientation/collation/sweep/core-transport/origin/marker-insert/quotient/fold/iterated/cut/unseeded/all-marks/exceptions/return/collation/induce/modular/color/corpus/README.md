# Materializing the first-return map after level coloring

Across ten original Wikidata blocks, the level coloring removed 33.26% of rows. The historical matching cut removed more rows on every block and produced 7.58 times fewer normalized affine spelling bands. For the level coloring, dropping all spelling distinctions merged just 12 of 4,239,792 bands. The location of the successor governs this comparison.

|Cut|Removed rows|Physical removed spans|Joint source/image atoms|Affine successor bands|Affine spelling bands|Distinct return words¹|Spelling bytes¹|Longest return|
|---|---:|---:|---:|---:|---:|---:|---:|---:|
|Level, greedy|2,993,038|2,533,537|7,396,242|4,239,780|4,239,792|16,873|32,873|2|
|Type, greedy|3,025,271|92,564|740,370|567,598|567,599|5,541|10,229|2|
|Type, matched|3,170,769|84,178|724,041|559,693|559,695|5,652|10,484|2|
|Physical rank modulo two|4,499,958|4,499,958|8,999,915|512,577|512,625|111,841|452,406|20|

¹ Deduplicated exactly within each block using the entire ordered byte string including its length, then summed across blocks. There were 8,999,915 source rows and 690,484 maximal original L runs. Physical spans are counted on the linear array. Joint atoms split each original run at every physical or image retained-status change. They measure work for an explicit interval sweep. Parity admits a direct arithmetic selector despite its large atom count. A normalized affine successor band is a maximal source-rank interval with constant target minus source. Affine spelling bands additionally require an equal return word. These are exact minimum piece counts for that specific representation.

## Work and storage

The 841,933 run/image tiles across the ten inputs certify a representation smaller than one record per row. A mask on one or two coordinate bits and a cumulative retained count per tile support direct rank, select and first return. Constructing the first layer can avoid enumerating the 7.40 million joint atoms. The actual tile and piece vectors use 48 live bytes per tile on this build. Including reserved capacities, the sparse graph and both owning objects, the persistent footprint ranges from 3,000,880 to 10,193,800 bytes per block. The frozen block uses 3,067,072 bytes for graph and map (62,138 tiles) and emits 426,528 affine spelling bands. These numbers exclude the original runs, temporary constructor allocations, input and output. They are representation counts, not measured peak RSS.

The supplied direct map uses four binary searches per retained source with a one-hop return and five per two-hop return with assertions compiled out. A full marked lap thus uses 27,020,546 searches with this implementation. Caching repeated tile lookups or storing a succinct dense mask changes that work/storage choice. The observed minimal *explicit band* count says nothing about a lower bound on all possible implicit representations. The pair-weight solver and the two-run worst case are in the parent directory. No wall time was measured and no worker was changed.

## Verification and reproduction

The census expands size-127 capped records, hashes both input and committed-oracle pre-RLE1 bytes, then computes original LF by stable occurrence rank. For the greedy level cut it checks the analytic map at every row, every survivor rank and every normalized return target and spelling. It also checks the exclusive rank at the end. For all four arms the induced map is a permutation, its complete marked lap visits every survivor, and emitting its backward spellings reproduces every raw byte of the oracle. Rows in a fixed point cannot be removed by the level rule. These ten marked originals are primitive; the parent's finite suite covers multiple cycles.

The type controls call the original seed, greedy and matching routines at b4b43506. Seed splits the primary's F bucket to protect the singleton. Each physical target is removed on an entering oriented edge. Its source-slice mask reproduces both published ingress counts on every block. The modulo-two control retains the primary's physical parity. Its 2,255,298 consecutive removed edges account for its longer words. For all controls the zero map-storage fields mean that the level-map layout was inapplicable. The corpus receipt is the capped original-block experiment rooted at 923f215; color.hpp was frozen at 1684903d. The recorded counts include the per-arm primary rank and all output lengths in blocks.csv.

```sh
p=$(dirname "$(rg --files | rg '/modular/color/corpus/panel.py$')")
data=/workspace/work/astra-powers-original/capped
out=$(mktemp -d)
python3 -B "$p/panel.py" "$data" "$out"
diff "$p/blocks.csv" "$out/blocks.csv"
```
