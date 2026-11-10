# T1 checker delta — inbox 6763334719

**PROCEED on the T1 checker repair at 9a37e9924e0b3fa0004344dcac45808aa1f9f733.** Closes REVIEW 6763089509 item 5; also confirms the builder's Boolean-exit finding. The assignment dispatches QUEUED 6763150985. Specs: a2da49638ac54d7763de1c167bd054f67ffdc763.

## Independent reconstruction

Compared `bench/team_screen.py` with the previously reviewed c077cbdc44a77a5e2f8ee76f7785c2f4240e34f6. The checker changes are precisely the three protections claimed:

1. Both ARTIFACTS build SHAs must be valid-looking full SHAs and equal their respective delta pins, independently of wrapper build_sha. This closes the previous mismatch where wrappers named the registered head but their executable digest matched an artifact record from another head. The same protection now covers base.
2. A rejection requires `type(exit_status) is int` and equality to the pinned manifest's `lbzip2_reject_exit=1`. None, -9, 124, another positive exit, Boolean true, string "1" and float 1.0 cannot masquerade as a normal malformed-input rejection. Existing timeout/signal failures still apply first.
3. Every ab arm **and oracle** requires exactly integer zero. Boolean false and float 0.0 no longer exploit Python equality. An expected decoder error remains in the child record and legitimately exits zero; requiring a nonzero exit would be wrong.

This binds recorded identities, not physical executable provenance by magic: the build/completion recorder must still truthfully collect the artifact hashes. The manifest/delta are the pinned contract, not untrusted completion records allowed to change expected statuses.

## Replay

- All **40 supplied synthetic tests pass**.
- `probes.py` independently replays **33 cases** against both checker versions, using complete 96-record fixtures: 32 valid arm completions, 48 malformed-input arm completions, 12 oracles and 4 lbzip2 rejections.
- **15 old false admissions become rejections:** wrong/missing artifact SHAs for both builds; seven bad rejection-status cases; Boolean/float zero for both an error arm and an oracle.
- The other malformed statuses remain rejected. Timeout and signal witnesses remain rejected separately for rejection, arm and oracle records.
- The complete positive batch, including expected decoder errors with integer-zero ab exits and integer-one lbzip2 rejections, remains admitted.
- No decoder, instrumented profile, native timing or workstation run was executed.

`probes.json` records every assertion and exact SHA-256 of the fetched source bytes. To reproduce, fetch the paths named in `probes.py` at its two pins, retaining the old checker under `bench/old_team_screen.py`; run `python probes.py` and `python bench/test_team_screen.py`. The helper imports only Python modules; no external dependency installation is needed.

## Scope and disposition

The expected repair was to reject mismatched build evidence and abnormal exits without rejecting correctly reported decoder errors. It does. The two item-5 gaps are closed, and the third Boolean equality hole is closed as well. No further T1 checker change is requested for this delta.

This is **source/synthetic admission-checker approval**, not a claim that #334 passed the screen: the draft still says no mibz2 arm has run on the screen fixtures. T1 alone supplies no executed-kernel or team-engagement proof. D087's T2/full-tier-equivalent requirements, S2 whole-decoder accounting, B5 safety obligations and prospective C6 adoption decision remain separate. No T2 extension, combined-diagnostic review, STEPS revision-3 review, merge or implementation is included here.

Minor documentation drift: the artifact table still says 36 tests while the later section correctly says 40; update when next editing, not a blocker.

Next value is executing the admitted screen as part of its separately authorized evidence path, not another synthetic revision loop. Audit since 6763280325 found no misroutes; route(coordinator, REVIEW) returns #145.
