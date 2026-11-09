# Clock-state shifts: measure the mechanism, retain the experiment

Researcher, 2026-11-09. Inbox [6736096739](https://github.com/lmmx/mibz2/pull/162#issuecomment-6736096739), QUEUED 6735327338. Spec a4f97714cc14366984015869d9d094d89488ac57.

**PROCEED with recorded-only matched-interval telemetry and the unchanged full-data analysis. Do not promote clock strata, CPU time or cycles to a replacement verdict.** D064's distinction between logging, an additional whole-attempt qualification veto, and deletion/replacement is correct. No numeric veto is proposed or approved here.

## Evidence boundary

Read the assignment, original [#270 result at 94a205c](https://github.com/lmmx/mibz2/blob/94a205c34d35205ab1e4d72c92deb15945265cb4/docs/journal/2026-11-09-run-fed-leader-workstation-result.md), source for #290 at merge **637ad34b75169587f25a5666a05d0f85479b9688**, and [D064](https://github.com/lmmx/mibz2/pull/145#issuecomment-6737091245). The channel reports #297 corrected the causal overclaim beside the original. I do not reopen that verdict.

The published observations are 41/140 slow records, primary wall .9681 [.8535,1.0981], CPU .9698 [.8459,1.1119], and control fidelity FAILED. I did not independently unpack the evidence bundle or recount the 41 records. Original VALID / INCONCLUSIVE stands. No native timing or host configuration changes were performed.

## 1. What could have been robust?

Let a serial on-CPU path have compute demand N cycles, actual frequency f and on-CPU memory-stall duration M. An illustrative model is

    T_cpu = N/f + M; actual cycles C = N + f M.

For compute-only work, halving f doubles CPU time and wall while C stays fixed. Doubling N at fixed f also doubles both times, but doubles C. Thus CPU time doubling excludes *pure off-CPU delay as the sole explanation*, not extra work, spinning or on-CPU stalls. No clock was recorded in #270; its cause remains unidentifiable.

Cycles help distinguish these hypotheses but are not universally robust. With N=4,M=1, f=4 gives T=2,C=8; f=2 gives T=3,C=6. **The slower execution has fewer actual cycles.** Memory latency in seconds, contention, uncore clocks, synchronisation and changed scheduling prevent an invariant cycle budget. Reference cycles or invariant TSC ticks are different again: a constant-rate clock counts more ticks when execution takes longer; it does not remove the slowdown.

For identical frequency factors in both members, a ratio may cancel the factor under the compute-only model. If candidate demand is .9 of reference but only candidate is halved in frequency, its time ratio is 1.8; reversing the mismatch gives .45. The geometric mean of perfectly balanced mismatches is .9, but an individual pair is badly contaminated. Equal marginal frequencies across arms do not guarantee matched pairs, independence or equal sensitivity. Existing CPU ratios are not a rescue because CPU seconds also scale with execution rate.

A useful joint diagnostic is instructions/byte, actual cycles/byte, CPI, task-clock, wall, task-clock/wall and measured frequency. Rough patterns, not proofs:
- stable instructions and cycles, larger task-clock, lower measured frequency: supports reduced execution rate;
- more instructions and cycles: extra work/spinning is plausible;
- stable instructions, larger CPI: stalls/contention plausible;
- wall rises while task-clock is stable: off-CPU delay or reduced overlap plausible.

Use the same task/mode/interval coverage. Sum-of-thread CPU work is not critical-path wall latency. Aggregate cycles/task-clock weights each thread by its active time; it can hide a slow critical worker behind fast noncritical helpers.

## 2. Instrument choice and prior art

**Best first combination: task-scoped PMU diagnostics over the decode, plus configuration and thermal metadata.** Hardware feedback sampled over the relevant interval is stronger frequency evidence than before/after sysfs snapshots; neither alone proves a thermal cause.

- [Linux CPUFreq documentation](https://cdn.kernel.org/doc/html/latest/admin-guide/pm/cpufreq.html) explains that scaling_cur_freq often reflects the last requested P-state, and even hardware-assisted reporting need not equal instantaneous frequency. Keep per-policy/CPU identity, timestamps and availability. A mean over allowed idle CPUs is not workload frequency. Before/after package temperatures can miss a transient; equal readings do not prove thermal equivalence.
- [Linux turbostat documentation](https://github.com/torvalds/linux/blob/master/tools/power/x86/turbostat/turbostat.8) distinguishes busy frequency, average frequency including idle, and TSC rate. Its APERF/MPERF-based busy-frequency estimate uses counter deltas. It also warns that collection is non-atomic and very short intervals can be inconsistent. This matters for these roughly 0.15-second runs: a broad package sample can dilute the event. CPU-scoped counters include unrelated activity; record affinity, migration and coverage. APERF/MPERF is useful corroboration, not automatically a task-attributed measure.
- [perf_event_open documentation](https://man7.org/linux/man-pages/man2/perf_event_open.2.html) distinguishes actual and reference cycles, task clock, event scope, inheritance and multiplexing. Preserve raw counts and enabled/running durations. Missing counters are unknown, not zero. Independent counters may not cover identical intervals. Scaled multiplexed totals depend on representativeness; they are weak evidence for brief state shifts. Validate all contributing threads and privilege-mode coverage before labelling a quotient hardware frequency.
- [Google Benchmark random interleaving](https://google.github.io/benchmark/random_interleaving.html) provides established precedent for distributing repetitions across benchmarks to reduce temporal variance. The applicable principle is temporal balance, not its reported improvement magnitude or automatic transfer of its repetition scheme.

**Interleaving beats retrospective filtering, but does not erase carryover.** Freeze an arm order/seed before execution and record predecessor, position and elapsed gap. Within the existing fixed round population, balanced positions reduce confounding with drift. With a shared reference and seven arms, not every comparison can be adjacent without additional reference executions; adding those changes the repeat policy and needs C6 approval. Do not casually substitute ABBA and double the samples. Diagnose arm-by-position and predecessor effects descriptively. A hot arm can slow the next arm, so random order alone is not a thermal reset.

**Cooldown is not a free fix.** A fixed, identical preparation protocol is reproducible, but cool-start performance is a different operating condition from sustained throughput. “Wait until stable” may accept a stable low clock or converge differently after different arms. Use one bounded preflight with archived refusal, not a wait/retry loop conditioned on observed performance. Governor writes, turbo changes, privileged access or new infrastructure are not authorised here.

## 3. Concrete #290 limitations before any gate uses it

Inspected [examples/ab.rs at 637ad34](https://github.com/lmmx/mibz2/blob/637ad34b75169587f25a5666a05d0f85479b9688/examples/ab.rs), machine module, especially open/each/finish (~1887–2044). This is a targeted source review, not full B5 or real-hardware validation.

- It opens cycles, instructions and task-clock separately, with inheritance. Enabling/disabling is sequential around the wall region. Therefore “exactly the same interval” is an approximation with boundary skew, not simultaneous PMU capture.
- **Perf::each ignores ioctl return values.** Record enable/disable failures explicitly and invalidate the affected telemetry. A successful open/read does not certify successful interval control. This is a diagnostic reliability fix before using these fields for any qualification gate; it does not invalidate old runs without the opt-in flag.
- eff_ghz rejects multiplexing of cycles or task-clock, but its predicate does not check the instruction counter. Thus the PR description “null if any counter was multiplexed” is stronger than the code. A multiplexed instruction event need not invalidate C/task-clock, but does invalidate an unqualified joint CPI/work interpretation. Report per-counter validity and correct the claim.
- Store raw enabled/running values per counter, not only the minimum fraction rounded to three decimals; otherwise the report loses coverage evidence. Preserve event identity, scope and availability reasons.
- A user-only hardware numerator needs demonstrated compatible task-clock semantics on the target kernel. Passing the same flags is not itself that demonstration. Validate user+kernel and fallback modes, pre-existing versus newly created threads, and boundary coverage. Software-event stand-ins do not establish real PMU behaviour.
- “settled” is a stability predicate, not a normal-clock predicate. Its existence in the tool does not authorise adding adaptive waits to a frozen recipe.

Also avoid circular correction: if f_hat=C/task_clock, then C/f_hat equals task_clock by definition. That is not an independent frequency-normalised estimate of useful work. Wall multiplied by aggregate f_hat likewise has no general multicore critical-path interpretation.

## 4. A registration rule that cannot become retry-to-pass

Recommended next registration text, for Coordinator assessment and freeze:

1. Preserve the original workload/grid, fixed repeats, pair construction, estimand, multiplicity and acceptance rules. Archive every planned observation and the full-data result. Clock fields affect no judged ratio, confidence interval, pair membership or stopping condition.
2. Add a frozen telemetry schema with source, scope, units, timestamps, raw counter coverage and missing/error reasons. Preflight checks capability and formatting; it does not infer a good result from a fast warm-up. If essential diagnostics cannot be collected, archive refusal/diagnostic incompleteness with the exact reason.
3. One bounded preflight and one fixed-population attempt after the approved freeze; no replacement pairs, automatic extra rounds or alternative seed chosen after inspection. A failed diagnostic objective returns to review, not to an automatic new run. A new commit or recipe name is not a new justification.
4. Report a chronological table of all arms and rounds, their clock/thermal fields and pair log ratios. Report unknown/mixed states explicitly. Clock-stratum tables may be exploratory diagnostics, with counts and missingness, never alternative qualification results.
5. Do not add a numeric quality veto in this registration without a separate concrete specification and review. If later adopted as narrowing, calculate/archive the original result unchanged and record an additional whole-attempt refusal to qualify. A veto does not turn the remaining subset into evidence and does not grant a repeat.
6. Every actual workstation attempt remains Louis's C5 execution. Deletion, replacement, new repeated references, frequency-normalised judged estimands or changed multiplicity require explicit C6 approval.

This agrees with D064 but adds a statistical reason against even predeclared “healthy-clock-only” qualification. Clock during execution can be a *consequence of the arm*: power-hungry code lowers its clock and loses elapsed performance. Conditioning on clock removes part of the treatment effect. In the witness, all-pair time ratios [.9,.9,1.8,1.8] have geometric mean 1.273 (harm); retaining only the high-clock pairs gives .9 (benefit). Pre-registration makes selection transparent, not causally harmless.

Absolute thresholds independent of outcome do not solve that mediation problem. Relative-to-run-median thresholds additionally cannot detect an entire attempt at low clock. Pre-run state strata reduce direct conditioning on the current arm, but predecessor thermal carryover and missing overlap remain. For diagnosis, keep both members' states (HH,HL,LH,LL,unknown), not one label for the whole pair. Do not fit a post-hoc clock regression and call its adjusted coefficient the original paired estimand.

Temporal clustering also warrants showing the sequence and serial dependence of pair log ratios. No revised degrees of freedom or bootstrap scheme is adopted here; if a new inferential method is needed, register it under the relevant authority rather than silently replacing the original interval.

## 5. Expected value

Expected: paired runs would control shared machine state. Observed: the published broad intervals and failed fidelity control leave the i9 question unanswered; frequency was not measured. No measured performance gap is closed here.

Learned: wall and CPU together narrow the possibilities but do not identify clock; actual cycles discriminate only with a work/stall model; sampling scope and treatment-induced clock changes matter; #290 contains concrete telemetry-contract gaps before any use as a gate.

Highest-value next work is untimed error/coverage validation of the recorder, then a separately frozen recorded-only diagnostic attempt under C5. It separates frequency changes, extra work and on-CPU stalls while retaining the full population. Another filtered reanalysis or blind rerun has less value. The included algebraic witness script ran successfully; it does not model #270's actual traces or qualify a decoder change. Existing frozen results and queued assignments remain untouched.

Routing audit: only five already-resolved historical exceptions; REVIEW routes to #145.
