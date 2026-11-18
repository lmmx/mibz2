mibz2 0.5.6 is published with a new one-pass way to write decoded blocks from its Cells grammar. This is the first production gain from the team's mathematical investigation of that grammar: paired tests found faster Wikidata decoding at one and two threads, while a release check found no slower workload. Louis has started a release-scaling run on his ten-core workstation; its high-thread result is not yet known. A downstream Wikidata pipeline update is in progress.

## What changed in the decoder?

The [one-scan writer](https://github.com/lmmx/mibz2/pull/581) uses a verified ownership rule so each output part can be placed in one pass. The mathematical result was checked independently before implementation. Paired benchmarks measured 4.8% less Wikidata wall time at one thread and 3.5% less at two. The four-thread wall difference was within noise. The Guardian initially [held it for safety evidence](https://github.com/lmmx/mibz2/pull/145#issuecomment-6988584322), then [lifted the hold](https://github.com/lmmx/mibz2/pull/145#issuecomment-6988977627) after Miri, mutation receipts and source identity were checked. It merged and shipped in [0.5.6](https://github.com/lmmx/mibz2/pull/145#issuecomment-6990622941).

The release's separate four-core comparison against 0.5.5 found no slower row and measured Wikidata gains at two and four threads. The tag's workstation scaling run has started, and the crossover is queued. Neither result is covered yet, so the writer's effect at eight to twenty threads is unestablished.

## Why did the high-thread direction change?

The [0.5.4 tag checks](https://github.com/lmmx/mibz2/pull/145#issuecomment-6952656453) confirmed its pinned worker reductions, with no cooperative team beating the shipped plan. [Version 0.5.5](https://github.com/lmmx/mibz2/pull/145#issuecomment-6976975591) then let the smaller-memory worker method admit more blocks. On the measured i9, that cut main-memory traffic per output byte by about a quarter and made Wikidata decoding 12–16% faster in the adoption experiment, at about 25 MB more peak memory. The Coordinator accepted that memory cost under the performance policy. Tag [scaling](https://github.com/lmmx/mibz2/pull/145#issuecomment-6977328773) found five to thirteen percent faster wall time at eight to twenty threads.

A registered [worker-count recalibration](https://github.com/lmmx/mibz2/pull/145#issuecomment-6978881191) kept the shipped settings. Pinned low-thread choices held, but high-thread comparisons were too noisy to decide. A subsequent sustained-load run was [void under its clock screen](https://github.com/lmmx/mibz2/pull/145#issuecomment-6986593076). The apparent slow-clock cause was later [corrected](https://github.com/lmmx/mibz2/pull/145#issuecomment-6986860998): slow observations followed a short idle gap and carried a fixed startup penalty, not a sustained power clamp. Timed decodes now avoid short gaps. No worker-count change was released from these runs.

A [direct memory-controller measurement](https://github.com/lmmx/mibz2/pull/145#issuecomment-6963745901) put the seven-worker plateau at about 75% of measured bandwidth. This is an inconclusive result under its registered saturation rule. Memory traffic remains a useful lead, as the wider admission's measured gain shows; a hard bandwidth ceiling has not been proved.

## What else is being tested?

A second verified writing rule, placing output at first occurrence, was implemented byte-exactly but [ran about 1.79 times slower](https://github.com/lmmx/mibz2/pull/145#issuecomment-6989495656) than the one-scan writer on Wikidata. It stopped. An [endpoint-sweep study](https://github.com/lmmx/mibz2/pull/591) reconstructs the induction without stored widths, but its ordering cost fails its practical screen; independent verification is pending. An [open-return comparison](https://github.com/lmmx/mibz2/pull/145#issuecomment-6990634441) awaits research review of its diagnostic registration. None is a second production gain yet.

## What reaches users next?

[Version 0.5.6](https://github.com/lmmx/mibz2/releases/tag/v0.5.6) is available. The Coordinator reports a [wikidata-pq update](https://github.com/lmmx/wikidata-pq/pull/227) toward 0.5.6 in progress; at inspection its open PR still named 0.5.2. Its whole-pipeline benefit has not been measured here. No action from Louis is needed for the published decoder; the pending tag and downstream measurements are the next evidence.
