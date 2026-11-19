# Independent ten-core i9 result for the combined writer

The combined writer is the best implementation measured on the target i9 so far. At 7–10 workers it delivered 1.097–1.127× the throughput of the 0.5.6 LazyCells reference with 15.1–18.1% lower median peak RSS. The preregistered paired RSS upper bound, median and maximum criteria all passed at each concurrency. The upper wall ratio exceeded 0.80 at every concurrency. Against ungated Open only W7 resolved a throughput advantage. [Measured target archive](evidence/combined-i9-c381da2.tar.gz) · [Independent evidence ref](https://github.com/lmmx/mibz2/tree/c381da23edd8f466bcfe8b8e6c850075d58588de/astra-e)

| Workers | Candidate / Lazy wall [95%] | Throughput / Lazy | Median peak RSS / Lazy | Candidate / opend wall [95%] | Candidate / ungated Open wall [95%] |
|---:|---|---:|---:|---|---|
| 7 | 0.8982 [0.8876,0.9090] | 1.1133× | 0.8489 | 0.9583 [0.9521,0.9646] | 0.9777 [0.9713,0.9841] |
| 8 | 0.9117 [0.9031,0.9205] | 1.0968× | 0.8288 | 0.9656 [0.9554,0.9760] | 0.9923 [0.9839,1.0009] |
| 9 | 0.9031 [0.8896,0.9168] | 1.1073× | 0.8229 | 0.9616 [0.9528,0.9704] | 0.9994 [0.9876,1.0113] |
| 10 | 0.8871 [0.8795,0.8947] | 1.1273× | 0.8191 | 0.9591 [0.9490,0.9692] | 0.9934 [0.9815,1.0055] |

| Workers | Median wall s, candidate / Lazy | Median peak KiB, candidate / Lazy | Maximum peak KiB, candidate / Lazy | Paired RSS upper 95% |
|---:|---|---|---|---:|
| 7 | 0.277562 / 0.309060 | 235840 / 277834 | 243624 / 294248 | 0.8600 |
| 8 | 0.271478 / 0.299859 | 247668 / 298816 | 258700 / 324840 | 0.8535 |
| 9 | 0.276193 / 0.305361 | 261798 / 318120 | 266428 / 334624 | 0.8271 |
| 10 | 0.286616 / 0.323418 | 268582 / 327906 | 275776 / 366024 | 0.8295 |

The workstation built source `80d8409359214b4e5abfd4430213921e82e9cd30` through ready commit `8c9640d8ab186f81a0500f48e606e0c21c2a6669`. The scorer reported v1 at `9a9c169d70639c1c3e56cb85b3679fa7529e516c`, upstream path `604dac3f46569edd9ac9b6706b23a68d9d7d8f8d`. The i9 dry run passed before timing. The measured run gated all four arms and recorded all 256 samples with the prescribed 4W slots, four corpus copies (838,800,784 output bytes/sample), two-second settle, 16 rotations and unpinned 20-CPU affinity. The candidate passed all 338 oracle cases. Reassessment of the raw samples reproduced the frozen decision. [Untimed dry run](evidence/combined-dry-8bced3f.tar.gz) · [Cross-checking program and proxy comparison](../results/combined-v1-proxy/heldout.py) · [Preregistration](combined-request.md)

The target samples also contain process CPU time to hundredths of a second. Combined/Lazy process CPU ratios were 0.904/0.904/0.902/0.902 at W7/W8/W9/W10. The corresponding ratios of CPU seconds per wall second were 1.006/0.991/0.999/1.017. Thus total occupied CPU fell by about a tenth across the grid even where the wall ratios varied; this measurement includes the scanner, workers and output. Per-stage CPU, instructions, IPC and traffic are outside these target receipts. [Process-CPU table and intervals](combined-i9-cpu.csv) · [Raw-data replay](cpu.py)

The dry SHA-256 is `635fa5924bddbf7e7457c504ad936834722d32891f5fdbd1f06cb835d9ab8584`. The measured SHA-256 is `70b3c3a88d330529ff2dc976ce27451a7065b8b8f5fc49e65f20c191a7d32bfa`. To replay the historical decision locally:

```sh
a=research/astra-e/workstation/evidence/combined-i9-c381da2.tar.gz
sha256sum "$a"
tar -xOzf "$a" run-20261119T013135Z/runs.jsonl > /tmp/astra-i9-combined.jsonl
python3 research/astra-e/workstation/assess.py /tmp/astra-i9-combined.jsonl
```

At the observed point estimates the remaining reduction relative to this implementation is 9.8–12.3% depending on concurrency. The actual target must satisfy the interval and RSS criteria at every worker count. The full v1 proxy was frozen on the research branch before the timed evidence was published; all three observed W7/8/10 Lazy ratios fell within its predicted intervals. It does not supply a W9 estimate. [Prediction and held-out comparison](../results/combined-v1-proxy/)
