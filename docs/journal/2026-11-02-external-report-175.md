# External research report 175: the researcher's review of the compact30 traversal harness (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02.
- **Source:** the independent researcher.
- **In reply to:** the traversal harness and analysis at `b12b49b` and the recipe at
  `65f7169` (PR #134).

**Evidence cutoff:** 2 November 2026. A reading of the pinned source, and synthetic Python
records run through the unchanged analyser. No decoder, benchmark, counter or workstation
change.

**Its attachment** is kept unchanged in `2026-11-02-external-report-175/`: the review
bundle (`review-evidence.zip`) and its unpacked files (`compact30_protocol_review/`).
- The bundle's copies of `bench/c30_traversal.py` and `bench/test_c30_traversal.py` match
  the Git blobs at `b12b49b` (`3c560a9…`, `a0fb65c…`).
- Its script, run here unchanged, reproduces its `gate-review-results.json` byte for byte:
  the 14 original tests pass, and all five malformed cases are accepted as valid with
  ADVANCE.

**It asks:** hold the i9 capture; keep the walker unchanged; repair the harness's
completion ordering, the CPU-cost measure and the record validation.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

`````text
**I would hold the i9 run for one more narrowly scoped harness repair.** The revised investment rule and workload allocation are reasonable, but the actual code has a measurement-boundary problem: **a worker that finishes early validates its output while other workers are still inside their timed traversals.**

I also found that the reported “worker cost” is summed **elapsed time, not CPU time**, and independently reproduced several cases where the analyser accepts records that violate the registered experiment.

These findings concern the **measurement harness**, not compact30’s walker. They do not reopen the architecture review or justify another implementation variant.

I reviewed the harness and analysis at **`b12b49b44e27b96c8e16e0ccab4e348c1ba56bba`**, and the recipe at **`65f7169`**. I ran only Python tests on synthetic records—no decoder, benchmark, hardware counters or workstation changes.

## 1. Required repair: validation must wait until every timed traversal has finished

The current execution order is:

```text
Worker::run:
    prepare table
    start barrier
    record t0
    traverse and assemble
    record t1
    compare output against expected output
    return

Caller:
    append measurement record
    completion barrier
```

The equality assertion is inside `Worker::run`, after that worker’s timer stops. The completion barrier is outside it, after validation and a `Vec::push`. This ordering is explicit in the source.  

That excludes validation from **the finishing worker’s timer**, but not from **its neighbours’ measurement environment**.

For example:

```text
Worker A:  [timed traversal] [reads output + expected bytes to validate]
Worker B:  [---------------- timed traversal ----------------------]
```

On a full block, the comparison operates on two roughly 900,000-byte arrays. That is additional memory activity precisely while we are trying to assess traversal under concurrent memory-system pressure.

The effect need not cancel between arms. Different completion patterns determine when validation starts, how many validators overlap remaining walkers, and which arm is exposed to that traffic. We do not know its magnitude or direction without measuring it—and should not introduce that uncertainty into this experiment.

### The minimal correction

The ordering should be:

```text
prepare
start barrier
record start
traverse and assemble
record end
completion barrier
validate output
record results
proceed to the next preparation
```

**No output comparison, result-vector growth or other substantial postprocessing should happen before the completion barrier.** Preallocating the result vector is sensible too, but does not replace the barrier correction.

The next step’s start barrier already prevents its traversal from beginning before every participating worker has completed its preceding validation and next preparation. This does not require redesigning the walker.

Add a measurement-path check establishing, for each step:

\[
\min(\text{validation starts})
\;\geq\;
\max(\text{traversal ends}).
\]

An intentionally uneven synthetic execution schedule is the relevant regression test: one worker finishes much earlier than another, but cannot begin validation early.

### The failure path also needs checking

At present, an output assertion can panic before that worker reaches the completion barrier. The remaining workers can then wait indefinitely for a participant that has exited. A pinning failure can create a similar problem before the first barrier. This follows from the assertions and fixed four-party barrier in the current harness.  

The harness should propagate a shared failure coherently or terminate the process promptly, preserving evidence. An injected mismatch should produce a recorded failed run—not a hang.

**This is the clearest blocker. Moving a timer boundary is not sufficient when untimed work can interfere with another thread’s timed work.**

## 2. Resolve the metric mismatch: elapsed worker time is not CPU cost

Claude’s statement about “recording the CPU per walk” refers to this function:

```text
cpu_now() → sched_getcpu()
```

That records **which CPU the thread is executing on**, not how much CPU time it consumed.

The traversal timestamps come from `std::time::Instant`, and the analyser calculates:

\[
W_4^{\text{elapsed}}
=
\frac{\sum_i(\text{end}_i-\text{start}_i)}
{\sum_i n_i}.
\]

There is no per-thread CPU-duration field in the reviewed records.  

Rust documents that `Instant` uses a monotonic elapsed-time clock on Unix. Linux separately provides `CLOCK_THREAD_CPUTIME_ID` for CPU time consumed by the calling thread. [doc.rust-lang.org](https://doc.rust-lang.org/stable/std/time/struct.Instant.html?utm_source=chatgpt.com)

### The existing metric is useful—but its interpretation needs discipline

The guardian correctly called summed traversal durations an **approximation** of worker CPU cost. For continuously executing threads, each pinned to a separate otherwise-idle core, it may be a good approximation.

But pinning does not establish uninterrupted execution. A descheduled worker’s elapsed timer continues advancing. The current gates check placement, start synchronization and variability; they do not establish that elapsed worker time closely tracks CPU time.

That matters because the decision subsequently treats the ratio as an input to a model denominated in **CPU-seconds per decoded GB**, and describes a passing candidate as saving CPU. 

### My preferred resolution

Record both:

| Quantity | Purpose |
|---|---|
| Per-thread CPU duration around each traversal | Aggregate worker CPU-cost ratio and constructor budget |
| Existing monotonic start/end timestamps | Completion-time ratio, start skew and overlap |

This needs no PMU counters or sudo. Clock-reading overhead should be handled consistently and assessed separately, especially for the tiny tail blocks; it should not be silently subtracted or assumed zero. [man7.org](https://www.man7.org/linux/man-pages/man2/clock_gettime.2.html?utm_source=chatgpt.com)

A lower-instrumentation alternative is to retain summed elapsed time **explicitly as a proxy**, with its uninterrupted-execution assumption added to the planning model and claims. That would be a deliberate weaker evidential choice.

**My recommendation is to measure thread CPU time directly.** The experiment is supposed to inform CPU-cost economics, and we already want a separate elapsed-time measure. They should not be given different names while being derived from the same clock.

## 3. Required repair: the analyser must validate the executed assignment, not only its metadata

I reproduced all **14 existing tests** successfully. I then exercised the unchanged analyser with additional synthetic records.

The local copies of the analyser and its existing tests match their repository Git blob hashes exactly. These checks did not execute the Rust harness.

The additional results were:

| Synthetic violation | Current analyser’s response |
|---|---|
| Only **one measured pair**, instead of the registered 20 | **Valid; ADVANCE**, with a zero-width interval |
| Every W4 block attributed to a different worker than its declared assignment | **Valid; ADVANCE** |
| Candidate W4 row counts doubled, while block metadata remains unchanged | **Valid; ADVANCE** |
| One expected four-worker full step split into a three-worker step and a singleton | **Valid; ADVANCE** |
| A next-step preparation placed inside a preceding traversal window | **Valid; ADVANCE** |

The gate-review-results.json[synthetic check results](sandbox:/mnt/data/compact30_protocol_review/gate-review-results.json) contain the complete outputs.

### The row-count case demonstrates the practical risk

The uncorrupted synthetic example has a candidate/control worker-cost ratio of **0.95** and receives **STOP**.

Changing only the candidate walk records’ `rows` fields—doubling them without changing the block metadata or durations—changes the computed ratio to **0.475** and produces **ADVANCE**.

This is not evidence that the current Rust code writes incorrect row counts. It demonstrates that the purported validity layer does not detect an incorrect denominator.

### Why these cases pass

The checks are individually sensible, but not sufficiently connected:

- G0 checks `(round, mode, arm, block)` completeness.
- G4 checks the assignment lists in the metadata.
- The CPU check checks the CPU against the recorded worker.
- The lockstep check only rejects duplicate worker IDs within a recorded step.
- None of those establishes that a particular walk’s **block, worker, step and row count agree with the declared assignment**.
- The analyser takes the number of rounds from metadata rather than enforcing the registered capture length. 

The current Rust assignment itself is deterministic and consistent with the described snake allocation. The problem is not that I found it assigning blocks incorrectly; it is that the analysis would fail to catch a subsequent recording or synchronization error. 

### The narrow correction

Construct the expected record structure from the frozen assignment and validate against it before calculating ratios:

\[
(\text{round},\text{mode},\text{arm},\text{worker},\text{step},\text{block},\text{rows}).
\]

That should establish the exact expected participants in each step, permit only the declared incomplete final step, check every row count against `block_rows`, and enforce the registered number of measured rounds. Also check cross-step preparation/traversal ordering and the new validation boundary.

The single-pair case should never receive an experimental decision. Returning a point estimate as both confidence bounds is not an adequate fallback for an insufficient sample.

**These are small, testable validation repairs—not a request for a different statistical framework or more workstation measurements.**

## 4. The revised design choices are otherwise defensible

I would retain several of Claude’s decisions.

### The one-block allocation tolerance is appropriate

The original 1.01 balance requirement was incompatible with dividing 230 large blocks among four workers. Replacing it before timing with a granularity bound of one block is legitimate.

The revised protocol records the assignment, per-worker rows, full-block counts and RAW-group distribution. Its primary-workload totals are consistent with the unavoidable extra large block on two workers. 

No need to split blocks artificially or introduce another workload-allocation sweep.

### Lockstep is a valid controlled condition

Preparing tables, synchronizing starts and preventing the next preparation from overlapping traversal creates an interpretable **lockstep traversal workload**.

It does not reproduce the full decoder’s mixture of scanning, construction, traversal and output work. The protocol now says that explicitly. The control-build versus candidate-copy cache-state asymmetry is also correctly retained as a limitation.  

I would accept that boundary for a first investment screen, once post-traversal validation is separated properly.

### Uneven finishes should remain a result, not automatically invalidate the run

Claude is right not to require all walks to finish together. An arm that creates more variability among comparable blocks may genuinely have worse completion behaviour.

However, synchronized starts do not mean sustained four-way traversal throughout every window. Faster workers finish; the remaining ones then run with fewer active neighbours. The mixed large/short step and final partial step are further explicit exceptions.

Keep the overlap statistics descriptive. Do not describe every measured duration as occurring continuously beside three other active walks.

### The 2% completion-time tolerance is a policy choice, not a correctness issue

The proposed rule is declared before timing, and its handling of conclusive harm and inconclusive intervals is explicit. That is acceptable.

The wording should acknowledge what it permits: **up to 2% more completion time within the chosen tolerance**. It should not be paraphrased as proof that the candidate makes no speed-versus-CPU trade-off at all. 

## 5. Two small interpretation corrections belong in the final protocol

### `e4` is a sum of traversal-window makespans

The actual calculation is:

\[
E_4
=
\sum_s
\left[
\max_k(\text{end}_{s,k})
-
\min_k(\text{start}_{s,k})
\right].
\]

That is useful. But the harness does not record the barrier’s release timestamp itself, and the sum omits preparation, validation and gaps between steps. 

Call it **summed lockstep traversal completion time**, measured from each step’s earliest recorded traversal start. It is not the wall-clock duration of the whole capture or full decoder.

This is a naming and scope correction; it does not require a more elaborate barrier implementation.

### The `0.865` screen is slightly rounded

The exact unchanged-constructor threshold in the stated model is:

\[
1-\frac{0.25}{1.847}
=
0.8646453709.
\]

At the rounded threshold:

\[
b_{\max}(0.865)=0.9988935811,
\]

not quite 1.

The difference is tiny relative to the model’s approximate coefficients. Keeping **0.865 as the rounded investment screen** is reasonable. But remove the literal implication that every accepted upper bound guarantees \(b_{\max}\geq1\); continue calculating the actual budget from the observed endpoint.

This does not change the guardian’s main distinction: STOP is an investment decision under the selected policy, not a proof that a faster constructor is impossible.

## 6. What I would send back to Claude

> **Hold the i9 capture. The compact30 walker remains accepted and should stay unchanged. The remaining repairs are confined to the harness and analysis.**
>
> In `Worker::run`, output validation currently precedes the W4 completion barrier, and record-vector insertion also happens before that barrier. Move validation and record allocation after all workers have ended their timed traversals. Add an explicit check that no validation window overlaps any traversal in that step. Preserve the start barrier before the next traversal.
>
> Resolve the distinction between elapsed worker time and thread CPU time. The current `cpu_now()` records CPU identity, while both cost metrics use `Instant` durations. Prefer recording per-thread CPU durations for the constructor-budget ratio and retaining wall timestamps for completion time and synchronization. Document and check clock overhead. No PMU collection is required.
>
> Strengthen validation against the registered assignment: enforce 20 measured pairs, exact expected worker/step/block membership, correct row denominators, declared partial-step participation, and cross-step ordering. Add the supplied synthetic counterexamples as rejection tests. Test an injected correctness failure and pinning failure so they terminate cleanly rather than stranding peers at a barrier.
>
> Retain the revised investment rule, deterministic row-balanced allocation, completion-time tolerance and explicit cache-state limitation. Describe `e4` by its actual earliest-start-to-last-end formula and treat 0.865 as rounded. Return a new pinned harness, passing regression tests and an untimed dry-run validation before capture approval. No walker changes, tuning arms or constructor work.

## Bottom line

**The experiment’s question and basic design are now sound enough to pursue. The current measurement implementation is not yet ready to freeze.**

The critical fix is to separate validation from **all workers’ timed windows**, not merely from each worker’s own timer. The CPU metric and record-validation issues should be resolved in the same bounded harness revision.

The review-evidence.zip[review evidence bundle](sandbox:/mnt/data/compact30_protocol_review/review-evidence.zip) contains the hash-verified analyser and original tests, the additional synthetic checks, their results and a short review note.

After these corrections, I would support freezing the comparison around the **unchanged, reviewed 21-instruction walker**. No further architectural investigation is needed before that decision.
`````
