# Private-cache successor pages at the original width 16

On the measured edge worker's 233 Wikidata admissions, independent 32-row displacement pages and 16-row numeric label pages store the same 26,567,581 marked edges in 49,915,971 bytes including all block headers. The frozen packed-edge layout uses 106,270,324 bytes. With the original spelling dictionary and fourteen guard bytes per block, 145 blocks fit the 256 KiB footprint test and six fit the 192 KiB test. These blocks account for 130,494,775 and 5,399,889 final output bytes respectively. The original width-16 representation had zero such fits. The quantities describe addressable data during emission. The constructor and its retained buffers have a separate cost. [Exact 699-row census](census.csv) · [Coverage and totals](summary.csv) · [Frozen structural and native plan](plan.md)

## Measured structure and complete representations

Each physical page uses the original successor rank y and original numeric word code. At P=32, 675,185 of 830,348 pages have strictly increasing successors throughout. Only 204,982 of 25,737,233 within-page transitions descend, and 11,833,272 advance by exactly one. A single descent can spread a page widely in rank space. These counts motivate separately bounded positions and exact raw fallbacks.

| Successor size | Successor bytes | Numeric label bytes | Total with headers | Any-integer-slope floor for this format | Complete 256 KiB fits | Complete 192 KiB fits |
|---:|---:|---:|---:|---:|---:|---:|
| 16 rows | 44,703,347 | 8,700,626 | 53,413,293 | 44,426,259 | 105 | 3 |
| 32 rows | 41,206,025 | 8,700,626 | 49,915,971 | 41,017,092 | 145 | 6 |
| 64 rows | 40,737,605 | 8,700,626 | 49,447,551 | 42,826,853 | 146 | 6 |

The two global bitpacked channels take 94,847,265 bytes including the same block headers. Globally bitpacked successors use 57,436,659 bytes. Combining those with only the independent numeric label pages gives 66,146,605 bytes including headers, and 13 or one complete private-budget fits at 256 or 192 KiB. Local successor modeling therefore contributes both storage and applicability beyond compressing the labels. The prior residual24 P16 approach on these same tapes takes 86,683,272 bytes, including its per-page headers, mixed-label page payloads and padding. It preserves the full next rank in each three-byte low record. Its measured local stage already showed the CPU price of extra encoding and label lookup. [Historical comparison](../../residual-24/README.md)

**Format.** For each successor page, subtract slope times the local index and fit the remainder to an integer interval with arbitrary base. The four slope candidates are zero, one, the rounded endpoint slope, and median adjacent difference. For k exceptional positions, test every way to trim k entries from the sorted ends. This yields the minimum possible regular bit width at that slope and k. Try k from zero through the smaller of eight and one quarter of the page. A 32-bit page offset and byte tag locate a raw page or a predictor. The predictor stores a width byte, a four-byte signed base and, for general slope, a four-byte signed slope. Exceptional pages also store a bitmap, packed regular residuals and packed absolute exception ranks. One bitmap population count locates either field. A per-page raw choice and a blockwide global choice preserve arbitrary inputs.

Numeric codes are accessible independently at the current row. A four-byte 16-row page entry holds a constant code or the offset of a byte-aligned payload. The payload has a one-byte tag. A raw payload packs all code values at the blockwide numeric code width. A palette payload stores its distinct numeric codes and their fixed-width row indices. The block can also use a single global packed code column. The ten 32-bit header words cover row count, mark, coded output length, the two numeric widths, successor page width, both encoding modes and both section lengths. The experiment retains the existing dense code namespace and charges the whole spelling and offset tables in the cache-footprint result.

**Why the floor is valid.** For any fixed stride h, the difference y(i+h)−y(i) for two regular endpoints equals slope × h plus a difference between two fitted residuals. If their residual range is R, the spread of surviving differences is at most 2R. Each exception can delete at most two pairs for that stride. Sort the differences and find the narrowest interval after removing up to 2k entries. Half its spread, rounded up, lower-bounds R for *any* integer slope. Take the strongest bound from strides 1, 2, 4 and 8, price the implied residual bits with the bitmap, all exception values and the cheapest legal predictor header, then minimize across k and both raw fallbacks. The label section stays independently addressable and fully charged. This floor constrains the specified formats and exception cap. The census verifies it against every attained page and separately checks 1,312,541 exhaustive small slope/exception comparisons over 5,913 permutations. [Floor validation](floor-check.txt) · [Executable derivation](census.cpp)

## Native implementation and working set

The P32 experiment first materializes the frozen worker's original edge, scans numeric codes, selects page representations, and builds the two byte pools and directories. It clears the superseded original edge lengths and retains their capacities for reuse. All recognition and encoding belong to construction. For each marked step the decoder reads an independent label and reconstructs the destination using the current page. It starts the next lookup before appending the current literal or spelling. The original clamp and marked-cycle repetition produce the requested output length. Each nonempty byte field allows an eight-byte extraction across the final byte through seven guard bytes at the end of its pool. Zero-width fields return zero directly. The census adds fourteen guard bytes per block when testing cache fit. [Native generator](native.py) · [Encoder, decoder and storage accounting](native.inc.rs)

For frozen block 1 the native live edge data total 170,418 bytes. The edge and compressor retain 548,048 bytes including original edge capacity and format metadata. The spelling and offsets occupy an additional 57,343 live bytes and retain 73,728 bytes. During page selection the implementation uses fixed stack arrays of 32 successors, up to 31 differences, 32 residuals and four slopes. Code palette construction uses two 16-element arrays. The storage method charges owned vector capacities and the extra state of the compressed representation. Its measurements exclude those call-stack temporaries and unchanged common constructor data. Fitting the active walk therefore does not entail a reduced worker peak. The common stage comparator charges compressed construction, marked emission and both original and compressed retained tables. [Native smoke receipt](native-smoke.txt) · [Common benchmark recipe](../price-plan.md)

The native-format audit reproduced every original successor and numeric code across all 26,567,581 tape rows. Every per-block size was exactly the independently implemented census result plus the two guard areas. Another 2,376 checks covered separate wide codes, single and multiple cycles, partial pages, reversed and randomly permuted successors, buffer reuse and marked repetition. The generated complete inverse matched Segmented for all 209,697,662 admitted coded output bytes. [Native format verifier](audit.rs) · [Raw audit](audit.txt)

From the repository root, restore the input edge tape using the parent experiment's committed compressed artifact and tape recipe. The census requires a little-endian host.

```sh
d=research/astra-e/edge-displacements/local
g++ -std=c++20 -O3 -Wall -Wextra -Werror "$d/census.cpp" -o /tmp/astra-local-census
/tmp/astra-local-census --check
nice -n 16 /tmp/astra-local-census /tmp/astra-edges.bin > /tmp/astra-local-census.csv
cmp /tmp/astra-local-census.csv "$d/census.csv"
rustc --edition 2021 -C opt-level=3 -C overflow-checks=yes "$d/audit.rs" -o /tmp/astra-local-audit
/tmp/astra-local-audit /tmp/astra-edges.bin "$d/census.csv"
python3 "$d/report.py" > /tmp/astra-local-summary.csv
cmp /tmp/astra-local-summary.csv "$d/summary.csv"
```

The restored tape SHA-256 is `670b97cbe1f6ba513df848ba68e428e9004a80bf74807254e507fb263ebf0697`. The census SHA-256 is `fb248ff534774c466f88d99251edd7738e3a08f97b33427da146381dbe6e80c3`.
