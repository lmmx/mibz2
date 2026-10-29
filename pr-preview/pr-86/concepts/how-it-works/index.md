# How it works

## Threads

- **The scanner.** One thread reads each block's header and Huffman-coded symbols in order. This is the one sequential phase: a block's end is known only once its symbols are read. It finds blocks exactly, not by searching for the block magic, so a magic inside a block's data is never taken for a block. It is roughly a tenth of a decode on well-compressed text, more on poorly compressible data.
- **The workers.** Each worker finishes a block: move-to-front, the T-vector, the inverse BWT walk, RLE1 and the CRC.
- **The reader** takes the blocks in order. A block's error reaches the reader only once every block before it has been read.
- **Memory is bounded:** at most `slots` blocks are in flight (by default twice the threads), and the buffers are reused from block to block.

With one thread, the same phases run in order on the calling thread.

## The walk

The inverse BWT is a chain of dependent loads: each step's row is the previous step's load. A plain walk keeps one load in flight. The default walk (`bwt::walk_segments_room`, the flagged walk in passes) keeps many, one a lane ([Why 32 couriers beat one](<https://lmmx.github.io/mibz2/pr-preview/pr-86/concepts/couriers/index.md>) explains why, and where it stops paying):

- 512 pseudo-random rows, the walk's first among them, are segment starts;
- each start is flagged in its own T-vector entry, which ends the segment before it;
- 32 lanes walk segments in lockstep, each taking the next segment when its own ends, and each lane's next entry is prefetched as soon as its row is known;
- the lanes step in passes of `room` rounds, `room` the fewest bytes any lane's buffer has spare, each byte written at its lane's base plus the pass's offset, so a step has no pointer store or buffer-end test (after libsais's chain decode, [\#37](<https://github.com/lmmx/mibz2/pull/37>));
- the segments are joined in order.

It equals the plain walk on every list, lists of several cycles included: the join follows the segments' links until the block's length is reached. The flags are cleared before the walk returns. Blocks under 64 Ki are walked plainly. The kernel is behind `bwt::InverseBwt` (`Segmented`, the default; `Plain`), so a walk can be swapped and timed as part of the whole decoder.

## Kernels

`Kernels::detect()` chooses at run time: AVX2 (with BMI1) for move-to-front and RLE1, and PCLMULQDQ for the CRC. Each has a scalar twin, used on any other CPU and on aarch64 (`Kernels::scalar()`). The walk's gain is scalar code, memory-level parallelism, so it holds on any CPU.
