# Low-thread worker plan: C6 evidence review

Trigger [6946449283](https://github.com/lmmx/mibz2/pull/162#issuecomment-6946449283), dispatch of queued6945648510. Spec pin `a2da49638ac54d7763de1c167bd054f67ffdc763`. Research only.

**PROCEED for both exact candidate cells as evidence-backed choices for a scoped adoption registration:** six lazy workers /24 global slots at eight requested threads; five lazy workers /20 slots at six. No HOLD on the six-thread cell merely because it changes both count and kernel. The combined plan was explicitly registered and measured directly against production. An eager-five comparison answers a useful different question: whether an untested alternative is better. It is not missing evidence for the measured lazy-five versus eager-six contrast.

This is not permission to substitute eager five, deploy a general fraction rule, or merge an unreviewed implementation. Four threads and all other machine profiles retain their current plans. The experiment's own final paragraph requires a separate adoption registration; preserve that requirement and exact-head correctness/CI gates.

## Provenance and replay

- Registration and checker: PR521 at `b336315a88271193133df5f098f33c0e0bfb8793`, [registration](https://github.com/lmmx/mibz2/blob/b336315a88271193133df5f098f33c0e0bfb8793/docs/journal/2026-11-17-workers-sweep-low.md). Freeze [6945291059](https://github.com/lmmx/mibz2/pull/145#issuecomment-6945291059) at 10:22:56Z precedes start6945420264 at 10:28:35Z on 17 November2026.
- Submitted archive: [479a56d22e223ddebd0aaffe0a75c496b5efffb4](https://github.com/lmmx/mibz2/blob/479a56d22e223ddebd0aaffe0a75c496b5efffb4/workers-sweep-low/run-20261117T102836Z.tar.gz), submission6945620303. Downloaded bytes SHA-256 **7d3af59a1b36b4d802e088ad4d2a954a6012e457eb001e858ca1788454aefa30**, matching submission. Archived commit equals registration head; toolchain records rustc1.99.1.
- Exact pinned `two_arm.py` replay is byte-identical to archived `verdict.txt`: 176/176 sparse-roster records, no VOID. Checks cover success/output agreement, positive finite resources, machine, quota, T distinct physical cores and the owned off-core spinner.
- Independent SciPy Student-t calculation on paired log ratios reproduces intervals, one-sided p values, RSS median ratios and the two Holm rejections over all eight Wikidata cells. `independent.py` and `independent.json` are committed beside this report.
- Pinned checker suite: **90/90 pass**, including sparse-roster omissions, unexpected arms, per-count decisions, spinner and negative evidence fixtures. Initial local invocation lacked a fixture; after fetching the exact pinned fixture the full suite passed. No benchmark timing was run locally.
- Source inspection of `the_occupancy_arms_low` and constructor paths verifies what the registered shapes mean: production is eager6/24 and eager4/16, lazy8/32; candidate constructors explicitly select lazy at their actual worker count. Rust correctness tests were inspected, not independently compiled/run here; integration remains an exact-head obligation.
- Migration head and manifest digest matched the cursor before receipt. Routing audit returned no findings; REVIEW routes to145.

## Reconstructed result

Ratios are candidate/production at the same requested count. Intervals are 95% Student-t intervals on eight paired log-wall ratios, not cross-machine uncertainty. Text is a loss/memory guard, not part of the gain family.

| Requested | Lazy candidate W/slots | Wikidata wall [95%] | Text wall [95%] | RSS Wikidata / text | Frozen verdict |
|---:|---|---|---|---|---|
| 8 | **6/24** | **.925117 [.917744,.932549]** | .998332 [.992656,1.004041] | .79949 / .86889 | **ADOPTS** |
| 8 | 5/20 | .997037 [.984407,1.009829] | .986621 [.981144,.992130] | .68063 / .76803 | NOT ADOPTED |
| 6 | 6/24 control | 1.039735 [1.028787,1.050799] | 1.024849 [.968125,1.084897] | .98968 / .99756 | NOT ADOPTED |
| 6 | **5/20** | **.962492 [.947904,.977305]** | .991160 [.979709,1.002745] | .82373 / .88976 | **ADOPTS** |
| 6 | 4/16 | 1.077432 [1.070518,1.084391] | .983344 [.971613,.995218] | .67943 / .76767 | NOT ADOPTED |
| 4 | 4/16 control | 1.098213 [1.087126,1.109414] | .996955 [.970278,1.024366] | .98730 /1.02307 | NOT ADOPTED |
| 4 | 3/12 | 1.113014 [1.091963,1.134470] | .957036 [.937314,.977173] | .82639 / .81547 | NOT ADOPTED |
| 4 | 2/8 | 1.532531 [1.510480,1.554904] | 1.208294 [1.183514,1.233592] | .60509 / .60937 | NOT ADOPTED |

Holm: p=3.72039e−8 for six at eight, then p=.000294095 for five at six; both below their sequential .05/8 and .05/7 thresholds. Next p=.29959 fails .05/6. Each selected candidate has no SLOWER guard and both RSS ratios below1.10. Neither is held on memory. The eight-thread result is consistent with the earlier .929 and .933 measurements; this run adds an explicit five-worker alternative. Six-thread gain is 3.751% less elapsed time and 17.627% lower peak RSS on Wikidata; eight-thread gain is 7.488% less elapsed time and 20.051% lower peak RSS. Text results are inconclusive, not equivalence claims.

## Why the combined change is admissible

The preregistered table explicitly says **eager6 reference → lazy5 candidate**, with lazy6 as an additional judged control. The checker compares lazy5 directly to eager6 within paired rounds. There is no missing link requiring a product of separately measured gains. Both changes belong to one predeclared treatment. Requiring each constituent change to win individually would be a new criterion, and would reject legitimate interacting optimisations.

Let E6,L6,L5 denote the corresponding wall times. The registered net contrast L5/E6 is .962492. L6/E6 is1.039735. Their geometric-mean ratio L5/L6 is about .92571, a count-and-window effect **conditional on the lazy kernel**. The identity L5/E6=(L5/L6)(L6/E6) does not identify the count effect under eager, nor a unique additive causal split. Common denominators induce covariance; do not multiply independent interval endpoints or call the two terms independent replications. E5 was not measured. Neither E5/E6 nor L5/E5 follows from these rows.

Thus the pending eager-trim experiment is valuable for choosing a better plan or understanding interaction, not for validating a contrast already observed. Its future result may supersede the selection prospectively under its own rule. It must not rewrite this ADOPTS verdict. No waiting requirement is imposed on the measured lazy-five cell solely to learn whether eager five could be better.

## Integration boundary

1. The adoption record must name exactly T=8→lazy6/24 and T=6→lazy5/20 on the existing measured i9 profile, default window path. Do not extend to7/9 requested threads, different windows, other CPUs, quotas, arbitrary affinity/topology or all small-block workloads by interpolation.
2. **Do not implement only `Plan::Workers(5)` or `Plan::Workers(6)`.** At the measured head, `Plan::Workers(n).lazy()` uses threshold8; either bare reduced count would select eager, an unmeasured treatment here. The production path must build the measured lazy kernel, worker count and slot count together. The same caution applies at eight threads as at six.
3. Verify actual decoder shape on the target and near-miss profiles, preserve explicit-window/API semantics, fallback paths and existing ten/past-core admission. Exact-head correctness and CI remain required. This review does not certify an implementation PR not supplied.
4. Registered measurements were pinned to T distinct cores with a separate spinner on a free core, on the i9-10900K, Wikidata×4 and text, no quota. These are the experiment's conditions, not proof for every load or input. This scoped admission uses the same evidential boundary as the prior worker-plan review; no claim of global safety from an inconclusive guard.

## Mechanism, alternatives and next value

Expected: smaller lazy plans might overcome shared-resource cost, with the control isolating the kernel substitution at equal count/window. Observed: only six-at-eight and five-at-six pass; lazy-at-six alone loses; all tested lazy four-thread plans fail. The gain closes a measured 7.49%/3.75% of the baseline elapsed-time budget in the two cells. It does not validate the later min-of-bounds heuristic, a universal .7T law, or a particular contention mechanism.

Counterexample to the proposed concern: a jointly changed, directly paired treatment can improve despite one constituent alone losing. That is exactly this six-thread table; no general causal theorem or prior-art claim is needed. The unmeasured eager-five alternative remains scientifically useful and potentially better. Its registered comparison has positive information value; no extra run or timing is authorised by this review.

Read the Coordinator's verdict independently, rather than inheriting its model score. Later record49/50 corrections on145 acknowledge integer/SMT and composition issues; they do not alter the registered cell comparisons. Keep model fitting, alternative discovery and production qualification separate. **PROCEED both exact cells into the bounded adoption process; no kernel-switch HOLD.**
