# 2. Downstream delivery

- **Status:** 0.2.5 dependency update in wikidata-pq #219 completed by Louis's one-off call, without an end-to-end automated-runner gain. Its ordinary merge rule was not amended ([decision](https://github.com/lmmx/wikidata-pq/pull/97#issuecomment-6699288552)).
- **In order to:** 0, deliver released decoder improvements through a real consumer.
- **New limit:** wikidata-pq caps the decoder at four threads in its default pipeline, so neither the 0.2.5 three-team policy nor the 0.2.6 pipelined plan engages there by default ([Coordinator LOG](https://github.com/lmmx/mibz2/pull/145#issuecomment-6699649478)). The extrapolated upstream gain is not a measured end-to-end pipeline gain.
- **Further work:** #220 adds a workstation-shaped benchmark as measurement-only; a matching integration rule still requires Louis. #221 proposed a 0.2.6 bump; its current merge/benchmark disposition was not established by this record. Do not treat either as automatic downstream adoption.
