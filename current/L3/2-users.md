# 2. Downstream delivery

- **Doing:** record wikidata-pq#226's merged 0.3.1 bump and its explicit no-gain exception.
- **In order to:** 0, deliver and measure benefits in a real consumer.
- **Known:** wikidata-pq#224 merged 0.2.9 after Louis overruled an initial close. Its split caps decoder threads at four, below the retained team threshold of six; the new team-path speedup cannot engage on that default split. Parity passed and no regression was observed in the bump, but no end-to-end gain from #359 is established ([correction](https://github.com/lmmx/mibz2/pull/145#issuecomment-6774165215), [PR](https://github.com/lmmx/wikidata-pq/pull/224)).
- **#225:** merged at d1536bb from head 6512290; bumps to 0.3.0 without changing the four-thread cap. Its decoder-only evidence is not an end-to-end pipeline result; the covered channel does not establish its benchmark verdict ([PR](https://github.com/lmmx/wikidata-pq/pull/225)).
- **#226:** merged at cd305f33 with the four-thread cap. Its pipeline benchmark found no change beyond noise and parity passed. Louis explicitly instructed this exception to wikidata-pq's end-to-end-gain rule; decoder CI gains are not a measured pipeline gain ([decision](https://github.com/lmmx/mibz2/pull/145#issuecomment-6886180866), [PR](https://github.com/lmmx/wikidata-pq/pull/226)).
- **Next:** retain the distinction between shipped decoder gains and unmeasured end-to-end benefit in the downstream pipeline.
