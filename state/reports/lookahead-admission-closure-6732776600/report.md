# #271 admission re-review: PROCEED at 8a0fc8e0

Inbox **6732776600**, dispatching QUEUED **6731064440**. Reviewed runnable head **8a0fc8e099f932c14cd746c4c5d81aaef8dcc75d**, input freeze **4ac25fcd1a3c6a898221d23fbcf7d3a904deb2c5**. Instructions: SPEC, SPEC.RESEARCHER, MODE, RULES and PROTOCOL at **a4f97714cc14366984015869d9d094d89488ac57**.

**PROCEED for the registered admission experiment at that exact runnable head. My narrowed HOLD 6730774618 is resolved.** LA3, LA4, N1 and every numerical threshold remain unchanged. Guardian D057 separately lifts the D045/D053 evidence hold for this runtime; I do not recreate or widen that safety decision. This is admission to the prospective workstation experiment, not a performance verdict, production approval or authorisation for agent timing.

## 1. The four original counterexamples close

I fetched the original `probe.py` from research commit `bab88d7476b4b3ebce3ec069e391c67be755c444` and ran it unchanged against the new head's reporter and fixture helpers. This is independent replay of the actual witnesses, not merely acceptance of the replacement tests.

| Input to the otherwise-valid fixture | New result | Source-level reason |
|---|---|---|
| Each trace requests one round, has three children and reconciled summaries | INVALID, exit 1 | Request is bound to six; inventory is the fixed six rounds × three modes, not the supplied request |
| Raw off child has `hash=BAD err=failed`, while summary parity remains true | INVALID, exit 1 | Every child's own hash, output length and error convention are checked |
| One complete round has `idle_prep_share=99`, hidden by five ordinary values | INVALID, exit 1 | Every complete round's required shares/utilisations are checked in [0,1] before reconciliation |
| One complete round has `idle_prep_share=NaN` | INVALID, exit 1 | Nonfinite required values fail their own domain check and cannot be dropped from the median |
| Unmodified positive fixture | VALID, exit 0 | Control preserved |
| Six requested rounds, eighteen accounted-for children, only one complete timeline round | VALID, exit 0 | The inherited ≥1-complete rule is preserved; no new six-complete-round gate |

All **71 repository Python fixtures pass**. Their additional coverage rejects mismatched raw digests, wrong raw output size, missing metrics, utilisation above one and negative waits; a genuinely incomplete round remains ignored for timeline aggregation. `probe.json` retains exact failed checks and `tests.log` the suite result. The first local import attempt lacked `wikidata.json`; after fetching that pinned dependency, both commands completed. No test failure was discarded or changed into a pass.

The new code checks statuses and scheduled identities for **all** raw children before filtering incomplete timeline rounds. Thus the incomplete-round allowance cannot excuse a failed/missing child, wrong output length or raw error. It then validates each complete timeline round's domains, service count, 431 blocks and `1 <= P_median <= P_max <= L+K-1`, and finally reconciles summaries. That order repairs the identified loss of contradictory evidence.

The documented 431 remains an expected scanner count tied to the fixed corpus digest and prior traces, not a field allegedly supplied by `wikidata.json`. The tolerance is now accurately described as `1e-6 * max(1, |median|)`, separate from raw-domain validation. Neither requires another HOLD.

## 2. Contract and freeze are unchanged where they must be

GitHub's comparison from prior reviewed head `562e4ecfc16c7dd75eb62f9edc8800f89a62cd13` to 8a0fc8e0 changes only the freeze file, reporter, its tests and registration text. The diff does not change the comparison/outcome functions, controls, thresholds, runtime or recipe. Earlier H1/H3 and native-child-status findings accepted in 6730774618 remain closed. In particular, mechanism reporting still intersects gaining and relieved arms, and describes a higher-L **policy effect**, not identified queue-supply mediation. Individual nominal 95% intervals do not become a familywise claim.

The runnable recipe still drives eight cells × twenty rounds at requested 10 threads, and five occupancy passes × six requested rounds × three modes. It refuses changed measured inputs, retains failed traces in the bundle while excluding them from admission, and requires both report exit 0 and JSON VALID. An INVALID dry run is never evidence for the performance question. The claimed native six-round L4/w8 pass and dry run were read from the PR/registration; I did not independently execute or obtain their raw local bundles in this review. They are author-reported compatibility checks, distinct from the independently replayed Python evidence above.

**Set the ready tag to `8a0fc8e099f932c14cd746c4c5d81aaef8dcc75d`.** `4ac25fcd1a3c6a898221d23fbcf7d3a904deb2c5` is the measured-input reference recorded in that head, not the runnable checkout to tag. The full freeze-to-head comparison changes only `bench/service-lookahead.frozen`: at 4ac25fcd that file still points to de495219; at 8a0fc8e0 it points to 4ac25fcd. Running 4ac25fcd would compare the repaired reporter against the superseded input freeze and refuse it. This is a pin distinction, not a demand for another code revision or freeze.

At inspection #271's head is exactly 8a0fc8e0. Its 27 check-runs are completed: **24 success, three skipped**, including successful Miri, Rust, submit, occupancy and bench checks. These are API status observations, not a fresh audit of every job's log. CI and exact tag identity must still hold when the Coordinator queues it under PROTOCOL §9.

## 3. Safety clearance is separate and scoped

I read Guardian [D057](https://github.com/lmmx/mibz2/blob/aa0538a305b51368e8274c1bb508d8316c30bd95/state/decisions.md#d057--d045d053-evidence-repaired-scoped-service-core-resumption) and its published review **6732679972**. It explicitly covers #271 runtime e64469f6 / 8a0fc8e0 under its stated input/kernel/pool-use and external-source exclusions. My independent GitHub comparison e64469f6→8a0fc8e0 confirms six changed harness/recipe/documentation files and **no `src/` change**. Do not silently move this admission to a guarded integration or newer master runtime.

D057's reported pass B has 3,694 cases, build marginal 2,603 and walk marginal 2,603, zero failures. Equal marginals are not an exact joint count: inclusion–exclusion gives a joint lower bound of 1,512. Prepared maxima 3/4/5 are observations consistent with L+K−1, not proof of that bound by sampling. D057 records completed 17/17 Miri and the completion-fence argument; it lifts its own material objection. I have not rerun that Rust fuzz/Miri evidence or issued a new B5 clearance. Its source-panic exclusions and separate production/integration obligations survive.

The frozen registration's old “held under D045” wording is historical. Record D057 and this review beside it; do not edit measured registration inputs merely to erase the old status. Louis's own opt-in run/hand-over or submission is still necessary under C5; its archived bundle and digest must be checked before judging any result. No authority for further runs, relaxed admission or production adoption follows.

## 4. Consequence

Expected: reject four contradictory traces without changing the experiment. Observed: all four rejected; positive and genuinely partial-completeness controls preserved; 71 fixtures pass; freeze and runtime identities reconcile. The evidence-contract gap closes. No decoder speed gap has been measured here.

The useful next step is the already-registered workstation experiment, through the exact-head ready/queue and C5 submission procedure. It can distinguish higher-L policy effects and arm-specific trace consistency while retaining the window controls. Another admission rewrite or another safety-approval loop has no demonstrated value for the resolved findings.

Independent work: pinned source/diff review, original counterexample replay, fixture suite, freeze/CI read-back and channel audit. No native build, timing, capture, fuzz, Miri, implementation, merge, tag, queue or automation change. Migration pin/manifest matched; channel audit found five resolved historical misroutes and no unresolved findings. `validation.json` lists source blobs and checks. To reproduce: fetch its bench files from 8a0fc8e0 into `bench/` beside `probe.py`, then run `python3 -W ignore::ResourceWarning -m unittest discover -s bench -p test_service_lookahead.py` and `python3 -W ignore::ResourceWarning probe.py`.
