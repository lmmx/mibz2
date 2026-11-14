[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-382/blog/index.md>)

# Investigating inverse-BWT memory interference in mibz2

Four workers finish decompression sooner, but together spend 32% more processor time per byte. We're testing whether competition for shared memory is responsible.

1 November 2026 · 14 min read · [Research in progress](<#7-where-things-stand>)

A bzip2 file is a series of independent blocks, so four CPU cores can decode four blocks at once, each with its own data, its own working memory and its own instructions. That's what `mibz2` does, and a file finishes much sooner on four cores than on one.

You'd expect the total work to stay about the same, just spread across more cores. It doesn't. With four workers instead of one, `mibz2` spends about 32% more CPU time, added up over all the cores, for every gigabyte it decodes.

Little of the extra time goes on handing work between threads. Most of it is inside the decoding itself, where the decoder runs almost the same instructions per byte as before (1.3% more) but needs about 30% more processor cycles to get through them. The same instructions are running slower.

Memory is the obvious suspect. The most expensive step in decoding a block is undoing the Burrows–Wheeler transform, which means following a long chain of links through a table of about 3.6 MB. A core working alone can keep much of that table close at hand in the processor's caches. With four cores at work, four such tables, along with the buffers each worker uses to build and assemble its output, all have to share the processor's last-level cache and the hardware that serves its memory requests.

The suspected problem

A sketch, not a measurement of what's in the cache. Each core has its own table, but all four share the last-level cache and the path to main memory.

If that's what's happening, one worker's memory accesses push out data that another worker was about to use, so its next lookup has to fetch it from further away. Or the shared cache is simply busier, so every request takes longer to answer. Either way, the core spends more of its cycles waiting for data. That's what we mean by memory interference: workers whose logic is independent, competing for the same hardware.

The measurements point that way, but they don't settle it. This post is about two experiments proposed to find out more. One changes how the decoder reuses its memory, and measures whether the extra cost shrinks. The other keeps one decoder fixed and varies what the other cores do with memory, to see which kinds of competition reproduce the slowdown. We're starting with the first, and the rest of the post explains why.

## 1\. What a worker does with each block

Each block goes through five steps. Two of the worker's memory buffers, `ll` and `rle`, come up throughout the rest of the post.

From compressed symbols to decoded bytes

What a worker does with each block. The worker keeps ll and rle from block to block and refills them each time.

1. **`ll` holds the transformed bytes.** bzip2 compressed the block after rearranging its bytes with the Burrows–Wheeler transform (BWT), and the move-to-front (MTF) decoding stage recovers those rearranged bytes into `ll`. They are the right bytes in the wrong order: the last column of the transform's sorted table.
2. **The T-vector is built from `ll`.** Building this table reads each byte of `ll` once. Each entry says which row comes next and which byte to write, so following the links from the right starting point spells out the block in its original order. An [earlier post](<https://lmmx.github.io/mibz2/pr-preview/pr-382/blog/2026-10-29-compact-navigation-tables/index.md>) explains the table in more detail.
3. **The traversal follows the links, and assembly joins the pieces.** Each lookup depends on the one before, so a single chain of lookups spends most of its time waiting for memory. `mibz2` runs 32 chains at once, each rebuilding its own stretch of the block, and switches between them while one waits (the [32 lanes post](<https://lmmx.github.io/mibz2/pr-preview/pr-382/blog/2026-10-31-room-walk-overlap/index.md>) covers how). The finished stretches are then copied, in order, into one buffer.
4. **That buffer is `rle`.** The block is back in its original order, with one encoding left: before applying the transform, bzip2 had shortened every run of four or more identical bytes into a few bytes and a count.
5. **RLE1 expands those runs** into the decoded output, and checks the block's checksum (CRC).

`ll` and `rle` aren't registers, and they aren't algorithms. They are two areas of memory holding the same block at different stages: `ll` before the BWT is undone, `rle` after it but before the runs are expanded. In the code they are two fields of each worker's scratch space:

```rust
ll: Vec<u8>,   // The BWT's last column, written by MTF
rle: Vec<u8>,  // The assembled sequence, before RLE1 expands its runs
```

## 2\. Where the extra time goes

We measure efficiency in CPU-seconds per gigabyte of decoded output (CPU-s/GB): the processor time of all the workers added together, per GB produced. Four workers decode a file in much less time than one, but on this measure they cost more.

The cost of decoding on four cores

Measured on four physical cores. Left: the whole decoder. Right: the traversal and assembly step, on 197 matched full-sized blocks. Both measure CPU time, not elapsed time.

The whole decoder goes from 2.707 to 3.579 CPU-s/GB. Timing each step of each block separately shows where the increase lands:

| Step | One worker, ns per row | Four workers, ns per row | Increase |
| --- | ---: | ---: | ---: |
| MTF decoding | 0.456 | 0.487 | 1.07× |
| T-vector construction | 0.497 | 0.571 | 1.15× |
| Traversal and assembly | 1.234 | 1.806 | 1.46× |
| RLE1 and CRC | 0.134 | 0.180 | 1.35× |

Traversal and assembly take about 79% of the added time across these four steps. Over the whole file, short blocks included, they add about 0.61 CPU-s/GB.

The timing so far can't split that step in two. It covers both the link-following and the copying of the finished pieces into `rle`, so we don't yet know how much of the 1.46× belongs to each.

The traces also show a pattern. A block's traversal took a median 1.29× its one-worker time when no other worker was traversing at the same moment, and 1.66× when all three others were. That fits the memory story, but it doesn't prove it: a slower traversal also overlaps more of the others' traversals simply by lasting longer.

## 3\. Why the counters don't settle it

The processor's performance counters narrow things down without picking a culprit.

Per decoded byte, the number of data requests that miss each core's private caches barely changes between one and four workers. Each request seems to take longer, though: by an approximate measure, it stays outstanding for about 58 processor cycles rather than 39. Traffic to main memory (DRAM) also rises sharply, especially writes.

Those clues fit several explanations:

- **The tables crowd each other out.** Each worker's table pushes the others' entries out of the last-level cache (the L3, shared by all the cores), so more lookups have to go to DRAM.
- **The shared cache is congested.** The lookups still find their data in the L3, but they queue for longer because four cores are asking at once.
- **The writes get in the way.** To write to memory, a core first takes exclusive ownership of the cache line, and the changed line must later be written back. Writes can also push out useful data and tie up the hardware that loads and prefetches need.

These feed into each other. A little more eviction makes prefetches arrive later, later requests hold hardware resources for longer, and that slows the requests behind them. The counters can't tell us which one starts the chain.

The request-duration figure is a good example. It counts the prefetches the traversal issues in advance alongside the lookups it actually waits for, so it shows that memory behaviour changed without telling us how long any single lookup waited. And the extra DRAM writes could be a symptom rather than a cause: the same crowding that slows the traversal would produce them too. The [research record](<https://github.com/lmmx/mibz2/blob/docs/bwt-memory-proposal/docs/journal/2026-11-01-bwt-memory-proposal.md>) has the counters in full.

So there are two ways forward: change how the decoder uses memory and see what happens, or recreate the competition under controlled conditions and see what it does. Astra and Claude each proposed one.

## 4\. Astra's proposal: write the output into `ll`

Our [research register](<https://github.com/lmmx/mibz2/blob/9723f06847b88a10ba93d07c3f35680e58766015/docs/journal/2026-10-30-research-portfolio-register.md>) is a list of about 165 ideas for speeding up the decoder, gathered from the project's journals, branches, issues and pull requests, each with its status and what would test it. Entry P87, "walk output in L's freed buffer", had been recorded there but never tried. Astra's experiment is a careful test of it.

The idea rests on when each buffer's contents stop mattering. Once the T-vector is built, nothing reads the bytes in `ll` again for that block: the traversal reads only the table. The bytes are dead, but the memory holding them, about 0.9 MB for a full block, is still allocated, and it sits idle until the next block's MTF output fills it.

P87 writes the assembled bytes into that idle memory instead of into `rle`. The bytes written are identical, and so is the copy that writes them. What changes is which memory the copy lands in.

The proposed P87 comparison

Both arms build and traverse the same table, produce the same pieces and copy them the same way, so the assembled bytes are identical. Only the memory receiving them differs.

### Why it could help

Writing into `ll` means the worker touches about 0.9 MB less distinct memory per block, and so needs fewer separate cache lines in a writable state. That could mean less of the crowding, fewer ownership requests and fewer write-backs to main memory. Any saving would matter most when another worker is following its own table at the same time.

There's a subtlety in the timing. Assembly happens after the traversal it follows, so changing where the assembly writes can't speed up that same traversal. If the traversal gets cheaper under P87, the saving must come indirectly: from less competition with the other workers, from a different cache state carried into the next block, or from the steps overlapping differently. A cheaper traversal would therefore tell us more than a cheaper assembly.

### Why it might not

Both buffers are already reused from block to block. `ll` was written by MTF and read by the table build shortly before; `rle` was last read by RLE1 during the previous block. By the time assembly starts, the traversal may have pushed most of either one out of the cache, in which case reusing `ll` saves nothing.

The experiment measures whether the difference matters rather than assuming it does. To keep the comparison fair, both versions allocate both buffers at full size and touch their memory once in advance. Otherwise a gain could come from simply allocating less, or from the operating system mapping fewer fresh pages. Whether a production version could drop `rle` altogether, and save the memory, would be a separate question.

## 5\. Claude's proposal: vary what the neighbours do

Claude's proposal comes at the problem from the other side. Instead of changing the decoder, it measures one decoding worker while controlling what the other three cores do.

The measured worker decodes as usual, building and traversing its tables, with counters around each step. The three neighbouring cores run one chosen kind of workload at a time.

One measured worker, three controlled neighbours

The measured worker is held fixed while the three neighbours run controlled workloads. The diagram shows what the cores share; it doesn't imply that each shared resource can be varied on its own.

Each workload stresses a different part of the memory system:

| Neighbour workload | What it tests |
| --- | --- |
| Real decoding work | Whether the setup reproduces the real slowdown at all |
| Only traversal, or only table building | Whether traversal suffers more from other traversals or from table builds |
| Read-only data of several sizes | Whether a bigger footprint in the shared cache makes traversal steadily slower |
| Heavy writers | Whether writes and write-backs are especially damaging |
| Many L3 requests from a small footprint | Whether request traffic alone, without much cache space, reproduces the slowdown |
| Streaming to and from DRAM | Whether main-memory traffic reproduces it |

The results could separate the explanations from section 3. If traversal slows steadily as the neighbours' read-only data grows, even though they make few requests, cache capacity looks like the problem. If a small workload making many L3 requests slows it just as much, while large but rarely touched data does little, congestion in the shared cache looks more likely. Neighbours that only traverse, or only build, would also show whether the pattern from section 2, slower traversals when more traversals overlap, holds up under control.

### The catch

These workloads don't isolate one hardware resource each. A workload that holds data in the cache has to keep touching it, which makes requests of its own. A heavy writer changes the ownership traffic as well as the write-backs. A DRAM stream can match the decoder's average bandwidth with a quite different mix of reads, writes and bursts. So the experiment would measure how sensitive a traversal is to each kind of neighbour, rather than assign the real slowdown, fraction by fraction, to particular hardware.

It also depends on the setup behaving like the real decoder in the first place. Four independent decoding loops lack the real decoder's scanner, its buffer reuse and its mix of overlapping traversals. Claude proposed requiring real decoding neighbours to reproduce at least 70% of the real slowdown before going further. That's a sensible gate, but failing it would only show that the setup wasn't realistic enough. It wouldn't show that the real decoder is free of interference. Making the setup realistic enough is a project in itself.

## 6\. Why P87 comes first

|  | P87: reuse `ll` | Controlled neighbours |
| --- | --- | --- |
| Changes | Which memory the assembly writes into | What the other cores do with memory |
| Measures | The whole decoder's efficiency | One fixed traversal's cost |
| Strength | Directly tests a change we could ship | Can identify a broader cause |
| Limitation | Tests one change only | Results depend on how realistic the setup is |
| If it succeeds | Validate the change for production | Design a change aimed at the resource it identifies |

P87 doesn't come first because it has more to gain. The neighbour experiment may well point to a bigger opportunity, in how the tables sit in the cache or how requests compete for it. P87 comes first because it asks a smaller question whose answer changes what we do next. If it works, we have an improvement that keeps the current four-byte table and the 32-chain traversal. If it doesn't, we can drop this idea without first having to explain the whole memory system.

### What would count as success

Astra proposed measuring both versions, the current one (A) and the one that reuses `ll` (B), at one worker and at four. Write \\(C\_{X,w}\\) for the CPU-seconds per GB of version \\(X\\) with \\(w\\) workers. The saving that appears only under concurrency is then

\\\[ D = (C\_{A,4} - C\_{B,4}) - (C\_{A,1} - C\_{B,1}) \\\]

which takes the four-worker saving and subtracts whatever saving B already gives a single worker. Astra suggested 0.10 CPU-s/GB as the size that would count as material.

\\(D\\) answers one question, and a second needs its own test:

The two can disagree. A change could make both versions slower while shrinking the gap between them, giving a positive \\(D\\) with a slower four-worker decoder. And reuse might make assembly or RLE1 cheaper by the same amount at one and four workers: a worthwhile small optimisation, with \\(D\\) near zero and nothing learned about concurrency.

To tell these cases apart, the experiment times table building, traversal, assembly and RLE1 separately. It also keeps the table building and traversal code identical in both versions, gives both the same memory, and leaves the scheduling of workers alone, so that the comparison is about where the output goes and nothing else.

The research register's original entry for P87 set a different bar, for six to ten workers. That remains a separate question from this four-core experiment.

## 7\. Where things stand

Neither experiment has shown a speedup yet. P87 is the one we're running first, and the neighbour experiment remains a broader follow-up. What we'd learn depends on the result:

| P87 result | What it would tell us | What we'd do next |
| --- | --- | --- |
| Faster and cheaper at four workers, with a material \\(D\\) | Where the output goes contributes to the four-worker cost | Validate the change, and look at the lifetimes of other buffers |
| Cheaper assembly or RLE1, but \\(D\\) near zero | Reuse helps locally without explaining the traversal slowdown | Treat it as a small optimisation |
| Fewer writes to main memory, but the same traversal cost | These particular writes weren't what slowed the traversal | Stop treating them as a likely cause |
| No meaningful change | This kind of reuse doesn't help under these conditions | Go back to the neighbour experiment, with a realistic setup |
| The measurement itself proves unreliable | Nothing yet | Fix the experiment before reading anything into it |

Even a material result of 0.10 CPU-s/GB would recover only about 2.8% of the four-worker decoder's CPU cost. That's worth having, but well short of the 1.5–2× gain in throughput we're ultimately after. Getting there will probably take a deeper change to how the table is laid out in memory, how far ahead it's prefetched, or how lookups are served, and the earlier work shows how carefully that has to be done: a smaller table saves cache space for nothing if recovering each link from it becomes too slow. For now the fast four-byte table stays, and P87 leaves it untouched.

The two proposals ask different things. Changing the decoder asks whether one decision about where data lives makes it more efficient. Changing the neighbours asks what kinds of competition make an unchanged traversal more expensive. Both are sound ways to study a performance problem; they just produce different kinds of evidence. We're starting with the direct change because it's smaller, easier to interpret and bears directly on the decoder people use. If it fails, that's a better reason to invest in the broader experiment.

So the question for now is narrower than "why is four-worker decoding expensive?": can writing the output into memory the worker has finished with make concurrent decoding cheaper, without touching the traversal itself?

---

**Research records:** [Inverse-BWT memory proposal and phase analysis](<https://github.com/lmmx/mibz2/blob/docs/bwt-memory-proposal/docs/journal/2026-11-01-bwt-memory-proposal.md>) · [Research register, including P87](<https://github.com/lmmx/mibz2/blob/9723f06847b88a10ba93d07c3f35680e58766015/docs/journal/2026-10-30-research-portfolio-register.md>) · [Dispatch-causality investigation](<https://github.com/lmmx/mibz2/blob/docs/dispatch-causality-records/docs/journal/2026-11-01-dispatch-causality-i9-10900K/analysis.md>).
