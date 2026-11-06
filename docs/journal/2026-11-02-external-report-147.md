# External research report 147: the researcher on the walker capture: the repaired perf setup trusted for the main comparison, rare-event counters not (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02.
- **Source:** the independent researcher of reports 138, 140 and 144.
- **In reply to:** the walker-ladder capture at 29a0215 (`walker-ladder-20261102T004501Z.tar.gz`) and the coordinator's §8 write-up.

**Evidence cutoff:** 2 November 2026, after the capture. An independent check of the archive, its 618 raw records and the pinned source, and a review of the interpretation; no new run.

It is kept exactly as received (rendered text, cards flattened, tracking parameters included).

## The message, as received

`````text
**Yes—substantially. This strengthens my confidence in the repaired perf setup, and reduces the reason to switch to a Rust counter library now.** I would use this capture for the main registered walker comparison, while keeping specific qualifications on the rare-event counters.

The update is quite concrete: we now have a successful hardware capture showing that the repaired control path works, that the groups count together, and that the control overhead is very small for the principal measurements.

I verified the archive checksum you supplied, all three payload checksums in its MANIFEST.json[manifest](sandbox:/workspace/scratch/eb4b5d8c4a8d/analysis/walker_20261102T004501Z/MANIFEST.json), and the walker-ladder-20261102T004502Z.jsonl[raw measurement records](sandbox:/workspace/scratch/eb4b5d8c4a8d/analysis/walker_20261102T004501Z/walker-ladder-20261102T004502Z.jsonl). I also reviewed the relevant collection code at **`29a0215e3d06cdb436bf212501b01e54c48c7bf0`**.

## 1. The repaired collection now has convincing evidence behind it

Here is what I checked independently of the summary’s `valid: true` flag:

| Check | What this capture shows |
|---|---|
| Complete collection | **618 records:** 525 one-core measurements, 21 control-only measurements, 72 four-core measurements |
| Raw counter results | **3,012 event readings**, all matching the parsed values, running times and percentages |
| Counter activation | Every record contains the disabled → enabled → disabled sequence |
| Actual counting | Every event has positive running time and displays **100.00% running** |
| Group consistency | All events within each recorded group report the same running time |
| Workload and capacity checks | All records report successful checking, controlled collection and unchanged result-vector capacity |
| Four-worker synchronization | All 18 releases contain four workers; the worst overlap is **99.44%**, above the 95% gate |
| Deadline capacity | Workers performed **110–205 walks**, well below the 4,096-walk cap |
| Repeatability gate | P32’s maximum/minimum time ratio is **1.0258**, below the registered 1.05 limit |

This time, the printed `100.00%` accompanies real counted values and positive running time. The earlier zero-time formatting problem is absent.

The source also implements the relevant repairs: the FIFO paths and barrier setting travel as workload arguments, the ACK reader handles the NUL framing, and the result vector is reserved before the measured interval. The observed walk counts confirm that the capacity cap did not prematurely terminate these windows. [Pinned Python collector](https://github.com/lmmx/mibz2/blob/29a0215e3d06cdb436bf212501b01e54c48c7bf0/bench/walker_ladder.py#L174), [pinned Rust control and measurement code](https://github.com/lmmx/mibz2/blob/29a0215e3d06cdb436bf212501b01e54c48c7bf0/examples/walker_ladder.rs#L414).

**That is a meaningful change in evidence.** Previously, we had a plausible repaired design and stand-in tests. Now we have the repaired design exercised successfully across the actual measurement matrix.

## 2. The control machinery is cheap enough for the main comparison

The control-only runs are particularly useful here.

Across the matched measurements:

- The largest control-only cycle count was **0.01196% of an A32 measurement**.
- The largest was **0.01672% of a P32 measurement**.
- For P32’s execution group, the median control/window ratios were approximately **0.00429% for issued µops** and **0.00325% for retired slots**.

Those are small compared with the percentage differences the main experiment is trying to distinguish.

The counter and clock boundaries still differ slightly. The walker waits for the enable acknowledgment before starting its clock, and sends disable after stopping that clock. Consequently, the counters include some boundary activity surrounding the timed walks. That ordering is visible in the [pinned Rust source](https://github.com/lmmx/mibz2/blob/29a0215e3d06cdb436bf212501b01e54c48c7bf0/examples/walker_ladder.rs#L580).

But the controls now quantify that activity. **For cycles and the main execution counters, its magnitude is too small to plausibly explain the principal results.**

This directly weakens the practical argument for replacing the collection backend merely to obtain usable measurements for this diagnostic.

## 3. There is a remaining limitation: some rare counters are dominated by comparable boundary activity

This is where I would retain caution.

For the shipped P32 walker, the summary reports these median per-round control/window ratios:

| Event | Control-only count relative to measured-window count |
|---|---:|
| `cycle_activity.stalls_l3_miss` | **62.47%** |
| `mem_load_retired.l3_miss` | **47.41%** |

Several page-walk measurements in the reduced rungs also have substantial control counts. These cases are explicitly marked as comparable in the walker-ladder-20261102T004502Z-summary.json[archived summary](sandbox:/workspace/scratch/eb4b5d8c4a8d/analysis/walker_20261102T004501Z/walker-ladder-20261102T004502Z-summary.json).

This explains why “the overhead is only 0.017%” cannot be applied indiscriminately to every event. A short control sequence can account for very few cycles overall while producing a substantial fraction of a scarce event.

Those percentages also **do not establish that exactly 62.47% or 47.41% of the workload measurement is removable overhead**. The control and workload are separate executions. Subtracting one from the other would imply a precision we do not have. The current protocol correctly retains both and qualifies their interpretation.

My assessment is therefore:

- **Cycles and the principal execution/MLP measurements:** sufficiently clean for the registered comparisons.
- **Fine claims about L3 misses, L3-miss stalls and certain page-walk counts:** insufficiently isolated in this capture.

An in-process Rust collector might reduce that boundary activity. This run gives us a specific potential benefit to investigate if those rare events become central. It does not demonstrate how much improvement a library would deliver.

## 4. The main performance result is useful—and supports stopping the scheduling investigation

The 32-lane cycle measurements are:

| Rung | Meaning | Median cycles per row |
|---|---|---:|
| A | Counter-terminated chase without the software prefetch | **9.763** |
| B | A with the prefetch | **6.659** |
| M | Reduced shipped-style walk without output stores | **6.293** |
| P | Shipped walker | **6.897** |

The main registered comparison is:

\[
\frac{P32}{M32}
=
\frac{6.89746}{6.29308}
=
1.09604.
\]

So the shipped walker costs **about 9.6% more cycles than the best reduced reference at the same lane count**. That satisfies the registered “within 15%” stopping rule. The clock-based ratio is almost identical, approximately **1.09631**, which provides a useful consistency check.

Another way to express the gap: bringing P all the way down to M’s measured cost would reduce P’s time by about **8.8%**. M omits required output work, so that gap is an optimistic comparison, not an available production optimization.

**I would accept this as evidence against a walker scheduling rewrite on the basis of this diagnostic.** It does not establish an absolute hardware limit or cover every possible layout, but it provides the bounded answer the stopping rule asked for. [Frozen decision rules](https://github.com/lmmx/mibz2/blob/29a0215e3d06cdb436bf212501b01e54c48c7bf0/docs/journal/2026-11-01-walker-ladder-protocol.md#L325).

The A→B contrast is also informative: adding the prefetch reduces the measured cycle cost by approximately **31.8%**. That is a substantial, coherent signal emerging from the repaired measurement setup.

## 5. The counter interpretation still needs the narrower wording we agreed

Successful collection makes the numbers usable; the conclusions must still follow the definitions of the events.

### Issued µops do not establish an execution bottleneck

For P32, the reported issued-µop share is **57.1% of the four-per-cycle reference**, equivalent to approximately **2.285 issued µops per cycle**. The retired-slot share is **54.0%**, and the front-end-undelivered share is **1.23%**.

The registered 70% issue-pressure condition is therefore not met. I agree with the summary’s restrained conclusion: **no bottleneck classification follows from that test**.

In particular, 57.1% should not be read as “42.9% of execution capacity is available for extra work.” The issued event counts transfer into the scheduling machinery; it does not directly measure spare capacity on every execution resource. [Intel’s event definitions](https://github.com/intel/perfmon/blob/main/SKL/events/skylake_core.json).

### The demand-MLP results illustrate why prefetch matters to interpretation

For B, conditional demand MLP rises from approximately **1.354 at 16 lanes** to **1.612 at 32 lanes**—about a **19% increase**. The fill-buffer-full event rate increases by roughly **34%**, rather than doubling. Thus, the registered D4 miss-capacity signature is absent.

That does not rule out memory-related constraints.

There is a useful comparison in the data: A32 has conditional demand MLP around **5.03**, whereas the faster, prefetched B32 has a value around **1.61**. This is consistent with prefetch reducing what demand loads have to wait for. Intel’s definition counts prefetch-allocated buffers in this demand measure only after demand encounters them; it does not count all prefetch activity from allocation onward. [cdrdv2-public.intel.com](https://cdrdv2-public.intel.com/671378/335279-performance-monitoring-events-guide.pdf?utm_source=chatgpt.com)

The low demand-MLP figure therefore cannot be interpreted as only one or two memory requests being outstanding in total.

### The four-core transfer check has a specific scope

D5 passes, with a ratio of ratios of **1.0552**, within its 15% tolerance.

However, D5 compares **P against B**. There are no four-core M measurements in this archive. It therefore supports the stability of the P/B relationship under four-worker contention; it does not directly establish that the P/M gap remains 9.6% there. The implementation makes that scope explicit. [D5 calculation](https://github.com/lmmx/mibz2/blob/29a0215e3d06cdb436bf212501b01e54c48c7bf0/bench/walker_ladder.py#L592).

These are also repeated measurements of one warmed 900,000-row block. They should retain that scope when being applied to end-to-end decompression.

## How this changes my recommendation about perf versus Rust

I would now put it this way:

**Keep the repaired perf backend for interpreting and closing this diagnostic. Defer a Rust-backend migration unless the harness becomes reusable infrastructure or we need cleaner isolation of the rare events that remain problematic.**

The earlier failed probe established defects in our integration. This successful capture shows those defects were repairable, and that the resulting setup can produce coherent, controlled measurements with very small overhead for its main purpose.

A Rust library remains attractive for maintainability and direct control of the measurement window. The evidence now makes that an engineering improvement to weigh against its implementation and validation cost. It is no longer a prerequisite for trusting this capture’s principal cycle-based result.

I would also retain this successful archive as a reference if a migration eventually happens: it gives us concrete event groups, workload checks, expected relationships and validity criteria against which to assess the replacement.
`````
