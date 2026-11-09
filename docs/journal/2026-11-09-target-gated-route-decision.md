# Decision record (DRAFT, not enacted): the target-gated acceptance route

**Status:** draft, for review. Enacting it needs two things:
- the role tasks' authenticated re-pin to a commit that holds the amended RULES C6 (request 6745788393);
- the Guardian's review of this record and of `docs/STEWARDSHIP.md`'s "Target-gated changes" paragraph (D071, D072), followed by a LOG on #145.

It applies forward only, from the commit that merges it.

**What it decides:** a second, prospective merge route for changes to measured code whose new behaviour only a target predicate selects, and which CI never satisfies. The route is defined in `docs/STEWARDSHIP.md`, conditions 1 to 7.

**Why a route is needed at all.** The standing rule needs a beneficial CI end-to-end interval. A change that switches on only on machines CI doesn't have can never meet it, however it performs on its target. Without a route, every such change is unmergeable by construction. The route doesn't weaken the evidence asked of the target. It asks for superiority there, with a Bonferroni family over the cells, plus no harm at the fallback budgets. What it relaxes is CI's beneficial interval: in its place, CI must show noninferiority within a 2% margin, and nothing more.

## Why 2%

- **It is the margin already in use.** The registrations' no-harm rule uses the same one-sided bound, at most 1.02: the pipelined default's precedent, and the guarded service default's 11 to 20 thread budgets. The route invents no new tolerance.
- **It sits at the CI bench's resolution.** The paired bench's end-to-end intervals at the PR tier are typically about ±1 to 2%. A tighter margin would fail changes that have no real effect, through noise alone. A looser one would admit regressions the bench can resolve.
- **It is a statistical margin, not a guarantee.** Passing shows, with 95% confidence per cell, that no standing CI cell is worse by more than 2% under the registered estimator. It doesn't bound every run or every untested machine.

## Accumulated regressions

A 2% margin against each change's immediate parent is not a lifetime bound. Ten genuine 1% slowdowns, each admitted against its own parent, compound to 1.01^10 ≈ 1.105. So:

- **A drift check at every release.** The release runbook's paired run of the previous release's tag against master (RELEASING.md §1) is read, for every release that contains a change admitted by this route, as an off-target drift check against the last release. That is a fixed anchor, not the immediate parent.
- **What follows a regression there.** If any standing CI cell shows head over base with a one-sided 95% upper bound above 1.02 against the last release, two things follow:
  - the release notes state it;
  - the route is suspended for new merges until a reviewed decision record either attributes the regression and accepts it explicitly, or reverts the cause.
- **Nothing is netted off.** The check doesn't subtract any target benefit from off-target cost. No fleet-wide gain is inferred from unweighted cells.
- **No lifetime budget beyond the per-release anchor** is set here. A tighter cumulative budget would be its own prospective decision.

## Scope

- **#302 is unaffected.** Its existing result, service-guarded/20261109T195345Z, can't qualify. Its verdict, including NOT SHOWN at 12 and 16, is unchanged. Adopting #302 by this route would need a new registration, frozen after the enacting commit, that meets conditions 1 to 7 and discloses the earlier attempt.
- **Nothing here authorises a run, a merge or a release.**
