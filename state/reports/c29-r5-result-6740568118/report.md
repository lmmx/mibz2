# C29 R5: judged uncertainty, no renewed timing authority

Responds to inbox [6740568118](https://github.com/lmmx/mibz2/pull/162#issuecomment-6740568118). Reviewed head **f693e0d6b7429a8325dbea95979c147778d0c060**, [Actions run 42615047229](https://github.com/lmmx/mibz2/actions/runs/42615047229), attempt 1. Specs: a4f97714cc14366984015869d9d094d89488ac57.

**Concur: VALID, JUDGED, primary INCONCLUSIVE. Close this registered timing attempt. Correct the effect-size wording and “equal peak RSS.” Further C29 work needs a concrete mechanism question; imprecision alone is not a reason for another run.**

## 1. Artifact admission and replay

Downloaded artifact **13671533599** and independently verified ZIP SHA-256 **b89bdfff7050d6b03678334963cef05c8e31170f1cc081429db159507dd8fc44**. Actions metadata identifies the exact head above, workflow_dispatch on perf/c29-collector, successful attempt 1.

Replayed the exact-head `bench/c29_checkpoints.py report` with the raw JSONL and identity. Exit 0; JUDGED; both printed threshold readings INCONCLUSIVE. Independently paired all observations by thread setting and round, recomputed mean log(C29/control) and its t19 interval, and checked raw admission facts with adjacent `audit.py`. No filtering, replacement, native build or new timing. The independent t19 constant is more precise than the reporter's table; interval endpoints differ by less than 0.0000002 and agree at all reported digits.

| Setting | Wall C29/control, 95% interval | CPU C29/control, 95% interval | Status |
|---|---|---|---|
| 9 | 0.9890 [0.9732, 1.0050] | 0.9903 [0.9758, 1.0050] | Single judged primary; INCONCLUSIVE |
| 3 | 0.9871 [0.9769, 0.9974] | 0.9907 [0.9829, 0.9985] | Secondary only; never promoted |

Below one means less time. R5.6 LEAD requires wall point ≤0.98, upper <1 and CPU upper ≤1.02. The primary fails both wall conditions. STOP requires wall lower >0.99; that also fails. The secondary's interval below one does not rescue the primary, and its point estimate also misses the registered LEAD minimum.

The identity is schema identity/4, clean tree, release, no features, rustc 1.99.1 / LLVM 23.1.1. Recorded before/after binary hashes both equal `fdd83fecaed06e60b42bfdc47709464597df11b6226c1cecba85137fe4aeb334`; corpus hashes both equal `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`. This checks recorded consistency: neither binary nor corpus is in this ZIP for an independent rehash.

The separate raw before/after environment files exactly equal their identity embeddings. Both show AVX2, nproc/affinity/available parallelism 4, CPU IDs 0–3, Xeon Platinum 8370C. The reported virtual topology is two cores/four CPUs, not the target i9.

Raw cgroup evidence is sufficient for **R5 admission**: v2 membership /system.slice/hosted-compute-agent.service; mount root /; initial cgroup namespace cgroup:[4026531835]; contiguous leaf → /system.slice → /; both non-root cpu.max files say max 100000; real-root cpu.max and cgroup.type explicitly absent. No missing ancestor. Re-deriving with the frozen identity code yields complete/unlimited before and after, stable true with no differences. Independent raw checks agree. Counters change, configuration does not. The timed workflow records its preflight before decoding and a distinct after-sample after decoding; preflight replay passes.

“Stable” means equal sampled configurations, not proof of no intermediate change. Unlimited quota is not exclusive CPU service or controlled frequency. These are limitations already acknowledged by R5, not retrospective invalidations.

All 80 records have exit 0, output length 209700196 and hash a8830caee5c511a6, with all 20 round pairs at both settings. This establishes the artifact's output agreement, not a new universal correctness proof.

**RSS correction:** “equal peak RSS” is only equal rounded displayed medians of per-decode peaks. Raw medians, C29/control, are **75498/75794 kB** at 9 and **40354/39574 kB** at 3. Maxima are 77920/78216 and 41400/41216 kB. Neither equality nor a memory bound follows. This wording correction does not change the registered verdict.

## 2. What can be said

Use: **“On this runner and workload, the primary estimate is a 1.10% elapsed-time reduction; the registered 95% interval spans a 2.68% reduction to a 0.50% increase. A benefit is not established, and the registered LEAD criterion is not met.”**

“A percent-scale or smaller effect, not established” is acceptable only as an informal description of the point estimate. As a bound it is wrong: this interval still permits more than 2% reduction. Nor does inclusion of one establish zero effect, equivalence or STOP.

Keep run **42582708226**, wall 0.9700, terminal VALID/UNJUDGED and separate. R5 disclosed that result before its new freeze; it is not an unobserved preregistered discovery. Do not pool, select the better run, repair the old verdict, or claim programme-wide multiplicity-adjusted evidence. The newer point estimate is smaller; the two runs do not isolate a causal reason for that difference. No default-on or production performance qualification follows.

## 3. Where diagnosis can buy information

The strongest next C29 question is **transfer**, not another estimate on this runner: which machine work survived the move from the copied research worker to the pipelined front, and is that work exposed on the critical path?

C29 combines a register-list representation with short-run stores. Its journal predicts transfer of the earlier approximately 0.974 result; this observation is approximately 0.989 with a substantially wider interval. It does not identify register-list benefit separately from short stores, worker scheduling, build layout or host interaction.

A bounded untimed investigation could justify itself with these deliverables:

1. **One explicit machine-work discrepancy or a negative result.** Compare the research and ported kernel's existing listings/source paths: register spills, dependent selection, checkpoint splits and short-run-store reach. Charge physical store bytes and fallback paths, not merely source operations. Record the fixed comparison; do not search padding for a favourable result.
2. **One critical-path prediction.** For a simplified stage model T=max(F,H)+W, removing front work ΔF saves at most min(ΔF,max(F−H,0)) if H and W remain unchanged. Front CPU reductions can disappear entirely when helpers bind. A diagnostic must identify exposed work or interference; aggregate CPU and wait-entry labels cannot do that. If shared-resource effects matter, name the violated model assumption and the observable that distinguishes it.
3. **A route to actual use.** The explicit 9-thread arm forces three teams; the public default reviewed by the Optimiser cannot select C29. A concrete proposal must prove exact natural dispatch and prefetched-stage reach, with D060's held-prior/current/next and partial-front unwind obligations, before paying for adoption evidence. Counts or source reach alone do not discharge independent B5.

These are proposed information objectives, not new timing permission. A finding of a removable discrepancy could support an independently judged new candidate or target question. “The interval is wide,” “the first run was better,” and “try another host” cannot reopen R5. This configuration's registered timing is closed; the wider C29 idea has not been proved impossible.

**Ranking:** Optimiser [6736939946](https://github.com/lmmx/mibz2/pull/145#issuecomment-6736939946) ranked collector repair/fresh diagnostic above C43; that purchase has now been made and yielded no LEAD. Its priority cannot be carried forward unchanged. [6739420751](https://github.com/lmmx/mibz2/pull/145#issuecomment-6739420751) puts guarded service adoption first and C29 first among bounded kernel investigations. Retain service adoption first: it addresses the production decision directly, while keeping its own current-candidate safety, reach and performance gaps explicit. Retain C29 only as a small, falsifiable transfer audit with a stopping deliverable. If it finds no new discrepancy or exposed gain mechanism, pause it rather than spending another performance package. Service integration difficulty alone is not affirmative C29 evidence.

## C3 accounting and scope

Expected: transfer a roughly 2–3% kernel result sufficiently to reach a ≥2% registered lead. Observed: primary approximately 1.1%, uncertain, no LEAD. Gap closed: admission/identity uncertainty for this attempt; **no established production-speed gap closed**. Mechanism learned: the combined implementation's end-to-end result on this four-vCPU setting, not causal separation of its components. Next value: limited untimed transfer/critical-path audit; guarded service adoption remains higher priority.

Sources: exact-head follow-up registration R5, original C29 journal, reporter, identity builder, team-size helper and sweep workflow; raw artifact above; cited Optimiser reviews. No raw binary disassembly was independently obtained here, no old run reanalysis, and no new safety qualification. One research report, no implementation, merge, runner dispatch or authority change.

Migration check passed against prior branch 1283b969e9e295e268257cd1b6be3ad7da29e6dd and manifest digest 9b6bfe5b8e2193a68d8e788873da11947197c3457b6ad6e454b580a05f7b8fd0. Channel routing audit: five resolved historical findings, zero unresolved.
