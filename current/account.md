Version 0.2.9 has shipped a measured Wikidata gain at six to twenty threads, but its release comparison also found a small one-thread loss. The downstream project adopted the release; its four-thread decoder cap means the new team-path gain cannot reach that project yet. A separate faster inversion method is being prepared for production, but its first slice is held for missing safety-test coverage and has not made whole decoding faster.

## What is the team doing now?

The [release-scaling run](https://github.com/lmmx/mibz2/pull/145#issuecomment-6774193623) found Wikidata about 10–15% faster at eight to twenty threads on Louis's workstation, and 0.9% slower at one thread; two and four threads were level. Pathfinder was asked to make a small inlining change separately, with its own benchmark, to address the measured one-thread cost. The team also [tested a lower team threshold](https://github.com/lmmx/mibz2/pull/145#issuecomment-6775000939): forced teams at four and five threads did not beat the existing plan, so the threshold stays at six.

The [wikidata-pq update](https://github.com/lmmx/mibz2/pull/145#issuecomment-6774165215) merged after Louis overruled an initial close. Its four-thread cap still prevents this particular team gain from engaging. Raising the cap is a separate workstation comparison, not an established downstream benefit.

The researcher has moved from the merged mathematical example toward a [one-thread production route](https://github.com/lmmx/mibz2/pull/145#issuecomment-6774426061). A pinned i9 run measured its inversion stage at about 82% of the current stage's time. The first implementation is [off by default](https://github.com/lmmx/mibz2/pull/365); a permitted follow-up found no default performance loss, yet the experimental route was level on whole Wikidata decoding and much slower on some other inputs. The stage saving has not become a whole-decoder gain.

An independent [safety review](https://github.com/lmmx/mibz2/pull/145#issuecomment-6782367271) now holds that implementation: its successful Miri tests did not reach the long-word writer path. The reviewer supplied a bounded test and mutation target for the author to add and then recheck. This is missing safety coverage, not a demonstrated memory-safety bug. The macOS jobs were cancelled, so they are not passing evidence. The default route, team threshold and thread cap have not changed.

## What will settle the remaining questions?

The Optimiser [replayed both archives](https://github.com/lmmx/mibz2/pull/145#issuecomment-6782313693) and found that predicted admission covers about 84% of Wikidata rows; coverage alone does not explain the absent whole-decoder gain. The older phase trace times another path, so it cannot locate the cost. The Coordinator accepted one bounded live-path diagnostic in principle, conditional on actual route and phase evidence from the Codex investigation already under way. No second build or workstation run has been launched for that question.

A separate [one-thread diagnostic](https://github.com/lmmx/mibz2/pull/145#issuecomment-6782294636) for an earlier candidate passed Guardian review of its preparation, but its exact-head CI has a failed occupancy assertion. It has no ready workstation job or run signal. Even a favourable result would not erase that candidate's earlier text loss. Louis would start an eligible workstation queue after its checks; no one is treating the missing dispatch route as a completed run.

## Who is coordinating?

Louis [activated one existing acting Coordinator](https://github.com/lmmx/mibz2/pull/145#issuecomment-6782075529) during Claude's quota pause, after the authorised failover change merged. The original queues continue. This adds no parallel scientific role or automatic merge instruction. Codex has reported an owner-launched investigation, but comments do not automatically restart that session. The Observer records these intentions and results; it does not judge or dispatch the experiments.
