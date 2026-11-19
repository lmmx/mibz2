# Two-seed nominal proxy panel

All six matched cells completed on the preserved binaries using the official v1 scorer at path `75297a1d`. Each decision used the same warm record, measured symbol tape, output length and latency setting in both arms. Candidate/control ratios follow.

| W | Block | Instructions | L2 transfers | L3 transfers | DRAM transfers | Past-L2 dependent links | Longest past-L2 chain | Overlap per DRAM dependent link | Modeled cycles |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 7 | 1 | 1.1238 | 1.0486 | 1.0583 | 0.9683 | 0.9160 | 0.6016 | 0.9492 | 1.0514 |
| 7 | 85 | 1.1326 | 1.0271 | 1.0348 | 0.9760 | 0.9002 | 0.8819 | 1.0733 | 1.0273 |
| 7 | 165 | 1.1377 | 1.0472 | 1.0459 | 1.0000 | 0.8849 | 0.6147 | 0.9463 | 0.9752 |
| 10 | 1 | 1.1238 | 1.0486 | 1.0583 | 0.9879 | 0.9220 | 0.7096 | 0.9492 | 1.0752 |
| 10 | 85 | 1.1326 | 1.0271 | 1.0348 | 0.9963 | 0.9038 | 2.8166 | 1.1133 | 1.0656 |
| 10 | 165 | 1.1377 | 1.0472 | 1.0459 | 1.0293 | 0.8892 | 0.7665 | 1.0684 | 1.0053 |

The candidate shortened the frozen block’s longest modeled past-L2 chain at W7 from 364 to 219 while requiring 12.4% more instructions per output byte. The dense block’s modeled cycle direction changes with W. These nominal samples have no whole-corpus weighting or latency feedback. Each estimate represents one simulated worker. Independent paired whole-worker wall time and RSS decide performance.

Reproduce with the registered matching gate binaries, corpus tapes under `research/astra-e/results/scalar-route-pricing/symbols`, and the paths below. The collector creates its raw observations exclusively. A fresh checkout preserves the original receipt.

```bash
source /workspace/work/env.sh
source /workspace/work/proxy-tools/env.sh
cd research/astra-e/two-seed/proxy
taskset -c 4 nice -n 15 python3 -B run.py /workspace/work/astra-two-seed-proxy-scorer /workspace/work/astra-two-seed /workspace/work/astra-scalar-route-build/chaingrind
python3 -B report.py /workspace/work/astra-two-seed-proxy-scorer
```
