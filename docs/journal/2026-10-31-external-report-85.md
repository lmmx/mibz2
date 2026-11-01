# External research report 85: endorsing the full-decoder trial; three corrections to its statistics (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-10-31 10:33 UTC.
- **Source:** the researcher.
- **In reply to:** the P74-C full-decoder protocol.

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

````text
researcher:

I endorse proceeding. My only substantive change is to the acceptance statistics, not the engineering plan.
The four-core experiment is now the correct use of engineering time. P74-C already has replicated stage gains; another MTF microbenchmark has diminishing value. Meanwhile, Astra can independently investigate the BWT walk without consuming the P74 experiment's budget.
Three protocol corrections matter:

* New-dump Wikidata should be the primary generalization test. Keep fitted and same-dump held-out results as separate replication checks. Requiring three simultaneous ≥2% lower-confidence-bound gains is unnecessarily likely to reject a useful optimization.
* RSS needs fresh-process measurements. `ru_maxrss` is a process high-water mark; running arms sequentially in one process cannot attribute it correctly. Use separate child processes and paired measurements. Keep +1% as the desired limit, but don't apply ordinary timing confidence intervals blindly.
* Separate failure from inconclusiveness. A measured harmful regression rejects shipping. An upper confidence bound that narrowly misses a threshold means the benefit or safety hasn't been established—not that the implementation is disproved. Preserve a strict one-experiment stop without calling uncertain findings proof of failure.

And mathematically, four-worker performance is not simply the single-core MTF gain multiplied by its stage fraction. The parallel pipeline is constrained by both total CPU work and scanner/worker scheduling. Measuring three workers alongside four is worthwhile.
Here is the instruction I recommend sending Claude:
I authorise the following P74-C work:

1. Integrate the frozen implementation from `8b6caf1` behind an off-by-default research switch on a new branch. Preserve its inclusive policy and the already-measured error-state optimization. No further tuning or master integration.
2. Prepare and preregister one production-equivalent full-decoder experiment, then run exploratory CI and provide the one-command i9 runner for me to execute. Measure 1, 2, 3 and 4 workers with the entire process restricted to four distinct physical cores, leaving SMT siblings idle where possible. Four workers is the primary deployment configuration.
3. Fetch and hash-register the approximately 12 MiB of windows from the separately dated 20261026 Wikidata dump, with positions frozen before fetching. Treat this as the primary external-source validation set; retain fitted and same-dump held-out corpora separately.
4. Freeze acceptance before execution: at least a 2% four-worker throughput gain at the lower 95% confidence bound on the new-dump primary corpus; negative-control regression bounded by 3%; CPU-seconds/GB and peak RSS within +1%, with RSS measured in fresh processes and aggregation defined explicitly. Report the other Wikidata datasets and all worker counts separately. Include same-binary shipped/P74-C/A-A comparisons, exact output and CRC validation, adversarial correctness, and exact-binary assembly auditing.

Distinguish a demonstrated regression, an inconclusive statistical bound and a demonstrated pass. No post-hoc threshold changes, optimization or selective reruns. Stop after publishing the results; do not enable P74-C by default or merge it into master without another decision.
In parallel, prepare an exact production-disassembly evidence package for Astra's read-only analysis of `room_segments::<32, PF_L1>`. No walker modifications or benchmarks are authorised under that task.
My research priority after this: BWT walker, with Astra investigating whether the scalar implementation exposes sufficient memory-level parallelism. The target is approximately 1.5 cycles per decoded byte of walk savings to combine with P74 toward 20% total throughput improvement on the historical Zen 3 cost model.
I would spend Astra's first call on that dependency analysis, not on another P74 error-state or function-size review.

guardian:

I think the researcher has identified the right refinements. I would adopt its statistical changes while retaining Claude's engineering plan. There is no need to restart preparation or reopen P74-C's stage-level investigation.

The most important change is making the October 26 Wikidata dump the primary generalization test. The other two Wikidata datasets remain valuable replications, but requiring all three to independently clear a 2% lower confidence bound is a stricter condition than necessary to establish the principal hypothesis.

### My decisions

| Question                                               | Recommendation                                                 |
| ------------------------------------------------------ | -------------------------------------------------------------- |
| New-dump Wikidata as the primary acceptance corpus     | Yes                                                            |
| Fitted and same-dump held-out Wikidata                 | Report separately; don't make all three independent pass gates |
| Fresh child process for RSS measurement                | Yes, essential                                                 |
| Distinguish statistical uncertainty from observed harm | Yes                                                            |
| Four physical cores, testing 1–4 workers               | Keep                                                           |
| Freeze P74-C and prohibit further tuning               | Keep                                                           |
| Astra's first assignment                               | BWT walker dependency analysis                                 |

There's one statistical refinement I would add: the new-dump result should carry the primary 2% threshold, but the fitted and held-out results should still have clearly defined secondary expectations. Otherwise, a surprising regression on the original Wikidata workload could be dismissed because it wasn't the primary gate.

I would report them alongside the primary result and explicitly flag material deterioration, without inventing new acceptance criteria after measurement.

### The failure-versus-inconclusive distinction matters

Suppose the four-core throughput improvement is estimated at 2.5%, but its lower confidence bound is 1.6%.

That does not establish the registered 2% benefit. But it also doesn't prove P74-C is harmful.

I'd preserve three outcomes:

- Pass: the preregistered benefit and safety requirements are established.
- Inconclusive: the estimates are compatible with success, but uncertainty prevents establishing the required bounds.
- Negative: the measurements establish material regression or an economically insufficient gain under a separately defined criterion.

All three can lead to stopping the one authorized experiment. They simply mean different things scientifically.

### One issue I'd resolve before the protocol freezes

Claude's proposed RSS rule compares the maximum peak RSS across rounds. That's conservative, but one anomalous allocation or system effect can determine the entire result.

Fresh subprocesses solve the attribution problem. I would retain the maximum as a safety diagnostic, while also reporting paired RSS distributions. If the maximum is the formal acceptance criterion, that should be an explicit engineering choice, not mistaken for a statistically stable estimate.

And the CPU rule—point estimate no worse than shipped, upper confidence bound no greater than +1%—is appropriately strict given the objective of improving performance without purchasing that improvement with additional CPU.

## My recommendation to Claude

I agree with the researcher's refinements to the P74-C full-decoder protocol. Please incorporate them before the final freeze.

Make the separately dated 20261026 Wikidata corpus the primary generalization test, with a four-worker throughput lower 95% confidence bound of at least +2%. Retain fitted and same-dump held-out Wikidata as separately reported replications, not additional independent primary pass gates. Explicitly flag any material deterioration on either replication corpus.

Measure peak RSS using fresh child processes for each arm and observation. Predefine the formal RSS aggregation, keep the +1% memory ceiling, and report paired RSS distributions alongside the maximum.

Preserve the four-physical-core process allocation, 1–4 worker scaling curve, CPU-seconds/GB requirements, negative controls, correctness checks, same-binary A/B design, and frozen P74-C policy.

Report pass, inconclusive, and demonstrated-negative outcomes distinctly, without weakening the stopping rule or selectively repeating measurements.

Continue preparing the exact production-disassembly evidence pack for Astra. Its first assignment remains the BWT room walker, with no modifications or benchmarks authorized.

No production-default change, master merge, additional P74 tuning or new walker experiment is authorized.

My overall view: the trial is becoming well designed. The researcher is improving the link between statistical evidence and engineering decisions, while Claude is correctly attending to production-thread topology, reproducibility and machine-code provenance.

We're now measuring the right thing: whether the optimization improves the real four-core decoder economically—not whether it can win another isolated MTF benchmark.

If it passes, I would favor shipping. If it's inconclusive, I would stop and assess the evidence without pretending it disproves the optimization. If it establishes a material regression, I would archive P74-C and return our attention to the BWT walker.

researcher:

Proceed. Claude has the right scope and sequence. Freeze P74-C, measure the complete decoder, and use Astra independently on the BWT walker. No further P74 assembly tuning before the result.

I have two mathematical corrections worth sending before the protocol freezes.

1\. Astra's 1.5 c/B target is revision-dependent.

The two recorded baselines imply different thresholds. For a 20% throughput increase, the required decoder-time saving is \\(T/6\\).

| Baseline         | Total     | Walk     | Required walk saving if P74 saves 0.65 c/B |
| ---------------- | --------- | -------- | ------------------------------------------ |
| Earlier revision | 12.93 c/B | 6.69 c/B | 1.505 c/B (22.5% of walk)                  |
| Later revision   | 10.78 c/B | 4.49 c/B | 1.147 c/B (25.5% of walk)                  |

The 0.65 c/B P74 contribution is hypothetical and must be established on each respective revision. Astra should not reject a 1.2 c/B improvement against the later baseline merely because it misses an inherited 1.5 c/B target.

There is an additional useful diagnostic. With single-chain latency \\(L=38.55\\) cycles and measured walk time \\(C=4.49\\) c/B, a simplified Little's-law model implies effective concurrency \\(L/C\approx8.6\\). Reaching 3.34 c/B requires approximately 11.5 concurrent chains. The code has 32 logical lanes, so the key question is why effective concurrency is much smaller than logical concurrency. These figures are only comparable if the latency and walk measurements share the relevant memory conditions.

2\. Avoid an unnecessarily conservative interpretation of the three Wikidata gates.

Requiring all three datasets to exceed 2% is an intersection–union test: the null hypothesis is that at least one corpus fails to exceed 2%. Testing each at the same one-sided significance level already controls the global false-success probability; Holm adjustment across those three is unnecessary for that specific conjunction. Their correlations do not invalidate this logic.

For CPU/RSS, retain the strict gates, but distinguish failure to establish the limit from a measured regression beyond it. Otherwise numerous small safety comparisons risk creating another inconclusive experiment.

One operational guard: when retrieving byte ranges from the 103.6 GB dump, require HTTP `206 Partial Content`, verify each `Content-Range` and exact byte count, and abort if the server sends the whole object.

### Research decision

The evidence pack should focus Astra on effective memory-level parallelism, not generic instruction reduction. Ask it to trace actual lane-state residency, load issuance, prefetches and instruction-window limitations; assess whether four groups of eight scalar lanes retain 32 independent chains.

The P74-C trial needs no further speculative changes. Its decisive result will be the four-core throughput, CPU-seconds/GB and RSS on the exact frozen decoder.

Keep both projects moving, but allocate the next scarce Astra call to the BWT dependency graph.

guardian:

The researcher is right on both mathematical points. I'd incorporate these corrections into Claude's protocol and Astra evidence pack, without changing the experimental scope or delaying the full-decoder trial.

The most important insight is that our two performance objectives need to be tied to the exact decoder revision being measured.

## 1. Astra's target should be revision-specific

For a 20% throughput increase, total decoder time must fall by one-sixth. Assuming, purely for planning, that P74 saves 0.65 cycles per byte:

| Historical baseline            | Required BWT-walk saving | Percentage of walk |
| ------------------------------ | ------------------------ | ------------------ |
| 12.93 c/B total; 6.69 c/B walk | 1.505 c/B                | 22.5%              |
| 10.78 c/B total; 4.49 c/B walk | 1.147 c/B                | 25.5%              |

This is an important correction to my earlier Astra brief.

Astra shouldn't reject a plausible 1.2 c/B improvement because we supplied an outdated target of 1.5 c/B.

More fundamentally, Astra's job should be to identify the machine-code bottleneck and quantify a credible improvement—not force a predetermined saving from the disassembly.

The effective concurrency estimate of approximately 8.6 chains is a useful clue, too. But it doesn't establish that the CPU has exactly 8.6 outstanding memory requests. It is an approximate latency-to-throughput relationship that includes non-load work and depends on comparable measurement conditions.

## 2. The researcher's statistical correction is valid

Requiring all three Wikidata corpora to independently exceed a 2% improvement is indeed an intersection–union test.

If the claim is that all three exceed the threshold, individual one-sided tests at the selected significance level are sufficient for that conjunction; an additional Holm correction is unnecessary.

However, I still prefer the researcher's earlier recommendation for the actual shipping decision:

Use the new-dump corpus as the primary generalization test, with the fitted and same-dump held-out corpora reported separately.

That asks a cleaner question: does the optimization generalize to previously unseen Wikidata data?

The other datasets remain essential replication evidence, and any meaningful deterioration should be explicitly reported.

For CPU and RSS, the researcher is also correct that failing to establish a tight upper bound isn't automatically evidence of harm. We should retain the acceptance limits but distinguish inconclusive results from measured regressions.

## 3. The HTTP-range guard is worth adding

Fetching 12 MiB of windows from a 103.6 GB dump is sensible, but a server that ignores a `Range` header could accidentally send the entire dump.

Claude should require:

- HTTP status `206 Partial Content`.
- Exact expected `Content-Range`.
- Exact byte length for each window.
- Hash registration after successful retrieval.

This is a practical correctness and resource-safety requirement, not a change to the performance experiment.

## What I'd send Claude

I agree with the researcher's latest corrections. Please incorporate them into the full-decoder protocol and Astra evidence pack before the final freeze.

For Astra, make the BWT-walk performance target revision-specific. The historical 12.93 c/B decoder requires approximately 1.505 c/B of walk savings for 20% higher throughput if P74 saves 0.65 c/B; the later 10.78 c/B baseline requires approximately 1.147 c/B. These are conditional budgets, not demonstrated savings.

Include the effective-concurrency diagnostic L/C ≈ 8.6 for the 38.55/4.49 measurements, explicitly noting that latency and throughput conditions must be comparable. Focus Astra on the dependency graph and why 32 logical lanes produce substantially less effective overlap, rather than generic instruction-count reductions.

For the full-decoder trial, retain the new-dump Wikidata corpus as the primary generalization gate and the other two as separately reported replications. If testing a conjunction across all three corpora, do not apply an unnecessary Holm correction to that intersection–union criterion.

Preserve the CPU and RSS limits, but distinguish failure to establish compliance from a statistically demonstrated harmful regression. Keep the preregistered stopping rule.

For the new-dump HTTP-range fetch, require 206 Partial Content, validate Content-Range and exact byte counts, and abort on unexpected full-object responses.

No additional P74 tuning, walker implementation, production-default change or unrelated performance experiment is authorized. Continue the already approved preparation and stop for review after reporting the frozen protocol and evidence pack.

### My overall assessment

We're now doing the kind of work that makes a serious optimization programme credible: tightening the experimental method without changing the optimization whenever the evidence becomes inconvenient.

The full-decoder trial should proceed. And I increasingly agree that Astra's greatest potential contribution is understanding why the BWT walker exposes far less effective parallelism than its 32 logical traversal lanes suggest.

That is a much more consequential machine-code question than whether P74-C's function could shrink another few hundred bytes.
````
