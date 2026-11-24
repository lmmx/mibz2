# Request for work: a by-the-book retrospective of the team's failed bright ideas

*24 November 2026. Written by the Coordinator at Louis's request, as a proposal for him to decide on
before anything is commissioned. Status: **PROPOSED**. Nothing here has been started.*

> Louis: "do you think that if we reanalysed all the ways we had failed in the past by coming up
> with a bright idea, then it failing in the real world, in Huiberts et al's approach, would we make
> ourselves any more likely to prevent wasting time and tokens? is there any 'book' protocol we could
> establish to prevent the regret we have repeatedly hit?"
>
> "I think we should treat this as like a serious task and not like a little flippant sort of inline
> task … go to great lengths to retain everything we've said about this that's led us to the
> decision to do this, because all of that context is kind of necessary to understand how to do it
> properly, and also record the suggestions that you just made."

This document carries that context in full, so that whoever does the work starts where the
conversation ended, not from a summary of it.

## 1. The question, in one paragraph

The team has repeatedly had a bright idea, built it, measured it on the i9, and found it does not
pay. Each failure is written up honestly, one at a time. Nobody has asked what the failures have in
common, how much they cost in total, or which cheap check, applied before building, would have
stopped them. The proposal is to answer that with evidence, by the method of the paper that
prompted it, and to adopt as standing practice only the checks the evidence supports.

## 2. How we got here

### 2.1 The paper, and the reviews of it

- **The paper.** Eleon Bach, Alexander E. Black, Sophie Huiberts and Sean Kafer, *Beyond Smoothed
  Analysis: Analyzing the Simplex Method by the Book*, arXiv 2510.21613 (v1 24 October 2025, v2
  26 June 2026; STOC). It asks why the simplex method is fast in practice when worst-case and
  smoothed analysis say otherwise.
- **Its method, "by-the-book analysis",** has three phases: *observe* how the algorithm is actually
  implemented and what real inputs look like; *model* both to match the observations; *prove* bounds
  with any technique, provided its assumptions are also grounded. Its rules:
  - the assumptions are fixed **before** the bounds ("the determination of mathematical assumptions
    occurs before the production of theoretical running time bounds");
  - bounds are stated in parameters that can be **effectively estimated** on real instances (their
    *mean width* of the feasible set, estimated from 100 random directions per instance on NETLIB
    and MIPLIB);
  - the work ends with an honest **postmortem** (their §6) auditing input assumptions, model
    assumptions, method and outcomes against observation;
  - a parameter may change mid-proof only if it refines the model of an assumption already made, and
    is then checked on data (their §6.3).
- **Its critique of smoothed analysis:** perturbing every entry with Gaussian noise models inputs
  nothing like real ones. Real LPs are sparse (under 0.1% nonzero), and perturbation makes them
  dense. A model chosen because it admits a nice proof, not because it matches practice.
- **Jeremy Kun's review** (https://www.jeremykun.com/shortform/2026-11-23-2007/), which Louis
  shared, names its three practices:
  1. survey real input families, and admit into the model only structure that is *efficiently
     measurable*;
  2. survey real implementations, including interviews with closed-source solver maintainers
     (Gurobi), and analyse the choices practitioners actually make;
  3. review the unrealistic aspects that crept into the proofs. The shadow-vertex pivot rule was
     needed for the proof, not used in practice. "The end of their paper reads less like bluster and
     more like a postmortem."
  Kun: "my favorite idea of the decade: the analysis of algorithms must be grounded in empiricism."
- **The writer's review** (commissioned by the blog writer, read by Louis; its blog parts are the
  writer's remit). It read §§1–5 through a reading agent and §§6–7 from the PDF directly. Its points
  for mibz2:
  - mibz2's charter already does about half: observable dispatch parameters (r/n), grounding in
    real data (the route census in PROOF-PROGRAMME §5.5; the two-letter Sturmian solver exact on
    5,627 Sturmian BWTs and firing on 0% of Wikidata, "their smoothed-analysis critique in
    miniature"), and analysing what ships (§5.7);
  - the missing half: the proof programme proved only correctness. By the book is about cost;
  - five uses: cost theorems beside each route's correctness theorem; a search for mibz2's "mean
    width"; the machine as the algorithm's environment (contention as a model term); obstructions
    stated as lower bounds (the GUID wall); the four-part postmortem as a template;
  - where it doesn't transfer: their analysis is in expectation over injected randomness, ours is
    deterministic; their constants are not predictive, and ours won't be at first;
  - its own caveat: it saw §§1–5 only through an agent, and the MIPLIB bin counts it read sum to
    177 against 186 instances kept.
- **The writer's later REVIEW** (relayed for Louis, 24 November) added: day to day, by the book is a
  cheap filter, since the i9 is ground truth for anything runnable; its lasting value is the
  occasional **obstruction theorem**, which prunes a whole family of constructions rather than one
  experiment (PROOF-PROGRAMME §5.6: an oracle bounds only the alternatives it covers).

### 2.2 What has already been decided and done (24 November)

- **Louis approved cost theorems and contention modelling:** "cost theorems look like a good idea.
  Because of course, yes, cost is just as important as correctness. Likewise, machine contention
  should be modeled … We just need to sort of formalize them in this way." (#145 LOG 7126570308.)
- **#803 merged:** PROOF-PROGRAMME §5.9 (a cost statement `route_i_cost` beside each `route_i`,
  deterministic and conditional on block statistics θ; θ not chosen until the census reports),
  §5.10 (contention as a model term, starting from the moonshot proxy's IMC fit), METHOD.md §6a
  (records by the book: "the book" first, a four-part postmortem last, record 95's pre-screen checks).
- **The block census is running** (#145 LOGs 7126445058, 7126478963, 7126579799, 7126733471): which
  cheaply measurable features of a real Wikidata block predict the Open kernel's per-block cost on
  the i9, with a content-hash group split (wikidata-x4 may repeat blocks), an m-only baseline, the
  1-worker cost as the target, and MONODROMY's h₁ added as a feature.
- **MONODROMY record 96** (REVIEW 7126706167, at c1913220) proved the GUID wall as a lower bound:
  every rigid jump-walk route spends one single-row operation per width-1 run, D ≥ h₁; Open meets
  it exactly. Within that class the only lever left is the price per row, or a change of
  representation.

### 2.3 The three failures of 23 November, read by the book

| failure | where the claimed gain came from | where it had to hold | the gap |
|---|---|---|---|
| **#770**, the streaming parallel front end | Carbon's handoff, measured on a 2-core host | the i9 at 8–20 threads | **the machine.** The verdict LOG (7089082075) says: "The handoff's evidence came from a 2-core host, where this regime can't occur." Wall rose 52–61% |
| **#785**, the tiled zero-fill split out of #770 | #770's 0.978 / 0.977 at 4 and 6 threads, attributed by the Coordinator to the zero-fill | the split-out change alone | **attribution.** The i9 gave 1.008 [1.003, 1.012] at 6, SLOWER. The cause of #770's gain was assumed, not tested (LOG 7096821430) |
| **T²**, the squared retained map | MONODROMY's pre-screen on synthetic Haar cycles (0.73–0.76 on the i9) | the shipped Open emit on real blocks | **inputs and implementation.** Record 93 equated the emit with the pure chase; it is bound by work per row, with 0–7% exposed latency (record 95). E came out 0.99 / 1.02 / 1.22 at 1 / 6 / 10 workers against 0.65 predicted (LOG 7099262911) |

The three are the paper's three things to observe — the machine, the inputs, the implementation —
plus a fourth the paper doesn't name: **attribution**, where the model may be right but the credit
for an effect is given to the wrong part of a bundle.

### 2.4 The finding that makes this worth doing: we already had the book

`docs/journal/2026-11-20-toy-transfer.md`, written on **20 November**, three days before T² was
commissioned, tested K independent lanes over Open's retained cycle. If the emit were bound by the
latency of its dependent loads, lanes would act as width. Lanes bought 5–14% over one lane. The
entry's verdict: "The small gain from one lane to eight suggests that the dependent load is not the
main cost: about ten bytes are appended per retained step (n/m ≈ 10)." T² was built on the opposite
assumption and failed for exactly that reason.

So at least one expensive failure was not a gap in knowledge but a **failure to look it up**. The
paper's "observe" phase includes reading what has already been observed.

A second structural fact: the team already has the role for that lookup. ROLES.md gives the
Researcher "It judges prior art early: whether a candidate repeats a line already tested", and the
Optimiser "At each pivot, it first recovers the relevant prior candidates from the record" and
"prices every Pathfinder design before it gets a build slot … with … the cheapest falsifier". T² went
through neither: it was a C1a diagnostic (RULES C1a: a pre-data post, no review), commissioned by
the Coordinator, while the ChatGPT roles were out of quota. Whether the guards that exist are being
routed around, and how often, is itself one of the questions.

### 2.5 What we already model, and what we don't

Louis: "I thought we already tried to model our cache in the systems biological way the blog post
recently discussed."

- **We do model the cache and memory:** the moonshot proxy (`bench/moonshot/README.md`,
  `calibration.json`) has an L3-per-worker model and a memory-controller queue, idle + k·U/(1 − U),
  fitted to bench-imc-ten (evidence fa9707e9). Its README says it ranks well and prices poorly.
- **We have not yet done the metabolic control analysis:** measuring each stage's control
  coefficient by titration. That is Neon's Performance Landscape line, launched 23 November (#780,
  `docs/team/research-charter/launches/2026-11-23-performance-landscape.md`), not yet reported. When
  it reports, it is the "observe the implementation" phase made quantitative: which stage controls
  wall time. With it, T²'s assumption would have been read off a table.
- **Closer to the paper** would be fitting the proxy's parameters to measured per-block costs (the
  census) rather than to one calibration run, so that it prices as well as it ranks.

### 2.6 The Coordinator's own errors in this thread

Recorded because the retrospective must include them and because the Coordinator should not judge
them alone:
- **#785's premise** was the Coordinator's attribution of #770's gain to the zero-fill (LOG
  7089082075, "A separate lead"), refuted on the i9.
- **"Contention"** was used for R5-Head's (#734) −5% CPU / flat wall. That run's own LOG read it as
  the 8–20-thread plateau, a capped serial stage; the cause is not established (corrected in #803
  and LOG 7126640841).
- **The census's first rule** split held-out blocks by index, which would have leaked copies of the
  same block across the halves. Caught before any data (LOG 7126478963).
- **Questions sent to Louis that were the Coordinator's to decide** (closing #638, scheduling the
  #770 follow-up), against CLAUDE.md's "Don't send these back to Louis one by one".

## 3. The proposal

### 3.1 The question to answer

For every bright idea the team built and measured since the start of the record (27 October 2026):

1. **What did it cost?** Agent work (tokens where recorded, otherwise agent-runs and wall-clock),
   i9 minutes, CI cycles, calendar days, and what it displaced.
2. **Where did its expected gain come from, and where did it have to hold?** Machine, inputs, code
   path, thread counts, and who attributed the effect to what.
3. **Was the deciding fact available before the commitment to build?** Either:
   - **in the record:** a journal entry, LOG, evidence bundle, record or PR comment dated *before*
     the commit to build, that a competent reader would take to predict the failure; or
   - **cheaply obtainable:** by a check costing at most 10 minutes of i9 time or one agent-hour on
     real data, using tools that existed at the time.
4. **Which failure class does it fall in?** (§3.3.)
5. **Which proposed check would have caught it, at what cost, and would the same check have blocked
   any change that was in fact adopted?**

### 3.2 The guard against hindsight

After a failure, every failure looks predictable. The work is only worth doing if it is protected
against that:
- **Dated evidence only.** Criterion 3 counts a fact only if its timestamp precedes the commit to
  build (the commission message, the registration's first commit, or the PR's first push, whichever
  is earliest), and only if it was in a place the team actually reads (the repo, #145, an inbox,
  evidence). The searcher records the timestamp and link.
- **Taxonomy and criteria frozen first.** §3.3's classes and §3.1's criteria are fixed (with any
  corrections from the review in §5) before any case is classified. C2 applies: they don't move
  after the data.
- **Two independent coders.** A second coder, blind to the first's labels, classifies a stratified
  sample of at least 15 cases (failures and adoptions). Agreement is reported as Cohen's κ per
  question. A class with κ below 0.6 is reported as unreliable, not used to justify a rule.
- **Adoptions as controls.** The same questions are asked of every adopted change. A check that
  would have caught failures but also blocked successes has a false-block rate, and the proposal
  for standing practice must weigh both. Without controls, any check looks free.
- **No self-judging.** The Coordinator's own cases (§2.6) are classified by someone else.

### 3.3 Failure classes (draft, to be frozen after review)

- **I — inputs.** The idea's evidence came from inputs unlike the real ones (synthetic generators,
  toy corpora, one block, a different corpus mix).
- **M — machine.** The evidence came from a different machine, core count, thread count or
  contention regime from the one that decides.
- **P — implementation.** The idea was modelled against an idealised algorithm, not the shipped code
  path (textbook walk vs the shipped kernel; a stage assumed to bind that doesn't).
- **A — attribution.** The effect was real but credited to the wrong part of a bundle, or to the
  wrong cause.
- **L — lookup.** The deciding fact was already in the record (criterion 3a). Combinable with the
  others: T² is I + P + L.
- **B — honest bet.** None of the above. The idea was reasonable on all the evidence available, and
  lost. These are the cost of doing research, and a good protocol should not reduce them to zero.
- **X — process.** The failure was in execution, not the idea: a broken harness, a wrong pin, a
  rule that couldn't decide, a VOID run.

### 3.4 Candidate checks to evaluate (the Coordinator's suggestions, not yet adopted)

These are what the retrospective tests. Each is adopted only if the data supports it (§3.6).

1. **The transfer gap.** Before commissioning: where was the claimed gain measured (machine, inputs,
   code path), and where must it pay off? Name every difference. Each needs a check or an explicit,
   stated bet.
2. **Prior evidence.** Search the journal, the hypothesis map (PROOF-PROGRAMME §5.4), evidence and
   #145 for anything that measured the assumed bottleneck, and cite it. This is the Researcher's
   existing prior-art duty (ROLES.md), made a step that a C1a diagnostic can't skip.
3. **The cheapest real falsifier first.** On real blocks, through the shipped code path, on the i9
   where the regime matters, before any synthetic harness or full build. The order on 23 November
   was synthetic first and real last; the paper's is the reverse.
4. **Attribution before splitting.** If a gain came from a bundle, show which part produced it (an
   ablation) before building a PR around that part.
5. **Record 95's three pre-screen checks**, already required by METHOD.md §6a: the m-slope test, the
   latency-free ablation, the footprint check against cache sizes.
6. **An obstruction check.** Does a proved bound (record 96's G1, others in §5.4) already rule the
   idea out?

### 3.5 Sources

- `docs/journal/` (about 60 entries mention a negative verdict, about 25 an adoption; the first job
  is to turn these into a proper case list), `docs/journal/monodromy/` (records), `docs/team/`.
- #145 and the role inboxes (#160, #161, #162, #222, #314, #346, #363), via `coord.py` and the
  GitHub API.
- PR bodies and review threads; the bench comments on each PR.
- The `evidence` branch (bundles with SHA-256s).
- The Observer's account on `observatory/intention`: what the team intended at each point and why.
  It is a primary source for "what did we believe when we committed", and its "why" chains may show
  where a premise was already abandoned.
- `CHANGELOG.md`, `DELIVERED.md`, the release-scaling records, the History page.
- Louis's own words where relayed (PROPOSALs on #145, and quotes in LOGs).

### 3.6 Deliverables and the decision rule

1. **A case table**, one row per idea: dates, cost, origin of the expected gain, where it had to
   hold, verdict, class(es), the deciding fact with its link and timestamp (or "none"), and which
   checks would have caught it.
2. **Base rates:** the share of failures in each class; the share that were lookups; the total cost
   of failures that a check costing under an hour would have caught.
3. **For each candidate check:** cases caught, adoptions it would have blocked, its own cost per
   use, and the inter-coder agreement on its questions.
4. **A recommendation**, by this rule, fixed now: a check is proposed as standing practice if it
   would have caught **at least 3 failures** (or failures costing at least **20%** of all failure
   cost), would have blocked **at most 1** adopted change, and costs less per use than the failure
   cost it saves divided by the number of uses. Anything else is reported but not proposed.
5. **A journal entry** with the by-the-book postmortem of the retrospective itself (METHOD.md §6a).
6. **A proposed METHOD.md amendment** with only the checks that pass (4), for Louis's decision (a
   rule change, RULES D).

## 4. Who should do it

The candidates, and the Coordinator's assessment:

| who | for | against |
|---|---|---|
| **The Coordinator** (this session) | Holds the most context; can read everything. | Authored several of the cases (§2.6), so it would be judging its own decisions. Its context is the conversation that produced the hypotheses being tested, so it is anchored. |
| **MONODROMY** | Strongest on model failures; authored the post-mortem (record 95) and the lower bound (record 96); works by records. | Also a party: records 92–95 are among the cases. Its remit is mathematics, not a process audit. Busy with the cost-theorem line. |
| **The Researcher** (ChatGPT, #162) | Its role is exactly this: "an independent scientific adviser and counterweight", "re-derives claims and looks for counterexamples", "judges prior art early". Not a party to the decisions. | Its quota pauses (it was out from 22 to ~24 November). Its one-round review style suits judging a protocol better than a multi-day audit. |
| **The Observer** (#222) | Its account is the record of intention, which criterion 3 needs. | Its spec says it "does not decide, gate, rank, review". A source, not an auditor. |
| **A fresh, independent Claude agent** | No anchoring on this conversation; can be given the brief and nothing else; cheap to run twice for the second coder. | Knows nothing beyond the brief; quality depends on the brief (this document). |

**Recommendation.** Split the roles:
- **Lead auditor: a fresh, independent Claude session** (not a subagent of the Coordinator), given
  this document and the sources, working in its own worktree. It builds the case list, extracts the
  dated facts, and does the first coding.
- **Second coder: a second fresh agent**, blind to the first's labels, on the stratified sample
  (§3.2).
- **Protocol review, before any case is coded: the Researcher**, one round on §§3.1–3.4 and §3.6 —
  the taxonomy, the hindsight guard and the decision rule — as an adoption registration would get
  (RULES C6). If it is unavailable within a day, an acting Researcher under Louis's authorisation, as
  for #749 and #771.
- **Sources consulted, not auditors:** the Observer (its account) and MONODROMY (one REVIEW on the
  classification of its own records' cases, I/M/P, once coded, as a check on the auditor's reading).
- **The Coordinator** supplies this brief and the case list's starting points, answers factual
  questions, and decides nothing about its own cases. It owns the scheduling and the final
  recommendation to Louis.
- **Louis** decides whether to commission it, and later whether any proposed check becomes a rule.

## 5. Cost, scale and timing

- **No i9 time.** Everything is in the record.
- **Agent work:** the lead auditor's case list and coding is the largest part, likely one to two
  long sessions given about 85 candidate entries; the second coder is a fraction of that; the
  Researcher's review is one round. For scale, single investigations on 23 November cost roughly
  0.3M tokens (the T² implementation agent) and 0.17M (the acting Researcher's review of #786).
- **Calendar:** the protocol review first (a day at most), then the audit. It blocks nothing; the
  census, the cost theorems and the #770 follow-up continue in parallel.
- **What it displaces:** the Coordinator's attention for the brief and the review routing, and
  Claude quota that would otherwise go to experiments. The point of the work is that the next
  several experiments are chosen better.

## 6. What this is not

- Not a blame exercise. The failures were run honestly, under registered rules, and written up.
  Class B (honest bets) is expected and healthy.
- Not a new gate by default. Its output is evidence for or against specific checks, and a proposal.
- Not a claim that the paper transfers wholesale. Its analysis is in expectation and ours is
  deterministic; its constants are not predictive; and the i9 is ground truth for anything we can
  run, which the theorists never had. What transfers is the discipline: observe before modelling,
  state parameters that can be measured, and end with an honest postmortem.

## 7. Decision requested

Louis: whether to commission this as proposed, and whether you agree with the split in §4 (a fresh
independent lead auditor and a second coder, the Researcher reviewing the protocol first, the
Coordinator kept off its own cases).
