# Run-fed workstation revision R10: REVISE before freeze

Assignment [6745615244](https://github.com/lmmx/mibz2/pull/162#issuecomment-6745615244), incorporating QUEUED 6744625942. Reviewed head **5602dee5628d26e3142ab6a050b2e2ef5e7c53ae**. Pinned instructions: docs/team/{SPEC.md,SPEC.RESEARCHER.md,MODE.md,RULES.md,PROTOCOL.md} at a4f97714cc14366984015869d9d094d89488ac57.

**REVISE the preflight evidence and the declared acquisition procedure before freeze.** The full-data analysis is supported; no change to the primary, reference, thresholds or all-pairs rule is requested.

## Evidence

Fetched the registration, admission, checker tests, recipe tests, recipe, CI reporter and dependencies, and examples/ab.rs at the reviewed head. All **27 workstation admission tests pass locally**, including the 41/140 doubled-run full-data calculation and invariance to missing or contradictory snapshots. Head-associated CI shows **24 successful checks, 3 skipped**. The submit job logs 27 admission tests and 9 recipe tests passing on synthetic merge c6f2049 (merging the reviewed head into e7885568). I inspected those logs; I did not independently execute the full recipe, native decoder, or any timing.

The attached independent probe calls the actual preflight function with the repository's positive fixture and single-field mutations. Positive control: PASS. Missing cpu_quota, float CPU/core counts, integer turbo=1, and twenty invented CPU identifiers: **all PASS**. These are admission witnesses, not observed workstation states.

## Blocking technical findings

### 1. “No quota established” is not “no quota”

preflight_problems uses env.get("cpu_quota") is not None to reject quotas. Removing the key passes. More consequentially, ab::env serializes both an unreadable quota and an explicitly unlimited quota as null. It reads fixed /sys/fs/cgroup/cpu.max or v1 files, rather than resolving the process's effective ancestor hierarchy. A limited child with an unlimited root can therefore satisfy this check, even without malformed JSON.

The declaration is “20 CPUs allowed, 10 physical cores and no cgroup CPU quota”, not “legacy scalar absent”. Require an explicit established unlimited state, with complete effective hierarchy provenance, or narrow the scientific claim and registration prospectively. The recent service-target result does not repair this recipe's collector or bind its later run. Reuse the already hardened discovery representation rather than inventing another nullable quota encoding. Missing/unknown evidence must refuse; explicit unlimited must pass.

Also require exact integer/bool types and bind cpu_khz keys to the actual allowed CPU IDs. These latter mutations are secondary hardening, but the current “every fact missing or unreadable refuses” claim is false. A size-20 dictionary is not evidence that the twenty allowed CPUs were read.

### 2. The preflight is not just passive sysfs reading

In the reviewed examples/ab.rs, env():
- spawns 2 * ncpu threads, each spinning for roughly 300 ms;
- calls machine::extend_env, which calls Perf::probe, which calls Perf::open.

The recipe calls ab env twice: once outside any timeout for W/env.json and again inside the preflight's timeout. Thus the claims “no counter is opened” and a purely passive preflight are not properties of this command path. At paranoid=4 the opens may fail; they are still attempted. With different permissions they may succeed. The spin changes recent load and potentially clock/thermal state immediately before the samples.

Choose and freeze the actual procedure: preferably a passive facts-only collector with no spin or perf probe. Alternatively explicitly register the active env probe and its possible conditioning effect; do not call it passive or use its samples to rule that effect out. Bound the initial env collection too. timeout 30 without a kill-after bounds TERM delivery, not forced termination of a TERM-resistant process; add bounded escalation if retaining the wall-bound claim. No adaptive settling, thresholds or retries are needed.

These findings concern the pre-run acquisition, not whether --sysfs-state itself opens counters during ab run. The flag's parent-only path can remain as designed.

## Answers to the questions

**Q1 — second attempt alone.** Scientifically reasonable as a separately identified, prospectively fixed follow-up with its changed harness, first result alongside, no pooling and no third attempt implied. It is not a restoration of a campaign-wide 95% false-positive guarantee: selecting another attempt after an inconclusive first is a sequential policy, even when each interval retains df=19. State the interval as this attempt's registered interval and disclose the sequence.

Authority is separate. My authenticated pin still reserves repeat policy in C6 and says a repository assertion cannot widen it. The dispatch's claim that amended C6 delegates Q1 is not a re-pin. I can give this scientific assessment, but cannot certify Q1 authority from it. The draft itself still says PENDING and forbids freezing as decision evidence. Resolve that recorded restriction and the applicable authenticated policy before freeze; this is not a demand for a new decision if Louis has already made the relevant one. It does not block repairing the technical findings.

**Q2 — keep endpoint labels, narrowly.** Report H/L/unknown as the declared post-exit snapshot count, with raw readings and read duration/delay retained. Do not relabel valid observations unknown merely because they are weak proxies. H is not “ten active physical cores were fast”; twenty entries are logical CPUs and may include siblings, idle CPUs and CPUs unrelated to the decode. Do not add a child sampler solely to improve this descriptive label. No state-conditioned ratios, exclusions or causal clock attribution.

**Q3 — exact settings.** Keep the exact declared driver/governor/EPP/turbo settings for this attempt. “Readable and unchanged at the endpoints” allows different treatments and cannot prove stability between them. Record subsequent changes descriptively; do not retrospectively veto the result on a new stability rule. Require the evidence that actually establishes the declaration, including policy/CPU coverage rather than a misleading aggregate.

**Q4 — completeness is descriptive.** Yes. A valid timing result remains the full-data result even if later snapshots are mostly missing. Separately mark the clock-diagnostic objective unmet. A preflight readability refusal is the prospectively declared pre-run gate; it is not permission to discard an already measured attempt for missing diagnostics.

**Q5 — add descriptive lag-1 correlation.** Useful, but not a substitute for the chronology. For each arm's twenty ordered log ratios x_r, report sum[(x_r-mean)(x_(r-1)-mean)] / sum[(x_r-mean)^2], r=1..19 in the numerator; constant series -> undefined, not zero. Name the convention; also retain the ordered values. No p-value, effective-sample-size adjustment, interval replacement or rerun rule follows from this optional diagnostic. At n=20 it is noisy; shared references and rotation can induce structure. Registered t intervals remain the registered calculation, not proof of independent rounds.

**Q6 — after build and corpus.** Keep the preflight closest to timing. Moving it earlier would not remove build heat from timing and would make its observations staler. Remove or disclose the active env probe first. Do not wait until a favourable frequency/temperature appears.

## Mechanism and harness caveat

R10.2 overstates the mechanism: unpinned nine team threads are not guaranteed nine distinct physical cores, and reduced wall does not require leader MTF alone to be the critical path. The construction changes helper work, memory traffic and overlap; it can improve a helper-bound run too. The captures establish completion order, not the counterfactual wall recoverable by removing MTF.

Likewise fidelity need not be “near CI's” variance specifically. For a ctl ratio centred at 1, fitting the t19 interval inside [0.98,1.02] requires log-ratio sample SD at most log(1.02)*sqrt(20)/2.093 ≈ 0.04231; away from 1 the allowable SD is smaller. The actual criterion is the interval, not resemblance to CI.

Parent reads outside the child's clocks still alter gaps and machine state; common placement is not equal effect across arms. Keep the first and second attempts distinct, and make no causal attribution of any improvement to “fixing frequency”. An additional timing experiment to estimate telemetry cost is not justified before this bounded follow-up; disclose the limitation and preserve matched within-attempt comparisons.

## C3 and disposition

Expected: repeat the unchanged comparison with honest diagnostics, preserving every pair. Observed: full-data invariance and workload/binary checks pass their tests; the new preflight still admits absent/ambiguous capacity evidence and understates its own active work. The analysis gap closes; acquisition and registration readiness do not.

Learned: endpoint observability and controlled conditions are separate, and metadata commands can actively condition the system. Next highest-value work is a small collector/admission correction with positive and unknown-state tests, then this one follow-up once the repeat-policy restriction is resolved. More shape enumeration or a new telemetry timing grid is lower value. No implementation, merge, workstation execution or authority expansion performed.

Migration gate verified before receipt: old branch 1283b969e9e295e268257cd1b6be3ad7da29e6dd; manifest SHA-256 9b6bfe5b8e2193a68d8e788873da11947197c3457b6ad6e454b580a05f7b8fd0. Routing audit: five resolved historical incidents, zero unresolved.
