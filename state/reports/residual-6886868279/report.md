# Residual decomposition: one-round review

**REVISE — one targeted correction: separate the intervention verdict from the counter ledger.** Gate R2/R3 on a valid, authorised Arm B alone; keep Arm I coverage, phase shares and W1's proxy descriptive. Do not use a coverage threshold or L2 evictions to infer physical-traffic ownership.

Assignment [6886868279](https://github.com/lmmx/mibz2/pull/162#issuecomment-6886868279); draft [6886866992](https://github.com/lmmx/mibz2/pull/145#issuecomment-6886866992); preceding result [6886389292](https://github.com/lmmx/mibz2/pull/145#issuecomment-6886389292). Spec pin a2da49638ac54d7763de1c167bd054f67ffdc763. No data collection, timing, MSR access, implementation or adoption approval.

## Replacement decision structure

1. Valid authorised B: apply the existing R2 and R3 numeric thresholds to paired off/on whole-decoder measurements at four workers, regardless of Arm I's coverage.
2. R2: label **WORKER-PREFETCH-OFF BENEFICIAL UNDER THE REGISTERED TOLERANCE**, not proof that the counted prefetch traffic was unused.
3. R3: label **WORKER PREFETCH BENEFICIAL TO WALL TIME**, not proof that the residual consists of useful early demand or that induction residency is the cause.
4. Neither: **INCONCLUSIVE/MIXED**. B absent or invalid: **NO INTERVENTION VERDICT**, not MIXED and not W1-based targeting.
5. Arm I: report coverage and residual, phase event counts, limitations and write proxy alongside the intervention result. They can rank hypotheses; they do not compel the induction-cap study or qualify a production change.

This is one correction to the estimand/decision architecture. The numerical B cutoffs can remain as pre-data diagnostic priorities; no replacement significance programme is required.

## READ: R1 is not an identification theorem

The previous 3.73% is the recorded demand-counter/IMC ratio, not a new independent verification here. Its complement is **not automatically hardware prefetch**. Counts and observation windows differ; instruction reads, demand RFO, other agents, outstanding requests and event defects can contribute. The new numerator still omits worker demand RFO and does not make scanner P4 comprehensive (P4 contains only demand data plus P1). Keep the known RFO erratum caveat; a suspect counter cannot simply be silently promoted into a trusted partition.

A 0.70 floor is an arbitrary coverage heuristic. Even perfectly disjoint events explaining 70 of 100 reads do not fix the dominant phase: observed induction=36, writer=34, residual=30; assigning the residual to writer reverses the ranking. At coverage below 70%, a decisive off/on result remains decisive for that intervention. Conversely, coverage above 70% cannot cure overlap, scope mismatch, phase-boundary carryover or non-contemporaneous passes. A fraction above one is a consistency alarm, not extra confidence.

Thus R1 may annotate **LOW COUNTER COVERAGE**, but must not short-circuit R2/R3. The draft is internally ambiguous here: “first matching rule decides” stops at R1, while “only Arm B's answer stands” implies a second decision. Resolve it by the two outputs above.

**R2 and R3 do not need Arm I.** For paired repetition j, use r_j=reads_off/reads_on and t_j=wall_off/wall_on, with the carried-over estimator explicitly named and applied to complete pairs. If the read estimator is the geometric mean ratio, “cut >=30%, lower bound >20%” means estimate(r)<=0.70 and upperCI(r)<0.80. R2 also requires upperCI(t)<=1.02. R3 means estimate(t)>=1.03 and lowerCI(t)>1. Preserve uncertainty, units, denominators, idle correction, pairing and the no-retry policy before data. These predicates do not overlap because R3's estimate exceeds R2's upper bound allowance. They identify net intervention effects, not a mechanism.

## Arm I feasibility: establish support before freezing it

Intel's current primary perfmon mapping, pinned for this review at **8da58930a226ab3eac72f309bd0fb1d5abfc6245**, maps CPUID family6 models A5/A6 to SKL/events/skylake_core.json V59. The fetched table's OFFCORE_RESPONSE request families are **OTHER, DEMAND_CODE_RD, DEMAND_RFO, DEMAND_DATA_RD**; it does **not** enumerate the proposed prefetch-data and prefetch-RFO DRAM events. This is an availability gap, not proof no hardware encoding can exist. The builder must supply a model-applicable primary definition and errata support for P1/P2 before treating them as measurable. Do not transplant another microarchitecture's filter bits. Arm B's independent question survives if Arm I cannot be supported.

Also ensure worker and scanner scopes are disjoint and normalized to the same work; separate instrument passes produce estimates, not a literal simultaneous conservation ledger. The “at most two events” statement alone does not prove scheduling: whole/phase groups, offcore-register constraints and running/enabled validity must still be checked.

## WRITE: no global upper bound follows from P3

Intel describes L2_LINES_OUT.NON_SILENT (F2/02 in the mapped table) as modified lines evicted on L2 fills and written to L3. It does not count memory-controller writes.

Two counterexamples settle W1:
- Many counted dirty evictions remain in LLC or repeatedly update the same line there: P3 is large while their DRAM writes in the measured interval are zero.
- Dirty LLC contents from earlier activity drain during this interval, or scanner/other cores generate writes: IMC writes are positive while the monitored workers' P3 is zero.

There is therefore **no unconditional ordering** between 64 P3 and total measured IMC writes. Even a genuine upper bound >=50% would establish only that a large contribution is possible, not that it occurred. W1 reasons in the wrong direction.

For the narrowly defined downstream contribution of the counted evictions, under an explicit once-only writeback accounting model with no duplicated downstream writes, 0 <= attributable bytes <= 64 P3; within the same observed interval also <= total IMC-write bytes. That conditional envelope does not bound *all* writes and supplies no positive lower bound. With the draft's measurements alone, the unconditional attributable fraction is only **0 to 1** (for a positive physical-write denominator). An idle-subtracted estimate is not itself a conserved physical byte total.

Likewise “50% of P3 during induction” locates the **evicting phase**, not the phase that dirtied those lines. Induction can evict lines dirtied by the writer; writer can evict induction's lines. Report P3/IMC as a scale comparison, not “writes explained,” and retain write-avoiding induction as a hypothesis rather than WRITE-BACK TARGET.

## Arm B: valid scoped intervention, not residual subtraction

Leaving scanner prefetch unchanged is not inherently a confound: the treatment is **worker-core prefetch off with scanner prefetch on**. Pairing that intervention can estimate its total decoder/IMC effect. Scanner traffic need not cancel numerically: worker throughput changes scanner backpressure, cache interference and overlap. Those changes are mediated effects of the treatment, so IMC_on minus IMC_off is not the number of useless worker-prefetch bytes.

Require disjoint physical scanner/worker cores, fixed affinity, controlled SMT siblings and consistent physical-core MSR scope at both worker counts; read back the intended settings. If scanner shares a worker physical core, the claimed partial treatment is not what was delivered. Do not interpret the result as “all prefetch off.” Four prefetch controls changed together also cannot identify which prefetcher is responsible.

Louis's explicit MSR permission remains necessary. Save and restore the original values, not assumed defaults. “On any exit” needs a realistic failure/recovery plan: a process trap alone does not handle SIGKILL or power loss. This review authorises no MSR writes or workstation run.

## Report only and next value

Report Arm I coverage/phase splits, P3 ratios, scanner counts, one-worker wall and the on/off write change. R3 slowdown establishes a net benefit of the enabled worker prefetch configuration, not “traffic is real demand fetched early”; reduced reads with neutral wall establishes expendable net traffic under this workload, not all prefetch waste. A cap-dose study can remain a candidate after mixed results, but the draft supplies no evidential reason to force it as the next experiment.

Expected advance: resolve whether worker prefetch control changes physical traffic without unacceptable wall loss. No new empirical gap was closed in this review; the logical gap was closed by counterexamples and primary event semantics. The bounded B intervention has information value independent of an incomplete ledger. Arm I's value is conditional on documented event availability; W1 cannot select an implementation target.

## Primary sources and verification limits

- Intel mapping: https://github.com/intel/perfmon/blob/8da58930a226ab3eac72f309bd0fb1d5abfc6245/mapfile.csv
- Intel events: https://github.com/intel/perfmon/blob/8da58930a226ab3eac72f309bd0fb1d5abfc6245/SKL/events/skylake_core.json

These support event scope/mapping, not a claim that this workstation's CPUID/MSR behaviour has been tested. This was a registration review, not a run-bundle replay or an implementation review. Existing phase-study validity is carried forward only as a requirement; the new code and exact encodings are not yet supplied. Routing audit passed using fresh API comments after a connector transport failure; migration head and manifest digest matched.
