# External research report 70: zero-scan dispatch, a chunk-weighted guard, the handoff layout (verbatim)

A relayed message from the maintainer, unattributed in the relay.

**What it proposes:**
- **Zero-scan dispatch** by σ = S/N_max, with an online guard kept.
- **A guard on tail-chunk work:** K_t = Σ_{q≥32} k(q), with k(q) = 1 + ⌊(q − 32)/32⌋ chunks.
  Switch when K_t + k(q) > τn_t + b. The same monotone invariant holds.
- **A 320-byte handoff layout:** slack, head slot, tail, with one ymm store at the switch.
- **Feasibility:** the region R_controls ≤ 0.03 ∧ G_Wikidata,decoder > 0 must be shown to
  be non-empty from traces before coding.
- **B2 stays separate:** walk ≤ 4.75 c/B for 20% combined.

**Checked by the coordinator in master's `src/mtf.rs` (lines 86–145):**
- **The shipped side:** the list sits at store[32..288] of a 64-aligned 320-byte buffer,
  with an aligned load at base, `loadu(base − 1)` on the < 32 path, and
  `ptr::copy(base, base + 1, nn)` on the ≥ 32 path.
- **The P74 side:** P74's `Tail([u8; 32 + 224])` has no slack, so the message's layout
  finding holds.
- **A further obligation:** the shipped kernels hold their list in their own frame, so a
  switch needs a resume entry taking the list pointer, cursor, n, counts, run state and
  checkpoint state.

All of it was passed to the option C analysis.

## The message, as received

New result from code inspection: P74 may not need online classification on most blocks.
Use a zero-scan dispatch, then event-triggered protection only where needed.

1. Exploit metadata already available

Symbols exposes and before MTF.

Define

\sigma=S/N_{\max}.

Full Wikidata blocks have relatively sparse symbol streams; random/records approach one
symbol per output byte. Test this zero-cost classifier on existing traces:

\sigma\ge\theta\Rightarrow\text{shipped},\qquad
\sigma<\theta\Rightarrow\text{P74 with online guard}.

No census, no per-token fast-path arithmetic. Caveat: short final blocks make misleading.
The online guard remains mandatory.

Agent task: compute byte-weighted false-selection rates and the range of that preserves
Wikidata coverage while rejecting full random/records blocks. No benchmark needed.

2. Replace fallback counts with actual tail-work counts

The existing P74 fallback executes

k(q)=1+\left\lfloor\frac{q-32}{32}\right\rfloor,\quad32\le q\le255

vector-chunk iterations: 1–7 per fallback.

Thus discards the strongest existing cost signal. Instead:

K_t=\sum_{q_i\ge32}k(q_i).

Switch before processing a high-rank token when

\boxed{K_t+k(q)>\tau n_t+b}.

No fast-path monitoring. No window delay. The threshold can first be violated only at a
high-rank event.

A conservative high-rank excess model suggests monitoring , not merely . The coefficients
require measurement or defensible bounds; chunk count is not equivalent to execution time.

Agent task: calculate , , switching positions and retained P74 decoded-byte share for every
existing block. This is a materially stronger analysis than rank-frequency thresholds.

3. Fix the handoff layout before implementation

Concrete code finding: current P74 allocates Tail([u8; 256]), with the tail at offset 32.
A single head store produces 256 contiguous list bytes, but the shipped AVX2 fast path also
performs loadu(base - 1). The resulting list has no valid prefix slack. Directly reusing it
would be unsafe.

Allocate 320 aligned bytes initially:

Offset	Contents
0–31	Shipped fast-path prefix slack
32–63	Reserved head storage
64–287	P74 tail

On switch, store the YMM head at offset 32. Shipped list begins there; its 32-byte prefix
slack exists. Handoff requires one head store, not a 224-byte tail copy. This layout is
derived from the inspected code; correctness and allocation effects remain untested.

4. Mathematical feasibility gate

Let be the measured low-rank gain, the conservatively estimated excess cost per tail chunk,
an upper bound on other excess work, and a shipped cost lower bound.

A conditional regret bound becomes

R\le\frac{u\tau+v}{c_{\min}}
+\frac{ub+H+M}{c_{\min}n}.

Require feasible satisfying both:

R_{\mathrm{controls}}\le0.03,\qquad
G_{\mathrm{Wikidata,decoder}}>0.

Use existing traces to establish whether the feasible region is nonempty before coding. Do
not represent fitted costs as adversarial guarantees.

5. Breakthrough track remains separate

P74 cannot independently deliver a 20% decoder gain. The shipped walk consumes
approximately 52% of decode time.

With a hypothetical P74 saving of cycles/B on the 7763, a 20% combined decoder-time
reduction requires

\boxed{T_{\mathrm{walk,new}}\le4.75\text{ cycles/B}}

versus 6.69 shipped.

Research candidate B2: AVX2 gathered traversal of the existing T-vector, tiled output,
scalar replay at segment boundaries. No N1 construction or new index. Its decisive unknown
is dependent-gather throughput plus transposition cost. Do not claim speedup without
measuring those.

Decision: prioritize the zero-scan P74 dispatch plus chunk-weighted fallback guard. This
attacks the actual reason Gate 1 failed and exposes a plausible shippable improvement. For
a genuine ≥20% breakthrough, the separate T-vector walk must also become substantially
cheaper.
