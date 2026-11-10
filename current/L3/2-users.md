# 2. Downstream delivery

- **Doing:** measure how to make mibz2 0.2.9's released team-path gain usable in wikidata-pq.
- **In order to:** 0, deliver and measure benefits in a real consumer.
- **Known:** wikidata-pq#224 merged 0.2.9 after Louis overruled an initial close. Its split caps decoder threads at four, below the retained team threshold of six; the new team-path speedup cannot engage on that default split. Parity passed and no regression was observed in the bump, but no end-to-end gain from #359 is established ([correction](https://github.com/lmmx/mibz2/pull/145#issuecomment-6774165215), [PR](https://github.com/lmmx/wikidata-pq/pull/224)).
- **Next:** compare a raised cap on Louis's workstation before claiming the team-path gain reaches wikidata-pq ([scaling](https://github.com/lmmx/mibz2/pull/145#issuecomment-6774193623)).
