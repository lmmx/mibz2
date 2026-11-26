# A paid per-block choice between the engineering and delimiter leaders

Across the 228 admitted Wikidata blocks, choosing the faster arm separately on each block in hindsight saved **2.28%** of the better fixed arm's measured decode-block CPU. Requiring each choice to repeat on an independent set of observations reduced the saving to **0.80% and 1.10%**, depending on which set supplied the choices. Both were below the preregistered 5% screening criterion. The result gives little local support for keeping these two specializations resident solely to choose between them by block. [Protocol, committed before timing](protocol.md) · [Full decision table](prices.csv) · [Raw paid observations](observations.csv.xz)

The engineering arm used the gated narrow/immediate endpoint writer with one-edge lookahead at 80d8409359214b4e5abfd4430213921e82e9cd30. The second arm used the counted delimiter constructor with runtime POPCNT dispatch at 3651470cb0ef5458e75cf97d3de363a1c73f2f76. Both used the same block-level clocks and worker source as the prior full-corpus budget. The measured block cost includes route recognition, construction of its actual run front, mark validation, inversion including output, RLE1 expansion and block CRC verification. All 10,944 observations checked against the Python libbzip2 output slice. Separate uninstrumented normal calls passed on the warm iteration. The initial full 431-block validation combined the computed CRCs across each of twelve streams and checked the trailers. All admitted inverses were checked again when capturing retained capacity. No scorer output enters the new ratios. The source manifest and checksums identify the exact local capture. [Architecture and scope](../README.md) · [Validation](check.txt) · [Manifest](manifest.txt)

| Decision | Evaluation blocks | POPCNT choices | CPU ratio to the best fixed arm on the same evaluation blocks |
|---|---:|---:|---:|
| Full-sample hindsight | 228 | 102 | .9772 |
| Choose on group A, price on group B | 228 | 120 | .9920 |
| Choose on group B, price on group A | 228 | 99 | .9890 |
| Fit quote density on training streams, price held-out streams | 93 | 7 | 1.0052 |
| Hindsight on the held-out streams and rounds | 93 | 49 | .9719 |

Each block received 24 observations per arm on one fixed Xeon CPU. The two calls were adjacent and alternated first position across iterations. Each disjoint group contains 12 observations at six of each first position per arm and block. Cost for each table entry is the sum of per-block group medians. Across all 24 observations, the writer cost 676.95 ms and counted POPCNT cost 691.99 ms. Choosing separately in hindsight reduced this sum to 661.54 ms. Only 143 of 228 block winners repeated between groups. As a sensitivity check, summing the raw whole-rotation durations for the independently chosen block policies gave median ratios of .992 (A choices paid on B) and 1.009 (B choices paid on A). Host variability contributes appreciably to individual-block minima. The hindsight figure is an optimistic ceiling within this observed two-arm comparison. [Per-block identities, features and prices](blocks.csv) · [Raw-duration sensitivity](paid-rotations.csv)

The one-cut rule used the front histogram's count of byte 34 and the row count, both available when the worker chooses its run kernel. Training on 135 blocks chose POPCNT when quote_count/rows ≤ 2243/15000 and the writer above that threshold. The 93 held-out blocks comprised complete compressed streams 0, 3, 6 and 9. Applying the gate there cost 248.71 ms against 247.42 ms for fixed POPCNT and 249.09 ms for the writer. This table selects between costs already measured on each block. A physical gate would additionally read the histogram bin and apply the integer comparison. The evidence does not support a worker or workstation benchmark for this gate. [Predeclared holdout and tie handling](protocol.md) · [Threshold and agreement](selection.txt)

## Retained worker memory

A separate untimed replay fed the same admissions to both reusable kernels and recorded each vector's Rust capacity times its element size, including nested FIFO, endpoint, lane and second-level buffers. The cumulative capacities increased monotonically. Final specialized vectors occupied **3,957,624 bytes** for the writer and **7,197,968 bytes** for POPCNT. Keeping both fully warmed took **11,155,592 bytes per worker**. The incremental fully warmed F2 storage beside the writer would be 50,385,776 bytes across seven workers or 71,979,680 across ten if each reached these capacities. These totals cover the two specialized kernels on the measured admission sequence. A particular subset of blocks could warm different buffers. Shared front and output storage, scalar object headers, the fallbacks used outside the measured admission, allocator overhead, temporary in-call allocations and process RSS are outside this capacity count. The production no-increase condition remains unverified. [Per-block retained capacities](capacity.csv)

## Reproduction

Run from a checkout containing this receipt with the recorded toolchain, a free pinned CPU and the canonical compressed corpus. The script extracts both candidate revisions from git, regenerates the historical writer and builds a temporary instrumented library without modifying the shipping code. The optional second argument names an existing libbzip2-expanded reference file or the destination for generating it.

```sh
d=research/astra-e/budget/portfolio
bash "$d/run.sh" /tmp/astra-portfolio /tmp/astra-portfolio/oracle 3
mkdir -p /tmp/astra-portfolio-tables
python3 "$d/report.py" /tmp/astra-portfolio /tmp/astra-portfolio-tables
```

Running `python3 research/astra-e/budget/portfolio/report.py` reproduces the tables directly from the preserved compressed observations.
