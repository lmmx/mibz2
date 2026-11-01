# External research report 132: the researcher's last contribution: a walkthrough of the pipeline and a minimal-walker ceiling test (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01.
- **Source:** the independent researcher, in conversation with the maintainer. The
  maintainer's two prompts (the second an audio transcription) are kept in place.
- **In reply to:** the maintainer's request, while Astra was writing report 131, for a
  walkthrough of the decoder in search of opportunities. The researcher's session hit its
  limit after the second reply, so this is its last contribution.

**Evidence cutoff:** 1 November 2026, after the P87 four-core capture and before report 131
arrived. It is source review, literature and experiment design: nothing was implemented or
run. Its literature references, caveats and cited figures (the 38.55 and 4.49 cycles per
byte walks among them) are the researcher's own and are not re-verified here.

It is kept exactly as received (rendered text, link cards and tracking parameters
included).

## The conversation, as received

`````text
ok and meanwhile i was chatting to the researcher while astra was responding and i asked it

while we  are waiting on astra, how about you walk me through it intelligibly and see if I can spot any opportunities...

it said:

Yes. And I'd like to approach this as an architecture-design conversation rather than another research review.

I'll explain what the machine is actually doing, where we spend resources, and which choices are fundamental to decompressing bzip2 versus merely choices made by `mibz2`. That's where I suspect you might spot something.

The central question I'd keep in mind is:

Why are we constructing, moving and copying all this data when ultimately we just want the original bytes in the correct order?

## 1. What happens when we decompress one block?

Imagine a block representing about 900 KB of data before the final run-length expansion.

The compressed file doesn't contain those bytes in their original sequence. It contains information from which the decoder reconstructs them.

Here's the actual pipeline, simplified.

Compressed bzip2 block

Huffman-coded information

A. Recover the rearranged bytes — `ll`

Move-to-front decoding produces the BWT's last column. About 0.9 MB for a full block.

B. Build a navigation table — T-vector

About 3.6 MB. For each position, it encodes where to go next and which byte to reconstruct.

C. Follow the links — 32 lanes

Reconstructs 512 separate pieces in temporary segment buffers, totalling roughly 0.9 MB.

D. Join the pieces — `rle`

Copy the segments into their correct order, into another roughly 0.9 MB buffer.

E. Expand repeated-byte runs — final output

RLE1 converts the reconstructed sequence into actual decompressed bytes, and the CRC verifies them.

And then, because we're decoding blocks in parallel, the finished block must be handed to a reader that delivers blocks in their original file order.

There's quite a lot happening between compressed input and final output.

Notice that we have several substantial arrays representing different stages of essentially the same data.

That alone is worth questioning.

## 2. Why is the T-vector necessary?

Here's a subtlety.

The Burrows–Wheeler Transform doesn't merely shuffle bytes randomly. It arranges them in a particular order that allows the original sequence to be recovered.

The decoder can reconstruct that sequence by starting at a particular row and repeatedly asking:

> Which row comes next?

Think of a map where each location tells you the next location to visit.

For a tiny, hypothetical navigation table:

In the real decoder, every entry also supplies the byte to reconstruct.

The catch is that the next lookup's address depends on the previous lookup's answer.

You can't fetch the tenth entry until you've discovered the ninth. That is why ordinary sequential traversal spends so much time waiting on memory.

`mibz2`'s clever solution is to break the reconstruction into many independent pieces and follow 32 of those chains simultaneously.

While one lookup waits for data, the CPU can advance another.

So far, very clever.

But there's a cost to that cleverness.

## 3. Why do we have 512 separate pieces?

The 32 lanes aren't walking consecutive stretches of the final output.

Instead, the decoder selects numerous starting positions in the navigation structure, follows their links and builds temporary pieces.

Those pieces are discovered in an order that's convenient for parallel traversal, not necessarily the order in which they'll appear in the original data.

Afterward, the decoder follows the recorded segment connections and copies the pieces into the right sequence.

This is an important architectural trade:

We made the memory lookups faster by allowing the output to be reconstructed out of order. Then we introduced an extra assembly stage to restore order.

The code makes this very explicit. In `SegScratch::assemble`, it essentially does:

```
let mut seg = self.first;

while out.len() < n {
    let piece = &self.bufs[seg];
    out.extend_from_slice(piece);
    seg = self.next[seg];
}
```

This is simplified, but captures the actual operation. The decoder already knows which segment follows which; assembly is a copy along that chain. See [the implementation](https://github.com/lmmx/mibz2/blob/master/src/bwt.rs?utm_source=chatgpt.com).

### Here's our first opportunity to question the design

Do we really need the fully assembled intermediate byte array?

Immediately afterward, RLE1 reads that array and produces yet another array containing the final output.

Could RLE1 instead follow the segments directly?

Conceptually:

Today

512 segments

Copy to `rle`

RLE1 decode

Final output

Proposed R2

512 segments

RLE1 consumes the segments in logical order

Final output

One intermediate array and its assembly copy disappear

The difficulty is that run-length encoding can span two segments.

Suppose one segment ends in `AAAA` and the next starts with the byte representing how many additional `A`s to emit. The decoder must carry state across that boundary.

That's manageable conceptually, but the existing RLE1 implementation uses AVX2 to scan 32 contiguous bytes at a time. We would need to preserve most of that speed rather than accidentally create a slower decoder with dozens of tiny operations.

And here's the catch: the measured assembly stage costs only around 0.147 CPU-s/GB at four workers, about 4% of total CPU time in that diagnostic. We cannot simply claim a giant gain from deleting it.

We might gain more indirectly by reducing pressure on the memory system, but that remains to be established.

This is Claude's R2, and an earlier version of the idea was already considered under P85.

## 4. The second opportunity: why construct the giant T-vector at all?

Now let's return to the 3.6 MB table.

It provides extraordinarily cheap navigation. Each row has a four-byte entry. The CPU loads it and obtains both the next position and the corresponding byte.

But the original succinct-data-structure research identified a fascinating property: a large fraction of those links follow simple consecutive patterns.

For some stretches, instead of storing:

```
position 100 -> 581
position 101 -> 582
position 102 -> 583
position 103 -> 584
```

we could represent the entire stretch as something equivalent to:

```
For positions 100–103:
    next = position + 481
```

That's an illustrative pattern, but the underlying redundancy is real.

The census found that roughly 77% of rows in the measured full blocks belong to simple consecutive groups.

This gives us a second trade:

Do we spend memory storing an answer for every row, or spend instructions reconstructing answers from a smaller description?

And that's where the previous compact-table experiments struggled.

A 24-bit link representation reduced storage but made getting the reconstructed byte more expensive. Although it improved cache residency, the extra decoding operations outweighed that advantage.

So the opportunity isn't merely to compress the table.

It's to invent a representation where the CPU can cheaply calculate both the next position and the required byte, without introducing more dependent memory lookups.

The fixed-record idea P40 attempts that with cache-line-sized groups. It remains untested as a complete implementation.

This is the research direction I am especially reluctant to abandon.

## 5. The third opportunity: four workers versus two teams

Now we get to the multicore architecture.

Currently, with four workers, each core can work on a different compressed block.

Four independent workers

CORE

## A

Own table

CORE

## B

Own table

CORE

## C

Own table

CORE

## D

Own table

Two teams of two cores

Block A

One shared T-vector

Block B

One shared T-vector

Sharing a table reduces the number of large independent working sets. And the previous cooperative experiment found something important.

Two teams of two cores used about 13% less CPU time than four independent workers in the historical comparison.

But they produced only 891 MB/s versus 1,007 MB/s.

Why?

Because some decoding stages, particularly move-to-front decoding, run on only one core within each team. The second core doesn't have useful work to perform during that stage.

We're saving CPU but leaving resources idle.

Claude's R1 proposes keeping that second core busy preparing another block while the first block goes through inversion.

That sounds natural. But a team needs both cores during cooperative inversion, so we're not magically creating a spare core. We have to schedule different phases so that one block's serial work occupies otherwise unused core time.

And we must consider whether preparing the next block adds another large live buffer that undermines the cache benefit.

There's a genuine scheduling puzzle here.

## 6. The biggest mystery

Here's what I find most thought-provoking about the measurements:

What changes when we move from one to four workers?

DRAM reads per output byte

# 0.38 → 9.01

Approximately 24×

Average live T-vectors

# 2.19

Not four at all times

We are moving vastly more data through main memory with four workers.

But the average number of simultaneously live T-vectors is only about 2.19. That's approximately 7.9 MB of table storage, against 20 MB of nominal L3 cache.

So the simple story that four huge tables don't fit in cache is incomplete.

It could be that the extra intermediate buffers, cache replacement, temporary peaks in concurrency, dirty lines, prefetch activity or request contention cause the problem.

And the data movement is striking: we write the rearranged bytes, construct a much larger navigation table, write temporary segments, copy them into `rle`, then read them again to generate the final bytes.

Some of those movements are fundamental. Some are just implementation choices.

That distinction is where I think our largest opportunities might hide.

## Let me hand the architectural question to you

Imagine you could redesign the inside of `mibz2` while keeping the bzip2 file format completely unchanged.

You have three broad directions available:

| Direction            | The question I'd challenge                                                                                         |
| -------------------- | ------------------------------------------------------------------------------------------------------------------ |
| Represent less       | Can the original sequence be reconstructed without ever materializing all 900,000 four-byte navigation entries?    |
| Move less            | Can we avoid constructing temporary byte sequences only to copy and immediately consume them?                      |
| Schedule differently | Can the four cores collaborate without either competing over too much memory or sitting idle during serial phases? |

I'm particularly interested in the boundary between traversal and output.

We've designed an extremely clever way to reconstruct 512 pieces simultaneously. But then we largely stop exploiting that structure: we copy the pieces back into a conventional contiguous array and pass that array to another decoder.

My first question to you is this: when you look at that pipeline, does anything strike you as unnecessarily materialized, duplicated or serialized?

It doesn't have to be a complete optimization proposal. Even an intuition like “why can't we calculate that destination earlier?” or “why is that table built in full?” would be worth exploring.


I said:

> Okay, so I'm just going to put on audio recording. Right, so the first thing that came into my head was whether we should just simply be instrumenting better if we've kind of not used instrumenting properly. We're kind of making ourselves blind to the causal structure of the problem. The second is that maybe if you went to fewer than 32 lanes, we would remove the contention. But then I think we already benchmarked that as not true. Like unless we're missing something The next is I guess we should surely be looking at other people who've tried to do the same, like experts who've tried to get the like ratio shown by Little's Law to more like the ratio of you know, the lanes. And like both how they did it and what they used to get it, like how they overcome this limit and what tools they used. And also what like the state of the art in this is. And if maybe we could even say like what it is that we're missing to be able to instrument it better, presuming that we are missing something And yeah, we've said this thing about several possible explanations. But I don't think I've seen like simply approaches to measure them I think also that we should maybe be thinking in terms of like ideal performance and how to demonstrate ideal performance in the smallest scale possible. And then like scale that up. And maybe that would give us like an easy testing ground. That's just a thought I had And yeah, in general we want to like narrow the hypothesis space towards something. I think what we might have been doing wrong is sort of throwing out a load of ideas and then just kind of crossing them off. And that leaves us kind of back to square one rather than like progressively getting closer to something

it said:

I think the most useful shift is from proposing more optimizations to identifying which hardware limit the walker is actually hitting. I’ll check the established pointer-chasing research and profiling methods, then turn that into a small set of experiments where each result narrows the explanation rather than just eliminating another idea.

I think you've identified a more productive way to approach this.

The problem isn't necessarily that we've run out of optimization ideas. It's that we haven't yet built a sufficiently discriminating explanation of why the current walker takes 4.49 cycles per byte. Without that, one failed experiment tells us only that a particular implementation didn't work, not what the processor is capable of.

And there is a particularly relevant discovery in your own code: `src/bwt.rs` explicitly cites Kärkkäinen, Kempa and Puglisi's Slashing the Time for BWT Inversion (2012). It describes exactly the idea you're asking about—interleaving multiple BWT traversals—and mentions two further techniques, a larger effective alphabet and Precopy. So there is a direct research lineage here, not merely generic advice about CPU optimization.

I would make measurement and an idealized walker the next piece of research, before changing the production algorithm.

## 1. The 8.6 figure may not indicate a problem at all

This is the first thing I'd test.

The ratio \\(38.55 / 4.49 \approx 8.6\\) is an observed speedup over a single chain. It is not a measurement of how many requests are outstanding in the processor.

There may be a perfectly good explanation for 32 software lanes producing an 8.6× speedup: the hardware may have enough resources to handle only roughly 8–12 outstanding cache misses of the relevant kind at once.

That isn't a hypothetical limitation invented to explain these results. In Asynchronous Memory Access Chaining (AMAC), Kocberber, Falsafi and Grot found that performance on their Xeon stopped improving beyond about ten in-flight lookups because the core had reached its limit of ten outstanding L1 cache misses. They also found that excessively high concurrency could cause TLB problems.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://homepages.inf.ed.ac.uk\&sz=32)

homepages.inf.ed.ac.uk



The specific limit needs measuring on your i9-10900K; we shouldn't transfer a Xeon result directly.

But consider a deliberately crude model:

\\[ T\_{\mathrm{memory}} \gtrsim \frac{\text{single-chain latency}}{\text{available miss concurrency}} \\]

If the hardware can overlap ten relevant misses:

\\[ 38.55 / 10 \approx 3.86\text{ cycles/byte} \\]

Against 4.49 cycles/byte, that would suggest the walker is already fairly close to its memory-parallelism ceiling.

That calculation is not a valid bound yet: those historical timings need to be made comparable, and your explicit prefetching changes the relationship between observed single-chain latency and demand-load latency. But it illustrates why the current 8.6 figure could signify success rather than failure.

It also changes the research question from “Where did the other 23 lanes go?” to:

“Is the 4.49 c/B walker limited by hardware concurrency, instruction throughput, or unnecessary work?”

Those are measurable alternatives.

## 2. The experts have already developed methods to separate them

There are three particularly useful precedents.

Slashing the Time for BWT Inversion — 2012

This is the directly relevant algorithmic work your own `src/bwt.rs` cites. The reported 2.3–4× BWT inversion gains came from both reducing the number of cache misses and reducing their effective cost by overlapping traversals.

The distinction matters: if concurrency is already near its ceiling, the next improvement may require fewer dependent cache misses, not more lanes.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.researchgate.net\&sz=32)

Request PDF



[Asynchronous Memory Access Chaining](https://homepages.inf.ed.ac.uk/bgrot/pubs/AMAC_VLDB16.pdf?utm_source=chatgpt.com) — 2015

They compared group prefetching, software-pipelined prefetching and asynchronous state machines. Crucially, they measured where additional lookups stopped helping, examined instruction overhead and used counters to establish hardware resource saturation. Their method is relevant even if AMAC itself isn't the best implementation for your BWT walk.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.research.ed.ac.uk\&sz=32)

University of Edinburgh Research Explorer



Understanding and Improving Graph Algorithm Performance — 2016

Scott Beamer used a minimal parallel pointer-chasing benchmark, varied the number of independent streams, and deliberately inserted instructions between loads to discover the limits imposed by the reorder/instruction window. His model distinguished parallelism available in the program from parallelism the machine actually achieves. That is almost exactly the experiment you are proposing.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://escholarship.org\&sz=32)

escholarship.org



In other words, there is a fairly mature experimental method here. We don't have to invent one.

## 3. We can measure much more than we have

A significant detail from your own compute-frontier journal: the original measurements explicitly say hardware counters were unavailable on that VM. They therefore used timings, cache-size sweeps and controlled pointer-chasing tests.

Those were useful experiments, but they couldn't directly measure the processor's execution bottlenecks.

On a physical Linux host with access to the CPU performance monitoring unit, I'd start with `perf`, then use Intel's top-down analysis through `toplev` or VTune.

Intel even defines an MLP metric:

\\[ \mathrm{MLP}\_{\mathrm{demand}} = \frac{\texttt{L1D\\\_PEND\\\_MISS.PENDING}} {\texttt{L1D\\\_PEND\\\_MISS.PENDING\\\_CYCLES}} \\]

That measures the average number of outstanding L1-miss demand requests during cycles when at least one exists. It is fundamentally different from our inferred 8.6× speedup.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://gbmc.googlesource.com\&sz=32)

Git at Google



On the i9 machine, check event availability first:

```
perf list | grep -Ei \
  'l1d_pend_miss|cycle_activity|resource_stalls|dtlb|uops'
```

Then, assuming the events are supported:

```
perf stat -r 5 \
  -e cycles,instructions,l1d_pend_miss.pending,l1d_pend_miss.pending_cycles \
  -- taskset -c <physical-core> <walker-benchmark>
```

I'd use separate small counter groups for the remaining events rather than allowing uncontrolled multiplexing.

Here's what we want to establish:

| Competing explanation                       | What would distinguish it                                                                                                                           |
| ------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| Hardware miss capacity exhausted            | Outstanding-miss count plateaus as lane count rises, with fill-buffer-full stalls or other capacity evidence                                        |
| Instruction window cannot expose more loads | Low achieved MLP despite available miss capacity; relevant reorder-buffer/resource pressure; strong response to reducing instructions between loads |
| Execution throughput is the limit           | High retired µops per byte, relatively little load-stall time, and a minimal walker substantially faster than the real one                          |
| TLB or cache behavior is worsening          | Page-walk or cache-miss rates change with lane count and working-set size                                                                           |
| Prefetching already hides the latency       | Prefetch-on sharply reduces demand-load stalls and changes the MLP counter even though throughput improves                                          |

The last distinction is important: a low demand-load MLP counter does not mean low overall overlap if your software prefetches have already brought the lines in.

Intel's [VTune Memory Access analysis](https://www.intel.com/content/www/us/en/docs/vtune-profiler/user-guide/2023-1/memory-access-analysis.html?utm_source=chatgpt.com) provides latency, cache-level and memory-bound breakdowns. Andi Kleen's [pmu-tools](https://github.com/andikleen/pmu-tools?utm_source=chatgpt.com) supplies the top-down analysis for finding front-end, back-end and execution bottlenecks.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.intel.com\&sz=32)

intel.com+1



I'd regard getting these measurements as a prerequisite for the next ambitious machine-code rewrite.

## 4. Your minimal ideal-performance experiment is particularly valuable

I would build an experimental ladder around one fixed, real 3.6 MB BWT T-vector extracted from a Wikidata block. That preserves the actual locality and pointer structure, rather than assuming uniformly random accesses represent the real workload.

The experimental ladder

Every step preserves the same essential pointer-chase workload while introducing one more source of cost.

For each level, sweep 1, 2, 4, 8, 12, 16, 24, 32, 48 and 64 lanes. Record cycles per link, instructions and µops per link, achieved miss parallelism, and relevant cache/TLB events.

This would give us a much more informative result than another 32-versus-24 production benchmark.

For example, suppose the bare walker reaches 3.8 c/B, while the full walker takes 4.49. That suggests only about 0.7 c/B is available from eliminating bookkeeping, under the test's assumptions.

But suppose the bare walker achieves 1.6 c/B while the real one stays at 4.49. Now we have evidence of a major software inefficiency and a quantitative target.

We could then pinpoint where the gap first appears: prefetch placement, output stores, boundary handling or lane-state management.

One caveat: these differences will not necessarily add linearly, because the extra instructions may overlap with memory stalls. But the ladder still localizes interactions.

## 5. The experiment that makes this genuinely cumulative

I'd make the result a simple branching decision:

1\. If even the bare walker saturates at roughly the same speed and lane count, stop treating unexploited lanes as the principal problem. Investigate reducing cache misses or pointer dependencies—for instance, the BWT-specific methods in the 2012 paper.

2\. If the bare walker is much faster and the gap arises when bookkeeping is added, inspect the exact assembly and counters for that transition. That's where register placement, spills, branch execution or instruction-window pressure become concrete targets. Only then does something like four groups of eight lanes become a well-motivated hypothesis.

3\. If the gap appears specifically when prefetching is changed, study prefetch timing and whether the next demand load encounters the line in L1, a fill buffer or a deeper cache. That's a latency-hiding problem with its own experimental tools.

4\. If the behavior depends strongly on T-vector footprint or data distribution, investigate locality and translation separately from instruction scheduling. A synthetic random permutation and real BWT links should not automatically be treated as equivalent.

Notice how different this is from trying four implementations and discarding three. Every result tells us which family of explanations remains plausible.

## 6. What I think has been missing

Not primarily ideas, and not necessarily instrumentation code inside the decoder.

What's missing is a calibrated reference for what the hardware can achieve on this precise workload, alongside direct measurements of why the real walker falls short of that reference.

I'd also be careful with the existing measurements because the single-chain 38.55 c/B and the 32-lane 4.49 c/B are from different historical measurement contexts. Before building theoretical conclusions from their ratio, we should remeasure both on the same hardware, pinned revision and benchmark.

The question I would now prioritize is:

Can a minimal 32-chain walker on the i9-10900K substantially beat 4.49 c/B on the same real BWT table?

That single experiment, combined with performance counters, could settle whether we're searching for a better way to schedule the same work or whether we need an algorithm that does less dependent work.

And that feels like exactly the progressive narrowing you're describing.
`````
