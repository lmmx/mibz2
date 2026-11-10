# Decision record (DRAFT, not enacted): the target-gated acceptance route

**Status: enacted 2026-11-10,** by the merge of #309, forward only from that merge commit.
- **Authority:** RULES C6, as amended in #306 (Louis, in the Coordinator's session, 2026-11-09).
  The role tasks are re-pinned to `a2da49638ac54d7763de1c167bd054f67ffdc763`, which carries the
  amended C6. The read-back is verified character for character: request 6745788393, REVIEW
  6755222109.
- **Review:** the Guardian, D071 to D074. D074 ruled the final text, at `552d1d3c`, ADMISSIBLE
  with no further edit. The enacting PR changes nothing in it but this status line and a merge
  of master.
- **Log:** on #145, the Coordinator's enactment LOG.

**What it decides:** a second, prospective merge route for changes to measured code whose new behaviour only a target predicate selects, and which CI never satisfies. The route is defined in `docs/STEWARDSHIP.md`, conditions 1 to 7.

**Why a route is needed at all.** The standing rule needs a beneficial CI end-to-end interval. A change that switches on only on machines CI doesn't have can never meet it, however it performs on its target. Without a route, every such change is unmergeable by construction. The route doesn't weaken the evidence asked of the target. It asks for superiority there, with a Bonferroni family over the cells, plus no harm at the fallback budgets. What it relaxes is CI's beneficial interval: in its place, CI must show noninferiority within a 2% margin, and nothing more.

## Why 2%: the proposed tradeoff

This is a prospective tolerance choice. The route permits evidence consistent with a small
off-target cost, assessed against a 2% margin, in exchange for separately established target
superiority. It does not show that the fleet benefits overall. The margin follows earlier
no-harm registrations as a design precedent, not as authority to reuse their data or a proof
that the cost is acceptable everywhere.

Acceptance of that tolerance is limited by the deployment/coverage argument, the complete CI
wall/CPU grid, the target and fallback requirements, the predeclared other-regression criteria
and the release-anchor surveillance below. The authorised policy decision explicitly accepts
that tradeoff and its lack of a lifetime cumulative bound. Each adoption still states why that
exposure is acceptable for its proposed deployment scope; the margin is a ceiling for this
route, not a claim that all costs below it are harmless.

Precision and tolerance are separate. The registered sample size and interval method determine
how often an unchanged or improved candidate can establish noninferiority. Inconclusive
evidence fails qualification; it is not grounds to widen the margin or retry to pass. No fixed
CI "resolution" or power guarantee is asserted here.

Passing all fixed CI cells supports the level-5% intersection-union noninferiority conclusion
defined in STEWARDSHIP. It is not simultaneous 95% coverage of every interval, a posterior
probability that all cells satisfy the margin, or a guarantee about untested machines.

## Accumulated regressions

A 2% margin against each change's immediate parent is not a lifetime bound. Ten genuine 1% slowdowns, each admitted against its own parent, compound to 1.01^10 ≈ 1.105. So:

- **One pre-release anchor check.** For every release containing a route-admitted change, use
  the single paired run required by [docs/RELEASING.md §3 item 2](../RELEASING.md). Before its
  observations, record the previous released tag and resolved commit, the exact
  release-candidate head, bench revision, complete standing runner/corpus/thread wall-and-CPU
  matrix, paired geometric-mean estimator, one-sided 95% bound calculation, sample/stopping and
  invalidity rules. Archive the raw paired observations and calculation. Compute the required
  bounds without display rounding or best-thread selection; missing cells, invalid runs and
  unmatched comparisons are not evidence of noninferiority. This specifies the analysis of that
  run, not permission for an additional run.
- **Failure to establish the margin.** If any required cell has U >1.02, or a required bound
  cannot validly be obtained, disclose which result is inconclusive, missing/invalid, or
  demonstrates harm under the declared inference rule. Suspend this route for new merges.
  Preserve the anchor and the unresolved result across subsequent tags; a new tag alone cannot
  clear the suspension. Resume only through the predeclared escalation: a reviewed decision
  record either accepts the exposure explicitly under the applicable authority with prospective
  resumption terms, or records the corrective/reversion evidence resolving it. Attribute a cause
  only where evidence supports attribution. Neither path converts NOT SHOWN into PASS, rewrites
  a frozen verdict, relaxes a registered qualification rule, or permits retry-to-pass.
- **What the anchor measures.** The comparison measures all changes since the prior release,
  not necessarily harm caused by the route-admitted candidate. Disclosure and suspension of
  this route do not by themselves create a new blanket release prohibition; the existing
  release and unacceptable-regression obligations still apply.
- **Nothing is netted off.** The check doesn't subtract any target benefit from off-target cost. No fleet-wide gain is inferred from unweighted cells.
- **No lifetime budget beyond the per-release anchor** is set here. A tighter cumulative budget would be its own prospective decision.

## Scope

- **#302 is unaffected.** Its existing result, service-guarded/20261109T195345Z, can't qualify. Its verdict, including NOT SHOWN at 12 and 16, is unchanged. Adopting #302 by this route would need a new registration, frozen after the enacting commit, that meets conditions 1 to 7 and discloses the earlier attempt.
- **Nothing here authorises a run, a merge or a release.**
