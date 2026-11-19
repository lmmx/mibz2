# Linear sparse publication in image order

The linear publisher reproduced every byte of all thirty archived weighted columns, including each marked phase, and every original marked output. At depth eight it used **162,511 key equality checks and 254,718 band-index scatters** across ten blocks. The map-based reference used 2,923,961 ordered-key comparisons, 1,721,250 destination-range comparisons and 1,542,131 class-base sort comparisons on those same objects. The linear implementation materializes a 12-byte descriptor for each of Q refined fragments. Its largest per-block fragment allocation was 393,216 bytes at depth eight. Neither implementation has a measured i9 time or memory advantage. [Raw block counts](radix.csv) · [Equivalent complete objects](../columns.tar.xz)

| Depth | Q fragments | Key equality checks | Band-index scatters in two radix passes | Total logical fragment bytes | Maximum per-block allocated fragment bytes |
|---|---:|---:|---:|---:|---:|
| 4 | 322,322 | 321,498 | 538,894 | 3,867,864 | 1,572,864 |
| 8 | 163,280 | 162,511 | 254,718 | 1,959,360 | 393,216 |
| 16 | 125,900 | 125,170 | 202,550 | 1,510,800 | 393,216 |

1. Stable radix-sort the band indices by destination start. The band images partition the original true-exception ordinals. A single forward sweep records the starting destination-byte range for every band.
2. Visit bands in their original source order. Split against source descriptor ranges and destination-byte ranges, starting from the saved destination cursors. Record the width and two descriptor indices per fragment. The source cursor is monotone through the entire operation. The earlier array holding destination start cursors now holds fragment-array offsets.
3. Revisit each band's contiguous fragment slice in increasing band-image order. In each byte bⱼ, the shifted forward-source ordinals now appear in strictly increasing order, because the shift is stable within that byte. The cylinder theorem makes all occurrences of a given class consecutive in that shifted source order. On a key change within a byte, the current shifted ordinal is the new class's exact base. Accumulate widths and store the class ID in the fragment slot previously holding the destination descriptor. Keep the last class and a class-ID list per byte. Concatenating the lists gives the complete class order. Keys include bⱼ, so no class spans two such lists.
4. Scan fragments once in original source order and append class IDs and widths to the incoming bᵢ queues. Each queue receives the correct shifted forward-destination order. Concatenate the queues and merge adjacent runs. Build the same per-class select directory and marked writer as the reference. [Kernel at 7c262538](kernel.inc) · [Original proof and marked phase](../README.md)

The fixed-width implementation uses up to four 10-bit radix digits for 32-bit ordinals, with a smaller final digit. All thirty real cases used two. Each pass performs a band histogram and a stable scatter; the table counts the scatter items once per pass. Elsewhere each band, descriptor boundary, class, fragment and incoming run has bounded work per pass. The publication stage therefore takes O(B+S+D+Q+K+R) work on this fixed-width machine, plus the bounded radix histograms and initial alphabet tables. The full pipeline also includes physical context discovery, first-return induction, the mark splice, select construction and output. On the original ten depth-eight inputs the prior induction alone performed 2,350,011 packet translations.

Additional live storage includes two B-index arrays for the radix pass; after the radix scratch dies, there are B image-order indices, B+1 fragment offsets, a growable array of Q triples, class records and per-byte class lists, per-byte incoming records, and the final select directory. The reported fragment allocation measures capacity after growth. Reallocation may transiently hold old and new buffers simultaneously. No process RSS or terminal wall time was measured in this structural test. The independent literal audit materializes dense arrays and words solely for checking; the publisher and marked writer operate on compressed representations. [All counts and capacities](radix-summary.json)

Finite replay checked 9,837 arbitrary models and all 63,972 marks. The real gate checked every key, word, incoming symbol, forward partner and marked byte against the literal oracle, parsed original inputs, and compared the complete serialized column against the previously archived SHA-256 for each block and depth. Reproduction uses the same authenticated raw inputs as the parent publisher. [Finite gate](radix-small.csv) · [Build identities](radix-build.json) · [Authenticated objects](../inputs.json)

```sh
work=$(mktemp -d)
python3 run.py "$work" build
python3 run.py "$work" small
python3 run.py "$work" real /tmp/astra-exception-contexts/raw /tmp/astra-clock-wires /workspace/work/astra-powers-original/capped
cmp "$work/radix-small.csv" radix-small.csv
cmp "$work/radix.csv" radix.csv
cmp "$work/radix-summary.json" radix-summary.json
```
