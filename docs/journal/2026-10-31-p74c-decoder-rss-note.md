# P74-C decoder trial: what B's extra peak RSS on small controls is (pre-trial note)

**Status:** written before the i9 run. This is a diagnosis on the preparation VM, using
scratch builds of pin `3870c12` that are not committed. Nothing in the frozen trial
changes.

**The observation (measured, CI runs 41196164480 and 41198956443):**
- B's peak RSS exceeds A's on the small controls at w = 2, most clearly on
  mixed-independent: the paired mean is +2% to +8%. The registered RSS rule is NEGATIVE
  there in every job.
- Locally, mixed-independent at w = 2 reproduces it: the paired median B − A is
  **+0.83 MB**, and 19 of 30 runs exceed +0.5 MB. At w = 1 it is +0.16 MB, and text at
  w = 2 shows +0.03 MB.

**Hypotheses refuted (measured):**
1. *Faster decoding queues more decoded blocks.*
   - On mixed-independent B is not faster than A: 26.5 against 26.2 ms on CI.
   - Within each arm, wall time and peak RSS are uncorrelated (Spearman |ρ| mostly
     < 0.4, with mixed signs, across 4 jobs).
2. *Out-of-order completion needs more output buffers.*
   - An instrumented scratch build counted the output buffers each worker took fresh
     rather than from the recycled pool (`free_out`). There were 3 per run in both
     arms, every run.
   - The block completion orders are similar in both arms.

**Mechanism (measured, by a scratch intervention):**
- The shipped AVX2 kernel `mtf_avx2` does `ll.reserve(max)` (`src/mtf.rs:86`). P74-C's
  combined kernel does `ll.reserve(max + 32)` (`src/mtf/p74_c.rs:119`), because its
  short-run store can write 31 bytes past n.
- A worker reuses its `ll` across blocks. When its previous block was bypassed (the
  shipped kernel, capacity exactly `max`), the next P74-C block's reserve is 32 bytes
  short.
- `Vec` then grows amortised to 2 × `max`. A `realloc` that can't extend in place copies
  the old contents into the new region (touching it) and frees the old region, which
  stays resident: about +0.9 MB on a 900 kB block size.
- **The intervention:** a scratch build where `mtf_avx2` also reserves `max + 32`, so both
  kernels ask for the same capacity. Over 30 paired runs it reduced B − A on
  mixed-independent from +0.83 MB to **+0.24 MB** at w = 2, and from +0.16 MB to
  **0.00 MB** at w = 1.
- Text is unchanged, at +0.03 MB. The +0.24 MB residual isn't separated from noise here:
  no A/A null was run locally.

**What this means for the i9 run (deduced):**
- The excess comes from integration (a 32-byte capacity mismatch between the two kernels'
  buffer reservations), not from the P74 algorithm or the selector.
- It arises only when one worker runs a bypassed block and then a P74-C block.
- **Risk to the RSS gate:**
  - It can recur at any w where a worker decodes more than one block. The registered RSS
    rule is therefore at risk at w = 1 and 2 on the small controls.
  - On Wikidata it adds roughly 0.9 MB per worker at most, about 0.35% of a ≈ 250 MB
    process per worker. At w = 4 that is up to ≈ 1.4% if every worker re-grows,
    against the 1.01 limit.
  - CI at w = 2 measured RSS B/A 0.990–1.005 on Wikidata, so in practice `realloc` often
    extends in place.
- **What stays as it is:** the trial runs frozen, with no kernel, buffer or threshold
  change. This note records the mechanism before the result, so an RSS NEGATIVE can be
  read against a cause stated in advance.
- **A production fix,** if P74-C is ever promoted, is a separate, pre-registered change.
  The shipped kernels would reserve the same padded capacity, or P74-C's tail store would
  stay within `max`.
