# History

Loading the chart…

How mibz2's decoding speed, CPU time and peak memory have changed, merge by merge. The figures are measured on GitHub's runners ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-217/benchmarks/index.md>)).

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

## Release scaling, past 4 threads

CI's runners have 4 vCPUs on 2 physical cores, so the charts above stop at 4 threads. A release-scaling run (`just bench-release-scaling`) fills in the rest. It runs two releases and lbzip2 on one machine of a developer's, with the same harness and paired rounds, from 1 thread to every CPU the machine has. When one is recorded, the chart at the top offers it as the **Release scaling** view.

These runs inform; they don't decide. Nothing merges or releases on a release-scaling figure: pull requests are measured on CI, and pass without any run here. Each run is kept as it was recorded, with its machine, its date and its raw records, in [`bench/release-scaling/`](<https://github.com/lmmx/mibz2/tree/master/bench/release-scaling>). Runs on different machines are never compared or combined.

Each table gives an arm's median wall time. It also gives the geometric mean of the rounds' paired wall ratios against the reference arm, with its 95% interval: a minus sign is faster. Up to the machine's physical core count, each thread runs on a core of its own (`--spread`); above it, threads share cores.

No release-scaling run is recorded yet.

To make a run, from a checkout of mibz2 with lbzip2 installed:

```sh
just bench-release-scaling                                    # v0.2.4 against v0.2.1, every CPU
just bench-release-scaling v0.2.4 v0.2.1 10 "1,4,10,20" big.bz2   # releases, rounds, threads, more files
```

It builds both releases in `target/release-scaling/`, runs the rounds, and records the run in `bench/release-scaling/<date>-<machine>/`, to commit in a pull request.
