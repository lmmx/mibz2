# The run-fed leader's workstation diagnostics (descriptive, never judged)

descriptive only: no pair is excluded, replaced or reweighted by anything here, and `admit`'s reading reads none of it (§R10).

Status: incomplete: 0 of 14 records lack a snapshot, 14 have no endpoint label.

Endpoint label: post-exit snapshot: H if at least 10 of 20 CPUs read scaling_cur_freq >= 3700 MHz, L if every CPU was read and fewer did, else unknown; a pair's state is the incumbent's label then the arm's. An endpoint label, not the clock during the decode.

## Pair states, counts only

| arm | HH | HL | LH | LL | unknown |
|---|---|---|---|---|---|
| `mibz2-pipe3-both` | 0 | 0 | 0 | 0 | 2 |
| `mibz2-pipe3-runfed-ctl` | 0 | 0 | 0 | 0 | 2 |
| `mibz2-pipe3-runfed-reg` | 0 | 0 | 0 | 0 | 2 |
| `mibz2-pipe3-runfed-ls` | 0 | 0 | 0 | 0 | 2 |
| `mibz2-pipe3-runfed-hs` | 0 | 0 | 0 | 0 | 2 |
| `mibz2-pipe3-runfed-dir` | 0 | 0 | 0 | 0 | 2 |
| all | 0 | 0 | 0 | 0 | 12 |

## Every record, in run order

| seq | round | pos | arm | predecessor | gap s | wall | cpu | MHz pre | MHz post | °C pre | °C post | label |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 0 | 0 | 0 | `mibz2-default33` | - | - | 0.0406 | 0.1236 | - | - | - | - | - |
| 1 | 0 | 1 | `mibz2-pipe3-both` | mibz2-default33 | 0.201 | 0.0445 | 0.1357 | - | - | - | - | - |
| 2 | 0 | 2 | `mibz2-pipe3-runfed-ctl` | mibz2-pipe3-both | 0.200 | 0.0383 | 0.1268 | - | - | - | - | - |
| 3 | 0 | 3 | `mibz2-pipe3-runfed-reg` | mibz2-pipe3-runfed-ctl | 0.200 | 0.0396 | 0.1204 | - | - | - | - | - |
| 4 | 0 | 4 | `mibz2-pipe3-runfed-ls` | mibz2-pipe3-runfed-reg | 0.201 | 0.0385 | 0.1218 | - | - | - | - | - |
| 5 | 0 | 5 | `mibz2-pipe3-runfed-hs` | mibz2-pipe3-runfed-ls | 0.201 | 0.0366 | 0.1106 | - | - | - | - | - |
| 6 | 0 | 6 | `mibz2-pipe3-runfed-dir` | mibz2-pipe3-runfed-hs | 0.200 | 0.0385 | 0.1166 | - | - | - | - | - |
| 7 | 1 | 0 | `mibz2-pipe3-both` | mibz2-pipe3-runfed-dir | 0.200 | 0.0357 | 0.1125 | - | - | - | - | - |
| 8 | 1 | 1 | `mibz2-pipe3-runfed-ctl` | mibz2-pipe3-both | 0.201 | 0.0381 | 0.1153 | - | - | - | - | - |
| 9 | 1 | 2 | `mibz2-pipe3-runfed-reg` | mibz2-pipe3-runfed-ctl | 0.200 | 0.0366 | 0.1109 | - | - | - | - | - |
| 10 | 1 | 3 | `mibz2-pipe3-runfed-ls` | mibz2-pipe3-runfed-reg | 0.200 | 0.0401 | 0.1217 | - | - | - | - | - |
| 11 | 1 | 4 | `mibz2-pipe3-runfed-hs` | mibz2-pipe3-runfed-ls | 0.201 | 0.0417 | 0.1300 | - | - | - | - | - |
| 12 | 1 | 5 | `mibz2-pipe3-runfed-dir` | mibz2-pipe3-runfed-hs | 0.201 | 0.0356 | 0.1128 | - | - | - | - | - |
| 13 | 1 | 6 | `mibz2-default33` | mibz2-pipe3-runfed-dir | 0.200 | 0.0383 | 0.1147 | - | - | - | - | - |

## Each pair's log ratios over the incumbent, in run order (descriptive)

| arm | round | state | log wall ratio | log CPU ratio |
|---|---|---|---|---|
| `mibz2-pipe3-both` | 0 | unknown | +0.0917 | +0.0934 |
| `mibz2-pipe3-runfed-ctl` | 0 | unknown | -0.0583 | +0.0256 |
| `mibz2-pipe3-runfed-reg` | 0 | unknown | -0.0249 | -0.0262 |
| `mibz2-pipe3-runfed-ls` | 0 | unknown | -0.0531 | -0.0147 |
| `mibz2-pipe3-runfed-hs` | 0 | unknown | -0.1037 | -0.1111 |
| `mibz2-pipe3-runfed-dir` | 0 | unknown | -0.0531 | -0.0583 |
| `mibz2-pipe3-both` | 1 | unknown | -0.0703 | -0.0194 |
| `mibz2-pipe3-runfed-ctl` | 1 | unknown | -0.0052 | +0.0052 |
| `mibz2-pipe3-runfed-reg` | 1 | unknown | -0.0454 | -0.0337 |
| `mibz2-pipe3-runfed-ls` | 1 | unknown | +0.0459 | +0.0592 |
| `mibz2-pipe3-runfed-hs` | 1 | unknown | +0.0851 | +0.1252 |
| `mibz2-pipe3-runfed-dir` | 1 | unknown | -0.0731 | -0.0167 |
