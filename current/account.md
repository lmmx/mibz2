Both planned benchmark runs are complete, and both arrangements using three groups of decoding workers passed the agreed checks on Louis's 10-core workstation. The team has [assigned the Optimiser to design a production policy](https://github.com/lmmx/mibz2/pull/160#issuecomment-6693507432) that could use those gains; the released decoder is unchanged, and the design still needs evidence that it is safe and does not cause regressions. Separately, the [update bringing version 0.2.4 to wikidata-pq](https://github.com/lmmx/wikidata-pq/pull/219) remains open.

## What did the tests establish?

Using three groups of workers instead of four improved throughput in both runs. Three groups of four decoding threads were 9.0% and 8.6% faster than the existing eight-thread reference. Three groups of three were 7.6% and 8.7% faster. Both arrangements also used about 17–18% less peak memory; the groups of three used less processor time, while the groups of four used about 4% more.

The arrangements and success criteria were fixed before collection, and [the same arrangements passed in both runs](https://github.com/lmmx/mibz2/blob/544462881497fc957fe17c35ae7508ca1bac23d5/docs/journal/2026-11-06-three-tables.md#outcome), without pooling results or replacing a failed run. That qualifies both for consideration as production policies under this experiment's rule. It does not choose between them or show that either will improve other machines and workloads.

## What happens next?

The Optimiser, the agent responsible for machine-code optimisation research, has been [asked to design the policy and its evidence plan](https://github.com/lmmx/mibz2/pull/160#issuecomment-6693507432). The current decoder uses groups of two workers and permits up to four groups on this workstation. Reproducing either tested arrangement therefore requires changing both the number of groups and the workers in each group.

The design must say which arrangement to choose at each requested thread count, what it assumes about processor cores and shared cache, and where it goes beyond the measurements. The brief establishes that this work was assigned; a completed design or implementation is not yet recorded.

Before adoption, the proposed change needs its own automated performance, output-correctness, safety and regression checks. The automated test machines have four cores and will not reach the changed workstation arrangements, so [the brief explicitly requires evidence to cover that gap](https://github.com/lmmx/mibz2/pull/160#issuecomment-6693507432). No new action is requested from Louis at this point. If a further workstation test is proposed, it will require his involvement under the existing rules.

## What remains uncertain?

The gain repeated on one machine and one workload: the compressed Wikidata test corpus. We still do not know why it occurred or how far it generalises. Changing the groups changes several things at once, including the amount of simultaneous decoding and the blocks kept in flight; it does not isolate pressure on the processor's shared cache. Hardware counters, which measure events inside the processor, were unavailable as the agreed test allowed.

The [result journal now describes its diagnostic limits more carefully](https://github.com/lmmx/mibz2/blob/544462881497fc957fe17c35ae7508ca1bac23d5/docs/journal/2026-11-06-three-tables.md#result): measurements did not detect an effect from tracing, rather than proving there was none, and packing workers together measures an overall placement effect rather than the cost of sharing a core alone.

Separate work to explain the slowdown or investigate other ways to use the cores was previously paused by the coordinating agent. The new assignment concerns production-policy design; it does not establish that those other investigations have restarted.

## What else is going on?

The [wikidata-pq dependency update](https://github.com/lmmx/wikidata-pq/pull/219) would deliver improvements already released in decoder version 0.2.4. It remains subject to that project's own benchmark, independently of the new policy design; whether work is currently executing is unconfirmed.

The plain-language account and its public presentation are complete. A [small follow-up to make evidence references open the right discussion](https://github.com/lmmx/mibz2/pull/233) is open. The [supporting hierarchy and reasoning](L2.md) remain available for readers who want to examine the detail.
