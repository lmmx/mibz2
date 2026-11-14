← [Building a smaller bzip2 decoding table](<https://lmmx.github.io/mibz2/pr-preview/pr-385/blog/2026-11-04-compact30-constructor/index.md>)

# Technical appendix: the ideas behind building a smaller bzip2 decoding table

[The main post](<https://lmmx.github.io/mibz2/pr-preview/pr-385/blog/2026-11-04-compact30-constructor/index.md>) moves quickly through ideas that come from several fields at once: the Burrows–Wheeler transform, sorting, compact integer formats and machine code. This appendix introduces them one at a time, for a programmer who hasn't met them before. Simple things get simple explanations. The sections build on one another, but each can be read on its own. A [glossary](<#14-glossary>) is at the end.

All code is quoted from the frozen sources:

- the record format and the walker, [`examples/c30walk.rs` at `e003a32`](<https://github.com/lmmx/mibz2/blob/e003a32/examples/c30walk.rs>);
- the first builder, [`examples/c30walk/ctor.rs` at `e003a32`](<https://github.com/lmmx/mibz2/blob/e003a32/examples/c30walk/ctor.rs>);
- the second builder, [`examples/c30walk/ctor2.rs` at `8beffe2`](<https://github.com/lmmx/mibz2/blob/8beffe2/examples/c30walk/ctor2.rs>).

## 1\. A table of next steps

Imagine an array in which every entry says which entry to visit next:

```text
index:      0  1  2  3  4  5
next[i]:    3  4  5  2  0  1
```

Start at index 3 and follow it: \\(3 \\to 2 \\to 5 \\to 1 \\to 4 \\to 0 \\to 3\\). Every index appears exactly once as a destination, so the table is a *permutation*: it reorders the indices without losing or repeating any.

Undoing the Burrows–Wheeler transform comes down to following a permutation like this one, emitting a byte at each step. mibz2's conventional table stores one 4-byte entry per row. Compact30, named for the thirty rows it packs into each 64-byte record, stores the same permutation in less memory. The catch, and the subject of the main post, is that the compact table has to be built first, and building it must not cost more than it saves.

## 2\. The BWT in one example

The Burrows–Wheeler transform (BWT) is the reversible reordering at the heart of bzip2. [Section 1 of the space–time post](<https://lmmx.github.io/mibz2/pr-preview/pr-385/blog/2026-11-02-revisiting-space-time-tradeoff-bwt-inversion/#1-from-sorted-rotations-to-the-transformation-vector>) animates it. The short version: write down every rotation of the block, sort them, and keep the last column. For \\(\\mathtt{banana}\\):

| Row | Sorted rotation | First column \\(F\\) | Last column \\(L\\) |
| ---: | --- | :---: | :---: |
| 0 | abanan | a | n |
| 1 | anaban | a | n |
| 2 | ananab | a | b |
| 3 | banana | b | a |
| 4 | nabana | n | a |
| 5 | nanaba | n | a |

So \\(L = \\mathtt{nnbaaa}\\) and \\(F = \\mathtt{aaabnn}\\). The original block is in row 3, and the decoder is given that row number along with \\(L\\). Decoders never build the rotations; the table only explains the idea. The fact everything below uses is that **\\(F\\) holds exactly the bytes of \\(L\\), sorted**. That is what makes counting sort work.

## 3\. Counting sort, buckets and cursors

A *bucket* is a contiguous range of positions reserved for one kind of item. Because \\(F\\) is sorted, each byte's occurrences are adjacent: in \\(\\mathtt{aaabnn}\\), the a's occupy positions \\(\[0, 3)\\), the b position \\(\[3, 4)\\) and the n's \\(\[4, 6)\\). The notation \\(\[0, 3)\\) means 0, 1 and 2: the start is included and the end isn't, as in Rust's `0..3`.

*Counting sort* sorts values from a small known range without comparing them:

1. Count each value. A byte has 256 possible values, so the counts fit in an array of 256.
2. Turn the counts into bucket starts. Byte \\(c\\)'s bucket starts at the number of bytes smaller than \\(c\\); for \\(\\mathtt{banana}\\), a at 0, b at 3, n at 4. This running total is a *prefix sum*, and the code calls the array of starts \\(C\\).
3. Place each item at the next free position of its bucket.

The *cursor* in step 3 is nothing special: it is a variable holding the next free position in one byte's bucket. Place an item there, then add one:

```rust
let mut next = bucket_starts;          // C: one cursor per byte value
for (i, &c) in l.iter().enumerate() {
    let r = next[c as usize];          // this row's destination
    next[c as usize] += 1;
    table[r] = i as u32;               // store the source row at the destination
}
```

For \\(\\mathtt{banana}\\) the scan assigns:

| Source row \\(i\\) | \\(L\[i\]\\) | Destination \\(r\\) |
| ---: | :---: | ---: |
| 0 | n | 4 |
| 1 | n | 5 |
| 2 | b | 3 |
| 3 | a | 0 |
| 4 | a | 1 |
| 5 | a | 2 |

This placement is *stable*: equal bytes keep their original order. The three a's come from rows 3, 4 and 5 and go to positions 0, 1 and 2, in that order. A left-to-right scan with a cursor per byte gives stability for free. The builders rely on its consequence: **within one byte's bucket, consecutive destinations hold increasing source rows.**

## 4\. The navigation table

Store the source row at each destination, as the loop above does, and you have the table the walk follows:

| Destination row | \\(F\\) | Next row |
| ---: | :---: | ---: |
| 0 | a | 3 |
| 1 | a | 4 |
| 2 | a | 5 |
| 3 | b | 2 |
| 4 | n | 0 |
| 5 | n | 1 |

That is the permutation of section 1. Start at row 3, emit \\(F\\) at each row, and move to the next row. The output is b, a, n, a, n, a: \\(\\mathtt{banana}\\).

The space–time post calls this table \\(T^{-1}\\), because Seward's \\(T\\) runs the other way. bzip2's decoder, and mibz2's, store \\(T^{-1}\\) so that the walk emits the block forwards. The research journals call the same table ψ. Compact30 changes how this table is stored, not the permutation itself.

Two coordinates are easy to confuse here:

- a *destination* \\(r\\) is which entry of the table is being filled;
- a *source row* \\(i\\) is which position of \\(L\\) supplied it, and so which row the walk visits next.

The builder decides *where* to write from \\(r\\), and *what* to write from \\(i\\).

## 5\. Labels: numbering rows thirty to a block of 32

Compact30 doesn't store a next row \\(i\\) as a plain number. It stores the row's *label*:

\\\[ \\varphi(i) = 32\\left\\lfloor \\frac{i}{30} \\right\\rfloor + (i \\bmod 30) \\\]

(in Rust, `32 * (i / 30) + i % 30`). It isn't a hash or an identifier; it's just a renumbering in which each group of 30 rows uses 30 of a block of 32 numbers:

| Row \\(i\\) | \\(\\lfloor i / 30 \\rfloor\\) | \\(i \\bmod 30\\) | Label \\(\\varphi(i)\\) |
| ---: | ---: | ---: | ---: |
| 0 | 0 | 0 | 0 |
| 29 | 0 | 29 | 29 |
| 30 | 1 | 0 | 32 |
| 42 | 1 | 12 | 44 |
| 59 | 1 | 29 | 61 |
| 60 | 2 | 0 | 64 |

Labels 30 and 31 are never used, nor 62 and 63, and so on. The gaps are deliberate. A record holds 30 two-byte fields and a 4-byte header, the size of 32 two-byte slots, and the two unused labels in each block line up with the header. Because 32 is a power of two, the walk can split a label \\(y\\) into its record and its field with two cheap operations:

- `y >> 5`, a shift right by five bits, divides by 32: the record;
- `y & 31` keeps the low five bits: the field.

With plain row numbers, it would have to divide by 30, which is much slower. The label function exists to make the walk cheap; the builder has to work with it.

## 6\. Records, offsets and RAW groups

The [record](<https://github.com/lmmx/mibz2/blob/e003a32/examples/c30walk.rs#L73-L79>) is:

```rust
#[repr(C, align(64))]
struct Rec {
    f: [u16; 30],   // bytes 0–59: one 16-bit field per row
    h: u32,         // bytes 60–63: the header
}
```

`u16` and `u32` are unsigned 16- and 32-bit integers. `align(64)` places every record at a multiple of 64 bytes, so each record fills exactly one 64-byte cache line, the unit in which the processor moves memory into its caches.

Each record holds 30 consecutive *destinations*: destination \\(r\\) is in record \\(\\lfloor r/30 \\rfloor\\), field \\(r \\bmod 30\\).

**Ordinary records.** All 30 rows output the same byte, and their next rows' labels are close together. The header holds the byte and a *base*, the first row's label:

\\\[ h = (\\text{base} \\ll 8)\\ \\text{|}\\ c, \\\]

so `h & 255` is the byte and `h >> 8` is the base. Each field holds its row's label minus the base, so the walk computes the next label as base plus field. Because of section 3's ordering, the first row has the smallest label and the last row the largest.

**RAW groups.** Two situations break the ordinary form:

- **crossing:** the record's 30 destinations straddle two buckets, say 18 a's and 12 b's, so no single header byte fits;
- **wide:** the labels are too far apart for 16-bit offsets.

Such a group is stored as a [`RawG`](<https://github.com/lmmx/mibz2/blob/e003a32/examples/c30walk.rs#L81-L84>), 32 four-byte entries (128 bytes), of which 30 are used. Each entry is `(label << 8) | byte`, so every row can have its own byte and its own full label. The record keeps all fields at `0xFFFF`, and its header becomes `RAW_BIT | index`: bit 28 set, and the RAW group's index in the low 20 bits. RAW doesn't mean uncompressed input; it means this group's navigation is spelled out in full.

A third kind, *forced* RAW, exists only for testing. The research harness can force a fraction of groups to RAW to exercise those code paths. The measured configuration forces none.

**Why the limit is 65,533.** A `u16` holds 0 to 65,535, but compact30 reserves the top two values. 65,534 (`0xFFFE`) marks the row where one of the walk's segments starts, and 65,535 (`0xFFFF`) fills RAW records, padding and the parking record. An ordinary offset can therefore be at most 65,533, and the code's `MAX_OFF` is that. A group whose labels span more is wide.

**Parking.** After the last group comes one more record, all `0xFFFF`. When one of the walk's lanes reaches a RAW row, it starts fetching the RAW entry and moves to this record's label for a turn. It resumes from there once the entry has had time to arrive.

A note on the toy example: \\(\\mathtt{banana}\\)'s six destinations all fall in record 0, but they belong to three buckets. So its one record would be crossing RAW. The ordinary record above is the common case in real blocks, not something this toy example can show.

## 7\. Epochs and snapshots

An *epoch* here is simply a fixed stretch of source rows: 61,440 of them. It has nothing to do with time.

The number isn't arbitrary; it falls out of the label function. The second builder stores only a label's low 16 bits, so it wants to know where the high 16 bits go up by one, every \\(2^{16} = 65536\\) labels. Each 30 source rows use 32 labels, so 65,536 labels are \\(65536 / 32 = 2048\\) groups, and \\(2048 \\times 30 = 61440\\) source rows. Therefore

\\\[ \\varphi(i + 61440) = \\varphi(i) + 65536. \\\]

| Source row | Label |
| ---: | ---: |
| 0 | 0 |
| 61,439 | 65,533 |
| 61,440 | 65,536 |
| 61,441 | 65,537 |
| 122,880 | 131,072 |

Within epoch \\(e\\), every label equals \\(65536 \\cdot e\\) plus its low 16 bits. The useful low values stop at 65,533, because of the two unused labels per block of 32.

The builder can't afford to store an epoch for every row. It doesn't need to, because of the ordering in section 3: the rows arriving in one bucket come in source order. So at the start of each epoch it saves a *snapshot*, a copy of the 256 cursors: each byte's next free destination. Write \\(A\_e\[c\]\\) for byte \\(c\\)'s cursor at the start of epoch \\(e\\). Then

\\\[ A\_e\[c\] = C\[c\] + \\text{the number of } c\\text{'s before source row } 61440\\,e. \\\]

These are exact counts, not estimates. Suppose byte a's snapshots are:

| Epoch \\(e\\) | \\(A\_e\[\\mathtt{a}\]\\) |
| ---: | ---: |
| 0 | 100 |
| 1 | 104 |
| 2 | 104 |
| 3 | 106 |

Four a's arrived in epoch 0 (destinations 100–103), none in epoch 1, and two in epoch 2 (104–105). Which epoch filled destination 104? Epoch 2: the *latest* epoch whose snapshot is at most 104,

\\\[ h(r) = \\max\\{\\, e : A\_e\[c\] \\le r \\,\\}. \\\]

Choosing the first epoch with the value 104 would give epoch 1, wrongly. Repeated snapshot values mean the byte was absent for a while, and the latest one is right. The code's [`epoch_of`](<https://github.com/lmmx/mibz2/blob/8beffe2/examples/c30walk/ctor2.rs#L333-L341>) searches backwards from the newest snapshot for exactly that.

If destination 104 holds the 16-bit value 12, its full label is \\(2 \\times 65536 + 12 = 131084\\). The builder recovered it from a value already sitting in the final record, a destination it already knows, and a small table of counts.

A 900,000-row block spans 15 epochs, so its snapshots take \\(15 \\times 256 \\times 4\\) bytes, 15 KiB, against 3.6 MB for a 32-bit temporary per row. The general trick is worth remembering: **periodic summaries of state you already keep can replace per-item metadata.**

## 8\. Descriptors and portions

A *descriptor* is a small struct holding what an operation needs to carry on. The second builder keeps one per byte value, [`Desc`](<https://github.com/lmmx/mibz2/blob/8beffe2/examples/c30walk/ctor2.rs#L229-L249>):

```rust
struct Desc {
    write: *mut u16,   // the next field to write
    end: *mut u16,     // one past the last field of the current portion
    w0: *mut u16,      // the portion's first field
    r0: u32,           // the destination of that first field
    lim: u32,          // the end of the byte's bucket
    g: u32,            // the record the portion is in
    raw: u32,          // its RAW index, if the record is RAW from the start
}
```

The `*mut u16` fields are raw pointers, memory addresses. With them ready, the per-row path writes through `write` and bumps it, without recomputing a record and field from a destination. Each descriptor is 40 bytes: three 8-byte pointers and four 4-byte integers.

A *portion* is one byte's share of one record. Usually it is the whole record. But if record 10 covers destinations 300–329 and a's bucket ends at 320, then a's portion is 300–319 and the next byte's is 320–329. That record is crossing RAW, and each portion is finished into its RAW block separately.

A portion isn't always finished the moment its last field is written. It's finished when the byte next turns up and finds its portion full (`refill`), when a run continuing past it closes it, or at the end of the block, whichever comes first. The snapshot rule still works then, because a snapshot counts the rows already written to open portions.

## 9\. Finishing a group

When an ordinary candidate's portion is full, `finish` does four things:

1. It recovers the full labels of the group's first and last rows (section 7). In the common case the whole group arrived in the latest epoch, so no search is needed.
2. It classifies the group. The labels increase through the group, so the first is the smallest and the last the largest, and their difference is the group's whole span. No scan of all 30 is needed.
3. If the span is at most 65,533, it writes the header and converts the fields to offsets.
4. If not, it builds a RAW block from the 30 full labels.

**Why classify from full labels.** Take first label 2 and last label 65,536. Their true span is 65,534, too wide. Their low 16 bits are 2 and 0, which would make the group look tiny, or backwards. Only the full labels give the right answer. The test suite pins the exact boundary: source rows 3 → 61,440 span 65,533 (ordinary), and rows 2 → 61,440 span 65,534 (RAW).

**Why 16-bit subtraction is then enough.** *Normalisation* here just means subtracting the group's first label from each label, turning absolute labels into offsets: \\(\[1000, 1004, 1009\]\\) becomes \\(\[0, 4, 9\]\\). Suppose a group crosses an epoch boundary, with labels 65,530 and 65,540. In the fields they're stored as 65,530 and 4. Ordinary arithmetic gives \\(4 - 65530 = -65526\\), which is nonsense. But 16-bit unsigned subtraction wraps around modulo 65,536:

\\\[ (4 - 65530) \\bmod 65536 = 10, \\\]

the true offset. So once step 2 has shown that the span fits, `wrapping_sub` on the 16-bit fields gives every offset correctly. The builder never needs the 30 full labels of an ordinary group, only two.

**Where SIMD comes in.** Normalisation does the same subtraction 30 times. Modern processors have SIMD (single instruction, multiple data) instructions, which do one operation on several values at once. x86's `psubw` subtracts eight pairs of 16-bit numbers held in a 128-bit register. The compiler turned the builder's plain Rust loop into five of them:

```text
movdqu (%rdi),%xmm2       ; load 16 bytes: eight fields
psubw  %xmm1,%xmm2        ; subtract the first label from all eight
movdqu %xmm2,(%rdi)       ; store them back
```

Three such groups of eight cover 24 fields, and two narrower loads, of 8 bytes and 4 bytes, cover the last 6. The arithmetic is cheap. The loads and stores aren't free: the fields are read and rewritten after already being written once as labels.

## 10\. RAW groups and their canonical order

*Canonical* means "the one agreed form" of something that could be written several equivalent ways, as a set is often listed sorted so that {3, 1, 2} and {2, 3, 1} both appear as \[1, 2, 3\].

Compact30's agreed form puts RAW blocks in record order. Record 2's RAW block comes before record 5's, which comes before record 9's, and each RAW record's header holds its block's position. The builder doesn't find RAW groups in that order, though:

- crossing groups are known from the byte counts before the scan, and get blocks in advance;
- wide groups are found during the scan, when they finish, and get the next free block.

So the blocks might come out as \[record 9, record 2, record 5\]. The walk would cope with any consistent numbering, but the builder must match the reference byte for byte. A different order would be a different output.

The [canonicaliser](<https://github.com/lmmx/mibz2/blob/e003a32/examples/c30walk/ctor.rs#L651-L702>), the same in both builders, does three things:

1. **Ranks the RAW records.** A *bitmap*, one bit per record set if the record is RAW, is scanned in record order. Each RAW record's rank becomes its block's final position, and is written into the header.
2. **Tags each block with its destination.** Each RAW block has two spare entries, and the last one, word 31, temporarily holds the block's final position.
3. **Moves the blocks into place.** It follows *permutation cycles*: pick up a block, put it where its tag says, pick up the block it displaces, and carry on until the cycle closes. This moves the blocks in place with one spare block of memory, then clears every word 31.

## 11\. Runs, fragments and the bulk emitter

A *run* is a stretch of equal bytes, like the b's in `aaaabbbbbbbbbcccc`. BWT output has many, which is why bzip2 compresses it well.

A run's rows go to consecutive destinations of one bucket, because each takes the cursor's next value. Their labels are consecutive source labels, which follow a fixed pattern determined only by where the run starts within a block of 30 rows. So when a run covers a whole record, aligned to its start, the final offsets and the header are known in advance. The builder writes the record directly from one of thirty [precomputed patterns](<https://github.com/lmmx/mibz2/blob/8beffe2/examples/c30walk/ctor2.rs#L74-L90>), with no labels and no finish. That is the *bulk emitter*. It handles 74.3% of rows on the main corpus.

A run rarely starts exactly at a record boundary. Its first few rows may complete a record that was already part-filled, then come whole records, then a remainder:

```text
prefix fragment → complete records → suffix fragment
```

The fragments go through provisional labels, written in bulk by `fill`, and are finished later like any other portion.

The builder finds runs by testing 16 bytes of \\(L\\) at a time for uniformity. A 16-byte *tile* that isn't uniform is handled one row at a time. The tile's labels are generated from its starting label, with one adjustment for the gap after each 30th row, so the source position is advanced once per tile, not once per row.

## 12\. Instructions, dependencies and time

The research reports discuss the builder at three levels, which are related but not the same:

- **Mathematics:** which output must be produced. Every destination needs the right next label and byte.
- **Algorithm:** how it's produced, for example by stable counting sort, provisional labels and a finish per group.
- **Machine code:** the instructions the compiled program actually runs: loads, stores, arithmetic, comparisons, branches, SIMD.

A change at one level shows up at the levels below. Removing a class of work from the algorithm removes many instructions. A change to how the Rust is written can remove a redundant load without changing the algorithm. A bottleneck in the processor can survive both.

**Rust becomes assembly through the compiler.** Two lines like

```rust
d.write.write(u);
d.write = d.write.add(1);
```

don't map one-to-one onto two instructions. The compiler decides what lives in registers, which loads it can drop, which branches to keep and which loops to vectorise. That is how the builder gets `psubw` without any hand-written SIMD, and why restructuring the Rust, or the algorithm, is how the instruction count goes down.

**A dependency chain** is a sequence of operations in which each needs the previous result. The first builder's per-row path was roughly: load the cursor, compute the record, load its header, extract the base, subtract, compare with the limit, store. No step can start before the one it depends on finishes. Independent operations can overlap, because a modern core runs several at once. So removing one link of a long chain can matter more than removing several independent instructions.

**Fewer instructions needn't mean less time.** An instruction count says how long a code path is, not how long it takes:

- instructions differ in cost, and some split into several internal operations (*micro-ops*);
- independent instructions can run at the same time;
- a load can wait a long time for memory;
- a mispredicted branch throws work away;
- a short path that runs millions of times can still dominate the total.

So the reports weight each path by how often it runs on the real corpus. Even then they keep the result as an estimate until it's measured.

**Lockstep steps.** In the construction measurement, four workers each build a block, and the next step starts only when all four have finished. A step's time is its slowest worker's time. So a builder whose cost varies a lot from block to block loses time even if its average cost is unchanged: the slow blocks set the pace.

**The instruction floor.** A floor is the cost that remains once unnecessary work is gone. The current question is how much of the second builder's machine-level work the frozen output truly requires, and how much is removable. Some work is required: the right labels must reach the right fields. Some is removable, such as recomputing state that is already known, or reloading a value already in a register. Some is a trade, like provisional labels, which shorten the per-row path but add a second pass over the fields. And some costs, like cache misses, can't be read off the assembly at all.

## 13\. Four groupings, three numbers

The builder juggles four different ways of dividing up the same rows:

| Grouping | Rows in it share | It decides |
| --- | --- | --- |
| Bucket | one byte value in \\(F\\) | where each row goes |
| Record | 30 consecutive destinations | how the output is laid out |
| Epoch | 61,440 consecutive source rows | how full labels are recovered |
| Run | consecutive equal bytes in \\(L\\) | when the bulk emitter can be used |

They don't line up. A run can feed several records, a record can straddle two buckets, and one record's rows can come from several epochs. Most of the builder's complexity comes from reconciling these four without a full-sized temporary table.

And three numbers that are easy to confuse:

| Symbol | What it is | Example |
| --- | --- | ---: |
| \\(i\\) | a source row: a position in \\(L\\) | 42 |
| \\(r\\) | a destination: a position in \\(F\\), and a slot in the records | 67 |
| \\(y = \\varphi(i)\\) | the label of the row the walk visits next | 44 |

If source row 42 is placed at destination 67, the builder writes information derived from label 44 into the field for destination 67, field 7 of record 2. Where the value goes comes from \\(r\\), and what it is comes from \\(i\\). Keeping those two apart is most of what it takes to read `ctor2.rs`.

## 14\. Glossary

| Term | Meaning |
| --- | --- |
| BWT | Burrows–Wheeler transform: the reversible reordering bzip2 is built on |
| \\(L\\), \\(F\\) | the last and first columns of the sorted rotations; \\(F\\) is \\(L\\) sorted |
| Bucket | the positions in \\(F\\) that hold one byte value |
| Prefix sum | a running total, used to turn counts into bucket starts |
| Cursor | the next free position in a bucket |
| Stable | equal items keep their original order |
| \\(T^{-1}\\), ψ | the navigation table: for each row, the next row of the walk |
| Label, \\(\\varphi\\) | a row number recoded as 30 of every 32 values |
| Record | 64 bytes: 30 16-bit fields and a 4-byte header |
| Base, offset | a group's first label, and each row's label minus it |
| RAW group | 30 full entries, each a label and a byte, for a group that can't be ordinary |
| Crossing, wide, forced | RAW because of two buckets, because of too large a span, or because a test says so |
| Portion | one byte's share of one record |
| Descriptor | one byte's write state: pointers into its current portion |
| Epoch | 61,440 consecutive source rows, over which a label's high 16 bits are fixed |
| Snapshot | the 256 cursors saved at an epoch's start |
| Normalisation | subtracting a group's first label from each label |
| Bulk emitter | writes whole records for aligned runs directly from patterns |
| Fragment | the part of a run that doesn't fill a whole aligned record |
| Canonicalisation | putting RAW blocks into record order and renumbering their references |
| SIMD | instructions that work on several values at once |
| Oracle | the reference builder whose output must be matched byte for byte |
| Dependency chain | operations that each need the previous one's result |

## 15\. Where the design comes from

No single obscure primitive explains the builder. Each ingredient is a standard technique, and the combination is specific to compact30:

- **The navigation permutation and counting sort** are as old as the BWT. Burrows and Wheeler's 1994 report already builds the table from counts, as the space–time post describes.
- **Base-plus-offset encoding** is common in compressed integer formats and databases.
- **The record layout** follows from fitting a group's offsets and a header into one cache line: 60 bytes leave 30 two-byte fields. Addressing them in blocks of 32 gives the label function, and \\(2048 \\times 32 = 65536\\) gives the epoch, 61,440 rows. Each constant follows from the one before.
- **Bulk emission** exploits the runs that make BWT output compressible.
- **Deferring the offsets** came from asking whether each row's final offset had to be known when the row arrived. Answering no, and recovering the lost high bits from snapshots, is the [second builder's design](<https://github.com/lmmx/mibz2/blob/25891bd/docs/journal/2026-11-03-compact30-constructor-v2-design.md>).

Seward's 2001 paper, discussed in the space–time post, is the background to the space–time trade-off itself. It doesn't propose this builder.

## 16\. Where to read the source

In roughly this order:

| To understand | Read |
| --- | --- |
| The record, the RAW block and the labels | [`Rec`, `RawG`, `phi` and `phi_inv`](<https://github.com/lmmx/mibz2/blob/e003a32/examples/c30walk.rs#L73-L100>) |
| Descriptors | [`Desc`](<https://github.com/lmmx/mibz2/blob/8beffe2/examples/c30walk/ctor2.rs#L229-L249>) |
| Snapshots, the epoch search, opening and refilling a portion | [`snapshot` to `refill`](<https://github.com/lmmx/mibz2/blob/8beffe2/examples/c30walk/ctor2.rs#L321-L395>) |
| The per-row append and the tiles | [`one` and `tile`](<https://github.com/lmmx/mibz2/blob/8beffe2/examples/c30walk/ctor2.rs#L538-L578>) |
| Finishing a group, and RAW entries | [`finish` and `write_entries`](<https://github.com/lmmx/mibz2/blob/8beffe2/examples/c30walk/ctor2.rs#L398-L536>) |
| Runs and the bulk emitter | [`fill`, `complete` and `run`](<https://github.com/lmmx/mibz2/blob/8beffe2/examples/c30walk/ctor2.rs#L580-L681>) |
| RAW canonicalisation | [`canonicalise`](<https://github.com/lmmx/mibz2/blob/e003a32/examples/c30walk/ctor.rs#L651-L702>) |
| The first builder's per-row path, for contrast | [v1's `one`](<https://github.com/lmmx/mibz2/blob/e003a32/examples/c30walk/ctor.rs#L597-L648>) |
| The compiled loops, annotated | [Excerpts of the machine code](<https://github.com/lmmx/mibz2/blob/cd14fa9/docs/journal/2026-11-03-compact30-constructor-v2-prototype/excerpts.asm>) |
| The design, its correctness evidence and its measurement | [Design](<https://github.com/lmmx/mibz2/blob/25891bd/docs/journal/2026-11-03-compact30-constructor-v2-design.md>) · [Prototype](<https://github.com/lmmx/mibz2/blob/cd14fa9/docs/journal/2026-11-03-compact30-constructor-v2-prototype.md>) · [Measurement](<https://github.com/lmmx/mibz2/blob/25891bd/docs/journal/2026-11-03-compact30-construction-v2-capture.md>) |
