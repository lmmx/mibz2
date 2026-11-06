# External research report 71: B2's threshold corrected; gathers are heavy on Zen 3/4 (verbatim)

A relayed message from the maintainer, unattributed in the relay.

**What it corrects:**
- **The threshold mixed runs.** 12.93/6.69 c/B (PR #114's run) and 4.49/10.78 c/B (CI run
  40579732655) are different measurements. On the consistent run: walk ≤ 2.33 c/B for −20%
  decoder time, or ≤ 2.69 for +20% throughput. 4.10 buys only about 3.6%. B2 needs a
  40–48% cheaper walk kernel.
- **VPGATHERDD ymm (uops.info):** 39, 42 and 32 µops on Zen 3, 4 and 5, that is 4.88, 5.25
  and 4.00 µops per byte from the gather alone. The coordinator's "3–4 µops/B for the
  loop" is untenable on Zen 3/4.
- **ROB occupancy:** 256/16.9 is not a valid occupancy figure, since 16.9 is an i9
  instruction count. The window hypothesis is plausible but unquantified.
- **The necessary comparator:**
  - A: the shipped room walk;
  - B: the same scalar loads, with tiled control and output;
  - C: gathers with the same tiling.

  Kill B2 as a breakthrough if the complete room kernel can't reach 2.33 c/B.

**The coordinator's response:** all four corrections accepted. The mixed-run target and the
µop and ROB figures were the coordinator's errors in the conversation (none is in a
committed journal note). They were passed to the running B2 analysis with the
three-kernel comparator and the kill rule.

## The message, as received

Critical finding: B2's current breakthrough threshold is wrong. It mixes benchmark runs.

I checked src/bwt.rs and the four-copy journal. CI run 40579732655 reports 10.78 c/B total,
4.49 c/B for the production room walk, and 0.16 c/B for assembly, at one copy. The earlier
12.93 c/B total and 6.69 c/B walk come from different measurements.

For a 20% decoder-time reduction using B2 alone, on the internally consistent run:

\boxed{T_{\mathrm{B2,room}}\le4.49-0.20(10.78)=2.33\ \mathrm{c/B}}

For 20% higher decoder throughput instead:

\boxed{T_{\mathrm{B2,room}}\le4.49-\frac{10.78}{6}=2.69\ \mathrm{c/B}}

Both assume unchanged non-walk costs and assembly. The previously proposed 4.10 c/B target
is insufficient: reducing 4.49 to 4.10 saves only approximately 3.6% of decoder time in
that run.

B2 therefore needs 40–48% lower walk-kernel cost, not merely a small gather improvement.

Second finding: Zen 3 AVX2 gather is much heavier than B2's model assumes.

Measured VPGATHERDD costs:

CPU	Executed µops/gather	Gather µops/byte	Independent throughput
Zen 3	39	4.88	8 cycles/8 elements
Zen 4	42	5.25	8 cycles/8 elements
Zen 5	32	4.00	5.67 cycles/8 elements

uops.info

The proposed 3–4 µops/B for B2's entire loop is therefore untenable on Zen 3/4: gathering
alone exceeds it. Transposition, flags, stores and replay add more.

Moreover, is not a valid ROB occupancy calculation: 16.9 is an instruction-count estimate
from another processor, not measured Zen 3 µops/B. The instruction-window hypothesis
remains plausible but unquantified.

Constructive alternative: scalar-load SIMD tiling

Keep the 32 independent scalar load chains. Vectorize/amortize control and output, not
random loads. Compare three kernels on identical T-vectors:

* A: shipped 32-lane room walk, 4.49 c/B.
* B: scalar loads, tiled flag handling and output commits.
* C: AVX2 gathers, identical tiling and replay.

The comparison isolates tiling/control savings; isolates gather's actual contribution. Both
must include replay and output assembly.

Kill B2 as a breakthrough candidate if its projected complete room kernel cannot reach
2.33 c/B. A smaller positive gain may still justify separate consideration, but must not be
described as a 20% decoder improvement.

Decision: retain B2 for analysis, add scalar-load tiling as its necessary comparator, and
remove the unsupported 3–4 µops/B and 4.10 c/B assumptions. The breakthrough hypothesis is
now precise: can we almost halve the production room walk's cost without adding an index or
sacrificing memory efficiency?
