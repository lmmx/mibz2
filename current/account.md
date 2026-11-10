A valid workstation test found that an idle gap before decoding puts Louis's machine into its slow mode: 26 of 30 runs were slow after a quarter-second gap, against none of 30 without it. The team is checking whether earlier performance comparisons exposed their arms to gaps equally. Louis has directed the team to finish the strongest paths to a measured production gain, with the failed run-record decoder's rescue first and guarded service second. Neither gain has been qualified for production.

## What did the slow-mode test settle?

The [registered test](https://github.com/lmmx/mibz2/pull/145#issuecomment-6763508177) was valid and independently recounted from its archive. Slow runs used roughly twice the processor time per block, consistent with the same decoder work running at a lower speed. The gap is a demonstrated trigger in this test; the specific clock or power mechanism is not identified. The secondary probes had no slow runs, so they could not distinguish the proposed mechanisms. Future workstation tests must state how they handle gaps. Earlier judged results are not reopened by this test, but the team is checking whether gap exposure was balanced in the guarded-service comparison.

## What is being finished first?

Louis [prioritised integration](https://github.com/lmmx/mibz2/pull/145#issuecomment-6762859780). The run-record decoder previously gained 23–26% wall time on text and 13–16% on Wikidata in CI, but harmed fallback inputs and failed its own no-harm rule. A proposed placement diagnostic would test one possible cause of that harm before a corrected successor is registered. Its design is still under Researcher and Guardian revision; a positive result would not prove the historical cause, and a null result would only bound this placement intervention. The Optimiser has not supported a production layout contract on present evidence. No successor is qualified or merged.

The guarded-service candidate previously gained 6.8% throughput on the ten-thread workstation target, but its old adoption was refused. The Researcher identified a minimum programme of twelve independent complete runner dispatches, each with 240 rounds, for a new qualification attempt. The judge and registration have been implemented in draft, but a master merge conflict leaves CI unavailable, and rare large neutral contrasts threaten the proposed normal-theory bound in some cells. Its statistical and admission review remains open. The new gap finding also requires the earlier target data's gap balance to be checked.

## What else is open?

The cooperative decoder's three ports reduced counted instructions but failed their individual measurement conditions. One combined diagnostic is under review to ask what the stacked ports cost together and whether they execute. It cannot turn the earlier failures into passes or qualify production. A roster-check repair merged, and the feature-gated path instrument merged earlier; their CI evidence is not a measured speed gain.

A smaller-table and worker-feed comparison remains a separate possible direction. Its record-width experiment has been drafted but its build is deferred while the integration candidates reach decisive gates. The old load-law speed estimates remain withdrawn.

The missing version 0.2.7 release-scaling result now has an identified cause: the workstation's tag fetch collided with a moved ready tag and exited before measuring. A [tooling fix merged](https://github.com/lmmx/mibz2/pull/342); the queued older recipe still needs its tags refreshed before it can run. No release-scaling bundle was confirmed at this account's watermark.
