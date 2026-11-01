# Engineering lesson: a reused buffer needs one capacity contract

**The defect it comes from:** the P74-C decoder trial's RSS NEGATIVE.
- The evidence:
  - `2026-11-01-p74c-decoder-i9-10900K/postmortem.md` §1;
  - `2026-10-31-p74c-decoder-rss-note.md`.
- **Status:** P74-C is archived, so nothing is fixed here. This records the general
  lesson.

## What happened

- **The buffer:** a decoder worker keeps one `ll: Vec<u8>` (the MTF output, up to the
  block size `max` = 900,000 bytes) and reuses it across blocks.
- **The two kernels that fill it reserve differently:**
  - the shipped AVX2 kernel calls `ll.reserve(max)` (`src/mtf.rs:86`);
  - P74-C calls `ll.reserve(max + 32)` (`src/mtf/p74_c.rs:119`), because its short-run
    vector store may write up to 31 bytes past `n`.
- **A per-block selector chooses between them.** When a worker's previous block went to
  the shipped kernel (capacity exactly `max`) and the next goes to P74-C:
  1. The reserve is 32 bytes short.
  2. `Vec` grows amortised to 2 × `max`.
  3. A `realloc` that can't extend in place touches a fresh 1.8 MB region and frees the
     old one.
  4. Peak RSS rises by about one block buffer.
- **The measured signature:**
  - +876 kB median, in 19 of 20 rounds, on a 3-block corpus at 2 workers;
  - absent where no worker decodes two blocks;
  - visible in minor faults on every multi-block corpus that interleaves the kernels.
- **Each kernel is correct on its own,** and both pass every equivalence test. The
  defect exists only in the alternation.

## The general rule

When several implementations share a reused buffer, its capacity requirement is one
contract, owned in one place, and every implementation reserves through it.

## Practices

1. **Name the slack.** Use one constant or function, such as `ll_capacity(max)`, that
   every kernel reserves through. That includes kernels that don't need the slack, so the
   kernel with the widest overrun sets it for all of them.
   - The alternative is to make the overrunning kernel stay within the common capacity:
     handle its tail scalar, or store masked.
2. **Test the alternation, not just each kernel.** Run a sequence such as bypass → P74 →
   bypass → P74 on one reused buffer, and assert that the capacity, the pointer, or both
   are unchanged after the first block.
   - It is a cheap, deterministic test. It catches this whole class without any
     measurement.
3. **Measure faults, not only peaks.** Minor faults (`ru_minflt`) showed the realloc on
   every interleaved corpus. Peak RSS showed it only where the buffer is a large share of
   the peak: small corpora at low worker counts.
4. **Put a small multi-block, mixed-selection corpus at w = 1 and w = 2 in any
   per-block-dispatch trial.** That is where reuse across kernels happens most, and where
   a buffer-sized offset is a visible share of RSS.
5. **Rely on the paired RSS rule.**
   - The max-over-rounds rule was fooled by one A outlier: it read 0.970.
   - The paired lower bound, 1.021, exposed the offset.

## Where else this applies

Any per-block or per-chunk dispatch between implementations that differ in padding or
alignment needs on a shared buffer, for example:
- SIMD tails;
- over-read guards;
- an alternative T-vector layout;
- a different run-length kernel;
- a decoder fast path.
