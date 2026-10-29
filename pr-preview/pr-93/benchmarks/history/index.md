# History

Loading the chart…

Master's mibz2 figures over its commits, oldest to newest. The chart above draws every figure at once; below it, one chart per corpus and thread count. Each point is a recorded commit (linked to it): the mean of its 10 rounds, and its 95% interval as a whisker. The figures are measured on GitHub's runners ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-93/benchmarks/index.md>)).

- **Separate lines, never joined:** a line holds one runner CPU model and core count, one set of corpora, and one content of its corpus. Runners of one kind differ in speed from another (an AMD EPYC 7763 from a 9V45), so a commit measured on another kind of runner starts or joins that kind's line, in that kind's colour.
- **Joined only between neighbouring commits:** where commits measured on other runners come between two of a kind's points, its line breaks, so no segment spans commits it didn't measure. Its points either side still compare (same runner model, same corpus); they are different runs on possibly different hosts, not a paired comparison.
- **Bench versions:** a line runs on across bench versions that left these figures' meaning alone, and splits where a version redefined them ([the benchmark's changelog](<https://github.com/lmmx/mibz2/blob/master/bench/CHANGELOG.md>)). Each point's tooltip names the version that measured it.
- **`text`'s points stand alone:** that corpus is generated from mibz2's own sources, so its bytes change from commit to commit, and no two of its points are the same workload.
- **Trunk only:** pull requests and experiments are not recorded.
- **Peak RSS** is one of the figures (the selector above the charts).

These are decoder-level results. End-to-end results, the decoder in wikidata-pq's pipeline, are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

Loading the benchmark history…
