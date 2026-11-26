# Prepared heavy-chain LF emission

With free preparation, the depth-16 smooth jump terminal took **1.279× scalar CPU** [1.239, 1.320] on the ten fixed Wikidata blocks. Depth 16 was the fastest tested depth on every block. Its per-block ratios against scalar ranged from 1.203 to 1.418. All three depths failed the registered terminal screen.

| Arm | Median summed CPU, ms | Inclusive CPU / scalar, 95% interval | CPU / LazyCells | CPU / OpenGated | One-block VmHWM, median / maximum KiB |
|---|---:|---:|---:|---:|---:|
| Smooth depth 4 | 109.11 | 4.526 [4.367, 4.690] | 4.505 | 4.657 | 6,340 / 10,504 |
| Smooth depth 8 | 43.94 | 1.842 [1.749, 1.939] | 1.833 | 1.895 | 7,156 / 14,596 |
| Smooth depth 16 | 30.96 | 1.279 [1.239, 1.320] | 1.273 | 1.316 | 8,948 / 19,204 |
| Scalar | 24.01 | 1 | | | 4,660 / 5,980 |
| LazyCells | 24.11 | | | | 4,404 / 9,120 |
| OpenGated | 22.89 | | | | 4,834 / 9,184 |

At depths 4, 8 and 16 the ten-block orbit required 3,768,860, 1,108,546 and 575,661 branching-state visits respectively, interspersed with 1,933,051, 708,399 and 376,879 deterministic stretches. The native indices store shared heavy-chain spellings and locate a branch row in its source slices. Deeper contexts substantially reduced conditional terminal time. The depth-16 implementation still needed 30.96 ms against 24.01 ms for scalar, which paid its complete inverse from the original capped last column. Measured wall or CPU on the target workstation remains unavailable for these arms. [Structural counts](../../blocks.csv) · [Exact index construction](../INDEX.md)

The depth-4/8/16 indices occupied at most 2,877,805 / 3,391,105 / 4,250,049 retained vector bytes for one prepared block. The conditional CPU process retained ten per depth, totaling 14,149,015 / 16,451,804 / 22,135,178 bytes before output buffers and donor data. Each fresh memory process held only one selected arm and block, including the cost of constructing the index, and sampled VmHWM before opening the reference output. The first ten depth-16 emissions immediately following index construction took 24.35 ms total in the timing process's initial unpaired gate. The registered steady comparison revisited indices in the donor's rotated block order. Cache residency can therefore affect the terminal price. The single initial traversal has no matched interval, and does not establish a faster complete computation. [Full comparisons and memory](panel/summary.json) · [Cold and steady stamps](panel/stamps.csv)

The unmodified donor checked all 1,440 paid calls against the exact marked inverse, plus all 60 outputs in each gate. Ten fixed true capped records and marked-output hashes matched their registered corpus receipts. Their 8,999,915 pre-RLE1 output bytes map to independent libbzip2 and block-CRC checks in the original donor. Separately, the native gate verified 26,999,745 physical row/depth combinations, all 30 full real outputs and 39,052 finite outputs. The current paid panel begins after the scalar frontend and ends before RLE1, CRC and worker output. [Input identities](panel/gate.json) · [Independent finite and real gate](../GATE.md)

## Reproduction

Preregistration 29d42a6c, adapter e8008fb8, exact native ef679d8d020748d7819c7913c8efb82e7519500d, scalar reference d0a42980e896211906234c7679b38760ae44a46e. Each of the 24 rounds sums thread CPU over all ten blocks before pairing. Four mean log ratios, each from six consecutive rounds, supply the two-sided 95% Student interval. The arm and block positions rotate within the original Rust executable. The shared object and donor binaries are pinned in the build manifest. [Preregistration](PLAN.md) · [Build](panel/build.json)

From the worktree root with the original capped donor and input receipts available, run

```bash
bench=$(dirname "$(rg --files | rg '/jumps/smooth/native/bench/PLAN.md$')")
donor=/workspace/work/astra-powers-original/capped
scratch=/tmp/astra-smooth-paid
taskset -c 0 nice -n 10 python3 "$bench/run.py" "$donor" "$scratch" build
taskset -c 0 nice -n 10 python3 "$bench/run.py" "$donor" "$scratch" gate
taskset -c 0 nice -n 8 python3 "$bench/run.py" "$donor" "$scratch" price
taskset -c 0 nice -n 8 python3 "$bench/run.py" "$donor" "$scratch" rss
PYTHONDONTWRITEBYTECODE=1 python3 "$bench/report.py" "$scratch"
```

The paid CPU interval ran 07:53:13–07:53:21 UTC on 2026-11-19, pinned to CPU0 at nice +8 on a Xeon Platinum 8573C. The container exposed logical CPUs 0–4 and a four-equivalent cgroup quota. Other research tasks had separate lanes, including planned CPU3 work and a low-priority CPU2 compiler. Shared cache and frequency effects remain possible. The raw Rust times, 750 experimental phase stamps, 180 fresh process records and per-block intervals are retained. No independent CI or i9 run was requested. [Raw times](panel/times.csv) · [Fresh HWM](panel/rss.csv) · [Joined observations](panel/paired.csv)
