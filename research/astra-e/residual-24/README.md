# Three-byte links for the measured combined residual table

The target-verified writer 80d8409 emits 25,484,201 marked residual visits over its 228 canonical admissions. Residual emission used 38.39% of its measured native inverse stage. The constructor assigns one word code to a contiguous destination interval for each band, while its packed successor varies within that interval. A three-byte record can preserve every next rank and the low (24-b) label bits, where b is the current successor width. A page can supply the remaining high label bits. On a uniform-high page its header is the high value. A mixed page points to one byte of high bits per row; unsupported wider codes or overflowing directory indices can use the incumbent representation. Reconstructing the label needs no access to the next row. Its full successor is available from the three-byte record. This differs from literal-pair contraction and dictionary anchors, which redirect marked edges or stage intervening literal paths.

All 228 actual marked inverses agreed with Segmented at 205,197,794 coded bytes. With 16 rows per page, storing the 3-byte records, a padding record for an unaligned final read, a 2-byte header per page and byte exceptions for every mixed page requires 83,201,465 logical bytes summed across these blocks. The incumbent uses 101,936,804. Mixed pages hold 13.98% of marked visits. Every high label fitted in a byte and every real exception page index fitted below 32,768. The 810,224 original destination bands touched 2,352,303 pages/fragments, which must be charged when deriving headers. At 8 rows, mixed visits fall to 7.69% and total bytes rise to 84,783,873. At 32 rows they are 21.58% and 83,546,843. These are logical complete edge storage sizes before capacity rounding, and exclude other kernel data. They motivate a paid native experiment; they do not establish a time or RSS improvement. [Full per-block census](census.csv) · [All page sizes](summary.csv) · [Measured phase costs](../narrow-short/cost/README.md)

```sh
d=research/astra-e/residual-24
PAGE_DEPS=/path/to/release/deps bash "$d/census.sh" /tmp/astra-e-edge-pages
python3 "$d/sum.py"
```
