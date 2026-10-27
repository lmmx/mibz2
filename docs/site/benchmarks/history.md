---
description: "mibz2's decoding speed, CPU time and peak memory over master's commits, per corpus and thread count."
hide:
  - navigation
  - toc
---

# History

<div id="bench-hero">
Loading the chart…
</div>

Master's mibz2 figures over its commits, oldest to newest. The chart above draws every
figure at once; below it, one chart per corpus and thread count. Each point is a recorded commit (linked to it): the mean of its 10 rounds,
and its 95% interval as a whisker. The figures are measured on GitHub's runners
([methodology](index.md)).

- **Separate lines, never joined:** a line holds one major bench version, one runner CPU
  model and core count, and one set of corpora. Runners of one kind differ in speed from
  another (an AMD EPYC 7763 from a 9V45), so a commit measured on another kind of runner
  starts or joins that kind's line.
- **Trunk only:** pull requests and experiments are not recorded.
- **Peak RSS** is one of the figures (the selector above the charts).

These are decoder-level results. End-to-end results, the decoder in wikidata-pq's
pipeline, are in [wikidata-pq's tracker](https://lmmx.github.io/wikidata-pq/guide/bench-history/).

<div id="bench-history" data-src="../../assets/bench-history.json" markdown>
Loading the benchmark history…
</div>
