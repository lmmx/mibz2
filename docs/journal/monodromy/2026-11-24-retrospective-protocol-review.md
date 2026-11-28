# 2026-11-24 · MONODROMY 97 · The retrospective's protocol, reviewed in the Researcher's place: PROCEED with six corrections. The main one: code whether each check would have fired blind to the outcome, so that "caught" and "falsely blocked" are computed rather than judged. Cases MONODROMY had a hand in are listed for exclusion

**The request.** The Coordinator's CHECKPOINT 7127116135, phase 1 of Louis's retrospective commission. It asks for one review of the brief's protocol, `docs/journal/2026-11-24-retrospective-proposal.md` at 722f487e (#804):
- §3.1, the questions;
- §3.2, the hindsight guard;
- §3.3, the taxonomy;
- §3.4, the checks;
- §3.6, the decision rule.

The answer is PROCEED, or one round of corrections. I read the brief in full.

**Verdict: PROCEED, with the six corrections below.** Each one closes a place where hindsight or too little evidence could enter. None changes the brief's aims.

## The corrections

**C1. Code "would it have fired" blind to the outcome** (§3.1 Q3a and Q5, §3.2).
- *The problem:* "a fact that a competent reader would take to predict the failure" is judged with the failure known. That is the hindsight the guard exists to stop. The extractor is also asked to record "any deciding fact", which selects facts by the outcome.
- *The fix, for the extractor:* for each case, record
  - the idea's own premises, quoted from its documents dated before the commit to build;
  - every dated document that measured the quantity or bottleneck a premise names, found by searching on the premise, not on the outcome. It names no "deciding fact".
  - The verdict goes in a sealed column.
- *The fix, for the coders,* who code without the verdict:
  - (a) whether a listed earlier fact contradicts a stated premise, quoting both;
  - (b) for each §3.4 check, whether it would have fired, by its trigger (C4).
- *Phase 5 computes the rest:* caught = fired and failed; falsely blocked = fired and adopted.
- *Mixing adoptions into the case list* (§3.2) is what makes blind coding possible. Coders can't tell a failure from a control without the sealed column.

**C2. "Cheaply obtainable" without counterfactual experiments** (§3.1 Q3b).
- *The problem:* any failure can be "obtainable by a cheap check" if the coder may invent the check.
- *The fix:* count only a check in §3.4's frozen list whose result is in the record, run at some date within the cost limit (10 i9 minutes, or one agent-hour). If it was never run, code it unknown, not caught.

**C3. Separate gaps from causes, and define B and the outcomes** (§3.3).
- **I, M and P as gaps:** code each as a gap present or absent, outcome-blind. Do the evidence's inputs, machine or code path differ from the ones that decide?
  - P includes **"the base moved":** the shipped path at decision differs from the one the evidence was measured on. #668 moving the knee under record 87 is an example.
- **Established cause:** coded only where a later dated measurement identifies the mechanism (record 95 for T², LOG 7096821430 for #785). Otherwise "cause unestablished".
- **A** requires an ablation or a split-out measurement as its witness.
- **B, defined positively:** every gap was named, and checked or stated as a bet, before the build. A residual without that is "unexamined gap", not B.
- **X** is reported separately, not in the ideas' base rates.
- **Outcomes,** coded first:
  - negative;
  - null;
  - **positive but not adopted** (the rule's power, or a gate: pinned 4's lazy 3/12 at 0.982 under Holm, record 91). This is not an idea failure;
  - adopted.
- **The case set:** an idea is a change, harness or bench arm built and measured on the deciding machine with a verdict. Adoptions from the same period are the controls.

**C4. An operational trigger for every check, and one added check** (§3.4).
- *The triggers,* each codable from dated facts:
  1. *Transfer gap:* fires if a gap of C3 is present and was neither checked nor named as a bet before the build.
  2. *Prior evidence:* fires if C1(a) holds.
  3. *Real falsifier first:* fires only if a measurement of at most 10 i9 minutes on real blocks through the shipped path exists in the record and showed the failure (C2). Otherwise unknown.
  4. *Attribution:* fires on a split-out of a bundle's gain with no ablation dated before it.
  5. *Record 95's checks:* fire if the m-slope, latency-free or footprint test, computable from data in hand, puts the claimed gain outside its interval.
  6. *Obstruction:* fires if a proved bound dated before the build covers the idea's class.
- **Check 7, added: routed through review.** Before the build, did the idea pass the Researcher's prior-art check or the Optimiser's pricing? This is coded as a variable, to test the §2.4 question of whether bypassing the existing guards predicts failure.

**C5. A decision rule that is robust to the coders and to overlap** (§3.6).
- *Counting:* count catches as the **smaller** of the two coders' counts, or adjudicated agreement, and false blocks as the **larger**.
- *Reliability:* a check's trigger needs κ ≥ 0.6, with at least 5 cases on which it fires. Otherwise it is "unreliable", whatever its counts.
- *The 20%-of-cost branch* also needs at least 2 failures caught. One large failure must not carry a rule.
- *Overlap:* propose checks greedily by marginal catches, after the ones already proposed, so overlapping checks aren't double-counted.
- *Blocked adoptions:* report the value of any blocked adoption beside the count (its measured gain).
- *Cost units:* agent-hours and i9 minutes. Tokens only where recorded.

**C6. Agreement per label** (§3.2).
- *How κ is computed:* per binary label, one for each class present or absent and each check fired or not, with that label's prevalence.
- *Rare labels:* a label positive in fewer than 5 cases gets "insufficient", not a κ.

## Cases I must not code (phase 3)

**Out:** every case whose idea a MONODROMY record authored, modelled, predicted or scored. That is records 92–96 and T², and also:
- the worker-count rule and its machine models (2026-11-16 to 17: `workers-*`, `worker-rule`, `contention-counters`);
- every recalibration and trim prediction and scoring (2026-11-18 to 22: `recalibration-*`, `sustained-*`, `imc-ten-scoring`, `knee-dram`);
- the acquisition tower and its registry edges (record 84);
- the origami lift (records 86 and 92).

**Verified but not authored** (Astra A–E, Open return, the endpoint sweep, the forest, bispecial, rank-gap, mixed steps, live reuse, owner schedules, Neon rounds 2–6, the DOE, the Neon teamwork audit):
- *My recommendation:* exclude these too. My verdicts on them were among the facts the team built on.
- *The decision is the Coordinator's.*

**The full list** is the 95 records in `docs/journal/monodromy/` on #497. The extractor can tag each case by whether any of them cites it.
