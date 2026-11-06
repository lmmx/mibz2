# External research report 78: bb17a24's exploratory timing is non-qualifying (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-10-31 08:57 UTC.
- **Source:** unattributed in the relay (the researcher's register).
- **In reply to:** the P74-C handoff's measurement record.

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

````text
Worked for 1m 38s

Immediate finding: bb17a24's exploratory timing is non-qualifying. Its committed environment reports 6.71% CPU steal time, zero cgroup throttling, and `predeclared_noise_flags_triggered=true`. Zero throttling does not imply an uncontended core. Do not use these timings to establish a 3% regression bound or a shipping gain. [Measurement record](https://github.com/lmmx/mibz2/commit/bb17a240ccc9967a60528d72008279e1dc41e126?utm_source=chatgpt.com).

The independent Wikidata windows are useful holdouts, but not independent of the source distribution: they come from the same dated dump. Freeze the policy, code and acceptance rules before examining their results; use separate-source data for external generalization.

Code-review findings: `src/mtf/p74_c.rs` preserves the shipped suffix and switches before applying the triggering rank. The one-store list handoff is structurally credible, although this inspection is not a correctness proof. The optimized low-rank loops still contain +1 instruction in plain and checkpoint modes; that fails the frozen assembly gate, despite no additional low-rank branch or YMM-head spill. [Assembly audit](https://github.com/lmmx/mibz2/blob/bb17a24/docs/journal/2026-10-31-p74-c-assembly-correction.md?utm_source=chatgpt.com).

The switching audit is decisive: zero online handoffs in 431 fitted Wikidata blocks; all 20 random/records blocks bypass P74 entirely. Thus the observed benefit mainly validates the cheap dispatch, not the online guard's economics. Only one mixed-control block and one text block actually switch.

Breakthrough decision: do not discard B2 solely for missing its standalone 20% threshold. Its model predicts a 12–18% decoder-time reduction. P74's exploratory 26.54% MTF-stage reduction, applied illustratively to the historical 17.7% MTF share, contributes another . If both transfer to the same hardware without interaction:

\\boxed{G_{\\mathrm{combined}}\\approx16.7\\%\\text{–}22.7\\%}

That is a modeled route across 20%, not measured evidence. Require B2's construction-free walk model to account explicitly for gather dependency latency, tile transposition, exceptional replay and final assembly. Its high potential return merits continued read-only scrutiny.

Next engineering decision: complete the independent-window correctness/selection audit; require reviewer sign-off on bb17a24; then consider an explicit, documented waiver of the +1-instruction screening rule for a controlled i9 experiment. One independent address-calculation instruction is a risk signal, not proof of a performance regression. Do not silently relax the gate or execute timing without authorization.

Priority: qualify the P74 gain, retain B2 as the higher-upside research candidate, and require same-host full-decoder measurements before claiming either a shippable gain or a breakthrough.

---

Decision: P74-C deserves one properly controlled validation experiment, not a series of opportunistic replications. No execution authorized by this assessment.

First, the review contains a mathematical error: including the triggering symbol can delay switching by arbitrarily many events, not necessarily one.

For , construct 126 high-rank symbols, one low-rank symbol at position 127, then high ranks at positions , with low ranks elsewhere. At position 128:

F_{\\rm new}=127,\\quad
\\lfloor127/32\\rfloor+123=126,\\quad
\\lfloor128/32\\rfloor+123=127.

The pre-trigger rule switches; the implemented rule does not. Subsequent high events can remain exactly on the threshold indefinitely. Identical observed switches do not establish equivalent policies.

My five dispositions:

* : retain the implemented inclusive rule only as a separately documented policy. Add the boundary counterexample to correctness tests. Do not call it equivalent to the original.

* Waivers: historical second fix and excluded-host run remain exploratory, not retrospectively authorized qualification. The +1/+1 instruction gate needs an explicit maintainer waiver before performance qualification. Preserve truthful Git provenance; do not rewrite history to conceal it.

* Harness: prefer one newly preregistered experiment with T0/T1/T2, same-binary P74 Free, and an A/A control. T1 already supplies the combined shipped-loop control; avoid a redundant T0′. Reinspect the exact timed binary. Random/records differences on effectively identical paths measure combined layout, dispatch and noise—not a proven ±2.5% stochastic floor.

* Host: prefer bare-metal i9 for clean single-core evidence, plus Zen for architecture transfer. GitHub runners are exploratory substitutes, not four-physical-core evidence. Determine topology dynamically; never assume CPUs 2 and 3 are SMT siblings. Neither host is authorized here.

* Holdout: register the 12-window SHA-256 and selection rule now, before timing. Label it same-dump held-out Wikidata, not independent-source validation. Add a separately frozen, valid `.bz2` adversarial suite with verified actual handoffs; otherwise the online guard remains performance-unvalidated.

Performance accounting: fitted checkpoint is encouraging but disqualified by host interference. At an illustrative 18% MTF-stage decoder share, it corresponds to only decoder-time reduction, before transfer and integration costs. The 20% breakthrough still needs a major walk improvement.

Hard go/no-go: one preregistered stage experiment; qualified host; exact binary audit; matched holdout; negative-control upper regression bound ≤3%; observed guard handoffs with bounded cost. No threshold retuning, selective reruns or i9 execution without the maintainer's explicit approval.
````
