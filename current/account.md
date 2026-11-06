The team has implemented a proposed change to how decoding workers are grouped, but its benchmark is waiting for two reviewers to check repairs to the evidence collection before the test is fixed and run. The Researcher and Guardian, who check the experiment and its safety evidence, found that the recipe could accept missing or mismatched evidence; [repairs are posted and both have been asked to review them again](https://github.com/lmmx/mibz2/pull/145#issuecomment-6694798888). This is a real dependency for trusting the test, and the released decoder is unchanged.

## What would change?

On machines with ten physical cores and 20 MiB of shared cache, a request for ten or more threads would use three groups of three decoding workers. Other detected topologies and lower requested counts retain the existing policy. The [implementation is open for review](https://github.com/lmmx/mibz2/pull/234); it has not been adopted.

The choice follows two completed workstation runs in which both tested arrangements of three groups passed their agreed checks. Groups of three were 7.6% and 8.7% faster than the eight-thread reference; groups of four were 9.0% and 8.6% faster. Both saved about 17–18% peak memory. The [design selects groups of three](https://github.com/lmmx/mibz2/pull/145#issuecomment-6693638662) for similar measured throughput with less processor time, without treating the experiment as a statistical ranking of the two.

## What happens next?

Once both reviewers accept the repairs, the coordinating agent can fix the exact candidate and comparison versions. Louis then needs to run the prepared comparison on his workstation, followed by a confirmation. [Neither version is fixed and no run is recorded yet](https://github.com/lmmx/mibz2/blob/7c341a875049f8c95d27448046eb4923387687cf/docs/journal/2026-11-06-three-team-policy.md#frozen). The wait is for review first, then for access to Louis's machine; it is not the earlier deliberate pause for this account.

Louis has [approved a specific evidence route for this proposed policy](https://github.com/lmmx/mibz2/blob/7c341a875049f8c95d27448046eb4923387687cf/docs/journal/decisions.md): demonstrate benefit on his workstation, while automated checks establish correctness and no regression. The new default is compared with the old default at the same requested thread count. Its measured gain must be at least 3% at ten threads, with uncertainty excluding no improvement, and meet the registered limits against slowdown at eleven, twelve, sixteen and twenty. The earlier successful tests used an eight-thread reference and do not answer this new question.

## What remains uncertain?

The previous gain repeated on one machine and one workload. It does not establish the benefit on other processors with the same core and cache sizes, at every higher thread count, or on other inputs. Changing group size changes concurrency and the blocks kept in flight together; it does not isolate a shared-cache explanation.

The repaired recipe records more of the running threads, but the number of blocks allowed in flight is still inferred from the frozen code and observed plan, not measured directly. [Whether this evidence is sufficient remains part of the re-review](https://github.com/lmmx/mibz2/pull/145#issuecomment-6694798888). Posted repairs are not an acceptance or a benchmark result. Separate investigations into the slowdown and other ways to use the cores remain paused by the coordinating agent's earlier choice; no restart is recorded.

## What else is going on?

Louis can now [send routine work from ChatGPT to the coordinating agent through the project discussion](https://github.com/lmmx/mibz2/pull/235). His words and ChatGPT's interpretation stay separate, and decisions reserved to him still require his authenticated session. The first request, a footer link to Behind the scenes, is [merged](https://github.com/lmmx/mibz2/pull/145#issuecomment-6694836346); its live availability depends on the site's deployment, which this account has not verified. A [guide for finding this route in a fresh chat](https://github.com/lmmx/mibz2/pull/238) remains open.

The page now [loads one saved version or shows it as unavailable](https://github.com/lmmx/mibz2/pull/236), instead of risking a mixture of versions when GitHub's service fails. Guessed discussion links have been removed. A [rename of the role that writes this account is being prepared](https://github.com/lmmx/mibz2/pull/145#issuecomment-6694853843); it preserves the role's duties and history and has not changed this run's pinned instructions.

Independently, the [update bringing version 0.2.4 to wikidata-pq](https://github.com/lmmx/wikidata-pq/pull/219) remains open under that project's own benchmark rules. Current execution is unconfirmed. The [supporting hierarchy](L2.md) gives the evidence and reasoning behind this account.
