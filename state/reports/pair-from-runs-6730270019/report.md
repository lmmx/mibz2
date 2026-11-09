# Pair-from-runs: REVISE before freeze

Assignment: #162 comment **6730270019**. Reviewed #272 at **280643cef83b09ac89707b747670e8198b21b68e**. Specs, MODE, RULES and PROTOCOL: **a4f97714cc14366984015869d9d094d89488ac57**. C1 assessment; no timing permission, B5 clearance or production qualification is implied.

**Disposition: REVISE.** Keep the proposed Q1/Q2 numerical thresholds. Include the admission rule in R, prospectively frozen with the complete algorithm. Repair the evidence predicate and the interpretation below before freezing. The construction is worth a bounded diagnostic; neither its instruction count nor the old consumer's loss proves it cannot win.

## 1. Construction and prior art

#34 is the relevant internal predecessor: the consumer and representation are recovered, not new. The contribution under review is construction by run intersections with ordered destination writes. I proposed the earlier construction in report `run-constructions-6729486092`; this is not an uninvolved review of that idea. I did not implement #272. Its correction from my F-coordinate low byte to **L[q]** is necessary for the recovered consumer and is accepted.

For an L run with source interval J_i=[p_i,p_i+ell_i) and F image I_i=[a_i,a_i+ell_i), psi maps q=a_i+x-p_i to x. If x also lies in I_j, then psi²(q)=p_j+x-a_j. Hence on their intersection:

`D[q+t] = L[q+t] | ((p_j+x-a_j+t) << 8)`.

The two tilings have at most 2r−1 nonempty intersections. Iterating in destination F order can stream D and L[q], paying the run-sized indexing/gather work instead of scattering D. This does not make L[q] cell-constant, nor eliminate the linear read and write traffic.

Independent `check.py` constructs psi by stable sorting positions, forms the intersections and checks every word and cell bound on **17,520** columns: ternary lengths 1–8, binary lengths 9–12. The journal's 17,511 is an arithmetic typo; the source test's displayed sum is 17,520. These are mathematical checks, not execution of the Rust fills, marker writes, walker, Miri or SIMD. The reported 431-block/Miri results remain attributed evidence; Guardian owns the separate reached-path B5 review. D044 on #269 establishes the sequencing principle, not a blanket approval or an automatically transferred case-specific HOLD.

## 2. Pilots and admission: keep the bar, disclose the adaptation

**No numerical tightening follows automatically from having seen 2–3 pilot rounds.** The journal says Q1/Q2 thresholds preceded the first timed run; retain that chronology and identify the versions used. Label those observations development data, excluded from registered intervals and verdicts. Candidate revisions after profiling are still adaptation, even when the decision thresholds were unchanged. A new CI run can test the final fixed candidate on its registered runner; it is not a pristine discovery on unseen corpus families.

**Include admission.** The scientifically useful candidate is the deployable hybrid: charge sampling, decision, fallback and whole entry cost. Keep Q1 over all segmented blocks without admission; it isolates the builder. Q2 compares admitted R with real production P. R/O is a hybrid-policy contrast because O lacks admission; do not call it an isolated builder effect. No extra timing arm is necessary for this diagnostic.

Freeze the literal sampling offsets/stride, short-input behavior, strict `>` comparison, 0.15 constant, fallback and per-corpus taken/declined accounting. State: *the need for admission was discovered after the records/random pilot losses; 0.15 was chosen using observed corpus counts to separate those families, before the confirmatory run*. That is corpus-informed selection even without timing a threshold sweep. `src/pair_runs.rs:908–912` currently says “Fixed before any timing”, contradicting journal §8; correct it. The journal's “without it ... could not pass on any CI result” is also too strong: the pilot establishes a serious observed problem, not a cross-machine impossibility.

**Sampling is not a bound.** The source samples comparisons at offsets 1–8 in each 128-byte group. An alternating 16,384-byte column with positions 0–8 of every group replaced by zero has sampled density **0**, true r/n **0.9375**. It passes admission. This is a column-level counterexample to a deterministic density guarantee, not a timed slowdown or a claim that this column occurs in the registered corpus. Archive the witness; call the rule a heuristic. Production robustness needs selector-aliasing/near-threshold cases, not a promise of universal protection from five synthetic means. Do not tune it after the registered result.

## 3. Material VALID repairs

**V1 — M1's registered difference is outside the predicate.** `examples/pair_runs.rs:489` checks the M1 *ratio*, not `m1d`. `GHZ=NaN` makes `diff` return None, while valid positive timing totals and ratio intervals can still yield VALID. `GHZ=0` yields a zero difference interval and MET. `diff` also fails to check its returned endpoints after computing the variance. Require finite positive clock metadata, every required difference/ratio endpoint finite, sensible ordered intervals, and required primary paired-block coverage. Model witnesses are in `checks.json`; no decoder was executed.

M1 currently multiplies wall seconds by one `/proc/cpuinfo` MHz sample. Those are **nominal clock-scaled wall units**, not measured core cycles: frequency variation and descheduling are not integrated. Report the raw seconds/row difference and label the conversion; either narrow MET to that explicitly nominal convention or use an appropriate cycle measurement for the archive's literal cycle question. M1 is exploratory, so this need not change Q1/Q2.

**V2 — available artifacts are mistaken for the planned set.** The workflow's Validity step loops over `sweep-*.txt`, then tests nonempty raw files and equal counts. Extracting and executing those exact loops gives no problems for (a) only the single-copy files, with all four planned concurrent copies absent, and (b) a header-only raw CSV for every copy. These fixtures exercise the evidence predicate, not the complete workflow or genuine benchmark output. A failed/interrupted Sweep can leave only an early subset; the subsequent `!cancelled()` validity step does not reconstruct the plan. Current per-existing-child nonzero/missing statuses are rejected, which is good, but absent children cannot be discovered from surviving files alone.

Before launch, persist expected copy IDs/CPU set and nproc. Require the exact output/status/raw set, each status zero, each process VALID, and no omitted planned copy. Capture single-copy failure status without allowing shell errexit to skip the write; retain failure artifacts. Parse raw CSV to require the exact corpus × round × six-arm matrix, finite positive required totals, intentional no-paired-block zeros only, no duplicate/missing rows, and reconstruct registered intervals/verdicts. Bind the artifact to the frozen full source identity and corpus digest rather than merely a 40-hex string. The workflow already records actual HEAD and checks the main corpus SHA; preserve those useful checks. Add untimed fixtures for missing child, empty/header-only/truncated raw, duplicate round, bad clock, missing M1 and failed child.

**V3 — invalid readings precede validation.** The harness prints Q1/M1/Q2 readings before examining `problems`; worst-synthetic calculation filters out missing intervals. A bad synthetic is later rejected, so this is not evidence of a false final VALID for that case. Nevertheless, suppress verdicts until the full predicate passes, or label each as invalid diagnostic arithmetic. Missing evidence must not leave a visible POSITIVE for readers to quote.

## 4. Fairness and inferential scope

The in-place comparison is substantially right: fresh column copy outside the bracket, actual P/R/O `invert_in_place` inside, RLE1/CRC outside and checked; persistent per-kernel state and rotated arm order. Q2 is the **inverse-BWT entry point**, not the whole decoder. Keep that limitation prominent: MTF, scheduling, decoding concurrency and downstream stages remain outside the estimand. Multiple simultaneous processes are contention diagnostics, not multithreaded decode.

Build tables are separate, but `Bufs::lf` is shared by row/run builders. Give it per-arm ownership or explicitly initialize identical capacity outside timing, and register first-growth/warm-state policy. It is probably a small transient effect; the present “each arm in its own buffers” claim is literally false. The shared untimed input-copy buffer in the whole-entry comparison is a different, disclosed choice and not itself a defect.

The paired log-ratio t intervals are appropriate conditional on approximately exchangeable round errors; rounds share one process/host and are not independent machine replications. Fifteen rotated rounds balance order; retain raw data. The synthetic ≤1.05 rule is a **point-estimate screen**, not an upper confidence bound on regression. Keep it if that is the diagnostic intention; correct “no regression above 5%” language. No automatic multiplicity penalty or post-pilot harsher threshold is required for these two separately specified diagnostic questions. Production adoption remains a separate registered comparison.

## 5. Count model and stopping: do not revive retired gates

The journal's claim that an extra 6.2 Ir/row needs IPC >6 to fit a one-cycle gap is false unless the extra work is strictly additive at an unchanged schedule/IPC. Correct accounting is:

`delta cycles = I_run / IPC_run - I_one / IPC_one`.

Counterexample: 12.4/4 − 6.2/2 = **0 cycles**, with both IPCs below 6. This is not a predicted speedup; it disproves the necessity claim. Reordering, instruction mix and memory overlap change effective IPC. C4 explicitly preserves that possibility. Replace “model: not met” with “counts alone do not decide”; M1 estimates the specified cost difference subject to its units.

Q1 STOP should close this registered candidate/configuration, not all run-built pair tables. Q2 STOP may pause this old-consumer candidate on expected-value grounds, but “reopens only with a different consumer” is unjustified: a materially cheaper builder, changed admission or a different target can also change total cost. Any continuation needs a distinct mechanism, registration and better marginal value than alternatives, not proof that the entire line is impossible. Conversely, Q2 POSITIVE would not contradict the older measurements: changed builder, production baseline, hardware, allocation, topology and admission mean it would challenge a transferred prediction. Investigate the mechanism before adoption without rewriting #34's verdict.

## 6. What advances

Expected: the new construction would preserve the recovered consumer while reducing builder work. Found: its corrected formula survives the finite oracle; no registered performance gap has yet been closed. Learned: ordered filling trades per-row scatter for run-sized indexing, but selector aliasing and the missing evidence checks constrain what the proposed diagnostic could establish. Next value: repair and freeze one bounded Q1/Q2 run after applicable B5 conditions are satisfied. Its result can separate inadequate construction savings from total-path failure; it cannot independently identify the binding consumer cost or qualify production.

No native benchmark, decoder test, Miri, fuzz, compilation, implementation, merge, dispatch, queue/tag or automation change was performed. Research only on `team-state/researcher`. `check.py`, `checks.json`, source snapshots and routing audit accompany this report. Audit: five previously resolved historical misroutes; no new unresolved misroute at the read snapshot. Channel's #271 item is QUEUED, not an inbox assignment and not taken here.
