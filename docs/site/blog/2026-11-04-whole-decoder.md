---
title: "A faster builder, a slower decoder, and an unrolling our record had already tested"
image: assets/social/blog-2026-11-04-whole-decoder.png
description: "A table builder that ran a fifth fewer instructions made the bzip2 decoder 7–19% slower. Counting the whole decoder led to the walk, and to a change that made one thread 2–4% faster. Our own record had already tested that direction, and found no gain at four workers."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# A faster builder, a slower decoder, and an unrolling our record had already tested

A table builder that ran a fifth fewer instructions made the bzip2 decoder 7–19% slower. Counting the whole decoder led to the walk, and to a change that made one thread 2–4% faster. Our own record had already tested that direction, and found no gain at four workers.
{: .post-dek }

4 November 2026 · 11 min read · [Research in progress](#7-what-is-still-open)
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
table exactly as it was and removes a fifth of the instructions it spends on each block. It
made one thread faster, with no gain detected at four. Then we found that our own record had
already tested a closely related change, with the same outcome.

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
cheaply. It is what the production walk spends on each step.

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

## 5. Instructions are not time

The continuous-integration benchmark measured it twice, on the same walk machine code, landing
on two different processors. The Wikidata corpus, paired over ten rounds each:

| Threads | EPYC 9V45, wall | EPYC 7763, wall |
| --: | --: | --: |
| 1 | −4.2% (−8.0%, −0.3%) | −2.4% (−3.6%, −1.2%) |
| 2 | +2.2% (−0.6%, +5.1%) | −4.9% (−9.5%, −0.1%) |
| 4 | +0.4% (−2.8%, +3.8%) | +0.2% (−2.1%, +2.7%) |

At one thread it is faster on both. At four, the configuration a full decode uses, neither
machine shows a gain beyond noise. That is not a demonstrated zero: both intervals include
small gains and small losses. On the 7763 the benchmark also times each stage on one thread: the walk
went from 4.04 to 3.78 cycles per byte, a ratio of 0.925 (0.856 to 0.999). A 36% cut in the
walk's instructions bought about 7.5% of its time on one thread, and nothing the benchmark
could detect on the decoder at four.

The benchmark covers six other corpora as controls. One of them moved the wrong way. On the
7763, `pattern`, a short repeating input, was 1.6% slower at one thread (+0.1% to +3.1%). Its
CPU interval included no change, and on the 9V45 the same row was 5.3% *faster* (−6.8% to
−3.8%). With 21 wall rows a run, a flag like that can be chance. It can also be a real cost on
one kind of core. It hasn't been reproduced either way, and until it has, it is a trade-off to
weigh, not a regression we can rule out.

## 6. We had tested this direction before

On 1 November, three days earlier, a registered trial had unrolled the same walk. It wrote all
32 lane steps out with fixed offsets, and [the 2 November post on memory-level
parallelism](2026-11-02-memory-level-parallelism.md) reported it:

| | 1 November trial | This change (#157) |
| --- | --: | --: |
| Instructions per lane step | 16.4 → 10.44 | 16 → 9 |
| Walk alone, EPYC 7763 | 0.877 (0.863–0.891), isolated walk | 0.925 (0.856–0.999), stage timing |
| Decoder, one worker | +1.7% throughput (i9-10900K) | −2.4% to −4.2% wall (CI) |
| Decoder, four workers | 1.000 (0.987–1.012) (i9-10900K) | +0.2% to +0.4% wall, neither interval clear of zero (CI) |
| A control costing more | `records`: +1.8% CPU at four workers | `pattern`: +1.6% wall at one thread, 7763 only |
| Verdict | negative, at four workers | met the one-thread merge gate |

The two changes are not identical. #157 unrolls eight lanes a turn rather than all 32, and
defers the handovers to a mask. The machines and harnesses differ too, so the rows are not a
paired comparison. It is a precedent, not a replication. But the shape is the same: a large cut in
instructions, a small gain on one worker, none detected on four, and a control workload that
got more expensive. The earlier trial's
exploratory CI rows at one worker were, if anything, larger than #157's: +17.9% throughput on
the 9V45 and +8.1% on the 7763.

So the record already predicted, in outline, what #157 would find. We didn't read it before building. A search
for "unroll" found the trial, but its output was cut short before the table that held the
result, and nothing required the decisive evidence to be read before implementation began. Our
independent agents recovered it, but only after the merge.

Two earlier claims in drafts of this post are withdrawn. The first said the change is the
"kernel B" that an analysis from 30 October, B2, had designed and priced. It isn't. B2's kernel
B buffers eight successive steps per lane and commits them eight bytes at a time, replaying a
tile that fails. #157 writes every byte as it goes and unrolls across lanes, not steps. Kernel
B remains unbuilt. The second claim was that the 7763's walk figure rejected B2's model of the
walk's time. It can't: the model's predictions were for kernel B, and the stage figure includes
segment assembly, which B2's boundary excludes. The model is untested.

## 7. What is still open

**Shipping.** The change met the merge gate, a one-thread gain beyond noise confirmed on two
processors, and is on master, unreleased. It does not meet the campaign's goal, which is a
faster decoder at four workers. Before a release carries it, the `pattern` flag will be
measured again on a 7763.

**Why more workers erase the gain.** Neither trial detected a gain at four workers, across
three kinds of machine. The earlier trial's explanation, pressure on the load-address ports, came from a
static model, not a measurement. Three readings make different predictions:
- **a limit private to each core,** such as how many misses it can track: SMT pairs lose the
  gain first;
- **a penalty shared across cores:** the gain shrinks towards nothing as copies are added,
  whether or not they share a core;
- **higher latency for every miss as more are in flight:** the ratio between the two walks
  holds as copies are added.

The experiment that separates them times the walk alone, the old against the new, at one copy,
two copies on separate cores, and two on the siblings of one core, on each machine. [Memory
interference](2026-11-01-inverse-bwt-memory-interference.md) measured a related effect on a
four-core machine, about 30% more cycles at four workers for 1.3% more instructions, without
settling its cause. CI's four-thread runs are two cores with SMT, not four cores.

**What fewer instructions can't buy.** Two experiments have now cut the walk's instructions by
more than a third, and neither detected a gain at four workers. The next gains at four are more likely
to come from fewer misses per byte, or from overlapping them better across cores, than from
fewer instructions per step. That is a hypothesis, and the experiment above is what tests it.

**The other walks.** The cooperative walk, used on machines with many cores, and the flagged
walk carry the same rolled loop. Porting the change waits on that experiment.

The lesson is about memory. A faster builder made the decoder slower because nobody had
counted the walk. Counting it led to a change in a direction our own record had already tested, and the
record held the outcome we went on to rediscover. The project now retrieves a candidate's prior record
before building it and again before integrating it, reading the results themselves rather
than a search excerpt. The pull request then says whether the work is new or a re-test, and
what could change the earlier verdict.

---

**Research records:** [The port, PR #155](https://github.com/lmmx/mibz2/pull/155) · [Its
write-up](https://github.com/lmmx/mibz2/blob/8abefea4/docs/journal/2026-11-04-compact30-production-port.md)
· [Its profiles](https://github.com/lmmx/mibz2/tree/c493a45d/docs/journal/compact30-channel/evidence/port-b4786f66)
· [The walk change, PR #157](https://github.com/lmmx/mibz2/pull/157) · [Its profiles, listings
and tests](https://github.com/lmmx/mibz2/tree/b0f6e89d/docs/journal/compact30-channel/evidence/walk-75b13f00)
· [The 1 November walk-unroll trial: decoder
results](https://github.com/lmmx/mibz2/blob/270333c2d32fe16dc24ae7ab64f3697d87939418/docs/journal/2026-11-01-walk-unroll-decoder-i9-10900K/results.md)
· [Its assembly
comparison](https://github.com/lmmx/mibz2/blob/270333c2d32fe16dc24ae7ab64f3697d87939418/docs/journal/2026-11-01-walk-unroll-astra-fidelity/README.md)
· [B2, the gathered walk: the model and kernel
B](https://github.com/lmmx/mibz2/blob/9723f068/docs/journal/2026-10-30-b2-gathered-walk.md)
· [The research register, §6](https://github.com/lmmx/mibz2/blob/3ace9d33/docs/journal/2026-10-30-research-portfolio-register.md)
· [Whole-decoder accounting, charter
§5.6](https://github.com/lmmx/mibz2/blob/45e6cfeb/docs/journal/2026-11-04-compact30-research-charter.md)
