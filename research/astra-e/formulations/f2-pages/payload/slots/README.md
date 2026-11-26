# Return all eight compact responses to fixed F2 slots

The split64, eight-byte eager endpoint at ab7076541459b8b66a137c9f1cc0804e0cd72de0 is the direct reference. It already locates eight live rows with a fixed loop. It then processes the responses by enumerating set bits in the live mask, loading each payload, testing for retirement and updating its word path and row. The new programs specialize the response portion of a fully occupied wave. Fixed walks the eight lane numbers in ascending order. Preload first reads all eight compact run responses into a fixed array and then updates them in ascending order. Bulk accumulates a retirement mask before updating, with a fixed loop for the all-advancing case. All use the historical set-bit loop for partial waves. The entry preload.rs selects the preloaded schedule explicitly.

All rank queries in a wave use the old lane rows, whose run records stay immutable. Updating one path changes only that lane's private vector and row. Retired jobs are published after these updates under the original ascending-lane mask schedule. This preserves allocation order, spelling, merge and exact marked output, including on multiple cycles and periodic inputs. Preload has an eight-element temporary with 64 bytes of logical stack contents. Its worker heap buffers have the same capacities as the control; register allocation, spills and instruction scheduling depend on the compiler.

Across all 228 real admissions, native instrumentation observed 2,476,362 waves with 19,750,839 first-level predecessor queries. Exactly 2,464,245 waves had eight active lanes (99.51% of waves). Of these, 1,729,332 retired no jobs (70.18%). All real outputs totaled the same 205,197,794 inverse bytes as the independent Segmented kernel. All three schedules also passed the 11,264-case forced arbitrary-mark comparison. The instrumented binary that produced waves.csv was separate from the timed one.

Two fresh processes compared the historical compact kernel and all three schedules over 64 laps and nine systematic blocks. The crossover used all eight chosen forward/reverse orders eight times per block. All four arms occupied each position sixteen times. Many CPU2 intervals are wider than on CPU4. The table shows paired thread-CPU ratios relative to the unmodified compact kernel. The wall/CPU clocks and all four phases are in first.csv and repeat.csv.

| Response schedule | CPU4 discovery | CPU2 discovery | CPU4 complete inverse | CPU2 complete inverse |
| --- | ---: | ---: | ---: | ---: |
| Fixed slots | 0.9140 [0.8752,0.9545] | 0.8915 [0.8141,0.9763] | 0.9622 [0.9329,0.9924] | 0.9413 [0.8838,1.0026] |
| Preload responses | 0.8723 [0.8350,0.9112] | 0.8413 [0.7675,0.9222] | 0.9485 [0.9255,0.9721] | 0.9206 [0.8596,0.9859] |
| Bulk retirement mask | 0.9428 [0.9119,0.9749] | 0.9288 [0.8499,1.0150] | 0.9728 [0.9534,0.9925] | 0.9489 [0.8986,1.0021] |

Preload had the smallest complete CPU ratio in each process. Compared directly with fixed slots, the gain in complete CPU was less precise. The single-binary, matched native results identify a promising incremental schedule on the complete inverse stage. They do not independently establish the cause in hardware, worker-wall effects, peak process RSS or i9 performance.

```sh
d=research/astra-e/formulations/f2-pages/payload/slots
bash "$d/run.sh" /tmp/astra-e-fixed-slots
python3 "$d/report.py" /tmp/astra-e-fixed-slots/{first,repeat}.csv
```

## Independent result and follow-on experiments

The preserved entry is preload.rs at 6758831810b6a913c88ff079dede881b578ca03a. Independent EPYC CI run 44440381983 gave W1/W2/W4 ratios of 0.9134/0.9151/0.9225 against its guarded POPCNT predecessor and 0.8468/0.8631/0.8904 against LazyCells. Its W4 opend ratio was 1.0069 [1.0033,1.0105], and its W4 median RSS was 1.0735 of opend. All five arms passed 338 authoritative checks. This result is specific to that independent CI machine. The target i9 winner is maintained separately. [Original receipts](../../../../ci/evidence/compact-preload-44440381983.md) · [Campaign register](../../../../REGISTER.md)

| Frozen follow-on | Exact opportunity | Paid decision against 6758831 |
|---|---|---|
| cf8cf27 | 77.6% of queries have width one; split hot/cold words keep total payload bytes unchanged | +7–9% complete CPU. [Inputs and trials](singleton/) |
| a4f90a7 | Second-token discovery has 1.42M rank queries; fuse its mask construction and runtime-dispatch POPCNT | No resolved whole gain in either process. [Instructions and timings](second-rank/) |
| 68e14cb | Copy already published terminal ancestor spellings at the first level, second level, or both | Second-level cached tails cover only 6,753 bytes. First-level saves 571,594 splices after charging replacements, no resolved gain. [Proof and actual counts](suffix/) |
| ba8e9e0 | Eight-way AVX2 gathers with scalar or vector rank counts | +42–45% discovery CPU. [Mixed-lane checks and pricing](gather/) |
| 0dc5935 | Existing per-lane capacities permit 18.76M direct writes; natural growth unchanged | No resolved complete gain; exact capacities agree after every block. [Checks and trials](path-writes/) |
| 7a816ce | Validated last observed successor saves 6.17M ranks, including 728,401 hits outside whole-run images | More than doubles discovery CPU in side and embedded formats. [Rank and cache writes](successor/) |

Earlier frozen studies also priced alternate [lane widths](width/), [early survivor rank](early/), and the tiny bound for the [same-run shortcut](../self-run/). The fixed preload remains the delimiter control for new mathematical work. The official local rerun under master scorer path 75297a1 used the same frozen entry and the baseline grid alongside other preserved candidates. [Same-version series](../../../../results/refresh-75297a1/)
