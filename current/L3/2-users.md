# 2. Downstream delivery

- **Doing:** evaluate wikidata-pq#225's update to mibz2 0.3.0 under its own parity, CI and end-to-end benchmark rule.
- **In order to:** 0, deliver and measure benefits in a real consumer.
- **Known:** wikidata-pq#224 merged 0.2.9 after Louis overruled an initial close. Its split caps decoder threads at four, below the retained team threshold of six; the new team-path speedup cannot engage on that default split. Parity passed and no regression was observed in the bump, but no end-to-end gain from #359 is established ([correction](https://github.com/lmmx/mibz2/pull/145#issuecomment-6774165215), [PR](https://github.com/lmmx/wikidata-pq/pull/224)).
- **#225:** open at 6512290; bumps to 0.3.0 without changing the four-thread cap. Its decoder-only evidence is not an end-to-end pipeline result; parity, green CI and a beyond-noise end-to-end gain with no unacceptable regression govern its merge ([PR](https://github.com/lmmx/wikidata-pq/pull/225)).
- **Next:** retain `BZ2_THREADS_CAP=4` so the split uses Cells-enabled Workers; measure the pipeline and decide #225 by its own merge rule. No downstream gain or merge is yet established ([PR](https://github.com/lmmx/wikidata-pq/pull/225)).
