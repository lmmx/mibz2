[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-379/blog/index.md>)

# Building a smaller bzip2 decoding table: from 49 instructions to 12

Writing a cheap placeholder for each row, and finishing each 30-row group once, shrank the inner loop. It still builds too slowly.

4 November 2026 · 12 min read · [Research in progress](<#8-fewer-instructions-not-yet-a-cheaper-build>)

[The last post](<https://lmmx.github.io/mibz2/pr-preview/pr-379/blog/2026-11-02-revisiting-space-time-tradeoff-bwt-inversion/index.md>) ended with a smaller navigation table, compact30 (named for its thirty rows to each 64-byte record), beating mibz2's conventional one when four workers walk at once, and a warning: [the table still has to be built](<https://lmmx.github.io/mibz2/pr-preview/pr-379/blog/2026-11-02-revisiting-space-time-tradeoff-bwt-inversion/#9-the-table-still-has-to-be-built>). The records in that comparison came from a reference builder that computed every row's successor in a full-sized temporary array first. A decoder can't afford that. It needs a builder that goes straight from the block's last column \\(L\\) and its byte counts to the final records, and costs about as much as the builder mibz2 uses today.

This post follows that builder through two versions. The second one's inner loop is a quarter the length of the first's, and the change behind it wasn't hand-written assembly. It was a different order of work, which let the compiler emit a much shorter loop. The second builder still costs too much with four workers, so the post ends with what we're doing about that.

The ideas a reader may not have met before, such as stable counting sort, the row labels, epochs and RAW canonicalisation, are introduced slowly in the [technical appendix](<https://lmmx.github.io/mibz2/pr-preview/pr-379/blog/2026-11-04-compact30-constructor-notes/index.md>). It works through a small example from scratch, and its sections can be read on their own.

## 1\. What the builder has to produce

The navigation table says, for each row of the block, which row the walk visits next. mibz2's conventional table stores that as a 4-byte entry per row. Compact30 groups the rows thirty at a time. Each group gets a 64-byte record, the size of one cache line: thirty 16-bit fields (60 bytes) and a 4-byte header.

In an *ordinary* record, all thirty rows output the same byte, and their next rows lie close together. The header holds that byte and a *base*, the first row's successor. Each field holds its own row's successor minus the base:

\\\[ \\text{successor}(j) = \\text{base} + \\text{offset}\[j\]. \\\]

If a group's first three successors are 40, 44 and 49, the base is 40 and the first three fields are 0, 4 and 9. That's base-plus-offset encoding, used in many compressed integer formats. Compact30 keeps every offset directly addressable, so the walk reads one field instead of decoding packed bits.

Two kinds of group can't be ordinary, and are stored instead as *RAW* groups, thirty full 4-byte entries in a separate 128-byte block:

- a *crossing* group holds the end of one byte's rows and the start of the next byte's, so no single header byte fits it;
- a *wide* group's successors are too far apart for 16-bit offsets: more than 65,533 apart, because the two largest field values are reserved as markers.

The builder must produce the reference builder's output exactly, byte for byte. That includes every header, the padding in the last record, a final "parking" record, and the order of the RAW groups.

The successors themselves come from counting sort: each row of \\(L\\) goes to the next free position in its byte's *bucket*, the run of positions in the sorted first column \\(F\\) where that byte lives. The free position is tracked by a *cursor* per byte. Because the scan goes left to right, the rows that land in one bucket arrive in increasing order. [Section 3 of the appendix](<https://lmmx.github.io/mibz2/pr-preview/pr-379/blog/2026-11-04-compact30-constructor-notes/#3-counting-sort-buckets-and-cursors>) works this through for \\(\\mathtt{banana}\\). For the builder, it means that within one ordinary group the first row has the smallest successor and the last row the largest. So a group fits an ordinary record exactly when its first and last successors are at most 65,533 apart.

## 2\. The first builder: final offsets, one row at a time

The obvious way to build the records is to scatter rows as counting sort does, and to write each row's final offset as it lands. That is what the [first builder](<https://github.com/lmmx/mibz2/blob/e003a32/examples/c30walk/ctor.rs#L597-L648>) did. Abbreviated:

```rust
let x = self.x[ci];                         // byte c's next slot, in record coordinates
self.x[ci] = x + 1 + 2 * ((x & 31) == 29) as u32;
let (g, j) = ((x >> 5) as usize, (x & 31) as usize);
let idx = if j == 0 {
    // first slot of a group: start the record's header, unless it's RAW
    ...
} else {
    let h = self.header(g);                 // read the record's header back
    if h & RAW_BIT != 0 {
        Some(h & M20)                       // already RAW
    } else {
        let d = y - (h >> 8);               // offset from the record's base
        if d > MAX_OFF {
            Some(self.promote(g, j, h, true)) // too wide: turn the group into RAW now
        } else {
            self.fields(g).add(j).write(d as u16);
            None
        }
    }
};
```

Every row that isn't part of a long run goes through this. It reads the record's header back from memory, tests whether the group is RAW, subtracts the base, compares the result with the 16-bit limit, and only then writes. Most of those steps depend on the one before. The compiled loop for one row was 49 instructions long.

The first builder was correct: it matched the reference byte for byte, and passed an independent source audit. But a measurement on the i9 workstation put it at about 1.32 times production's CPU cost with four workers, against a budget of at most 1.067. That measurement was formally invalid: on one core the production builder it was compared with didn't stay stable from round to round, and one synchronised step started unevenly. But its figures left no doubt that the first builder was far over budget.

## 3\. The second builder: write a provisional label, finish later

The second builder, designed by Astra, asks a different question: does the builder need to know a row's final offset when the row arrives? It doesn't. The offset depends on the group's base and on whether the group stays ordinary, and neither is known until the group is complete. So the builder writes something cheaper now, and does the rest once per group.

What it writes is the row's *label*. Compact30 numbers the rows so that each group of thirty uses thirty of a block of 32 consecutive numbers:

\\\[ \\varphi(i) = 32\\left\\lfloor \\frac{i}{30} \\right\\rfloor + (i \\bmod 30). \\\]

Row 42 is the 13th row of group 1, so \\(\\varphi(42) = 32 + 12 = 44\\). The two unused numbers in each block of 32 line up with the record's 4-byte header. With 32 a power of two, the walk can split a label into record and field with a shift and a mask. Successors are stored as labels; that is why a 30-row record can be addressed so cheaply.

The second builder takes the low 16 bits of the label, \\(\\varphi(i) \\bmod 65536\\), and writes them straight into the field where the row's final offset will eventually go. It uses no temporary array: the fields of the final records are the workspace.

To know where that field is, it keeps a small *descriptor* for each of the 256 byte values: a pointer to the next field to write, a pointer to the end of the current stretch of fields, and some bookkeeping. The per-row work, [`one`](<https://github.com/lmmx/mibz2/blob/8beffe2/examples/c30walk/ctor2.rs#L538-L552>), is then:

```rust
unsafe fn one(&mut self, c: u8, u: u16) -> bool {
    let ci = c as usize;
    if self.desc[ci].write == self.desc[ci].end && !self.refill(ci) {
        return false;                   // the byte's bucket is full: malformed counts
    }
    let d = &mut self.desc[ci];
    d.write.write(u);                   // the provisional label, into its final field
    d.write = d.write.add(1);           // and on to the next field
    true
}
```

It has no header read, no RAW test and no span check. When the descriptor's current stretch is full, `refill` finishes it and opens the next. `refill` is marked `#[cold]`, which tells the compiler to keep it out of the way of the common path.

Writing only 16 bits loses information. A bzip2 block can hold 900,000 rows, and their labels run far past 65,535. The next section is about how the builder gets the missing high bits back without storing them.

## 4\. Getting the high bits back: epochs and snapshots

The label function has a convenient period. After 2,048 groups of 30 rows, which is 61,440 rows, the label has advanced by exactly \\(2048 \\times 32 = 65536\\):

\\\[ \\varphi(i + 61440) = \\varphi(i) + 65536. \\\]

So the builder splits the scan of \\(L\\) into *epochs* of 61,440 source rows. Within one epoch, every label's high 16 bits are the same number, the epoch's index \\(e\\), and

\\\[ \\text{full label} = 65536 \\cdot e + \\text{the 16 bits in the field}. \\\]

The builder still has to know which epoch each row came from, and it can't afford to store an epoch per row. It doesn't need to. At the start of each epoch it saves a *snapshot*: the 256 byte cursors, each one the next free position in that byte's bucket. Rows land in a bucket in scan order, so the snapshots divide each bucket into epochs. A row of byte \\(c\\) at position \\(r\\) came from the latest epoch whose snapshot for \\(c\\) is at most \\(r\\):

\\\[ h(r) = \\max\\{\\, e : A\_e\[c\] \\le r \\,\\}, \\\]

where \\(A\_e\[c\]\\) is byte \\(c\\)'s cursor at the start of epoch \\(e\\). "Latest" matters. If \\(c\\) doesn't occur at all in some epoch, its cursor doesn't move, and two snapshots hold the same number. The row at that position arrived in the later of the two epochs.

A full 900,000-row block spans 15 epochs, so its snapshots take \\(15 \\times 256 \\times 4\\) bytes, 15 KiB. A temporary 32-bit successor for every row would take 3.6 MB. Both the [snapshot](<https://github.com/lmmx/mibz2/blob/8beffe2/examples/c30walk/ctor2.rs#L321-L331>) and the [epoch search](<https://github.com/lmmx/mibz2/blob/8beffe2/examples/c30walk/ctor2.rs#L333-L341>) are a few lines each. [Section 7 of the appendix](<https://lmmx.github.io/mibz2/pr-preview/pr-379/blog/2026-11-04-compact30-constructor-notes/#7-epochs-and-snapshots>) has a worked example.

## 5\. Finishing a group once

When a group's thirty fields are full, [`finish`](<https://github.com/lmmx/mibz2/blob/8beffe2/examples/c30walk/ctor2.rs#L398-L499>) turns it into its final form. It recovers the full labels of the first and last rows from the snapshots. In the common case it doesn't search at all, because the whole group arrived in the latest epoch. It then classifies the group from those two full labels:

- If they differ by at most 65,533, the group is ordinary. The header becomes `y0 << 8 | c`, the first full label and the byte, and each field becomes its offset from the first.
- If not, the group is wide. Its thirty full entries are rebuilt into a new RAW block, and the record is replaced by a reference to that block.

The classification has to use the full labels. The 16-bit values can mislead: a first label of 2 and a last of 65,536 are 65,534 apart, one too many for an ordinary group, but their low 16 bits are 2 and 0. The tests check that exact boundary. Rows 3 to 61,440 span 65,533 and stay ordinary; rows 2 to 61,440 span 65,534 and become RAW.

Once the group is known to be ordinary, though, the 16-bit values are enough for the offsets. Subtracting in 16-bit arithmetic that wraps around gives the right answer even across an epoch boundary. A first label whose low bits are 65,530 and a later one whose low bits are 4 differ by \\((4 - 65530) \\bmod 65536 = 10\\), which is the true difference. In Rust, the normalisation is a short loop over the thirty fields:

```rust
let mut f = fp.read();               // all 30 fields, 60 bytes
for x in f.iter_mut() {
    *x = x.wrapping_sub(u0);         // u0: the first field's 16 bits
}
fp.write(f);
```

The compiler turned that into SIMD code. `psubw` subtracts eight 16-bit numbers at once, so three of them cover 24 fields, and two narrower ones cover the last 6. There is no hand-written vector code in the source. The arithmetic is cheap, but the 60 bytes of fields are still read and written a second time. That matters in section 8.

Crossing groups are known from the byte counts before the scan starts, so their RAW blocks are made in advance, and each byte's share of a crossing group is finished into its block separately. RAW groups found to be wide during the scan are numbered in the order they're found, which isn't record order. So at the end, a [canonicalisation](<https://github.com/lmmx/mibz2/blob/e003a32/examples/c30walk/ctor.rs#L651-L702>) pass, unchanged from the first builder, permutes the RAW blocks into record order in place and rewrites each record's reference. [Section 10 of the appendix](<https://lmmx.github.io/mibz2/pr-preview/pr-379/blog/2026-11-04-compact30-constructor-notes/#10-raw-groups-and-their-canonical-order>) explains it.

## 6\. Most rows never take the per-row path

Long runs of one byte are common in a BWT's last column. A run's rows land in consecutive positions of one bucket, and consecutive source rows have labels that follow a fixed pattern. So when a run covers a whole group, aligned to the group's start, the builder can write the final record directly from one of thirty precomputed patterns, header included. It writes no provisional labels and needs no later finish. This *bulk emitter*, [`complete`](<https://github.com/lmmx/mibz2/blob/8beffe2/examples/c30walk/ctor2.rs#L598-L641>), was carried over unchanged from the first builder. Before a run fills whole groups, and after, the builder writes its leftover rows with `fill`, which still produces provisional labels.

On the main Wikidata corpus, 207,006,515 rows in 434 blocks:

| Path | Rows | Share |
| --- | ---: | ---: |
| Bulk emitter: 5,127,300 complete records | 153.8 million | 74.3% |
| Run fragments, through provisional labels | 14.9 million | 7.2% |
| One row at a time, through `one` | 38.3 million | 18.5% |

So the 12-instruction loop of the next section handles about a fifth of the rows. The builder also finishes 1,723,371 full ordinary groups and 30,949 wide ones.

## 7\. How the Rust becomes twelve instructions

Nothing in either builder is hand-written assembly. Both are Rust, compiled by `rustc` with LLVM. The method is to read what the compiler produced, find work that doesn't need doing, and change the Rust so that the compiler has less to emit. The first builder's 49-instruction loop wasn't badly compiled; it was doing a lot. The second builder asks for less, so it gets a shorter loop.

This is that loop, from the [prototype's build](<https://github.com/lmmx/mibz2/blob/cd14fa9/docs/journal/2026-11-03-compact30-constructor-v2-prototype/excerpts.asm>), in the AT&amp;T syntax `objdump` prints by default (source first, destination last). The [build that was measured](<https://github.com/lmmx/mibz2/blob/1dc2edf/docs/journal/2026-11-03-compact30-construction-v2-harness/listings/ctor2-build-false.asm>) has the same instructions at different addresses.

```text
movzbl (%r15),%esi              ; c = the next byte of L
lea    (%rsi,%rsi,4),%rcx       ; 5c
lea    (%r8,%rcx,8),%r14        ; &desc[c]: descriptors are 40 bytes, 8 × 5c
mov    (%r8,%rcx,8),%rax        ; desc[c].write
cmp    0x8(%r8,%rcx,8),%rax     ; == desc[c].end ?
je     3440d                    ;   then refill (out of line)
mov    %bx,(%rax)               ; *write = the 16-bit label
addq   $0x2,(%r14)              ; desc[c].write += 2 bytes
inc    %r15                     ; next byte of L
inc    %ebx                     ; next label
cmp    %r12,%r15                ; end of this stretch of L?
jne    343e0
```

| Instructions | What they do |
| --- | --- |
| `movzbl` | read one byte of \\(L\\) |
| two `lea` | compute that byte's descriptor address: `lea` does arithmetic on addresses without touching memory |
| `mov`, `cmp`, `je` | load the write pointer, compare it with the end pointer, leave if full |
| `mov`, `addq` | store the label, advance the pointer |
| `inc`, `inc`, `cmp`, `jne` | next byte, next label, loop |

A few Rust details helped:

- `#[inline(always)]` on `one` puts its body in the loop.
- `#[cold]` and `#[inline(never)]` on `refill` keep the rare case out of it.
- The builder is generic over a constant, `build::<COUNT>`. With `COUNT = true` it keeps structural counters for analysis, and with `false`, the version that is measured, every counting branch disappears at compile time.

You can see all of this yourself on an x86-64 Linux machine. Build the frozen source and disassemble it:

```sh
git clone https://github.com/lmmx/mibz2 && cd mibz2
git checkout 8beffe2   # the frozen second builder
rustup toolchain install 1.97.0 --profile minimal
cargo +1.97.0 build --locked --release --example c30walk
objdump -d -C --no-show-raw-insn target/release/examples/c30walk > c30walk.asm
```

Then search `c30walk.asm` for `ctor2::Ctor2>::build::<false>`. The [annotated excerpts](<https://github.com/lmmx/mibz2/blob/cd14fa9/docs/journal/2026-11-03-compact30-constructor-v2-prototype/excerpts.asm>) are a shorter place to start. They pick out the two one-row loops, the bulk emitter and the fast path through `finish`. Addresses will differ from build to build. The archived listings are the reference for the counts quoted here.

## 8\. Fewer instructions, not yet a cheaper build

The second builder passed the same checks as the first, and more:

- byte-for-byte equality with the reference builder on 11 corpora and 53,924 synthetic cases, chosen to hit epoch boundaries, absent bytes, crossing groups, both span limits and malformed counts;
- an independent audit of its unsafe code;
- 542 cases run under [Miri](<https://github.com/rust-lang/miri>), Rust's checker for undefined behaviour, including targeted runs of the cases a first, interrupted run hadn't reached.

Then it was timed against production's builder on the i9 workstation, under a protocol fixed in advance. That measurement is also formally invalid. Production's own four-worker cost wasn't stable enough across its 20 rounds, almost entirely because of one slow round. Read only as descriptions, the figures are:

| Four workers, compact30 ÷ production | First builder | Second builder | Limit |
| --- | ---: | ---: | ---: |
| CPU cost of building | 1.316 | 1.086 | 1.067 |
| Time until all four have built | 1.364 | 1.114 | 1.02 |

The two builders were measured in separate runs, so the drop isn't a controlled comparison. But it's large, and every one of the 19 normal rounds showed the second builder taking between 1.110 and 1.144 times as long as production. Re-running the same measurement just to turn "invalid" into "fail" wasn't worth the workstation's time.

The figures point to two reasons the shorter loop wasn't enough.

**The work moved rather than vanished.** Each row that goes through a provisional label is written twice, once as a label and once as an offset, and read in between. Across the corpus, normalisation reads about 103 MB of fields and writes about 103 MB back, roughly one extra byte each way per row. The first builder did almost none of that: its field loads were 0.002 per row, against the second's 0.274. The fast path through `finish` is 81 instructions, and a group that rolls over through `refill` adds another 62. That's about 143 per group, against 12 per row in the loop.

**The slowest worker sets the pace.** The second limit is about completion. The four workers build in lockstep, and a step ends when the last of them finishes. If three take 9 ms and one takes 11 ms, the step takes 11 ms, and speeding up the other three changes nothing. The second builder's cost varies more from block to block than production's does: blocks that use the bulk emitter less are relatively more expensive. On the full blocks, about 1.18 times production's cost in the quarter with the least bulk coverage, against about 0.98 times in the quarter with the most. So the slowest worker in a step falls further behind the average under compact30. Weighted by time, the slowest worker took 1.073 times the mean worker's time with the second builder, against 1.047 with production's. That difference is enough, as arithmetic, to turn a CPU ratio of 1.086 into a completion ratio of about 1.114. It is a description, not a diagnosis: the records can't tell whether those builds are slower because of more instructions, cache misses or something else.

An assembly-level review of the second builder found more removable work. One example is in the loop above. `mov (%r8,%rcx,8),%rax` has already loaded the write pointer into a register. But `addq $0x2,(%r14)` adds to the copy in memory, which means reading the pointer again. The compiler could instead store the incremented register value, removing one load from each of 38 million rows. Whether that would save time depends on what the processor was waiting on, which an instruction count can't tell you.

The review's strongest candidate was a specialised path for the common case of an ordinary group followed by another group of the same byte. The candidate would finish one and set up the next without the general machinery, removing perhaps 40 to 50 instructions per group. The review modelled what that would buy, using the measured records. To reach just the completion limit, without the margin a valid pass needs, it would have to save about 6.6 ns, around 33 cycles, on every one of the 1.65 million fast group finishes. The assembly doesn't show that much removable time. So the review advised against building that change on its own.

That leaves the question this work is now on. A single local change won't close the gap. Would all the removable work together, counted across the whole builder without counting any saving twice, be enough? A fuller accounting of the builder's machine code is under way, weighted by how often each path runs and by which workers finish last. Until it reports, the second builder stays as it is. It is correct, and much cheaper than the first, but it doesn't yet meet the budget the traversal result set.

The general lesson outlasts this builder. A computation that is expensive done once per element can become cheap if its results are held in a different form for a while, and the expensive part is done once per group. The cost doesn't disappear, though: it moves, and it has to be found again where it went.

---

**Research records:** [Technical appendix](<https://lmmx.github.io/mibz2/pr-preview/pr-379/blog/2026-11-04-compact30-constructor-notes/index.md>) · [First builder](<https://github.com/lmmx/mibz2/blob/e003a32/examples/c30walk/ctor.rs>) · [Second builder](<https://github.com/lmmx/mibz2/blob/8beffe2/examples/c30walk/ctor2.rs>) · [Its design](<https://github.com/lmmx/mibz2/blob/25891bd/docs/journal/2026-11-03-compact30-constructor-v2-design.md>) · [Prototype, counters and machine code](<https://github.com/lmmx/mibz2/blob/cd14fa9/docs/journal/2026-11-03-compact30-constructor-v2-prototype.md>) · [Construction-cost protocol](<https://github.com/lmmx/mibz2/blob/1dc2edf/docs/journal/2026-11-03-compact30-construction-protocol-v2.md>) · [The measurement and its analysis](<https://github.com/lmmx/mibz2/blob/25891bd/docs/journal/2026-11-03-compact30-construction-v2-capture.md>) · [Assembly-level review](<https://github.com/lmmx/mibz2/blob/25891bd/docs/journal/2026-11-03-compact30-constructor-v2-reassessment.md>).
