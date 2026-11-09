# C29 R5: collector and preflight review

**PROCEED to freeze R5 for one timed attempt at `0ce4dca4489c4a87b9658d6c2656290173113c99`.** The earlier coverage objection is closed on the observed runner layout. This is a research assessment under RULES C1/C3, not a dispatch, merge, production qualification, or retrospective admission of the first run.

Inbox [6738933099](https://github.com/lmmx/mibz2/pull/162#issuecomment-6738933099), dispatching queued [6736130294](https://github.com/lmmx/mibz2/pull/145#issuecomment-6736130294). Instructions: SPEC, SPEC.RESEARCHER, MODE, RULES and PROTOCOL at `a4f97714cc14366984015869d9d094d89488ac57`. PR [#288](https://github.com/lmmx/mibz2/pull/288) remains open, unmerged, at the exact reviewed head when checked.

## What closes the objection

For a complete ancestor chain, its nominal bandwidth ceiling is the minimum of finite Q/P readings, or unbounded if every level has justified absence/unlimited status. With one unread ancestor, the observed minimum is only an upper bound. A child allowance of 4 or 8 CPUs does not exclude a hidden 2-CPU parent.

The Rust collector now keeps those cases separate: every path level is visited, read errors remain unknown, and completeness also requires the mount boundary to be the real hierarchy root. v1 CPU placement takes precedence over a coexisting v2 hierarchy; mount roots, escaped mount points and overmounts participate in resolution. Its finite minimum uses u128 cross-products. The Python identity reparses quota file text, verifies the leaf-to-root path sequence and boundary evidence, and the reporter uses integer Q >= 4P. Decimal `cpu_quota` is not the gate.

Missing root `cpu.max` is not an unread finite limit: the kernel specifies both `cpu.max` and `cgroup.type` on non-root cgroups. The collector permits this exception only at mount root `/` in the initial cgroup namespace, with both files absent. It does not generalise ENOENT to unlimited at arbitrary levels. Elsewhere, missing `cpu.max` requires readable controller-absence evidence; EACCES cannot take that path.

Scope correction: Python re-derives quota classifications and path continuity; it does **not** independently reimplement mount selection from the full raw mountinfo. The Rust collector remains part of the trusted measurement chain. This review cross-checks that selection against separately archived raw proc/mount data. Do not describe identity/4 as a general authenticator of arbitrary edited JSON.

## Independent evidence

Downloaded [artifact 13660767430](https://github.com/lmmx/mibz2/actions/runs/42589756805/artifacts/13660767430), 13,116 bytes, from untimed [run 42589756805](https://github.com/lmmx/mibz2/actions/runs/42589756805), attempt 1. Locally computed ZIP SHA-256 matches GitHub metadata and the upload log:

`ff50767ab52c399137cf4830a17ae0f17dd1b05460dd10b53714b4745022206f`

Run metadata, log and raw meta file all name the reviewed head. The job selected `c29-preflight`; corpus and decoder execution were skipped. Fourteen archived files include both environment samples, raw membership, full mountinfo, namespaces, status, directory listings and level-file reads.

Raw evidence agrees with the census:

| Fact | Observed |
|---|---|
| Membership | `0::/system.slice/hosted-compute-agent.service` |
| cgroup namespace | `cgroup:[4026531835]` |
| Mount | ID 36, cgroup2, root `/`, point `/sys/fs/cgroup` |
| Leaf and `/system.slice` | `cpu.max = max 100000`, burst 0, weight 100 |
| True root | Neither `cpu.max` nor `cgroup.type` exists |
| Effective CPU set / affinity IDs | `0-3` / [0,1,2,3] |
| Execution environment | AVX2 true; nproc, affinity count, available_parallelism all 4; SCHED_OTHER |
| Image / kernel | ubuntu26 `20261101.186.1`; `7.0.0-1019-azure` |

The independently reconstructed ancestor chain is exactly leaf → /system.slice → /. All three directory paths resolve through the selected mount in full mountinfo. Both endpoint censuses give complete/unlimited; the declared configuration comparison is stable. Replaying the exact-head preflight against the downloaded samples passes.

Local `bench/test_c29_checkpoints.py`: **18 tests pass**, covering all 27 census fixtures and synthetic reporter runs. This includes the four earlier hidden-parent witnesses, all refused; the old rule would falsely admit three. An additional [review-only audit](audit.py) checks raw layout correspondence, inserts EACCES into each of the three real-run levels (all become unknown), and tests two rational boundaries: 399999/100000 and 1199999/300000 are refused despite rounding to 4.00; exact 4 is accepted. Results: [audit-results.json](audit-results.json).

GitHub's exact-head check read returned **27 successful, 3 skipped, 0 pending/failed** checks. Skips are Release, deploy-branch and Publish crate. The successful c29-reporter job independently reports 18 passing tests. I did not rebuild Rust locally (rustc unavailable), run the decoder, or reproduce performance.

## What the pass does not establish

Unlimited quota is absence of a bandwidth cap in the visible, verified hierarchy, not exclusive CPU ownership or constant service rate. Matching endpoint configurations cannot prove there was no transient change between them. The registration already states that limit; neither zero throttling counters nor a preflight pass upgrades it to continuous proof. The timed attempt must repeat its own preflight and keep its after-sample; today's different job cannot stand in for either.

The collector repairs an observability failure. It establishes **no speedup**. The first attempt, 42582708226, remains terminal VALID, UNJUDGED with 0.9700 [0.9647, 0.9753] disclosed separately. It must not be pooled, reclassified or used to select the better attempt. A nominal 95% interval from the follow-up is not a programme-wide error guarantee after inspecting the first result.

R5 preserves the two arms, corpus, 20 rotated rounds, nine-thread primary and three-thread secondary, paired-log analysis and thresholds: LEAD wall estimate <= .98, wall upper < 1, CPU upper <= 1.02; STOP wall lower > .99; otherwise INCONCLUSIVE. Its value is a bounded answer to the still-unjudged C29 question, now with the missing environment evidence available. No further collector exploration is needed before that one attempt.

Freeze must name the exact measured head and record the completed preflight/CI (the draft's opening chronology is stale). Archive a failed preflight without decoding. INVALID or UNJUDGED on the timed follow-up suspends C29 timing on this runner configuration; no automatic third attempt, runner shopping, extra rounds, or selection of favourable cells. STOP and INCONCLUSIVE are terminal for this registration too. A later investigation needs a separate diagnosis and decision. Clock-telemetry work is outside this review.

## Sources

- Reviewed code and R5 registration: [tree at 0ce4dca4](https://github.com/lmmx/mibz2/tree/0ce4dca4489c4a87b9658d6c2656290173113c99), specifically `examples/cgroup/mod.rs`, `bench/c29_identity.py`, `bench/c29_checkpoints.py`, its tests/fixtures, `.github/workflows/sweep.yml`, and `docs/journal/2026-11-09-c29-follow-up-registration.md`.
- [Kernel cgroup v2 documentation](https://docs.kernel.org/admin-guide/cgroup-v2.html): hierarchy restrictions, controller availability, cpu.max and cgroup.type.
- [Kernel initial namespace constant](https://code.googlesource.com/linux/torvalds/linux/+/78273df7f646f8daf2604ec714bea0897cd03aae/include/linux/proc_ns.h): `PROC_CGROUP_INIT_INO = 0xEFFFFFFB` = 4026531835.
- [Exact-head checks](checks.json), artifact manifest/digest above; raw artifact replay is reproducible with audit.py's command.
