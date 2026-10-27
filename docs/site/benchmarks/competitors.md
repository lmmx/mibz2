---
description: "mibz2 against pbz2 and lbzip2 on master's latest benchmark run: paired ratios with 95% intervals, per corpus and thread count."
---

# Against other decoders

mibz2 against pbz2 (Soof Golan's Rust decoder, the harness's dev-dependency) and lbzip2
(C, `lbzip2 -dc -n T`) on master's latest recorded commit, decoded in the same rounds on
one runner ([methodology](index.md)). A ratio above 1 is the other decoder taking longer
than mibz2. These are decoder-level results; end-to-end pipeline results are in
[wikidata-pq's tracker](https://lmmx.github.io/wikidata-pq/guide/bench-history/).

--8<-- "docs/generated/competitors.md"
