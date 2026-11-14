---
description: "mibz2's decoding speed, CPU time and peak memory, merge by merge: each merged pull request's own paired benchmark, chained, and master's absolute figures on one runner at a time."
hide:
  - navigation
  - toc
---

# History

<div id="bench-hero" data-ws="../../assets/release-scaling.json">
Loading the chart…
</div>

How mibz2's decoding speed, CPU time and peak memory have changed, merge by merge. The
figures are measured on GitHub's runners ([methodology](index.md)).

- **The chart chains each merge's own benchmark.** A pull request's benchmark runs its
  head against its base on one runner, both builds, for 10 rounds with their order
  alternating. Each merge keeps that paired change ([`bench/links.py`](https://github.com/lmmx/mibz2/blob/master/bench/links.py)),
  and the chart multiplies the changes, from the first merge whose arms ran in balanced
  order (bench 3.0.0). The band is the product's 95% interval: the links' intervals added
  in quadrature, on the log scale.
- **A merge that left the decoder alone** (the harness, docs, CI) is an A/A pair. It is
  drawn hollow and adds nothing.
- **A decoder change with no paired run** breaks the chain into a new segment. No gap is
  bridged.
- **What the chain assumes:** that a relative change measured on one runner holds on
  another (the merges ran on several CPU models, each named), and that the links' errors
  are independent.
- **Before bench 3.0.0** the head ran first in every round, which favoured it by about
  1.8% at 1 thread in an A/A run. Those merges' changes are listed in the table below the
  charts, not chained.
- **`text`** is generated from mibz2's own sources, so its content changes from merge to
  merge, and its chain multiplies ratios measured on different files.

These are decoder-level results. End-to-end results, the decoder in wikidata-pq's
pipeline, are in [wikidata-pq's tracker](https://lmmx.github.io/wikidata-pq/guide/bench-history/).

## Absolute figures, one runner at a time

Master's own records: each commit measured once, on whichever runner its push drew, the
mean of its 10 rounds with its 95% interval as a whisker.

- **One runner at a time.** Runners differ by up to 2× in speed, so a chart shows one
  runner CPU model, by default the one with the most records. Its consecutive points are
  that runner's consecutive records; the commits between them ran elsewhere, and their
  points are under their own runner.
- **A line never joins** another bench meaning (a version that redefined these figures;
  [the benchmark's changelog](https://github.com/lmmx/mibz2/blob/master/bench/CHANGELOG.md))
  or another content of a corpus.
- **All runners together** are an option, for the figures themselves. They are not a
  trajectory.

<div id="bench-history" data-src="../../assets/bench-history.json" markdown>
Loading the benchmark history…
</div>

## Release scaling, past 4 threads

CI's runners have 4 vCPUs on 2 physical cores, so the charts above stop at 4 threads. A
release-scaling run (`just bench-release-scaling`) fills in the rest. It runs two releases
and lbzip2 on one machine of a developer's, with the same harness and paired rounds, from 1
thread to every CPU the machine has. When one is recorded, the chart at the top offers it as
the **Release scaling** view.

These runs inform; they don't decide. Nothing merges or releases on a release-scaling
figure: pull requests are measured on CI, and pass without any run here. Each run is kept as
it was recorded, with its machine, its date and its raw records, in
[`bench/release-scaling/`](https://github.com/lmmx/mibz2/tree/master/bench/release-scaling).
Runs on different machines are never compared or combined.

**All releases**, the view's first choice, puts every release against every other without
re-running the old ones. It is inferred from one machine's pairwise runs; no single run had
every release in it. lbzip2, the same build in every run, is a fixed anchor:

- **Pooling.** At each thread count, each release's figure is its wall time relative to
  lbzip2's. That is the mean over a run's rounds of the paired log ratio. A release measured
  in two runs, once as the new release and once as the old, gets the inverse-variance
  weighted mean of the two.
- **Any pair** is the ratio of the two figures. Its interval adds their variances, which
  is cautious for two releases from the same run, since their rounds share lbzip2's.
- **The curves** apply each release's figure to lbzip2's measured throughput in the newest
  run, so all of them are on one scale. Only lbzip2's curve is measured as drawn.
- **The cross-check** is the chain, the product of each run's direct comparison of its two
  releases. It needs no anchor, but its errors add up along the chain.
- **The drift diagnostic** compares a release with itself: its ratio to lbzip2 in its two
  runs, made days apart. A difference beyond noise means the machine, or lbzip2's figure on
  it, moved between the runs.

Each table gives an arm's median wall time. It also gives the geometric mean of the rounds'
paired wall ratios against the reference arm, with its 95% interval: a minus sign is
faster. Up to the machine's physical core count, each thread runs on a core of its own
(`--spread`); above it, threads share cores.

--8<-- "docs/generated/release-scaling.md"

To make a run, from a checkout of mibz2 with lbzip2 installed:

```sh
just bench-release-scaling                                    # the current release against the last, every CPU
just bench-release-scaling v0.2.4 v0.2.1 10 "1,4,10,20" big.bz2   # releases, rounds, threads, more files
```

With no releases named, it compares the newest release tag with the one before it. It builds
both releases in `target/release-scaling/`, runs the rounds, and records the run in
`bench/release-scaling/<date>-<machine>-<release>/`. After each release, the release runbook
(`docs/RELEASING.md`) asks for one such run, made with `just submit bench-release-scaling`,
and commits its record.

