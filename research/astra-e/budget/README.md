# The inverse budget on real blocks

The 228 large blocks admitted to the delimiter construction consumed **82.70% of LazyCells' worker CPU in inverse reconstruction** on this local full-corpus diagnostic. Across ten corpus passes the fraction ranged from 80.30% to 84.89%. Including separately measured serial scanner and output-copy CPU in the denominator gives 73.58% (70.69–78.17%). Applying the prepaid rank-tape result of .675× to every admitted inverse would yield .7312 of worker CPU or .7609 of the latter serial CPU sum. The oracle measured nine actual blocks. Applying its point estimate to only those nine would give .9892 and .9906 respectively. A physical constructor still has to acquire its rank answers. [Oracle study](../formulations/f2-scheduler/publication/README.md) · [All conditional scenarios](model.csv) · [W7–10 i9 role evidence](target.md)

## How much remains to pay for an implementable constructor?

For an additive baseline, if the affected stage occupies fraction f, a new stage ratio r gives whole ratio 1−f(1−r). A .675 stage needs f≥.6154 to reach .80. A free stage needs f≥.20. With the measured local fraction of .7358 for all 228 admissions including serial scanner and copy, the new complete admitted inverse must cost at most .7282× Lazy. If the .675 point result generalized, the difference would leave about **.0532 of admitted Lazy inverse CPU** to discover the answers which the replay received free, assuming the rest of the computation stays at its baseline cost. On a worker-only denominator, the admitted stage ceiling is .7582. On either denominator, making just the nine oracle blocks free cannot approach .80.

| Local additive denominator | Affected Lazy inverse | Baseline fraction | Whole at .675× | Whole at free inverse |
|---|---|---:|---:|---:|
| Worker | Nine oracle blocks | .0332 | .9892 | .9668 |
| Worker | All 228 admissions | .8270 | .7312 | .1730 |
| Worker | Every block | .8539 | .7225 | .1461 |
| Worker + scanner + output copy | Nine oracle blocks | .0288 | .9906 | .9712 |
| Worker + scanner + output copy | All 228 admissions | .7358 | .7609 | .2642 |
| Worker + scanner + output copy | Every block | .7615 | .7525 | .2385 |

Each number is based on the per-rotation sum for a complete corpus. The ratios at .675 and zero are conditional substitutions. Median scanner-only CPU was 108.27 ms (95.04–131.60) and copy-only CPU was 26.09 ms (22.03–31.70). These isolated scopes omit feeder acquisition, queueing, reader locks, scheduling and pipeline startup/drain. They can overlap with each other and with inversion in the shipped worker pipeline. The ordered output-copy test uses the actual decoded block lengths and one reusable 4 MiB caller buffer with a checked byte copy per block. Its source is a preloaded oracle buffer. The scanner test runs the production Huffman parser on the in-memory compressed file with symbol-buffer reuse, verifies every symbol and stored header field against a separate scan, and includes the final end-of-stream call. [Isolated scan and copy](solo.csv) · [Raw scanner scopes](scanner.csv.xz) · [Raw copy scopes](copy.csv.xz)

## Workers, phases and input classes

The identical pinned Xeon process ran a full corpus in normal block order for each arm at each rotation. Entries are medians of the sums of thread CPU milliseconds for 431 blocks. The median of row totals can differ from the sum of independently calculated phase medians.

| Arm | Front | Complete inverse | Separate RLE1 | CRC + return | Decode block total |
|---|---:|---:|---:|---:|---:|
| LazyCells | 105.44 | 1006.36 | 38.16 | 15.92 | 1180.67 |
| Gated Open-Return | 96.56 | 735.90 | 38.69 | 15.59 | 889.51 |
| Narrow/immediate endpoint with one-edge lookahead | 91.57 | 711.21 | 35.38 | 16.29 | 856.51 |
| Dynamic eight-lane delimiter refill | 98.87 | 909.52 | 32.80 | 15.26 | 1072.69 |
| Bundled RLE1 plus narrow/immediate endpoint and lookahead | 100.95 | 775.87 | 0.79 | 15.81 | 898.44 |

Front includes the normal route test, selected MTF/run-front, count preparation and primary validation. Inverse includes the complete inversion on its actual route, including fallback. The bundled worker's inverse column includes expanded output for its 228 fused blocks. CRC + return includes CRC verification and the return to caller. The five arms share the rest of the worker code. Even those phases vary across rotations in this VM. The dynamic refill/Lazy complete-worker CPU ratio ranged from .637 to 1.347 and opend/Lazy from .516 to 1.270. This run supports a budget for a large isolated change. Local scorer, CI and i9 paired timings remain the sources for performance ranking. [Per-arm and paired totals](scope-summary.csv) · [Raw block × phase × rotation durations](stages.csv.xz)

| Class | Blocks | Inverse rows | Output bytes | Lazy share of total worker CPU spent in the class's inverse |
|---|---:|---:|---:|---:|
| Accepted large delimiter blocks | 228 | 205,197,794 | 205,191,352 | .82704 |
| Other large blocks on the run route | 5 | 4,499,868 | 4,499,952 | .02670 |
| Small run-route blocks | 150 | 8,430 | 8,427 | .00035 |
| Small column-route blocks | 48 | 465 | 465 | .00008 |
| Nine in the oracle study, also in the first row | 9 | 8,099,915 | 8,099,814 | .03321 |

Class identities agree with the complete native delimiter admission record, checked against run density and the 65,536-row size split. Global block indices 1, 85, 134, 176, 225, 269, 313, 359 and 397 name the nine oracle blocks. The current arms and baseline all passed with every computed block CRC equal to its stored CRC and every expanded output byte equal to Python libbzip2. A separate normal call to each arm also passed for every block. The initial pass combined the computed block CRCs by stream and checked the twelve stored bit-level trailers. Full input SHA-256 is 82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915. Expanded SHA-256 is 3f075e30f4f8c73414150655afbf57caa2647c7927e743fc0635b4912d65cdaf. [Class weights](classes.csv) · [Checks](check.txt) · [Source, toolchain and machine](manifest.txt)

## Scope and reproduction

The script copies the library into a scratch directory and generates a second version of the actual worker by inserting four boundary timestamps in its existing source. The caller takes the fifth immediately after return. Linux CLOCK_THREAD_CPUTIME_ID advances only while this thread runs. The median adjacent read was 224 ns, p95 238 ns. No correction is subtracted. On tiny blocks the clock cost is significant, so their combined contribution is retained in the full corpus. Oracle comparisons and CSV writes are outside the measured block scope. The production mark and CRC checks stay inside. The temporary process retains all symbols, the libbzip2 oracle and five reusable scratches for repeated checks. Its footprint cannot measure the production process RSS. One warm corpus precedes ten measured orders, rotating arm order once per corpus. The binary uses rustc opt-level 3 and one codegen unit with the detected kernels.

Run from the repository root with Linux x86-64, the manifest toolchain and an available CPU number. To reproduce the summary without repeating any measurement, run the last command only. Use a separate directory for the new summary when comparing two captures.

```sh
d=research/astra-e/budget
bash "$d/run.sh" /tmp/astra-worker-budget corpora/wikidata.bz2 10 3
mkdir -p /tmp/astra-worker-tables
python3 "$d/summarize.py" /tmp/astra-worker-budget /tmp/astra-worker-tables
python3 "$d/summarize.py"
```

The i9 sweep's serial scanner already consumed .720–.778 of Lazy wall across the target workers. If scanner CPU per corpus held, at W9 and W10 a slowdown of only about 3.5% or 2.9% in the scanner would exhaust that stage's room to the .80 target. A decisive follow-up pairs the admitted inverse scopes in the independent W7–10 i9 recipe with whole-worker wall/RSS and scanner, worker and reader thread CPU. A matched pre-scanned-symbol diagnostic can price the worker algebra separately. The end-to-end gate must still run the real compressed input and drain checked output. [Measured target roles and conditions](target.md)
