# History

Loading the chart…

How mibz2's decoding speed, CPU time and peak memory have changed, merge by merge. The figures are measured on GitHub's runners ([methodology](<https://lmmx.github.io/mibz2/pr-preview/pr-382/benchmarks/index.md>)).

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
