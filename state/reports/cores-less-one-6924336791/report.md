# Cores less one: registration review

**REVISE — one targeted correction: restrict this adoption decision to the measured i9 execution profile, including the window change.** One i9 run can decide the nine-versus-ten-worker production change on that profile. It cannot qualify the distinct eight/nine-core changes currently reached by the patch. Keep the wider policy as a separately registered extension; do not require a multi-machine programme before making the narrow integration decision.

Assignment [6924336791](https://github.com/lmmx/mibz2/pull/162#issuecomment-6924336791); rule [6923522255](https://github.com/lmmx/mibz2/pull/145#issuecomment-6923522255).
Reviewed **c7c11c73244dc763b7c46c0562178604161f77e0**, baseline **4ffe2bcbef1d4296e7d7bdf35ba3972c03edd6e3**. SPEC pin a2da49638ac54d7763de1c167bd054f67ffdc763. No decoder timing, production changes or merge.

## What the source actually changes

The production-arm mismatch from the stop-walk review is removed: this recipe runs mibz2 from both builds, using production's BzReader::parallel. The compared decoder change is in src/parallel.rs. This is a joint worker-count/window treatment, not a pure worker-count test.

The new workers_past_the_cores deliberately keeps eight workers on eight physical cores, avoiding a switch to the eager kernel. That is sound, but **the window still changes there**. plan_with reduces nine-core machines to eight workers and ten-core machines to nine. The window rule scales every Workers(n) plan with n<requested threads, not just a named CPU model.

Independent arithmetic for ordinary slots=4×requested:

| physical cores | requested | old→new workers | old→new slots |
|---:|---:|---:|---:|
| 8 | 16 | 8→8 | 64→32 |
| 9 | 18 | 9→8 | 72→32 |
| 10 | 12 | 10→9 | 48→36 |
| 10 | 16 | 10→9 | 64→36 |
| 10 | 20 | 10→9 | 80→36 |

Thus “eight cores keep eight” is not inactivity. Nor do core count alone, a different LLC capacity, SMT configuration, memory subsystem or an affinity-limited view of a larger machine reproduce the i9's resource balance. The gain could arise from reduced in-flight storage as well as one fewer worker. The nine-core transition is a different worker-count contrast.

**Concrete correction:** gate both changed count and scaled window to the intended measured hardware/affinity profile, retain old behaviour elsewhere, and archive deterministic route checks. Restrict the registered ADOPTS statement accordingly. At minimum the run must establish the intended ten-core i9 topology and allowed CPUs; on a nine-core host the declared 10-thread control would itself be a treatment. If a wider deployment is desired later, register it separately. Merely checking that the lazy kernel stays selected is not a performance guard.

No additional CPU experiment is needed to answer the narrow i9 question. An implementation restriction must still be reflected in the exact frozen build; this report does not grant a pass to the broader current patch.

## Controls: keep them as guards, not calibration

The controls are useful: at 9/10 requested threads on the intended i9, the plan and window are unchanged. A resolved difference is evidence against interpreting the two builds as exchangeable under that run. But the statement “identical decoders differing means the harness moved” is too strong: chance, build/code-layout effects, environment and timing variation remain possibilities. It is an alarm, not diagnosis.

Predeclared VOID is acceptable. **Keep all records and intervals.** VOID declines an adoption verdict; it need not discard information. Do not divide judged ratios by control ratios: that would assume one multiplicative nuisance effect transfers across thread counts and spinner states, with additional covariance that this registration does not model. Such calibration could erase a real loss.

Four unadjusted two-sided95% controls can flag by chance: under independent exact null tests, P(any flag)=1−0.95^4=**18.549375%**. Dependence changes this value. It is a transparent false-VOID cost, not evidence the rule is invalid. No automatic rerun-until-controls-pass. Archive the first result; any subsequent attempt needs its own recorded reason and admission, preserving the original outcome. Holm's unconditional gain-family protection does not turn post-selection confidence statements into conditional95% claims.

## Spinner-free judged cells

The recipe holds only through requested9, and the shared-core check rejects reported overlap. At12/16/20 the decoder sees no held spinner in either arm. This is an appropriate paired production comparison; a free core left by fewer workers is part of the treatment, not a confound to cancel.

Do not pool the old forced-placement ratios into this estimate or impose their ~4.3% gain as an observed result here. That run used different forced arms and eight copies; this uses two production builds and four copies plus text. I read LOG6913804367 as motivation, not replayed bundle evidence. Even identical spinner status would not make those estimands identical. Fresh paired cells decide; a difference in outcome would not invalidate either correctly scoped result.

Idle-state effects may still exist without a spinner. Controls and recorded placement help detect some failures; quiet controls do not prove the absence of every thread-specific nuisance. No extra causal study is required before this narrow comparison.

## Statistical and evidence checks

The new design is materially stronger than the prior stop-walk registration: one primary candidate, a three-cell Wikidata Holm gain family, held-out text loss/memory guards, corpus in the record key, full roster, and explicit INCONCLUSIVE language. Holm implementation is correct in the inspected step-down procedure. At R=10, the log-t computation uses df9; validity remains conditional on the usual paired-log sampling assumptions. The loss screen is explicitly only “no resolved loss,” not proof of non-inferiority, consistent with the stated pragmatic rule.

I ran **26/26 supplied Python checks**, including rejection of my earlier fabricated bundle both unchanged and at its own two rounds. Independent checks compared t_cdf at df1 to its analytic Cauchy formula (maximum absolute error 2.22e−16) and tested Holm boundaries. These are untimed synthetic/mathematical checks, not decoder or workstation measurements.

The defaults generate 2 corpora ×2 arms ×5 counts ×10 rounds=200 records. RSS is a ratio of medians of per-run peaks, gated at every judged corpus/count. A memory justification applies to measured growth, not missing evidence.

Two execution qualifications must be enforced; they do not require another statistical review:
- The recipe checks the candidate's tracked cleanliness and the baseline worktree HEAD, but **does not check the baseline worktree's cleanliness**. git checkout --detach can preserve local modifications. Require a clean baseline tree before building; binary hashes identify bytes, not correspondence to the stated source. The ancestor test likewise does not prove the entire source delta; the exact compared head must stay pinned.
- two_arm.py rejects a spinner object with overlap=true but does not reject missing/malformed spinner metadata. Its integer conversion also coerces fractional thread/round values. For the unmodified known harness, the raw fields are evidence of the declared treatment, not an arbitrary-input security boundary. Independently verify masks/topology/held-spinner records before admission; do not claim the script proves a free core from absence of an overlap flag. These limits do not undo the successful roster repairs.

## Outcome and next action

Expected: a useful production gain from relieving worker/service contention and reducing in-flight storage. Observed here: source and adjudicator correspondence substantially improved; broader hardware effects remain untested. No empirical speed gap closed in this review. The mechanistic distinction is that worker count and window change jointly, including a window-only effect on eight-core hosts. The next bounded i9 run has direct integration value after scope is narrowed; further generalisation is not a prerequisite.

Research evidence: exact-head recipe, parallel.rs, two_arm.py, test_two_arm.py, examples/ab.rs, baseline-to-head comparison, registration and motivational LOG. Migration verified; inbox/cursor and routing audit passed. The attached independent.py/independent.json and tests.txt record the checks. No safety or CI certification is claimed.
