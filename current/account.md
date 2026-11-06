Version 0.2.5 is released, carrying the decoding policy that passed both workstation tests. The remaining work is to record how the released packages scale across Louis's workstation and to complete a separately benchmarked update in wikidata-pq. [The release itself is finished](https://github.com/lmmx/mibz2/pull/145#issuecomment-6698812800); the new workstation comparison is a follow-up and does not hold publication.

## What is available now?

The [GitHub release](https://github.com/lmmx/mibz2/releases/tag/v0.2.5) is published. The coordinating agent also [reports verified publication](https://github.com/lmmx/mibz2/pull/145#issuecomment-6698812800) on crates.io, PyPI with five wheels and a source distribution, and docs.rs. The policy is the release's only decoder change: on machines with ten physical cores and 20 MiB of shared cache, requests for ten or more threads use three groups of three decoding workers instead of four groups of two. Other machine shapes retain the previous selection.

In the two registered tests on Louis's workstation, ten-thread throughput improved by 8.9% and 8.8%. Confirmation used 11% less processor time and 19% less peak memory. Both runs passed the agreed checks. Before release, the [paired automated comparison against 0.2.4 completed successfully](https://github.com/lmmx/mibz2/actions/runs/42310037840), with matching output and no detected change on its small runner, where the new policy does not apply.

## What does the next workstation run answer?

The completed experiment decided whether to adopt the policy. The new run compares the actual releases across thread counts, so the History page can show behavior that a four-CPU automated runner cannot expose. It is informative, not another admission or release gate.

The [release runbook](https://github.com/lmmx/mibz2/blob/8823f617717aa8bf02e91e95f9b4b24284e388e9/docs/RELEASING.md#5-release-scaling-the-new-release-against-the-last-on-the-workstation) now asks Louis for this comparison after each release. The [updated recipe is merged](https://github.com/lmmx/mibz2/pull/244): after updating the checkout, `just submit bench-release-scaling` selects the newest release and its predecessor, currently 0.2.5 and 0.2.4, and submits the results automatically. This run is pending Louis. If he cannot provide it, the runbook says to report that; the release does not wait.

The coordinating agent will check the submitted bundle and commit its recorded results for the History page. The [release notes are also being expanded](https://github.com/lmmx/mibz2/pull/245), independently of collection.

## What is happening downstream?

The [wikidata-pq dependency update](https://github.com/lmmx/wikidata-pq/pull/219) now targets 0.2.5 rather than 0.2.4, and its new benchmark is running. Its previous 0.2.4 revision was left unmerged because no end-to-end improvement exceeded noise; that earlier run also flagged roughly 9.8% higher peak memory across all processes. Those findings belong to the previous revision, not to a completed test of the new one.

The new worker-grouping policy does not apply on that project's small automated runners. The expected benefit there still comes from the earlier improvement to move-to-front decoding, a step that reconstructs the data, which was small relative to the full pipeline in the previous test. The downstream project's own benchmark decides integration; the workstation gain in mibz2 does not automatically qualify this dependency update.

## What remains uncertain?

The policy's measured gain remains specific to one processor and workload. Other processors with the same core and cache sizes, untested thread counts, different inputs and contention are not established by those results. Grouping and blocks in flight changed together, so the tests do not isolate a cache mechanism.

The first run's eight-thread control offset did not recur in confirmation. That does not establish equivalence or explain the offset. The new release-scaling run has not yet supplied additional evidence. Separate scaling investigations remain paused by the coordinating agent's earlier choice, with no restart recorded. The [supporting hierarchy](L2.md) retains the detailed evidence and its limits.
