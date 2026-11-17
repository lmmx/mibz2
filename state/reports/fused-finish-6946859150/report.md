# Fused finish: registration review for inbox 6946859150

**Verdict: targeted correction, then PROCEED with the existing executable experiment.** The affinity mismatch does not require changing the arms, thread grid, pinning, repeat count or statistical rule. Correct and commit the description before the workstation run. A pass is evidence for the measured configurations; it is not, by itself, certification of an all-host default.

Reviewed head: `2bcd82721d637c7e21326006b8475c84a57e9fd1` (#535). Baseline: `77fa4cd34e6ceab28d172e5dfed1ef3aafa7be03` (0.5.3). Instruction pin: `a2da49638ac54d7763de1c167bd054f67ffdc763`.

## 1. Re-derived plans, not inferred from the registration

The baseline and candidate's entire `src/parallel.rs` are identical. `BzReader::parallel` and `parallel_lazy_as` both supply four slots per requested thread. `new` and `new_lazy_as` apply the same topology predicate and window calculation; the latter substitutes the inverse kernel only where production selects lazy workers.

`ab run --spread` restricts a ten-thread child to one logical CPU on each physical core. `target_topology` counts the present, online CPUs in that child's `Cpus_allowed_list`. Ten allowed logical CPUs fail `trims_to_seven`'s requirement of twenty. At twelve requested threads, spread returns no restriction; the full i9 profile permits the trim.

| Judged cell | Child affinity | Reference | Candidate |
|---|---|---|---|
| 10 requested threads | one CPU per core, ten CPUs | lazy 10 workers / 40 slots | fused lazy 10 / 40 |
| 12 requested threads | full 20-CPU affinity | lazy 7 / 28 | fused lazy 7 / 28 |

The counter runs at ten threads also use spread pinning: they measure 10/40. The explicit ten-worker context pair measures the same worker/window configurations as the judged ten-thread pair. It remains context; it is neither an A/A check nor an additional gain cell. Do not pool it, substitute its result, or use it as a retry.

**Why correction suffices:** the registered operational comparison was production at the fixed baseline against that same plan with the finish fused. That comparison survives. Each paired ratio conditions on one configuration, so the change from ten workers to seven across cells does not confound either within-cell contrast. It does prevent interpreting the difference between cells as a pure worker-count, affinity or SMT effect.

Forcing seven workers at ten threads would instead change the registered experiment. Removing pinning would change it too. Neither is necessary to answer the current question. Future topology-predicate fixes belong to their own comparison.

## 2. What remains frozen

Retain ten paired rounds, Wikidata ×4 and text, judged counts 10 and 12, no controls, no spinner; candidate/reference paired log-wall ratios; two-sided t intervals; Holm one-sided gain tests over the two Wikidata cells at family alpha .05; no judged SLOWER cell; and a 1.10 RSS gate at every judged cell.

The memory statistic in the executable reader is **median(candidate peak RSS) / median(reference peak RSS)**, not median of per-round RSS ratios. State that explicitly.

With two gain cells, the first Holm threshold is .025. A significant gain at only ten threads can satisfy the gain rule while twelve is inconclusive. My synthetic full-roster replay demonstrates precisely this: 80 fabricated records, gain only at ten, ADOPTS. That is the intended existential gain rule, not evidence of a gain at both counts. The replay with invalid ten-thread pinning is VOID. Existing adjudicator checks pass **52/52** after fetching the missing fixture; the initial partial local run failed only because that fixture had not yet been materialised.

No-SLOWER is a loss screen, not equivalence or noninferiority. The registration already admits that limitation. Four copies of one sample improve run duration; they are not four independent workload samples. Ten rounds quantify repeat variability on these inputs, not across arbitrary dumps.

There is no new inferential method to approve here. Do not add a post-data minimum, extra rounds to obtain a pass, another gain family, or a mechanism gate. At an observed ratio .985 and ten rounds, the first Holm rejection needs log-ratio standard deviation below approximately .0211. This is a resolution calculation, not a power estimate or permission to extend the run.

## 3. Correct the mechanistic claim without blocking the wall experiment

Prior art first: #183 already tried post-walk fusion; its committed journal records failure of its own four-thread benefit/attribution criteria. Keep that verdict. Cells supplies contiguous output, so this construction removes no segment-assembly copy. It changes the locality of CRC reads after RLE1; it does not eliminate the logical CRC reads themselves. The scalar RLE path still computes CRC after RLE1; the incremental fold operates on the AVX2 path.

The reported simulated CRC L2 read reduction is .0157 to .0003 misses/output byte, about .0154; process reads .3077 to .2920 differ by .0157, about 5.1%. That is a credible locality hypothesis. It is not a measured i9 whole-decoder speedup. The additional checks/fold setup and CRC dependency are costs; +.33% simulated instructions neither qualify nor disqualify it.

Replace “if aggregate L2 misses do not fall, the CRC re-read was not missing L2, and the mechanism is refuted” with:

> Failure to resolve the registered 3–6% net reduction rejects or leaves unsupported that aggregate prediction. It does not alone show that CRC reads were already L2 hits: offsetting traffic, prefetch/event semantics, changed overlap and noise can hide a phase-local reduction. A net reduction likewise does not isolate CRC causally.

The counters are diagnostic and decide nothing under the frozen wall rule. This correction must not become a new acceptance condition. Nor may a counter failure erase a wall gain.

The proposed .0154 × 65 / 16.85 is about 5.94%, but only if those reads each add the whole latency serially. The stated .985 wall expectation is a hypothesis about the exposed fraction after overlap and new work, not derived by multiplying misses and latency. Moreover, the 65-cycle reference came from 7/28; it is not the ten-thread 10/40 cell's reference. Relabel the expectation by actual configuration rather than silently carrying that number across. No new measurement is required before the registered test.

## 4. Experiment qualification versus global adoption

The phrase “wherever lazy workers run, on every host” is an adoption-policy scope, not a consequence of holding the plan fixed. Cache size, AVX2/CLMUL availability, expansion distribution and the fraction of time in the finish can change the payoff. A host with the output resident in private cache can pay the fold overhead without saving the intended cache traffic. That is a counterexample to the inference of universal benefit, not a demand to test every host.

The existing PERFORMANCE-POLICY distinguishes a general production change, judged on its CI paired bench, from a hardware-gated change whose gain is judged on its target and whose other routes stay inactive. This candidate is research-only: a green bench on unchanged production defaults does not measure the proposed fused default on CI.

**Near-term route:** run this bounded i9 experiment after the descriptive correction. If it passes, bring the actual default-changing head through the standing CI performance policy and B5/Guardian obligations. Alternatively, an explicit forward C6 adoption-policy decision must identify any different admission rule and obtain the required Guardian review. Do not treat the i9 reader's string ADOPTS as already satisfying either route. This does not impose exhaustive cross-host evidence, nor require a causal explanation before integration.

The frozen journal correction should therefore name the integration decision accurately: establish i9 evidence for the fused lazy finish; a global default remains subject to the existing default-head CI/safety/policy route. Preserve the original numeric verdict regardless of the later integration decision.

## 5. Evidence and limits

- [Assignment 6946859150](https://github.com/lmmx/mibz2/pull/162#issuecomment-6946859150).
- [Registration 6946436207](https://github.com/lmmx/mibz2/pull/145#issuecomment-6946436207).
- [Affinity finding 6946756938](https://github.com/lmmx/mibz2/pull/145#issuecomment-6946756938). Its timings were not independently replayed here; the predicate result was independently established from source.
- [Plan dispatch and predicate](https://github.com/lmmx/mibz2/blob/2bcd82721d637c7e21326006b8475c84a57e9fd1/src/parallel.rs): new/new_lazy_as, window, trims_to_seven, target_topology.
- [Harness](https://github.com/lmmx/mibz2/blob/2bcd82721d637c7e21326006b8475c84a57e9fd1/examples/ab.rs), [recipe](https://github.com/lmmx/mibz2/blob/2bcd82721d637c7e21326006b8475c84a57e9fd1/.just/bench.just), [adjudicator](https://github.com/lmmx/mibz2/blob/2bcd82721d637c7e21326006b8475c84a57e9fd1/bench/two_arm.py), [counters](https://github.com/lmmx/mibz2/blob/2bcd82721d637c7e21326006b8475c84a57e9fd1/bench/counters.py).
- [Prior fusion registration and recorded result](https://github.com/lmmx/mibz2/blob/2bcd82721d637c7e21326006b8475c84a57e9fd1/docs/journal/2026-11-05-fused-postwalk.md).
- [Performance policy](https://github.com/lmmx/mibz2/blob/2bcd82721d637c7e21326006b8475c84a57e9fd1/docs/team/PERFORMANCE-POLICY.md).

The fetched PR head equals the reviewed SHA. All completed check runs report success, with deployment/release jobs skipped; pre-commit.ci status is success. These are checked statuses, not a new adversarial safety certification. I inspected the changed code and its error/CRC path but did not independently build Rust, rerun Miri/fuzz/mutation, validate the cachegrind artifacts, or run workstation timings. New unsafe-path admission remains with the existing B5 review. No implementation, merge, policy mutation or workstation execution was performed.

Migration checks passed; inbox listing found this one unfinished assignment; routing audit since the previous assignment found no misroutes. Research receipt and this report live only on team-state/researcher.

