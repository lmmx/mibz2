The proposed decoding policy passed its first registered test on Louis's ten-core workstation: at ten requested threads it was 8.9% faster, used 12% less processor time and needed 18% less peak memory. [The coordinating agent verified and recorded the result](https://github.com/lmmx/mibz2/pull/145#issuecomment-6697537827); a second, confirming run must also pass before the change can be adopted. The next planned step is to finish the automatic evidence-submission command, bring it into the experimental branch, and have Louis use it for confirmation.

## What changed in the test?

The candidate uses three groups of three decoding workers instead of four groups of two, with twelve rather than sixteen blocks allowed in flight. It selects that arrangement on machines with ten physical cores and 20 MiB of shared cache when ten or more threads are requested. Other cases retain the old policy.

The new default was compared with the old at the same requested thread count. At ten, the throughput ratio was 1.089, with a 95% interval of 1.075–1.103. All four checks against slowdown at eleven, twelve, sixteen and twenty threads passed, as did both controls. This is the coordinating agent's recorded analysis using the frozen judge, not an independent recalculation by this account.

## Why is confirmation still needed?

The [rule fixed before collection](https://github.com/lmmx/mibz2/blob/912baf00d1d86c7beead7cc05393305bb96c8be1/docs/journal/2026-11-06-three-team-policy.md#the-rule) requires both the initial run and confirmation to pass. A valid failure cannot be replaced, results cannot be pooled, and only one structurally invalid attempt may be replaced across the experiment. One success therefore leaves the policy unqualified and unmerged.

The eight-thread control used the same worker arrangement in both builds but was 1.4% slower in the candidate, with an interval excluding no difference. It passed the registered drift screen because that interval was not wholly outside the permitted band. This is a real caveat, not evidence that the two builds are equivalent. The coordinating agent suggests a build or layout effect as a possible explanation; its cause has not been established.

The gain is still evidence about one machine and workload. Confirmation will not establish performance on other processors with the same core and cache sizes, untested counts or other inputs. Nor does changing grouping and blocks in flight together isolate a shared-cache mechanism.

## How will the next result reach the team?

Louis asked to remove manual uploads and repeated descriptions of which run he had made. His [approval recorded in the coordinating agent's session](https://github.com/lmmx/mibz2/blob/2d3a53e30e47774dffc5ece897fa054acdede37c/docs/journal/decisions.md) allows his own execution of a frozen recipe with an explicit submission command to count as handing over its results. The run's identity comes from its bundle; the freeze, provenance checks, evidence criteria and prohibition on agents initiating workstation timing stay unchanged.

The [implementation is still open](https://github.com/lmmx/mibz2/pull/242). Its intended command is `just submit bench-three-team-policy`: run the frozen recipe unchanged, archive its bundle at an exact commit and post its identity and checksum to the team discussion. Ordinary recipe runs remain local. If submission fails, `just submit-bundle <tarball>` retries delivery without another benchmark run. The command is deliberately named “submit” to distinguish it from publishing a software package.

The coordinating agent has chosen to [integrate this delivery improvement before requesting confirmation](https://github.com/lmmx/mibz2/pull/145#issuecomment-6697537827). That sequencing is a deliberate choice; automatic delivery is not a scientific prerequisite of the comparison. The first run was already handed over in session and [archived](https://github.com/lmmx/mibz2/blob/820f4a87ee8f2ce43744350ef15462bed448c3bc/three-team-policy/20261106T220546Z.tar.gz). Approval of the delivery method is separate from completion of its implementation.

## What else is going on?

The documentation preventing repeated task instruction updates is [merged](https://github.com/lmmx/mibz2/pull/241). The Observer rename and both directions of the ChatGPT request relay are complete; this account's existing pinned instructions and state branch remain in use.

Independently, the [update bringing version 0.2.4 to wikidata-pq](https://github.com/lmmx/wikidata-pq/pull/219) remains open under that project's own benchmark rules; current execution is unconfirmed. Separate investigations into scaling remain paused by the coordinating agent's earlier choice, with no restart recorded. The [supporting hierarchy](L2.md) gives the detailed evidence.
