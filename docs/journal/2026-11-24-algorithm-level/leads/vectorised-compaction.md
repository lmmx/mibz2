# Lead: hand-vectorised stream compaction (copy_if), for the workers' stages

*Louis, 24 November 2026, ~22:30, found it after following causal profiling (Coz) into performance engineering: https://loonatick-src.github.io/posts/vectorized-copy-if-analysis/ (Chaitanya Kumar, 25 May 2026). Only about 60% of the page was read verbatim. The later sections, including the fixed version's speedups, came back as a summary and are unverified.*

## What the post shows

- `copy_if` (stream compaction) does not autovectorise. Each output position depends on all the earlier predicate results: a loop-carried dependency.
- **The trick:** compare a whole vector at once to get a bitmask, compress the selected lanes into contiguous order, store the full vector unaligned, and advance the output pointer by `popcount(mask)`. The loop is branchless, and the trailing garbage is overwritten by the next store, so the output buffer needs padding.
- The post does this with AVX-512 (`vpcompressd`). **On Zen 4 the memory-destination form is microcoded** (144 uops), and runs at about 4 GB/s. The register-destination form followed by a plain store is the fix, reported at about 10–40× over `std::copy_if` (unverified). Lesson: check uops.info before choosing an intrinsic.

## Fit for mibz2

**The workstation's i9-10900K has AVX2 but no AVX-512,** so `vpcompress` is unavailable there. The AVX2 equivalent is a 256-entry shuffle table indexed by an 8-bit movemask, plus `vpermd` and a popcount advance. Alternatively, BMI2 `pext` can do the compaction.

Where compaction-shaped work sits in the decoder:
1. **Run-record extraction (T3, `src/runs.rs`).** The scan already builds 32-symbol bitmaps with `movemask` (the `lit` and `one` words). Turning bitmaps into position or record lists is compaction. Whether the current bit-iteration is a measurable cost is unknown.
2. **Thin/thick partition in Open's construction (T2).** Selecting width-1 runs, or runs at or below the strip cap, is a filter. The per-byte queues are a multi-way partition, which is harder than single copy_if, though the same masks help.
3. **Lanes' branchless appends (T1).** The earlier lanes result (0.55–0.78× of emit) already used branchless appends; this is the same family.

**Verdict:** an engineering lead, not a mathematical one. It lowers the unit price of filter steps inside the workers; it does not change what the algorithm does.

**When to use it:** only after the decomposition measurement (`briefs/decomposition.md`) shows that one of these worker stages controls throughput at the counts that matter. Then prototype the AVX2 left-pack there and bench it.
