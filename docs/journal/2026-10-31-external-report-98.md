# External research report 98: amend the P74-C harness (option b) before the i9 run (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-10-31 21:07 UTC.
- **Source:** unattributed in the relay (the guardian's register).
- **In reply to:** the thread-lifecycle race.

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

````text
I recommend option (b): amend the P74-C harness before tonight's i9 run. I would not risk losing an 8,200-child experiment to a now-reproduced thread-lifecycle race.

I also support a narrowly scoped amendment to the walker harness and a fresh exploratory CI validation. But the two experiments need to remain separate, and we must preserve the failed and partial CI records.

There is one important distinction in the Intel results: the unrolled walker currently has a real production-equivalent control regression on the Xeon, not merely an inconclusive statistical bound. The `records` throughput ratio of 0.928 \[0.924, 0.933\] at one worker is a roughly 7.2% slowdown with a tight confidence interval. We should treat that seriously even though the AMD evidence remains incomplete.

### Decision 1: P74-C i9 — fix the harness first

The proposed fix is acceptable in principle, with one safeguard.

A thread disappearing during inspection of `/proc/self/task` should be classified as exited, not as a thread whose CPU affinity is unknown but acceptable.

The runner must distinguish:

* A thread present and alive with an empty or invalid CPU mask: failure.

* A thread confirmed to have exited while the snapshot was being read: not an affinity violation.

* A thread whose status cannot be resolved: failure, rather than silently ignoring it.

The original start-of-thread affinity evidence must remain mandatory. That is the strongest containment evidence because every relevant thread is alive when inspected.

The amendment should be committed before timing, with the original frozen revision retained, and should not change the decoder kernel, selection policy, performance thresholds, corpora or measurement schedule.

Given the already established failure mechanism, a targeted dry-run validation plus exploratory CI integrity checks would be appropriate. We do not need another performance optimization exercise.

### Decision 2: walker — repair the harness, but don't rescue the performance result

The walker experiment is in a different position.

The registered stage-level experiment passed on Zen 3. But the full-decoder exploratory trial has now exposed a potentially important architectural limitation:

| Evidence                                        | Interpretation                               |
| ----------------------------------------------- | -------------------------------------------- |
| Zen 3 stage walk: ~12% less time                | Strong positive result                       |
| Zen 4 stage walk: ~13% less time                | Positive replication                         |
| Xeon decoder Wikidata: approximately unchanged  | No demonstrated decoder benefit on that host |
| Xeon decoder records: 7.2% slower at one worker | Demonstrated regression                      |
| AMD decoder jobs aborted                        | AMD decoder performance remains unresolved   |

I would authorize one replacement exploratory CI run after fixing the lifecycle race, explicitly labelled a second exploratory validation, rather than pretending the first run never happened.

The purpose is to obtain the AMD decoder observations that the first attempt failed to produce—not to retune the walker or wash away the Intel result.

If the AMD decoder improves substantially while Intel regresses, we may eventually have a CPU-specific optimization rather than a universal replacement. That's a potentially useful outcome, although architecture-specific dispatch would be a separate, future hypothesis.

### Send Claude this

I authorize the narrowly scoped harness-lifecycle corrections for both experiments, with P74-C taking priority.

**P74-C v1.2:** amend the frozen harness so an end-snapshot thread confirmed to have exited during /proc inspection is treated as exited, rather than as an affinity escape. Keep the start-of-thread affinity check mandatory and unchanged. Any live thread with an invalid or out-of-allocation CPU mask must still fail; unresolved status must not be silently accepted.

Commit and document the amendment before timing, preserve the original freeze and records, and validate the precise race with targeted tests, a dry run and CI integrity checks. Do not change the P74-C kernel, corpora, selection policy, schedule or thresholds. Give me a new pinned SHA and i9 command only after validation.

**Unrolled walker:** authorize the equivalent harness-only correction and one explicitly recorded replacement three-job exploratory CI run after the fix is validated. Preserve run 41235849514 and its aborted AMD jobs as originally recorded. The completed Intel result, including the significant records regression, must remain part of the evidence. No kernel tuning or statistical threshold changes.

Please prioritize making the P74-C i9 experiment safe to run. Do not delay it unnecessarily for walker work.

Do not start any i9 benchmark, merge production code, enable either optimization by default, or combine P74-C and the walker. Stop and report the validation findings, exact new pins and CI results for review.

### My broader interpretation

This hasn't diminished my enthusiasm for the unrolled walker. It has made the engineering question more interesting.

Astra found and removed a genuine instruction-level inefficiency. That produced a substantial stage improvement on AMD. But the Intel full-decoder result shows why we mustn't equate a successful microarchitectural transformation with a universally beneficial production kernel.

We now have two different optimization candidates:

P74-C is approaching a shipping decision, with consistently positive exploratory decoder throughput and CPU-efficiency measurements.

The unrolled BWT walker has a successful stage-level result but a mixed early decoder picture, potentially dependent on CPU architecture and input type.

I'd prioritize getting the P74-C i9 result tonight. Meanwhile, Claude can finish the walker harness fix and collect the missing AMD decoder evidence.

The immediate priority is experimental reliability, not another optimization.
````
