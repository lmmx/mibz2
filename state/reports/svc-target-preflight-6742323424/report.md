# Target-only preflight: narrow REVISE before freeze

Inbox [6742323424](https://github.com/lmmx/mibz2/pull/162#issuecomment-6742323424). Reviewed **7f6919290b645e9d0aeddfd2fc67d8e25746fe88**, #302. Specs a4f97714cc14366984015869d9d094d89488ac57.

**REVISE the strict record checker before freezing this entry.** The preflight-only design answers the intended question, is independent of the timed experiment, and closes the earlier success-bit-only problem. No new decoder safety objection. The remaining repair is small: its asserted complete/typed schema is not enforced. After that and green exact-head CI, proceed under the existing one-attempt queue conditions; this is not an approval of the timed registration or adoption.

## Reproduced admission failures

Ran the pinned checker suite: **7/7 pass**. Adjacent probe.py imports the pinned fixture and checker; it changes one property at a time. Execute beside those two files from the reviewed commit.

| Single change to the valid target fixture | Actual checker result |
|---|---|
| Positive fixture, unchanged | TARGET |
| Finite quota replaced by Infinity | TARGET |
| Same quota through JSON numeric literal 1e309 | TARGET |
| At9 plan reduced to {"kind":"teams"} | TARGET |
| Budget9 encoded as9.0 | TARGET |
| service_plan.teams encoded as2.0 | TARGET |
| At10 pooled count set to−1 | TARGET |
| Whole plan removed | INVALID RECORD |
| Quota Unknown, retaining claimed success | INVALID RECORD |

The first two quota cases are one defect through two encodings. Python json accepts 1e309 as infinity even without the nonstandard Infinity token. The checker requires positive quota but not finite quota. It then uses >=10, turning an invalid numeric observation into affirmative target evidence.

The shape checker checks only plan.kind, not the workers or teams/k payload. Budget order and service-plan equality use Python equality, so floats compare equal to the intended integer constants. Counts permit negative values. These contradict “every field present and of its type, booleans never numbers” and a whole-record TARGET. They are malformed-record witnesses, **not proof that Rust discovery emits infinity or negative usize values**, and not a safety finding against the production decoder.

**Closure:** require finite positive numeric quota for kind=cpus; strict integer budget and service-plan fields; complete typed plan variants; nonnegative service counts and positive built thread/window sizes where a decoder exists. Reject missing required data. Add each witness plus retained positive/non-target/failed controls to the checker tests. Check relevant semantic invariants rather than demanding top-level policy facts equal the separate topology snapshot: production service_target intentionally uses the latter, and the two observations need not be atomic.

The checker correctly rejects the prior only-ok, absent-snapshot and Unknown-quota success claims. It independently recomputes kernel/topology/quota eligibility and verifies the10-only predicate, svc2 at8 decoding threads/window8, completion and equal prepared/finished counts. The source service_target also has a compile-time Linux/x86-64 restriction: the JSON recomputation alone does not prove that fact; preserve executable/build platform provenance with the bundle. No new machine-brand gate.

## Classification and record qualifications

On a non-target fixture, decoded=false returns NOT TARGET with the decode failure in problems. That is fail-closed for timing eligibility, but not evidence of a clean fallback decode. Keep the problem conspicuous; do not summarise NOT TARGET as “only the host was wrong.” If the intended contract is that any decode failure outranks machine mismatch, make that precedence explicit before freeze. No need to change thresholds or open a timed attempt.

The Rust producer checks ended but not prepared==finished when assigning ok; the checker is stricter. A contradictory success therefore becomes INVALID RECORD, conservatively. This does not demonstrate an actual completion failure.

“Every outcome keeps every file” overstates the recipe. An unfrozen refusal occurs before frozen.txt/build.txt/inputs.txt/preflight.json; a crash cannot yield a valid checker record. Preserve the files that exist, with explicit outcome and reason; never synthesize successful evidence for stages not reached.

The raw directory captures shell-visible proc membership/mountinfo/status/cpuinfo, online/present and lscpu. It does **not** archive all per-CPU topology/cache and ancestor quota-file contents read by discovery. Thus the structured record can establish what discovery reported, but “what exactly discovery read” is not fully independently replayable from raw/. Either narrow that objective accurately or retain the relevant paths, values and read errors, especially for an Unknown diagnosis. This is diagnostic coverage, not permission to relax Unknown.

The recipe uses timeout120 without a kill-after grace. That normally terminates this executable, but is not an unconditional hard wall-clock cap for a process that ignores TERM. If “at most120s” is meant literally, bound termination escalation and test timeout packaging; otherwise state the timeout's termination semantics. Build time is outside the stated preflight bound.

## Recipe and CI evidence

Inspected the exact recipe's clean/frozen checks, Cargo-reported executable selection, pre/post executable digest, diagnostic capture, checker invocation and outcome mapping. It runs svc_target alone, no ab grid. Refusals preserve the frozen-input boundary; Unknown remains ineligible. The expected information is target reach, not speed.

The workflow includes both checker and recipe tests. Fresh exact-SHA check-runs snapshot on9November: **23 success,3 skipped,2 cancelled** (bench and macos-15/x86_64); no active checks. Submit succeeded. Consequently this head is not all-green; resolve cancellations through the existing incident process, without treating them as performance or source failures and without retry-to-pass.

Independent local recipe-suite execution was attempted against fetched exact files. It stopped at bash process substitution, /dev/fd/63 unavailable in this sandbox, before the substantive checks; the just-submit test skipped because just is not installed. This is an execution-environment limitation, not an independently reproduced recipe defect. I do not claim those local tests passed. The brief's real four-core NOT TARGET exercise remains attributed evidence; no native build or workstation run performed here.

## Decision and value

A frozen preflight-only entry is still worth doing: a cheap refusal can identify a discovery obstacle before a timed slot is spent. Fix the small admission holes; freeze the reviewed inputs; complete green exact-head CI, dry-run and exact ready-tag conditions; then one Louis-run/submitted attempt. Do not weaken discovery to obtain TARGET, or infer that TARGET persists until the later A/B. REFUSED/INCOMPLETE follow the registration's stated cause-fix rule, never silent retries; a substantive source change needs a new freeze.

C3: expected a whole-record reach decision; observed earlier contradictions rejected but incomplete/malformed records still accepted. The evidence-admission gap is reduced, no production-speed gap closed. Mechanism information remains actual discovery and natural default dispatch, not performance or class-wide qualification. Next value is the bounded target-only observation after these repairs; timed registration findings6742157313 and D067 adoption boundary remain separate.

Sources: exact-head registration, examples/svc_target.rs, bench/svc_target_check.py, fixtures/tests, .just/bench.just, src/parallel.rs and test workflow; exact-SHA GitHub check-runs. Migration verified at previous branch1283b969e9e295e268257cd1b6be3ad7da29e6dd and manifest9b6bfe5b8e2193a68d8e788873da11947197c3457b6ad6e454b580a05f7b8fd0. Channel audit: five resolved historical findings, zero unresolved. No implementation, merge, tag, queue edit or authority change.
