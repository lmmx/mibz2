# What the original retained-row queues append

On the 233 Wikidata admissions of the i9-verified edge worker, **10,851,747 of 14,386,594 queue append operations have width one**. Across blocks, their share ranges from 67.57% to 92.02%. A width-one `push` and the existing one-row integer range `extend` have identical queue contents, growth count and final capacity. Extending this specialization to widths 1–4 covers 92.64% of calls and 64.21% of the enqueued rows. Under the frozen Rust build this also preserves every final capacity and growth count on naturally constructed queues, independent of which blocks a worker receives. [All width totals and conditional capacities](summary.json) · [Real queue and output checks](checked.txt)

| Width | Append calls | Width | Append calls |
|---:|---:|---:|---:|
| 1 | 10,851,747 | 9 | 120,547 |
| 2 | 1,601,666 | 10 | 90,178 |
| 3 | 491,356 | 11 | 74,980 |
| 4 | 382,550 | 12 | 68,479 |
| 5 | 166,534 | 13 | 49,787 |
| 6 | 147,762 | 14 | 46,484 |
| 7 | 100,694 | 15 | 41,432 |
| 8 | 113,369 | 16 | 39,029 |

The census instruments the original kernel's `each_run` at `badb2aec3624bd65606c429b2e6f79e9d9d0050f` and repeats the exact frozen density/edge admission. Each recorded call appends a contiguous range of retained ranks to its actual byte queue. Seven of the width-one calls were forced by primary marks inside maximal runs wider than 16, on blocks 54, 63, 101, 216, 231, 303 and 415. Their truncated neighbors retain their original omitted classification. All seven encountered spare queue capacity in every tested history. The full trace records the original marked row and run width, retained rank, queue label, initialized length, and capacity before and after the forced append. A primary in a naturally retained run leaves its existing width intact. All 26,567,581 queue entries and 209,697,662 marked coded bytes are covered. [20,024 block/label histograms](widths.csv.xz) · [Seven split cases across policies and histories](marks.csv.xz)

## Exactly when the Rust capacity paths agree

The measurements use Rust 1.99.1 at `58cf8f9c3` with the existing allocator. These observations concern the actual standard `Vec<u32>` with an exact-size `Range<u32>`, a non-zero-sized element, and representable allocations. They depend on this implementation's growth policy.

Let c be capacity, l initialized length and w append width. When l + w ≤ c, both paths keep c. When the range needs space, its trusted-length reservation grows once to max(4, 2c, l+w). Repeated pushes fill any spare slots first. At each full-capacity crossing they grow to max(4, 2c, c+1). From a naturally constructed zero-capacity queue every nonzero capacity reached is at least four. If w ≤ 4 and c ≥ 4, then l+w ≤ 2c. Exactly one growth reaches 2c on either crossing. From empty, both reach four with one allocation. Clearing a vector preserves its capacity. Induction therefore preserves these capacities through mixed short-specialized and longer-original operations on any worker history. With w=1, even the initialized length at growth matches. For widths 2–4, pushes can place up to w−1 extra initialized entries in the old buffer before crossing. Actual allocator movement and cache traffic are outside the model.

For larger widths, a single `extend` can request above the next doubling. From empty, a range of width five grows once to five and pushes grow twice to eight. Width eight takes both to eight but still requires two growths for pushes. The width sequence [5,6,1] ends at 22 for ranges and 16 for pushes. An explicitly preallocated capacity below four falls outside the natural-history theorem. With capacity and length two, width four ends at six for a range and eight for pushes. For all widths an explicit `reserve(w)` before the individual pushes matches the original initial reservation and avoids another growth while emitting that range. The property executable checked actual contents and capacities over 70,785 combinations of capacity 0–64, all legal starting lengths and widths 0–32. [Native property and witnesses](property.txt) · [Checks](property.rs)

## Conditional queue high-water

The table counts the storage reported by the byte queues' vector capacities, in bytes. “Fresh” recreates all queues per block. A persistent slot clears lengths before its next admission. For the W7–10 comparisons, the full corpus block ordinal modulo W assigns each block to a slot and each slot retains its buffers. Work-stealing or another schedule can change these numbers. The real serial baseline's 256 individual capacities match the model after every block.

| History | Original ranges | Push widths ≤8 | Push all widths |
|---|---:|---:|---:|
| Fresh maximum | 1,386,192 | 1,387,376 | 1,377,232 |
| Serial persistent | 1,619,264 | 1,591,040 | 1,633,024 |
| 7 persistent slots | 10,030,920 | 9,996,968 | 10,054,016 |
| 8 persistent slots | 11,208,568 | 11,272,872 | 11,284,704 |
| 9 persistent slots | 12,446,912 | 12,480,820 | 12,553,936 |
| 10 persistent slots | 13,512,620 | 13,588,484 | 13,829,520 |

For each label, the original call sequence is represented by its cumulative initialized lengths. Binary search finds the first append crossing capacity. At that append the model executes the exact growth rule and attributes each first allocation or reallocation to its original width. The fresh and serial models for the original, ≤8 hybrid and all-push paths were checked against the actual Rust queue operations on every kept piece, including growth counts by width, initialized lengths at reallocation and every element after construction. Every queue value was compared with the unmodified frozen writer, totaling 159,405,486 compared entries. A separate clean run reproduced all block, label, growth and width tables byte for byte. The 7–10-slot histories apply the checked recurrence; they were not multithreaded timing runs. The cap-growth table records initialized elements at possible reallocation. Physical copies, allocator fragmentation, RSS and throughput require their own measurements. [Per-block conditional peaks](blocks.csv.xz) · [Conditional per-label capacity](capacities.csv.xz) · [Growth counts by width](growth.csv.xz)

From the repository root using Rust 1.99.1 and the frozen Wikidata corpus:

```sh
d=research/astra-e/queue-short-fill/census
python3 "$d/build.py" /tmp/astra-queue-census
/tmp/astra-queue-census/observe corpora/wikidata.bz2 /tmp/astra-queue-census/results > /tmp/astra-queue-census/checked.txt
cmp /tmp/astra-queue-census/checked.txt "$d/checked.txt"
for f in widths capacities blocks growth marks
do
    xz -dc "$d/$f.csv.xz" | cmp - "/tmp/astra-queue-census/results/$f.csv"
done
rustc --edition 2021 -C opt-level=3 "$d/property.rs" -o /tmp/astra-queue-census/property
/tmp/astra-queue-census/property | cmp - "$d/property.txt"
python3 "$d/report.py" | cmp - "$d/summary.json"
```

[Input, compiler, executable and raw table hashes](manifest.txt)
