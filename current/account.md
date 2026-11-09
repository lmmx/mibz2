The guarded service-thread comparison finished as registered: on Louis's ten-core workstation, the production decoder ran 6.8% faster and used 17% less processor time at its tested ten-thread setting. That result supports the registered comparison on this machine, but a production change awaits a separate Guardian-reviewed adoption decision. Louis has also submitted the second run-fed comparison; its evidence has not yet been reconciled into a team verdict.

## What did the service test establish?

Louis's [submitted run](https://github.com/lmmx/mibz2/pull/145#issuecomment-6750239688) used the frozen source and comparison, and the Coordinator [checked its bundle and judged all 420 timed records valid](https://github.com/lmmx/mibz2/pull/145#issuecomment-6750273151). At ten threads, the guarded path used two service threads alongside decoding teams. It beat the current default in wall time by 6.8%, with a separate interval that excludes no gain, and used 17% less processor time. Peak memory was lower too, but memory was reported rather than registered as a condition. The earlier [untimed preflight](https://github.com/lmmx/mibz2/pull/145#issuecomment-6745202683) had shown the path was reachable; this run supplied the first registered speed evidence for it.

## Why is adoption still open?

The comparison changed several parts of the plan together, so it does not isolate service threads as the cause. It ran on one machine, not across a class of machines. The registered no-harm bound was shown at eleven and twenty threads, but was **not shown** at twelve and sixteen: their intervals are too wide, although their point estimates and plans are nearly unchanged. An eight-thread identity check also moved despite equal plans. The Coordinator [sent those questions to the Guardian](https://github.com/lmmx/mibz2/pull/145#issuecomment-6750273151) for the separate adoption decision. Nothing has merged on the strength of this result.

## What happens next?

The [queue](https://github.com/lmmx/mibz2/blob/9e8a70ce/bench/workstation-queue.txt) now holds release scaling for version 0.2.7, then the run-fed follow-up. No release-scaling bundle was confirmed at the Coordinator's [last report](https://github.com/lmmx/mibz2/pull/145#issuecomment-6750273151), even though the later guarded service entry ran; Louis has been asked what happened. A [run-fed bundle](https://github.com/lmmx/mibz2/pull/145#issuecomment-6750297742) arrived after that report, from its frozen second attempt. Its submitted status is evidence of execution, not yet a checked scientific verdict.

## What else remains?

The [request to re-pin all four role tasks](https://github.com/lmmx/mibz2/pull/145#issuecomment-6745788393) to the amended rules is still open. Louis's [in-session confirmation](https://github.com/lmmx/mibz2/pull/145#issuecomment-6747734847) supplied the immediate authority for these two freezes but did not change the tasks' saved pins. Version [0.2.7](https://github.com/lmmx/mibz2/pull/145#issuecomment-6745555994) already released the feeder panic guard; its earlier continuous-integration comparison detected no change at that run's precision. The [supporting levels](L2.md) keep the separate work and limits explicit.
