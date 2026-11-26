# Bounded select from the original writer's monotone queues

The frozen width-16 writer publishes one band code on each physical destination interval. Within such an interval its next ranks occur in increasing order because the source is a slice of the same-byte retained row queue. Consecutive bands for a byte consume that queue from left to right. This gives an exact reconstruction of the packed successor and the label without materializing a successor for every physical row. The measured edge constructor remains the control at `badb2aec3624bd65606c429b2e6f79e9d9d0050f`. The earlier range-queue experiment compressed working storage for constructing the old packed table. Here the marked walk uses the factored representation itself. [Earlier queue experiment](../../queue-spans/README.md) · [Frozen tape capture](../dump.inc.rs)

## Band location and queue lookup

Concatenate the byte queues in their existing stable byte order, obtaining Q. Suppose band b starts at position k in Q and publishes at physical address d. For every physical row x in that interval its next rank is exactly Q[k+x−d]. Mark every physical band start in a length-m bit vector. A 16-bit prefix count per 64-row page plus one masked population count returns the physical band ordinal of x. For that ordinal save the label and the low B bits of k−d in a four-byte descriptor, where B is the unchanged row width. Adding x modulo 2ᴮ recovers the true Q index because k+x−d belongs to [0,m). The inherited packed-code check guarantees that the label occupies the remaining bits. The operator can recover the location of every original band without a sort. Two linear passes over the live bands first mark the starts, then scatter the descriptors after computing the prefix counts.

Split Q globally into 32-entry groups. A group within a byte queue is strictly increasing. A group crossing a queue boundary is checked for monotonicity and carries full-width raw ranks if it fails. For a monotone group q of length t let b=q[0] and choose the smallest w such that floor((q[t−1]−b)/2ʷ)+t ≤ 64. Store all low w bits of q[j]−b, followed in logical terms by a 64-bit Elias–Fano unary mask with one bit at j+floor((q[j]−b)/2ʷ). These positions increase strictly, even when successive high values coincide. Depositing 1≪j into the mask with PDEP and counting trailing zeros selects its j-th one. Subtracting j recovers the high value. A zero unary mask identifies the raw fallback, whose base is zero. The 16-byte group header contains the packed base/width, low-payload bit offset and unary mask. Each lookup is bounded on both data-dependent paths. The 10900K's BMI2 and POPCNT support the instructions used in this interpolation.

The competing slope representation subtracts the within-group ordinal and stores a per-group minimum plus the complete fixed-width residuals. Packing modulo the original row universe covers groups crossing byte boundaries. Its lookup avoids the high-bit select but retains more low bits. Every layout explicitly recovers the original code and next rank of every physical row, hence preserves precisely the same marked cycles and byte spellings on arbitrary valid partitions. [All physical-edge replays](model.rs)

## Frozen width-16 corpus

The complete edge tape contains 26,567,581 physical rows across 233 actual admissions. Eight independently decoded layouts (two encodings × four group sizes) reproduced 212,540,648 packed edges. For 32-entry Elias–Fano groups, 16,882 groups covering 539,733 entries used a raw fallback. The band locator uses 3,321,880 bytes for masks, 830,470 for actual 16-bit prefix counts, and 3,311,432 for physical band descriptors. Group header fields occupy 13,285,568 bytes including all unary masks. Packed low ranks use 15,993,816 bytes including last-word rounding. The total is **36,743,166 bytes, 1.3830 per row**, versus 106,270,324 bytes for the original edge arrays. The unchanged dictionary and offsets add 13,090,022 bytes to each.

| Representation | Total edge/index bytes | Full bytes with spellings and offsets | Real admissions fitting 192 KiB | Fitting 256 KiB |
|---|---:|---:|---:|---:|
| Unchanged packed edge | 106,270,324 | 119,360,346 | 0 | 0 |
| 32-entry slope | 45,775,870 | 58,865,892 | 9 | 176 |
| 32-entry Elias–Fano | 36,743,166 | 49,833,188 | 80 | 203 |

The native decoder adds one eight-byte sentinel per block for a final unaligned low-field load, preserving both fit counts. The 203 admissions comprise 182,691,444 final bytes and 80.0% of the frozen complete-block CPU census. The 80 admissions comprise 71,997,508 final bytes and 28.13% of that CPU. The 192 and 256 KiB numbers measure initialized marked structures. Output stores, additional working state, cache conflicts and retained Vec capacities all have separate costs. [All 1,864 rows](bucket.csv) · [Eight aggregate comparisons](summary.csv) · [Original width-16 footprints and CPU weights](../../private-cache/README.md)

## Native construction and memory accounting

The source transform applies only to the width-16 queued induced writer with a packed label, a 16-bit band directory and BMI2/POPCNT. It also requires the worst-case fully raw low-payload charge to fit the original four-byte edge budget. All 233 recorded admissions pass this bound. Other cases use their original linked writer, segmented policy or wide-edge fallback. The native builder consumes the existing per-byte queues in order. It feeds aligned 32-entry slices directly and copies the pending cross-queue group into 128 bytes of stack. It never constructs an intermediate full edge table or a concatenated Q array. It visits the bands twice, the prefix directory once and the queued ranks once. The scratch and compressed fields are reused across blocks and cleared on fallback. The original and compressed Vec capacities can both remain after alternating routes. The original production queues, endpoint dictionary, output vector and other phase buffers still contribute to peak memory.

The native marked walker uses one page index to load its mask and prefix in parallel, then loads the packed band descriptor, then its 16-byte group header, then one unaligned 32-bit low payload. Each resulting rank determines the next physical lookup. It reads the successor before appending the current band spelling and repeats a short primary cycle exactly as the incumbent does. This longer chain is a direct reason to price both construction and the full marked emission. Initialized byte count alone supplies no machine speed result. The public audit `displacement_entry` recovers any row's actual (next,code) from the selected route. `displacement_arrays` reports seven pairs of current initialized and retained capacity bytes in order: starts, prefix, descriptors, headers, low payload and sentinel, original edges, original wide labels. `displacement_storage` reports their two sums followed by initialized and retained capacities for spelling and offsets. 772 complete marked comparisons with the independent dense LF walk include 750 active shorter primary cycles and two wide-node endpoint cases. [Source transform](native.py) · [Complete implementation](native.inc.rs) · [Adversarial receipt](marked.txt)

```sh
d=research/astra-e/edge-displacements/bucket
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$d/model.rs" -o /tmp/bucket-census
/tmp/bucket-census /path/to/edges.bin > /tmp/bucket.csv
cmp /tmp/bucket.csv "$d/bucket.csv"
python3 "$d/summarize.py"
python3 "$d/marked.py" /tmp/edge-bucket-marked
```
