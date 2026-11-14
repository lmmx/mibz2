# 2. Downstream delivery

- **Doing:** track open wikidata-pq#226's 0.3.1 bump at cap four under its own benchmark and merge rule.
- **In order to:** 0, deliver and measure benefits in a real consumer.
- **Known:** wikidata-pq#224 merged 0.2.9 after Louis overruled an initial close. Its split caps decoder threads at four, below the retained team threshold of six; the new team-path speedup cannot engage on that default split. Parity passed and no regression was observed in the bump, but no end-to-end gain from #359 is established ([correction](https://github.com/lmmx/mibz2/pull/145#issuecomment-6774165215), [PR](https://github.com/lmmx/wikidata-pq/pull/224)).
- **#225:** merged at d1536bb from head 6512290; bumps to 0.3.0 without changing the four-thread cap. Its decoder-only evidence is not an end-to-end pipeline result; the covered channel does not establish its benchmark verdict ([PR](https://github.com/lmmx/wikidata-pq/pull/225)).
- **#226:** open at db28b33a; proposes mibz2 0.3.1 with the four-thread cap. Decoder release CI is not an end-to-end pipeline verdict ([PR](https://github.com/lmmx/wikidata-pq/pull/226)).
- **Next:** retain `BZ2_THREADS_CAP=4` so the split uses Cells-enabled Workers; seek the downstream benchmark record before claiming an end-to-end gain.
