Version 0.2.5 is released and adopted by wikidata-pq, and the team has resumed work on why the decoder stops getting faster beyond eight to ten threads. The next idea is to overlap a worker group's preparation and finishing work with its decoding, so fewer workers sit idle. The coordinating agent reports a prototype under way; the Researcher is checking its reasoning before a test plan is frozen, and Louis has ruled out new workstation runs tonight.

## Why try overlapping the work?

The released change uses three groups of decoding workers instead of four on the target ten-core workstation. It passed both registered tests, improving ten-thread throughput by about 8.8–8.9%. The [release comparison is now recorded](https://github.com/lmmx/mibz2/tree/853dd1af32fd1df3a8132a0452d7fa5a2f0ee8c0/bench/release-scaling/2026-11-06-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.2.5), and the [new analysis](https://github.com/lmmx/mibz2/blob/853dd1af32fd1df3a8132a0452d7fa5a2f0ee8c0/docs/journal/2026-11-07-knee-mechanism.md) reports that 0.2.4 took 11.9% longer at ten threads and 8.1–8.7% longer at twelve to twenty.

That establishes a gain, not its cause. The coordinating agent's explanation is that fewer simultaneously active decoding tables compete less for the processor's shared cache. Its analysis of existing traces also attributes about a third of each group's time to work performed by the leader alone, while helpers wait. Overlapping that work could save time—but could also keep more tables active at once and increase competition for the cache.

## What happens next?

The [Researcher's assignment](https://github.com/lmmx/mibz2/pull/162#issuecomment-6699480548) tests the duty-cycle calculation, the claim that memory limits the parallel phase, the predicted gain and the proposed ways to distinguish outcomes. The coordinating agent predicts 10–30% more throughput for one pipelined arrangement; that is a prediction under review, not a measured result.

[Prototype work is reported](https://github.com/lmmx/mibz2/pull/145#issuecomment-6699486962) as opt-in, with the default decoder unchanged. Its named branch was not available through GitHub when checked, so implementation and test completion are not independently confirmed here. Review and preparation can proceed without Louis. A later frozen workstation test is intended to assess overlap and worker placement; none is requested tonight.

The other ranked ideas are placing groups on distinct physical cores, making the decoding tables smaller, and reducing other memory traffic. This is the coordinating agent's current ranking, not evidence that those routes have passed tests.

## What changed downstream?

[The wikidata-pq update merged](https://github.com/lmmx/wikidata-pq/pull/219). Louis's [recorded decision](https://github.com/lmmx/wikidata-pq/pull/97#issuecomment-6699288552) explicitly permits this one update without an end-to-end gain on the automated runner: output parity passed, and the ten-core policy cannot apply on that four-core machine. The general merge rule remains unchanged.

A workstation benchmark shaped like a real downstream release, with a matching rule proposal, is queued for Louis's approval. It is not an approved replacement for the existing rule, and the upstream decoder gain is not a measurement of the whole downstream pipeline.

## What remains uncertain?

The cache explanation, the attribution of time outside active tables to serial work, and the predicted benefit of overlap are being examined. The existing measurements come from different experiment boundaries; they do not by themselves isolate the cause of the full decoder's gain. The [supporting record](L2.md) keeps those limits separate from the completed release and downstream merge.
