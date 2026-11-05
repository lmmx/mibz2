# History

Loading the chart…

How mibz2's decoding speed, CPU time and peak memory have changed, merge by merge. The figures are measured on GitHub's runners ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-212/benchmarks/index.md>)).

- **The chart chains each merge's own benchmark.** A pull request's benchmark runs its head against its base on one runner, both builds, for 10 rounds with their order alternating. Each merge keeps that paired change ([`bench/links.py`](<https://github.com/lmmx/mibz2/blob/master/bench/links.py>)), and the chart multiplies the changes, from the first merge whose arms ran in balanced order (bench 3.0.0). The band is the product's 95% interval: the links' intervals added in quadrature, on the log scale.
- **A merge that left the decoder alone** (the harness, docs, CI) is an A/A pair. It is drawn hollow and adds nothing.
- **A decoder change with no paired run** breaks the chain into a new segment. No gap is bridged.
- **What the chain assumes:** that a relative change measured on one runner holds on another (the merges ran on several CPU models, each named), and that the links' errors are independent.
- **Before bench 3.0.0** the head ran first in every round, which favoured it by about 1.8% at 1 thread in an A/A run. Those merges' changes are listed in the table below the charts, not chained.
- **`text`** is generated from mibz2's own sources, so its content changes from merge to merge, and its chain multiplies ratios measured on different files.

These are decoder-level results. End-to-end results, the decoder in wikidata-pq's pipeline, are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

## Absolute figures, one runner at a time

Master's own records: each commit measured once, on whichever runner its push drew, the mean of its 10 rounds with its 95% interval as a whisker.

- **One runner at a time.** Runners differ by up to 2× in speed, so a chart shows one runner CPU model, by default the one with the most records. Its consecutive points are that runner's consecutive records; the commits between them ran elsewhere, and their points are under their own runner.
- **A line never joins** another bench meaning (a version that redefined these figures; [the benchmark's changelog](<https://github.com/lmmx/mibz2/blob/master/bench/CHANGELOG.md>)) or another content of a corpus.
- **All runners together** are an option, for the figures themselves. They are not a trajectory.

Loading the benchmark history…
