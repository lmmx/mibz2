# History

Loading the chart…

How mibz2's decoding speed, CPU time and peak memory have changed, merge by merge. The figures are measured on GitHub's runners ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-668/benchmarks/index.md>)).

- **The chart chains each merge's own benchmark.** A pull request's benchmark runs its head against its base on one runner, both builds, for 10 rounds with their order alternating. Each merge keeps that paired change ([`bench/links.py`](<https://github.com/lmmx/mibz2/blob/master/bench/links.py>)), and the chart multiplies the changes, from the first merge whose arms ran in balanced order (bench 3.0.0). The band is the product's 95% interval: the links' intervals added in quadrature, on the log scale.
- **A merge that left the decoder alone** (the harness, docs, CI) is an A/A pair. It is drawn hollow and adds nothing.
- **A decoder change with no paired run** breaks the chain into a new segment. No gap is bridged.
- **What the chain assumes:** that a relative change measured on one runner holds on another (the merges ran on several CPU models, each named), and that the links' errors are independent.
- **Before bench 3.0.0** the head ran first in every round, which favoured it by about 1.8% at 1 thread in an A/A run. Those merges' changes are listed in the table below the charts, not chained.
- **Releases past 4 threads.** On wikidata's throughput, the chart also offers 8 to 20 threads, from the release-scaling runs below, on one i9-10900K. Those figures exist only at releases, so each is a diamond at the last merge its release tag contains, joined to the next by a dotted line that is not measured. They are modelled from release-against-release runs (the **All releases** inference below). They share the chain's axis and its meaning, a change since the start with up faster, but start from the oldest release and run on another machine. At 1, 2 and 4 threads they overlay CI's chain; above 4, CI has no figure and they stand alone. **All** overlays every thread count in one chart, a colour each along the releases' ramp, low to high.
- **Relative or Absolute.** The chart shows change since the start by default. **Absolute** shows each figure in its own units instead: throughput in MB/s of output, CPU time in CPU seconds (user + sys) for the corpus's decode, peak RSS in MB. CI's chain has no absolute scale of its own, because its merges ran on several runner CPUs and it multiplies ratios (of wall, CPU time and peak RSS alike). It borrows one runner model's scale: the model with the most master records at that corpus and thread count, through its newest record at a chained merge, which the chart rings and names. Every other merge is that record carried by the chain's ratios. The band is the links' interval between the merge and the anchor, with the record's own. Throughput and CPU time scale with the runner's speed, so their levels are that runner's. Peak RSS hardly depends on the runner: at a given corpus and thread count the runner models' newest records mostly agree within a few percent (`text` aside, whose content changes), and the note under the chart gives their range for each view. The i9-10900K's releases are in MB/s on the i9, another machine on another scale, so beside CI's chain they get a panel of their own. In **All** every thread count sits at its own level on the i9. The release-scaling runs record throughput only, so CPU time and peak RSS have no releases and no **All**.
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
