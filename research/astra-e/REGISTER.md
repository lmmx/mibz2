# Astra E best-so-far register

The reference is 0.5.6 LazyCells in the same shipped worker loop and 4W slots. The full Wikidata input decodes to 209,700,196 bytes per copy. Each gate result and timing scope below belongs to one immutable build. The local Xeon measurements do not establish the i9 target. [Frozen target rule](workstation/README.md) · [Historical i9 frontier](prior.md) · [Astra E draft PR and CI](https://github.com/lmmx/mibz2/pull/607/checks)

## Local leaders by tested design

These are the lowest recorded point estimates within each group of experiments with matching scorer revision, corpus hash, compiler, host, design and worker count. Each experiment pairs its candidate with its own lazy control. Between-experiment order remains uncertain when timing variation is large. A shared quiet comparison is needed to establish which candidate wins head to head. Pilot runs may overlap other work. Confirmation runs have their own rows. The historical control stays separate. RSS leaders remain visible when timing and memory favour different candidates.

| Scorer / design / W | Lowest wall estimate vs lazy [95% CI] | Lowest median RSS ratio vs lazy |
|---|---|---|
| `604dac3` confirmation, grid 1,4, 9r ×2, settle 2s, full, 3 arms, **W1** | v1-short-confirm · 0.876 [0.757, 1.013] | v1-edge-confirm · 0.897 |
| `604dac3` confirmation, grid 1,4, 9r ×2, settle 2s, full, 3 arms, **W4** | v1-edge-confirm · 0.743 [0.621, 0.888] | v1-edge-confirm · 0.732 |
| `9c31461` local score, grid 1,4, 5r ×1, settle 2s, full, 3 arms, **W1** | hierarchy-f4 · 1.940 [1.080, 3.487] | hierarchy-f4 · 0.878 |
| `9c31461` local score, grid 1,4, 5r ×1, settle 2s, full, 3 arms, **W4** | hierarchy-f4 · 2.159 [1.407, 3.313] | hierarchy-f4 · 0.819 |
| `9c31461` local score, grid 1,4, 6r ×2, settle 2s, full, 3 arms, **W1** | delimiter-f2b · 1.460 [1.210, 1.762] | tableless-f1b · 0.951 |
| `9c31461` local score, grid 1,4, 6r ×2, settle 2s, full, 3 arms, **W4** | delimiter-f2b · 1.415 [1.176, 1.702] | tableless-f1b · 0.861 |
| `9c31461` pilot, grid 1,4, 3r ×1, settle 0s, quick, 3 arms, **W1** | lookahead-pilot · 0.560 [0.183, 1.714] | compact-pilot · 0.733 |
| `9c31461` pilot, grid 1,4, 3r ×1, settle 0s, quick, 3 arms, **W4** | combined-ahead-pilot · 0.655 [0.360, 1.192] | short-words-pilot · 0.566 |
| `9c31461` settled confirmation, grid 1,2,3,4, 6r ×4, settle 2s, full, 3 arms, **W1** | short-words-confirm · 0.923 [0.659, 1.292] | short-words-confirm · 0.968 |
| `9c31461` settled confirmation, grid 1,2,3,4, 6r ×4, settle 2s, full, 3 arms, **W2** | short-words-confirm · 1.003 [0.735, 1.369] | short-words-confirm · 0.950 |
| `9c31461` settled confirmation, grid 1,2,3,4, 6r ×4, settle 2s, full, 3 arms, **W3** | short-words-confirm · 1.099 [0.664, 1.818] | short-words-confirm · 0.924 |
| `9c31461` settled confirmation, grid 1,2,3,4, 6r ×4, settle 2s, full, 3 arms, **W4** | short-words-confirm · 0.766 [0.610, 0.962] | short-words-confirm · 0.873 |

## Partial proxy · conjugate-w7

Build `e12e5f63173ee7cc6fcc413510ee76cfee0c1eb1` used scorer `9c31461df7bef24cecf79348545b27ee26846835`. Rows identify the actual stream-block indices. This single-worker screen uses the listed LLC share and latency settings. It omits the scorer's full reference-arm calibration checks. [Manifest](proxy/conjugate-w7/manifest.json)

| Stream blocks | W model | Arm | Instructions/B | L2 / L3 / DRAM B/B | Dependent DRAM / past-L2 links/B | DRAM MLP | Raw |
|---|---:|---|---:|---|---|---:|---|
| 41,85 | 7 | candidate | 21.582 | 55.552 / 48.310 / 16.239 | 0.00662 / 0.22320 | 2.927 | [receipt](proxy/conjugate-w7/density-pair-candidate.json) |
| 41,85 | 7 | lazy | 23.466 | 65.672 / 55.678 / 17.085 | 0.00661 / 0.20641 | 2.926 | [receipt](proxy/conjugate-w7/density-pair-lazy.json) |
| 41,85 | 7 | opend | 20.879 | 57.037 / 49.455 / 17.434 | 0.00661 / 0.22209 | 2.923 | [receipt](proxy/conjugate-w7/density-pair-opend.json) |
| 1 | 7 | candidate | 16.205 | 18.841 / 11.905 / 6.674 | 0.00004 / 0.03206 | 2.593 | [receipt](proxy/conjugate-w7/frozen-candidate.json) |
| 1 | 7 | lazy | 19.475 | 36.077 / 23.576 / 6.987 | 0.00004 / 0.01466 | 2.603 | [receipt](proxy/conjugate-w7/frozen-lazy.json) |
| 1 | 7 | opend | 14.553 | 19.860 / 13.029 / 7.419 | 0.00004 / 0.03162 | 2.595 | [receipt](proxy/conjugate-w7/frozen-opend.json) |

## Partial proxy · narrow-short-ahead-w7

Build `0694800bf4b2ef5ece720f7c43b8ab0ed7dced9b` used scorer `9c31461df7bef24cecf79348545b27ee26846835`. Rows identify the actual stream-block indices. This single-worker screen uses the listed LLC share and latency settings. It omits the scorer's full reference-arm calibration checks. [Manifest](proxy/narrow-short-ahead-w7/manifest.json)

| Stream blocks | W model | Arm | Instructions/B | L2 / L3 / DRAM B/B | Dependent DRAM / past-L2 links/B | DRAM MLP | Raw |
|---|---:|---|---:|---|---|---:|---|
| 41,85 | 7 | candidate | 20.633 | 56.075 / 48.654 / 17.001 | 0.00662 / 0.22218 | 2.924 | [receipt](proxy/narrow-short-ahead-w7/density-pair-candidate.json) |
| 41,85 | 7 | lazy | 23.466 | 65.675 / 55.687 / 17.086 | 0.00661 / 0.20637 | 2.927 | [receipt](proxy/narrow-short-ahead-w7/density-pair-lazy.json) |
| 41,85 | 7 | opend | 20.879 | 57.037 / 49.453 / 17.435 | 0.00661 / 0.22208 | 2.923 | [receipt](proxy/narrow-short-ahead-w7/density-pair-opend.json) |
| 1 | 7 | candidate | 14.228 | 18.124 / 12.411 / 7.119 | 0.00004 / 0.03182 | 2.589 | [receipt](proxy/narrow-short-ahead-w7/frozen-candidate.json) |
| 1 | 7 | lazy | 19.475 | 36.069 / 23.580 / 6.988 | 0.00004 / 0.01466 | 2.608 | [receipt](proxy/narrow-short-ahead-w7/frozen-lazy.json) |
| 1 | 7 | opend | 14.553 | 19.860 / 13.030 / 7.419 | 0.00004 / 0.03159 | 2.598 | [receipt](proxy/narrow-short-ahead-w7/frozen-opend.json) |

Reproduction commands below replay the historical scorer on the recorded master commit. For a new scoring round, use `bash research/astra-e/score.sh <new-label> <candidate> ...` to refresh master first. Per-run `assessment.json` may record `kind`, `assessment` and `verification` (including independent run links). Regenerate with `python3 research/astra-e/register.py`.

## anchors-pilot

Pilot. **PASS** on 98 scorer cases (quick). [Per-case results](results/anchors-pilot/score/check.jsonl) · [manifest](results/anchors-pilot/manifest.txt) · [score](results/anchors-pilot/score/summary.txt)

The build tree, including sibling modules, is [`38ad5600dfe4e621d2c44d04cc460e375bfbe522`](https://github.com/lmmx/mibz2/tree/38ad5600dfe4e621d2c44d04cc460e375bfbe522). [Candidate entry](https://github.com/lmmx/mibz2/blob/38ad5600dfe4e621d2c44d04cc460e375bfbe522/research/astra-e/anchors/candidate.rs). Fetched master `9c31461df7bef24cecf79348545b27ee26846835`, scorer path `9c31461` (no version field printed). Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

| W | Pairs | Candidate / lazy [95% CI] | opend / lazy [95% CI] | Candidate / opend¹ | Median peak KiB, candidate / lazy / opend | Candidate / lazy peak |
|---:|---:|---|---|---:|---|---:|
| 1 | 3 | 0.938 [0.811, 1.086] | 0.964 [0.900, 1.032] | 0.974 | 46,976 / 61,320 / 45,768 | 0.766 |
| 4 | 3 | 0.877 [0.649, 1.185] | 0.752 [0.302, 1.874] | 1.166 | 86,808 / 133,464 / 83,672 | 0.650 |

¹ Ratio of paired geometric means sharing the same lazy observations. [Individual runs and process peaks](results/anchors-pilot/score/runs.jsonl)

This score has no completed instruction, cache-transfer, dependency or overlap model.

Independent candidate CI and target evidence have not been recorded for this run in this register.

Reproduce in a fresh checkout of the build tree above.

```sh
git checkout 9c31461df7bef24cecf79348545b27ee26846835 -- bench/moonshot .just Justfile
bash research/astra-e/prepare.sh research/astra-e/anchors/candidate.rs
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/anchors/candidate.rs --workers 1,4 --rounds 3 --settle 0 --quick --with-opend --no-proxy
```

## combined-ahead-pilot

Pilot. **PASS** on 98 scorer cases (quick). [Per-case results](results/combined-ahead-pilot/score/check.jsonl) · [manifest](results/combined-ahead-pilot/manifest.txt) · [score](results/combined-ahead-pilot/score/summary.txt)

The build tree, including sibling modules, is [`f1b962aaefe51e5bf56130ee1a9c934a9d160a7d`](https://github.com/lmmx/mibz2/tree/f1b962aaefe51e5bf56130ee1a9c934a9d160a7d). [Candidate entry](https://github.com/lmmx/mibz2/blob/f1b962aaefe51e5bf56130ee1a9c934a9d160a7d/research/astra-e/narrow-short/ahead.rs). Fetched master `9c31461df7bef24cecf79348545b27ee26846835`, scorer path `9c31461` (no version field printed). Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

| W | Pairs | Candidate / lazy [95% CI] | opend / lazy [95% CI] | Candidate / opend¹ | Median peak KiB, candidate / lazy / opend | Candidate / lazy peak |
|---:|---:|---|---|---:|---|---:|
| 1 | 3 | 0.797 [0.325, 1.950] | 0.892 [0.328, 2.431] | 0.893 | 45,936 / 60,932 / 45,848 | 0.754 |
| 4 | 3 | 0.655 [0.360, 1.192] | 0.624 [0.285, 1.364] | 1.051 | 81,984 / 135,476 / 81,908 | 0.605 |

¹ Ratio of paired geometric means sharing the same lazy observations. [Individual runs and process peaks](results/combined-ahead-pilot/score/runs.jsonl)

This score has no completed instruction, cache-transfer, dependency or overlap model.

Independent candidate CI and target evidence have not been recorded for this run in this register.

Reproduce in a fresh checkout of the build tree above.

```sh
git checkout 9c31461df7bef24cecf79348545b27ee26846835 -- bench/moonshot .just Justfile
bash research/astra-e/prepare.sh research/astra-e/narrow-short/ahead.rs
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/narrow-short/ahead.rs --workers 1,4 --rounds 3 --settle 0 --quick --with-opend --no-proxy
```

## combined-full-proxy

Advisory proxy. **PASS** on 98 scorer cases (quick). [Per-case results](results/combined-full-proxy/score/check.jsonl) · [manifest](results/combined-full-proxy/manifest.txt) · [score](results/combined-full-proxy/score/summary.txt)

The build tree, including sibling modules, is [`b981d78984a195408501ff7fb495f77a1075be02`](https://github.com/lmmx/mibz2/tree/b981d78984a195408501ff7fb495f77a1075be02). [Candidate entry](https://github.com/lmmx/mibz2/blob/b981d78984a195408501ff7fb495f77a1075be02/research/astra-e/narrow-short/ahead.rs). Fetched master `9c31461df7bef24cecf79348545b27ee26846835`, scorer path `9c31461` (no version field printed). Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

| W model | Instructions/B | L2 / L3 / DRAM B/B | Dependent DRAM / past-L2 links/B | DRAM MLP | Calibration |
|---:|---:|---|---|---:|---|
| 1 | 16.836 | 21.530 / 15.085 / 1.380 | 0.00001 / 0.05290 | 2.343 | within envelope |
| 2 | 16.836 | 21.530 / 15.085 / 1.380 | 0.00001 / 0.05288 | 2.355 | within envelope |
| 4 | 16.836 | 21.530 / 15.085 / 1.530 | 0.00001 / 0.05283 | 2.373 | outside |
| 7 | 16.836 | 21.530 / 15.085 / 6.203 | 0.00004 / 0.05238 | 1.914 | within envelope |
| 8 | 16.836 | 21.530 / 15.085 / 7.373 | 0.00004 / 0.05225 | 1.946 | within envelope |
| 10 | 16.836 | 21.530 / 15.085 / 7.873 | 0.00005 / 0.05216 | 2.006 | within envelope |

V0 16-block calibrated point estimates for candidate/opend were .951 at W7, .962 at W8 and .963 at W10. At W10 the same model overstates historical opend/lazy by about 7.5 percentage points. The quick gate passed, and W4 was outside the reference envelope. [Full calibration, admissions and receipts](results/combined-full-proxy/README.md).

v0 designated frozen candidate and matching local source passed the full 338-case local rehearsal. Independent performance belongs to the separate CI and i9 evidence. [CI](https://github.com/lmmx/mibz2/actions/runs/44427324957)

Reproduce in a fresh checkout of the build tree above.

```sh
git checkout 9c31461df7bef24cecf79348545b27ee26846835 -- bench/moonshot .just Justfile
bash research/astra-e/prepare.sh research/astra-e/narrow-short/ahead.rs
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/narrow-short/ahead.rs --target-dir /workspace/work/narrow-proxy/full-target --quick --no-timing --proxy-blocks 16
```

## combined-pilot

Pilot. **PASS** on 98 scorer cases (quick). [Per-case results](results/combined-pilot/score/check.jsonl) · [manifest](results/combined-pilot/manifest.txt) · [score](results/combined-pilot/score/summary.txt)

The build tree, including sibling modules, is [`8775403b45da1ceafb8ba7ca5fc441c4fced5314`](https://github.com/lmmx/mibz2/tree/8775403b45da1ceafb8ba7ca5fc441c4fced5314). [Candidate entry](https://github.com/lmmx/mibz2/blob/8775403b45da1ceafb8ba7ca5fc441c4fced5314/research/astra-e/narrow-short/candidate.rs). Fetched master `9c31461df7bef24cecf79348545b27ee26846835`, scorer path `9c31461` (no version field printed). Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

| W | Pairs | Candidate / lazy [95% CI] | opend / lazy [95% CI] | Candidate / opend¹ | Median peak KiB, candidate / lazy / opend | Candidate / lazy peak |
|---:|---:|---|---|---:|---|---:|
| 1 | 3 | 0.816 [0.310, 2.149] | 1.030 [0.345, 3.074] | 0.792 | 45,796 / 61,168 / 45,664 | 0.749 |
| 4 | 3 | 0.946 [0.400, 2.236] | 0.877 [0.532, 1.446] | 1.078 | 77,908 / 128,172 / 81,424 | 0.608 |

¹ Ratio of paired geometric means sharing the same lazy observations. [Individual runs and process peaks](results/combined-pilot/score/runs.jsonl)

This score has no completed instruction, cache-transfer, dependency or overlap model.

Independent candidate CI and target evidence have not been recorded for this run in this register.

Reproduce in a fresh checkout of the build tree above.

```sh
git checkout 9c31461df7bef24cecf79348545b27ee26846835 -- bench/moonshot .just Justfile
bash research/astra-e/prepare.sh research/astra-e/narrow-short/candidate.rs
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/narrow-short/candidate.rs --workers 1,4 --rounds 3 --settle 0 --quick --with-opend --no-proxy
```

## compact-pilot

Pilot. **PASS** on 98 scorer cases (quick). [Per-case results](results/compact-pilot/score/check.jsonl) · [manifest](results/compact-pilot/manifest.txt) · [score](results/compact-pilot/score/summary.txt)

The build tree, including sibling modules, is [`a990f1e3e39b92a616cb1b9fa08a2b4defebd2eb`](https://github.com/lmmx/mibz2/tree/a990f1e3e39b92a616cb1b9fa08a2b4defebd2eb). [Candidate entry](https://github.com/lmmx/mibz2/blob/a990f1e3e39b92a616cb1b9fa08a2b4defebd2eb/research/astra-e/open/compact/candidate.rs). Fetched master `9c31461df7bef24cecf79348545b27ee26846835`, scorer path `9c31461` (no version field printed). Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

| W | Pairs | Candidate / lazy [95% CI] | opend / lazy [95% CI] | Candidate / opend¹ | Median peak KiB, candidate / lazy / opend | Candidate / lazy peak |
|---:|---:|---|---|---:|---|---:|
| 1 | 3 | 1.132 [0.567, 2.257] | 1.046 [0.409, 2.680] | 1.081 | 44,808 / 61,088 / 45,568 | 0.733 |
| 4 | 3 | 0.844 [0.460, 1.550] | 0.779 [0.423, 1.437] | 1.083 | 74,472 / 121,448 / 82,036 | 0.613 |

¹ Ratio of paired geometric means sharing the same lazy observations. [Individual runs and process peaks](results/compact-pilot/score/runs.jsonl)

This score has no completed instruction, cache-transfer, dependency or overlap model.

Independent candidate CI and target evidence have not been recorded for this run in this register.

Reproduce in a fresh checkout of the build tree above.

```sh
git checkout 9c31461df7bef24cecf79348545b27ee26846835 -- bench/moonshot .just Justfile
bash research/astra-e/prepare.sh research/astra-e/open/compact/candidate.rs
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/open/compact/candidate.rs --workers 1,4 --rounds 3 --settle 0 --quick --with-opend --no-proxy
```

## conjugate-pilot

Pilot. **PASS** on 98 scorer cases (quick). [Per-case results](results/conjugate-pilot/score/check.jsonl) · [manifest](results/conjugate-pilot/manifest.txt) · [score](results/conjugate-pilot/score/summary.txt)

The build tree, including sibling modules, is [`e12e5f63173ee7cc6fcc413510ee76cfee0c1eb1`](https://github.com/lmmx/mibz2/tree/e12e5f63173ee7cc6fcc413510ee76cfee0c1eb1). [Candidate entry](https://github.com/lmmx/mibz2/blob/e12e5f63173ee7cc6fcc413510ee76cfee0c1eb1/research/astra-e/open/candidate.rs). Fetched master `9c31461df7bef24cecf79348545b27ee26846835`, scorer path `9c31461` (no version field printed). Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

| W | Pairs | Candidate / lazy [95% CI] | opend / lazy [95% CI] | Candidate / opend¹ | Median peak KiB, candidate / lazy / opend | Candidate / lazy peak |
|---:|---:|---|---|---:|---|---:|
| 1 | 3 | 0.965 [0.660, 1.413] | 0.907 [0.831, 0.989] | 1.064 | 44,552 / 58,876 / 45,768 | 0.757 |
| 4 | 3 | 1.018 [0.705, 1.470] | 0.923 [0.677, 1.259] | 1.102 | 71,052 / 117,804 / 77,652 | 0.603 |

¹ Ratio of paired geometric means sharing the same lazy observations. [Individual runs and process peaks](results/conjugate-pilot/score/runs.jsonl)

This score has no completed instruction, cache-transfer, dependency or overlap model.

Independent CI measured identical base and candidate executables on two host types. At four workers conjugation reduced wall time against LazyCells by 2.27% on a Xeon 6975P-C and 10.69% on an EPYC 7763. The W1 and W2 comparisons against LazyCells changed sign between hosts. Both hosts favored eager Cells at all three worker counts. Median peak RSS at W4 was approximately 15% below LazyCells in both runs. These three-arm measurements leave the direct comparison with opend open.

Each CI host passed the 338-case gate on all three arms and completed 12 paired rounds on Wikidata ×4. Candidate/LazyCells wall ratios at W1/W2/W4 were 1.0357/1.0272/0.9773 on Intel and 0.9511/0.9508/0.8931 on AMD. Candidate/eager ratios were 1.3810/1.3239/1.1351 and 1.3368/1.2891/1.1068. Target i9 verification is pending. [Intel CI](https://github.com/lmmx/mibz2/actions/runs/44423427194) · [Intel raw artifact](ci/evidence/conjugate-44423427194.zip) · [AMD CI](https://github.com/lmmx/mibz2/actions/runs/44424367557) · [AMD raw artifact](ci/evidence/conjugate-44424367557.zip)

Reproduce in a fresh checkout of the build tree above.

```sh
git checkout 9c31461df7bef24cecf79348545b27ee26846835 -- bench/moonshot .just Justfile
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/open/candidate.rs --workers 1,4 --rounds 3 --settle 0 --quick --with-opend --no-proxy
```

## delimiter-f2b

Local score. **PASS** on 338 scorer cases (full). [Per-case results](results/delimiter-f2b/score/check.jsonl) · [manifest](results/delimiter-f2b/manifest.txt) · [score](results/delimiter-f2b/score/summary.txt)

The build tree, including sibling modules, is [`d28fc0d10cbb29a30ef06a2c882e4cfc161a67ef`](https://github.com/lmmx/mibz2/tree/d28fc0d10cbb29a30ef06a2c882e4cfc161a67ef). [Candidate entry](https://github.com/lmmx/mibz2/blob/d28fc0d10cbb29a30ef06a2c882e4cfc161a67ef/research/astra-e/formulations/f2b/candidate.rs). Fetched master `9c31461df7bef24cecf79348545b27ee26846835`, scorer path `9c31461` (no version field printed). Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

| W | Pairs | Candidate / lazy [95% CI] | opend / lazy [95% CI] | Candidate / opend¹ | Median peak KiB, candidate / lazy / opend | Candidate / lazy peak |
|---:|---:|---|---|---:|---|---:|
| 1 | 6 | 1.460 [1.210, 1.762] | 1.007 [0.753, 1.348] | 1.450 | 78,004 / 80,144 / 77,700 | 0.973 |
| 4 | 6 | 1.415 [1.176, 1.702] | 1.109 [0.800, 1.538] | 1.276 | 116,954 / 129,042 / 116,736 | 0.906 |

¹ Ratio of paired geometric means sharing the same lazy observations. [Individual runs and process peaks](results/delimiter-f2b/score/runs.jsonl)

This score has no completed instruction, cache-transfer, dependency or overlap model.

Independent candidate CI and target evidence have not been recorded for this run in this register.

Reproduce in a fresh checkout of the build tree above.

```sh
git checkout 9c31461df7bef24cecf79348545b27ee26846835 -- bench/moonshot .just Justfile
bash research/astra-e/prepare.sh research/astra-e/formulations/f2b/candidate.rs
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/formulations/f2b/candidate.rs --workers 1,4 --rounds 6 --repeat 2 --with-opend --no-proxy
```

## dictionary-pilot

Pilot. **PASS** on 98 scorer cases (quick). [Per-case results](results/dictionary-pilot/score/check.jsonl) · [manifest](results/dictionary-pilot/manifest.txt) · [score](results/dictionary-pilot/score/summary.txt)

The build tree, including sibling modules, is [`5c21a8c5e100368886349ddf8b25828774a78836`](https://github.com/lmmx/mibz2/tree/5c21a8c5e100368886349ddf8b25828774a78836). [Candidate entry](https://github.com/lmmx/mibz2/blob/5c21a8c5e100368886349ddf8b25828774a78836/research/astra-e/dictionary/candidate.rs). Fetched master `9c31461df7bef24cecf79348545b27ee26846835`, scorer path `9c31461` (no version field printed). Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

| W | Pairs | Candidate / lazy [95% CI] | opend / lazy [95% CI] | Candidate / opend¹ | Median peak KiB, candidate / lazy / opend | Candidate / lazy peak |
|---:|---:|---|---|---:|---|---:|
| 1 | 3 | 0.846 [0.471, 1.518] | 0.868 [0.423, 1.780] | 0.974 | 45,856 / 61,292 / 45,856 | 0.748 |
| 4 | 3 | 0.922 [0.682, 1.245] | 0.879 [0.802, 0.964] | 1.048 | 83,456 / 125,096 / 82,584 | 0.667 |

¹ Ratio of paired geometric means sharing the same lazy observations. [Individual runs and process peaks](results/dictionary-pilot/score/runs.jsonl)

This score has no completed instruction, cache-transfer, dependency or overlap model.

Independent candidate CI and target evidence have not been recorded for this run in this register.

Reproduce in a fresh checkout of the build tree above.

```sh
git checkout 9c31461df7bef24cecf79348545b27ee26846835 -- bench/moonshot .just Justfile
bash research/astra-e/prepare.sh research/astra-e/dictionary/candidate.rs
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/dictionary/candidate.rs --workers 1,4 --rounds 3 --settle 0 --quick --with-opend --no-proxy
```

## endpoint16-pilot

Pilot. **PASS** on 98 scorer cases (quick). [Per-case results](results/endpoint16-pilot/score/check.jsonl) · [manifest](results/endpoint16-pilot/manifest.txt) · [score](results/endpoint16-pilot/score/summary.txt)

The build tree, including sibling modules, is [`ffb54d10d40a721e96db8e1342d51244a04a84ef`](https://github.com/lmmx/mibz2/tree/ffb54d10d40a721e96db8e1342d51244a04a84ef). [Candidate entry](https://github.com/lmmx/mibz2/blob/ffb54d10d40a721e96db8e1342d51244a04a84ef/research/astra-e/narrow-endpoint/candidate.rs). Fetched master `9c31461df7bef24cecf79348545b27ee26846835`, scorer path `9c31461` (no version field printed). Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

| W | Pairs | Candidate / lazy [95% CI] | opend / lazy [95% CI] | Candidate / opend¹ | Median peak KiB, candidate / lazy / opend | Candidate / lazy peak |
|---:|---:|---|---|---:|---|---:|
| 1 | 3 | 1.962 [0.205, 18.788] | 1.660 [0.884, 3.115] | 1.182 | 45,604 / 57,372 / 45,716 | 0.795 |
| 4 | 3 | 0.726 [0.551, 0.957] | 0.705 [0.457, 1.088] | 1.030 | 83,796 / 124,528 / 79,072 | 0.673 |

¹ Ratio of paired geometric means sharing the same lazy observations. [Individual runs and process peaks](results/endpoint16-pilot/score/runs.jsonl)

This score has no completed instruction, cache-transfer, dependency or overlap model.

Independent candidate CI and target evidence have not been recorded for this run in this register.

Reproduce in a fresh checkout of the build tree above.

```sh
git checkout 9c31461df7bef24cecf79348545b27ee26846835 -- bench/moonshot .just Justfile
bash research/astra-e/prepare.sh research/astra-e/narrow-endpoint/candidate.rs
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/narrow-endpoint/candidate.rs --workers 1,4 --rounds 3 --settle 0 --quick --with-opend --no-proxy
```

## hierarchy-f4

Local score. **PASS** on 338 scorer cases (full). [Per-case results](results/hierarchy-f4/score/check.jsonl) · [manifest](results/hierarchy-f4/manifest.txt) · [score](results/hierarchy-f4/score/summary.txt)

The build tree, including sibling modules, is [`99e90871536108bed01e44809aad44ee729d517b`](https://github.com/lmmx/mibz2/tree/99e90871536108bed01e44809aad44ee729d517b). [Candidate entry](https://github.com/lmmx/mibz2/blob/99e90871536108bed01e44809aad44ee729d517b/research/astra-e/formulations/f4/candidate.rs). Fetched master `9c31461df7bef24cecf79348545b27ee26846835`, scorer path `9c31461` (no version field printed). Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

| W | Pairs | Candidate / lazy [95% CI] | opend / lazy [95% CI] | Candidate / opend¹ | Median peak KiB, candidate / lazy / opend | Candidate / lazy peak |
|---:|---:|---|---|---:|---|---:|
| 1 | 5 | 1.940 [1.080, 3.487] | 0.681 [0.263, 1.764] | 2.851 | 50,696 / 57,752 / 45,672 | 0.878 |
| 4 | 5 | 2.159 [1.407, 3.313] | 0.777 [0.611, 0.988] | 2.780 | 102,448 / 125,136 / 84,864 | 0.819 |

¹ Ratio of paired geometric means sharing the same lazy observations. [Individual runs and process peaks](results/hierarchy-f4/score/runs.jsonl)

This score has no completed instruction, cache-transfer, dependency or overlap model.

Independent candidate CI and target evidence have not been recorded for this run in this register.

Reproduce in a fresh checkout of the build tree above.

```sh
git checkout 9c31461df7bef24cecf79348545b27ee26846835 -- bench/moonshot .just Justfile
bash research/astra-e/prepare.sh research/astra-e/formulations/f4/candidate.rs
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/formulations/f4/candidate.rs --workers 1,4 --rounds 5 --with-opend --no-proxy
```

## historical-control

Historical control. **PASS** on 338 scorer cases (full). [Per-case results](results/historical-control/score/check.jsonl) · [manifest](results/historical-control/manifest.txt) · [score](results/historical-control/score/summary.txt)

The build tree, including sibling modules, is [`775e604849d4bba0c474bf9f48cda486d3dbf3c8`](https://github.com/lmmx/mibz2/tree/775e604849d4bba0c474bf9f48cda486d3dbf3c8). [Candidate entry](https://github.com/lmmx/mibz2/blob/775e604849d4bba0c474bf9f48cda486d3dbf3c8/research/astra-e/controls/open.rs). Fetched master `9c31461df7bef24cecf79348545b27ee26846835`, scorer path `9c31461` (no version field printed). Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

| W | Pairs | Candidate / lazy [95% CI] | opend / lazy [95% CI] | Candidate / opend¹ | Median peak KiB, candidate / lazy / opend | Candidate / lazy peak |
|---:|---:|---|---|---:|---|---:|
| 1 | 3 | 0.338 [0.007, 16.528] | 1.184 [0.028, 50.390] | 0.286 | 40,456 / 57,520 / 45,704 | 0.703 |
| 2 | 3 | 0.901 [0.732, 1.109] | 1.799 [0.136, 23.791] | 0.501 | 49,348 / 93,636 / 56,068 | 0.527 |
| 3 | 3 | 1.032 [0.231, 4.601] | 1.056 [0.271, 4.118] | 0.977 | 59,284 / 110,140 / 67,768 | 0.538 |
| 4 | 3 | 0.446 [0.035, 5.641] | 0.434 [0.076, 2.484] | 1.028 | 67,748 / 131,340 / 83,948 | 0.516 |

¹ Ratio of paired geometric means sharing the same lazy observations. [Individual runs and process peaks](results/historical-control/score/runs.jsonl)

This score has no completed instruction, cache-transfer, dependency or overlap model.

The i9 measured the historical Open arm separately. [Matched i9 results](prior.md)

Reproduce in a fresh checkout of the build tree above.

```sh
git checkout 9c31461df7bef24cecf79348545b27ee26846835 -- bench/moonshot .just Justfile
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/controls/open.rs --workers 1,2,3,4 --rounds 3 --with-opend --no-proxy
```

## hybrid-pilot

Pilot. **PASS** on 98 scorer cases (quick). [Per-case results](results/hybrid-pilot/score/check.jsonl) · [manifest](results/hybrid-pilot/manifest.txt) · [score](results/hybrid-pilot/score/summary.txt)

The build tree, including sibling modules, is [`b88a02738799f8e021db356157ef4239bbea911e`](https://github.com/lmmx/mibz2/tree/b88a02738799f8e021db356157ef4239bbea911e). [Candidate entry](https://github.com/lmmx/mibz2/blob/b88a02738799f8e021db356157ef4239bbea911e/research/astra-e/cells/hybrid/candidate.rs). Fetched master `9c31461df7bef24cecf79348545b27ee26846835`, scorer path `9c31461` (no version field printed). Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

| W | Pairs | Candidate / lazy [95% CI] | opend / lazy [95% CI] | Candidate / opend¹ | Median peak KiB, candidate / lazy / opend | Candidate / lazy peak |
|---:|---:|---|---|---:|---|---:|
| 1 | 3 | 0.627 [0.227, 1.728] | 0.609 [0.330, 1.126] | 1.029 | 49,480 / 60,980 / 45,596 | 0.811 |
| 4 | 3 | 0.866 [0.353, 2.125] | 0.791 [0.289, 2.164] | 1.096 | 102,712 / 133,708 / 87,876 | 0.768 |

¹ Ratio of paired geometric means sharing the same lazy observations. [Individual runs and process peaks](results/hybrid-pilot/score/runs.jsonl)

This score has no completed instruction, cache-transfer, dependency or overlap model.

Independent candidate CI and target evidence have not been recorded for this run in this register.

Reproduce in a fresh checkout of the build tree above.

```sh
git checkout 9c31461df7bef24cecf79348545b27ee26846835 -- bench/moonshot .just Justfile
bash research/astra-e/prepare.sh research/astra-e/cells/hybrid/candidate.rs
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/cells/hybrid/candidate.rs --workers 1,4 --rounds 3 --settle 0 --quick --with-opend --no-proxy
```

## lanes-pilot

Pilot. **PASS** on 98 scorer cases (quick). [Per-case results](results/lanes-pilot/score/check.jsonl) · [manifest](results/lanes-pilot/manifest.txt) · [score](results/lanes-pilot/score/summary.txt)

The build tree, including sibling modules, is [`445d1851e26ba75ebb183f8eca7136593416f089`](https://github.com/lmmx/mibz2/tree/445d1851e26ba75ebb183f8eca7136593416f089). [Candidate entry](https://github.com/lmmx/mibz2/blob/445d1851e26ba75ebb183f8eca7136593416f089/research/astra-e/open/lanes/candidate.rs). Fetched master `9c31461df7bef24cecf79348545b27ee26846835`, scorer path `9c31461` (no version field printed). Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

| W | Pairs | Candidate / lazy [95% CI] | opend / lazy [95% CI] | Candidate / opend¹ | Median peak KiB, candidate / lazy / opend | Candidate / lazy peak |
|---:|---:|---|---|---:|---|---:|
| 1 | 3 | 0.619 [0.173, 2.209] | 0.872 [0.360, 2.112] | 0.710 | 47,776 / 57,504 / 45,736 | 0.831 |
| 4 | 3 | 0.928 [0.665, 1.297] | 0.829 [0.791, 0.869] | 1.120 | 90,548 / 124,656 / 89,316 | 0.726 |

¹ Ratio of paired geometric means sharing the same lazy observations. [Individual runs and process peaks](results/lanes-pilot/score/runs.jsonl)

This score has no completed instruction, cache-transfer, dependency or overlap model.

Independent candidate CI and target evidence have not been recorded for this run in this register.

Reproduce in a fresh checkout of the build tree above.

```sh
git checkout 9c31461df7bef24cecf79348545b27ee26846835 -- bench/moonshot .just Justfile
bash research/astra-e/prepare.sh research/astra-e/open/lanes/candidate.rs
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/open/lanes/candidate.rs --workers 1,4 --rounds 3 --settle 0 --quick --with-opend --no-proxy
```

## lookahead-pilot

Pilot. **PASS** on 98 scorer cases (quick). [Per-case results](results/lookahead-pilot/score/check.jsonl) · [manifest](results/lookahead-pilot/manifest.txt) · [score](results/lookahead-pilot/score/summary.txt)

The build tree, including sibling modules, is [`e4239b92adf9e542782cfe1e1bbf853f836a6fd7`](https://github.com/lmmx/mibz2/tree/e4239b92adf9e542782cfe1e1bbf853f836a6fd7). [Candidate entry](https://github.com/lmmx/mibz2/blob/e4239b92adf9e542782cfe1e1bbf853f836a6fd7/research/astra-e/lookahead/candidate.rs). Fetched master `9c31461df7bef24cecf79348545b27ee26846835`, scorer path `9c31461` (no version field printed). Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

| W | Pairs | Candidate / lazy [95% CI] | opend / lazy [95% CI] | Candidate / opend¹ | Median peak KiB, candidate / lazy / opend | Candidate / lazy peak |
|---:|---:|---|---|---:|---|---:|
| 1 | 3 | 0.560 [0.183, 1.714] | 0.513 [0.228, 1.151] | 1.093 | 45,732 / 61,072 / 45,900 | 0.749 |
| 4 | 3 | 0.829 [0.466, 1.476] | 0.668 [0.243, 1.836] | 1.240 | 85,820 / 128,312 / 84,220 | 0.669 |

¹ Ratio of paired geometric means sharing the same lazy observations. [Individual runs and process peaks](results/lookahead-pilot/score/runs.jsonl)

This score has no completed instruction, cache-transfer, dependency or overlap model.

The independent EPYC run resolved a wall-time gain of about 3% against opend at one and two workers. At four workers the 95% interval against opend was [0.9546, 1.0202]. Against LazyCells at four workers the ratio was 0.8475 [0.8124, 0.8842]. Low-worker eager Cells remained faster at each count.

Independent CI ran 16 paired rounds of Wikidata ×4 on AMD EPYC 7763 64-Core Processor, using four slots per worker and a 2 s settle. At W1/W2/W4 the candidate/LazyCells wall ratios were 0.8896/0.8959/0.8475, with peak RSS ratios 0.9689/0.9314/0.8682. Candidate/opend wall ratios were 0.9694/0.9690/0.9869. Candidate/eager wall ratios were 1.2464/1.1947/1.0550. All four 338-case correctness gates passed. Target i9 verification is pending. [CI run](https://github.com/lmmx/mibz2/actions/runs/44425372339) · [Raw checks and measurements](ci/evidence/lookahead-44425372339.zip)

Reproduce in a fresh checkout of the build tree above.

```sh
git checkout 9c31461df7bef24cecf79348545b27ee26846835 -- bench/moonshot .just Justfile
bash research/astra-e/prepare.sh research/astra-e/lookahead/candidate.rs
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/lookahead/candidate.rs --workers 1,4 --rounds 3 --settle 0 --quick --with-opend --no-proxy
```

## narrow-pilot

Pilot. **PASS** on 98 scorer cases (quick). [Per-case results](results/narrow-pilot/score/check.jsonl) · [manifest](results/narrow-pilot/manifest.txt) · [score](results/narrow-pilot/score/summary.txt)

The build tree, including sibling modules, is [`29f98ee83b5096eb9c9b49ff5d10c31b4c2db348`](https://github.com/lmmx/mibz2/tree/29f98ee83b5096eb9c9b49ff5d10c31b4c2db348). [Candidate entry](https://github.com/lmmx/mibz2/blob/29f98ee83b5096eb9c9b49ff5d10c31b4c2db348/research/astra-e/cells/narrow/candidate.rs). Fetched master `9c31461df7bef24cecf79348545b27ee26846835`, scorer path `9c31461` (no version field printed). Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

| W | Pairs | Candidate / lazy [95% CI] | opend / lazy [95% CI] | Candidate / opend¹ | Median peak KiB, candidate / lazy / opend | Candidate / lazy peak |
|---:|---:|---|---|---:|---|---:|
| 1 | 3 | 0.858 [0.191, 3.843] | 0.820 [0.103, 6.546] | 1.046 | 48,064 / 60,964 / 45,504 | 0.788 |
| 4 | 3 | 1.469 [0.900, 2.397] | 0.854 [0.659, 1.106] | 1.720 | 95,760 / 134,884 / 84,784 | 0.710 |

¹ Ratio of paired geometric means sharing the same lazy observations. [Individual runs and process peaks](results/narrow-pilot/score/runs.jsonl)

This score has no completed instruction, cache-transfer, dependency or overlap model.

Independent candidate CI and target evidence have not been recorded for this run in this register.

Reproduce in a fresh checkout of the build tree above.

```sh
git checkout 9c31461df7bef24cecf79348545b27ee26846835 -- bench/moonshot .just Justfile
bash research/astra-e/prepare.sh research/astra-e/cells/narrow/candidate.rs
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/cells/narrow/candidate.rs --workers 1,4 --rounds 3 --settle 0 --quick --with-opend --no-proxy
```

## packed-eager-pilot

Pilot. **PASS** on 98 scorer cases (quick). [Per-case results](results/packed-eager-pilot/score/check.jsonl) · [manifest](results/packed-eager-pilot/manifest.txt) · [score](results/packed-eager-pilot/score/summary.txt)

The build tree, including sibling modules, is [`491bb839464ea9a7c35b1a60f932f39a79c95c60`](https://github.com/lmmx/mibz2/tree/491bb839464ea9a7c35b1a60f932f39a79c95c60). [Candidate entry](https://github.com/lmmx/mibz2/blob/491bb839464ea9a7c35b1a60f932f39a79c95c60/research/astra-e/cells/packed.rs). Fetched master `9c31461df7bef24cecf79348545b27ee26846835`, scorer path `9c31461` (no version field printed). Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

| W | Pairs | Candidate / lazy [95% CI] | opend / lazy [95% CI] | Candidate / opend¹ | Median peak KiB, candidate / lazy / opend | Candidate / lazy peak |
|---:|---:|---|---|---:|---|---:|
| 1 | 3 | 1.263 [0.465, 3.434] | 0.885 [0.558, 1.403] | 1.428 | 48,436 / 61,248 / 45,776 | 0.791 |
| 4 | 3 | 0.992 [0.281, 3.499] | 0.662 [0.197, 2.221] | 1.499 | 95,668 / 134,396 / 77,968 | 0.712 |

¹ Ratio of paired geometric means sharing the same lazy observations. [Individual runs and process peaks](results/packed-eager-pilot/score/runs.jsonl)

This score has no completed instruction, cache-transfer, dependency or overlap model.

Independent candidate CI and target evidence have not been recorded for this run in this register.

Reproduce in a fresh checkout of the build tree above.

```sh
git checkout 9c31461df7bef24cecf79348545b27ee26846835 -- bench/moonshot .just Justfile
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/cells/packed.rs --workers 1,4 --rounds 3 --settle 0 --quick --with-opend --no-proxy
```

## page-trace-pilot

Pilot. **PASS** on 98 scorer cases (quick). [Per-case results](results/page-trace-pilot/score/check.jsonl) · [manifest](results/page-trace-pilot/manifest.txt) · [score](results/page-trace-pilot/score/summary.txt)

The build tree, including sibling modules, is [`396fc717c67ea5dd779a150f13283ab3a440653f`](https://github.com/lmmx/mibz2/tree/396fc717c67ea5dd779a150f13283ab3a440653f). [Candidate entry](https://github.com/lmmx/mibz2/blob/396fc717c67ea5dd779a150f13283ab3a440653f/research/astra-e/dense/candidate.rs). Fetched master `9c31461df7bef24cecf79348545b27ee26846835`, scorer path `9c31461` (no version field printed). Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

| W | Pairs | Candidate / lazy [95% CI] | opend / lazy [95% CI] | Candidate / opend¹ | Median peak KiB, candidate / lazy / opend | Candidate / lazy peak |
|---:|---:|---|---|---:|---|---:|
| 1 | 3 | 1.190 [0.593, 2.387] | 1.091 [0.875, 1.362] | 1.090 | 45,528 / 57,860 / 45,580 | 0.787 |
| 4 | 3 | 0.672 [0.175, 2.591] | 0.794 [0.399, 1.583] | 0.847 | 80,528 / 125,560 / 84,636 | 0.641 |

¹ Ratio of paired geometric means sharing the same lazy observations. [Individual runs and process peaks](results/page-trace-pilot/score/runs.jsonl)

This score has no completed instruction, cache-transfer, dependency or overlap model.

Independent candidate CI and target evidence have not been recorded for this run in this register.

Reproduce in a fresh checkout of the build tree above.

```sh
git checkout 9c31461df7bef24cecf79348545b27ee26846835 -- bench/moonshot .just Justfile
bash research/astra-e/prepare.sh research/astra-e/dense/candidate.rs
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/dense/candidate.rs --workers 1,4 --rounds 3 --settle 0 --quick --with-opend --no-proxy
```

## pairs-pilot

Pilot. **PASS** on 98 scorer cases (quick). [Per-case results](results/pairs-pilot/score/check.jsonl) · [manifest](results/pairs-pilot/manifest.txt) · [score](results/pairs-pilot/score/summary.txt)

The build tree, including sibling modules, is [`31503cc2c69a9dd227e3d7a7e2293bd0a230368e`](https://github.com/lmmx/mibz2/tree/31503cc2c69a9dd227e3d7a7e2293bd0a230368e). [Candidate entry](https://github.com/lmmx/mibz2/blob/31503cc2c69a9dd227e3d7a7e2293bd0a230368e/research/astra-e/pairs/candidate.rs). Fetched master `9c31461df7bef24cecf79348545b27ee26846835`, scorer path `9c31461` (no version field printed). Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

| W | Pairs | Candidate / lazy [95% CI] | opend / lazy [95% CI] | Candidate / opend¹ | Median peak KiB, candidate / lazy / opend | Candidate / lazy peak |
|---:|---:|---|---|---:|---|---:|
| 1 | 3 | 1.021 [0.757, 1.376] | 1.152 [0.311, 4.268] | 0.886 | 45,776 / 61,108 / 45,852 | 0.749 |
| 4 | 3 | 0.894 [0.743, 1.074] | 0.905 [0.789, 1.038] | 0.987 | 77,564 / 128,364 / 83,092 | 0.604 |

¹ Ratio of paired geometric means sharing the same lazy observations. [Individual runs and process peaks](results/pairs-pilot/score/runs.jsonl)

This score has no completed instruction, cache-transfer, dependency or overlap model.

Independent candidate CI and target evidence have not been recorded for this run in this register.

Reproduce in a fresh checkout of the build tree above.

```sh
git checkout 9c31461df7bef24cecf79348545b27ee26846835 -- bench/moonshot .just Justfile
python3 research/astra-e/pairs/prepare.py
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/pairs/candidate.rs --workers 1,4 --rounds 3 --settle 0 --quick --with-opend --no-proxy
```

## short-words-confirm

Settled confirmation. **PASS** on 338 scorer cases (full). [Per-case results](results/short-words-confirm/score/check.jsonl) · [manifest](results/short-words-confirm/manifest.txt) · [score](results/short-words-confirm/score/summary.txt)

The build tree, including sibling modules, is [`04b730e67b456c44db16cab3d6416d606684ffee`](https://github.com/lmmx/mibz2/tree/04b730e67b456c44db16cab3d6416d606684ffee). [Candidate entry](https://github.com/lmmx/mibz2/blob/04b730e67b456c44db16cab3d6416d606684ffee/research/astra-e/short-words/candidate.rs). Fetched master `9c31461df7bef24cecf79348545b27ee26846835`, scorer path `9c31461` (no version field printed). Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

| W | Pairs | Candidate / lazy [95% CI] | opend / lazy [95% CI] | Candidate / opend¹ | Median peak KiB, candidate / lazy / opend | Candidate / lazy peak |
|---:|---:|---|---|---:|---|---:|
| 1 | 6 | 0.923 [0.659, 1.292] | 0.807 [0.696, 0.935] | 1.144 | 104,354 / 107,752 / 104,202 | 0.968 |
| 2 | 6 | 1.003 [0.735, 1.369] | 1.026 [0.797, 1.321] | 0.978 | 123,836 / 130,340 / 123,960 | 0.950 |
| 3 | 6 | 1.099 [0.664, 1.818] | 0.849 [0.573, 1.258] | 1.294 | 141,418 / 153,118 / 141,552 | 0.924 |
| 4 | 6 | 0.766 [0.610, 0.962] | 0.785 [0.532, 1.158] | 0.977 | 157,160 / 180,054 / 158,568 | 0.873 |

¹ Ratio of paired geometric means sharing the same lazy observations. [Individual runs and process peaks](results/short-words-confirm/score/runs.jsonl)

This score has no completed instruction, cache-transfer, dependency or overlap model.

Six rotated rounds per worker count, Wikidata ×4, 2 s settle, complete gate. Candidate/opend at W1,2,3,4: 1.144 [0.914,1.432], 0.978 [0.678,1.411], 1.294 [1.051,1.594], 0.977 [0.798,1.196], computed from same-round walls with the scorer's paired_ratio. W4 is faster than LazyCells; comparison against opend is inconclusive there. W3 regresses vs opend.

Separate sources and worktree from the concurrent research edits. The independent CI receipts for conjugate and lookahead refer to different candidates. The short-word implementation has no independent i9 timing.

Reproduce in a fresh checkout of the build tree above.

```sh
git checkout 9c31461df7bef24cecf79348545b27ee26846835 -- bench/moonshot .just Justfile
bash research/astra-e/prepare.sh research/astra-e/short-words/candidate.rs
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/short-words/candidate.rs --workers 1,2,3,4 --rounds 6 --repeat 4 --settle 2 --with-opend --no-proxy
```

## short-words-pilot

Pilot. **PASS** on 98 scorer cases (quick). [Per-case results](results/short-words-pilot/score/check.jsonl) · [manifest](results/short-words-pilot/manifest.txt) · [score](results/short-words-pilot/score/summary.txt)

The build tree, including sibling modules, is [`5dcaff2b8612b17b78af526fd72d872d834d6c49`](https://github.com/lmmx/mibz2/tree/5dcaff2b8612b17b78af526fd72d872d834d6c49). [Candidate entry](https://github.com/lmmx/mibz2/blob/5dcaff2b8612b17b78af526fd72d872d834d6c49/research/astra-e/short-words/candidate.rs). Fetched master `9c31461df7bef24cecf79348545b27ee26846835`, scorer path `9c31461` (no version field printed). Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

| W | Pairs | Candidate / lazy [95% CI] | opend / lazy [95% CI] | Candidate / opend¹ | Median peak KiB, candidate / lazy / opend | Candidate / lazy peak |
|---:|---:|---|---|---:|---|---:|
| 1 | 3 | 0.877 [0.682, 1.127] | 0.918 [0.810, 1.040] | 0.955 | 46,088 / 61,224 / 45,832 | 0.753 |
| 4 | 3 | 0.813 [0.567, 1.167] | 0.854 [0.650, 1.122] | 0.952 | 75,584 / 133,428 / 81,912 | 0.566 |

¹ Ratio of paired geometric means sharing the same lazy observations. [Individual runs and process peaks](results/short-words-pilot/score/runs.jsonl)

This score has no completed instruction, cache-transfer, dependency or overlap model.

Independent candidate CI and target evidence have not been recorded for this run in this register.

Reproduce in a fresh checkout of the build tree above.

```sh
git checkout 9c31461df7bef24cecf79348545b27ee26846835 -- bench/moonshot .just Justfile
bash research/astra-e/prepare.sh research/astra-e/short-words/candidate.rs
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/short-words/candidate.rs --workers 1,4 --rounds 3 --settle 0 --quick --with-opend --no-proxy
```

## streamed-pilot

Pilot. **PASS** on 98 scorer cases (quick). [Per-case results](results/streamed-pilot/score/check.jsonl) · [manifest](results/streamed-pilot/manifest.txt) · [score](results/streamed-pilot/score/summary.txt)

The build tree, including sibling modules, is [`a48e94ed83595000c60a264c13357c801eb5b99b`](https://github.com/lmmx/mibz2/tree/a48e94ed83595000c60a264c13357c801eb5b99b). [Candidate entry](https://github.com/lmmx/mibz2/blob/a48e94ed83595000c60a264c13357c801eb5b99b/research/astra-e/streamed/candidate.rs). Fetched master `9c31461df7bef24cecf79348545b27ee26846835`, scorer path `9c31461` (no version field printed). Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

| W | Pairs | Candidate / lazy [95% CI] | opend / lazy [95% CI] | Candidate / opend¹ | Median peak KiB, candidate / lazy / opend | Candidate / lazy peak |
|---:|---:|---|---|---:|---|---:|
| 1 | 3 | 0.907 [0.538, 1.530] | 0.859 [0.322, 2.287] | 1.056 | 46,044 / 57,740 / 45,836 | 0.797 |
| 4 | 3 | 0.953 [0.811, 1.121] | 0.785 [0.524, 1.175] | 1.215 | 77,052 / 123,500 / 80,812 | 0.624 |

¹ Ratio of paired geometric means sharing the same lazy observations. [Individual runs and process peaks](results/streamed-pilot/score/runs.jsonl)

This score has no completed instruction, cache-transfer, dependency or overlap model.

Independent candidate CI and target evidence have not been recorded for this run in this register.

Reproduce in a fresh checkout of the build tree above.

```sh
git checkout 9c31461df7bef24cecf79348545b27ee26846835 -- bench/moonshot .just Justfile
bash research/astra-e/prepare.sh research/astra-e/streamed/candidate.rs
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/streamed/candidate.rs --workers 1,4 --rounds 3 --settle 0 --quick --with-opend --no-proxy
```

## tableless-f1b

Local score. **PASS** on 338 scorer cases (full). [Per-case results](results/tableless-f1b/score/check.jsonl) · [manifest](results/tableless-f1b/manifest.txt) · [score](results/tableless-f1b/score/summary.txt)

The build tree, including sibling modules, is [`625a4ff15e6d8f6c065e749fad6aa69c1629fa66`](https://github.com/lmmx/mibz2/tree/625a4ff15e6d8f6c065e749fad6aa69c1629fa66). [Candidate entry](https://github.com/lmmx/mibz2/blob/625a4ff15e6d8f6c065e749fad6aa69c1629fa66/research/astra-e/formulations/f1b/candidate.rs). Fetched master `9c31461df7bef24cecf79348545b27ee26846835`, scorer path `9c31461` (no version field printed). Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

| W | Pairs | Candidate / lazy [95% CI] | opend / lazy [95% CI] | Candidate / opend¹ | Median peak KiB, candidate / lazy / opend | Candidate / lazy peak |
|---:|---:|---|---|---:|---|---:|
| 1 | 6 | 1.806 [1.448, 2.253] | 0.905 [0.705, 1.161] | 1.997 | 76,252 / 80,142 / 77,380 | 0.951 |
| 4 | 6 | 1.583 [1.305, 1.919] | 0.888 [0.650, 1.213] | 1.782 | 111,412 / 129,454 / 116,410 | 0.861 |

¹ Ratio of paired geometric means sharing the same lazy observations. [Individual runs and process peaks](results/tableless-f1b/score/runs.jsonl)

This score has no completed instruction, cache-transfer, dependency or overlap model.

Independent candidate CI and target evidence have not been recorded for this run in this register.

Reproduce in a fresh checkout of the build tree above.

```sh
git checkout 9c31461df7bef24cecf79348545b27ee26846835 -- bench/moonshot .just Justfile
bash research/astra-e/prepare.sh research/astra-e/formulations/f1b/candidate.rs
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/formulations/f1b/candidate.rs --rounds 6 --repeat 2 --workers 1,4 --with-opend --no-proxy
```

## two-ended-pilot

Pilot. **PASS** on 98 scorer cases (quick). [Per-case results](results/two-ended-pilot/score/check.jsonl) · [manifest](results/two-ended-pilot/manifest.txt) · [score](results/two-ended-pilot/score/summary.txt)

The build tree, including sibling modules, is [`9d746dd1825880b86739fd1045bda612f5677d82`](https://github.com/lmmx/mibz2/tree/9d746dd1825880b86739fd1045bda612f5677d82). [Candidate entry](https://github.com/lmmx/mibz2/blob/9d746dd1825880b86739fd1045bda612f5677d82/research/astra-e/bidirectional/candidate.rs). Fetched master `9c31461df7bef24cecf79348545b27ee26846835`, scorer path `9c31461` (no version field printed). Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

| W | Pairs | Candidate / lazy [95% CI] | opend / lazy [95% CI] | Candidate / opend¹ | Median peak KiB, candidate / lazy / opend | Candidate / lazy peak |
|---:|---:|---|---|---:|---|---:|
| 1 | 3 | 1.072 [0.826, 1.392] | 0.813 [0.697, 0.948] | 1.319 | 45,444 / 60,836 / 45,704 | 0.747 |
| 4 | 3 | 1.152 [0.961, 1.380] | 0.855 [0.676, 1.081] | 1.347 | 84,252 / 124,716 / 80,604 | 0.676 |

¹ Ratio of paired geometric means sharing the same lazy observations. [Individual runs and process peaks](results/two-ended-pilot/score/runs.jsonl)

This score has no completed instruction, cache-transfer, dependency or overlap model.

Independent candidate CI and target evidence have not been recorded for this run in this register.

Reproduce in a fresh checkout of the build tree above.

```sh
git checkout 9c31461df7bef24cecf79348545b27ee26846835 -- bench/moonshot .just Justfile
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/bidirectional/candidate.rs --workers 1,4 --rounds 3 --settle 0 --quick --with-opend --no-proxy
```

## two-step-pilot

Pilot. **PASS** on 98 scorer cases (quick). [Per-case results](results/two-step-pilot/score/check.jsonl) · [manifest](results/two-step-pilot/manifest.txt) · [score](results/two-step-pilot/score/summary.txt)

The build tree, including sibling modules, is [`3ebdba68dc52233b47c33a6d06cf4b3ca86724f7`](https://github.com/lmmx/mibz2/tree/3ebdba68dc52233b47c33a6d06cf4b3ca86724f7). [Candidate entry](https://github.com/lmmx/mibz2/blob/3ebdba68dc52233b47c33a6d06cf4b3ca86724f7/research/astra-e/dense/two_step.rs). Fetched master `9c31461df7bef24cecf79348545b27ee26846835`, scorer path `9c31461` (no version field printed). Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

| W | Pairs | Candidate / lazy [95% CI] | opend / lazy [95% CI] | Candidate / opend¹ | Median peak KiB, candidate / lazy / opend | Candidate / lazy peak |
|---:|---:|---|---|---:|---|---:|
| 1 | 3 | 1.123 [0.501, 2.513] | 0.420 [0.004, 45.653] | 2.674 | 45,616 / 58,164 / 46,516 | 0.784 |
| 4 | 3 | 0.956 [0.257, 3.550] | 0.455 [0.055, 3.770] | 2.103 | 83,048 / 126,136 / 86,708 | 0.658 |

¹ Ratio of paired geometric means sharing the same lazy observations. [Individual runs and process peaks](results/two-step-pilot/score/runs.jsonl)

This score has no completed instruction, cache-transfer, dependency or overlap model.

Independent candidate CI and target evidence have not been recorded for this run in this register.

Reproduce in a fresh checkout of the build tree above.

```sh
git checkout 9c31461df7bef24cecf79348545b27ee26846835 -- bench/moonshot .just Justfile
bash research/astra-e/prepare.sh research/astra-e/dense/two_step.rs
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/dense/two_step.rs --workers 1,4 --rounds 3 --settle 0 --quick --with-opend --no-proxy
```

## two-step-proxy

Local score. **PASS** on 98 scorer cases (quick). [Per-case results](results/two-step-proxy/score/check.jsonl) · [manifest](results/two-step-proxy/manifest.txt) · [score](results/two-step-proxy/score/summary.txt)

The build tree, including sibling modules, is [`ff36563702a8ecf92b641681f939c3fbbc71d02c`](https://github.com/lmmx/mibz2/tree/ff36563702a8ecf92b641681f939c3fbbc71d02c). [Candidate entry](https://github.com/lmmx/mibz2/blob/ff36563702a8ecf92b641681f939c3fbbc71d02c/research/astra-e/dense/two_step.rs). Fetched master `9c31461df7bef24cecf79348545b27ee26846835`, scorer path `9c31461` (no version field printed). Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

| W model | Instructions/B | L2 / L3 / DRAM B/B | Dependent DRAM / past-L2 links/B | DRAM MLP | Calibration |
|---:|---:|---|---|---:|---|
| 1 | 16.548 | 20.355 / 15.888 / 1.372 | 0.00001 / 0.06154 | 2.342 | within envelope |
| 2 | 16.548 | 20.355 / 15.888 / 1.372 | 0.00001 / 0.06155 | 2.354 | within envelope |
| 4 | 16.548 | 20.355 / 15.888 / 1.534 | 0.00001 / 0.06150 | 2.369 | within envelope |
| 7 | 16.548 | 20.355 / 15.888 / 6.692 | 0.00004 / 0.06095 | 1.921 | within envelope |
| 8 | 16.548 | 20.355 / 15.888 / 7.607 | 0.00004 / 0.06081 | 1.922 | within envelope |
| 10 | 16.548 | 20.355 / 15.888 / 8.005 | 0.00014 / 0.06072 | 1.867 | within envelope |

Independent candidate CI and target evidence have not been recorded for this run in this register.

Reproduce in a fresh checkout of the build tree above.

```sh
git checkout 9c31461df7bef24cecf79348545b27ee26846835 -- bench/moonshot .just Justfile
bash research/astra-e/prepare.sh research/astra-e/dense/two_step.rs
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/dense/two_step.rs --target-dir /workspace/work/two-step-proxy-target --quick --no-timing --proxy-blocks 16
```

## v1-compound-gate

Local score. **PASS** on 338 scorer cases (full). [Per-case results](results/v1-compound-gate/score/check.jsonl) · [manifest](results/v1-compound-gate/manifest.txt) · [score](results/v1-compound-gate/score/summary.txt)

The build tree, including sibling modules, is [`ee35db00e5e8b9b726d6bc1c16817a9fabb0d5cc`](https://github.com/lmmx/mibz2/tree/ee35db00e5e8b9b726d6bc1c16817a9fabb0d5cc). [Candidate entry](https://github.com/lmmx/mibz2/blob/ee35db00e5e8b9b726d6bc1c16817a9fabb0d5cc/research/astra-e/formulations/f2-word/candidate.rs). Fetched master `604dac3f46569edd9ac9b6706b23a68d9d7d8f8d`, scorer path `604dac3` (v1 at 6a8a366cd786d58a528000b58da12fd2cfb47be8). Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

This score has no completed instruction, cache-transfer, dependency or overlap model.

Independent candidate CI and target evidence have not been recorded for this run in this register.

Reproduce in a fresh checkout of the build tree above.

```sh
git checkout 604dac3f46569edd9ac9b6706b23a68d9d7d8f8d -- bench/moonshot .just Justfile
bash research/astra-e/prepare.sh research/astra-e/formulations/f2-word/candidate.rs
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/formulations/f2-word/candidate.rs --no-proxy --no-timing --target-dir /workspace/work/f2-word-build
```

## v1-edge-confirm

Confirmation. **PASS** on 338 scorer cases (full). [Per-case results](results/v1-edge-confirm/score/check.jsonl) · [manifest](results/v1-edge-confirm/manifest.txt) · [score](results/v1-edge-confirm/score/summary.txt)

The build tree, including sibling modules, is [`e293f7c7262042023098406891b8a45d081879f5`](https://github.com/lmmx/mibz2/tree/e293f7c7262042023098406891b8a45d081879f5). [Candidate entry](https://github.com/lmmx/mibz2/blob/e293f7c7262042023098406891b8a45d081879f5/research/astra-e/edge-budget/candidate.rs). Fetched master `604dac3f46569edd9ac9b6706b23a68d9d7d8f8d`, scorer path `604dac3` (v1 at 6a8a366cd786d58a528000b58da12fd2cfb47be8). Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

| W | Pairs | Candidate / lazy [95% CI] | opend / lazy [95% CI] | Candidate / opend¹ | Median peak KiB, candidate / lazy / opend | Candidate / lazy peak |
|---:|---:|---|---|---:|---|---:|
| 1 | 9 | 0.970 [0.668, 1.408] | 0.984 [0.656, 1.473] | 0.986 | 71,920 / 80,196 / 77,308 | 0.897 |
| 4 | 9 | 0.743 [0.621, 0.888] | 0.778 [0.691, 0.876] | 0.954 | 100,508 / 137,316 / 115,196 | 0.732 |

¹ Ratio of paired geometric means sharing the same lazy observations. [Individual runs and process peaks](results/v1-edge-confirm/score/runs.jsonl)

This score has no completed instruction, cache-transfer, dependency or overlap model.

The W4 point estimate against LazyCells is below 0.80 on this host, and its upper confidence limit is above 0.80. Direct paired wall comparisons to opend were W1 0.986 [0.821,1.184] W4 0.954 [0.851,1.070]. Shared-host load varied. Separate-epoch leader rankings and three-round pilot differences do not establish a production gain.

Independent candidate CI and target evidence have not been recorded for this run in this register.

Reproduce in a fresh checkout of the build tree above.

```sh
git checkout 604dac3f46569edd9ac9b6706b23a68d9d7d8f8d -- bench/moonshot .just Justfile
bash research/astra-e/prepare.sh research/astra-e/edge-budget/candidate.rs
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/edge-budget/candidate.rs --workers 1,4 --with-opend --rounds 9 --repeat 2 --settle 2 --no-proxy
```

## v1-final-slot-gate

Local score. **PASS** on 338 scorer cases (full). [Per-case results](results/v1-final-slot-gate/score/check.jsonl) · [manifest](results/v1-final-slot-gate/manifest.txt) · [score](results/v1-final-slot-gate/score/summary.txt)

The build tree, including sibling modules, is [`548ef34524e8d007febcf879a9b2c6cdf8cc6b88`](https://github.com/lmmx/mibz2/tree/548ef34524e8d007febcf879a9b2c6cdf8cc6b88). [Candidate entry](https://github.com/lmmx/mibz2/blob/548ef34524e8d007febcf879a9b2c6cdf8cc6b88/research/astra-e/final-slot/candidate.rs). Fetched master `604dac3f46569edd9ac9b6706b23a68d9d7d8f8d`, scorer path `604dac3` (v1 at 6a8a366cd786d58a528000b58da12fd2cfb47be8). Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

This score has no completed instruction, cache-transfer, dependency or overlap model.

Independent candidate CI and target evidence have not been recorded for this run in this register.

Reproduce in a fresh checkout of the build tree above.

```sh
git checkout 604dac3f46569edd9ac9b6706b23a68d9d7d8f8d -- bench/moonshot .just Justfile
bash research/astra-e/prepare.sh research/astra-e/final-slot/candidate.rs
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/final-slot/candidate.rs --no-proxy --no-timing --target-dir /workspace/work/f2-word-build
```

## v1-short-confirm

Confirmation. **PASS** on 338 scorer cases (full). [Per-case results](results/v1-short-confirm/score/check.jsonl) · [manifest](results/v1-short-confirm/manifest.txt) · [score](results/v1-short-confirm/score/summary.txt)

The build tree, including sibling modules, is [`e293f7c7262042023098406891b8a45d081879f5`](https://github.com/lmmx/mibz2/tree/e293f7c7262042023098406891b8a45d081879f5). [Candidate entry](https://github.com/lmmx/mibz2/blob/e293f7c7262042023098406891b8a45d081879f5/research/astra-e/short-words/candidate.rs). Fetched master `604dac3f46569edd9ac9b6706b23a68d9d7d8f8d`, scorer path `604dac3` (v1 at 6a8a366cd786d58a528000b58da12fd2cfb47be8). Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

| W | Pairs | Candidate / lazy [95% CI] | opend / lazy [95% CI] | Candidate / opend¹ | Median peak KiB, candidate / lazy / opend | Candidate / lazy peak |
|---:|---:|---|---|---:|---|---:|
| 1 | 9 | 0.876 [0.757, 1.013] | 0.920 [0.821, 1.031] | 0.952 | 77,876 / 80,180 / 77,552 | 0.971 |
| 4 | 9 | 1.205 [0.838, 1.732] | 1.129 [0.928, 1.373] | 1.068 | 118,136 / 133,656 / 118,412 | 0.884 |

¹ Ratio of paired geometric means sharing the same lazy observations. [Individual runs and process peaks](results/v1-short-confirm/score/runs.jsonl)

This score has no completed instruction, cache-transfer, dependency or overlap model.

The earlier W4 point estimate for short words did not recur. Direct paired wall comparisons to opend were W1 0.952 [0.836,1.084] W4 1.068 [0.718,1.588]. Shared-host load varied. Separate-epoch leader rankings and three-round pilot differences do not establish a production gain.

Independent candidate CI and target evidence have not been recorded for this run in this register.

Reproduce in a fresh checkout of the build tree above.

```sh
git checkout 604dac3f46569edd9ac9b6706b23a68d9d7d8f8d -- bench/moonshot .just Justfile
bash research/astra-e/prepare.sh research/astra-e/short-words/candidate.rs
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/short-words/candidate.rs --workers 1,4 --with-opend --rounds 9 --repeat 2 --settle 2 --no-proxy
```
