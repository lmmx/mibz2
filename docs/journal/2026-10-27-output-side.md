# The decode's output side: what we believed, what changed it (#45, #47, #48)

Before this round, the working model was that a decode's cost is the T-vector walk's
dependent loads, and that the walk's output was incidental. Several of our beliefs changed
in a few hours. Each is recorded below with:
- what we believed;
- the evidence that changed it;
- where it stands now.

Labels:
- **measured:** a CI figure, or a local one marked indicative;
- **source:** a primary source, cited;
- **hypothesis:** ours, untested.

## 1. Our instruments measured a state no decode reaches

- **Believed:** the stage table (`examples/phases.rs`) and the factorial sweep measured
  the walk as a decode runs it.
- **Changed by (measured):**
  - The sweep's settings at 256, 1,024 and 2,048 segments read 40–75% slower than the
    default at 512. The 56 settings that shared the default's layout had each followed
    walks of the same block in the same layout.
  - Locally, the 4th walk of a block took 3.00 ns/B against 4.05 for the 1st.
  - A rebuild of the T-vector between walks left most of that gain in place; a 64 MB
    scrub removed it.
  - The stage table had the same bias: each stage kept its best of 2 runs on the same
    block, and the walks ran back to back on it.
- **Now:**
  - both instruments run in decode order (#45, #47);
  - in decode order, the walk is about a fifth costlier than the stage table had shown
    (CI: 4.02 → 4.97 c/B on an EPYC 9V74);
  - the stage total explains 90% of the whole decoder's one-thread cycles, against 81%
    before;
  - the warmth lived in the 1.8 MB of segment output buffers, not the T-vector. That was
    the first sign that the output side matters.
- **Lesson:** a benchmark that repeats an operation over retained working memory answers
  a different question from one in decode order. Say which one a figure measures. And
  production's own buffer lifetime is the reference: warm from the previous block, never
  from the same block, not "cold".

## 2. The prefetch curve's "instruction window" signature didn't survive

- **Believed (from the warm sweep):** without prefetch the walk flattens at 16–24 lanes,
  while with prefetch it keeps improving to 32. We read that as the core's instruction
  window running out before the memory system did.
  - **source:** a demand load at the head of the reorder buffer stalls retirement
    ([Mutlu et al., runahead execution, 2003](https://www.cs.cmu.edu/~18742/papers/Mutlu2003.pdf)).
  - **source:** a software prefetch retires without waiting for its data (V. Kiriansky,
    MIT dissertation, §2.4.5; found by the user's search, not read here).
- **Changed by (measured, decode order, one runner):** both curves flatten near 24 lanes.
  Prefetch saves a steady 13% alone and 15–22% with four copies.
- **Now:** the window hypothesis lost its main evidence. It is not refuted, and its
  filler-instruction test is shelved until a curve calls for it.

## 3. The bottleneck that matters is shared, not per-core

- **Believed:** optimising the one-thread walk is optimising the decode.
- **Changed by (measured, decode order):**
  - four copies at once double each copy's walk cost (5.0 → 10.4 c/B);
  - lanes beyond 12–16 then buy nothing;
  - the best setting flips: 16 lanes with the L2 hint is faster with four copies, and 23%
    slower alone.
- **Now:**
  - a multi-threaded decode, which is how mibz2 and wikidata-pq run, lives in the
    contended regime;
  - **hypothesis:** the L3's capacity or the memory bandwidth (four working sets near
    32 MB). Four processes are not four threads, and no counters were available to
    confirm it;
  - the diagnostics were all one-thread, and gain a four-copy view (#48's plan).

## 4. A smaller permutation table loses under memory-level parallelism (#42)

- **Believed:** a 0.6 MB run-based move structure in place of the 3.6 MB T-vector would
  change the memory problem.
- **Changed by (measured, local):**
  - walked as one chain, it is 60–63% faster;
  - under the 32-lane room walk, it is 90–108% slower, build included;
  - the room walk already hides the T-vector's misses, and the move step costs about
    41 instructions against 16.
- **Now:**
  - closed on its criterion;
  - reopens only under the condition its closing comment named: four-copy contention
    moving the T-vector walk towards memory-bound. Item 3 is that condition, measured;
    the pre-registered test (≥10% under build + room walk per copy, with four copies)
    is queued.

## 5. Non-temporal stores have no boundary on the decode's path

- **Proposed (source):** Intel® 64 and IA-32 Architectures Optimization Reference Manual,
  Vol. 1, §9.6.1, "Non-Temporal Stores and Software Write-Combining". Streaming stores for
  data written once, or large enough to thrash the cache, avoid write allocation and dirty
  writebacks.
- **Changed by (the code):** tracing every copy on the decode's output path (#48's
  table). Every buffer mibz2 writes is read again almost at once:
  - the walk's segments, by the assembly;
  - the assembled block, by RLE1;
  - RLE1's output, by the CRC;
  - the reader's copy, by the caller.
  The one write-once buffer belongs to the convenience API `decode_parallel`.
- **Now:**
  - non-temporal stores are last on #48's ladder, at a genuinely streaming boundary only;
  - removing a copy beats changing how it stores, since it saves the reads as well;
  - "written twice" counts stores, not DRAM traffic.

## 6. Removing the assembly copy doesn't mean fragmented, per-byte RLE1

- **Believed:** the only alternative to assembling the block's segments into one
  contiguous buffer before RLE1 was an RLE1 that follows the segment chain byte by byte,
  with a boundary check or iterator call on every byte. That is costly now that
  instructions per byte are what we count.
- **Changed by (source; the user's search):**
  - Intel IPP's `ippsDecodeRLEState_BZ2_8u` decodes bzip2's RLE from a source slice into
    a destination with a persistent state;
  - Go's `compress/bzip2` carries RLE1's repeat state across reads.
  RLE1 needs an ordered stream of bytes and a continuous state, not one contiguous
  buffer.
- **Now (hypothesis, #48 step 2):** RLE1 takes about 512 contiguous slices a block in
  successor order, with its state carried across boundaries.
  - Contiguity becomes local to each segment (about 1,750 bytes on average) instead of
    global to the block.
  - The tight inner loop survives, and boundaries cost a ≤9-byte scalar stitch about 512
    times a block.
  - The state is the unparsed tail from the last parse point: RLE1's parse is positional,
    so a 5th equal byte is a count, not a run.
  - These are precedents for the interface, not evidence of a speedup.
  - Gated on the assembly's measured share (locally 4.3%, against the 5% gate) and the
    paired whole-decoder A/B at 1 and 4 threads.
  - The interactions to watch: the copy may cost more than removing it saves, because
    assembly also hands RLE1 a contiguous stream; or removing it may save more at
    4 threads than the timer shows, if output traffic is part of the contention.
- **Lesson:** the choice is not between one contiguous buffer and per-byte fragmented
  access. A chain of contiguous slices keeps a tight local loop while removing the
  materialisation.

## Open, in order

1. CI's decode-order assembly share, and its check row: walk and assembly timed as one
   against the two rows summed (#48 step 1).
2. Only if the share is material: slice-at-a-time RLE1 (#48 step 2).
3. #42's pre-registered contention test.
4. A four-copy stage table beside the one-copy one.
