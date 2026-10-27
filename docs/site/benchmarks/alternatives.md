---
description: "mibz2 beside other bzip2 decoders on master's latest benchmark run: paired ratios with 95% intervals, per corpus and thread count."
hide:
  - navigation
  - toc
---

# Alternatives

<div id="alternatives-hero" data-src="../../assets/alternatives.json" markdown>
Loading the chart…
</div>

mibz2 beside other bzip2 decoders, pbz2 (Soof Golan's Rust decoder, the harness's
dev-dependency) and lbzip2 (C, `lbzip2 -dc -n T`), on master's latest recorded commit,
decoded in the same rounds on one runner ([methodology](index.md)). A ratio above 1× is
the alternative taking longer (or more memory) than mibz2: there, mibz2 is faster. These are decoder-level results; end-to-end pipeline results are in
[wikidata-pq's tracker](https://lmmx.github.io/wikidata-pq/guide/bench-history/).

--8<-- "docs/generated/alternatives.md"
