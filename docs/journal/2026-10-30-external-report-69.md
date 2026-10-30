# External research report 69: B2, an AVX2-gathered tiled T-vector walk (verbatim)

A relayed message from the maintainer, unattributed in the relay.

**What it proposes:**
- **The walk:** 8 segments per ymm. Q = gather(T, R), R' = (Q & LINK) >> 8, and an 8×8 byte
  tile transposed and committed only if no segment-start flag occurs in the tile;
  otherwise the tile is replayed by the scalar handler (≤ 3.64% of bytes).
- **The target on the 7763:** walk ≤ 4.10 c/B for a 20% decoder cut, or ≤ 4.75 with P74.
- **No construction and no new index.**
- **Kill it cheaply:** a tile-kernel microbenchmark against the scalar loop, one versus four
  vector groups. Not authorised; a read-only architectural candidate.

**The coordinator's reading, sent to a read-only B2 analysis:**
- **The decisive unknown** is the shipped walk's per-core binding resource.
- **Measured on the 7763:** a dependent-load latency of about 38.6 reported cycles (the
  one-chain walk) against a room walk of 4.49 c/B, so the effective MLP is ≈ 8.6, well
  under about 24 L1 miss buffers. The lane curves flatten near 24 lanes; prefetch gains
  13%; two SMT copies give 0.955× per core.
- **An instruction-window bound** (≈ ROB / µops per byte) fits these, and ROB partitioning
  explains SMT. A miss-buffer bound would give B2 nothing; a window bound could give it
  most of the walk.
- **The analysis derives which bound fits all the evidence,** then B2's µops per byte from
  the gather tables, and the predicted walk c/B under each.

## The message, as received

I found a third route worth testing mathematically: SIMD-vectorize the existing T-vector
walk. No N1 construction. No new index.

I inspected src/bwt.rs. The shipped segmented walker performs 32 independent scalar
chains, each with a load, flag check, link extraction, prefetch and byte store. It executes
approximately 16.9 instructions/B. That is a potential instruction-overhead target, not
just a memory-latency problem.

Candidate B2: eight-lane gathered, tiled traversal

Process eight independent segments per AVX2 vector:

Q_j=\operatorname{gather}8(T,R_j),\qquad
R{j+1}=(Q_j\mathbin{\&}\mathrm{LINK})\gg8.

Accumulate eight iterations, giving an byte tile. Transpose in registers; write eight
contiguous bytes to each segment buffer.

Critical optimization: defer all output commits until the tile is validated.

M=\bigvee_{j=0}^{7}Q_j.

If contains no segment-start flag, commit the tile. Otherwise discard the speculative tile
and replay it through the existing scalar segment handler. Segment state is unchanged until
commit, making correctness comparatively straightforward.

With 512 segment ends over 900,000 bytes, at most approximately

\frac{512\cdot64}{900000}=3.64\%

of byte positions require replay under this scheme, assuming one failed tile per segment
completion. Degenerate short cycles need scalar fallback.

Is the hardware budget credible?

Published AVX2 VPGATHERDD measurements report approximately 8 cycles per eight-element
gather on Zen 3/4, and 5.7 on Zen 5, with independent operands. uops.info +1

That gives an ideal reciprocal throughput near 1 cycle per gathered byte on Zen 3/4. It is
not the actual walker cost: dependent gathers, transposition, masks and segment management
remain.

The precise target on the EPYC 7763 is:

T_{\rm decoder}=12.93,\qquad T_{\rm walk}=6.69\ \mathrm{c/B}.

For a 20% decoder-time reduction from the walk alone:

\boxed{T_{\rm SIMDwalk}\le6.69-0.20(12.93)=4.10\ \mathrm{c/B}.}

With an additional hypothetical P74 saving of 0.65 c/B on that same processor:

\boxed{T_{\rm SIMDwalk}\le4.75\ \mathrm{c/B}.}

This is a concrete, falsifiable route to a 20% combined improvement. It is not yet a
speedup claim.

Kill the idea cheaply

Before engineering a complete walker, benchmark only the central tile operation against the
scalar loop on identical, already-built T-vectors.

Require:

T_{\rm gather+transpose+commit}<4.10\ \mathrm{c/B}

for a standalone 20% target, with an allowance for final segment assembly and exceptional
paths. If the central kernel already exceeds the entire budget, stop.

Test four independent vector groups to expose memory-level parallelism; compare against a
single dependent vector chain. Record actual cycles, µops, loads and cache misses. AVX2
gather's published throughput does not establish dependent-chain performance.

Decision: keep N1 archived. Continue P74-C. Add B2 as a read-only architectural candidate,
not an authorized implementation. Unlike N1, B2 has no construction bill and preserves the
existing 3.6 MB T-vector. Its principal risk is gather latency plus output transposition.

This is the first alternative in this exchange that targets a breakthrough without asking
the constructor to repay the walk's savings.
