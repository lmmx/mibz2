# Three-byte links for the measured combined residual table

The target-verified writer 80d8409 emits 25,484,201 marked residual visits over its 228 canonical admissions. Residual emission used 38.39% of its measured native inverse stage. The constructor assigns one word code to a contiguous destination interval for each band, while its packed successor varies within that interval. A three-byte record can preserve every next rank and the low (24-b) label bits, where b is the current successor width. A page can supply the remaining high label bits. On a uniform-high page its header is the high value. A mixed page points to one byte of high bits per row; unsupported wider codes or overflowing directory indices can use the incumbent representation. Reconstructing the label needs no access to the next row. Its full successor is available from the three-byte record. This differs from literal-pair contraction and dictionary anchors, which redirect marked edges or stage intervening literal paths.

All 228 actual marked inverses agreed with Segmented at 205,197,794 coded bytes. With 16 rows per page, storing the 3-byte records, a padding record for an unaligned final read, a 2-byte header per page and byte exceptions for every mixed page requires 83,201,465 logical bytes summed across these blocks. The incumbent uses 101,936,804. Mixed pages hold 13.98% of marked visits. Every high label fitted in a byte and every real exception page index fitted below 32,768. The 810,224 original destination bands touched 2,352,303 pages/fragments, which must be charged when deriving headers. At 8 rows, mixed visits fall to 7.69% and total bytes rise to 84,783,873. At 32 rows they are 21.58% and 83,546,843. These are logical complete edge storage sizes before capacity rounding, and exclude other kernel data. They motivate a paid native experiment; they do not establish a time or RSS improvement. [Full per-block census](census.csv) · [All page sizes](summary.csv) · [Measured phase costs](../narrow-short/cost/README.md)

```sh
d=research/astra-e/residual-24
PAGE_DEPS=/path/to/release/deps bash "$d/census.sh" /tmp/astra-e-edge-pages
python3 "$d/sum.py"
```

## Complete paid kernels

The P=8, 16, and 32 implementations construct their final table directly from the original band's source queue slices. A first pass over band intersections classifies pages. Uniform headers store a high label. Mixed headers store a tagged 15-bit exception-page index; a second pass fills the low code and successor for every row and the high bytes for mixed fragments. The existing wide-code path and blocks that exceed the 15-bit exception count use the original representation. The final record has initialized padding for the checked unaligned 32-bit load. The walker produces its next index from the low b bits, independently looks up the page's label, and emits the same first marked cycle and final repetition.

All 25,484,201 residual slots in the 228 admitted blocks had their complete ordinal and label checked against the frozen uncompressed table in each format. All resulting marked inverses agreed with Segmented. An independent stable-sort test covered 11,065 marks on arbitrary short and longer run-heavy columns, including one-row marked periods and multiple cycles. A synthetic overflow of the header's 32,768 exception-page capacity refused cleanly; reuse for a subsequent small directory succeeded. The old table's retained maximum was 1,437,984 bytes. Actual maxima for P=8/16/32 were 1,197,980 / 1,176,564 / 1,181,612 bytes, with smaller table storage on every canonical admission. Shared endpoint and output memory is outside those table measurements. Wide fallback may retain a previously allocated packed scratch on other inputs. These data do not establish whole-worker RSS.

| Complete inverse thread CPU vs frozen 80d8409 | CPU 4, 64 balanced laps | CPU 2, independent 64 laps |
|---|---|---|
| Page 8 | 1.2458 [1.2032,1.2898] | 1.2481 [1.2120,1.2854] |
| Page 16 | 1.2414 [1.2007,1.2835] | 1.2609 [1.2210,1.3022] |
| Page 32 | 1.2227 [1.1833,1.2633] | 1.2415 [1.2056,1.2785] |

These measurements start before run and endpoint construction and end after reconstructing the full marked raw block. The same eleven prespecified source blocks were used in both processes; every timed output was checked. P=16 preparation ratios were 1.173 / 1.161, and marked emission ratios were 1.342 / 1.429. The edge capacity reduction did not recover the time for encoding, split labels and lookup. The 32-bit combined/edge writer remains the target-speed reference. This experiment retains exact lower-table-space alternatives with known local throughput regressions. [Raw phases and intervals](price-summary.csv) · [Verified retained capacities](capacity-summary.txt) · [Properties](check.txt)

```sh
d=research/astra-e/residual-24
PAGE_DEPS=/path/to/release/deps bash "$d/run.sh" /tmp/astra-e-residual-24
PAGE_DEPS=/path/to/release/deps bash "$d/overflow.sh" /tmp/astra-e-residual-overflow
python3 "$d/report.py" /tmp/astra-e-residual-24/{first,repeat}.csv
```

The compiled standalone scorer entries are p8.rs, p16.rs and p32.rs. No change to the shipping decoder is needed. The separate census and native checks preserve the exact historical geometry and price both passes before the marked walk. No worker or target timing was requested for these candidates.

The P16 lower-table-space candidate at aed11af2d257006b87f6bf67a7655e217e26d6d7 passed the refreshed authoritative v1 gate on all 338 cases, including periodic and multiple-cycle marks, full Wikidata and malformed stream/CRC cases. Master was fetched at be74d5763d212864607d46521d74ebc0827f9f36, with scorer path revision 75297a1d57b2bdf262c3a19f3064848dedf2c493. The scorer printed v1 at 81585172f5fb9e7a59f330124779ba084fd6bf04. [Full gate receipt](official/score/summary.txt) · [Immutable candidate and scorer manifest](official/manifest.txt) · [Refresh and reproduce](gate.sh)
