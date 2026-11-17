# Eager-trim C6 review — inbox 6947285652

**PROCEED for the measured pinned cells: eager 5/20 at six requested threads and eager 3/12 at four. Eager 5/20 prospectively replaces lazy 5/20 there. HOLD only the extension to unpinned or otherwise unmeasured affinity shapes until separately admitted.**

Reviewed registration/code: 8426934abb7dc196a23a38cd62b633f43e8fb193. Bundle: eager-trim/run-20261117T114640Z, evidence commit 49fa252801f9e58f383040338514a797d29f9ca9. Instruction pin: a2da49638ac54d7763de1c167bd054f67ffdc763.

## Reconstruction

I fetched the original submission [6947258085](https://github.com/lmmx/mibz2/pull/145#issuecomment-6947258085), downloaded its base64 archive through the GitHub connector, decoded it and verified SHA-256 **ad97c13714667d394ff74a0a79b3b2beea88737d8af100a0899a9c45e16bd84f**. Its commit.txt matches the reviewed code. The registration precedes the run; LOG 6946756938 corrected the ten-thread reference before the recorded 11:46:39 start.

The frozen adjudicator replay reproduces archived verdict.txt byte for byte: 128/128 records, all five Wikidata hypotheses rejected by Holm, and the registered choices. An independent calculation in independent.py reconstructs the sparse roster, uniqueness, successful finite records, output size/hash agreement, masks, spinner state, paired log-wall geometric means, df=7 t intervals and ratios of median peak RSS. These are recorded-output checks, not an independent bytewise decode.

| Requested threads / candidate | Wikidata ratio [95% interval] | Text ratio [95% interval] | RSS Wikidata / text |
|---|---|---|---|
| 6 / eager 5/20 | .937852 [.927889, .947921] | .993856 [.977441, 1.010546] | .860041 / .881553 |
| 6 / lazy 5/20 | .973421 [.957600, .989503] | .992553 [.979664, 1.005613] | .867263 / .886653 |
| 4 / eager 3/12 | .959393 [.946328, .972638] | .967948 [.947825, .988498] | .761400 / .812598 |

All ratios are candidate over the concurrent production reference. At six threads that reference is eager 6/24; at four it is eager 4/16. Both eager candidates pass the frozen familywise gain, per-candidate loss screen and both memory gates. No noninferiority claim follows from the text interval at six threads.

## The replacement decision

At six, the rule chooses the adopting candidate with the lowest Wikidata geometric mean. Both candidates adopt and eager wins that predeclared selection. The descriptive eager/lazy pair (.9635 [.9525,.9746]) agrees, but it is not a newly invented selection gate or a separate multiplicity-adjusted superiority claim. The earlier lazy5 adoption was valid evidence against eager6. This result replaces the selected policy prospectively, without rewriting that result.

At four, earlier lazy3 failure was not a prohibition on eager3. This experiment holds the eager kernel and changes worker count plus its coupled window. It establishes a gain for that combined 3/12 plan, not an isolated worker-count causal effect. It closes about 4.1% of the measured Wikidata wall relative to eager4, with a text gain and lower peak RSS. At six the selected plan closes about 6.2% relative to eager6.

Keep the kernel explicit in any adoption table: eager5 and eager3, lazy6 at eight if independently admitted, lazy7 in its admitted cells. Do not merely change a universal LAZY_FROM_THREADS threshold from these data.

## Affinity: yes, four needs an unpinned check before the unpinned extension

The six- and four-thread records are restricted respectively to CPUs 0–5 and 0–3, one logical CPU per distinct physical core. They also have the registered spinner on CPU19, outside the decoder's cores. The worker, reader and scanner threads share the restricted decoder mask. Unpinning changes their scheduling opportunity, migration, core sharing and potentially clock/uncore conditions. Those are exactly quantities that can determine whether removing one worker helps.

A machine-wide hardware identity plus “at least T allowed logical CPUs” is **not** a proof of equivalence to this placement. Concrete counterexample on the same i9: an allowed mask {0,10,1,11} contains four logical CPUs but only two physical cores. Even a full twenty-CPU mask permits four requested decoder threads and auxiliaries to use more than four physical cores. Neither configuration is the measured 0–3 restriction. Hardware discovery fixes routing facts; it supplies no missing performance contrast.

Thus:
- The current evidence admits the pinned, one-CPU-per-core measured configurations through the usual exact-default-head CI and Guardian route.
- **Before default adoption on unrestricted affinity at four, register and run eager3/12 against eager4/16 there**, with the same workload guards and an explicit pre-data decision rule. It can be a small extension to an unstarted unpinned registration, or a separate registration if that run has already started. Do not add a cell to an already observed family or reinterpret the pinned verdict.
- At six, the unpinned candidate must be **eager5/20**. An unpinned test of lazy5 alone cannot qualify its replacement.
- Do not claim that any “allowed logical CPUs ≥ T” mask is admitted by a full-affinity run either. Restrict supported affinity classes or explicitly review the generalisation policy. No exhaustive host survey is requested.

There is a distinction between a forward, reviewed C6 policy permitting extrapolation and an empirical claim that unpinned performance was measured. The proposed predicate alone is neither. The team's existing hold on the unpinned extension is not resolved by renaming the hardware-detection scope.

**Source/status caveat:** the fetched #537 head was 9929df01bca3bff67906d9aa33507a5edd92731b. Its target_topology still starts from Cpus_allowed_list and measured_i9 still requires twenty logical CPUs in that view; its six-thread cell remains lazy. The machine-wide predicate described in the assignment is therefore treated here as a proposal, not as verified code at that fetched head. This is no approval of a later implementation or of the exact current head.

## Correct two interpretations in LOG 6947274621

1. The frozen eager3 prediction is **.99 [.95,1.03]**, not just .99. The observed .9594 point estimate is inside that band. The observed confidence interval excludes the exact .99 point, but that does not refute the registered predictive interval. Record: point prediction overestimated the ratio; outcome compatible with its stated band. The eager7/lazy7 prediction .98 [.95,1.02] does fail against 1.0906 [1.0821,1.0992]. Eager5's .9379 lies inside [.86,.95].
2. A universal eager/lazy crossover is not identified. Eager5 versus lazy5 and eager7 versus lazy7 are concurrent kernel comparisons in different requested-thread/occupancy conditions. The four-thread eager3 result compares against eager4; lazy3 was in the earlier run. It supports using the measured table, not a machine-independent kernel threshold.

Prior work matters here: the low sweep exposed the combined kernel/count question, and record49 supplied falsifiable eager alternatives. The new evidence resolves the six-thread choice and opens the four-thread eager cell; it does not rehabilitate failed lazy3 or the entire fitted model. The next experiment has direct integration value: resolving precisely the unpinned eager3/eager5 route rather than collecting another general sweep.

## Evidence and scope

Sources: [assignment](https://github.com/lmmx/mibz2/pull/162#issuecomment-6947285652), [registration](https://github.com/lmmx/mibz2/blob/8426934abb7dc196a23a38cd62b633f43e8fb193/docs/journal/2026-11-17-eager-trim.md), [frozen LOG](https://github.com/lmmx/mibz2/pull/145#issuecomment-6946664164), [pre-data correction](https://github.com/lmmx/mibz2/pull/145#issuecomment-6946756938), [result LOG](https://github.com/lmmx/mibz2/pull/145#issuecomment-6947274621), [bundle](https://github.com/lmmx/mibz2/blob/49fa252801f9e58f383040338514a797d29f9ca9/eager-trim/run-20261117T114640Z.tar.gz).

No new workstation timing, implementation or merge. No B5 certification or admission of arbitrary affinity masks. Migration check and pinned-helper routing audit passed. Reproduction: extract the linked bundle into data/, use the registration's exact adjudicator command, and run independent.py. The archived replay and independent numeric results accompany this report.

