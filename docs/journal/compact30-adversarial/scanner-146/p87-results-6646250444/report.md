# P87 results: supported effect, unresolved phase sensitivity

Answers [inbox 6646250444](https://github.com/lmmx/mibz2/pull/162#issuecomment-6646250444). Spec pin: beb7df121fddf69938fdf8c972a8c7d964267ac6. Reviewed write-up and both archived result.json/env.json files at a995ce87, phase source at ee12bb9e, and ef28ad32…ee12bb9e comparison. No benchmark, implementation, merge, or gate amendment performed.

## 1. The numbers support a narrower sentence

Independent extraction from the archived reports confirms 21 parity-true rows per run, no wall interval wholly above 1, and these wikidata 4-thread head/base ratios:

| quantity | run 41940471497 | run 41941330350 |
|---|---|---|
| wall | .9776 [.9602,.9953] | .9894 [.9772,1.0019] |
| CPU | .9867 [.9708,1.0029] | .9929 [.9807,1.0053] |
| peak RSS | .9319 [.9212,.9428] | .9320 [.9162,.9480] |

Correct replacement: **“A 4-thread wikidata wall improvement was detected on the 8573C run; the 6975P-C estimate favored the candidate but remained inconclusive, compatible with up to +0.19% harm at the reported marginal interval. RSS was lower in both. Neither run resolved a wikidata 4-thread CPU change.”** “Not slower in the other” is only true of the point estimate, not an established effect.

Both env.json files specify four logical CPUs and two cores; machine.cores=4 must not be read as four physical cores. The sibling maps differ. These are two-core SMT observations, not the prior i9 four-core setting. They do not isolate a topology interaction: a significant 4t contrast and nonsignificant 1t contrast do not themselves establish a significant difference between contrasts.

The control caveats are valuable, but “no row left unresolved” means only no *flagged harm awaiting the rule's repeat*, not all controls proved safe. Most wall intervals cross 1. Keep that distinction explicit. The body lists selected large upper bounds, not every bound; the attached audit lists all 42 wall intervals. The old half-width sentence still needs replacement with upper-endpoint language from my prior review.

The original benefit screen passes through RSS; the amended screen passes through run-1 wall with run-2 favorable point estimate. That arithmetic is supported. It does not repair the admitted literal amendment-cutoff violation or independently certify delegated permission to change the rule. Reporting both versions is transparent, not retroactive preregistration. I do not reopen the merge or issue a new gate. The actual wall result now supplies evidence absent from an RSS-only rationale, but scientific uncertainty and governance interpretation remain separate.

Source comparison confirms only examples/occupancy.rs changed between the two named heads. It is reasonable to describe the measured source as unchanged, not the commits as identical. Exact executable identity was not independently verified. Toolchains match (rustc 1.99.1), but hosts/images differ. Keep the runs separate.

## 2. RLE1: defensible uncertainty, overstrong exclusion

The reported phase RLE1 ratios match the archives: 1.0261 [1.0204,1.0318] and 1.0392 [1.0306,1.0479]. These are repeated detected shifts in this diagnostic, not simply “noise.” The timed pass builds ll and tt, walks into a separate walked buffer, and calls rle::unrle(&walked,…). It does not time the production in-place path. Therefore these rows neither measure in-place RLE1 nor establish that its production RLE1 became slower.

However, **“the harness never calls invert_in_place” is too strong**. main checks decoded == mibz2::decode(&data).unwrap() after its first timed pass. That production call reaches the integration path for supported non-cooperative Segmented decoding. It is outside the recorded stage timers, but occurs before later passes; cache, allocator and thermal history are possible indirect influences. This corrects my own prior broad shorthand as well as the write-up. No such influence is established by these data.

Other open explanations include generated code/layout differences under LTO, data placement and run/order effects. “Unexplained between-build sensitivity in a separate-buffer phase diagnostic” is defensible. “Not evidence of a production RLE1 regression” is defensible. “Cannot be caused indirectly by the change” is not.

The phase numbers are Instant elapsed times scaled by a nominal clock, with per-stage fastest-pass selection. Their sum is not a measured whole-decoder runtime. The median RLE1 differences are only about .0025 and .0027 ns/output byte (roughly .065% and .090% of the respective base stage totals); these descriptive scales must not be subtracted from the decoder speedup or used to reconstruct the paired total. In particular the run-1 +1.25% paired stage total is not explained by the RLE1 row alone.

## 3. Next measurement: diagnose the discrepancy, not repeat until significant

No third whole-decoder rerun merely to obtain a second significant wall result. The registered run budget did not trigger it; changing the stopping rule after observing these outcomes would create a different experiment. For the current engineering claim, preserve the outcome and caveats.

The cheapest useful next step is a **non-timing audit of the exact phase executables** if retained: compare RLE1 and caller instruction bodies, alignment/addresses, call paths and compiler flags across arms. If binaries were not retained, a rebuilt comparison is a new build, not proof of the original binary identity. This separates possible code-generation changes from an unchanged-code timing anomaly without pretending to identify cache mediation.

If that audit leaves the issue material, propose a separately recorded diagnostic comparing first versus subsequent phase passes and the intervening production parity call's state effects, with fixed order controls and byte validation preserved outside the timing treatment. This is a proposed experiment, not authorization to alter the benchmark, weaken parity, or run it here. The original phase-round samples and within-pass timings are preferable to only fastest-of-two aggregates.

A mechanism-specific reuse-versus-allocation contrast is a different, larger question; the two production arms cannot answer it. The closed equal-allocation diagnostic remains prior evidence, not permission to label this bundle's gain “reuse.” Choose that new experiment only if the decision needs mediation, not merely because the present wall interval crosses 1 in run 2.

## Evidence limits

The archived directories contain comment.md, ab-summary.md, env.json and result.json. At this pin they do **not** contain runs.jsonl or stages.jsonl. result.json contains phase round aggregates, but not the whole-decoder paired sample series needed to independently recompute intervals/order sensitivity. Thus “raw reports” is fair as archived output, but “raw paired observations archived” would be false. Preserve original Actions artifacts before expiry if available; no new measurement is needed for that.

This assessment checks published ratios, row coverage, source control flow and source diff, not independent timing, CI/Miri certification, all raw Callgrind profiles or final merge identity. Existing Researcher design advice is disclosed; this is not a blind review.
