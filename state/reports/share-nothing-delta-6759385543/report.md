# Share-nothing pricing delta: PROCEED to freeze the diagnostic

Inbox **6759385543**; PR #326 at **482df588e2a97b99c6797c7bc2a24098015968dd**. Prior review: 6758300190 against aae6dc5ed4779c094f1af932a384204f1933a25b. Instructions fetched at a2da49638ac54d7763de1c167bd054f67ffdc763.

**PROCEED to freeze this exact diagnostic design.** The prior material objections are resolved in the inspected delta. This is not production qualification, an adoption ruling, or permission for an agent to time the workstation. The Coordinator still owns freeze/queue admission; Louis runs the registered recipe under C5.

## What changed, and why it is enough for this question

The registration and judge now agree on the narrow estimand: G2 and G9 are within-round CPU10/CPU4 geometric means; R is their level × worker interaction. R spanning unity is INCONCLUSIVE, G9 containing 3.50 is compatibility only, and the recompression proxy does not isolate table capacity or identify finder-fed contention. The historical 3.76/G2 model carries its untested transfer assumptions. No kernel budget comes from the run; the 1.44× common gate is withdrawn and D4's independent filters remain unchanged.

All-round analysis and fixed block-1/5/10 sensitivity remain. Class-selected bootstrap intervals are gone: the sentinel classes have counts and point estimates only. The chronological table, actual round duration, waits and unread telemetry are retained. The intervals explicitly disclaim guaranteed 95% coverage under longer persistence or attempt-wide shocks. That is acceptable here because this is a fixed-budget descriptive diagnostic, with no production acceptance or design-closure gate reading its outcome. It would not resolve the separate formal NI coverage objection on #316.

Identity now requires well-formed head/frozen identifiers and present matching source digests, with the live wrapper checking measured inputs against the frozen commit. Present corpus and executable digests replace None==None admission. Preflight files are rejudged and the verdict must agree with the recorded exit. Empty/reasonless telemetry rejects; explicit unavailable readings remain separate from observed deviations. Runs and slots must follow the registered sequence and each timestamp series must be ordered. An independent whole-round swap rejects before analysis.

These are archive consistency checks, not cryptographic attestation of execution. External bundle identity and immutable submission remain necessary. No claim here that a fabricated bundle cannot mimic valid facts.

## The two requested answers

### 1. Gate stalls: acceptable bounded operational risk, not a statistical result

Yes. A slot stalled by the unchanged idle gate reaches its fixed 120-second bound, stops the drive and invalidates the whole attempt. Partial cells cannot become the primary diagnostic, and another attempt cannot silently replace it. The 5400-second drive bound also stays.

This risks losing information, not silently selecting a favourable ratio *within an admitted attempt*. Completion itself can select quieter environmental episodes, so even a successful result remains conditional on this harness and a completed attempt. Do not turn that into an unconditional workstation scaling claim.

There are 732 slots. As a sensitivity calculation only, if each independently stalls with probability p, failure probability is 1-(1-p)^732: about 51.9% at p=.001, 7.06% at .0001 and 0.729% at .00001. Real stalls can cluster; these are not probability estimates. The whole-drive budget averages 7.38 seconds per slot including waits, so many sub-120-second waits can invalidate the run too. The shared-VM DRYs do not estimate workstation risk.

No timeout increase or gate relaxation is needed to approve this bounded diagnostic. A stall returns the information that this harness could not complete under that load; preserve it as INVALID, not NO GAIN. Any later method change needs a separately identified prospective registration, preserving this attempt and the no-retry-to-pass rule.

### 2. Namespace limit: sufficient as a visible-path claim

Yes, with exactly the scope §5 already gives it. Missing mount/path, a delegated controller without cpu.max, unreadable cpu.max and v1 CPU control return unknown/refuse in the replayed cases. A properly read visible limit returns limited.

The collector cannot prove no quota above its namespace/mount view, nor that cgroup placement/settings remain unchanged throughout the run. The kernel documents namespace-relative paths and hierarchical ancestor restrictions:
https://www.kernel.org/doc/html/v6.12/admin-guide/cgroup-v2.html#namespace
https://www.kernel.org/doc/html/v6.12/admin-guide/cgroup-v2.html#delegation

Thus “unlimited” in this artifact means no finite limit found on the checked visible path at preflight, not host-wide or permanent absence of throttling. The text's “cannot be read by anyone” is overbroad literally: a sufficiently privileged host-side observer can see ancestors. Its intended inside-this-view limitation is clear and is not a freeze blocker. Carry the visible-path wording into the result. A known hidden restriction would have to be disclosed; this review does not establish a bare-metal, globally unthrottled environment.

## Independent verification

Downloaded the exact judge, tests, registration, recipe, bounded supervisor and finaliser. Compared the prior reviewed head to this head; unrelated role/decision changes in that comparison were not adopted as new authority.

Standard-library replay against the downloaded implementation, using AST-extracted fixture builders from its tests:

- Clean full 732-slot synthetic record admitted.
- All **32** malformed-evidence mutations rejected, including all prior missing-digest, telemetry and identity witnesses.
- Missing archived preflight and a fabricated PASS over REFUSED preflight rejected.
- All **8** quota fixture checks passed: visible unlimited/limited, missing mount/path/directory, delegated missing file, unreadable file, v1 CPU, and restricted mount root.
- Full-round swap rejected.
- Original R counterexample reproduced: 0.9797958971, interval [0.8793827605, 1.0843224043], now **INCONCLUSIVE**.
- Analysis retained 60 chronological rounds; mode-class outputs had no intervals.

The probe calls the actual admission and analysis functions; it does not substitute a judge. It uses synthetic identities and facts, not measured performance. pytest is unavailable in both local Python installations: **no independent 91-test suite pass claimed**. Fresh GitHub check-runs at this head show completed successes, including submit/rust/bench, with release/publication jobs skipped. That is CI status inspection, not independent execution.

Read #326 comment **6759366302**: it reports two retained INVALID idle-gate DRYs and one end-to-end DRY with refusal of the VM's preflight and all 36 DRY slots completed. The latest record used the registered idle gate. I inspected that report, not its raw bundle, and did not rerun a decoder, compressor or timed recipe. A DRY that bypasses failed host preflight remains plumbing evidence only. The reported magic counts give 1831/421 = 4.349; “about 4.5” is harmless context, and magic counting is not structural proof against accidental matches.

The unchanged process-group supervisor/finaliser is reused, not recertified for arbitrary escaping descendants or abrupt supervisor death. No new safety claim is inferred from these admission tests.

## Disposition and next value

Expected: repair the diagnostic's interpretation and missing-evidence admission. Achieved: those repairs, independently replayed; no performance gain measured and no production gap claimed closed. The next experiment's value is a current, paired workload-scaling description to replace speculative transfer from an old ladder. A favourable result motivates a concrete whole-cost candidate, while an inconclusive or contrary interaction does not close share-nothing.

Freeze the reviewed inputs, preserve all attempts, and let the Coordinator apply the remaining queue requirements. No further scientific revision requested in this scope. Migration head/digest verified; wake routing audit clean. Research only, no implementation, merge, dispatch or automation change.
