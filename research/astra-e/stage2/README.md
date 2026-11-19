# Second stage comparison

The generator takes the fixed sample, warm-up, checker and rotation from example commit `0f25dee737432feafb03b8b14dc10afbf5c141ad`. The seven arms are the shipped eager and Lazy Cells kernels, opend16, the one-edge lookahead, dictionary offsets, compact F-coordinate construction, and immediate two-/three-byte words. Each fresh process runs 21 measured passes. Each arm visits every position three times on every sampled block. The Linux thread CPU clock surrounds the wall clock and helps identify descheduling. Both clocks include construction and emission from ready run records. The same independently generated Lazy reference is checked after every call. Each run has 1,617 measured outputs and 77 warmed outputs, on the same 11 eligible block ordinals as stage 1.

The first process launched at checkout `b88a02738799f8e021db356157ef4239bbea911e` with the new harness files present locally. Their committed copies at `cec3a58635582885d9062c92e65a363c5fdff599` have matching hashes in [manifest.txt](manifest.txt). A second process launched at `210125c338a045e3b4047194267d967108660b8a`. Both builds produced source SHA-256 `c8eb7cc5a73a955428c226cb981d9beb2dc511108c31af89b8eebcfe03199b79` and executable SHA-256 `1e5a0b46b1dbe6176bf6f548922263040a7d88c343bbb4697de2a08f7e7e3ed3`. Both used the research container, during the exploratory scorer campaign. Timing uncertainty across processes matters.

| Arm over opend16 | First wall | Repeat wall | First thread CPU | Repeat thread CPU |
| --- | ---: | ---: | ---: | ---: |
| Eager | 0.733 | 0.820 | 0.695 | 0.791 |
| Compact F | 1.143 | 0.984 | 1.051 | 0.972 |
| Dictionary | 1.030 | 0.961 | 0.984 | 0.980 |
| Lookahead | 0.943 | 0.981 | 0.929 | 1.005 |
| Immediate short words | 1.052 | 0.969 | 0.946 | 0.981 |

Lookahead’s first wall interval was 0.903–0.984 and its thread CPU interval was 0.896–0.964. The repeat intervals were 0.914–1.054 and 0.956–1.056, respectively. Summing the per-block wall medians yields lookahead/opend ratios of 0.953 and 1.001. The first gain did not reproduce with comparable precision in a second process. Eager remains the local stage leader. The two streams of clocks and both processes are retained separately in [first raw laps](laps.csv), [first comparison](summary.csv), [repeat raw laps](repeat/laps.csv), [repeat comparison](repeat/summary.csv) and [repeat provenance](repeat/manifest.txt). The intervals treat the 21 pass sums within each process as repeated observations. They provide no i9 estimate.

Counting the same eleven inputs outside the timer, short words stored 47,847 binary rules against 74,180 in the historical width-16 queued endpoint, a reduction of 26,333 (35.50%). The probe and historical census agreed on steps, total representative spelling bytes, retained rows and block size at every sampled ordinal. The reduced rule count had an uncertain effect on paired wall time. [Rule counts](rules.csv) · [Raw probe](word-probe.txt)

```sh
d=research/astra-e/stage2
bash "$d/run.sh" corpora/wikidata.bz2 > trial.csv 2> trial-manifest.txt
python3 "$d/report.py" trial.csv
python3 "$d/rules.py" "$d/target/word-probe.txt"
```
