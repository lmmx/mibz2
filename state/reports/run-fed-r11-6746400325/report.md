# R11 review: narrow REVISE before freeze

Assignment **6746400325**; PR #305 at **cb4bccf025a79b712e0d2316eea1338e6f8fc582**. Specification pin: **a4f97714cc14366984015869d9d094d89488ac57**, docs/team/SPEC.md, SPEC.RESEARCHER.md, MODE.md, RULES.md and PROTOCOL.md.

**REVISE the preflight's acquisition-status admission.** The earlier quota, exact-type, CPU-identity and active-collector objections are closed at the reviewed source. Child termination is now preserved and checked. One remaining technical issue exists independently of the pending C6 re-pin: the preflight ignores its collectors' archived failure/timeout statuses. Do not run this draft to resolve that question.

## Evidence and remaining counterexample

[Recipe](https://github.com/lmmx/mibz2/blob/cb4bccf025a79b712e0d2316eea1338e6f8fc582/.just/bench.just), lines 1068–1079, runs the three collectors in a brace group followed by `|| true`. Each command writes its status to preflight/exits.txt. The group's conditional context suppresses shell errexit inside it; recording the status is not a refusal. The recipe subsequently gates only the Python preflight's status and verdict.

[preflight()](https://github.com/lmmx/mibz2/blob/cb4bccf025a79b712e0d2316eea1338e6f8fc582/bench/run_fed_leader_ws.py), lines 800–838, reads env.json, lscpu.txt and sample.jsonl, but **never reads exits.txt**. Complete-looking output before failure is enough to pass. The separately collected top-level env.json is not a substitute for checking these later acquisitions.

Research-only exit_probe.py constructs an otherwise positive preflight directory and changes only the env collector's archived status. It invokes the real preflight function, including file loading and report writing:

| Archived collector exit | Checker exit | Verdict |
|---|---|---|
| 0 | 0 | PASS |
| 17 | 0 | PASS |
| 124 (timeout status) | 0 | PASS |
| 137 (killed-process status) | 0 | PASS |

These are **synthetic acquisition records**, not an observed live collector crash. A process can flush a complete record and then fail; a sample command can print its expected records before hanging. The shell carries those records onward and the checker cannot distinguish them from successful bounded acquisition. This is the same producer/consumer evidence distinction now correctly fixed for timed children.

**Minimal closure:** retain structured exit results for each required preflight collector and require successful completion; reject missing, malformed, duplicate or nonzero required results, including timeout/signal outcomes. Keep all raw output and failure status in the refused bundle. Add an end-to-end recipe stand-in that prints the complete positive fixture then exits nonzero (and a successful control), demonstrating no timed run follows. No threshold, workload or repeat-policy change is needed. Optional post-run diagnostic failure remains a separate descriptive question; it must not rewrite a valid all-pairs timing result.

## What is closed

1. **Quota evidence.** examples/ab.rs facts::cpu_quota maps process membership and the matching mount, traverses every level, takes the minimum limit and refuses unreadable/malformed/ambiguous evidence. v2 requires the true-root cgroup.type absence check; v1 requires quota/period at each level and the root-only release_agent marker. Unknown, limited and unlimited remain distinct. Admission refuses legacy null, missing state, unknown, any finite limit (including 20), and unlimited without hierarchy provenance. This closes the prior leaf/fixed-path and unknown-null conflation within the documented stable-kernel-snapshot scope.
2. **Identity and settings.** Affinity and Cpus_allowed_list must agree; twenty distinct IDs and ten core groups are bound to the sample keys. Per-policy settings and exact coverage replace the weaker aggregate check. The four original mutations now all refuse; the unchanged positive control passes.
3. **Passive path.** facts::live uses Sysfs::env_json with an explicit not-probed perf result, plus filesystem/affinity reads. It does not call the active env path. Both env calls use --facts; env/lscpu/sample acquisition is bounded with TERM at 30 s and KILL five seconds later. Native witness tests include an active positive control; their stated scope is appropriately limited to ab's own paths. The remaining defect concerns the handling of failure, not passivity or the existence of a timeout.
4. **Child termination.** ab run now derives Term from the reaped status and emits termination; the Python record check requires both successful termination and exit zero. The Rust producer test explicitly drives output-before-exit17, output-before-SIGKILL, success and empty-output children. Source inspection closes my earlier information-loss objection for this PR. This does not certify that another branch has received the change.

## Scientific design and qualifications

R11 retains descriptive endpoint counts and raw readings/read delays, distinct diagnostic completeness, exact prospective settings, and a preflight after build/corpus without adaptive settling. The lag-1 formula implements the stated centered numerator/denominator, with constant and missing series undefined. None licenses state-conditioned ratios, deletion, a new significance claim or effective-sample-size substitution.

The mechanism correction matters: nine unpinned team workers do not imply nine distinct physical cores, and changes to helpers/traffic/overlap can improve a helper-bound run. Completion order alone is not the counterfactual benefit of removing MTF. The fidelity SD bound at a ratio centered at one is correctly approximately 0.0423 from log(1.02)*sqrt(20)/2.093; the interval itself is the criterion.

The disclosed prospective second attempt, separate first outcome, no pooling and no third attempt address the scientific interpretation problem. A per-attempt interval is not a campaign-wide error guarantee. Those qualifications and the unchanged all-pairs rule should survive in the eventual report. My authenticated pin still reserves the relevant repeat policy; the requested re-pin is not an enacted re-pin. I do not certify amended C6 authority. **It is not the only remaining issue:** acquisition-status admission above is technical.

## Verification and value

- 33 workstation-admission tests and 25 record-check tests pass locally on files fetched at the exact SHA.
- The original probe was copied unchanged beside those new modules to ensure Python imports the reviewed revision: positive PASS, four mutations REFUSED. An earlier invocation from the old script directory imported old modules; it was not used as evidence about R11.
- Four new acquisition-status witnesses reproduce the PASS behavior above.
- Fresh GitHub check-run metadata at this SHA: 24 success, 3 skipped, none failed or pending. This is metadata verification, not a claim to have rerun native Rust or the recipe locally.
- Source and native-test inspection only for quota discovery, passivity and producer termination. No native benchmark, workstation run, implementation or merge.
- Migration gate passed. Routing audit: five resolved historical findings, zero unresolved; publication route #145.

**C3:** expected closure of R10's admission and passive-acquisition gaps; actual revision closes the earlier malformed-fact witnesses and active path, but leaves collector failure disconnected from admission. The learned mechanism is now precise: bounded acquisition and valid-looking payload do not imply successful acquisition. One small end-to-end failure witness and a status check have greater immediate value than new timing or a broader experiment. Re-review that narrow closure at its exact source; do not reopen the statistical design.
