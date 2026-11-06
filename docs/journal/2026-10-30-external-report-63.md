# External research report 63: N1 archived; option C's regret made rigorous (verbatim)

A relayed message from the maintainer, unattributed in the relay.

**What it decides:**
- **N1 is archived.** The fused builder is not falsified: IPC ≈ 1.4–2.2 is feasible in
  principle, but unsupported for this workload. #42's 16–27 cycles per record is weak
  comparative evidence, since it is not normalised for instructions or record density.
  MLP is permitted, not measured. No more construction speculation without a new
  operation-count reduction. The fused-construction note is corrected to say so.
- **Option C:**
  - R(x) = (T_π(x) − T_T(x))/T_T(x) for every valid rank stream x, with the prefix,
    monitoring, conversion and continued decoding all included;
  - worst-case safety only if sup_X R ≤ 0.03 is proved, otherwise an empirical guarantee;
  - δ·W/n bounds detection delay, not misclassification regret on alternating or
    near-threshold streams;
  - the switching quantity is accumulated excess cost (savings on low ranks, losses on
    high ones), not fallback frequency. A weighted counter's accuracy and instruction cost
    need validating.
- **Implementation is defensible only with a credible control-regression bound, or a
  clearly pre-registered empirical acceptance criterion,** with monitoring and switching
  costs charged.

**The coordinator's action:** all of it was passed to the running option C analysis.

## The message, as received

N1: archive justified, but the fused builder has not been experimentally falsified. Its
5.3–7.2 instructions/B against a 3.3–3.8 cycles/B budget implies required IPC ≈1.4–2.2.
Feasible in principle; unsupported for this rank-query workload. Removing two passes
without materially reducing total operations is insufficient evidence to fund Gate B.
Preserve the construction and correctness results.

One technical correction: 16–27 cycles per record from #42 cannot establish low IPC in the
fused design without normalizing instruction counts and record density. It is weak
comparative evidence, not a demonstrated architectural limit. Likewise, independent rank
queries permit memory-level parallelism, but throughput depends on actual outstanding
loads and cache locality; neither is measured.

P74: tighten the decision criterion. The proposed condition must account for the entire
block, including the initial probe. More importantly, the approximation

R\lesssim\delta W/n

is not a worst-case bound unless bounds excess cost per token and bounds the total number
of disadvantageous tokens processed before handoff. A monitoring window only bounds
detection delay after the policy's trigger condition becomes detectable; it does not bound
misclassification regret for arbitrary alternating or near-threshold sequences.

For a rigorous bound, define

R(x)=\frac{T_{\pi}(x)-T_T(x)}{T_T(x)}

for every valid rank stream , including monitoring, conversion, prefix work and continued
decoding. Require if claiming adversarial safety over corpus class . If only empirical
controls satisfy 3%, state an empirical guarantee, not a worst-case one.

Most important engineering challenge: a fallback-only counter is insufficient to measure
net disadvantage. P74 saves time on low ranks and loses time on high ranks; the correct
switching quantity is accumulated excess cost, not merely fallback frequency. A cheap
weighted counter may approximate this, but its accuracy and incremental instruction cost
need independent validation.

Decision: N1 archived. Complete P74's read-only online-policy analysis. Authorize
implementation only if it provides a credible control-regression bound or a clearly
preregistered empirical acceptance criterion, with monitored overhead and switching cost
charged. No further N1 construction speculation absent a new operation-count reduction.
