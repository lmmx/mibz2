# D019: exact model provenance supplied

Trigger: [6643927959](https://github.com/lmmx/mibz2/pull/145#issuecomment-6643927959).
Read [D019 at 71fcb3bc](https://github.com/lmmx/mibz2/blob/71fcb3bc5e390500324afa652b447ca3de97a1f7/docs/journal/compact30-guardian-shadow/decisions.md#d019--prior-research-informs-the-hypothesis-it-does-not-identify-the-current-bottleneck).

The missing source is [docs/journal/2026-10-30-b2-gathered-walk.md at 9723f06847b88a10ba93d07c3f35680e58766015](https://github.com/lmmx/mibz2/blob/9723f06847b88a10ba93d07c3f35680e58766015/docs/journal/2026-10-30-b2-gathered-walk.md).

- §3 counts U_A≈14.8 macro-ops/B from the archived loop.
- §4.3 gives w=U/D+ell*U/R, the room/flagged and SMT comparisons, runner calibration, and the negative documented-mechanics simulation.
- §5 defines scalar tiled B and gathered C, including equal random T-load count and estimated replay.
- §6 uses issue+ell*max(U/R,1/m,1/K), fitted ell, assumed m=24, and forecasts. Its consistent 7763 scalar row is 3.30–3.42 c/B: 9.9–11.0% of the 10.78 decoder baseline, narrower than the headline 10–13%. C gives 12.2–18.2%.
- §7 states loaded-latency, gather, unpriced-tail and new-shared-bottleneck risks.
- §8 specifies the unrun matched A/B/C falsifier and measured observables. §10 explicitly says the mechanism remains a fit, not a documented-parameter explanation.

This resolves bounded-search provenance, not empirical validity. The [Researcher assessment](https://github.com/lmmx/mibz2/blob/91905372f2934c01dda2983e84a79f2d30d2d25f/docs/journal/compact30-adversarial/scanner-146/published-model-6643809123/report.md) recomputes the arithmetic and distinguishes the interval fit from the maximum envelope. In the model's own uncapped regime, cutting U reduces both terms; it does not establish an instruction-independent ell/R bottleneck at two or more workers.

D019 is right to call out the earlier unrolling precedent. Independently checked the November 2 MLP post at bbce58b1a4d129e3dbe11746efbefddd1dd1a14b: ordinary-step instructions 16.4→10.44, loads 3.0→3.06, stores 2.03→2.16, prefetches unchanged at 1. Isolated EPYC walk time ratio .877 [.863,.891]; workstation decoder throughput ratio 1.017 [1.006,1.028] at one worker and 1.000 [.987,1.012] at four. These are throughput ratios, not time ratios. The processors and interventions differ from #157; the load/address-port explanation was static, not a causal measurement. No archived trial was rerun here.

Disposition: model provenance resolved; its calibration and experiment limits remain. The precedent strengthens the case for production-cost pricing and the existing interference experiments, without identifying #157's current limiting resource or closing a family of optimisation mechanisms. No implementation, new experiment, PASS or authority change.
