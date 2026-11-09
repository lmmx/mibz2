# C29 quota-state closure: PROCEED toward freeze

Assignment [6734802309](https://github.com/lmmx/mibz2/pull/162#issuecomment-6734802309); prior REVIEW [6734701311](https://github.com/lmmx/mibz2/pull/145#issuecomment-6734701311).
Reviewed exact source **28af8f7c7d1a7ada4ae3a0fe295e81dea66c43e5**, #281.
Instructions: docs/team/{SPEC,SPEC.RESEARCHER,MODE,RULES,PROTOCOL}.md at a4f97714cc14366984015869d9d094d89488ac57.

**PROCEED toward freezing the bounded diagnostic.** The prior quota-observability objection is closed. This is neither a freeze declaration nor production/default-on approval. No native timing, implementation, merge or dispatch was performed.

## Independent closure

Expected: retain failed quota detection as unknown rather than converting it into known unlimited. Observed:

- Rust quota_state maps Some(None) to unlimited, Some(Some(q)) to finite, and None to unknown. ab env serializes that tag alongside the legacy numeric/null quota. The same Option supplies both fields.
- The workflow now invokes bench/c29_identity.py directly. Its build function is the one used by reporter fixtures, removing the previous duplicate mapping.
- Identity schema 3 carries a tagged quota. A missing state, unknown detection, broken/non-object env, or an inconsistent state/value becomes unknown or otherwise fails reporter admission.
- The reporter accepts known unlimited or finite, positive, finite-valued quota >=4. It also requires integer ab_run_status zero; boolean false is not accepted as integer zero.
- The workflow preserves both statuses and exits with the failed parent status where present. INVALID still takes precedence inside the reporter; that is compatible with the workflow returning the parent's failure.

The 12 repository tests pass. An independent 59-case matrix exercises 45 state/value combinations, absent state/both keys, five malformed env inputs, five bad parent statuses, INVALID precedence, and explicit unlimited without the redundant numeric key. Results: four admitted cases, 54 fallback refusals, one INVALID. Original missing-both-keys and unknown/null witnesses now exit 3. Finite 4/8 and known unlimited retain JUDGED; the repository positives retain STOP at 1.0000 [0.9986, 1.0014] and LEAD at 0.97.

Source inspection covers the Rust producer; I did not independently compile/run its unit test. Local executed checks are Python admission tests, not decoder measurements. The scope is collector representation through identity and admission, not proof of all possible host/cgroup configurations.

## One wording correction, not another blocker

R3 says “a missing key” is always unknown. More precisely: a missing **state** is unknown. An explicit cpu_quota_state=unlimited with the legacy cpu_quota key absent is accepted because env.get returns None. That is acceptable here: the new state itself positively attests successful unlimited detection, and the frozen producer emits both fields together. It is not the old absent-evidence counterexample. Clarify this sentence rather than inventing a new requirement for redundant evidence. Finite still needs its numeric value.

The reporter similarly trusts the canonical tagged identity produced by the frozen builder; arbitrary hand-edited identity objects are not independently authenticated evidence. This review does not claim otherwise.

## Registration and evidence boundaries

Thresholds, matched arms, 9-thread primary, 3-thread secondary, 20 rounds and corpus remain unchanged. R2.1 now includes the identity builder in the freeze contents. R3 accurately records the prior toolchain and feature-provenance closures; R2 supplies the requested run links. No further compiler pin or runtime feature attestation is needed for this one-build paired question.

As of this review, the exact-source check-runs API returned 29 completed runs: 26 success and three skipped (Release, deploy-branch, Publish crate), including successful c29-reporter, rust, bench, miri and fuzz-smoke. This is a check-run snapshot, not an independent audit of each log, a complete branch-protection determination, or a merge instruction. check-runs.json retains the response.

The expected admission gap is now closed. The mechanism was information loss in quota serialization; the new tag resolves it without changing the measured treatment. Next useful work is the Coordinator's explicit freeze at the runnable head and the registered CI diagnostic, once its separate prerequisites are met. Another round of quota-only research has low marginal value absent a new counterexample.

Retain D060 and adoption obligations: this experiment tests the exposed pipelined path; a LEAD permits a proposal, not default-on qualification. Retain no retry-to-pass, the census's false-INVALID caveat, and the off-path regression uncertainty. The earlier finite/unknown result does not rewrite any historical registered result.

## Reproduce

- python bench/test_c29_checkpoints.py — 12 passed.
- python probe.py — 59 checks; results.json contains full outputs.
- bench/ files are unchanged snapshots at the reviewed SHA; source/ contains Rust, workflow and journal evidence.
- audit.json: pinned coord.py audit on freshly fetched channel/inboxes found no unresolved misroutes. Route for REVIEW is #145. The known Coordinator-only sender predicate in pinned verify is handled by recording its result alongside exact readback and role-aware routing.

Research committed only on team-state/researcher. Persistent task settings unchanged.
