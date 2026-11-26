# Joint spellings across destination components

The multi-marker decoder previously discovered each destination component's tagged dictionary separately. A common suffix can instead have one shared trie node carrying an ordered list of its live source intervals, one per destination color. Within that list, the decoder can reuse the original run index when adjacent colors touch the same original run. This retains the holes in the section and its original marking rule.

Stable LF maps the positions of a fixed byte monotonically. Thus clipping every interval against the original runs and stable-concatenating equal-byte pieces within each color produces the exact contiguous interval for that color in the child. The different surviving colors remain in sorted order, possibly touching. After a positive extension u, all active rows begin with uP and have another P occurrence at offset |u|. If uP begins with P, the first positive parent return is already known from that prefix. Its membership in the selected set is therefore uniform across the active colors. Otherwise the child continues. Terminals retain their actual source intervals and destination tags. A mark in a hole has to follow the ordinary marked rewind rule.

Let Q be the sum of proper-suffix counts over colors, including each active empty suffix. Let D be the number of distinct uncolored proper suffixes. Even free routing between colors can save at most Q−D distinct node expansions through this sharing alone. For each suffix, one original run probe can serve all participating colors intersecting that run. Reusing the previous color's final run reduces both the count of paid starting searches and the number of distinct occupied run pieces by exactly the number of successful reuses. Traversing a hole can cost extra searches; this implementation jumps to the next occupied interval. It still constructs color lists and tagged source intervals, pays for each original run boundary within an interval, and retains the individual path nodes to recover terminal spellings.

On all ten systematic blocks the constructed tagged source intervals and words matched an independent affine-band decoder at k=1,2,4,8. At k=8, Q was 1,140,103 and D was 1,113,730. The strongest possible reduction in nodes from sharing was 2.31%. The implementation avoided 1.70% of its starting searches, with 1,264,424 distinct occupied run pieces left. Sharing terminal byte strings saved 16,391 of 1,366,833 bytes. At k=2 even the upper bound on saved expansions was 0.019%. The result prices this specific direction of trie sharing on these inputs. It leaves open constructors that omit or recover long suffixes by other means. These are structural counts, with no complete-worker throughput estimate. [All 40 real constructions](wikidata.csv) · [4,513 small subset checks and pooled counts](check.txt)

From a checkout with the ten [source tapes](../../../../dense/README.md) available, the replay is:

```sh
d=research/astra-e/formulations/f2-heaviest/union/joint
python3 "$d/count.py" research/astra-e/dense/target/wikidata > /tmp/astra-joint.csv
cmp /tmp/astra-joint.csv "$d/wikidata.csv"
python3 "$d/check.py"
```
