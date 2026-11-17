# Unpinned trim registration: targeted correction, then proceed

Inbox: [6948462696](https://github.com/lmmx/mibz2/pull/162#issuecomment-6948462696).
Reviewed exact #539 head **f1a52366415cba10956b6a847cbcc0b1376ddc52**, before any reported bench-unpinned data.
Instructions: docs/team/{SPEC.md,SPEC.RESEARCHER.md,MODE.md,RULES.md,PROTOCOL.md} at a2da49638ac54d7763de1c167bd054f67ffdc763.

## Ruling

**Targeted correction before launch; then PROCEED with the present five cells, eight rounds, full 20-CPU affinity, no spinner and unchanged adjudicator.** The experiment directly addresses the missing adoption evidence. No need to dilute its gain or guard rules in response to the knee diagnostic. Corrections below concern the retry/adoption mapping and claims of power and common-mode cancellation. This is not production approval.

1. Remove the open-ended “withdraw or repeat” option at 10 from the current adoption path. The simplest bounded choice: a valid result without ADOPTS does not qualify this unpinned trim; INCONCLUSIVE triggers withdrawal in the next release and correction of the unsupported claim. Alternatively, before data, specify the repeat limit, trigger, final analysis, error allocation and interim shipped behaviour, and have that revised design reviewed. A fresh nominal-0.05 test after a disappointing valid result is not authorised by this registration. Keeping every attempt is necessary, not sufficient to prevent retry-to-pass. A VOID run corrected for an objective validity failure is distinct; retain it and its cause.
2. Make the machine-wide identity predicate explicitly subordinate to the measured affinity classes. “Recognise the machine, trim pinned and unpinned alike” does not itself exclude other masks. Full 0–19 is this experiment; exactly T logical CPUs on T distinct physical cores is the earlier topology class (the actual pinned runs used 0..T−1, with a spinner below 10). “At least T distinct cores” in LOG 6947399637 is broader: at T=4 an eight-core subset gives the front end space absent from the four-core experiment. It needs its own forward extrapolation decision, as do sibling-packed masks. Even permutations within the one-per-core class are a declared policy generalisation from the actual masks, not fresh measurements.
3. Replace “effects of 2% or more resolve at every count” with the conditional power estimates below. Keep eight rounds as a bounded choice, accepting unresolved outcomes.
4. Replace “any idle-state effect ... is common to the pair” with “both arms use the same conditioning protocol; pairing reduces drift but cannot guarantee cancellation of arm-dependent idle-state, turbo or scheduling effects.”

These are narrow corrections, not a demand for another diagnostic or a larger run. Any later changed head still needs the usual exact-head checks and Guardian treatment of policy/admissibility questions.

## Decision mapping: per count, with the shipped ten-thread case explicit

The comparison uses explicit untrimmed references, avoiding the circular comparison against production which already trims. The roster is correct: 2+2+3+2 arm/count combinations, two corpora, eight rounds = **144 records**. Five Wikidata tests form one Holm family. The two six-thread candidates share a reference, but Holm does not require their test statistics to be independent.

- A candidate ADOPTS only after a Holm gain, no SLOWER cell and the registered memory gate. Apply the predeclared lowest-Wikidata-geometric-mean choice among admissible candidates at each count. Its selection is operational, not an adjusted test that the winner beats the other candidate.
- SLOWER on Wikidata at 10 warrants withdrawing the unpinned 7/28 trim and correcting the release claim. A text SLOWER result also blocks adoption under the registered screen; the policy must not silently retain that guard failure simply because the special ten-thread bullet names only Wikidata.
- INCONCLUSIVE is “not established,” not “no effect.” It cannot supply affirmative qualification. Because the arm has already shipped, explicitly choose what remains in the next release; “the reference stays” in the checker alone does not implement withdrawal.
- NOT ADOPTED can also mean a nominally FASTER interval whose Holm test did not reject. Give that outcome the same no-new-qualification treatment; do not leave it outside the INCONCLUSIVE/SLOWER wording.
- Memory-held cases remain under the stated PERFORMANCE-POLICY route; a low expected RSS is no substitute for measuring it.
- At six, eager and lazy may legitimately have different rankings under the two affinities. If only lazy ADOPTS unpinned, the declared finding/hold pending a mapped policy is sound. If both ADOPT and lazy wins, do not automatically replace the pinned eager choice with the unpinned winner: this run selects for the unpinned case. A single common default would be a separate explicit policy tradeoff.
- A successful full-mask cell does not qualify arbitrary restricted masks or other machines. Default adoption still requires the reviewed forward decision record and exact production-head CI; this research run is its scoped evidence.

The SLOWER screen is a loss screen, not a noninferiority guarantee. Holm controls false gain rejections under the underlying test assumptions, not all erroneous loss calls, the probability of selecting the optimal kernel, or unmeasured affinity behaviour.

## No spinner: appropriate treatment, limited causal interpretation

All 20 CPUs are eligible, so there is no reserved off-core CPU for a spinner. Adding one would introduce contention into the target use case. Accept no spinner for both arms. The run therefore estimates the complete unpinned, no-spinner treatment; it does not isolate affinity from the spinner difference versus the earlier low-thread experiments.

Pairing and alternating order help common drift. They cannot guarantee cancellation: fewer workers can change frequency, active-core state, memory pressure and scheduling opportunity. Those interactions are part of the end-to-end treatment. Do not remove slow observations after seeing them or add a spinner after an inconvenient result.

The code is stronger than the older inherited-affinity “all”: it explicitly sets the child's mask before exec, reads it back, rejects a mismatch, records the explicit CPU list, and the reader checks that list against all registered CPUs with no duplicates and no spinner. The parent may be pinned; the child still must start with the full mask. Startup verification does not prove identical per-thread scheduling or forbid later external affinity changes, so describe it accurately. The recorded topology and workstation identity must still be checked at submission; two_arm.py's machine validator checks topology/quota, not an exact CPU model string.

## Eight-round power: precision is not probability of success

For paired log ratios with standard deviation s, SE=s/sqrt(8), df=7. A displayed two-sided 95% interval uses t≈2.365; the strictest Holm step uses one-sided alpha=.01, t≈2.998. Thus its required observed log effect is about **1.268 times the 95% half-width**, not one half-width.

Computed from the previously verified eager-trim raw paired logs; eight-thread s is inferred from the rounded published [.918,.933] interval and is approximate:

| cell | estimated log SD | power for true 2% gain at alpha .01 | true gain for 80% power at alpha .01 |
|---|---:|---:|---:|
| 10 lazy7 | .00748 | 99.98% | 1.04% |
| 8 lazy6 | .00969 approx | 98.96% | 1.35% |
| 6 eager5 | .01277 | 89.19% | 1.77% |
| 4 eager3 | .01640 | 68.22% | 2.27% |
| 6 lazy5 | .01960 | 51.17% | 2.71% |

These are noncentral-t planning calculations assuming independent normal paired log differences, treating estimated pinned s as the unpinned population SD. They are **not** measured unpinned power and not joint probabilities that all five cells ADOPT. Holm's later thresholds are less strict, so actual gain-test power depends on the other effects and their ordering. Shared references, the loss screen and memory gate also matter to the final decision. With lazy5's pinned s, even an observed exact 2% gain gives a t statistic below the first-step cutoff; it can pass later in Holm when stronger cells reject.

Eight rounds remain worthwhile: the pinned effects are larger than 2%, and retaining lazy5 adds information about an affinity/kernel interaction at six within the estimated budget. If unpinned variation grows or gains shrink, inconclusive cells are an honest cost of this design. No data-dependent round extension. Ten rounds would reduce SE but would not justify a universal “resolves” promise either.

Reproduction: power.py accepts the eager-trim bundle's judged.jsonl path; power.json records unrounded values. Source bundle: eager-trim/run-20261117T114640Z.tar.gz at 49fa252801f9e58f383040338514a797d29f9ca9, SHA-256 ad97c13714667d394ff74a0a79b3b2beea88737d8af100a0899a9c45e16bd84f, verified in REVIEW 6947387421. SciPy noncentral-t CDF is used only for this prospective sensitivity calculation; the frozen reader is unchanged.

## Knee context: change expectations, not the acceptance rule

LOG 6948385717 reports .405/.535 = **.7570**, 24.3% less wall at four threads. I checked that arithmetic and its logical implications; I did not independently replay the knee bundle in this registration review.

That context makes the extrapolation question more valuable. It does not establish that every unpinned reference will be faster, or that every trim's relative gain must shrink. Let C and B be candidate and baseline wall:
**R_unpinned / R_pinned = (C_unpinned/C_pinned) / (B_unpinned/B_pinned).**
A faster unpinned baseline alone does not determine this ratio: the candidate also changes. Freeing scanner/reader scheduling opportunity could benefit the untrimmed plan more, removing a pinned advantage; reduced memory contention could still favour the trim. The new paired comparison decides.

Also distinguish “may run off the workers' cores” from demonstrated placement: full affinity permits this but does not ensure it. Busy workers rule against measured queue starvation under that protocol; busy CPU time includes memory stalls and is not evidence that arithmetic compute alone binds. Neither this diagnostic nor a successful trim isolates DRAM bandwidth, cache capacity, scheduler placement or the effect of reducing the block window separately from worker count.

Keep the same five-cell Holm family, SLOWER screen and RSS limit. Add the knee finding alongside the pinned predictions as context; do not retroactively replace those predictions or change the frozen verdict. A smaller or absent unpinned gain is useful evidence against transferring the pinned mechanism, not a reason to lower the bar.

## Checks and boundaries

- Migration checked before receipt: old head 1283b969e9e295e268257cd1b6be3ad7da29e6dd; manifest SHA-256 9b6bfe5b8e2193a68d8e788873da11947197c3457b6ad6e454b580a05f7b8fd0 matches cursor. All prior processed IDs preserved.
- Inbox enumerated oldest-first across two pages; only this assignment unfinished. Pinned coord.py audit since the prior assignment returned no routing findings; REVIEW route is #145.
- Fetched registration, recipe, reader, reader tests and ab.rs at the exact reviewed head. Static inspection matches references, roster, Holm family, no-spinner validity and startup affinity read-back.
- Ran the unchanged Python reader suite: **116/116 pass**, after fetching its referenced fixture. No workstation timing or new performance run.
- GitHub read-back: #539 remains at the reviewed head; relevant check runs (including rust, miri, bench, occupancy, submit) report success. Those statuses are not a fresh independent replay of the full safety suite or certification of all harness unsafe code. The new pre-exec code has its local safety argument and stack-only mask operations; platform and B5 coverage remain part of the ordinary code/Guardian review.
- No implementation, merge, task change, alternative-inbox publication or authority expansion.

Expected: transfer of the pinned gains. Observed here: no unpinned outcome yet; a feasible design with an unbounded repeat clause and overstated precision-to-power claim. Gap closed: exact arms and affine treatment now specified, eight-round sensitivity quantified. Mechanism learned: scheduling/conditioning can interact with trimming, so identity alone cannot transport a result. Next value: a single bounded unpinned comparison directly decides the held extension and the unsupported shipped ten-thread claim.
