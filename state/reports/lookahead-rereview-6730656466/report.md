# #271 admission re-review: HOLD, narrowed

Inbox **6730656466**; head **562e4ecfc16c7dd75eb62f9edc8800f89a62cd13**; freeze **de4952199cb3c62def68c717acce3ce5085e9ee6**. Spec/MODE/RULES/PROTOCOL pin **a4f97714cc14366984015869d9d094d89488ac57**. Re-review of my HOLD **6730484783**.

**HOLD admission on the remaining raw-evidence checks below, independently of D045.** The substantive H1 and H3 repairs are accepted; the child-status repair is accepted in source. The revised registration is materially better. No new timing arm, harsher performance minimum, six-complete-round gate or familywise criterion is requested. LA3, LA4 and N1 remain unchanged. No performance gap closed; this work removes ambiguity in the future evidence contract.

## Repairs accepted

- All **60 repository Python tests pass** at the exact head (ResourceWarnings for unclosed files, no test failures). Prepared counts are now checked in each complete round before aggregation, against 1 ≤ median ≤ max ≤ L+K−1 and 431 blocks. A single over-ceiling round cannot disappear into a valid median. The L3 positive fixture now uses four, L4 five. Aggregate labels and pass maximum are corrected.
- Occupancy now persists each OS status, including a negative signal on Unix, and fails the parent on nonzero status or child count mismatch. The report rejects nonzero/missing statuses and duplicate/missing records relative to its requested inventory. The shell requires both report status zero and JSON VALID. This closes the original source-level lost-status finding; I did not run native child-failure injection.
- Mechanism reporting intersects gaining and relieved arms. The former LA3-gain/LA4-relief witness is NOT CONCORDANT. Policy-effect, front-pool, idle-share and balanced-utilisation qualifications are recorded. These correct interpretation without changing the underlying experiment. Existing queue/admission comparisons are the relevant prior work; no novelty assumption is needed.
- GitHub's compare from freeze to head changes only `bench/service-lookahead.frozen`, which names the specified freeze. The change from the prior reviewed head affects report/test/recipe/occupancy output and documentation, not a new decoder implementation. Receipt and migration checks passed.

## Remaining counterexamples, executed

`probe.py` uses the revised repository's otherwise-valid 160-row test fixture; these are synthetic evidence mutations, not observed decoder faults or timing results. Every case below returned **VALID, exit 0, no failed checks**.

1. **The scheduled inventory is self-declared.** Set each occupancy `settings.rounds=1`, `children_expected=3`, retain round zero's off/timeline/full records, and set `complete_rounds=1`. All five passes are accepted with three children each instead of eighteen. `check_raw` accepts any positive integer request; it does not bind that request to the registered six. Require six requested rounds and eighteen accounted-for children per pass for a judged run. This preserves, rather than changes, the recipe. A separate control with **six requested / eighteen children / one complete timeline round** remains VALID. Do not require six complete rounds.

2. **Raw output/error evidence is not reconciled.** Change one off child's line from the common hash to `hash=BAD` and from `err=-` to `err=failed`, while leaving top-level `parity=true`, `output_bytes` and status zero. The report still accepts it. The repaired native parent should not generate that contradiction, but the report currently trusts the summary it is meant to verify. Recompute parity over every raw child's nonempty hash, registered output length and empty-error convention, then check the summaries. Keep failing raw lines. This is a bundle-validation finding, not a claim that the native parent ignores errors.

3. **Complete-round metrics can be impossible or discarded.** Change one complete timeline round's `t.svc.team.idle_prep_share` from .15 to **99**, leaving five at .15: median reconciliation accepts the impossible share. Set it to **NaN** instead: `check_raw` explicitly filters nonfinite values out before the median, and again accepts the report. Validate every required metric in each complete timeline round for presence, finiteness and its domain before reconciling. A round labelled complete cannot silently contribute to `complete_rounds` while contributing no value to a required metric. Preserve the intended handling of genuinely incomplete rounds and the ≥1-complete rule.

The first hole loses scheduled evidence; the latter two let contradictory raw evidence survive a successful summary check. Repair them in the existing validator, retain these witnesses and the genuine single-complete-round control, and identify the new freeze. No numerical acceptance criteria change is needed.

## The two uncertainties in the assignment

**431 blocks:** a corpus manifest need not contain every derived property. An explicit expected scanner count from the prior validated trace is reasonable when tied to the registered corpus digest and the same definition of a block. The report already pins the corpus digest. Record that derivation and preserve the supporting trace; do not pretend the number is read from `wikidata.json`. I did not rescan the corpus in this review. The constant is not the reason for HOLD.

**Reconciliation tolerance:** the implementation uses `1e-6 * max(1, abs(m))`, thus absolute tolerance 1e-6 below one, relative above one. It is not purely relative. That is a reasonable floating-summary reconciliation allowance, separate from invariant validation. It must never forgive an impossible raw share, missing metric, or over-ceiling count. Integer counts/inventory admit exact comparison; aggregate medians can be half-integral. No arbitrary tighter floating tolerance is requested.

## Evidence and next value

At inspection, 27 CI checks: **23 successful, three skipped, Miri in progress**. This is status evidence, not an audit of all job logs. The asserted untimed dry run was read from the assignment, not independently replayed. It establishes no performance result; an INVALID dry run is expected only because its dry marker/unregistered timing matrix makes it ineligible, not because contradictory trace evidence is acceptable.

After these bounded repairs and applicable D045 closure, the existing registered experiment remains useful: it compares higher-L policy effects directly, with arm-specific trace consistency, without conflating them with identified supply mediation. No native build, timing, Miri, fuzz, workstation run, implementation, dispatch, merge, queue or automation change was made.

Reproduce: fetch the `bench` files listed in `validation.json` from the source head above into `bench/` beside `probe.py`; run `python3 -W ignore::ResourceWarning -m unittest discover -s bench -p test_service_lookahead.py`, then `python3 -W ignore::ResourceWarning probe.py`. `probe.json` records every observed verdict. Routing audit: five resolved historical misroutes, none unresolved.
