# Confirm hardware bit counts against two controls

The experimental source is 3651470cb0ef5458e75cf97d3de363a1c73f2f76. A focused three-arm harness retains the historical counted candidate, then directly compares hardware and software through one matched function on the same Dispatch<2> type. Both matched arms reuse identical merge, second-level reconstruction and timed postlude code. All three have independent persistent decoder state and output buffers. The generic and hardware branches call the common source with their selected instruction policy. The normal scorer entry uses the runtime CPU guard.

Two fresh processes on CPUs 4 and 2 measured 60 laps. Each lap included nine adjacent systematic real blocks. Block order rotated and reversed across laps. Every six successive laps used all six arm orders at each block. Every output matched the independent Segmented output outside measurement. Each complete-inverse time includes preparation, mark transfer, full first-level discovery, merge and recursive marked expansion to raw inverse bytes. The front end, RLE1 and CRC remain in the independent whole-worker scorer.

| Hardware relative to reference | CPU4 estimate | CPU2 estimate |
| --- | ---: | ---: |
| Discovery vs counted | 0.8940 [0.8598,0.9296] | 0.9111 [0.8723,0.9517] |
| Full CPU vs counted | 0.9115 [0.8857,0.9380] | 0.9294 [0.9047,0.9547] |
| Full wall vs counted | 0.9140 [0.8834,0.9457] | 0.9017 [0.8409,0.9669] |
| Discovery vs matched generic | 0.8518 [0.8121,0.8934] | 0.8537 [0.8171,0.8919] |
| Full CPU vs matched generic | 0.8911 [0.8620,0.9210] | 0.9073 [0.8792,0.9362] |
| Full wall vs matched generic | 0.8865 [0.8524,0.9220] | 0.8661 [0.8215,0.9130] |

The matched generic/historical counted complete CPU comparisons were 1.0230 [0.9950,1.0517] and 1.0244 [0.9905,1.0594]. This narrower experiment also observed hardware/counted setup+mark 0.8265 and 0.8548. The storage and output representations are identical. Complete process peak RSS and the target workstation still require external measurement.

The source also passed the official 338-case v1 correctness gate including the full Wikidata stream, multiple-cycle and periodic blocks, CRC paths and mutations. Its raw official verdicts are in [the correctness receipt](../../official/rank-popcnt/score/summary.txt). Scorer master path revision 604dac3f46569edd9ac9b6706b23a68d9d7d8f8d and printed v1 at 6a8a366cd786d58a528000b58da12fd2cfb47be8 identify the same scorer contents.

```sh
d=research/astra-e/formulations/f2-pages/instructions/focus
bash "$d/run.sh" /tmp/astra-e-rank-focus
python3 "$d/report.py" /tmp/astra-e-rank-focus/{first,repeat}.csv
```
