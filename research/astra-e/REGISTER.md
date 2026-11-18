# Astra E best-so-far register

The reference is 0.5.6 LazyCells in the same shipped worker loop and 4W slots. The full Wikidata input decodes to 209,700,196 bytes per copy. Each gate result and timing scope below belongs to one immutable build. The local Xeon measurements do not establish the i9 target. [Frozen target rule](workstation/README.md) · [Historical i9 frontier](prior.md) · [Astra E draft PR and CI](https://github.com/lmmx/mibz2/pull/607/checks)

## Local leaders by tested design

These are the lowest recorded point estimates within each group of experiments with matching scorer revision, corpus hash, compiler, host, design and worker count. Each experiment pairs its candidate with its own lazy control. Between-experiment order remains uncertain when timing variation is large. A shared quiet comparison is needed to establish which candidate wins head to head. Pilot runs may overlap other work. Confirmation runs have their own rows. The historical control stays separate. RSS leaders remain visible when timing and memory favour different candidates.

| Scorer / design / W | Lowest wall estimate vs lazy [95% CI] | Lowest median RSS ratio vs lazy |
|---|---|---|
| `9c31461` pilot, grid 1,4, 3r ×1, settle 0s, quick, 3 arms, **W1** | conjugate-pilot · 0.965 [0.660, 1.413] | two-ended-pilot · 0.747 |
| `9c31461` pilot, grid 1,4, 3r ×1, settle 0s, quick, 3 arms, **W4** | page-trace-pilot · 0.672 [0.175, 2.591] | conjugate-pilot · 0.603 |

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

Reproduction commands below replay the historical scorer on the recorded master commit. For a new scoring round, use `bash research/astra-e/score.sh <new-label> <candidate> ...` to refresh master first. Per-run `assessment.json` may record `kind`, `assessment` and `verification` (including independent run links). Regenerate with `python3 research/astra-e/register.py`.

## conjugate-pilot

Pilot. **PASS** on 98 scorer cases (quick). [Per-case results](results/conjugate-pilot/score/check.jsonl) · [manifest](results/conjugate-pilot/manifest.txt) · [score](results/conjugate-pilot/score/summary.txt)

The build tree, including sibling modules, is [`e12e5f63173ee7cc6fcc413510ee76cfee0c1eb1`](https://github.com/lmmx/mibz2/tree/e12e5f63173ee7cc6fcc413510ee76cfee0c1eb1). [Candidate entry](https://github.com/lmmx/mibz2/blob/e12e5f63173ee7cc6fcc413510ee76cfee0c1eb1/research/astra-e/open/candidate.rs). Fetched master `9c31461df7bef24cecf79348545b27ee26846835`, scorer path `9c31461` (no version field printed). Corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.

| W | Pairs | Candidate / lazy [95% CI] | opend / lazy [95% CI] | Candidate / opend¹ | Median peak KiB, candidate / lazy / opend | Candidate / lazy peak |
|---:|---:|---|---|---:|---|---:|
| 1 | 3 | 0.965 [0.660, 1.413] | 0.907 [0.831, 0.989] | 1.064 | 44,552 / 58,876 / 45,768 | 0.757 |
| 4 | 3 | 1.018 [0.705, 1.470] | 0.923 [0.677, 1.259] | 1.102 | 71,052 / 117,804 / 77,652 | 0.603 |

¹ Ratio of paired geometric means sharing the same lazy observations. [Individual runs and process peaks](results/conjugate-pilot/score/runs.jsonl)

This score has no completed instruction, cache-transfer, dependency or overlap model.

Independent candidate CI and target evidence have not been recorded for this run in this register.

Reproduce in a fresh checkout of the build tree above.

```sh
git checkout 9c31461df7bef24cecf79348545b27ee26846835 -- bench/moonshot .just Justfile
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/open/candidate.rs --workers 1,4 --rounds 3 --settle 0 --quick --with-opend --no-proxy
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
bash bench/moonshot/corpus.sh
python3 bench/moonshot/moonshot.py score research/astra-e/pairs/candidate.rs --workers 1,4 --rounds 3 --settle 0 --quick --with-opend --no-proxy
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
