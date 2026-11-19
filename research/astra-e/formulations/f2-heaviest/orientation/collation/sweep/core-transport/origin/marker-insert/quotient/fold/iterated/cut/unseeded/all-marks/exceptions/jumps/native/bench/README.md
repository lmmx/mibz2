# A physical-rank jump inverse from real capped input

On ten systematic Wikidata blocks, walking an already-built physical-rank index took 2.57× the CPU of the frozen scalar's entire inverse for fixed K8. Variable K16 with threshold 16 took 2.96×. All ten per-block point estimates exceeded 1 in both panels. Paying for coalescing, context discovery, exception records, mask/rank construction and output increased the ratios to 5.89 and 5.69. Both implementations failed the registered complete-CPU and peak-memory screens.

| Input supplied | Fixed K8 / scalar | Variable K16 τ16 / scalar |
|---|---:|---:|
| Complete jump index | 2.571 [2.474, 2.673] | 2.965 [2.822, 3.115] |
| Coalesced runs and contexts | 3.906 [3.711, 4.111] | 3.089 [2.965, 3.218] |
| True cap records | 5.886 [5.764, 6.009] | 5.685 [5.436, 5.946] |

Each ratio pairs the sum across all ten blocks within each round. Intervals are two-sided 95% t intervals over four six-round batches of paired log ratios. Frozen scalar, LazyCells and OpenGated consumed 25.23, 25.12 and 24.20 ms respectively in the fixed panel and 26.21, 25.67 and 25.41 ms in the variable panel (medians of ten-block totals). All controls use the original Rust source and executable. Measured complete/Lazy was 5.782 [5.425, 6.162] fixed and 5.495 [4.768, 6.333] variable. The two panels ran sequentially, each with its own matched controls.

| Phase in the complete arm, median ms per ten blocks | Fixed | Variable |
|---|---:|---:|
| Read and coalesce original cap records | 1.08 | 1.10 |
| Construct context partition and discover modes | 50.94 | 67.67 |
| Construct h1 and h8 indices, including intermediate layers | 53.60 | 27.01 |
| Emit the marked output immediately after construction | 40.47 | 52.39 |
| Inclusive donor timing, including temporary releases | 148.11 | 148.03 |
| Emit with a supplied prebuilt index, inclusive | 65.72 | 76.70 |

The variable partition reduces components from 815,844 to 184,004 across the ten blocks, and doubles exception rows from 409,432 to 818,537. This halves index construction time, but its context discovery and output require more time. Fixed discovery uses the full direct context constructor, including temporary poststates and exception runs. A cheaper fixed constructor could remove some of its 50.94 ms. Even the 40.47 ms fixed emission immediately following construction exceeds the 25.23 ms scalar inverse. The independent supplied-index experiment revisits ten retained indices with different cache residency. Its slower output is an observed order/working-set effect consistent with locality; the experiments do not identify a specific cache or stall cause. These are native inverse-stage results on a Xeon, covering neither scanner throughput nor target-machine concurrency.

| Fresh one-block process, median / maximum VmHWM in KiB | Fixed panel | Variable panel |
|---|---:|---:|
| Prepared | 7,206 / 13,452 | 5,468 / 7,532 |
| Supplied geometry | 7,240 / 13,452 | 5,500 / 7,532 |
| Complete | 7,206 / 13,576 | 5,466 / 7,532 |
| Scalar | 4,676 / 6,000 | 4,676 / 5,996 |
| LazyCells | 4,422 / 9,140 | 4,422 / 9,200 |
| OpenGated | 4,820 / 9,200 | 4,822 / 9,140 |

The maximum retained per-block index capacity was 5.42 MiB fixed and 2.24 MiB variable. Peak simultaneously live index layers were 7.93 and 2.78 MiB respectively, before raw data, context geometry, output and implementation overhead. The fresh-process measurements include all cold setup for the selected arm. They sample HWM before loading the independent expected output. Variable had a lower maximum than LazyCells here, alongside higher median and maximum than scalar. The complete-arm process holds no prepared models or oracle inputs during the measurement.

Both bridges reproduced all 8,999,915 marked pre-RLE1 bytes for blocks 1, 41, 85, 134, 176, 225, 269, 313, 359 and 397. The donor checked every warm-up and paid output, and each gate verified the ten capped and marked output SHA256 values. The preserved input receipts bind those bytes to prior independent libbzip2 checks and block CRCs. Native source 8e3e6732 passed exhaustive/adversarial tests with 148,384 complete marked outputs, including 75,202 phase repairs, plus both models on every real block. No registered real mark required the phase repair. The current panel ends before RLE1 and CRC, and exercises the existing positive run-input admission only. It provides no new production or target result. [Native checks](../small.txt) · [Real rows and geometry](../real.csv)

## Reproduction and receipts

Preregistration 2047502883b03a5d9a53a1457b6dfdafd1865076, bridge fbad13526ca1a1a357f5b3253bb29598c181cd50, native index and real gate 8e3e673230d93b36e9313e554af76a42c4281f62. The donor's scalar source is d0a42980e896211906234c7679b38760ae44a46e. The build manifest pins both Rust binaries, all native source hashes, the generated selector, GNU C++ version, flags and shared object. [Build](build.json) · [Registered method](PLAN.md)

From the worktree root, use the unchanged capped donor binaries and its ten verified marked outputs. The build step generates the variable selector from the recorded git object.

```bash
bench=$(dirname "$(rg --files | rg '/jumps/native/bench/PLAN.md$')")
donor=/workspace/work/astra-powers-original/capped
scratch=/tmp/astra-clock-free-paid
python3 "$bench/run.py" "$donor" "$scratch" build
for mode in fixed variable; do
    taskset -c 0 nice -n 8 python3 "$bench/run.py" "$donor" "$scratch" gate "$mode"
done
for mode in fixed variable; do
    taskset -c 0 nice -n 8 python3 "$bench/run.py" "$donor" "$scratch" price "$mode"
done
for mode in fixed variable; do
    taskset -c 0 nice -n 8 python3 "$bench/run.py" "$donor" "$scratch" rss "$mode"
    python3 "$bench/report.py" "$scratch/$mode" > "$scratch/$mode/summary.json"
done
```

Each raw times file contains the 1,440 measured donor observations. Each raw stamps file contains 30 provisional supplies/verification calls and 720 paid bridge calls. The joined pairs preserve donor CPU, all component stamps, capacities and repair counts. Each mode also includes 180 independently launched one-block acquisitions. Run `report.py` directly against the archived mode directories to verify the rotation, joins and reported outputs. Exact gate calls use the Rust donor's original unmodified interface. [Fixed data](fixed/) · [Variable data](variable/)

On 2026-11-19 the fixed panel ran 07:37:51–07:38:02 UTC and variable ran 07:38:02–07:38:12 UTC. Linux 6.18.57 on Intel Xeon Platinum 8573C exposed logical CPUs 0–4 with a four-equivalent cgroup quota. Both sampled processes were pinned to CPU0 at nice +8. Research work on other CPUs was permitted, including a CPU4 held-out panel, a CPU2 job and a CPU1 low-priority structural check. A CPU3 ordered-live structural process was observed before the run. Thread CPU excludes time descheduled, and shared-cache or frequency interference remains possible. No performance counters or independent CI/i9 observations were taken for these arms.
