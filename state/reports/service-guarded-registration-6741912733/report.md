# Guarded service registration: REVISE before freeze

Inbox [6741912733](https://github.com/lmmx/mibz2/pull/162#issuecomment-6741912733), #302 at **3c9df6701c3a3fc60502598482a50cec266f9cb5**. Specs a4f97714cc14366984015869d9d094d89488ac57.

**REVISE.** The exact switched/current-default product comparison has high information value. The draft is not ready for a freeze: restore workload and evidence admission, settle the grid/replacement authority, and exercise the recipe beyond its initial refusal. This review neither reopens D069's resolved safety objections nor answers D067's adoption question.

## Reproduced findings

Ran the pinned Python suite: **8/8 pass**. Adjacent probe.py uses its positive synthetic grid, changes one property at a time, and calls the unmodified judge. No decoder timing, native build, implementation or workstation execution.

| Witness | Observed |
|---|---|
| Ordinary complete grid | VALID, GAIN / CPU SAVING |
| Every output size changed to 1 | Same valid gain verdict |
| Every output digest changed to non-hex “not-a-digest” | Same valid gain verdict |
| preflight only {"ok":true} | Same valid gain verdict |
| preflight ok=true, quota Unknown, target None, no observations | Same valid gain verdict |
| One record exit=17 with empty err | Same valid gain verdict |
| Round zero encoded as false | Same valid gain verdict |
| Base path changed to an unregistered directory ending /bin/ab-base | Same valid gain verdict |
| One output hash changed, retaining all others | INVALID, no P1 |

These are admission witnesses, not evidence that a real decoder emitted these records. The path case establishes the judge's reliance on the recipe, not that the recipe runs an arbitrary path.

**R1 — restore workload binding.** The judge accepts any common positive output length and nonempty string hash, and takes no corpus digest input. The recipe records the corpus SHA-256 in inputs.txt but the reporter does not check it. The earlier service-2x3 registration, fetched at **3e201246cf1266d4b2a984cb19eb9fdba5b9835d**, explicitly repaired this: 209700196 decoded bytes, hex output digest agreeing with the reference, and the registered compressed corpus digest. Reuse that evidence boundary. “The builder normally produces it” is weaker than reproducible bundle admission. Bind corpus digest, frozen candidate/base and build identities through either the report or an explicit mandatory bundle verifier; label the limited report accordingly. A bare VALID must not stand for checks it never performed.

**R2 — validate preflight evidence, not only its success bit.** svc_target emits useful observations, but they are mostly Rust Debug strings and the judge reads only ok. At minimum require a versioned, complete record tied to the candidate, kernels, known topology/quota, and the 9/10/11 predicate/build observations; reject contradictions and missing facts. Structured observations make this check reproducible. The recipe's genuine executable and exit check provide provenance; they do not make a contradictory archived record adequate for independent replay. No need for an atomic kernel snapshot or continuous topology monitor as a new prerequisite.

Harden row validation alongside this: strictly typed integer rounds, and explicit successful exit status where the harness schema provides it. Fixtures currently omit exit entirely. Test the actual producer's failed-child encoding rather than assuming empty err means success. These smaller schema defects are not a reason to rerun safety mutations.

**R3 — keep statistical labels honest.** The numeric outcome map matches the stated rule, but markdown loses the service precedent's qualifications. CPU INCONCLUSIVE is “no CPU cost detected, not shown to cost nothing”; NULL is bounded uncertainty within ±3%, not proof of no effect. A speed gain with inconclusive CPU may retain the historical reading label but cannot be described as meeting a demonstrated CPU non-inferiority condition. Report both intervals. Two co-primary labels are not automatically a simultaneous 95% guarantee.

Thread names verify the sampled plan counts, not a semantic proof of the window or unchanged kernel. Window follows only after inspecting the pinned base and head's dispatch/window code. Count equality at 9/11 is a fallback check, not binary identity or no-regression evidence.

## Answers to the six questions

1. **Base: master at freeze, by immutable SHA, integrated into the candidate first.** This asks whether the proposed default improves the default that would otherwise ship. A release comparison answers an additional user-upgrade question and can include unrelated changes; do not substitute it here. “Last merged master” is acceptable only when reconciled with current master at freeze. An ancestor check alone cannot establish that. Record both source identities, toolchain/configuration, and any unrelated measured differences.

2. **Scientifically prefer the narrow 9/10/11 design for this question, subject to C6 authorization.** Exactly 10 changes dispatch; neighbouring budgets test both sides. The pipelined precedent changed the default at every tested budget ≥10, which justified its wider no-harm family. That intervention differs here. Wider coverage could price fallback discovery/layout costs, but it is a separate coverage decision, not free evidence from plan equality. Neither design qualifies all budgets or all admitted machines. **Do not choose a new grid by analogy:** RULES C6 reserves the thread grid. Cite an applicable Louis decision for this exact design or resolve it before freeze. The fetched service-2x3 precedent is one budget with six arms; the pipelined precedent is seven budgets with two arms. Neither is this three-by-three design.

3. **RSS: retain reported-only for this diagnostic product comparison.** Do not invent a convenient ceiling from old 64/74 MB observations. Report per-process whole-process peaks, medians and paired head/base ratios. A corpus observation is not a resource bound; windows, front buffers and overlapping work need separate accounting. If the decision genuinely requires a memory budget, specify its engineering basis and decision role before freezing, through the applicable authority. No result without such a condition can be advertised as passing one.

4. **N1: retain the primary at 10; report the existing twins at 9/11.** Extending N1 validity to all three creates more ways to invalidate and potentially replace an attempt. That changes selection/repeat policy, not merely diagnostic reporting. Do not promote or drop controls after seeing results. The new twin is a copied base binary under a different path, whereas service-2x3 used plain team3 and its twin in one build: state that changed implementation explicitly. The A/A numerical rule does not certify differential overhead or between-build layout effects.

   **Replacement needs its own authority entry.** service-2x3 explicitly says an invalid run decides no rerun; the earlier pipelined experiment's single replacement was experiment-specific. This draft grants one replacement even for a noise-based N1 failure or preflight refusal. “At most one” is bounded but not thereby authorized by those precedents. Preserve all attempts; distinguish structural failures, N1 failures and preflight refusals; settle their slot consequences before freeze. No retry-to-pass.

5. **Yes: prepare a separate frozen preflight-only target entry before spending a timed slot.** It should build/run svc_target, archive the source/executable/toolchain identities and full observations even on refusal, and perform no ab timing grid. Louis's execution/submission supplies C5 authority; this review runs nothing. Keep Unknown fail-closed. This has strong value because an unreadable quota would otherwise consume setup and possibly the draft's replacement allowance. An earlier successful preflight does not replace the timed recipe's own preflight or guarantee stable facts later.

6. **Yes: require an end-to-end non-evidentiary recipe check before readiness.** It must pass the freeze check using a clearly labelled test fixture or a reviewed pointer-only freeze, execute both build paths, copy and hash binaries, record preflight pass/refusal, construct the corpus/input manifest, produce the expected incomplete-grid report, and bundle every exit path. Test refused, invalid and interrupted outcomes plus successful packaging. Do not edit the eventual frozen measured inputs to get through it. The current DRY branch still calls ab run for one round and produces timing fields: “untimed” means excluded from performance evidence here, not that no clock is read. Do not interpret those numbers. Put the judge/probes and refusal/bundle tests into CI, following the existing submit-test precedent. Eight local Python passes do not establish the shell recipe works.

## Scope, mechanism and value

D069 lifted the technical repair hold at 6642faf4; its publication-history and pooled-reach wording fixes are attributed to the brief, not independently re-certified here. D067's class/adoption boundary remains: a single i9 run and off-path CI cannot jointly be relabelled as qualification of every admitted automatic-default machine. No changed CI merge rule is authorized.

Expected: test whether the guarded current implementation retains the historical product gain. Happened: no target result yet; a useful draft with reproduced admission gaps. Gap closed: none of the production-speed gap; this review specifies how to make the next result interpretable. Mechanism: product changes plan, window and thread count together, so a gain would not isolate service-thread causality. Next value: preflight then one properly authorized, frozen current-default comparison is higher than more old-runtime replication; admission repairs protect that investment.

Evidence: exact-head registration, service_guarded.py, test_service_guarded.py, svc_target.rs, .just/bench.just and frozen file; pipelined-default registration at that head; service-2x3 registration at 3e201246; Guardian comments 6740148831 and 6741390621. The draft's service-2x3 path is absent at its own head: cite the historical commit instead of leaving an unresolvable relative reference.

Migration check passed (prior branch1283b969e9e295e268257cd1b6be3ad7da29e6dd; manifest9b6bfe5b8e2193a68d8e788873da11947197c3457b6ad6e454b580a05f7b8fd0). Fresh channel audit: five resolved historical findings, zero unresolved. No merge, tag, queue change, policy waiver or automation change.
