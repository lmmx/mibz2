# Dense-block admission on the i9-10900K

Source `badb2aec3624bd65606c429b2e6f79e9d9d0050f` is the new independently measured target reference. Admitting five additional dense Wikidata blocks into the combined writer beat the frozen predecessor for both wall and peak RSS at every tested worker count. Every candidate/Lazy wall interval remains above 0.80, so the 1.25× full-range objective remains open. The preregistered space claim passes at every W. [Original workstation archive](evidence/edge-i9-683c22e.tar.gz) · [Decision replay](evidence/edge-i9-683c22e.json) · [Preregistration and dry run](edge-request.md)

| Workers | Edge / 0.5.6 Lazy wall [95%] | Throughput / Lazy | Median peak RSS / Lazy | Edge / combined wall [95%] | Median peak RSS / combined | Edge / opend wall [95%] |
|---:|---|---:|---:|---|---:|---|
| 7 | 0.8670 [0.8600,0.8742] | 1.1534× | 0.6897 | 0.9789 [0.9720,0.9859] | 0.8094 | 0.9329 [0.9259,0.9399] |
| 8 | 0.8836 [0.8724,0.8950] | 1.1317× | 0.6811 | 0.9733 [0.9633,0.9835] | 0.8159 | 0.9455 [0.9375,0.9535] |
| 9 | 0.8970 [0.8788,0.9156] | 1.1148× | 0.6815 | 0.9763 [0.9553,0.9977] | 0.8113 | 0.9487 [0.9301,0.9677] |
| 10 | 0.8759 [0.8613,0.8907] | 1.1417× | 0.6788 | 0.9585 [0.9430,0.9742] | 0.8231 | 0.9319 [0.9205,0.9434] |

Edge used less memory in all 64 matched pairs versus each reference. It finished faster in 63/64 pairs against Lazy, 62/64 against opend and 61/64 against the measured combined writer. Its maximum peaks at W7/W8/W9/W10 were 194,276 / 206,464 / 212,836 / 223,640 KiB, versus 304,460 / 318,284 / 332,344 / 337,964 KiB for Lazy. Against combined, all four paired peak-RSS upper limits were below 0.834 and all four edge maxima were below their paired-arm maxima. The earlier ungated Open memory measurements belong to a separate four-arm run; direct edge/Open wall and RSS remain unmeasured. At the point estimates, reaching 0.80 requires reductions of 7.7%, 9.5%, 10.8% and 8.7% in the new candidate's elapsed time. [Original matched rows, process CPU, medians and maxima](edge-i9.csv) · [Earlier combined/Open sweep](combined-i9.md)

The complete edge processes used 0.8754/0.8831/0.8811/0.8666 times the CPU seconds of Lazy at W7/W8/W9/W10. Versus the frozen combined executable, the CPU ratios were 0.9785/0.9837/0.9711/0.9805. Counters for retired instructions, cache transfers and dependency stalls were outside the workstation timing contract. The prospective route diagnostic measured a real change on one sampled dense block. Its coverage of the other routes, and its calibration, belong to its separate report. [CPU readout](edge-i9.csv) · [Preserved prospective proxy](../results/edge-route-pricing/)

The Coordinator queued ready commit `e00b278656838aad9487ffa8fb7333eaa0c4705e` through master `be74d57`. The i9 dry archive `83c10a7` completed with 16 worker/arm gates before timing. In the timed sweep the candidate and predecessor each passed 338 cases, and each stock control passed its 98-case gate. The two binaries used the same 31 refreshed scorer projection blobs from upstream `75297a1d57b2bdf262c3a19f3064848dedf2c493`; both reports printed `v1 at 9c31461df7bef24cecf79348545b27ee26846835+dirty` after applying that projection to older frozen worktrees. The target hardware was unpinned on 20 logical CPUs, with 7–10 workers, 4W slots, two-second settles and 16 rotated rounds. Every one of the 256 measured processes produced 838,800,784 output bytes from four copies of Wikidata. The archive records candidate and predecessor binary hashes and fixed source entries. The separate frozen predecessor was the previous i9-ready build `8c9640d8ab186f81a0500f48e606e0c21c2a6669` for combined source `80d8409359214b4e5abfd4430213921e82e9cd30`. [Coordinator request](https://github.com/lmmx/mibz2/pull/145#issuecomment-6997357501) · [Coordinator queue](https://github.com/lmmx/mibz2/pull/618)

The measured archive SHA-256 is `3a10607c7d4ff026f4f3a7241a6d72dff10f590c07fd9b19eab9a7c6fbed2559` at evidence commit `683c22e98aa41e42f90209063aa3650b224c595f`. The verifier checks both builds, all gate transcripts and the rotor, and reproduces the complete stored decision. A checkout with the registered Git objects and Python 3 suffices.

```sh
d=research/astra-e/workstation
python3 "$d/verify_edge.py" "$d/evidence/edge-i9-683c22e.tar.gz" | diff - "$d/evidence/edge-i9-683c22e.json"
python3 "$d/edge_metrics.py" "$d/evidence/edge-i9-683c22e.tar.gz" | diff - "$d/edge-i9.csv"
```
