The new decoding policy has passed both registered workstation runs and is merged into the development branch. The team is preparing a release: [the comparison against the previous release is running](https://github.com/lmmx/mibz2/actions/runs/42310037840), alongside automated tests on the merged code. Confirmation is no longer outstanding, and no further workstation action is recorded as needed from Louis for this experiment.

## What has been established?

At ten requested threads, the new default was 8.9% faster in the first run and 8.8% faster in confirmation on Louis's ten-core workstation. In confirmation it used 11% less processor time and 19% less peak memory. [Both runs passed every registered condition](https://github.com/lmmx/mibz2/pull/145#issuecomment-6698087098), including the checks against slowdown at eleven, twelve, sixteen and twenty requested threads and the controls at eight and nine.

The policy uses three groups of three decoding workers instead of four groups of two, with twelve rather than sixteen blocks allowed in flight. It selects that arrangement on machines with ten physical cores and 20 MiB of shared cache when ten or more threads are requested. Other cases keep the existing rule.

The coordinating agent fetched the confirmation bundle, checked its checksum, and reproduced the frozen judge's verdict. It reports the same executables and input corpus by digest as the first run. The [policy was admitted and merged](https://github.com/lmmx/mibz2/pull/234) at the tested revision. This account records those checks and the merge; it has not independently recalculated the raw measurements.

## What happens next?

The coordinating agent has [dispatched the release comparison](https://github.com/lmmx/mibz2/pull/145#issuecomment-6698087098) between the previous Python release and the merged code, then plans to release once the necessary automated checks are complete. The benchmark and master-branch tests were still running when checked. The latest published release remains [version 0.2.4](https://github.com/lmmx/mibz2/releases/tag/v0.2.4), so merging the policy has not yet delivered it in a new package.

A [documentation update recording confirmation and admission](https://github.com/lmmx/mibz2/pull/243) is also open. The result is already recorded in the team discussion and the merge is complete; that documentation work does not mean the experiment is still waiting.

## What remains uncertain?

The improvement repeated on one processor and one workload. Other processors with the same core and cache sizes, untested thread counts, other inputs, custom buffering and contention remain outside the measurements. The test changes grouping and blocks in flight together; it does not isolate a shared-cache mechanism.

The first run's eight-thread control was 1.4% slower despite using the same worker arrangement. That offset did not recur: confirmation gave a ratio of 1.005 with a 95% interval of 0.992–1.019. Both runs passed the predeclared drift screen, but neither that screen nor the lack of recurrence proves equivalence or identifies the original offset's cause. [The result retains these limits](https://github.com/lmmx/mibz2/blob/83cf36d304ebf02adbb452cd011a334210191881/docs/journal/2026-11-06-three-team-policy.md#result).

## How did the confirmation reach the team?

The [automatic submission command is merged](https://github.com/lmmx/mibz2/pull/242), and confirmation was its first recorded use. Louis ran `just submit bench-three-team-policy`; it archived the bundle and [posted its identity and checksum](https://github.com/lmmx/mibz2/pull/145#issuecomment-6698069738), which the coordinating agent then verified. Louis uploaded nothing manually.

The opt-in wrapper runs the frozen recipe unchanged. Ordinary recipe runs remain local, and a failed delivery can be retried with `just submit-bundle <tarball>` without another benchmark run. Submission makes handover easier; it does not itself constitute a passing result or authorise agents to initiate workstation timing.

## What else is going on?

Independently, the [update bringing version 0.2.4 to wikidata-pq](https://github.com/lmmx/wikidata-pq/pull/219) remains open under that project's own benchmark rules; current execution is unconfirmed. It concerns the existing release, not automatic delivery of the newly merged policy. Separate investigations into scaling remain paused by the coordinating agent's earlier choice, with no restart recorded. The [supporting hierarchy](L2.md) gives the detailed evidence.
