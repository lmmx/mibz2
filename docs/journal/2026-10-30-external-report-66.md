# External research report 66: can a breakthrough be shipped, and how? (verbatim)

The maintainer's question, quoted, and the reply it was given, relayed together; the reply
is unattributed in the relay.

**What it says:**
- **A shippable gain is plausible; a breakthrough is not yet supported.**
- **P74-C:** at most about 5.8% (Zen 4) or 4.1% (Zen 5) of decoder time. Ship it by
  removing classification, not optimising it:
  - a one-way switch;
  - a counter on the fallback path;
  - checks at checkpoints;
  - an early check, so short blocks don't exhaust their budget.

  Feasibility: L(p) + H + M(n) ≤ 0.03·T_T(n) on the controls.
- **The gates:** stage; full decoder at 1, 2 and 4 workers; CPU-s/GB; RSS ≤ +1%; generality;
  target hardware.
- **A breakthrough must attack the walk:** 46–55% of one-core decode time. A 20% decoder cut
  needs a 36–43% walk cut. N1's lookups suffice in principle; its builder is not
  demonstrated.
- **Two tracks:**
  - A: P74-C, for 4–6%;
  - B: a traversal with a ≥40% lower walk cost and construction small enough to keep ≥20%
    net, derived first from operation counts and a memory budget.

## The message, as received

> Can we obtain a positive result (sufficient to ship a breakthrough in performance) and how? MATH ENG DECISIONING CONCISE ONLY

Yes, a shippable improvement is plausible. A breakthrough is not yet supported by the
measurements. P74 is the shortest route to a positive result; a large (>20%) full-decoder
gain needs a different architecture.

1. Establish the performance limits

Measured P74 stage reductions: 33.2% (Zen 4), 19.2% (Zen 5).

Assuming other stages unchanged:

G_{\rm decode}=s_{\rm MTF}g_{\rm MTF}G_{\rm Zen4}\approx0.174(0.332)=5.8\%,\qquad
G_{\rm Zen5}\approx0.215(0.192)=4.1\%.

Decision: P74 can plausibly deliver a valuable 4–6% decoder-time reduction. It cannot
deliver a 20–30% breakthrough merely by eliminating selector overhead.

2. Ship P74 by eliminating classification, not optimizing it

Use a one-way online switch:

* Start in branch-free P74.
* Increment a counter inside the existing high-rank fallback path.
* Inspect at existing checkpoint boundaries.
* Switch permanently to shipped when the estimated excess cost exceeds a fixed budget.
* Never scan the complete symbol stream; never reverse-switch.

Critical improvement: avoid a mandatory 4,096-token probe. Check early enough that short,
fallback-heavy blocks cannot exhaust their 3% regression budget before switching.

Define the unfavorable-prefix cost:

L(p)=T_P(p)-T_T(p).

For a block of tokens, empirical feasibility requires

L(p)+H+M(n)\le0.03T_T(n)

on controls, where is handoff and monitoring overhead.

Kill condition: if no switching frequency satisfies both the control budget and Wikidata
gain, archive P74-C.

3. Separate shipping criteria from breakthrough criteria

Gate	Required result
P74-C stage	Wikidata checkpoint ≥5% faster; control regressions ≤3% at upper 95% CI
Full decoder	Positive throughput gain at 1, 2 and 4 workers
CPU efficiency	No increase in CPU-seconds/GB
Memory	Peak RSS ≤1% above shipped
Generality	Correct on adversarial rank sequences and all supported alphabets
Shipping	Reproducible gains on actual target hardware, not only SMT-heavy CI

A successful stage gate authorizes investigation, not shipping.

4. For a breakthrough, attack the dominant stage

The shipped BWT walk consumes approximately 46–55% of one-core decode time.

To reduce total decoder time by 20% through the walk alone requires

g_{\rm walk}\ge\frac{0.20}{0.46\text{–}0.55}
=36\text{–}43\%.

N1's lookup reductions are theoretically sufficient, but its builder economics are not
demonstrated.

Reopen N1 only if a new constructor materially lowers operation count, rather than merely
predicting higher IPC. Otherwise, retain the shipped one-pass builder and seek a different
way to reduce dependent walk operations without an expensive auxiliary index.

Engineering decision

Two-track strategy:

Track A — shippable incremental gain: complete P74-C's cost and state-equivalence
analysis; if viable, authorize one preregistered implementation experiment. Target 4–6%
full-decoder improvement, with controls protected.

Track B — actual breakthrough: research a BWT traversal architecture with ≥40% lower walk
cost and construction overhead small enough to retain ≥20% net decoder improvement.
Require an explicit operation-count and memory-budget derivation before implementation.

Bottom line: P74 offers a credible route to a modest shipping win. A breakthrough requires
eliminating expensive BWT-walk work without paying N1's construction bill. That is the
unresolved mathematical engineering problem.
