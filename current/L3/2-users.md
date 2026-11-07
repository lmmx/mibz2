# 2. Deliver the current decoder downstream

- **Doing/status:** dependency update done as intended: wikidata-pq #219 merged 2026-11-07 00:08:02 UTC, head 93240f94d13bd15cd1c261525725596f2a45cc11, merge 520bb37851d04ab95313746a006454278de3e4b8.
- **In order to:** 0, deliver released decoder improvements to its consumer.
- **Decision:** [6699288552](https://github.com/lmmx/wikidata-pq/pull/97#issuecomment-6699288552) records Louis's one-off call to bump 0.2.2→0.2.5 without an end-to-end CI gain. Coordinator reports parity/no regression. The ten-core policy cannot apply on the four-core runner; extrapolated benefit is not a measured downstream pipeline gain.
- **Rule unchanged:** the recorded exception does not replace the ordinary merge rule. Earlier 0.2.4 no-gain and memory-rise findings remain scoped to head 66544d54; [prior detail](https://github.com/lmmx/mibz2/blob/bd440eac0c946aa3026ff13747984d9291eee25c/current/L3/2-users.md).
- **Next proposed step:** release-shaped i9 split benchmark, opt-in per PR with a matching rule proposal, in order to evaluate future workstation-specific changes. Queued pending Louis's approval, not an approved criterion or an assigned run.
