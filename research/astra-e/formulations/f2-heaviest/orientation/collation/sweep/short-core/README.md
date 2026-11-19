# Recovering original return words from a variable prefix and a positive core

For each original return W from s∈U to its next positive visit τ(s), select a maximum h. Cut at depth t(s)=min(h, |W|−1), write W as P_s·C_s, and keep the **nonempty** core C_s. The target object is the original incoming column with its own return labels, destination order and original primary. The constructor can discover a core from the destination while recovering its varying prefix from the source boundary. This permits advancement through different source bytes and arbitrarily short original returns. It does not require a family of cuts whose new words have unique spellings.

Compute the cut positions from the run index. Begin with one packet on U. Each packet maps a consecutive set of old sources to current ranks by translation and shares a prefix. Split it at first-column stable-select interval boundaries. Each image piece is again translated. In a piece whose next rank lies in U, publish the current position as a terminal and keep the current prefix. On every other piece, record the reached ranks and append the known outgoing byte. At depth h publish all surviving packets. A piece straddling an endpoint of U splits there. The published records carry the original source base, terminal base, width and prefix. Thus discovering the applicable depth and short returns needs no dense first-return table. [Constructor and independent physical checks](induce.py)

**Bounds.** Every row reached at a strictly positive successful depth is inside its own original return arc. All attempted source ranks, across all packets and all depths, are distinct because the original positive arcs partition their intersected cycles and the forward construction stops before entering another arc. The attempted image ranks are also distinct by injectivity. An original first-column select-run boundary therefore splits a packet at most once across the entire construction. The two endpoints of U can contribute at most two more interior splits in the attempted images. Starting with one packet, the final number of source-affine terminal bands is at most r+2 for r original BWT runs. The total number of visited packets can exceed the number of final bands; each depth still has to advance the live packets. The reference pays for those advances, original run indexing and storage of the hidden ranges. On a given block the total number of attempted rank occurrences is at most the number of rows on cycles meeting U.

Let F contain all reached ranks at positions 1 through t(s) on each original arc. Every old source in U remains outside F. From any surviving row, one LF step reaches either another surviving row or a terminal: if the image had been a strictly interior deleted position, its forward successor would itself have been deleted. A rank at t=0 is a live root on entry and an accepting terminal on positive arrival. Each original backwards proper-suffix interval loses precisely its members in F. Deleting F from the globally ordered universe therefore makes the surviving part of each old suffix interval contiguous. It leaves the original U roots in order, even with adjacent sections or short/periodic returns. The backward distance from an old destination to its tagged terminal is |C_s|>0. Reading the prefix and original source at that terminal reconstructs W=P_s·C_s and its exact stable partner. Cores of different words can be equal; the prefix/source tag must survive until assembly of the original token type. The independent source/primary handling of the original token column then applies. Marked cycles missing U still require their original fallback.

The choice t(s) depends only on the original word. One pair (P_s,C_s) names that type everywhere and concatenation recovers it. A dictionary that deduplicates prefixes and cores separately uses at most the original distinct spelling bytes, before its indices and tables are charged.

All 22,520 small section/depth cases matched every source and incoming spelling under direct reverse LF, including 14,696 with an early terminal. On the ten real tapes, every published source, cut rank and prefix was independently verified for each occurrence in six section/depth choices per tape. The real test prices source construction and the distinct core suffix nodes. It does not time a full native inverse. [Small receipts](small.json) · [All 60 real cases](wikidata.csv)

| Section and maximum depth | Old suffix nodes | Core suffix nodes | Forward run intersections | Terminal bands | Old dictionary bytes | Prefix + core spelling bytes |
|---|---:|---:|---:|---:|---:|---:|
| Quote, 3 | 908,546 | 810,427 | 69,166 | 36,351 | 1,077,638 | 982,112 |
| Quote, 8 | 908,546 | 679,088 | 201,689 | 60,000 | 1,077,638 | 920,407 |
| Quote, 16 | 908,546 | 538,445 | 380,395 | 78,177 | 1,077,638 | 1,006,268 |
| Frequent, 3 | 1,296,362 | 1,181,129 | 10,235 | 8,140 | 1,926,457 | 1,810,543 |
| Frequent, 8 | 1,296,362 | 1,005,990 | 111,815 | 25,503 | 1,926,457 | 1,639,676 |
| Frequent, 16 | 1,296,362 | 785,959 | 416,524 | 49,252 | 1,926,457 | 1,628,058 |

The frequent section at depth eight saves 290,372 backward lookup states and pays 111,815 forward run intersections. The two operations have different costs. Index projection, terminal label reconciliation, output and peak memory remain part of the complete test. At depth sixteen the original quote returns save fewer lookup states than the number of new forward intersections. That is a limit of this construction's balance; an operation count alone gives no wall ratio.

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/short-core
python3 "$d/induce.py" | diff - "$d/small.json"
python3 "$d/induce.py" research/astra-e/dense/target/wikidata | diff - "$d/wikidata.csv"
```
