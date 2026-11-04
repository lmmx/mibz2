---
title: "A faster table builder made the decoder slower: what the failure found in the walk"
image: assets/social/blog-2026-11-04-whole-decoder.png
description: "A builder that ran 20% fewer instructions made the whole bzip2 decoder 7–19% slower. Counting the whole decoder instead found the work in the walk, where an unbuilt design from our own record removed a third of it."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# A faster table builder made the decoder slower: what the failure found in the walk

A builder that ran 20% fewer instructions made the whole bzip2 decoder 7–19% slower. Counting
the whole decoder instead found the work in the walk, where an unbuilt design from our own
record removed a third of it.
{: .post-dek }

4 November 2026 · 10 min read · [Research in progress](#6-what-is-still-open)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

[The last post](2026-11-04-compact30-constructor.md) described a builder for compact30, a
smaller table for undoing the Burrows–Wheeler transform: about 2.1 MB per block where the
production decoder's table takes 3.6 MB. Its inner loop fell from 49 instructions to 12, and
across our reference corpus it built the table with about a fifth fewer instructions than the
builder it replaced.

This post is about what happened when we put it into the production decoder. The decoder got
slower. Understanding why led to a change in the production decoder itself, one that leaves its
table exactly as it was and removes a fifth of the instructions it spends on each block.

## 1. Three phases, one budget

After its Huffman and move-to-front stages, `mibz2` turns each bzip2 block back into bytes in
three steps:

1. **Build** a navigation table from the block's transformed bytes: for each row, which row
   comes next, and which byte to emit.
2. **Walk** the table. Starting from a known row, follow the links, writing one byte per step.
   The decoder runs 32 independent walks ("lanes") at once, each through its own segment of the
   block, so that while one lane waits for memory the others keep going ([32 lanes, 8.6
   chains](2026-10-31-room-walk-overlap.md)).
3. **The rest:** undo bzip2's run-length coding, check the CRC, and assemble the segments.

Here is where the instructions go on one full Wikidata block of 899,971 bytes, counted with
Callgrind on production `decode_block`:

| Phase | Instructions | Share |
| --- | --: | --: |
| Build | 5,429,062 | 21% |
| Walk | 15,426,892 | 60% |
| The rest | 4,895,842 | 19% |
| **Whole block** | **25,751,796** | |

The compact30 work had spent a week on the first row. The walk, three times as large, had been
treated as settled.

## 2. The port

The research builder went into production as the default kernel ([PR
#155](https://github.com/lmmx/mibz2/pull/155)), with the production table kept as a fallback.
On the same block:

| Phase | Production | compact30 | Change |
| --- | --: | --: | --: |
| Build | 5,429,062 | 4,369,204 | −1,059,858 (−19.5%) |
| Walk | 15,426,892 | 20,207,700 | **+4,780,808 (+31.0%)** |
| The rest | 4,895,842 | 4,953,557 | +57,715 |
| **Whole block** | **25,751,796** | **29,530,461** | **+3,778,665 (+14.7%)** |

The builder's saving survived the port. But the walk over the smaller table costs about 5.3
more instructions for every byte it writes, and that is more than the builder saves.

The reason is in the table's shape. A production entry is one 32-bit word holding both the next
row and the byte, so a step is one load. compact30 packs 30 rows into a 64-byte record: a 16-bit
offset per row and a 32-bit header shared by the group, holding the group's base row and its
byte. A step therefore loads the offset, works out which group the row is in, loads the header,
and adds. The table is smaller because each step does more.

The continuous-integration benchmark ran both builds on the same runner, paired over ten
rounds. The compact30 decoder was 18.7% slower at one thread, 11.0% at two and 7.3% at four,
with every interval excluding no change. CPU per byte rose about 18% at one thread, close to
the 14.7% rise in instructions, so a smaller table bought no net speed on that machine. The PR
was closed unmerged ([the write-up](https://github.com/lmmx/mibz2/blob/8abefea4/docs/journal/2026-11-04-compact30-production-port.md)).

Two things had made it look worthwhile. On our shared development machine it ran 3–4%
faster, with intervals clear of zero. A smaller table fitting better in cache was a plausible
explanation, and it didn't hold on the machine that decides. And the research gate counted the
builder's instructions only, against a research baseline rather than production. Both are now
fixed in the project's rules: every candidate is counted across the whole decoder against
production before it goes near a pull request, and local timing is never the reason to open
one.

## 3. Looking at the walk itself

If the walk is three-fifths of the work, the question is no longer how to build a table more
cheaply. It is what the production walk spends on each step. That question was already in our
own record. [32 lanes, 8.6 chains](2026-10-31-room-walk-overlap.md) had proposed "Path B": the
walk already overlaps its memory lookups well enough, so spend fewer instructions on each step
and keep all 32 chains. A read-only analysis from 30 October, B2 (linked below), had gone
further. It counted the shipped loop at 14.8 operations per byte, designed a kernel B ("the
same 32 scalar chains with deferred flags and tiled commits") at 8–9, and fitted a model
predicting what B would buy on each of CI's processors. Nobody had built it.

We came to it from the profiles, and only then recognised it as B. Here is what the profiles
showed.

Callgrind can count every instruction separately. Here is the production walk's hot loop, as
the compiler emitted it, with the count of each instruction on that block:

```text
  841,989  lea    r10,[r8+0x1]                 ; loop control
  841,989  mov    ebx,DWORD PTR [r11]          ; the lane's row, from memory
  841,989  mov    eax,DWORD PTR [r15+rbx*4]    ; T[row]: the lookup
  841,989  test   eax,eax                      ; does a segment start here?
  841,989  js     ...                          ;   (rarely)
  841,509  mov    edi,eax
  841,509  shr    edi,0x8                      ; the next row
  841,509  mov    DWORD PTR [r11],edi          ; the lane's row, back to memory
  841,509  prefetcht0 BYTE PTR [r15+rdi*4]     ; ask for the next lookup early
  841,509  mov    rdi,QWORD PTR [rsp+r8*8+0x70]; this lane's output pointer
  841,509  mov    BYTE PTR [rdi+rcx*1],al      ; the byte
  841,509  lea    rax,[r10-0x1]                ; loop control
  841,509  add    r11,0x20                     ; loop control
  841,509  mov    r8,r10                       ; loop control
  841,509  cmp    rax,0x1f                     ; loop control
  841,509  jb     ...                          ; loop control
```

That is sixteen instructions per step. About half of them do the decoding: read the lane's row,
look it up, test for a segment boundary, shift out the next row, prefetch, store the byte.
Most of the rest is the loop over the 32 lanes. The source says `for k in 0..32`, a fixed
count, and the compiler kept it as a counted loop instead of unrolling it.

It kept it rolled because of what sits inside the loop. When a lane reaches the end of its
segment, it hands that segment over and takes the next one from a queue. That handover is rare
(about 0.06% of steps) but large, and having it inside the loop body keeps the loop too big to
unroll.

## 4. Taking the handover out

The change has three parts ([PR #157](https://github.com/lmmx/mibz2/pull/157)).

**Step, then hand over.** Each round, every lane whose entry doesn't start a new segment takes
its step. A lane whose entry does is only marked, as a bit in a 64-bit mask. Once every lane
has stepped, the marked lanes are handed over in lane order.

This reorders work, so it has to be shown not to change the output. A step reads the table,
which nothing in the walk writes, and writes only the lane's own row and a byte in that lane's
own buffer. A handover changes only its own lane, the segment it finishes, the segment it takes
and the queue's position. So every step commutes with every other lane's handover, and the
handovers still run in lane order. The segments are taken in the same order, and every byte
lands where it did before.

The one delicate case is the queue running out partway through a round. The walk then leaves
the round for a slower tail loop, and the two orders can leave a later lane in different
states. In the old order it hasn't stepped yet. In the new one it has. The tail would take that
same step first, so the buffers end up the same, but the states in between are not. The
argument is a simulation relation over commuting steps, not equality of state. (Our
Researcher agent made that correction, and asked for tests that tell the two apart.)

**Unroll by hand.** Splitting the loop alone did nothing: the compiler still emitted the same
sixteen instructions. So the step is written out eight times per iteration, with a remainder
loop for lane counts that aren't a multiple of eight.

**Store before shifting.** The byte is the entry's low eight bits, and the next row is the
entry shifted right by eight. Storing the byte first lets the shift happen in place, which
saves a register copy.

A step is now nine instructions, plus about half an instruction of loop control:

| Block | Production | This change | Change |
| --- | --: | --: | --: |
| 1 (899,971 B) | 25,751,796 | 20,184,697 | −21.6% |
| 41 (900,000 B) | 25,168,380 | 19,480,512 | −22.6% |
| 233 (899,978 B) | 25,588,404 | 19,989,537 | −21.9% |
| 400 (899,993 B) | 25,129,254 | 19,490,536 | −22.4% |

The build and the rest are identical to the instruction. The whole difference is the walk,
which fell by 36% including its setup, handovers and tail. Small blocks take a simpler walk and
don't change. Output is identical on every block, and the tests include one built for this
change. It drives the walk through random columns until every new interleaving has occurred:
several lanes reaching segment ends in the same round, and the queue running out with marked
lanes still waiting, after a handover, and with a later lane already stepped onto its own
segment's end. It fails if the offsets the tail picks up from are changed.

That is kernel B, nine instructions a step against B2's predicted eight to nine. It is as large
a cut in whole-decoder instructions as anything the campaign has found. It leaves the table as
it was, so it composes with whatever else is done to the walk.

## 5. Instructions are not time, and the model was wrong about how much

The continuous-integration benchmark measured it twice, on the same walk machine code, landing
on two different processors. The Wikidata corpus, paired over ten rounds each:

| Threads | EPYC 9V45, wall | EPYC 7763, wall |
| --: | --: | --: |
| 1 | −4.2% (−8.0%, −0.3%) | −2.4% (−3.6%, −1.2%) |
| 2 | +2.2% (−0.6%, +5.1%) | −4.9% (−9.5%, −0.1%) |
| 4 | +0.4% (−2.8%, +3.8%) | +0.2% (−2.1%, +2.7%) |

At one thread it is faster on both. With more threads, the picture depends on the machine.

On the 7763 the benchmark also times each stage on one thread. The walk went from 4.04 to 3.78
cycles per byte, a ratio of 0.925 (0.856 to 0.999). A 36% cut in instructions bought 7.5% of
the walk's time.

B2 had predicted much more. Its best-supported model gives a step's time as its issue time
plus a miss penalty limited by the processor's window: w = U/D + ℓ·U/R. For the 7763 it
predicted B's walk at 0.73 to 0.76 of A's, and −10% to −11% of decoder time. The measurement
lies outside that range. So does the reading in which the walk is limited purely by its
instruction rate (0.55 to 0.65). What it doesn't exclude is the remaining hypothesis B2
listed: the walk is capped by how many outstanding misses one core can track, so fewer
instructions per step can't make it much faster (0.95 or more).

That is a useful negative result. It says the walk on this machine is close to limited by its
memory requests, not its instructions. The next large gains would then have to come from
fewer misses per byte, not fewer instructions per step.

## 6. What is still open

**Shipping.** The change met the merge gate. The one-thread gain was confirmed on two
processors, with nothing regressing beyond a small, unreplicated slip on one synthetic corpus.
It is now on master, unreleased.

**Why B2's model overestimated.** Either the miss term isn't ℓ·U/R as fitted, or the
latency of each miss rises when more are in flight, as B2's own risks list. B puts more
misses in flight than A. A refit to A and B together, which can now both be measured, should
say which term has to move.

**Why the multi-thread result depends on the machine.** On the 9V45, two threads lost the
gain; on the 7763 they kept it. There are several candidates:
- **A shared bottleneck.** Something shared across copies may bind.
- **The run's shape.** The benchmark's two-thread run puts four threads (two workers, the
  scanner and the reader) on two CPUs.
- **Interference.** It may be the effect [Investigating inverse-BWT memory
  interference](2026-11-01-inverse-bwt-memory-interference.md) measured on a four-core
  machine. That post proposed causes and tests, but didn't settle the cause. CI's four-thread
  runs are two cores with SMT, not four cores.

The cheapest decisive experiment is the falsifier B2 described: time the walk alone, A against
B, at one copy, two copies on separate cores, and on the two siblings of one core, on each
machine.

**The other walks.** The cooperative walk, used on machines with many cores, and the flagged
walk carry the same rolled loop. Whether to port the change waits on that experiment.

The lesson is about memory as much as measurement. A faster builder made the decoder slower
because nobody had counted the walk. Counting it found a change our own analysis had designed
and priced, and then left unbuilt. Building it tested the analysis, and part of it failed. The project's record is now treated as working memory:
every new direction starts from what earlier analyses proposed, measured or left untested.

---

**Research records:** [The port, PR #155](https://github.com/lmmx/mibz2/pull/155) · [Its
write-up](https://github.com/lmmx/mibz2/blob/8abefea4/docs/journal/2026-11-04-compact30-production-port.md)
· [Its profiles](https://github.com/lmmx/mibz2/tree/c493a45d/docs/journal/compact30-channel/evidence/port-b4786f66)
· [The walk change, PR #157](https://github.com/lmmx/mibz2/pull/157) · [Its profiles, listings
and tests](https://github.com/lmmx/mibz2/tree/b0f6e89d/docs/journal/compact30-channel/evidence/walk-75b13f00)
· [B2, the gathered walk: the model and kernel
B](https://github.com/lmmx/mibz2/blob/9723f068/docs/journal/2026-10-30-b2-gathered-walk.md)
· [The research register](https://github.com/lmmx/mibz2/blob/4dac7abc/docs/journal/2026-10-30-research-portfolio-register.md)
· [Whole-decoder accounting, charter
§5.6](https://github.com/lmmx/mibz2/blob/45e6cfeb/docs/journal/2026-11-04-compact30-research-charter.md)
