The benchmark is ready for Louis to run on his ten-core workstation: both reviewers have cleared the repaired evidence collection, and the test's exact versions and rules are fixed. It will establish whether grouping decoding workers in threes improves the default decoder at the same requested thread counts. [The team is waiting for Louis's first run and confirmation](https://github.com/lmmx/mibz2/pull/145#issuecomment-6695807880); this needs his machine and his handover of the results, and the proposed change is still unmerged.

## Why this change?

The proposed policy uses three groups of three decoding workers when a machine has ten physical cores and 20 MiB of shared cache and the caller requests ten or more threads. Other detected topologies and lower requested counts retain the existing policy.

Two earlier workstation runs qualified both tested three-group arrangements against the existing eight-thread reference. Groups of three were 7.6% and 8.7% faster; groups of four were 9.0% and 8.6% faster. Both saved about 17–18% peak memory. The [design chose groups of three](https://github.com/lmmx/mibz2/pull/145#issuecomment-6693638662) for similar measured throughput with less processor time. That experiment did not statistically rank the two arrangements or validate the new default at equal requested counts.

## What does Louis need to do?

The coordinating agent has [prepared the frozen recipe](https://github.com/lmmx/mibz2/blob/912baf00d1d86c7beead7cc05393305bb96c8be1/docs/journal/2026-11-06-three-team-policy.md#frozen), run with `just bench-three-team-policy`, for Louis to execute and then confirm on his workstation. Results count when he hands them over in the coordinating agent's session. The journal records no run yet.

The [registered comparison](https://github.com/lmmx/mibz2/blob/912baf00d1d86c7beead7cc05393305bb96c8be1/docs/journal/2026-11-06-three-team-policy.md#the-rule) requires a measured gain of at least 3% at ten requested threads, with uncertainty excluding no gain, and checks against more than 2% slowdown at eleven, twelve, sixteen and twenty. Controls at eight and nine screen for drift. Both runs must pass; a valid negative result cannot be replaced, results cannot be pooled, and only one structurally invalid attempt may be replaced across the experiment.

Automated checks on the frozen recipe's commit are passing. The automated benchmark detected no overall change on its small runner, which does not reach the new ten-core selection. This is consistent with the [specific evidence route Louis approved](https://github.com/lmmx/mibz2/blob/912baf00d1d86c7beead7cc05393305bb96c8be1/docs/journal/decisions.md): workstation evidence for benefit, automated evidence for correctness and no regression. A successful recipe review is not a performance result.

## What did the reviews resolve?

The reviewers found that the recipe could accept missing evidence or build the wrong comparison, and that an earlier claim about observing the baseline's threads was incorrect. The [coordinating agent corrected that claim publicly](https://github.com/lmmx/mibz2/pull/145#issuecomment-6695160336). Repairs now preserve the baseline executable before building the candidate, bind the relevant build and workload inputs, observe both arms' threads, and retain truthful outcomes for failures.

The [Guardian](https://github.com/lmmx/mibz2/pull/145#issuecomment-6695760685) and [Researcher](https://github.com/lmmx/mibz2/pull/145#issuecomment-6695772196) cleared their objections within the scope they tested. Their boundary checks used simulated build tools rather than a real workstation run. The number of blocks allowed in flight is derived from the frozen code and observed plan: sixteen for the old policy, twelve for the new. That difference is intentional, and both reviewers accepted the derivation.

## What remains uncertain?

The earlier gain repeated on one machine and one workload. Benefit on other processors with the same core and cache sizes, on untested thread counts or other inputs remains unestablished. The new test changes grouping and blocks in flight together; it does not isolate a shared-cache mechanism. Observing the threads adds instrumentation whose cost has not been separated: identical observation code does not prove equal or zero disturbance.

Separate investigations into the slowdown and other ways to use the cores remain paused by the coordinating agent's earlier choice. Their restart is not a dependency of this test, and none is recorded.

## What else is going on?

The route for [sending requests from ChatGPT to the team is documented and merged](https://github.com/lmmx/mibz2/pull/238). The [return route is also merged](https://github.com/lmmx/mibz2/pull/240): the coordinating agent can leave requests for ChatGPT, which Louis can ask a chat to inspect or process. Checking in only reads; processing authorises eligible work. Only a strictly checked update of an existing task's instruction pin may be carried out without a separate review from Louis; this does not authorise arbitrary task changes.

The role writing this account is now the Observer, with the same duties, inbox, state branch and history. Its new instruction pin is active in this run, and the [coordinating agent has verified and closed the re-pin request](https://github.com/lmmx/mibz2/pull/145#issuecomment-6696702367). A [small documentation refinement to avoid repeating completed requests](https://github.com/lmmx/mibz2/pull/241) remains open.

The page's single-version loading and footer link are merged; this run has not checked the live deployment. Independently, the [update bringing version 0.2.4 to wikidata-pq](https://github.com/lmmx/wikidata-pq/pull/219) remains open under that project's own benchmark rules; current execution is unconfirmed. The [supporting hierarchy](L2.md) carries the detailed evidence.
