# External research report 64: bounded estimation error, and a hardened option C (verbatim)

Two relayed messages from the maintainer, unattributed in the relay.

**What they decide:**
- **A debt threshold guarantees nothing by itself.** |D_n − D̂_n| ≤ Σ|d_i − d̂_i| grows as
  O(nε), and d_i may depend on context. Two outcomes are possible, and the analysis must
  say which is achievable:
  - **provable safety:** a conservative bound on cumulative excess, with the probe,
    monitoring and handoff charged; possibly too pessimistic;
  - **empirical safety:** pre-registered representative and adversarial workloads,
    paired CIs, a maximum regression and stopping rules. Never to be called worst-case.
- **The hardened design:**
  - start in P74, with no whole-block prefix commitment;
  - switch one way to the shipped kernel only, irreversibly, at a verified safe boundary;
  - monitor at existing loop boundaries. E_t = t − N_RUNA − N_RUNB − N_EOB, and the cost
    is compared in generated instructions;
  - use the surrogate D̂ = Σ_{q≥32} w(q) − α·E_low, with thresholds fixed before
    benchmarking;
  - target empirical robustness.
- **A pre-registered adversarial suite of valid encoded streams:** all high-rank; a low
  prefix then a high suffix; alternating windows; just below threshold; short blocks; long
  RLE2 runs at boundaries; alphabets up to 256.
- **A state-equivalence proof at the handoff,** tested at every eligible boundary on small
  inputs and at sampled positions on full blocks.
- **The next gate, defined before code:**
  - Wikidata checkpoint stage ≥ 5%, with the CI excluding zero;
  - controls' upper 95% bound ≤ +3%;
  - adversarial correctness mandatory;
  - the full decoder at 1, 2 and 4 workers;
  - CPU-s/GB no worse; RSS within +1%;
  - a layout control; the CPU, topology and frequency method recorded.

  Stage success qualifies only for a separately authorised decoder test.
- **Stop conditions:** material fast-path monitoring cost, an expensive state handoff, or
  adverse sequences defeating the control budget. No tuning after results.
- **No implementation or CI** until the costs and failure modes are quantified.

**The coordinator's action:** all of it was passed to the running option C analysis, which
will also draft (not propose) the pre-registration.

## Message 1, as received

The corrections are technically sound. N1 remains archived; P74 Option C is the only
active research question.

One further mathematical constraint matters: an estimated debt counter cannot guarantee
bounded regret unless its error is bounded.

Let , and let be the modeled excess cost. Then

D_n=\sum_{i=1}^n d_i,\qquad
\hat D_n=\sum_{i=1}^n\hat d_i,\qquad
|D_n-\hat D_n|\le\sum_{i=1}^n|d_i-\hat d_i|.

If the per-token model error is merely bounded by , cumulative error can grow as . A
threshold on estimated debt therefore cannot, by itself, guarantee a 3% regression bound
for arbitrary streams.

Furthermore, may depend on preceding ranks, memory dependencies and branch-predictor
state. A rank-only cost model cannot be treated as an exact additive decomposition without
validation.

Engineering implication: separate two possible outcomes.

* Provable safety: derive a conservative bound on cumulative excess cost, including initial
  probe, monitoring and handoff. This may be too pessimistic to retain useful acceleration.
* Empirical safety: preregister representative and adversarial workloads, paired
  confidence intervals, a maximum acceptable regression and explicit stopping rules. This
  is sufficient for a controlled performance experiment, but not a universal guarantee.

Decision: the present analysis should determine which outcome is achievable. Do not
require a universal bound if the intended acceptance standard is empirical; equally, do
not describe an empirical 3% result as a worst-case guarantee.

No further implementation or CI work is justified until the policy's costs and failure
modes are quantified.

## Message 2, as received

P74 Option C — hardened design

Objective: retain P74's measured Wikidata advantage while bounding additional work on
unfavorable inputs. No implementation authorization yet.

1. Eliminate the unnecessary prefix commitment

Do not classify the entire block after 4,096 tokens. Start in P74 and allow a one-way
switch to shipped MTF when sufficient evidence accumulates.

No reverse switching: it introduces oscillation, additional state conversion and harder
worst-case analysis.

2. Monitor at existing loop boundaries

Let count high-rank events, count explicit tokens, and denote consumed input symbols.

Avoid a new increment on every low-rank token. If is not already available, derive it from
existing state or charge its maintenance explicitly.

An exact identity is:

E_t=t-N_{\mathrm{RUNA}}(t)-N_{\mathrm{RUNB}}(t)-N_{\mathrm{EOB}}(t).

Maintaining the right-hand counters is not automatically cheaper than maintaining .
Compare generated instructions, not source expressions.

3. Separate monitoring from the switching decision

Use a conservative cost surrogate:

\widehat D_t=\sum_{q_i\ge32}w(q_i)-\alpha E_{\mathrm{low}}(t).

Here estimates fallback penalties and estimates low-rank savings. Neither is a proven cycle
cost.

Hardening rule: switching must be irreversible and occur only at a verified safe state
boundary. Trigger thresholds must be fixed before benchmarking.

A fallback-only counter is suitable for detecting unfavorable behavior, but cannot by
itself measure cumulative regret.

4. Establish the actual worst-case limitation

A bounded monitoring interval does not guarantee a 3% regression. Even perfect monitoring
cannot eliminate the cost already incurred before switching.

For a length- block, define

R(x)=\frac{T_\pi(x)-T_T(x)}{T_T(x)}.

A universal bound requires, at minimum, a defensible bound on the excess cost of every
executed P74 operation and on switching overhead.

Because timing depends on microarchitectural state, a rank-only model cannot supply that
proof.

Decision: target empirical robustness, not an unsupported universal guarantee.

5. Build adversarial acceptance cases before implementation

Pre-register one immutable workload suite covering:

Input	Failure exposed
Entirely high-rank	Initial probe and switching latency
Low-rank prefix → high-rank suffix	Late distribution change
Alternating favorable/unfavorable windows	Trigger evasion and delayed switching
Just below threshold	Persistent misclassification
Very short blocks	Fixed probe dominates work
Long RLE2 runs at switch boundaries	State handoff and checkpoint correctness
All alphabet sizes, including 256	Tail and conversion correctness

Generate valid encoded streams, not arbitrary rank arrays that violate the decoder's input
constraints.

6. Require a state-equivalence proof

At each handoff boundary:

(\mathrm{MTF\ list},\,\mathrm{symbol\ cursor},\,\mathrm{RLE2\ state},\,
\mathrm{output\ cursor},\,\mathrm{checkpoint\ state})
_P

(\cdots)_T.

The P74 head and tail must reconstruct the complete shipped list exactly. Resume with no
duplicated or skipped symbol, byte or checkpoint.

Test handoff at every eligible boundary on exhaustive small inputs and sampled positions on
full blocks.

7. Define the next gate before writing code

Metric	Acceptance
Wikidata checkpoint stage	≥5% improvement, 95% CI excludes zero
Random and records, plain and checkpoint	Upper 95% regression bound ≤3%
Adversarial suite	Correctness mandatory; predefined regression limits
Full decoder	Measure 1, 2 and 4 workers separately
CPU efficiency	CPU-seconds/GB no worse
Memory	Peak RSS within +1%
Code-layout effects	Measured same-binary control
Processor	Record actual model, topology and frequency methodology

Do not conflate stage acceptance with decoder promotion. Stage success only qualifies the
candidate for a separately authorized decoder-level test.

8. Explicit stop conditions

Reject Option C if monitoring adds material cost to the low-rank fast path, switching
cannot preserve state without expensive restructuring, or adverse sequences defeat the
control budget.

Do not tune thresholds against the acceptance suite after observing results.

Engineering decision: preserve P74's branch-free kernel, implement neither speculative
reverse switching nor an expensive whole-block census, and require the read-only analysis
to establish a credible one-way policy and cost budget first. This is the shortest
defensible route from the measured 19–33% stage acceleration to a potentially deployable
mibz2 improvement.
