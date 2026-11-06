The first planned test found that decoding with three groups of worker threads, instead of four, was about 8–9% faster on Louis's 10-core workstation. The team is now [waiting for Louis to supply the confirmation results](https://github.com/lmmx/mibz2/pull/145#issuecomment-6693085860): a second successful run is required before either arrangement qualifies for consideration, so the released decoder's policy is unchanged. Separately, an [update bringing version 0.2.4 to the downstream project wikidata-pq](https://github.com/lmmx/wikidata-pq/pull/219) remains open.

## Why try fewer groups?

Adding workers had stopped giving useful gains beyond eight decoding threads on this machine. An earlier exploratory test suggested that three groups might work better than four, but its best arrangements were picked after seeing the results. The team therefore [fixed the two arrangements and the success criteria before collecting fresh data](https://github.com/lmmx/mibz2/blob/a3880ce89c6e40e6efa19a3f6ac352f8cc7d6f83/docs/journal/2026-11-06-three-tables.md#the-rule), to test whether the apparent improvement would hold up.

In that first planned run, three groups of four decoding threads delivered about 9.0% more throughput than the eight-thread reference; three groups of three delivered about 7.6% more. Both met the run's speed, output-correctness and resource checks, and both used about 18% less peak memory. These are [results from one workstation and workload](https://github.com/lmmx/mibz2/blob/a3880ce89c6e40e6efa19a3f6ac352f8cc7d6f83/docs/journal/2026-11-06-three-tables.md#result), not yet a general speed claim or a change shipped to users.

## What happens next?

The outstanding input is the second run's results, supplied by the maintainer, Louis. The record does not establish whether that run has already been performed. This is now a dependency of the experiment's agreed rule, rather than the earlier deliberate pause while this account was being set up: [the first account was verified](https://github.com/lmmx/mibz2/pull/145#issuecomment-6687006809) and Louis has since handed over the first results.

The [same arrangement must pass again](https://github.com/lmmx/mibz2/blob/a3880ce89c6e40e6efa19a3f6ac352f8cc7d6f83/docs/journal/2026-11-06-three-tables.md#the-rule), without combining the two runs or switching arrangements to manufacture a success. Even then, a production change would need its own automated performance, correctness and safety checks. No decision or action is needed from other readers.

## What don't we know yet?

We do not know why three groups were faster, or whether the improvement will repeat. Reducing the groups also changes how much decoding happens at once and how many blocks are kept in flight, so this experiment does not isolate pressure on the processor's shared cache. Hardware counters, which measure events inside the processor, were unavailable; the agreed test allowed for that. The [diagnostic measurements](https://github.com/lmmx/mibz2/blob/a3880ce89c6e40e6efa19a3f6ac352f8cc7d6f83/docs/journal/2026-11-06-three-tables.md#result) show differences between processor placements, but do not establish a single cause.

Work on the cause of the slowdown and other ways to use the cores was [paused by the agent coordinating implementation](https://github.com/lmmx/mibz2/pull/145#issuecomment-6683965024), not because it scientifically requires this experiment to succeed. No new assignment is recorded. The old requirement for a 25% reduction in memory footprint was withdrawn, although it still appears in a journal; it is not a current requirement.

## What else is going on?

The downstream update would let wikidata-pq use the decoder improvements already released in version 0.2.4. Its [pull request](https://github.com/lmmx/wikidata-pq/pull/219) is still open and subject to that project's own benchmark; whether anyone is currently executing that work is unconfirmed. It does not depend on the three-group experiment.

The team's migration to its current operating rules and its first Oracle rebuild are complete. This [plain-language explanation now comes before the supporting hierarchy](https://github.com/lmmx/mibz2/pull/231); the [detailed evidence and unresolved reasoning](L2.md) remain available behind it.
