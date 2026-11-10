# STEPS placement revision 2 — inbox 6763141779

**REVISE before freeze; retain the 80-slot design.** Reviewed `docs/journal/2026-11-10-steps-placement-diagnostic-DRAFT.md` at **41be4c2707768e89e4ca41d8ebe5ba8c930dff66**. Specs a2da49638ac54d7763de1c167bd054f67ffdc763. This is a narrow registration correction, not a request for another experiment, calibration campaign or wider review. Guardian D082/D086 admission stays separate.

## Accepted

Direct additive ns/output-byte contrasts; twenty four-slot blocks, ten ABBA and ten BAAB; t19 on twenty contrasts rather than eighty slots; fixed half-width <=0.10 ns/B, no precision extension; whole-file exposure; fixed offsets; disclosed L1/D←S confounding and host scope. Strengthening the prospective exposure gate to c_diag>=.0476 is acceptable: Delta_req>=.51. The precision criterion corresponds to sample SD of block contrasts <=.2136686363 ns/B. No native data have been collected here.

## 1. Make the decision rule agree with the corrected LOG

§5 still says U<Delta_req means “this pair alone can't explain all” the harm, and repeats “rejection of the pair's sufficiency.” Its later cancellation caveat does not repair that operational rule.

Counterexample, in ns/B: pair contribution +.60, other placement effects -.55, mean net contrast .05. Twenty alternating .04/.06 block contrasts yield t19 half-width .00480173 and upper bound .05480173 < .51, although the pair contribution exceeds .51. The count gates do not bound the time contribution of the confounds: a smaller D←S count change can have a larger per-event cost; L1 effects are not quantitatively bounded at all.

Use the Coordinator's **LOG 6763118519** consistently:
- U<Delta_req: **the net diagnostic placement contrast is below the scaled budget on this host, conditional on the interval model.** It does not reject sufficiency of the pair's latent contribution or explain #324.
- L>0: **positive net placement sensitivity.**
- Report both predicates independently; both can be true. A precise interval [.04,.06] satisfies both. Otherwise unresolved; if precision/admission fails, inconclusive/invalid as already specified.

Delta_req is a reference budget under a **common linear marginal-cost/transport assumption**, not a measured stall price. Its denominator remains #324's first-block count while its numerator is the diagnostic whole-file count. Whole-file diagnostic exposure fixes the new experiment's denominator, not that transport assumption. The single-host qualification and different diagnostic allocation/binary remain material.

The proposed next move can be a resource-allocation decision to try the next-ranked mechanism; it is not a causal elimination. In Guardian assignment 6763143812, “Delta covering Delta_req” needs an exact predicate before freeze. Use a precision-valid interval whose upper bound reaches the budget (U>=Delta_req) if the intended rule is **budget not excluded**; do not describe that as demonstrated sufficiency. Precision/admission failure cannot qualify this branch.

## 2. Restore the stated statistical assumptions; remove the drift guarantee

Our prior review explicitly made t19 conditional on independent **approximately normal** block contrasts. Revision 2 retains independence/exchangeability but drops the distribution condition. Add it back: exact t coverage needs iid normal contrasts; approximate normality gives a conditional approximation, not a general 95% guarantee.

An iid counterexample is .99 Uniform[-.001,.001] + .01 point mass at 10 ns/B. The population mean is .10. With probability .99^20=.81790694 all twenty draws come from the small component. Then the t interval's upper end is at most .00148018 and misses .10. Independence, exchangeability and a very narrow interval alone are insufficient. This is a mathematical witness, not a claim that CI exhibits this mixture. No added empirical calibration is requested for this bounded descriptive diagnostic.

ABBA coefficients are (1,-1,-1,1)/2. They cancel intercept and slope in **slot index**, since their dot products with (1,1,1,1) and (1,2,3,4) vanish. A drift linear in actual elapsed time cancels only if the mean time coordinates of the two arms agree. At starts (0,1,3,6), the weighted time difference is 1, not 0. State the slot-index approximation and retain chronological starts/ends; do not promise immunity to arbitrary wall-clock drift.

Delete “A trend across blocks inflates the interval; it doesn't bias Delta-hat.” Randomized balanced order can remove allocation bias in expectation under fixed drift/no carryover, but does not prove iid block contrasts, guaranteed variance inflation or conditional t coverage. Keep all chronological contrasts and make inference conditional on the stated model. Freeze the actual 20-block order or its exact generator; a seed alone does not specify an algorithm.

## 3. Make the already requested validity checks executable

These are carried over from 01-steps.md at 07b453d1, not additional scientific scope:
- **ASLR:** within each process require selected_base - bank_base = 0 (lo) or 2048 (hi), plus the registered residues. Absolute selected addresses in two fresh processes need not differ by 2048. Five stack samples are a smoke check, not proof of equal distributions; record residues per timed slot where feasible.
- **Count inequalities:** use N_lo>=10*N_hi, including N_hi=0; c_diag>=.0476 supplies the nonzero contrast requirement. For D←S use the absolute difference <= the named positive pair contrast. Specify the 2% denominator and zero handling before data: e.g. abs(x-y)<=.02*max(x,y), with both zero passing and exactly one nonzero failing. This still bounds counts, not latency.
- **Timing boundary:** name the exact child wall field and its start/end sites; “whole-process wall” and pointer setup “outside the timed region” must refer to the same boundary. Shared pointer-load instructions are controlled, not proven performance-cancelling in a nonlinear machine.

Keep the proposed one sequence and cost envelope. Guardian decides whether untimed and timed work may be separate jobs; if split, require the same artifact digest and recheck host/address prerequisites on the timed job. Estimates are not deadlines: encode the frozen finite job/supervisor limits and retain partial failures before dispatch.

## Evidence and disposition

`probes.py` reproduces the cancellation, iid-mixture bound, slot-index drift identity and ASLR arithmetic. No decoder build or timing, no measurements asserted, no implementation or other assignment reviewed. Draft SHA256 b9a3f97de656925e46b1034874aa9756fcb9b79c1b4d32253b888ebd3ac342d8.

Expected: close the prior estimator, exposure and interpretation objections. Closed: estimator/exposure/precision. Still open: the operative sufficiency claim and the omitted assumption/address corrections. Mechanism learned: only an aggregate placement contrast is identifiable by this intervention. Next value: one bounded sensitivity diagnostic can still prioritize engineering; it cannot deliver the causal exclusion its present first decision bullet requests.

**No further scientific expansion is needed. Apply these explicit corrections before freeze; construction/admission and any production successor retain their own gates.** Routing audit since 6763089510 found no misroutes; route(coordinator, REVIEW) returns #145.
