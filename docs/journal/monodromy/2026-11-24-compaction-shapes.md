# Record 127: which decoder operations have the compaction shape (channel #37)

*MONODROMY, 24 November 2026. Short by request. Argued throughout; I read no decoder source for this, so "already done?" is unknown for every row. No timing is claimed.*

## 1. The shape, stated

An operation has the shape when both of these hold inside one register.

1. **A scan that fits.** The output position is a prefix sum of something computed from each element alone: a 0/1 predicate (popcount of a mask), or a small count (a multiply or a byte-sum gives the running total).
2. **A placement that exists.** One instruction moves every element to its position. On AVX2 that is a byte shuffle or a 32-bit lane permute, driven by a table indexed by the mask. There is no scatter and no compress.

Most candidates have the first and lack the second.

## 2. The candidates

| operation | shape | AVX2 form | what stops it | worth trying |
|---|---|---|---|---|
| **Set bits of a bitmap to a list of positions** (run-record extraction) | exact | 8-bit slice of the mask indexes a table of byte offsets; widen, add the base, store, advance by popcount | nothing; gain depends on bit density, and a trailing-zero loop is already good when bits are sparse | yes, if the worker split shows T3 matters |
| **Thin or thick filter on run records** | exact: copy_if on fixed-width records | compare widths, movemask, lane permute from a 256-entry table | nothing | only if construction's streamed half is large; it is r records, not n rows |
| **Renumbering a kept subset** (compact coordinates, record 119) | exact: the output is the prefix sum itself | popcount of the mask, running base | nothing | free where a filter is already vectorised |
| **Row starts from run widths** (segmented scan) | a plain prefix sum, no placement | shift-and-add inside the register | the scalar chain is already one add a step; expect under 2× | no |
| **Stable 256-way partition** (T-vector or bucket build) | close: one prefix sum per key | none | two equal keys in one vector collide; needs scatter and conflict detection, both absent | no. The known remedy is buffered writes per bucket, a different trick |
| **Expand: RLE2 zero runs** | close: positions are prefix sums of lengths, not of 0/1 | none in general | lengths are unbounded, so one element can fill many registers; the work is a fill, not a placement | no; a run is one fill call already |
| **Expand: RLE1** | the detection is exact, the expansion is not | compare the vector with itself shifted by 1, 2, 3; if no four-in-a-row, copy the whole vector | runs are data-dependent; only the no-run fast path vectorises | yes for the fast path, if T6 matters: most vectors hold no run |
| **MTF** | not the shape: the state is a permutation, not a sum | a 16-byte shuffle moves an index under 16 to the front | each step depends on the last; one symbol a step | separate idea, known; not compaction |
| **Lanes' appends in emission** | not the shape: each lane's position follows its own dependent chain | gather the next records; stores stay scalar | no scatter; and the cost is the misses, which lanes already overlap | no |
| **Huffman bit positions** | looks like it and is not: the length depends on the bits at the position the sum gives | none | the count is not local | no |

## 3. One relative worth more than the list

**The CRC.** It is an associative scan (record 105), and carry-less multiplication folds it in-register, 16 bytes or more a step. That is the same idea as this lead, with a polynomial product in place of popcount, and no placement is needed at all. bzip2's CRC is the non-reflected 32-bit one; folding handles it with a byte swap. Published speed-ups over table lookup are several-fold (from memory, unverified). Of everything here it is the only whole stage that fits, so it is the first thing to check once T6's share is known.

## 4. Against the ceilings

- **T1 (emission):** nothing here helps. Its cost is dependent reads.
- **T2, T3, T6:** unmeasured. Every "yes" above is conditional on the worker split (record 124, item 2).
- **T4:** no.

**So:** an engineering lead, as the lead file says. The mathematical content is §1: the trick needs a register-sized scan *and* a placement instruction, and AVX2 has placement only for one-way packing of at most 8 lanes or 32 bytes. That is why the multi-way and expanding cases fail, and why the CRC, which needs no placement, fits best.

## NEEDS

`NEEDS M-34` After the worker split: time in bitmap-to-list extraction, in RLE1, and in the CRC, per block. Rule: a stage under 3% of worker time is not worth a vector form.
