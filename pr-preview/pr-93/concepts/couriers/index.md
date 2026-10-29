# Why 32 couriers beat one: understanding memory-level parallelism

Most of a bzip2 decode is one loop, the inverse Burrows–Wheeler transform's walk, and that loop spends most of its time waiting for memory. mibz2 doesn't make memory any faster. It keeps many of the walk's memory requests in flight at once, so the waits overlap. That is **memory-level parallelism** (MLP), and it is the library's basic premise.

This page explains it with couriers and a dispatcher, then gives the mechanism behind each part of the picture, and the measurements so far. [How it works](<https://lmmx.github.io/mibz2/pr-preview/pr-93/concepts/how-it-works/index.md>) has the threads around the walk, and [Prior work](<https://lmmx.github.io/mibz2/pr-preview/pr-93/concepts/prior-work/index.md>) the work it builds on.

## One courier, one chain

A motorcycle courier learns each delivery address only on arriving at the previous one: open the envelope, read the next address. The third destination can't be known before reaching the second. The bike is fast, but most of the time goes on waiting to find out where to go next.

That is the ordinary inverse BWT:

- **The table.** After Huffman decoding and move-to-front, a block's last column becomes the T-vector: one 32-bit entry a row, holding the row's byte (bits 0–7) and the next row (the bits above). A block holds up to 900,000 bytes, so the table is up to 3.6 MB, larger than most cores' L2 cache.
- **The walk.** Start at the block's primary index. Each step loads the current row's entry, writes its byte, and takes the next row from the same entry. The next row is anywhere in the table, so the load usually misses the cache.
- **The wait.** The step after can't start until that load returns, because its address is in the load's result. A chain of dependent loads keeps one miss in flight, and the core idles through each one.

One courier. Each step (dark) is a few instructions; most of the time goes on waiting for the next address (light). Schematic, not to scale.

The walk was measured as latency-bound directly, on the machine where mibz2's prototype was built: walking 1, 2, 4, 8 and 16 independent chains through one block, the cost fell from 28.5 to 4.4 cycles a byte of output ([compute frontier, section 2](<https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-27-compute-frontier.md#2-the-bz2-decode-phases-scaling-and-the-prototype>)). Memory bandwidth was nowhere near its limit. The walk was waiting, not moving data. libbzip2 and lbzip2 walk the one chain.

## 32 couriers, 32 independent journeys

Give each of 32 couriers a different delivery chain. While courier 1 waits for its next address, couriers 2, 3, 4… make progress. That is memory-level parallelism: the couriers don't travel faster, but there are enough independent journeys that waiting on one doesn't stop the rest. One courier is limited by the time it takes to get each new address; 32 keep many requests in flight.

Four of mibz2's 32 couriers. Each still waits as long for its address, but the waits overlap, and the core takes the other couriers' steps in the gaps. Schematic, not to scale.

> [!NOTE]
>
> **The correction that keeps the picture faithful**
>
> The CPU doesn't employ 32 workers, and the lanes aren't threads. **One core interleaves the instructions of 32 traversal chains**, and its out-of-order machinery overlaps their memory requests. The better image is one dispatch centre managing 32 motorcycles, each getting its next destination only when its last delivery returns an address.
>
> What counts is deliveries a second for the whole centre, not how fast one bike finishes its route: sustained throughput, not the latency of any one load. Every load still takes as long as before.
>
> Several threads are several dispatch centres, one a block: mibz2's workers decode whole blocks in parallel ([How it works](<https://lmmx.github.io/mibz2/pr-preview/pr-93/concepts/how-it-works/#threads>)). The lanes are the parallelism inside each one.

### Where the independent journeys come from

A block has one chain, not 32: the T-vector is a single cycle through every row (or, for a periodic text, a few). bzip2 carries no metadata to start a walk anywhere but the primary index. mibz2 makes the journeys itself:

- **Random starts.** 511 pseudo-random rows, plus the walk's own first row, become segment starts. A random row is a random position in the text, so the cycle is cut into 512 segments of about equal length. (Evenly spaced rows clustered in the text and stalled the lanes.)
- **Flagged starts end a segment.** Each start is flagged in bit 31 of its own T-vector entry, which links never use. A lane that reads a flagged entry has run into the next segment's start: its segment ends there, recording which start it ran into, and the lane takes the next unwalked segment.
- **32 lanes walk in lockstep.** Each round, every lane takes one step, writing its byte into its own segment's buffer.
- **The segments are stitched together afterwards.** From the first row's segment, each segment is followed by the one it ran into, until the block's length is reached. The output is exactly the plain walk's, byte for byte, including a periodic block's several cycles. The flags are cleared before the walk returns.
- **Short blocks walk plainly.** Under 64 KiB, the segments' bookkeeping costs more than the waits it hides; only a stream's last block is ever that short.

One chain cut at flagged starts into segments, walked by four lanes in whatever order the lanes take them, then joined in order. mibz2 cuts each block into 512 segments for 32 lanes.

### Prefetching: phoning ahead

Lanes alone weren't enough. Without help, a lane's next load is issued only when its instructions enter the core's reorder buffer, which is full of the other lanes' steps. So the misses in flight were bounded by how many steps that window holds, and adding lanes didn't add misses.

mibz2 issues a software prefetch of each lane's next entry as soon as its row is known (`prefetcht0` on x86-64, `prfm pldl1keep` on aarch64). A prefetch retires without waiting for its data, so every lane's miss starts at once. In the picture, the dispatcher phones each request through the moment an address is read, rather than when that courier's turn comes round on the board.

Try the counts yourself in this schematic. It has one dispatcher, whose paperwork for a step takes a tick or two, and each address takes 31 ticks to come back. The timings are made up for the picture.

The schematic needs JavaScript.

## The dispatcher becomes the bottleneck

There is only so much road, and one dispatcher. Add enough couriers and every wait is covered. Then the limit is the dispatcher: each step's paperwork, done for each courier in turn. Add more couriers past that point and the dispatcher spends more on coordination than the extra journeys earn.

In the core, the dispatcher's limits are these:

- **Instructions a step.** Every lane's step is a handful of instructions: the load, the flag test, the byte's store, the write pointer's advance, the buffer-end check, the prefetch. With the misses overlapped, the walk runs at the rate the core can issue those instructions.
- **Register pressure.** One lane's state (its row, its write pointers, its segment) fits in registers, but 32 lanes' state doesn't. It lives in the L1 cache, and every step loads and stores some of it.
- **Cache contention.** The lanes' table lines, the 512 segments' buffers and the lanes' own state share the same caches. At four threads, four walks share the L3.
- **Outstanding misses.** A core tracks only so many misses at once (its line fill buffers, and the L2's own queue). Beyond that, more lanes only queue.

The record shows both sides. Without the prefetch, 16 lanes were within noise of 12, and 24 or 32 were slower ([PR \#21](<https://github.com/lmmx/mibz2/pull/21>)). With it, 16 to 32 lanes were about 26–28% faster than 12 lanes without it, and within noise of each other at one walk. With four walks at once, 32 lanes won clearly. So mibz2 runs 32 lanes and 512 segments ([the measured points](<#lane-counts>)).

It also explains why a cleverer-looking algorithm can lose. Once 32 misses are in flight, **the walk is bound by instructions a step**, not by the latency of each load. A change that saves steps but makes each one dearer loses. A change that keeps the steps and makes them cheaper wins.

## Make the dispatcher efficient

Once there are enough couriers, cut the paperwork a step. That's how the walk got faster:

- **The flagged walk** ([\#12](<https://github.com/lmmx/mibz2/pull/12>)). It replaced the first segmented walk, which tested every step's row against a separate bitmap of the starts (12 lanes, 256 segments). The flag moved into the entry the step loads anyway, bytes went out through raw per-lane pointers instead of a growable vector's push, and the test for idle lanes left the hot loop.
- **The prefetch, with 32 lanes and 512 segments** ([\#21](<https://github.com/lmmx/mibz2/pull/21>)): every lane's next miss starts at once, as [above](<#prefetch>).
- **The shared-offset walk, after libsais's chain decode** ([\#37](<https://github.com/lmmx/mibz2/pull/37>), the default walk since 0.1.5). libsais 2.12's `decode_chains` runs `room` rounds of every chain at a shared output offset, where `room` is the fewest bytes any chain's buffer has spare. Within a pass, a lane's byte goes to its buffer's base plus the pass's offset, so the per-step pointer store and buffer-end test are gone. The lanes, the prefetch and the segments are unchanged, so the same misses are in flight. Only the instructions a step drop: one dispatcher's paperwork, done in batches.

Two cleverer-looking ideas from the literature were built, measured and dropped. Both changed the journeys themselves, and both lost to their extra cost a step:

- **The pair walk, ψ²** ([\#34](<https://github.com/lmmx/mibz2/pull/34>), after libsais and Kärkkäinen, Kempa and Puglisi's super-alphabet). Each entry holds the row two steps on, so the walk takes half the steps, two bytes at each. But each step costs about twice as much: the second byte comes from a first-byte lookup. With the prefetch keeping 32 misses in flight, halving the dependent loads bought almost nothing, and the pair table took about three times as long to build. Envelopes holding two addresses mean fewer stops, but twice the paperwork at each, and the envelopes take three times as long to fill.
- **Precopy** ([\#36](<https://github.com/lmmx/mibz2/pull/36>), after Kärkkäinen and Puglisi). Where a stretch of the walk repeats one already walked, its bytes are copied instead of walked. It copied 31.5% of the Wikidata blocks' bytes, and was slower all the same. The repeat check ran on every step, and the steps it skipped were the cheap ones, whose table lines were already in cache. The cost is in the scattered steps, which repeats never cover. A courier who checks the logbook for a repeated stretch at every stop loses more time than the short, nearby stretches it saves.

The core's time once enough misses are in flight, schematically: the bytes out in the same time. The shared-offset walk keeps the journeys and cuts the bookkeeping. The pair walk halves the steps but doubles each one's work, and pays again in its build. Precopy adds a check to every step to skip a few cheap ones.

## The measurements

> [!WARNING]
>
> **Read these as indicative**
>
> These are the walk alone, on one thread, mostly on shared development machines. Each comparison holds only within its own row: the rows differ in machine, clock, corpus and repeats. The implementations differ in more than their lane count, too: the bitmap or the flag, the vector push or the raw pointers, the prefetch, and the segment count. Only the CI benchmark decides what merges.

### The walk alone

Cycles a byte of output (c/B), the walk on its own, one thread:

| change | before → after | where measured | source |
| --- | ---: | --- | --- |
| the plain walk → the first segmented walk (12 lanes, a bitmap of the starts) | 31.4 → 14.1 | the prototype, `kernels/bz2k`: the wikidata-pq bench slice, a Xeon VM at 2.1 GHz | [compute frontier, section 2](<https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-27-compute-frontier.md#2-the-bz2-decode-phases-scaling-and-the-prototype>) |
| the bitmap walk → the flagged walk (both 12 lanes) | 31.8 → 17.2 | `examples/phases`: 13 MB of Wikidata dump blocks, a shared VM, at a nominal 2.8 GHz | [PR \#12](<https://github.com/lmmx/mibz2/pull/12>) |
| the flagged walk at 12 lanes → 32 lanes, 512 segments and the prefetch | 21.20 → 12.28 (four walks at once: 50.99 → 20.35) | the walk alone: the 431 Wikidata blocks, best of 5, a shared VM, at a nominal 2.8 GHz | [PR \#21](<https://github.com/lmmx/mibz2/pull/21>) |
| the flagged walk (32 lanes, the prefetch), in a whole decode's stage split | 11.90, 53.4% of a 22.31 c/B decode | `examples/phases`: the 431 Wikidata blocks, best of 3, at a nominal 2.8 GHz | [research synthesis](<https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-27-research-synthesis.md#the-bottleneck-the-leads-are-measured-against>) |
| the flagged walk on CI's runner | 6.96 and 7.33 (medians: the base and head of a change that left the walk alone), 60.3% of a decode | CI's per-stage table: `examples/phases` on the Wikidata blocks, 5 rounds, an EPYC 9V45 at 4.5 GHz | [PR \#35's bench](<https://github.com/lmmx/mibz2/pull/35#issuecomment-6448968226>) |
| the flagged walk → the shared-offset walk | 8.30 → 7.37 | the walk alone: Wikidata blocks, best of 5 a block, the shared machine | [PR \#37](<https://github.com/lmmx/mibz2/pull/37>) |
| unjoined chains, 1 → 16 (no segments, no join: a ceiling, not a decoder) | 28.5 → 4.4 | the prototype's latency test, as the first row | [compute frontier, section 2](<https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-27-compute-frontier.md#2-the-bz2-decode-phases-scaling-and-the-prototype>) |

The losing ideas, the walk alone on the shared machine: the pair walk's walk was about 10 c/B against the flagged walk's 10.4–10.8, but its build was 7.1–7.6 against 2.5 ([pair walk journal](<https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-27-pair-walk.md>)); Precopy was 13.5 against 10.3 c/B on Wikidata, with 31.5% of the bytes copied ([PR \#36](<https://github.com/lmmx/mibz2/pull/36>)).

### By lane count: the measured points

Only measured points are drawn, each chart on its own axes and naming its source: two sweeps so far, on different machines, corpora and walks, not to be read against each other. A finer sweep of lanes and prefetching is planned. Its points will go into [`assets/data/lane-sweeps.json`](<https://github.com/lmmx/mibz2/blob/master/docs/site/assets/data/lane-sweeps.json>), one panel a sweep, and appear here.

Loading the measured sweeps…

### The whole decoder, on CI

The walk is about 53–60% of a decode, so a walk change moves the whole decoder by less. Whole-decoder figures come only from the CI benchmark: paired rounds of a pull request's head against its base on one runner ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-93/benchmarks/index.md>)). The walk's changes, on the Wikidata blocks (wall time, head over base, with its 95% interval; − is faster):

| change | 1 thread | 4 threads | bench |
| --- | ---: | ---: | --- |
| the flagged walk (\#12) | −10.7% (−11.3%, −10.2%) | −16.6% (−18.2%, −15.0%) | [\#12](<https://github.com/lmmx/mibz2/pull/12#issuecomment-6441420139>) |
| 32 lanes, 512 segments, the prefetch (\#21) | −13.2% (−13.8%, −12.5%) | −8.3% (−11.5%, −5.0%) | [\#21](<https://github.com/lmmx/mibz2/pull/21#issuecomment-6443760336>) |
| the pair walk, ψ² (\#34, not merged) | \+40.6% (+40.1%, +41.1%) | \+24.3% (+22.8%, +25.8%) | [\#34](<https://github.com/lmmx/mibz2/pull/34#issuecomment-6448612473>) |
| Precopy (\#36, not merged) | \+18.0% (+15.9%, +20.1%) | \+6.0% (+5.0%, +6.9%) | [\#36](<https://github.com/lmmx/mibz2/pull/36#issuecomment-6449155064>) |
| the shared-offset walk (\#37) | −10.3% (−10.6%, −10.0%) | −9.1% (−12.2%, −5.9%) | [\#37](<https://github.com/lmmx/mibz2/pull/37>), its first run (EPYC 9V74); its last, on a Xeon 6973P-C: −5.8% (−7.3%, −4.2%) and −2.7% (−4.1%, −1.3%) |

Each row is its own runner and its own base, so the rows don't add up. Master's figures over its commits are on the [History](<https://lmmx.github.io/mibz2/pr-preview/pr-93/benchmarks/history/index.md>) page, and mibz2 beside the other bzip2 decoders on [Alternatives](<https://lmmx.github.io/mibz2/pr-preview/pr-93/benchmarks/alternatives/index.md>).

## The picture, term by term

| in the picture | in the decoder |
| --- | --- |
| a courier | a lane: one traversal chain, walking one segment at a time |
| a delivery address, read on arrival | the next row, in the entry the step just loaded |
| waiting for the address | a cache miss on the T-vector |
| a delivery chain | a segment, from a flagged start row to the next start |
| the dispatcher | one core's instruction stream, interleaving all the lanes' steps |
| the paperwork for each step | the instructions a step: the flag test, the byte's store, the pointer and buffer bookkeeping |
| phoning ahead | the software prefetch of each lane's next entry |
| the road, the board's size | the core's limits: outstanding misses (line fill buffers), the reorder window, the caches |
| deliveries a second for the centre | bytes of output a second: throughput, not one load's latency |
| several dispatch centres | worker threads, one block each |
