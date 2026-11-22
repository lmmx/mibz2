# History

Loading the chart…

How mibz2's decoding speed, CPU time and peak memory have changed, merge by merge. The figures are measured on GitHub's runners ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-714/benchmarks/index.md>)).

- **The chart chains each merge's own benchmark.** A pull request's benchmark runs its head against its base on one runner, both builds, for 10 rounds with their order alternating. Each merge keeps that paired change ([`bench/links.py`](<https://github.com/lmmx/mibz2/blob/master/bench/links.py>)), and the chart multiplies the changes, from the first merge whose arms ran in balanced order (bench 3.0.0). The band is the product's 95% interval: the links' intervals added in quadrature, on the log scale.
- **A merge that left the decoder alone** (the harness, docs, CI) is an A/A pair. It is drawn hollow and adds nothing.
- **A decoder change with no paired run** breaks the chain into a new segment. No gap is bridged.
- **What the chain assumes:** that a relative change measured on one runner holds on another (the merges ran on several CPU models, each named), and that the links' errors are independent.
- **Before bench 3.0.0** the head ran first in every round, which favoured it by about 1.8% at 1 thread in an A/A run. Those merges' changes are listed in the table below the charts, not chained.
- **Releases past 4 threads.** On wikidata's throughput, the chart also offers 8 to 20 threads, from the release-scaling runs below, on one i9-10900K. Those figures exist only at releases, so each is a diamond at the last merge its release tag contains, joined to the next by a dotted line that is not measured. They are modelled from release-against-release runs (the **All releases** inference below). They share the chain's axis and its meaning, a change since the start with up faster, but start from the oldest release and run on another machine. At 1, 2 and 4 threads they overlay CI's chain; above 4, CI has no figure and they stand alone. **All** overlays every thread count in one chart, a colour each along the releases' ramp, low to high.
- **Relative or Absolute.** The chart shows change since the start by default. **Absolute** shows each figure in its own units instead: throughput in MB/s of output, CPU time in CPU seconds (user + sys) for the corpus's decode, peak RSS in MB. CI's chain has no absolute scale of its own, because its merges ran on several runner CPUs and it multiplies ratios (of wall, CPU time and peak RSS alike). It borrows one runner model's scale: the model with the most master records at that corpus and thread count, through its newest record at a chained merge, which the chart rings and names. Every other merge is that record carried by the chain's ratios. The band is the links' interval between the merge and the anchor, with the record's own. Throughput and CPU time scale with the runner's speed, so their levels are that runner's. Peak RSS hardly depends on the runner: at a given corpus and thread count the runner models' newest records mostly agree within a few percent (`text` aside, whose content changes), and the note under the chart gives their range for each view. The i9-10900K's releases are in MB/s on the i9, another machine on another scale, so beside CI's chain they get a panel of their own. In **All** every thread count sits at its own level on the i9. The release-scaling runs record throughput only, so CPU time and peak RSS have no releases and no **All**.
- **What changed in a release.** Under the chart, a panel gives the notes of the release in focus: the newest, until a release's diamond, a release or a release-scaling run is pointed at, tapped or chosen. It quotes the release's CHANGELOG section (the first sentence of each summary, and its changes with their pull requests) and links the blog posts that explain it. Every release's notes are [at the foot of the page](<#what-changed-release-by-release>).
- **`text`** is generated from mibz2's own sources, so its content changes from merge to merge, and its chain multiplies ratios measured on different files.

These are decoder-level results. End-to-end results, the decoder in wikidata-pq's pipeline, are in [wikidata-pq's tracker](<https://lmmx.github.io/wikidata-pq/guide/bench-history/>).

## Absolute figures, one runner at a time

Master's own records: each commit measured once, on whichever runner its push drew, the mean of its 10 rounds with its 95% interval as a whisker.

- **One runner at a time.** Runners differ by up to 2× in speed, so a chart shows one runner CPU model, by default the one with the most records. Its consecutive points are that runner's consecutive records; the commits between them ran elsewhere, and their points are under their own runner.
- **A line never joins** another bench meaning (a version that redefined these figures; [the benchmark's changelog](<https://github.com/lmmx/mibz2/blob/master/bench/CHANGELOG.md>)) or another content of a corpus.
- **All runners together** are an option, for the figures themselves. They are not a trajectory.

Loading the benchmark history…

## Release scaling, past 4 threads

CI's runners have 4 vCPUs on 2 physical cores, so CI's figures stop at 4 threads. A release-scaling run (`just bench-release-scaling`) fills in the rest. It runs two releases and lbzip2 on one machine of a developer's, with the same harness and paired rounds, from 1 thread to every CPU the machine has. When one is recorded, the chart at the top offers it as the **Release scaling** view.

These runs inform; they don't decide. Nothing merges or releases on a release-scaling figure: pull requests are measured on CI, and pass without any run here. Each run is kept as it was recorded, with its machine, its date and its raw records, in [`bench/release-scaling/`](<https://github.com/lmmx/mibz2/tree/master/bench/release-scaling>). Runs on different machines are never compared or combined.

**All releases**, the view's first choice, puts every release against every other without re-running the old ones. It is inferred from one machine's pairwise runs; no single run had every release in it. lbzip2, the same build in every run, is a fixed anchor:

- **Pooling.** At each thread count, each release's figure is its wall time relative to lbzip2's. That is the mean over a run's rounds of the paired log ratio. A release measured in two runs, once as the new release and once as the old, gets the inverse-variance weighted mean of the two.
- **Any pair** is the ratio of the two figures. Its interval adds their variances, which is cautious for two releases from the same run, since their rounds share lbzip2's.
- **The curves** apply each release's figure to lbzip2's measured throughput in the newest run, so all of them are on one scale. Only lbzip2's curve is measured as drawn.
- **The cross-check** is the chain, the product of each run's direct comparison of its two releases. It needs no anchor, but its errors add up along the chain.
- **The knee** is each release's thread count of highest modelled throughput, with that peak against the release's own 8- and 10-thread figures. Its table, under the curves, shows where the knee sits release by release, and how the roof above it moves.
- **The drift diagnostic** compares a release with itself: its ratio to lbzip2 in its two runs, made days apart. A difference beyond noise means the machine, or lbzip2's figure on it, moved between the runs.

Each table gives an arm's median wall time. It also gives the geometric mean of the rounds' paired wall ratios against the reference arm, with its 95% interval: a minus sign is faster. Up to the machine's physical core count, each thread runs on a core of its own (`--spread`); above it, threads share cores.

### 2026-11-21: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz

10 cores, 20 threads · 10 paired rounds · the reference arm is **0.5.11** · [the records](<https://github.com/lmmx/mibz2/tree/master/bench/release-scaling/2026-11-21-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.5.11>)

**`wikidata`**

| threads | 0.5.11 wall (median) | lbzip2 2.5 wall (median) | 0.5.10 wall (median) | lbzip2 2.5 against 0.5.11 | 0.5.10 against 0.5.11 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.387 s | 2.513 s | 0.386 s | \+544.9% (+523.5, +567.1) **slower** | \+0.2% (−4.1, +4.8) |
| 2 | 0.239 s | 1.331 s | 0.238 s | \+457.1% (+447.8, +466.6) **slower** | −0.4% (−2.5, +1.8) |
| 4 | 0.143 s | 0.714 s | 0.142 s | \+400.3% (+387.7, +413.3) **slower** | −0.0% (−2.0, +2.0) |
| 8 | 0.075 s | 0.514 s | 0.077 s | \+582.9% (+577.2, +588.6) **slower** | \+2.5% (+1.3, +3.6) **slower** |
| 10 | 0.069 s | 0.505 s | 0.072 s | \+634.4% (+629.8, +639.0) **slower** | \+4.1% (+3.6, +4.6) **slower** |
| 12 | 0.075 s | 0.508 s | 0.077 s | \+581.9% (+571.2, +592.6) **slower** | \+3.6% (+1.5, +5.8) **slower** |
| 16 | 0.075 s | 0.501 s | 0.078 s | \+563.1% (+546.0, +580.6) **slower** | \+5.6% (+2.1, +9.2) **slower** |
| 20 | 0.074 s | 0.492 s | 0.078 s | \+558.7% (+552.6, +564.8) **slower** | \+4.6% (+3.8, +5.5) **slower** |

Every arm's output was the same bytes as the reference's.

### 2026-11-21: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz

10 cores, 20 threads · 10 paired rounds · the reference arm is **0.5.10** · [the records](<https://github.com/lmmx/mibz2/tree/master/bench/release-scaling/2026-11-21-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.5.10>)

**`wikidata`**

| threads | 0.5.10 wall (median) | lbzip2 2.5 wall (median) | 0.5.9 wall (median) | lbzip2 2.5 against 0.5.10 | 0.5.9 against 0.5.10 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.374 s | 2.510 s | 0.398 s | \+566.1% (+549.4, +583.3) **slower** | \+4.6% (+0.4, +9.0) **slower** |
| 2 | 0.239 s | 1.329 s | 0.237 s | \+454.5% (+445.0, +464.1) **slower** | −0.9% (−2.9, +1.2) |
| 4 | 0.142 s | 0.719 s | 0.141 s | \+406.5% (+392.1, +421.2) **slower** | −1.4% (−4.5, +1.9) |
| 8 | 0.077 s | 0.513 s | 0.082 s | \+568.7% (+567.1, +570.4) **slower** | \+6.3% (+5.1, +7.5) **slower** |
| 10 | 0.072 s | 0.506 s | 0.078 s | \+603.7% (+595.0, +612.6) **slower** | \+8.5% (+7.5, +9.4) **slower** |
| 12 | 0.077 s | 0.508 s | 0.083 s | \+553.5% (+535.5, +572.0) **slower** | \+6.7% (+4.0, +9.6) **slower** |
| 16 | 0.077 s | 0.497 s | 0.083 s | \+542.1% (+533.9, +550.4) **slower** | \+7.3% (+6.2, +8.5) **slower** |
| 20 | 0.078 s | 0.492 s | 0.083 s | \+517.6% (+496.4, +539.6) **slower** | \+5.1% (+1.0, +9.4) **slower** |

Every arm's output was the same bytes as the reference's.

### 2026-11-21: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz

10 cores, 20 threads · 10 paired rounds · the reference arm is **0.5.9** · [the records](<https://github.com/lmmx/mibz2/tree/master/bench/release-scaling/2026-11-21-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.5.9>)

**`wikidata`**

| threads | 0.5.9 wall (median) | lbzip2 2.5 wall (median) | 0.5.8 wall (median) | lbzip2 2.5 against 0.5.9 | 0.5.8 against 0.5.9 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.380 s | 2.511 s | 0.398 s | \+555.8% (+537.6, +574.6) **slower** | \+2.2% (−2.5, +7.1) |
| 2 | 0.244 s | 1.339 s | 0.241 s | \+448.5% (+438.0, +459.2) **slower** | −1.9% (−5.5, +1.8) |
| 4 | 0.146 s | 0.718 s | 0.139 s | \+390.4% (+382.0, +398.8) **slower** | −3.6% (−6.5, −0.7) **faster** |
| 8 | 0.082 s | 0.513 s | 0.082 s | \+527.0% (+522.5, +531.5) **slower** | \+0.8% (−0.4, +1.9) |
| 10 | 0.079 s | 0.511 s | 0.079 s | \+543.8% (+535.0, +552.6) **slower** | −0.5% (−2.3, +1.3) |
| 12 | 0.083 s | 0.508 s | 0.084 s | \+514.0% (+504.4, +523.7) **slower** | \+0.7% (−0.3, +1.8) |
| 16 | 0.083 s | 0.501 s | 0.083 s | \+496.0% (+479.6, +512.8) **slower** | \+0.1% (−3.9, +4.3) |
| 20 | 0.083 s | 0.492 s | 0.084 s | \+486.0% (+472.3, +500.1) **slower** | \+0.1% (−3.5, +3.8) |

Every arm's output was the same bytes as the reference's.

### 2026-11-21: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz

10 cores, 20 threads · 10 paired rounds · the reference arm is **0.5.8** · [the records](<https://github.com/lmmx/mibz2/tree/master/bench/release-scaling/2026-11-21-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.5.8>)

**`wikidata`**

| threads | 0.5.8 wall (median) | lbzip2 2.5 wall (median) | 0.5.7 wall (median) | lbzip2 2.5 against 0.5.8 | 0.5.7 against 0.5.8 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.390 s | 2.509 s | 0.397 s | \+546.6% (+525.4, +568.4) **slower** | \+0.6% (−4.3, +5.8) |
| 2 | 0.238 s | 1.339 s | 0.236 s | \+468.3% (+456.2, +480.7) **slower** | −0.1% (−2.8, +2.7) |
| 4 | 0.142 s | 0.713 s | 0.142 s | \+399.9% (+387.5, +412.7) **slower** | −0.5% (−3.5, +2.6) |
| 8 | 0.082 s | 0.514 s | 0.088 s | \+527.0% (+522.1, +531.9) **slower** | \+7.1% (+6.0, +8.1) **slower** |
| 10 | 0.079 s | 0.506 s | 0.083 s | \+544.6% (+531.2, +558.2) **slower** | \+5.2% (+2.8, +7.5) **slower** |
| 12 | 0.084 s | 0.509 s | 0.087 s | \+510.4% (+502.8, +518.0) **slower** | \+3.2% (+1.3, +5.1) **slower** |
| 16 | 0.083 s | 0.498 s | 0.086 s | \+492.0% (+481.1, +503.1) **slower** | \+2.7% (+0.2, +5.2) **slower** |
| 20 | 0.084 s | 0.492 s | 0.086 s | \+490.5% (+483.9, +497.3) **slower** | \+3.7% (+1.6, +5.8) **slower** |

Every arm's output was the same bytes as the reference's.

### 2026-11-19: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz

10 cores, 20 threads · 10 paired rounds · the reference arm is **0.5.7** · [the records](<https://github.com/lmmx/mibz2/tree/master/bench/release-scaling/2026-11-19-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.5.7>)

**`wikidata`**

| threads | 0.5.7 wall (median) | lbzip2 2.5 wall (median) | 0.5.6 wall (median) | lbzip2 2.5 against 0.5.7 | 0.5.6 against 0.5.7 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.397 s | 2.514 s | 0.391 s | \+536.5% (+517.5, +556.0) **slower** | −1.2% (−4.7, +2.5) |
| 2 | 0.241 s | 1.335 s | 0.240 s | \+454.5% (+444.0, +465.3) **slower** | −0.9% (−4.0, +2.2) |
| 4 | 0.142 s | 0.714 s | 0.142 s | \+403.8% (+394.8, +412.8) **slower** | \+1.1% (−2.0, +4.3) |
| 8 | 0.088 s | 0.513 s | 0.097 s | \+485.7% (+481.3, +490.2) **slower** | \+10.8% (+8.8, +12.7) **slower** |
| 10 | 0.082 s | 0.505 s | 0.089 s | \+518.1% (+513.7, +522.7) **slower** | \+7.3% (+6.1, +8.5) **slower** |
| 12 | 0.087 s | 0.509 s | 0.094 s | \+482.1% (+465.3, +499.3) **slower** | \+8.1% (+4.5, +11.9) **slower** |
| 16 | 0.088 s | 0.501 s | 0.095 s | \+462.8% (+449.5, +476.3) **slower** | \+7.4% (+3.9, +11.0) **slower** |
| 20 | 0.088 s | 0.492 s | 0.094 s | \+460.3% (+457.7, +462.9) **slower** | \+9.1% (+6.1, +12.3) **slower** |

Every arm's output was the same bytes as the reference's.

### 2026-11-18: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz

10 cores, 20 threads · 10 paired rounds · the reference arm is **0.5.6** · [the records](<https://github.com/lmmx/mibz2/tree/master/bench/release-scaling/2026-11-18-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.5.6>)

**`wikidata`**

| threads | 0.5.6 wall (median) | lbzip2 2.5 wall (median) | 0.5.5 wall (median) | lbzip2 2.5 against 0.5.6 | 0.5.5 against 0.5.6 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.384 s | 2.519 s | 0.383 s | \+554.8% (+536.7, +573.4) **slower** | \+1.2% (−2.4, +5.0) |
| 2 | 0.236 s | 1.338 s | 0.241 s | \+464.8% (+455.1, +474.7) **slower** | \+1.7% (−0.0, +3.4) |
| 4 | 0.141 s | 0.714 s | 0.145 s | \+406.7% (+400.1, +413.5) **slower** | \+2.1% (−0.6, +4.8) |
| 8 | 0.098 s | 0.515 s | 0.100 s | \+431.9% (+425.5, +438.3) **slower** | \+2.7% (+1.5, +4.0) **slower** |
| 10 | 0.088 s | 0.506 s | 0.088 s | \+468.9% (+449.3, +489.2) **slower** | \+0.1% (−4.3, +4.6) |
| 12 | 0.095 s | 0.509 s | 0.095 s | \+441.4% (+430.3, +452.7) **slower** | \+1.2% (−1.1, +3.5) |
| 16 | 0.094 s | 0.500 s | 0.097 s | \+430.5% (+423.0, +438.1) **slower** | \+1.8% (+0.2, +3.5) **slower** |
| 20 | 0.094 s | 0.495 s | 0.096 s | \+425.7% (+419.5, +432.0) **slower** | \+1.9% (+0.1, +3.8) **slower** |

Every arm's output was the same bytes as the reference's.

### 2026-11-18: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz

10 cores, 20 threads · 10 paired rounds · the reference arm is **0.5.5** · [the records](<https://github.com/lmmx/mibz2/tree/master/bench/release-scaling/2026-11-18-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.5.5>)

**`wikidata`**

| threads | 0.5.5 wall (median) | lbzip2 2.5 wall (median) | 0.5.4 wall (median) | lbzip2 2.5 against 0.5.5 | 0.5.4 against 0.5.5 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.413 s | 2.512 s | 0.413 s | \+521.5% (+504.9, +538.7) **slower** | \+0.7% (−2.6, +4.1) |
| 2 | 0.241 s | 1.338 s | 0.238 s | \+455.9% (+447.5, +464.5) **slower** | −0.3% (−2.9, +2.3) |
| 4 | 0.143 s | 0.714 s | 0.145 s | \+399.4% (+386.6, +412.6) **slower** | \+1.1% (−1.6, +3.8) |
| 8 | 0.099 s | 0.514 s | 0.105 s | \+417.4% (+410.1, +424.8) **slower** | \+5.4% (+3.1, +7.8) **slower** |
| 10 | 0.089 s | 0.506 s | 0.100 s | \+474.3% (+467.4, +481.4) **slower** | \+13.5% (+11.9, +15.1) **slower** |
| 12 | 0.095 s | 0.508 s | 0.107 s | \+435.4% (+426.4, +444.7) **slower** | \+12.8% (+10.7, +15.0) **slower** |
| 16 | 0.097 s | 0.493 s | 0.109 s | \+406.2% (+390.5, +422.5) **slower** | \+13.2% (+7.9, +18.8) **slower** |
| 20 | 0.095 s | 0.490 s | 0.108 s | \+411.3% (+396.3, +426.7) **slower** | \+12.1% (+8.6, +15.7) **slower** |

Every arm's output was the same bytes as the reference's.

### 2026-11-17: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz

10 cores, 20 threads · 10 paired rounds · the reference arm is **0.5.4** · [the records](<https://github.com/lmmx/mibz2/tree/master/bench/release-scaling/2026-11-17-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.5.4>)

**`wikidata`**

| threads | 0.5.4 wall (median) | lbzip2 2.5 wall (median) | 0.5.3 wall (median) | lbzip2 2.5 against 0.5.4 | 0.5.3 against 0.5.4 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.385 s | 2.510 s | 0.384 s | \+540.3% (+520.8, +560.4) **slower** | −1.1% (−4.1, +2.1) |
| 2 | 0.241 s | 1.339 s | 0.241 s | \+455.7% (+448.3, +463.3) **slower** | −0.9% (−2.0, +0.3) |
| 4 | 0.144 s | 0.714 s | 0.147 s | \+396.7% (+386.0, +407.6) **slower** | \+2.0% (−0.6, +4.8) |
| 8 | 0.105 s | 0.513 s | 0.110 s | \+390.6% (+385.5, +395.8) **slower** | \+4.5% (+3.8, +5.3) **slower** |
| 10 | 0.102 s | 0.505 s | 0.115 s | \+396.9% (+390.8, +403.0) **slower** | \+12.3% (+9.4, +15.2) **slower** |
| 12 | 0.106 s | 0.508 s | 0.105 s | \+377.6% (+368.2, +387.3) **slower** | −0.8% (−2.5, +1.0) |
| 16 | 0.106 s | 0.501 s | 0.106 s | \+371.8% (+367.3, +376.3) **slower** | −0.4% (−2.0, +1.2) |
| 20 | 0.108 s | 0.492 s | 0.107 s | \+352.1% (+333.1, +371.9) **slower** | −2.0% (−6.7, +2.9) |

Every arm's output was the same bytes as the reference's.

### 2026-11-17: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz

10 cores, 20 threads · 10 paired rounds · the reference arm is **0.5.3** · [the records](<https://github.com/lmmx/mibz2/tree/master/bench/release-scaling/2026-11-17-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.5.3>)

**`wikidata`**

| threads | 0.5.3 wall (median) | lbzip2 2.5 wall (median) | 0.5.2 wall (median) | lbzip2 2.5 against 0.5.3 | 0.5.2 against 0.5.3 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.387 s | 2.510 s | 0.393 s | \+537.5% (+518.2, +557.4) **slower** | \+0.2% (−1.9, +2.3) |
| 2 | 0.244 s | 1.329 s | 0.239 s | \+450.9% (+443.1, +458.9) **slower** | −1.1% (−3.0, +0.8) |
| 4 | 0.148 s | 0.714 s | 0.147 s | \+385.0% (+378.8, +391.2) **slower** | −0.6% (−1.9, +0.7) |
| 8 | 0.110 s | 0.517 s | 0.108 s | \+370.3% (+365.6, +375.1) **slower** | −1.2% (−2.7, +0.2) |
| 10 | 0.115 s | 0.507 s | 0.113 s | \+342.6% (+337.6, +347.7) **slower** | −1.6% (−4.3, +1.1) |
| 12 | 0.107 s | 0.509 s | 0.113 s | \+380.7% (+369.8, +391.7) **slower** | \+6.9% (+4.4, +9.5) **slower** |
| 16 | 0.107 s | 0.501 s | 0.114 s | \+366.7% (+349.5, +384.6) **slower** | \+6.7% (+2.8, +10.7) **slower** |
| 20 | 0.106 s | 0.491 s | 0.114 s | \+361.0% (+353.5, +368.7) **slower** | \+7.5% (+4.6, +10.4) **slower** |

Every arm's output was the same bytes as the reference's.

### 2026-11-16: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz

10 cores, 20 threads · 10 paired rounds · the reference arm is **0.5.2** · [the records](<https://github.com/lmmx/mibz2/tree/master/bench/release-scaling/2026-11-16-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.5.2>)

**`wikidata`**

| threads | 0.5.2 wall (median) | lbzip2 2.5 wall (median) | 0.5.1 wall (median) | lbzip2 2.5 against 0.5.2 | 0.5.1 against 0.5.2 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.410 s | 2.509 s | 0.387 s | \+524.0% (+506.6, +542.0) **slower** | −1.6% (−5.2, +2.1) |
| 2 | 0.244 s | 1.331 s | 0.244 s | \+448.3% (+441.0, +455.8) **slower** | \+0.0% (−1.7, +1.7) |
| 4 | 0.149 s | 0.714 s | 0.147 s | \+382.7% (+377.8, +387.6) **slower** | −0.8% (−2.4, +0.9) |
| 8 | 0.109 s | 0.513 s | 0.110 s | \+370.7% (+365.4, +376.2) **slower** | \+0.8% (−0.7, +2.3) |
| 10 | 0.113 s | 0.506 s | 0.114 s | \+349.5% (+343.9, +355.3) **slower** | \+0.5% (−1.8, +2.9) |
| 12 | 0.115 s | 0.508 s | 0.121 s | \+343.1% (+338.4, +347.8) **slower** | \+4.7% (+2.4, +7.0) **slower** |
| 16 | 0.116 s | 0.501 s | 0.121 s | \+324.2% (+311.0, +337.8) **slower** | \+4.7% (+0.1, +9.5) **slower** |
| 20 | 0.115 s | 0.493 s | 0.121 s | \+327.1% (+317.8, +336.5) **slower** | \+5.0% (+1.2, +8.9) **slower** |

Every arm's output was the same bytes as the reference's.

### 2026-11-16: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz

10 cores, 20 threads · 10 paired rounds · the reference arm is **0.5.1** · [the records](<https://github.com/lmmx/mibz2/tree/master/bench/release-scaling/2026-11-16-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.5.1>)

**`wikidata`**

| threads | 0.5.1 wall (median) | lbzip2 2.5 wall (median) | 0.5.0 wall (median) | lbzip2 2.5 against 0.5.1 | 0.5.0 against 0.5.1 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.404 s | 2.510 s | 0.397 s | \+530.3% (+511.5, +549.7) **slower** | −0.7% (−3.2, +1.9) |
| 2 | 0.241 s | 1.331 s | 0.240 s | \+453.2% (+445.7, +460.7) **slower** | −0.9% (−2.5, +0.7) |
| 4 | 0.151 s | 0.714 s | 0.149 s | \+379.5% (+367.3, +392.1) **slower** | −0.8% (−3.4, +1.9) |
| 8 | 0.109 s | 0.513 s | 0.109 s | \+374.6% (+369.0, +380.3) **slower** | \+0.6% (−0.9, +2.2) |
| 10 | 0.113 s | 0.505 s | 0.123 s | \+349.0% (+345.3, +352.8) **slower** | \+7.9% (+6.8, +9.0) **slower** |
| 12 | 0.120 s | 0.508 s | 0.127 s | \+327.9% (+320.9, +334.9) **slower** | \+6.6% (+4.7, +8.5) **slower** |
| 16 | 0.119 s | 0.501 s | 0.129 s | \+317.1% (+309.9, +324.4) **slower** | \+8.4% (+6.4, +10.4) **slower** |
| 20 | 0.121 s | 0.492 s | 0.128 s | \+308.6% (+302.5, +314.9) **slower** | \+8.4% (+3.9, +13.2) **slower** |

Every arm's output was the same bytes as the reference's.

### 2026-11-16: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz

10 cores, 20 threads · 10 paired rounds · the reference arm is **0.5.0** · [the records](<https://github.com/lmmx/mibz2/tree/master/bench/release-scaling/2026-11-16-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.5.0>)

**`wikidata`**

| threads | 0.5.0 wall (median) | lbzip2 2.5 wall (median) | 0.4.0 wall (median) | lbzip2 2.5 against 0.5.0 | 0.4.0 against 0.5.0 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.386 s | 2.519 s | 0.389 s | \+545.9% (+525.2, +567.3) **slower** | \+0.6% (−3.3, +4.7) |
| 2 | 0.238 s | 1.335 s | 0.237 s | \+461.0% (+452.8, +469.2) **slower** | −0.2% (−1.4, +1.1) |
| 4 | 0.146 s | 0.713 s | 0.148 s | \+388.8% (+382.2, +395.6) **slower** | \+1.5% (−1.0, +4.0) |
| 8 | 0.110 s | 0.514 s | 0.114 s | \+368.6% (+363.7, +373.6) **slower** | \+4.0% (+2.5, +5.5) **slower** |
| 10 | 0.122 s | 0.512 s | 0.121 s | \+319.9% (+314.9, +324.9) **slower** | −0.6% (−2.0, +0.7) |
| 12 | 0.128 s | 0.509 s | 0.128 s | \+293.5% (+285.1, +302.2) **slower** | −2.1% (−5.3, +1.2) |
| 16 | 0.127 s | 0.501 s | 0.128 s | \+288.3% (+284.5, +292.2) **slower** | \+0.1% (−1.7, +1.9) |
| 20 | 0.128 s | 0.493 s | 0.128 s | \+283.8% (+277.6, +290.0) **slower** | −0.1% (−2.4, +2.4) |

Every arm's output was the same bytes as the reference's.

### 2026-11-15: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz

10 cores, 20 threads · 10 paired rounds · the reference arm is **0.3.4** · [the records](<https://github.com/lmmx/mibz2/tree/master/bench/release-scaling/2026-11-15-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.3.4>)

**`wikidata`**

| threads | 0.3.4 wall (median) | lbzip2 2.5 wall (median) | 0.3.3 wall (median) | lbzip2 2.5 against 0.3.4 | 0.3.3 against 0.3.4 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.385 s | 2.515 s | 0.399 s | \+541.8% (+522.3, +562.0) **slower** | \+1.4% (−2.0, +4.8) |
| 2 | 0.241 s | 1.339 s | 0.243 s | \+459.0% (+444.8, +473.5) **slower** | \+1.6% (−1.4, +4.8) |
| 4 | 0.149 s | 0.714 s | 0.150 s | \+379.5% (+372.7, +386.5) **slower** | \+1.3% (−0.7, +3.4) |
| 8 | 0.126 s | 0.514 s | 0.135 s | \+311.7% (+305.7, +317.7) **slower** | \+7.8% (+5.7, +10.0) **slower** |
| 10 | 0.121 s | 0.506 s | 0.120 s | \+320.1% (+315.7, +324.5) **slower** | −0.7% (−2.4, +1.1) |
| 12 | 0.127 s | 0.510 s | 0.129 s | \+302.2% (+293.5, +311.1) **slower** | \+1.0% (−1.4, +3.5) |
| 16 | 0.128 s | 0.501 s | 0.127 s | \+291.5% (+284.2, +298.8) **slower** | \+0.4% (−2.0, +2.8) |
| 20 | 0.128 s | 0.494 s | 0.127 s | \+283.1% (+279.0, +287.3) **slower** | −0.7% (−3.6, +2.3) |

Every arm's output was the same bytes as the reference's.

### 2026-11-15: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz

10 cores, 20 threads · 10 paired rounds · the reference arm is **0.3.3** · [the records](<https://github.com/lmmx/mibz2/tree/master/bench/release-scaling/2026-11-15-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.3.3>)

**`wikidata`**

| threads | 0.3.3 wall (median) | lbzip2 2.5 wall (median) | 0.3.2 wall (median) | lbzip2 2.5 against 0.3.3 | 0.3.2 against 0.3.3 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.384 s | 2.521 s | 0.398 s | \+553.2% (+543.6, +563.0) **slower** | \+4.3% (+1.3, +7.4) **slower** |
| 2 | 0.242 s | 1.339 s | 0.253 s | \+454.0% (+446.9, +461.2) **slower** | \+4.0% (+1.7, +6.4) **slower** |
| 4 | 0.151 s | 0.713 s | 0.160 s | \+377.5% (+370.1, +385.0) **slower** | \+7.0% (+5.0, +9.0) **slower** |
| 8 | 0.136 s | 0.514 s | 0.135 s | \+280.0% (+277.6, +282.4) **slower** | −0.0% (−1.0, +0.9) |
| 10 | 0.121 s | 0.505 s | 0.121 s | \+316.0% (+312.9, +319.0) **slower** | −0.3% (−1.1, +0.4) |
| 12 | 0.128 s | 0.508 s | 0.129 s | \+298.9% (+288.8, +309.3) **slower** | \+0.5% (−3.5, +4.6) |
| 16 | 0.129 s | 0.501 s | 0.129 s | \+284.7% (+276.5, +293.1) **slower** | −0.4% (−2.3, +1.5) |
| 20 | 0.129 s | 0.494 s | 0.130 s | \+280.3% (+273.1, +287.5) **slower** | −0.0% (−3.0, +3.0) |

Every arm's output was the same bytes as the reference's.

### 2026-11-15: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz

10 cores, 20 threads · 10 paired rounds · the reference arm is **0.3.2** · [the records](<https://github.com/lmmx/mibz2/tree/master/bench/release-scaling/2026-11-15-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.3.2>)

**`wikidata`**

| threads | 0.3.2 wall (median) | lbzip2 2.5 wall (median) | 0.3.1 wall (median) | lbzip2 2.5 against 0.3.2 | 0.3.1 against 0.3.2 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.403 s | 2.520 s | 0.386 s | \+519.6% (+500.3, +539.6) **slower** | −4.5% (−7.7, −1.3) **faster** |
| 2 | 0.253 s | 1.331 s | 0.256 s | \+426.3% (+414.0, +439.0) **slower** | \+1.5% (−1.3, +4.4) |
| 4 | 0.160 s | 0.713 s | 0.163 s | \+348.0% (+340.6, +355.5) **slower** | \+2.7% (+0.9, +4.6) **slower** |
| 8 | 0.136 s | 0.513 s | 0.136 s | \+278.7% (+275.8, +281.6) **slower** | −0.2% (−1.2, +0.7) |
| 10 | 0.121 s | 0.505 s | 0.120 s | \+321.0% (+316.2, +325.9) **slower** | −0.5% (−1.9, +1.0) |
| 12 | 0.128 s | 0.514 s | 0.128 s | \+299.5% (+291.9, +307.4) **slower** | −0.6% (−2.3, +1.2) |
| 16 | 0.131 s | 0.501 s | 0.127 s | \+279.4% (+273.3, +285.5) **slower** | −4.0% (−7.1, −0.8) **faster** |
| 20 | 0.133 s | 0.493 s | 0.128 s | \+274.7% (+265.8, +283.9) **slower** | −4.2% (−6.7, −1.6) **faster** |

Every arm's output was the same bytes as the reference's.

### 2026-11-14: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz

10 cores, 20 threads · 10 paired rounds · the reference arm is **0.3.1** · [the records](<https://github.com/lmmx/mibz2/tree/master/bench/release-scaling/2026-11-14-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.3.1>)

**`wikidata`**

| threads | 0.3.1 wall (median) | lbzip2 2.5 wall (median) | 0.3.0 wall (median) | lbzip2 2.5 against 0.3.1 | 0.3.0 against 0.3.1 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.388 s | 2.513 s | 0.416 s | \+534.1% (+509.5, +559.7) **slower** | \+5.1% (−1.0, +11.6) |
| 2 | 0.259 s | 1.335 s | 0.269 s | \+418.8% (+408.2, +429.6) **slower** | \+3.4% (+1.0, +5.9) **slower** |
| 4 | 0.164 s | 0.714 s | 0.173 s | \+332.3% (+323.9, +340.9) **slower** | \+4.2% (+2.2, +6.2) **slower** |
| 8 | 0.134 s | 0.514 s | 0.135 s | \+283.2% (+279.4, +286.9) **slower** | \+0.6% (−0.3, +1.6) |
| 10 | 0.120 s | 0.506 s | 0.120 s | \+321.2% (+318.1, +324.4) **slower** | −0.0% (−1.4, +1.3) |
| 12 | 0.126 s | 0.510 s | 0.130 s | \+307.8% (+298.2, +317.7) **slower** | \+2.4% (−0.4, +5.3) |
| 16 | 0.127 s | 0.501 s | 0.130 s | \+291.1% (+284.6, +297.7) **slower** | \+1.7% (−0.5, +3.9) |
| 20 | 0.128 s | 0.492 s | 0.130 s | \+273.5% (+260.3, +287.3) **slower** | −1.3% (−5.5, +3.1) |

Every arm's output was the same bytes as the reference's.

### 2026-11-14: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz

10 cores, 20 threads · 10 paired rounds · the reference arm is **0.3.0** · [the records](<https://github.com/lmmx/mibz2/tree/master/bench/release-scaling/2026-11-14-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.3.0>)

**`wikidata`**

| threads | 0.3.0 wall (median) | lbzip2 2.5 wall (median) | 0.2.9 wall (median) | lbzip2 2.5 against 0.3.0 | 0.2.9 against 0.3.0 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.392 s | 2.514 s | 0.468 s | \+523.9% (+499.2, +549.6) **slower** | \+16.1% (+12.0, +20.4) **slower** |
| 2 | 0.266 s | 1.339 s | 0.274 s | \+405.2% (+399.1, +411.5) **slower** | \+3.4% (+2.2, +4.6) **slower** |
| 4 | 0.173 s | 0.714 s | 0.187 s | \+315.6% (+308.5, +322.9) **slower** | \+9.2% (+7.6, +10.7) **slower** |
| 8 | 0.135 s | 0.514 s | 0.135 s | \+280.9% (+277.3, +284.5) **slower** | −0.2% (−1.2, +0.9) |
| 10 | 0.122 s | 0.506 s | 0.122 s | \+317.1% (+312.6, +321.7) **slower** | −0.4% (−2.1, +1.4) |
| 12 | 0.130 s | 0.508 s | 0.127 s | \+291.4% (+285.6, +297.3) **slower** | −1.7% (−3.4, −0.0) **faster** |
| 16 | 0.131 s | 0.501 s | 0.129 s | \+283.0% (+276.3, +289.9) **slower** | −0.2% (−2.6, +2.2) |
| 20 | 0.129 s | 0.493 s | 0.127 s | \+280.2% (+269.9, +290.7) **slower** | −1.6% (−4.2, +1.1) |

Every arm's output was the same bytes as the reference's.

### 2026-11-10: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz

10 cores, 20 threads · 10 paired rounds · the reference arm is **0.2.9** · [the records](<https://github.com/lmmx/mibz2/tree/master/bench/release-scaling/2026-11-10-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.2.9>)

**`wikidata`**

| threads | 0.2.9 wall (median) | lbzip2 2.5 wall (median) | 0.2.8 wall (median) | lbzip2 2.5 against 0.2.9 | 0.2.8 against 0.2.9 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.469 s | 2.516 s | 0.464 s | \+435.3% (+431.1, +439.5) **slower** | −0.9% (−1.7, −0.2) **faster** |
| 2 | 0.272 s | 1.339 s | 0.271 s | \+391.9% (+385.3, +398.7) **slower** | −0.2% (−2.0, +1.6) |
| 4 | 0.187 s | 0.714 s | 0.186 s | \+284.8% (+281.0, +288.6) **slower** | \+0.1% (−1.0, +1.2) |
| 8 | 0.135 s | 0.514 s | 0.155 s | \+279.4% (+275.7, +283.2) **slower** | \+14.7% (+13.4, +16.0) **slower** |
| 10 | 0.121 s | 0.506 s | 0.139 s | \+319.1% (+312.0, +326.3) **slower** | \+14.5% (+12.7, +16.4) **slower** |
| 12 | 0.129 s | 0.509 s | 0.143 s | \+296.5% (+291.5, +301.5) **slower** | \+11.1% (+9.4, +12.9) **slower** |
| 16 | 0.130 s | 0.501 s | 0.144 s | \+283.8% (+275.9, +291.9) **slower** | \+10.9% (+8.6, +13.3) **slower** |
| 20 | 0.128 s | 0.492 s | 0.141 s | \+286.1% (+280.3, +292.0) **slower** | \+10.2% (+7.5, +13.0) **slower** |

Every arm's output was the same bytes as the reference's.

### 2026-11-10: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz

10 cores, 20 threads · 10 paired rounds · the reference arm is **0.2.8** · [the records](<https://github.com/lmmx/mibz2/tree/master/bench/release-scaling/2026-11-10-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.2.8>)

**`wikidata`**

| threads | 0.2.8 wall (median) | lbzip2 2.5 wall (median) | 0.2.7 wall (median) | lbzip2 2.5 against 0.2.8 | 0.2.7 against 0.2.8 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.468 s | 2.521 s | 0.526 s | \+436.4% (+432.7, +440.2) **slower** | \+12.9% (+11.1, +14.8) **slower** |
| 2 | 0.271 s | 1.331 s | 0.308 s | \+391.5% (+389.5, +393.5) **slower** | \+13.3% (+11.8, +14.8) **slower** |
| 4 | 0.187 s | 0.718 s | 0.201 s | \+282.7% (+277.9, +287.5) **slower** | \+7.0% (+5.3, +8.8) **slower** |
| 8 | 0.155 s | 0.515 s | 0.155 s | \+233.0% (+230.1, +235.8) **slower** | −0.3% (−1.4, +0.7) |
| 10 | 0.138 s | 0.515 s | 0.137 s | \+269.2% (+265.7, +272.8) **slower** | −0.7% (−1.5, +0.1) |
| 12 | 0.143 s | 0.508 s | 0.143 s | \+258.5% (+251.9, +265.3) **slower** | \+0.3% (−1.8, +2.4) |
| 16 | 0.145 s | 0.501 s | 0.142 s | \+244.5% (+234.1, +255.2) **slower** | −1.5% (−4.6, +1.8) |
| 20 | 0.147 s | 0.492 s | 0.142 s | \+233.4% (+221.4, +245.8) **slower** | −3.2% (−6.9, +0.7) |

Every arm's output was the same bytes as the reference's.

### 2026-11-10: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz

10 cores, 20 threads · 10 paired rounds · the reference arm is **0.2.7** · [the records](<https://github.com/lmmx/mibz2/tree/master/bench/release-scaling/2026-11-10-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.2.7>)

**`wikidata`**

| threads | 0.2.7 wall (median) | lbzip2 2.5 wall (median) | 0.2.6 wall (median) | lbzip2 2.5 against 0.2.7 | 0.2.6 against 0.2.7 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.517 s | 2.503 s | 0.522 s | \+381.6% (+374.4, +388.9) **slower** | \+1.4% (−1.0, +3.8) |
| 2 | 0.302 s | 1.300 s | 0.301 s | \+332.5% (+327.4, +337.8) **slower** | −0.2% (−1.2, +0.9) |
| 4 | 0.204 s | 0.715 s | 0.208 s | \+250.5% (+248.1, +253.0) **slower** | \+1.3% (−0.3, +2.9) |
| 8 | 0.163 s | 0.525 s | 0.161 s | \+221.8% (+218.2, +225.4) **slower** | −1.2% (−2.7, +0.3) |
| 10 | 0.139 s | 0.519 s | 0.139 s | \+270.1% (+259.4, +281.1) **slower** | −1.6% (−4.5, +1.4) |
| 12 | 0.145 s | 0.510 s | 0.148 s | \+253.3% (+243.1, +263.9) **slower** | \+1.5% (−2.9, +6.0) |
| 16 | 0.148 s | 0.501 s | 0.144 s | \+239.9% (+231.6, +248.4) **slower** | −2.2% (−4.3, −0.1) **faster** |
| 20 | 0.143 s | 0.494 s | 0.144 s | \+244.5% (+239.8, +249.1) **slower** | \+0.3% (−2.2, +3.0) |

Every arm's output was the same bytes as the reference's.

### 2026-11-07: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz

10 cores, 20 threads · 10 paired rounds · the reference arm is **0.2.6** · [the records](<https://github.com/lmmx/mibz2/tree/master/bench/release-scaling/2026-11-07-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.2.6>)

**`wikidata`**

| threads | 0.2.6 wall (median) | lbzip2 2.5 wall (median) | 0.2.5 wall (median) | lbzip2 2.5 against 0.2.6 | 0.2.5 against 0.2.6 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.524 s | 2.503 s | 0.516 s | \+376.7% (+371.5, +381.9) **slower** | −1.1% (−2.6, +0.5) |
| 2 | 0.300 s | 1.301 s | 0.302 s | \+334.7% (+329.6, +339.8) **slower** | \+0.5% (−0.8, +1.7) |
| 4 | 0.205 s | 0.715 s | 0.205 s | \+250.6% (+246.2, +255.1) **slower** | \+0.5% (−1.0, +1.9) |
| 8 | 0.161 s | 0.524 s | 0.161 s | \+224.3% (+222.2, +226.5) **slower** | −0.7% (−1.7, +0.4) |
| 10 | 0.140 s | 0.518 s | 0.149 s | \+262.1% (+243.4, +281.8) **slower** | \+3.7% (−1.6, +9.3) |
| 12 | 0.145 s | 0.509 s | 0.152 s | \+248.9% (+241.0, +256.9) **slower** | \+4.6% (+2.2, +7.0) **slower** |
| 16 | 0.146 s | 0.501 s | 0.153 s | \+241.3% (+235.0, +247.7) **slower** | \+6.1% (+2.9, +9.5) **slower** |
| 20 | 0.144 s | 0.493 s | 0.153 s | \+242.9% (+235.1, +250.9) **slower** | \+6.5% (+3.1, +10.1) **slower** |

Every arm's output was the same bytes as the reference's.

### 2026-11-06: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz

10 cores, 20 threads · 10 paired rounds · the reference arm is **0.2.5** · [the records](<https://github.com/lmmx/mibz2/tree/master/bench/release-scaling/2026-11-06-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.2.5>)

**`wikidata`**

| threads | 0.2.5 wall (median) | lbzip2 2.5 wall (median) | 0.2.4 wall (median) | lbzip2 2.5 against 0.2.5 | 0.2.4 against 0.2.5 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.520 s | 2.504 s | 0.528 s | \+380.7% (+377.1, +384.4) **slower** | \+1.5% (−0.4, +3.4) |
| 2 | 0.299 s | 1.299 s | 0.299 s | \+334.9% (+330.5, +339.2) **slower** | −0.5% (−2.1, +1.1) |
| 4 | 0.206 s | 0.716 s | 0.205 s | \+246.9% (+243.1, +250.7) **slower** | −0.3% (−1.7, +1.2) |
| 8 | 0.161 s | 0.525 s | 0.162 s | \+225.5% (+223.0, +228.0) **slower** | \+0.8% (+0.0, +1.6) **slower** |
| 10 | 0.148 s | 0.519 s | 0.165 s | \+253.0% (+249.3, +256.8) **slower** | \+11.9% (+10.1, +13.8) **slower** |
| 12 | 0.153 s | 0.509 s | 0.165 s | \+234.7% (+227.2, +242.4) **slower** | \+8.7% (+6.0, +11.4) **slower** |
| 16 | 0.153 s | 0.501 s | 0.165 s | \+224.5% (+219.1, +230.1) **slower** | \+8.1% (+5.7, +10.5) **slower** |
| 20 | 0.154 s | 0.494 s | 0.166 s | \+219.0% (+214.3, +223.8) **slower** | \+8.2% (+5.8, +10.7) **slower** |

Every arm's output was the same bytes as the reference's.

### 2026-11-06: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz

10 cores, 20 threads · 10 paired rounds · the reference arm is **0.2.4** · [the records](<https://github.com/lmmx/mibz2/tree/master/bench/release-scaling/2026-11-06-intel-r-core-tm-i9-10900k-cpu-3-70ghz>)

**`wikidata`**

| threads | 0.2.4 wall (median) | lbzip2 2.5 wall (median) | 0.2.1 wall (median) | lbzip2 2.5 against 0.2.4 | 0.2.1 against 0.2.4 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.521 s | 2.502 s | 0.528 s | \+377.2% (+371.1, +383.5) **slower** | \+0.9% (−0.4, +2.2) |
| 2 | 0.296 s | 1.292 s | 0.327 s | \+334.0% (+328.8, +339.3) **slower** | \+10.1% (+8.8, +11.4) **slower** |
| 4 | 0.204 s | 0.715 s | 0.240 s | \+249.4% (+246.1, +252.7) **slower** | \+18.2% (+16.2, +20.3) **slower** |
| 8 | 0.161 s | 0.525 s | 0.167 s | \+225.4% (+222.3, +228.5) **slower** | \+3.5% (+3.0, +4.0) **slower** |
| 10 | 0.165 s | 0.518 s | 0.165 s | \+215.8% (+212.4, +219.2) **slower** | \+0.6% (−0.9, +2.2) |
| 12 | 0.164 s | 0.509 s | 0.169 s | \+210.8% (+208.2, +213.4) **slower** | \+3.1% (+1.3, +4.9) **slower** |
| 16 | 0.165 s | 0.501 s | 0.169 s | \+202.0% (+196.7, +207.4) **slower** | \+3.0% (+0.0, +6.0) **slower** |
| 20 | 0.166 s | 0.492 s | 0.169 s | \+196.1% (+190.4, +201.9) **slower** | \+2.1% (−0.5, +4.7) |

Every arm's output was the same bytes as the reference's.

To make a run, from a checkout of mibz2 with lbzip2 installed:

```sh
just bench-release-scaling                                    # the current release against the last, every CPU
just bench-release-scaling v0.2.4 v0.2.1 10 "1,4,10,20" big.bz2   # releases, rounds, threads, more files
```

With no releases named, it compares the newest release tag with the one before it. It builds both releases in `target/release-scaling/`, runs the rounds, and records the run in `bench/release-scaling/<date>-<machine>-<release>/`. After each release, the release runbook (`docs/RELEASING.md`) asks for one such run, made with `just submit bench-release-scaling`, and commits its record.

## What changed, release by release

Each release's notes, newest first, from [`CHANGELOG.md`](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md>) ([`docs/scripts/release_notes.py`](<https://github.com/lmmx/mibz2/blob/master/docs/scripts/release_notes.py>)): its opening sentence, the first sentence of each summary written for it, its changes with their pull requests, and the blog posts that explain it. A post names the releases it explains in its front matter (`releases: ["0.5.8"]`), and its header links back here. The figures behind each summary are in the CHANGELOG.

### 0.5.11

21 November 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#0511---2026-11-21>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.5.10...v0.5.11>)

- Dense-block admission (\#703): peak memory about 28% lower from 8 to 20 threads on the i9, and faster too.
- Release scaling against 0.5.10: +2.1% at 8 threads, +4.2% at 10, +3.6% to +4.8% at 12 to 20.

**Changes:** why each release happened: changelog notes and linked posts on the History page ([\#702](<https://github.com/lmmx/mibz2/pull/702>)); admit dense blocks (width certificate) with producer buffers in edges ([\#703](<https://github.com/lmmx/mibz2/pull/703>)).

**Read:** [A memory gain is a gain: the change our speed-only rule nearly shelved](<https://lmmx.github.io/mibz2/pr-preview/pr-714/blog/2026-11-21-a-memory-gain-is-a-gain/index.md>)

### 0.5.10

21 November 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#0510---2026-11-21>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.5.9...v0.5.10>)

- The Open kernel's memory diet (\#692): 5–8% faster from 6 to 10 threads on the i9, at about 0.89 of the peak memory.
- The worker-count recalibration: no trim moves.
- Release scaling against 0.5.9: +6.5% at 8 threads, +9.0% at 10, +5.2% to +7.4% at 12 to 20.

**Changes:** memory diet, Endpoint in edges and queues in out ([\#692](<https://github.com/lmmx/mibz2/pull/692>)).

**Read:** [Besieging hard problems: cheap checks first, after the Riemann proof](<https://lmmx.github.io/mibz2/pr-preview/pr-714/blog/2026-11-21-besieging-hard-problems/index.md>)

### 0.5.9

21 November 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#059---2026-11-21>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.5.8...v0.5.9>)

- 0.5.8's worker-count recalibration: one change, unpinned at 4 threads on the measured i9.

**Changes:** the lazy kernel at 4 threads on the measured i9, unpinned ([\#678](<https://github.com/lmmx/mibz2/pull/678>)).

**Read:** [Fast alone, slow in a crowd: which speed-ups survive many cores](<https://lmmx.github.io/mibz2/pr-preview/pr-714/blog/2026-11-21-fast-alone-slow-in-a-crowd/index.md>)

### 0.5.8

21 November 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#058---2026-11-21>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.5.7...v0.5.8>)

**Changes:** fixed-width queue fill, Vec16 queues and the 16-byte emit ([\#668](<https://github.com/lmmx/mibz2/pull/668>)).

**Read:** [Sixteen at a stroke: fixed-width writes that stop the processor guessing](<https://lmmx.github.io/mibz2/pr-preview/pr-714/blog/2026-11-21-sixteen-at-a-stroke/index.md>)

### 0.5.7

19 November 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#057---2026-11-19>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.5.6...v0.5.7>)

A kernel release. On LazyCells' admitted blocks (r/n from 1/64 to 0.15: Wikidata and other repetitive data), the workers now invert with the open kernel.

- Corpora with no admitted block (27 cells): pooled 1.0003 \[0.9954, 1.0051\], with no cell flagged.
- Guard corpora with admitted blocks (12 cells): no slowdown under Holm; points 0.982–1.006.
- The worker trims were recalibrated for the new kernel.
- The paired run on CI (the last tag against master, Benchmark run 44588824608, AMD EPYC 7763, 4 cores) is neutral.
- 0.5.7's worker-count recalibration: one change, at 6 threads on the measured i9.

**Changes:** the lazy kernel at 6 threads on the measured i9, pinned and unpinned ([\#642](<https://github.com/lmmx/mibz2/pull/642>)); the open kernel as the workers' kernel on LazyCells' admitted blocks ([\#630](<https://github.com/lmmx/mibz2/pull/630>)).

**Read:** [Open Returns: decoding a block from a tenth of its rows](<https://lmmx.github.io/mibz2/pr-preview/pr-714/blog/2026-11-19-open-returns/index.md>)

### 0.5.6

18 November 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#056---2026-11-18>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.5.5...v0.5.6>)

A kernel release: Cells' writer reads its rules once.

- Cells writes each full-period block in one scan of its rules (study 5, \#579; \#581).
- The worker trims are unchanged, without a recalibration.

**Changes:** write each full-period block in one scan of its rules (study 5, \#579) ([\#581](<https://github.com/lmmx/mibz2/pull/581>)).

### 0.5.5

18 November 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#055---2026-11-18>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.5.4...v0.5.5>)

A kernel release: the lazy kernel admits more blocks, so fewer of them take the T-vector walk.

- Wider admission for the lazy kernel (\#559). `LazyCells`, the workers' kernel from `LAZY_FROM_THREADS` threads and on the trimmed i9 plans, admits blocks up to r/n 0.15 (3 run records in 20 rows), not 0.10, with a budget of 0.32 n cells, not 0.16 n; the sparse gate is Cells' own.
- The release check, run 44290437026 on an AMD EPYC 9V74: one primary row flagged, Wikidata at 1 thread, +1.6% (+0.0, +3.1), at the edge of noise; \#559's only footprint at 1 thread is one branch a block.

**Changes:** wider admission for the lazy kernel, to r/n 0.15 with a budget of 0.32 n ([\#559](<https://github.com/lmmx/mibz2/pull/559>)).

### 0.5.4

17 November 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#054---2026-11-17>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.5.3...v0.5.4>)

A plan release: on the measured i9, the trimmed workers now fire by how the process is placed.

- The trimmed workers by mode (\#537). Pinned (one CPU on each of exactly the threads asked, no two on a core, as every sweep measured): lazy 7/28 at 10 threads (0.881 to 0.889 of lazy 10/40), lazy 6/24 at 8 (0.925 of lazy 8/32), eager 5/20 at 6 (0.938 of eager 6/24) and eager 3/12 at 4 (0.959 of eager 4/16).
- The machine read once a process (\#545). The machine-wide facts (the L3, cpuinfo's CPU identities and the topology of every CPU present and online) are cached on first use.
- The release check. Its first run, 44095874788 on an Intel Xeon 8370C, caught \#537's per-decoder sysfs and cpuinfo reads at 4 to 8 threads (0.5 to 1 ms a default decoder on a 4-CPU runner): 4-thread small inputs were 3.8–5.8% slower.

**Changes:** read the machine once a process ([\#545](<https://github.com/lmmx/mibz2/pull/545>)); on the measured i9, six lazy workers at 8 threads asked and five at 6 ([\#537](<https://github.com/lmmx/mibz2/pull/537>)).

**Read:** [Modelling the cache to rightsize workers](<https://lmmx.github.io/mibz2/pr-preview/pr-714/blog/2026-11-17-modelling-the-cache-to-rightsize-workers/index.md>), [Raising the knee: scaling throughput to the limit of physical cores](<https://lmmx.github.io/mibz2/pr-preview/pr-714/blog/2026-11-17-raising-the-knee/index.md>)

### 0.5.3

17 November 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#053---2026-11-17>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.5.2...v0.5.3>)

A plan release: on the measured i9, from 10 threads asked, the decoder runs seven lazy workers with 28 blocks in flight.

- Seven lazy workers from 10 threads asked (\#524). It applies only on the measured profile (the i9-10900K: 10 cores, 20 CPUs, a 20 MiB L3, no cpu.max limit), in place of 9 workers with 36 blocks past the cores and 10 with 40 at exactly 10.
- That run's 10-thread row doesn't measure the trim. It pinned the 10-thread cells to CPUs 0–9, and 0.5.3 recognised the machine through the process's affinity, so pinned it declined the trim and built 10 workers (0.5.2 over 0.5.3: 0.984 \[0.957, 1.011\]).

**Changes:** on the measured i9, seven lazy workers from 10 threads asked, with the window sized to them ([\#524](<https://github.com/lmmx/mibz2/pull/524>)); a 12-byte rules arm (W12LazyCells), the lazy induction storing each rule's children packed ([\#506](<https://github.com/lmmx/mibz2/pull/506>)).

### 0.5.2

16 November 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#052---2026-11-16>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.5.1...v0.5.2>)

A plan release: past the physical cores, the decoder runs one lazy worker fewer than the cores, and sizes its in-flight window by the workers.

- One lazy worker a core but one, past the cores (\#481). It applies only on the measured profile (10 cores, as on the i9-10900K).
- The toolchain is pinned to rustc 1.99.1 (\#504) for CI, the bench, the wheels and workstation runs.

**Changes:** the lazy kernel's induction, recorded block by block (induction-trace) ([\#486](<https://github.com/lmmx/mibz2/pull/486>)); past the cores, one lazy worker a core but one, with the window sized to the workers ([\#481](<https://github.com/lmmx/mibz2/pull/481>)); Pin the Rust toolchain to 1.99.1 for CI, bench, wheels and workstation runs ([\#504](<https://github.com/lmmx/mibz2/pull/504>)).

### 0.5.1

16 November 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#051---2026-11-16>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.5.0...v0.5.1>)

A plan release: the decoder now uses one block a thread (workers, with lazy letters) instead of teams at 10 threads, and past the physical cores on machines with 8 to 10 of them.

- Workers at 10 threads (\#462). `WORKERS_BELOW_THREADS` goes from 10 to 11.
- One lazy worker a core past the cores (\#468). With more threads than physical cores, on a machine with 8 to 10 of them, the plan is `Workers(physical cores)`: the hyperthreads stay idle, because two workers on one core share its L1 and L2.
- So on the i9, one block a thread now runs at every thread count from 6 to 20: eager Cells at 6–7, lazy letters from 8.
- The paired runbook bench, CI (4 CPUs, Xeon Platinum 8370C; run 43836782266, v0.5.0 → master 3dff14b): level, as expected for changes that only act at 10 threads or more.
- Research features reviewed: `memory-lever`'s run is read and its lazy arm promoted, so it is to go in its own PR; `trace`, `wait-stats`, `walk-stats`, `dose` and `phase-counters` stay.

**Changes:** Past the physical cores, on 8 to 10 of them: lazy workers, one a core ([\#468](<https://github.com/lmmx/mibz2/pull/468>)); Workers with lazy letters at 10 threads; teams only past the physical cores ([\#462](<https://github.com/lmmx/mibz2/pull/462>)).

### 0.5.0

16 November 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#050---2026-11-16>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.4.0...v0.5.0>)

- Lazy letters (\#452): from 8 threads, Cells' induction keeps a slot for a letter only while the letter is live, so the pool peaks at about 0.32 of the letters (right-end order) instead of all of them.

**Changes:** Workers invert with lazy letters from 8 threads ([\#452](<https://github.com/lmmx/mibz2/pull/452>)); Lazy letters: just-in-time memory for induction ([\#451](<https://github.com/lmmx/mibz2/pull/451>)).

**Read:** [Lazy letters: just-in-time memory for induction](<https://lmmx.github.io/mibz2/pr-preview/pr-714/blog/2026-11-16-lazy-letters/index.md>)

### 0.4.0

15 November 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#040---2026-11-15>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.3.4...v0.4.0>)

A soundness and memory release.

- The breaking change: `Kernels` can no longer claim a CPU feature from safe code (\#431, below).
- Less memory per block: Cells holds 22% less memory per block than 0.3.4 (\#433), with output, plan and speed unchanged.
- Research features trimmed: \#432 removes `slot24` and trims `phase-counters`.
- The paired runbook bench, CI: v0.3.4 against v0.4.0 (run 43754741304), to follow.
- `Kernels` can no longer claim a feature the CPU lacks from safe code (\#340). Its fields `avx2` and `clmul` were public, so safe code could build `Kernels { avx2: true, .. }` on a CPU without AVX2 and reach `#[target_feature]` code through the public decoders: undefined behaviour.

**Changes:** bottom slots in the dead output buffer, rule positions in the dead top slots ([\#433](<https://github.com/lmmx/mibz2/pull/433>)); Research features: remove slot24, trim phase-counters, retention notes ([\#432](<https://github.com/lmmx/mibz2/pull/432>)); private fields, so safe code can't claim a CPU feature ([\#340](<https://github.com/lmmx/mibz2/pull/340>), [\#431](<https://github.com/lmmx/mibz2/pull/431>)).

### 0.3.4

15 November 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#034---2026-11-15>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.3.3...v0.3.4>)

A plan release: the team threshold moves (\#425, docs/RELEASING.md §5a, \#426).

- The decision. The 9-thread CPU cost was over the rule's 10%, so the move went to Louis, who chose teams from 10.
- The PR's own CI bench (run 43732867824, AMD EPYC 9V45, 4 vCPU on 2 cores, 10 paired rounds), \#425's head against its base: CI's runners have 2 physical cores, so both arms run the same plan and the bench cannot see the change.
- The release's CI bench (run 43737282330, Intel Xeon Platinum 8573C, 4 vCPU on 2 cores, 10 paired rounds), v0.3.3 (1244d1a9) → master e18ccafc, the v0.3.4 tag commit.

**Changes:** Parallel plan: workers below 10 threads on their own cores; teams from 10 ([\#425](<https://github.com/lmmx/mibz2/pull/425>)); Bench, tooling and runbook changes: the crossover recipe, queued ([\#422](<https://github.com/lmmx/mibz2/pull/422>), [\#424](<https://github.com/lmmx/mibz2/pull/424>), [\#423](<https://github.com/lmmx/mibz2/pull/423>), [\#428](<https://github.com/lmmx/mibz2/pull/428>), [\#426](<https://github.com/lmmx/mibz2/pull/426>), [\#421](<https://github.com/lmmx/mibz2/pull/421>)).

### 0.3.3

15 November 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#033---2026-11-15>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.3.2...v0.3.3>)

A memory-traffic release for Cells.

- The census (\#414, the bench corpus `wikidata`, 431 blocks, 196 admitted by Cells, counts and no timing): slot bytes in both orders 443.6 → 310.5 MB (0.700), 0.63 B less per output byte; the induction's buffer lines (slots, arena words and rules used) 9.64 M → 7.56 M (0.784).
- The PR's own CI bench (run 43719302600, AMD EPYC 9V45, 4 vCPU on 2 cores, 10 paired rounds), \#414's head against its base: `wikidata` CPU −3.3% (−5.8, −0.8) at 4 threads, and amplification at 4 threads −3.2% (−5.6, −0.8); wall −0.1%, −0.2% and −1.8% at 1, 2 and 4 threads, each interval including no change.
- The release's CI bench (run 43723600944, AMD EPYC 9V45, 4 vCPU on 2 cores, 10 paired rounds), v0.3.2 (42c2aa6e) → master 898535524, which differs from the v0.3.3 tag only in the version bump.

**Changes:** Cells induction: 8-byte bottom slots, a letter's length and word read through its top slot ([\#414](<https://github.com/lmmx/mibz2/pull/414>)); Queue, record and docs changes: 0.3.2's release-scaling queue entry and record ([\#415](<https://github.com/lmmx/mibz2/pull/415>), [\#417](<https://github.com/lmmx/mibz2/pull/417>), [\#416](<https://github.com/lmmx/mibz2/pull/416>), [\#419](<https://github.com/lmmx/mibz2/pull/419>), [\#420](<https://github.com/lmmx/mibz2/pull/420>)).

**Read:** [Removing duplicated interval lengths and text references](<https://lmmx.github.io/mibz2/pr-preview/pr-714/blog/2026-11-15-bottom-slots/index.md>)

### 0.3.2

15 November 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#032---2026-11-15>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.3.1...v0.3.2>)

A memory release for Cells.

- The census (\#408, the bench corpus `wikidata`, 431 blocks, 196 admitted by Cells): slots touched 31.5 M → 22.2 M (−29.7%).
- The paired runbook bench, CI (run 43717843002, Intel Xeon Platinum 8573C, 4 vCPU on 2 cores, 10 paired rounds), v0.3.1 → master: `wikidata` peak RSS 33 → 30 MB, 75 → 62 MB and 118 → 97 MB at 1, 2 and 4 threads (−9.6%, −14.6% and −14.5%); wall −0.2%, +0.5% and −0.4%, each interval including no change.

**Changes:** Cells induction: recycle the dead front slot instead of taking a fresh one ([\#408](<https://github.com/lmmx/mibz2/pull/408>)).

**Read:** [Reduce, reuse, recycle: hand-me-down slots for every fragment of text](<https://lmmx.github.io/mibz2/pr-preview/pr-714/blog/2026-11-15-slot-recycling/index.md>)

### 0.3.1

14 November 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#031---2026-11-14>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.3.0...v0.3.1>)

A memory-traffic release for Cells.

- The paired runbook bench, CI (run 43646276187, AMD EPYC 9V45, 4 vCPU), v0.3.0 → master: `wikidata` −4.3% (−7.5, −1.0), −3.1% (−5.4, −0.6) and −4.0% (−6.0, −2.0) at 1, 2 and 4 threads; peak RSS −4.5%, −3.0% and −12.7%.
- The i9-10900K, forced Workers + Cells: 20 B ÷ 24 B wall 0.980 at 1 thread, 0.957 at 4, 0.926 at 6 and 0.911 at 10.
- Why: the memory-dose and working-set tests (\#387, \#390) showed that Workers + Cells is bound by DRAM traffic at 4 threads and up: 1.25 DRAM bytes per output byte with one block in flight, 15.5 at 4, 24.2 at 10.

**Changes:** the 20-byte slot (buffer diet step 1), and bench-cells-diet ([\#391](<https://github.com/lmmx/mibz2/pull/391>)); Cells memory-dose experiment: dose feature, recipe and analysis ([\#387](<https://github.com/lmmx/mibz2/pull/387>)).

### 0.3.0

14 November 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#030---2026-11-14>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.2.9...v0.3.0>)

A speed release for Wikidata-like data on up to 4 decoding threads.

- The paired runbook bench, CI (run 43604267716, Intel Xeon Platinum 8573C, 4 vCPU), v0.2.9 → master: `wikidata` −11.9% (−13.6, −10.2), −10.1% (−11.1, −9.0) and −10.3% (−11.9, −8.8) at 1, 2 and 4 threads; no other corpus slower in any cell.
- The gate it merged under: Wikidata decided.
- Downstream: wikidata-pq\#225 (`BZ2_THREADS_CAP` 4, so the split's decoder runs Cells): the split's release ETA 1h20m56s → 1h16m46s (−5.3% ±1.9%), parity passed.

**Changes:** initialise every MTF base with Rust 1.100 Clippy; iterate checked packet path for Rust 1.100 Clippy; account for AVX2 run fill slack in retained tables; validate retained allocations by worker route history; Cells (Rauzy–Veech induction) as the Workers plan's inverse, with a sparse admission gate ([\#383](<https://github.com/lmmx/mibz2/pull/383>)).

### 0.2.9

10 November 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#029---2026-11-10>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.2.8...v0.2.9>)

A speed release for many-core machines.

- Team paths read run records (\#359). Since 0.2.8 the one-thread `Segmented` kernel has built its backward table from run records, without writing the last column.
- Admission by the bound and the share of run symbols (\#353). A short block of high-entropy data keeps the column instead of taking the run records, which cost it 10–11% more instructions.
- The move-to-front list in one page (\#358). `mtf_checkpoints_avx2`'s list is aligned to 512 bytes, so it no longer straddles a page in 4 of its 64 possible layouts.
- Harness only (\#356). `ab summary` reads a record's top-level fields only (bench 3.8.1).
- The paired run against 0.2.8 (CI, Intel Xeon Platinum 8370C, 4 CPUs on 2 cores, run 42858124938, 10 paired rounds): no change detected at 1, 2 and 4 threads on any corpus.

### 0.2.8

10 November 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#028---2026-11-10>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.2.7...v0.2.8>)

A speed release. On the Wikidata sample a decode is 12–19% faster on CI's runner, from a new path for blocks of few symbols (\#324) and a faster table build (\#312).

- Blocks of few symbols (\#324). On a block with at most one symbol per two rows of its bound, the default `Segmented` kernel, which decodes a block on one thread, no longer writes the last column.
- The table build (\#312). The T-vector build cuts its 16-row windows into runs, with a cheap classifier for high-entropy windows.
- The i9-10900K (\#316). At 10 threads on exactly that machine (its CPU model and topology, AVX2 and CLMUL, Linux x86-64, an unlimited quota), the default is now the guarded service core.
- Harness only (\#350). The bench harness can hold a spinner on a spare CPU for a workstation run (`ab run --hold-cpu`, bench 3.8.0).

**Changes:** run-record front and LF table on blocks of few symbols, walked backward ([\#324](<https://github.com/lmmx/mibz2/pull/324>)); guarded service core as the default on the i9-10900K, target-gated ([\#316](<https://github.com/lmmx/mibz2/pull/316>)); the build's windows cut into runs, with a cheap classifier for high-entropy windows ([\#312](<https://github.com/lmmx/mibz2/pull/312>)); count the cooperative waits (wait-stats feature, off by default); the spin hypothesis for the i9 slow mode not confirmed ([\#311](<https://github.com/lmmx/mibz2/pull/311>)); optional held spinner for workstation runs (W1) ([\#350](<https://github.com/lmmx/mibz2/pull/350>)).

### 0.2.7

9 November 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#027---2026-11-09>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.2.6...v0.2.7>)

A safety release. A parallel `BzReader` whose source panics now fails its next read, instead of waiting for ever (\#278).

- The fix (\#278). A parallel reader reads its source on a thread of its own.
- Tests only (\#275, Guardian D047; and \#273, D042). A deterministic test unwinds a pipelined team's worker at three points and checks what the reader gets, with a mutant that the test kills, a Miri filter, and the provenance of the pipeline's fuzz entry.

**Changes:** a parallel BzReader's source panicking cancels the decode instead of stranding the reader ([\#278](<https://github.com/lmmx/mibz2/pull/278>)); deterministic worker-unwind test for M12, Miri filter, provenance (Guardian D047) ([\#275](<https://github.com/lmmx/mibz2/pull/275>)); pipeline fuzz candidates and invariant-mutation evidence for 0.2.6's default (Guardian D042) ([\#273](<https://github.com/lmmx/mibz2/pull/273>)).

### 0.2.6

7 November 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#026---2026-11-07>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.2.5...v0.2.6>)

On a machine with 10 physical cores and a 20 MiB L3, such as the i9-10900K, the decoder now pipelines its three teams of three at 10 or more threads: each team's leader undoes one block's RLE1 and checks its CRC, and runs the next block's move-to-front, while its helpers build and walk the block between (\#247).

**Read:** [Pipelined teams: how mibz2 0.2.6 decodes 7% faster on ten cores](<https://lmmx.github.io/mibz2/pr-preview/pr-714/blog/2026-11-07-pipelined-teams/index.md>)

### 0.2.5

6 November 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#025---2026-11-06>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.2.4...v0.2.5>)

On a machine with 10 physical cores and a 20 MiB L3, such as the i9-10900K, the decoder now splits 10 or more threads into three teams of three, where it used to split them into four teams of two (\#234).

**Changes:** three teams of three at 10 threads on a 10-core, 20 MiB-L3 machine ([\#234](<https://github.com/lmmx/mibz2/pull/234>)).

**Read:** [Why mibz2 stops scaling at eight threads, and why three teams help](<https://lmmx.github.io/mibz2/pr-preview/pr-714/blog/2026-11-07-why-mibz2-stops-scaling/index.md>)

### 0.2.4

5 November 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#024---2026-11-05>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.2.3...v0.2.4>)

The move-to-front decode keeps the first 32 bytes of its list in two registers, so a move whose index is below 32 no longer writes the list to memory and reads it back.

**Changes:** libbzip2's two-level list, its first 32 bytes in two registers ([\#207](<https://github.com/lmmx/mibz2/pull/207>)).

### 0.2.3

5 November 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#023---2026-11-05>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.2.2...v0.2.3>)

### 0.2.2

5 November 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#022---2026-11-05>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.2.1...v0.2.2>)

The inverse BWT's walk, the move-to-front decode and the Huffman scanner are each faster, and a decode keeps more blocks in flight per thread.

**Changes:** walk blocks under MIN\_SEGMENTED plainly at every setting, as the decoder does ([\#200](<https://github.com/lmmx/mibz2/pull/200>)); complete traces under a team plan; --teams K, checked in CI ([\#116](<https://github.com/lmmx/mibz2/pull/116>)); the symbol loop keeps the bit reader's state in registers ([\#204](<https://github.com/lmmx/mibz2/pull/204>)); the staged walk tests each lane's start flag, without the deferred flags ([\#181](<https://github.com/lmmx/mibz2/pull/181>)); the default window is 4 blocks a thread, not 2 ([\#180](<https://github.com/lmmx/mibz2/pull/180>)); the move-to-front list holds the bytes, not their indices ([\#172](<https://github.com/lmmx/mibz2/pull/172>)); the room walk tests its flags once an offset and stages its bytes in one tile ([\#171](<https://github.com/lmmx/mibz2/pull/171>)); a run of at most 64 bytes is two vector stores, not a memset call ([\#170](<https://github.com/lmmx/mibz2/pull/170>)); Segmented inverts in place over ll; rle is never allocated ([\#164](<https://github.com/lmmx/mibz2/pull/164>)); the room walk's pass steps eight lanes a turn, handovers after ([\#157](<https://github.com/lmmx/mibz2/pull/157>)).

**Read:** [Huffman decoding in mibz2: keeping the bit buffer in a register](<https://lmmx.github.io/mibz2/pr-preview/pr-714/blog/2026-11-05-huffman-bit-buffer/index.md>)

### 0.2.1

30 October 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#021---2026-10-30>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.2.0...v0.2.1>)

The one-core inverse-BWT table build now writes each entry in one pass.

**Changes:** the one-core build in one pass ([\#114](<https://github.com/lmmx/mibz2/pull/114>)).

### 0.2.0

29 October 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#020---2026-10-29>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.1.6...v0.2.0>)

The parallel decoder now shares each block between two threads on machines that have the cores for it.

- `parallel::Plan`, `StreamDecoder::planned` and `BzReader::parallel_planned` (\#105): choose the plan by hand.
- `mtf::mtf_checkpoints` and `mtf::CHECKPOINT` (\#101).

**Changes:** Cooperative inverse BWT, chosen automatically ([\#101](<https://github.com/lmmx/mibz2/pull/101>), [\#103](<https://github.com/lmmx/mibz2/pull/103>), [\#104](<https://github.com/lmmx/mibz2/pull/104>), [\#105](<https://github.com/lmmx/mibz2/pull/105>), [\#106](<https://github.com/lmmx/mibz2/pull/106>)).

### 0.1.6

29 October 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#016---2026-10-29>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.1.5...v0.1.6>)

An instrumentation release. Decoding is unchanged from 0.1.5: in one paired run of the two tags on one runner, every corpus decodes to the same bytes, no change in wall time was detected at any thread count, and the resource assessment found no change in CPU or peak memory.

**Changes:** `trace`: the parallel decoder's own timeline ([\#53](<https://github.com/lmmx/mibz2/pull/53>), [\#70](<https://github.com/lmmx/mibz2/pull/70>)); The walk sweep's controls ([\#41](<https://github.com/lmmx/mibz2/pull/41>), [\#49](<https://github.com/lmmx/mibz2/pull/49>)); 0.1.5's walk signatures kept ([\#97](<https://github.com/lmmx/mibz2/pull/97>)).

### 0.1.5

27 October 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#015---2026-10-27>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.1.4...v0.1.5>)

The inverse BWT's walk now runs in passes at a shared output offset, after libsais's chain decode.

- The journal: the external research reports and a source-grounded synthesis of them (\#33).

**Changes:** The walk in passes with a shared offset ([\#37](<https://github.com/lmmx/mibz2/pull/37>)); Bench 1.2.0: a paired per-stage table ([\#35](<https://github.com/lmmx/mibz2/pull/35>)).

### 0.1.4

27 October 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#014---2026-10-27>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.1.3...v0.1.4>)

**Changes:** 16 equal last-column bytes linked with one 16-wide store ([\#7](<https://github.com/lmmx/mibz2/pull/7>), [\#32](<https://github.com/lmmx/mibz2/pull/32>)).

### 0.1.3

27 October 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#013---2026-10-27>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.1.2...v0.1.3>)

**Changes:** every alternative on master's run, single-threaded decoders shown as their 1-thread runs ([\#26](<https://github.com/lmmx/mibz2/pull/26>)); up to two symbols a table lookup, four lookups a refill ([\#8](<https://github.com/lmmx/mibz2/pull/8>), [\#24](<https://github.com/lmmx/mibz2/pull/24>)); each lane's next T-vector entry prefetched as soon as its row is known, 32 lanes and 512 segments ([\#21](<https://github.com/lmmx/mibz2/pull/21>)).

### 0.1.2

27 October 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#012---2026-10-27>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.1.1...v0.1.2>)

**Changes:** the segment starts flagged in their own T-vector entries, raw per-lane write pointers, no idle-lane test until the last segments ([\#12](<https://github.com/lmmx/mibz2/pull/12>)).

### 0.1.1

27 October 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#011---2026-10-27>) · [its diff](<https://github.com/lmmx/mibz2/compare/v0.1.0...v0.1.1>)

### 0.1.0

27 October 2026 · [the CHANGELOG](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#010---2026-10-27>) · [its diff](<https://github.com/lmmx/mibz2/releases/tag/v0.1.0>)

**Changes:** the Python package: `decompress`, `open` (a buffered binary file over a path or a binary file object) and `BZ2Reader.verified`, true only once a read reaches the end with every CRC matched; `DecodeError`; abi3 wheels for Python 3.10+ on Linux x86-64 and aarch64, macOS x86-64 and arm64, and Windows x64 ([\#1](<https://github.com/lmmx/mibz2/pull/1>)).
