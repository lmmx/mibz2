Two new decoder comparisons are frozen and queued for Louis to run on his workstation. One tests whether guarded service threads help the production decoder; the other retests a faster result first seen in continuous integration. The release scaling check is ahead of them in the [workstation queue](https://github.com/lmmx/mibz2/blob/d54632f6/bench/workstation-queue.txt), and no submitted bundle from it was confirmed when this account was checked.

## Why are these comparisons next?

An [untimed check](https://github.com/lmmx/mibz2/pull/145#issuecomment-6745202683) showed that a public ten-thread request can reach the guarded service plan, but it did not measure speed. A separate [continuous-integration comparison](https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-09-run-fed-leader-result.md) reported a lead for the run-fed design, which prepares decoding tables from input runs. Its [first workstation attempt](https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-09-run-fed-leader-workstation-result.md) was inconclusive because the control measurements were unstable. The new attempts ask whether either path helps on Louis's workstation under its own frozen comparison; neither has a timed result yet.

## What allowed them to join the queue?

The Researcher [checked the exact source heads](https://github.com/lmmx/mibz2/pull/145#issuecomment-6747660830) after repairs to evidence collection and child-process exit checks. Louis then [confirmed in the Coordinator's session](https://github.com/lmmx/mibz2/pull/145#issuecomment-6747734847) that the amended experiment-design rule covers the two choices, clearing the immediate hold on freezing them. The Coordinator [froze both registrations, reran interrupted setup checks and queued their green commits](https://github.com/lmmx/mibz2/pull/145#issuecomment-6749838199). The separate [request to re-pin all four role tasks](https://github.com/lmmx/mibz2/pull/145#issuecomment-6745788393) remains open so later reviews can use the amended rule directly. The initial check failures occurred before jobs started; the successful reruns do not supply workstation results.

## What happens next?

Louis can run the [three ready recipes in order](https://github.com/lmmx/mibz2/blob/d54632f6/bench/workstation-queue.txt) on his workstation and submit their evidence. The Coordinator will read each bundle before removing its queue entry. The release scaling bundle was still unconfirmed at the last [queue report](https://github.com/lmmx/mibz2/pull/145#issuecomment-6749838199); whether any run has since started is not established here. A green build and a reachable decoder path do not yet qualify either new design for production.

## What else is established?

Version [0.2.7](https://github.com/lmmx/mibz2/pull/145#issuecomment-6745555994) released the feeder panic guard; its paired continuous-integration check detected no change at that run's precision. The register-based prototype's separate second diagnostic was valid and inconclusive, and timing on that runner is closed. The [supporting levels](L2.md) keep these findings and the open downstream update distinct.
